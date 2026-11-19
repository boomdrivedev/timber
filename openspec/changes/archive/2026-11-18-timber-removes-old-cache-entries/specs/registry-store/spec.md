## MODIFIED Requirements

### Requirement: A cache entry's bytes are named through a cache manifest

To name a digest held only as bytes, the registry store SHALL tag a
manifest of timber's cache-entry type whose one layer is those bytes,
annotated with the name it is made for and the moment it was put, and
every reading of that tag, by resolve, by name's check and read back,
and in a refusal, SHALL answer the bytes' digest (52, 70, 71, 100, 145,
C.1, ADR 0014). The cache manifest is no artifact: read SHALL never
answer it as one, and claims-of SHALL never list it (53, core.30).

#### Scenario: A cache entry named and resolved

- **WHEN** bytes are put and a name is made from nothing at their digest
- **THEN** resolve answers the bytes' digest, and the tag is at a manifest of the cache-entry type whose one layer is those bytes, annotated `io.github.agentplot.timber.store-name` with the name and `org.opencontainers.image.created` with the moment the store put it

#### Scenario: The same bytes named twice

- **WHEN** two names are made at the digest of the same bytes
- **THEN** both resolve to the bytes' digest, and each tag is at a cache manifest of its own naming it

#### Scenario: A cache entry's name moved

- **WHEN** a name at a cache entry's bytes is moved, with those bytes' digest as expected, to other bytes, and then moved with the first bytes' digest as expected
- **THEN** the first move succeeds and resolves to the other bytes' digest, and the second is refused naming the other bytes' digest

#### Scenario: The cache manifest is no artifact

- **WHEN** a name is made at bytes and the cache manifest's own digest is read, and claims-of is asked of the bytes' digest
- **THEN** read answers bytes and never an artifact, and claims-of lists no cache manifest

#### Scenario: Bytes not held

- **WHEN** a name is made at a digest the repository holds neither as a manifest nor as bytes
- **THEN** name is absent naming the digest, and no manifest or tag is put

#### Scenario: A name found held before the put

- **WHEN** a name is made from nothing at bytes and the name's check before the put finds it held
- **THEN** the answer names where it is, and no cache manifest is put

## ADDED Requirements

### Requirement: A cache repository's entries are listed with their push moments

The registry store SHALL list a cache entry for each tag of the bound
build profile's cache repository, reading the tag's cache manifest for
the layer's digest and its push moment, none where it carries no
moment. A tag at any other manifest, or one it cannot read, SHALL be
that entry's answer. A repository the registry does not hold SHALL list
no entries (145, C.1, rule core.19).

#### Scenario: Entries of both forms

- **WHEN** the cache repository holds one entry written with a moment and one whose cache manifest carries none
- **THEN** the listing answers the first with its moment and the second with none, each at its bytes' digest

#### Scenario: A tag at another manifest

- **WHEN** a tag of the cache repository names a manifest that is no cache manifest
- **THEN** its entry is answered unreadable naming the tag, and the other entries are listed

#### Scenario: A cache repository never made

- **WHEN** the store lists the cache of a repository whose cache repository the registry does not hold
- **THEN** the listing is empty

### Requirement: A cache entry is removed by deleting its cache manifest

The registry store SHALL remove a cache entry by deleting, through the
distribution protocol, the cache manifest its tag names, only where the
manifest's layer is the digest given, and SHALL delete nothing outside
the bound build profile's cache repository and no untagged manifest
(145, 54, record 0039). It SHALL read no mark and ask nothing of the
repository's own (record 0042).

#### Scenario: An entry removed

- **WHEN** a cache entry is removed at the digest its name is at
- **THEN** the tag is gone, another entry of the same bytes still resolves, and an artifact holding those bytes still reads

#### Scenario: A registry that refuses the delete

- **WHEN** the registry answers the delete with a refusal
- **THEN** the removal is refused in the registry's words, and the tag still resolves

#### Scenario: A root marked moved

- **WHEN** a cache entry is removed from a root that holds the mark `moved`
- **THEN** it is removed, and the mark was not read

### Requirement: On GitHub's registry a cache entry is removed through GitHub's API

Where the registry's host is `ghcr.io`, the store SHALL remove a cache
entry by deleting, through GitHub's REST API, the package version whose
name is the cache manifest's digest, in the cache repository's package
of the root's namespace, read as an organization's or a user's, with
the token the host's Docker credential for `ghcr.io` holds, reached at
use (145, rule core.21).

#### Scenario: An organization's package version deleted

- **WHEN** a cache entry is removed under a root whose namespace GitHub's API answers as an organization
- **THEN** the store finds the version by the manifest's digest under that organization's package, deletes it by its id, and sends the Docker credential's token as the bearer

#### Scenario: GitHub refuses the delete

- **WHEN** GitHub's API answers the delete `403`
- **THEN** the removal is refused in GitHub's words

#### Scenario: No version of that digest

- **WHEN** no version of the package is named the manifest's digest
- **THEN** the removal is absent naming the package and the digest

### Requirement: Removal passes against real registries

The store group's cache scenarios, and a prune leaving a version, its
claims and a channel's name, SHALL pass against zot and the reference
distribution in the tools layer; and the store group's cache scenarios
SHALL pass by hand against ECR and GitHub's registry once before the
change lands (145, rule core.38).

#### Scenario: A prune on both registries

- **WHEN** an entry past its time, an entry within it and a promoted version are held on zot or the reference distribution, and prune runs
- **THEN** the first entry is gone, the second resolves, and the version, its claims and the channel's name read as before
