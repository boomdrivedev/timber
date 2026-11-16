## MODIFIED Requirements

### Requirement: The gate runs the tools layer

The repository SHALL have a `check-tools` layer: the tests of the
profiles that need moon, sluice, OpenSSH, npm or a registry the test
starts itself, each run by a moon task of its project tagged
`check-tools`, one task per check (core.25). The gate SHALL require the
layer on `main` and on every bolt, beside `check-fast`. A test that
needs only git SHALL stay in `check-fast`.

#### Scenario: A landing to a bolt

- **WHEN** a unit is submitted to a bolt
- **THEN** the gate runs every task tagged `check-fast` and every task tagged `check-tools`, and no task tagged `check-ecr` or `check-kms`

#### Scenario: The fast layer alone

- **WHEN** `moon run ':#check-fast'` runs
- **THEN** no test that needs moon or sluice to run a scratch repository, npm, or a registry, runs, and nothing reaches AWS

#### Scenario: The registry store in the layer

- **WHEN** `moon run ':#check-tools'` runs
- **THEN** the registry store's tests run against a zot and a reference distribution each started for the test, and nothing reaches ECR or KMS

#### Scenario: npm in the layer

- **WHEN** `moon run ':#check-tools'` runs
- **THEN** the npm projection's pack scenarios and its `npm install` against a served layout, a first install with no lockfile among them, run with the npm the tree pins, under a home and an npm configuration of the test's own, and no registry but the served layout is asked for a family package

#### Scenario: The ECR check outside the gate

- **WHEN** `moon run ':#check-ecr'` runs on a host whose `devenv.local.nix` names an ECR repository, with a live session for it
- **THEN** the registry store's group runs against that ECR repository, in a task of its own

#### Scenario: The KMS check outside the gate

- **WHEN** `moon run ':#check-kms'` runs on a host whose `devenv.local.nix` names a KMS key and a second key, with a live session for their account
- **THEN** the key service signer's group runs against those keys, in a task of its own

#### Scenario: The KMS check on a host that names no key

- **WHEN** `moon run ':#check-kms'` runs on a host whose `devenv.local.nix` names no KMS key
- **THEN** it fails, naming the setting to set, and passes nothing it did not run

## ADDED Requirements

### Requirement: A check that needs a person's sign-in runs outside the gate

A check that needs a credential a person signs in for, as the runs
against ECR and against a KMS key do, SHALL run outside the gate, under
a tag of its own, by hand with a live session or by a job that holds an
identity of its own (core.38).

#### Scenario: The gate runs no check that needs a sign-in

- **WHEN** the gate runs for a landing to `main`
- **THEN** no task that needs a credential a person signs in for runs, and each such task runs only when its own tag, `check-ecr` or `check-kms`, is asked
