#!/bin/sh
# Each task that builds a binary of the workspace builds it in a target
# folder of its own (core.25), copied from boomdrivedev/flume's
# check-targets.sh at 99fb79ed. cargo writes `<folder>/debug/<binary>`
# again at each build, a build with nothing to do among them, and a gate
# runs its tasks at once in one checkout: a test that runs the binary
# while another task's cargo writes it finds no file there, or finds the
# binary another task built with other features.
#
# A task builds a package's binaries where its command or script, or a
# `.sh` file that its command or script names, runs `cargo build`,
# `cargo run`, `cargo test` or `cargo nextest run` with `-p <package>`,
# in a subshell too, and the package has a binary target. Its folder is its env's
# CARGO_TARGET_DIR, else the workspace's `target`; a relative
# CARGO_TARGET_DIR is read from the folder the task runs in. The projects are those
# .moon/workspace.yml lists, each read from its moon.yml. The first
# argument is the tree to read, the current folder by default.
set -eu

cd "${1:-.}"
root=$(pwd -P)

fail() {
  echo "check-targets: $*" >&2
  exit 1
}

# The packages that have a binary target, by cargo's own reading of the
# manifests.
packages=$(cargo metadata --no-deps --offline --format-version 1 |
  jq -r '.packages[] | select(any(.targets[]; .kind | index("bin"))) | .name') ||
  fail "cargo metadata could not read the workspace"
[ -n "$packages" ] || fail "the workspace has no package with a binary"

projects=$(awk '
  /^projects:/ { on = 1; next }
  on && /^[^ #]/ { on = 0 }
  on && /^  [A-Za-z0-9_-]+:/ {
    id = $1
    sub(/:$/, "", id)
    path = $2
    gsub(/["\047]/, "", path)
    print id "=" path
  }
' .moon/workspace.yml)
[ -n "$projects" ] || fail ".moon/workspace.yml lists no project"

# One line per task that builds: its folder, its package and its name.
builds=$(
  for entry in $projects; do
    id=${entry%%=*}
    project=${entry#*=}
    file="$project/moon.yml"
    [ -f "$file" ] || fail "$file is missing"
    # One line per task, fields split by a tab: its name, whether it runs
    # from the workspace root, its CARGO_TARGET_DIR or `-` where it names
    # none, and its command or script.
    awk '
      function value(line) {
        sub(/^[^:]*:[ ]*/, "", line)
        gsub(/^["\047]|["\047][ ]*$/, "", line)
        return line
      }
      function done() {
        if (name != "") printf "%s\t%s\t%s\t%s\n", name, fromroot, target, run
        name = ""
      }
      /^tasks:/ { on = 1; next }
      on && /^[^ #]/ { done(); on = 0 }
      !on { next }
      /^  [A-Za-z0-9_-]+:/ {
        done()
        name = $1
        sub(/:$/, "", name)
        fromroot = "no"; target = "-"; run = ""; inenv = 0
        next
      }
      /^    (command|script):/ { run = value($0); inenv = 0; next }
      /^    env:/ {
        inenv = 1
        line = $0
        if (match(line, /CARGO_TARGET_DIR:[ ]*["\047][^"\047]*["\047]/)) {
          target = substr(line, RSTART, RLENGTH)
          sub(/^CARGO_TARGET_DIR:[ ]*["\047]/, "", target)
          sub(/["\047]$/, "", target)
        }
        next
      }
      inenv && /^      CARGO_TARGET_DIR:/ { target = value($0); next }
      /^    [a-z]/ { inenv = 0 }
      /^    options:.*runFromWorkspaceRoot: *true/ { fromroot = "yes" }
      END { done() }
    ' "$file" | while IFS="$(printf '\t')" read -r name fromroot target run; do
      if [ "$fromroot" = yes ] || [ "$project" = . ]; then
        cwd=$root
      else
        cwd=$root/$project
      fi
      # The text that runs: the command or script, and each `.sh` file it
      # names.
      text=$run
      for word in $run; do
        case "$word" in
        *.sh) [ ! -f "$cwd/$word" ] || text="$text
$(cat "$cwd/$word")" ;;
        esac
      done
      # With no CARGO_TARGET_DIR, cargo builds in the workspace's `target`
      # from any folder of the workspace.
      folder=$target
      [ "$folder" != - ] || folder=$root/target
      folder=$(printf '%s' "$folder" | sed "s|^\\\$workspaceRoot|$root|")
      case "$folder" in
      /*) ;;
      *) folder=$cwd/$folder ;;
      esac
      folder=${folder%/}
      folder=${folder#"$root"/}
      for package in $packages; do
        if printf '%s\n' "$text" |
          grep -Eq "cargo +(build|run|test|nextest +run)( [^&|;]*)? -p +$package( |\$|[\"')])"; then
          printf '%s %s %s:%s\n' "$folder" "$package" "$id" "$name"
        fi
      done
    done
  done
) || exit 1
[ -n "$builds" ] || fail "no task builds a binary of the workspace"

shared=$(printf '%s\n' "$builds" | awk '
  { key = $1 " " $2; tasks[key] = tasks[key] " " $3; n[key]++ }
  END { for (key in n) if (n[key] > 1) print key ":" tasks[key] }
' | sort)
if [ -n "$shared" ]; then
  printf '%s\n' "$shared" | while read -r folder package tasks; do
    echo "check-targets: these tasks build ${package%:} in one target folder, $folder: $tasks" >&2
  done
  exit 1
fi
count=$(printf '%s\n' "$builds" | wc -l | tr -d ' ')
echo "check-targets: $count tasks build a binary, each in a target folder of its own"
