#!/bin/sh
# shellcheck disable=SC2016 # each case's moon.yml names moon's own $workspaceRoot.
# The cases of check-targets.sh, copied from boomdrivedev/flume's
# check-targets-cases.sh at 99fb79ed, which root:targets-cases runs: each runs
# sh check-targets.sh over a scratch workspace of its own, a package
# `app` with a binary, a package `app-lib` with none, and the tasks its
# case gives, and reads its exit and its words. It prints each failure as
# one line, check-targets-cases: <case>: <what>, exits non-zero where any
# case fails, and says every case holds otherwise.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
scratch=$(mktemp -d "${SLUICE_TMP:-${TMPDIR:-/tmp}}/check-targets.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
cases=0
failed=0

# A scratch workspace whose root project's tasks are `$1` and whose
# project `part`, at part/, runs from its own folder with the tasks `$2`;
# check-targets.sh's exit and its words over it.
try() {
  cases=$((cases + 1))
  dir=$scratch/$cases
  mkdir -p "$dir/app/src" "$dir/app-lib/src" "$dir/part/tests"
  for file in rust-toolchain.toml .prototools; do
    [ ! -f "$here/$file" ] || cp "$here/$file" "$dir/$file"
  done
  cat >"$dir/Cargo.toml" <<'EOF'
[workspace]
members = ["app", "app-lib"]
resolver = "2"
EOF
  printf '[package]\nname = "app"\nversion = "0.1.0"\nedition = "2021"\n' >"$dir/app/Cargo.toml"
  echo 'fn main() {}' >"$dir/app/src/main.rs"
  printf '[package]\nname = "app-lib"\nversion = "0.1.0"\nedition = "2021"\n' >"$dir/app-lib/Cargo.toml"
  : >"$dir/app-lib/src/lib.rs"
  echo 'cargo build --quiet -p app --bin app' >"$dir/part/tests/build.sh"
  echo '(cd .. && cargo build -q -p app)' >"$dir/part/tests/sub.sh"
  mkdir "$dir/.moon"
  printf 'projects:\n  root: "."\n  part: "part"\n' >"$dir/.moon/workspace.yml"
  printf 'tasks:\n%s\n' "$1" >"$dir/moon.yml"
  printf 'tasks:\n%s\n' "$2" >"$dir/part/moon.yml"
  status=0
  out=$(sh "$here/check-targets.sh" "$dir" 2>&1 </dev/null) || status=$?
}

# A failure of the case `$1`, saying `$2`, where the test after them fails.
expect() {
  named=$1
  what=$2
  shift 2
  if ! "$@"; then
    echo "check-targets-cases: $named: $what"
    failed=1
  fi
}

# `$1` holds `$2`.
says() {
  case $1 in *"$2"*) ;; *) return 1 ;; esac
}

named="Two tasks build one binary in one folder"
try "  test:
    command: 'cargo test -p app'
  tools:
    script: 'cargo test -p app --features tools'" ""
expect "$named" "check-targets.sh exits $status: $out" [ "$status" -eq 1 ]
expect "$named" "it names neither task: $out" says "$out" "build app in one target folder, target: root:test root:tools"

named="Each task builds in a folder of its own"
try "  test:
    command: 'cargo test -p app'
  tools:
    command: 'cargo test -p app --features tools'
    env: {CARGO_TARGET_DIR: '\$workspaceRoot/target/tools'}
  live:
    command: 'cargo test -p app --features live'
    env:
      CARGO_TARGET_DIR: 'target/live'" ""
expect "$named" "check-targets.sh exits $status: $out" [ "$status" -eq 0 ]
expect "$named" "it counts no 3 tasks: $out" says "$out" "3 tasks build a binary"

named="A script that a task runs builds the binary"
try "  test:
    command: 'cargo test -p app'" "  test:
    script: 'bash tests/build.sh && echo built'"
expect "$named" "check-targets.sh exits $status: $out" [ "$status" -eq 1 ]
expect "$named" "it names neither task: $out" says "$out" "root:test part:test"

named="A subshell of a script builds the binary"
try "  test:
    command: 'cargo test -p app'" "  test:
    script: 'bash tests/sub.sh'"
expect "$named" "check-targets.sh exits $status: $out" [ "$status" -eq 1 ]
expect "$named" "it names neither task: $out" says "$out" "root:test part:test"

named="\$workspaceRoot/target is the workspace's own target"
try "  test:
    command: 'cargo test -p app'
  run:
    command: 'cargo run -q -p app'
    env: {CARGO_TARGET_DIR: '\$workspaceRoot/target'}" ""
expect "$named" "check-targets.sh exits $status: $out" [ "$status" -eq 1 ]

named="A relative folder is read from the folder the task runs in"
try "  test:
    command: 'cargo test -p app'" "  test:
    script: 'bash tests/build.sh'
    env: {CARGO_TARGET_DIR: 'target'}"
expect "$named" "check-targets.sh exits $status: $out" [ "$status" -eq 0 ]

named="A package with no binary"
try "  test:
    command: 'cargo test -p app'
  lib:
    command: 'cargo test -p app-lib'
  lib-again:
    command: 'cargo test -p app-lib --all-features'" ""
expect "$named" "check-targets.sh exits $status: $out" [ "$status" -eq 0 ]
expect "$named" "it counts more than 1 task: $out" says "$out" "1 tasks build a binary"

named="A lint builds no binary"
try "  test:
    command: 'cargo test -p app'
  lint:
    command: 'cargo clippy -p app --all-targets -- -D warnings'" ""
expect "$named" "check-targets.sh exits $status: $out" [ "$status" -eq 0 ]

[ "$failed" -eq 0 ] || exit 1
echo "check-targets-cases: $cases cases hold"
