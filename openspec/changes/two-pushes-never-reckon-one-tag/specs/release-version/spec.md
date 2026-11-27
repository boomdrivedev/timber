## ADDED Requirements

### Requirement: Two pushes never reckon one tag

The runs of CI for pushes to main SHALL go one at a time, in the order
of the pushes. They SHALL go in one concurrency group of the workflow.
A push's run
SHALL NOT cancel the run that goes. No job SHALL name a group of its
own. The version job SHALL find the next version from the tags that the
repository holds when the job starts. A pull request's run SHALL wait
for no other run.

#### Scenario: The group of the runs

- **WHEN** `root:release` reads `.github/workflows/ci.yml`
- **THEN** the workflow's group is `release` for a push and the run's
  own for a pull request, with `cancel-in-progress: false`
- **AND** no job names a group

#### Scenario: A second push after the release of the first

- **WHEN** `next-version.sh` runs for a second push, with a fix since
  `v0.1.0`, after the release of the first push pushed `v0.1.1`
- **THEN** it prints `v0.1.2` ("two pushes" in
  `check-next-version.sh`)

#### Scenario: A run that a later release holds

- **WHEN** `next-version.sh` runs for the first push, after the release
  of the second push, which holds the commit of the first push, pushed
  `v0.1.2`
- **THEN** it prints nothing ("a late run" in `check-next-version.sh`)

## MODIFIED Requirements

### Requirement: CI reckons the next release from the newest tag by version

A release's tag names a commit off main's history. So CI's version job
SHALL run `next-version.sh`, which reckons the next version with
git-cliff's `--unreleased` and `--bumped-version`, whose range starts at
the newest tag. It SHALL give no version where that version has a tag.
The release's notes SHALL be git-cliff's `--unreleased` notes. The
conventional check SHALL check the commits since the newest `v*` tag by
version.

#### Scenario: The version, notes and conventional steps

- **WHEN** `root:release` reads `.github/workflows/ci.yml`
- **THEN** the version job runs `sh next-version.sh`, the release job's
  notes run `git-cliff --unreleased`, and the conventional job's range
  starts at the first of `git tag --list 'v*' --sort=-v:refname`, with
  no `git describe`

#### Scenario: No commit that bumps the version

- **WHEN** `next-version.sh` runs where the commits since the newest
  tag are chores alone
- **THEN** it prints nothing ("no bump" in `check-next-version.sh`)
