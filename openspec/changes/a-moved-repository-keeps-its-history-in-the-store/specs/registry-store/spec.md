## ADDED Requirements

### Requirement: A root left answers where it went

Where the repository's own holds the name `moved` at a mark naming
another repository, the registry store SHALL refuse every operation
reaching the repository's own, its components' or its cache
repositories, naming the repository and where it moved, and write
nothing; it SHALL read an artifact's name and what it reaches as before
(C.1). It SHALL read the mark once in its life, before its first such
operation.

#### Scenario: A channel read at the old root

- **WHEN** a store bound for `acme/app`, whose own holds a mark naming `acme/site`, resolves `channel/beta`, reads the claims on an index, or puts bytes
- **THEN** each is refused naming `acme/app` and `acme/site`, and nothing is written

#### Scenario: A pin read at the old root

- **WHEN** the same store resolves `artifact/eco/site@1.3.0` and reads its index and the claims on it
- **THEN** each is answered from `<root>/artifact/eco/site`

#### Scenario: The mark read once

- **WHEN** a store bound for a repository whose own holds no mark makes three operations there
- **THEN** it asks for `moved` once

### Requirement: A carry's store reads past the mark naming where it carries

A store bound for a root its repository left, to carry from it to
another, SHALL read and write that root as any where its mark names the
root it carries to, and SHALL refuse as a root left where its mark
names any other (C.1).

#### Scenario: Reading the root left

- **WHEN** a store bound for `acme/app`, carrying to `acme/site`, reads a version's name, its index and its claims at `acme/app`, whose mark names `acme/site`
- **THEN** each is answered as before the mark

#### Scenario: A mark naming a third root

- **WHEN** the same store finds a mark naming `acme/web`
- **THEN** its operations there are refused naming `acme/app` and `acme/web`

### Requirement: A carry between two roots on real registries

The tools layer SHALL carry a repository's history between two roots of
one registry, on zot and on the reference distribution, through two
registry stores: every version with its claims and a channel's chain
walked at the new root, the old root answering where it went, and an
artifact's name still read there (C.1, 103, 105).

#### Scenario: A carry on zot and the reference distribution

- **WHEN** a version accepted and promoted under `acme/app` is carried to `acme/site`
- **THEN** a store bound for `acme/site` resolves its version, reads its claims and walks the channel's chain to its head, and a store bound for `acme/app` is refused naming `acme/site` for the channel and still reads the artifact's name

### Requirement: A carry on GitHub's registry

The GHCR check SHALL carry a repository's history between two roots
under the run's prefix, and one passing run by hand SHALL be the
acceptance of the carry on GitHub's registry (core.38).

#### Scenario: The check's carry

- **WHEN** the GHCR check carries a version and a channel between two roots under its prefix
- **THEN** the new root's packages hold the version, its claims and the channel, and the old root's package answers where it went
