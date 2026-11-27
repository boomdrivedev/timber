## Purpose

The checks of the gate on the tree as a whole, which hold how the gate
runs its tasks in one checkout.

## ADDED Requirements

### Requirement: Each task that builds a binary builds it in its own folder

Each task that builds a binary of the workspace SHALL build it in a
target folder that no other such task uses. Thus no build of one task
writes the binary that the tests of another task run. `root:targets`
SHALL fail where two tasks build one binary in one folder, and SHALL
name the tasks and the folder (core.25).

#### Scenario: Two tasks build the timber binary in one folder

- **WHEN** `root:targets` reads a tree where `cli:test` and `cli:tools` set no `CARGO_TARGET_DIR`
- **THEN** it fails and names `cli:test`, `cli:tools` and the folder `target`

#### Scenario: A script that a task runs builds the binary in a subshell

- **WHEN** `root:targets` reads a tree where a task runs a script that runs `(cargo build -p timber)`, and that task and `cli:test` set no `CARGO_TARGET_DIR`
- **THEN** it fails and names the two tasks

#### Scenario: The tree as it is

- **WHEN** `root:targets` reads the tree
- **THEN** it passes, and `cli:tools`, each by-hand task of `cli`, `plugin-herdr:host` and `conformance:coverage` each build in a folder of their own under `target/`
