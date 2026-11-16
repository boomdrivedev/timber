## MODIFIED Requirements

### Requirement: The host keeps which versions it published from which revisions

After a publish moves a version's name to its index, the host SHALL
keep one entry for that version naming its component, version, tag,
revision, branch, the index's digest and the store it was offered to,
the store as the host's bindings name it, and, where the index names
notes, the last release tag and the revisions of the commits its notes
were rendered from (61, 118, 134, core.20).

#### Scenario: An entry after a publish

- **WHEN** a version is published
- **THEN** the host's record holds one entry for that component and version naming the revision, the branch, the index's digest, the store and the revisions its notes were rendered from, and no envelope, artifact digest, notes' text or credential

#### Scenario: The store as the bindings name it

- **WHEN** the built binary publishes a version on a host whose bindings file names `[store] layout` with a path beginning `~/`, and status is asked with `--json`
- **THEN** the version's entry names the store as `layout` and that path with `~/` expanded

#### Scenario: Nothing kept of a refused publish

- **WHEN** a publish is refused or stops before its index
- **THEN** the host's record holds no entry for that version

#### Scenario: An entry written before notes

- **WHEN** the record holds an entry written before entries named the notes' commits, and the same version, whose index names no notes, is published again
- **THEN** the entry reads back naming no notes and is answered held, its file unchanged

### Requirement: The record is rebuilt from the store and git

A rebuild SHALL write the host's record from what the store and git
hold: for each component the policy file declares at a landed revision,
each tag carrying its prefix that names a revision the branch landed,
and whose name the store resolves to that component's index at that
version, one entry, the same file a publish of it writes (61 to 63).

#### Scenario: S12, everything kept is deleted and rebuilt

- **WHEN** two components' versions are published, the host's record is deleted, and the record is rebuilt
- **THEN** every file of the record holds the same bytes it held before the deletion, the notes' commits among them, and status answers the same as before

#### Scenario: A claimed version with no index

- **WHEN** a version's tag is claimed but the store names no index under it, and the record is rebuilt
- **THEN** the record holds no entry for that version, as a publish that stopped before its index keeps none

#### Scenario: A version another branch landed

- **WHEN** a tag carrying a component's prefix names a revision the branch asked for did not land, and the record is rebuilt for that branch
- **THEN** the record holds no entry for that tag

#### Scenario: A version published before notes

- **WHEN** a tag's index names no notes and the record is rebuilt
- **THEN** that version's entry names no notes' commits

## ADDED Requirements

### Requirement: An entry keeps nothing the store or git holds besides

An entry SHALL keep nothing the store or git holds besides what it
names: no artifact, no claim, no notes' text and no credential (61, 64,
core.20, core.21).

#### Scenario: An entry of a version with notes and claims

- **WHEN** a version whose index names two artifacts, each with claims attached, and notes is published
- **THEN** its entry holds no artifact's bytes or digest, no envelope, no notes' text and no credential

### Requirement: A rebuilt entry's notes are derived again at the tag's revision

Where the index of a rebuilt entry names notes, the entry's commits
SHALL be those the version rule derives the version from again at the
tag's revision, under the policy file at that revision (21, 63, 134).

#### Scenario: A version with notes rebuilt

- **WHEN** a version whose index names notes is published, and the record is deleted and rebuilt
- **THEN** the rebuilt entry names the commits the version rule derives at the tag's revision, the same commits the publish's entry named
