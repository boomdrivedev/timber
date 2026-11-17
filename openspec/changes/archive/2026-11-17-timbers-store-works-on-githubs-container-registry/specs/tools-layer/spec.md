## MODIFIED Requirements

### Requirement: A check that needs a person's sign-in runs outside the gate

A check that needs a credential a person signs in for, as the runs
against ECR, against GitHub's registry and against a KMS key do, SHALL
run outside the gate, under a tag of its own, by hand with a live
session or by a job that holds an identity of its own (core.38).

#### Scenario: The gate runs no check that needs a sign-in

- **WHEN** the gate runs for a landing to `main`
- **THEN** no task that needs a credential a person signs in for runs, and each such task runs only when its own tag, `check-ecr`, `check-ghcr` or `check-kms`, is asked

## ADDED Requirements

### Requirement: The GHCR check's repository and credential are the host's

The repository of GitHub's registry the GHCR check pushes to SHALL be
the host's, named as `env.TIMBER_TOOLS_GHCR` in its `devenv.local.nix`,
and its credential SHALL be the one the host's Docker configuration
gives for `ghcr.io`, reached at use. No file of the tree SHALL name
either (64, core.21).

#### Scenario: The GHCR check's repository

- **WHEN** the GHCR check runs
- **THEN** it writes under the repository `env.TIMBER_TOOLS_GHCR` names and nowhere else, with the credential the host's Docker configuration gives for `ghcr.io`, and no file the tree tracks names either
