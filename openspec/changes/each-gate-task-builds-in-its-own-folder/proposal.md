# Each gate task that builds a binary builds it in its own folder

## Why

cargo writes `target/debug/<binary>` again at each build, also at a
build that has nothing to do. Each such build gives the file a new
inode. A `cargo test` of a package that has integration tests does the
same. This tree pins cargo 1.98.1. With it, three builds of
`timber-conformance` gave `target/debug/coverage` three inodes, and two
of the builds had nothing to do. A gate runs its tasks
at the same time in one checkout. sluice found this race on its own
gate (sluice 7fc9ca6). There, a test started the binary while the cargo
of another task wrote it, and the test found no file. flume found the
same race and repaired it (flume 99fb79ed).

timber's gate has the same race. Ten tasks build `target/debug/timber`:

| Task | Build | Layer |
|---|---|---|
| `cli:test` | `cargo test -p timber` | check-fast |
| `cli:tools` | `cargo test -p timber --features tools` | check-tools |
| `cli:tools-ecr`, `cli:tools-ghcr`, `cli:kms`, `cli:fleet`, `cli:dash0`, `cli:github-release`, `cli:cachix` | `cargo test -p timber --features tools` | each by hand, with a tag of its own |
| `plugin-herdr:host` | `cargo build -p timber`, in `tests/host.sh` | check-host, by hand |

`cli:tools` builds the binary with other features. Thus a test of
`cli:test` can run a binary of other features, and the test does not
tell. Two tasks also build `target/debug/coverage`: `conformance:test`,
for the integration tests, and `conformance:coverage`, which runs it.

## What Changes

- **Each task that builds a binary has its own target folder.**
  `cli:tools`, each by-hand task of `cli`, `plugin-herdr:host` and
  `conformance:coverage` each set `CARGO_TARGET_DIR` to a folder of
  their own under `target/`. `cli:test` and `conformance:test` keep the
  workspace's `target`.
- **A check holds the tasks to this.** `root:targets` runs
  `check-targets.sh`, which finds each task that builds a binary of the
  workspace and its folder. It fails and names the tasks where two tasks
  build one binary in one folder. `root:targets-cases` runs its cases
  on scratch workspaces. The two scripts are copies of flume's, so that
  the two repositories hold one check.

## Capabilities

### New Capabilities

- `the-gate`: the checks of the gate on the tree as a whole.

No clause changes. The check is a gate check (core.25).
