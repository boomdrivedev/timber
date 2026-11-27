# Tasks

## 1. The folders

- [x] 1.1 In `crates/timber/moon.yml`, set `CARGO_TARGET_DIR` of `tools`, `tools-ecr`, `tools-ghcr`, `kms`, `fleet`, `dash0`, `github-release` and `cachix` to `$workspaceRoot/target/<task>`, with a comment that says why (design 2). Check: `sluice exec main -- moon run cli:tools`.
- [x] 1.2 In `plugins/herdr/moon.yml`, set `CARGO_TARGET_DIR` of `host` to `$workspaceRoot/target/plugin-herdr`. `tests/host.sh` reads the folder from `cargo metadata`. Check: `moon task plugin-herdr:host --json` gives the expanded folder.
- [x] 1.3 In `crates/timber-conformance/moon.yml`, set `CARGO_TARGET_DIR` of `coverage` to `$workspaceRoot/target/coverage`. Check: `sluice exec main -- moon run conformance:coverage`.

## 2. The check

- [x] 2.1 Copy flume's `check-targets.sh` and `check-targets-cases.sh` (design 1, 2 and 4). Find `-p <package>` before a `)` too, with a case. Add the tasks `root:targets` and `root:targets-cases` in the check-fast layer. Check: `sluice exec main -- moon run root:targets root:targets-cases`. Over the tree of main before this change, `check-targets.sh` fails. It names `cli:test`, `cli:tools`, the by-hand tasks of `cli`, `plugin-herdr:host`, `conformance:test` and `conformance:coverage`.
