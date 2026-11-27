# Design

## 1. Which tasks build a binary

A task builds the binaries of a package where its command or script
runs `cargo build`, `cargo run`, `cargo test` or `cargo nextest run`
with `-p <package>`, and the package has a binary target. The check
also reads each `.sh` file that the command or script names, because
`plugin-herdr:host` builds the binary in `tests/host.sh`. That script
runs cargo in a subshell, `(cd "$tree" && cargo build -q -p timber)`.
Thus the check also finds `-p <package>` before a `)`. flume's check
did not find it, and flume gets the same text. `cargo metadata
--no-deps --offline` gives the packages that have a binary target. A
lint builds no binary: `cargo clippy` writes metadata only.

A `cargo test` of a library writes no binary at `target/debug/`. Thus
the tests of `seams`, `core` and `fakes` keep the workspace's `target`.
A folder of their own only adds builds.

## 2. The folder of a task

The folder is the task's `CARGO_TARGET_DIR`, which moon gives with
`$workspaceRoot` expanded. With no `CARGO_TARGET_DIR`, cargo builds in
the workspace's `target` from each folder of the workspace. cargo reads
a relative `CARGO_TARGET_DIR` from the folder that the task runs in.
Each new folder is under `target/`, which git ignores and `root:format`
does not read. `tests/host.sh` reads the folder from `cargo metadata`,
which obeys `CARGO_TARGET_DIR`.

Each by-hand task of `cli` gets a folder of its own, also where two of
them build the binary with the same features. A person can run two of
them at the same time, as `moon run ':#check-ecr' ':#check-kms'`, and
the build of one then writes the binary that the other runs.

## 3. The cost

Each new folder builds its crates again. In the gate, `cli:tools` and
`conformance:coverage` do this. A by-hand task does this at its first
run on a host. sluice and flume accepted the same cost for the same
race.

## 4. How the check reads moon.yml

No pinned tool reads YAML, because `devenv.nix` does not pin `yq`. Thus
`check-targets.sh` reads each `moon.yml` with awk. It reads `env` inline
or on the lines below it.
