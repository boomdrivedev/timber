## ADDED Requirements

### Requirement: The binding declares a registry and a root

The registry store SHALL read the text the host binds, `<host>/<root>`,
as the registry and the root every repository of the store is under,
and SHALL hold every name of the store as a tag of a repository under
that root (C.1). It SHALL never create a repository of its own accord;
a registry that creates repositories on push may create each one on
its first put.

#### Scenario: Everything under the root

- **WHEN** a store bound to `<host>/store` for the repository `acme/site` puts bytes, an artifact and an index of a declared component, attaches an attestation, and makes the component's version, an artifact's name and a cache entry
- **THEN** every repository written is `store` or under it, and no other repository is written

### Requirement: A repository's own root is its remote's owner and name

A store bound for a repository SHALL hold that repository's components
and cache entries under `<root>/<owner>/<name>`, the repository's own,
the owner and name its landing's remote gives, each lowercased and
escaped into one path component (114, 138, C.1). An owner written
`artifact` SHALL be refused naming it and the artifact space,
`<root>/artifact/`, which it would share, and nothing written.

#### Scenario: An owner and a name lowercased

- **WHEN** a store bound to `<host>/store` for the repository `Acme/Site` makes `lib:lib-v1.2.0` at an index
- **THEN** the tag `lib-v1.2.0` is held in `store/acme/site/lib`, and `store/acme/site` holds the index

#### Scenario: An owner named artifact

- **WHEN** a store bound for the repository `artifact/npm` puts bytes, or makes a version's name
- **THEN** each is refused naming the owner `artifact` and the artifact space, and nothing is written

### Requirement: A store bound for no repository reads the artifact space alone

Where the binding gives the store no repository, as where the landing
names no remote or its address ends in no owner and name, every write,
and every name of the repository's own spaces, SHALL answer as the
binding answered the repository, absent or unreadable naming the
remote, and write nothing; an artifact's name, a name's versions and
the digests they reach SHALL be read as for a repository (49, 58,
138).

#### Scenario: A consumer with no remote

- **WHEN** a store bound for no repository resolves `artifact/eco/site@1.3.0`, reads its index, the artifact and its bytes, and asks the claims on the index and the artifact
- **THEN** each is answered from `<root>/artifact/eco/site`

#### Scenario: A write with no remote

- **WHEN** the same store puts bytes, or makes the name `v1.2.0`
- **THEN** each is answered absent naming the landing's remote, and nothing is written

### Requirement: A digest is read where its name was resolved

The registry store SHALL keep, for its life alone, the repository each
name it resolved or made is a tag of, against the digest the name
names, and each digest a manifest it read there names, against that
repository. read, claims-of, attach and name SHALL find a digest in
the repositories so kept first, then in the repository's own, then in
the bound build profile's cache repository (72, 138, C.1, core.20).

#### Scenario: Another repository's pin read where its name is

- **WHEN** a store bound for the repository `acme/app` resolves `artifact/eco/site@1.3.0`, which `acme/site` published, reads the index, the artifact and its bytes, and asks the claims on the index and the artifact
- **THEN** each is answered from `<root>/artifact/eco/site`, and nothing is asked of `<root>/acme/app`

#### Scenario: A digest no name reached

- **WHEN** a store reads a digest that no name it resolved or made reached
- **THEN** it asks the repository's own and then the cache repository, and answers absent where neither holds it

### Requirement: A claim is held beside its subject in every repository holding it

Where the store holds a manifest in a name's repository, it SHALL hold
there each attestation of timber's envelope type attached to that
manifest and to each manifest it names. attach SHALL write the
attestation into the repository's own, each repository the subject was
reached in, and, for an index, its component's and each of its
artifacts' names' repositories, each where it holds the subject, and
answer held already only where each held it (8, C.1, core.27).

#### Scenario: Claims carried to an artifact's name

- **WHEN** a publish attaches an attestation to each artifact and one to the index, and then makes each artifact's name
- **THEN** each name's repository lists the attestation on the index and the one on the artifact it names, and a store reading through that name answers each byte for byte

#### Scenario: A claim attached after the names are made

- **WHEN** an acceptance is attached to an index whose component's version and artifacts' names are made
- **THEN** the repository's own, the component's repository and each artifact's name's repository list it

#### Scenario: An attestation held in part

- **WHEN** an attestation is held in the repository's own and not in an artifact's name's repository that holds its subject, and it is attached again
- **THEN** it is written into the name's repository, and the answer is not held already

#### Scenario: Another tool's referrer is not carried

- **WHEN** a manifest the repository's own holds has a referrer of another artifact type beside timber's attestation, and an artifact's name comes to hold it
- **THEN** the name's repository holds timber's attestation and not the other referrer

### Requirement: Three spaces under the root

Three spaces SHALL be held under the root: a declared component's
versions and channels, tags of its own repository under the
repository's own; each cache entry, a tag of its build profile's cache
repository under the repository's own; and each artifact's name, a tag
of a repository of its own under the root and no repository's own (C.1;
record 0040).

#### Scenario: A component's version, an artifact's name and a cache entry

- **WHEN** a store bound for the repository `acme/site` puts a declared component's version, an artifact's name and a cache entry
- **THEN** the version's tag is held in `<root>/acme/site/<component>`, the cache entry's in `<root>/acme/site/cache/<build>`, and the artifact's name in `<root>/artifact/<ecosystem>/<name>`

#### Scenario: Two repositories' components of one name

- **WHEN** `acme/site` and `acme/app`, each declaring the component `lib`, each make `lib:lib-v1.0.0` at an index of its own in one store
- **THEN** `<root>/acme/site/lib` and `<root>/acme/app/lib` each hold the tag at its own index, and a store bound for either resolves its own

## MODIFIED Requirements

### Requirement: An attestation is a referrer of its subject

attach SHALL push a referrer of timber's envelope type whose subject
is the digest and whose one layer is the envelope, with the statement's
predicate type annotated, and answer the envelope's own digest;
claims-of SHALL answer every envelope so attached, byte for byte (8,
68, 69, C.1, core.27). Both SHALL answer absent for a digest the store
finds in no repository it asks, and attach SHALL write nothing then.

#### Scenario: Two attestations read back

- **WHEN** two attestations are attached to one artifact
- **THEN** claims-of answers both envelopes byte for byte, and the registry lists two referrers of the artifact, each of the envelope type and annotated with its statement's predicate type

#### Scenario: The same attestation attached twice

- **WHEN** one attestation is attached twice to one digest
- **THEN** the second attach answers held already, and claims-of answers it once

#### Scenario: Another tool's referrer

- **WHEN** a digest has a referrer of another artifact type, such as a signature another tool pushed, beside timber's attestations
- **THEN** claims-of answers only timber's attestations, and the other referrer is left as it was

### Requirement: A declared component's versions and channels are tags of its own repository

The registry SHALL hold a declared component's version's name
`<component>:<tag>` as the tag `<tag>`, and its channel's name
`channel/<component>/<name>` as the tag `<name>`, of the component's
repository under the repository's own, the component escaped into one
path component, each moved with the digest expected and read back
there. A version's bare tag and `channel/<name>`, of a repository that
declares no component, SHALL be tags of the repository's own (C.1,
109, 125, core.6).

#### Scenario: A declared component's version

- **WHEN** a publish of the component `lib` makes `lib:lib-v1.2.0` at its index
- **THEN** the repository `lib` under the repository's own has the tag `lib-v1.2.0` at the index, resolve of `lib:lib-v1.2.0` answers it, and the repository's own holds no tag `lib-v1.2.0`

#### Scenario: Two components' channels of one name

- **WHEN** a promotion moves `channel/lib/beta` to one index and another moves `channel/app/beta` to another
- **THEN** the repository `lib` under the repository's own has the tag `beta` at the first index, the repository `app` has the tag `beta` at the second, each resolves to its own, and the repository's own holds neither tag

#### Scenario: A repository that declares no component

- **WHEN** a publish makes the version `v1.2.0` and a promotion moves `channel/stable` to its index
- **THEN** the repository's own holds the version's and the channel's tags, and no other repository is written for them

#### Scenario: A channel moved from where it is not

- **WHEN** `channel/lib/beta` is at one index and a move to another names a third as the digest expected
- **THEN** the move is refused naming the digest the tag is at in the repository `lib`, and the tag does not move

### Requirement: A declared component's repository holds its indexes

When the registry puts an index whose component is a declared one, it
SHALL hold the index, every manifest and blob it names and the
attestations on each in that component's repository as well as the
repository's own, mounted or, where the registry declines the mount,
uploaded under their own digests; and before a version's or a
channel's tag is put there it SHALL hold the index the tag names,
writing only what the repository lacks (8, C.1, core.5).

#### Scenario: An index published

- **WHEN** a publish of the component `lib` puts its index into a registry store
- **THEN** the repository `lib` under the repository's own holds the index and every artifact manifest and blob it names, untagged, with the attestations on each artifact, and the repository's own holds them too

#### Scenario: An index put before this change

- **WHEN** the repository `lib` lacks an index the repository's own holds, and a promotion moves `channel/lib/beta` to it
- **THEN** the index, what it names and the attestations on each are held in `lib` before the tag `beta` is put there

### Requirement: An artifact's name is a tag of a repository of its own

The registry SHALL hold the name `artifact/<ecosystem>/<name>@<version>`
as the tag `<version>` of the repository
`<root>/artifact/<ecosystem>/<name>`, one space for the whole store and
under no repository's own, the ecosystem and the name each lowercased
and escaped into one path component, and the version escaped into a
tag one way and read back. A repository path past 255 characters SHALL
be refused naming the name, and nothing written (70, 71, 138, C.1).

#### Scenario: A scoped package's name

- **WHEN** the name `artifact/npm/@acme/site@1.3.0` is made at an index and resolved, and so is `artifact/npm/acme-site@1.3.0` at another
- **THEN** each resolves to its own index, the first in the repository `artifact/npm/0_40acme_2fsite` and the second in `artifact/npm/acme-site` under the root, each tagged `1.3.0` and each listing the attestations on its index

#### Scenario: A version a tag cannot hold as written

- **WHEN** the name `artifact/cargo/site@1.0.0+build.1` is made at an index, and its name's versions are listed
- **THEN** the tag escapes the `+`, and the listing answers `1.0.0+build.1` as written

#### Scenario: A name moved from where it is not, in its own repository

- **WHEN** an artifact's name is made at one index, then made again from nothing at another
- **THEN** the second is refused naming the first index, and the name still resolves to it

#### Scenario: One name for every repository

- **WHEN** a store bound for `acme/site` makes `artifact/eco/site@1.3.0`, and a store bound for `acme/app` resolves it
- **THEN** both answer the same index, from `<root>/artifact/eco/site`

### Requirement: An artifact name's repository holds what its index names

Before an artifact name's tag is put, the index it names, every
manifest and blob the index names and the attestations on each SHALL
be held in that name's repository, mounted from the repository the
store found them in or, where the registry declines the mount, uploaded
under their own digests (8, 138, C.1, core.5).

#### Scenario: A mount declined

- **WHEN** the registry declines the mount of a blob the index names, and the name is made
- **THEN** the blob is uploaded under its own digest into the name's repository, the tag is put, and the name resolves to the index

### Requirement: A name's versions are its repository's tags

The registry SHALL answer the listing of an ecosystem and a name from
the tag list of that name's repository, every page of it, each tag read
back as a version and a tag in the shape of a subject's referrers tag,
which no version's tag takes, left out; a repository the registry does
not hold, or one with no tags, SHALL be the empty list, and any other
tag that does not read back SHALL be unreadable naming the repository
and the tag (138, C.1, core.19).

#### Scenario: Versions listed by tags

- **WHEN** two versions of `artifact/npm/@acme/site` are named and its versions are asked
- **THEN** the answer is the two versions and no other

#### Scenario: A name the registry never held

- **WHEN** the versions of a name whose repository the registry does not hold are asked
- **THEN** the answer is the empty list, neither absent nor refused

#### Scenario: A referrers tag beside the versions

- **WHEN** a name's repository on a registry with no referrers API holds two versions and the referrers tag of their index
- **THEN** the listing answers the two versions and no other

### Requirement: A cache entry is a tag of its build profile's cache repository

The registry SHALL hold the name `cache/<build identity>` as the tag of
the build identity in the repository `cache/<build>` under the
repository's own, `<build>` the bound build profile's name, with its
cache manifest and the bytes it names held there, mounted from the
repository the store found them in or uploaded under their own digest
where they are not (52, 135, C.1, core.5).

#### Scenario: An entry named from bytes the declared repository holds

- **WHEN** a store bound for a publish puts bytes into the repository's own and makes `cache/b7e1` at their digest
- **THEN** the bytes are mounted into `cache/moon` under the repository's own, or uploaded there under their digest where the mount is declined, before the cache manifest and the tag are put

### Requirement: A push writes into the cache repository alone

A store bound for a push SHALL put bytes into the cache repository
under the repository's own, and every other binding into the
repository's own. A read by digest SHALL find the digest as "A digest
is read where its name was resolved" says, the cache repository last
(54, 135, core.14).

#### Scenario: A push's cache entry

- **WHEN** a store bound for a push for the repository `acme/site` puts a cache entry's bytes and makes `cache/b7e1` at their digest, with the moon build profile bound
- **THEN** the repository `<root>/acme/site/cache/moon` holds the bytes, the cache manifest and the tag `b7e1`, the repository's own holds none of them, and resolve of `cache/b7e1` answers the bytes' digest

#### Scenario: An entry's bytes read back

- **WHEN** a store bound for a sync resolves `cache/b7e1` and reads the digest it answers
- **THEN** the read answers the entry's bytes from the cache repository

### Requirement: A repository nobody made is refused naming it

The registry store SHALL never make a repository by an API of its own.
Where a registry refuses a put because the repository it is for does not
exist, as one that makes repositories only by a creation template does
where none applies, the put SHALL be refused carrying the registry's
words verbatim and naming the repository, and nothing SHALL be retried
(C.1, 58, 59).

#### Scenario: An artifact name's repository missing

- **WHEN** a registry that makes no repository on push holds the repository's own and not `artifact/eco/site` under the root, and the name `artifact/eco/site@1.3.0` is made
- **THEN** the make is refused carrying the registry's words and naming `artifact/eco/site` under the root, and no tag is put

#### Scenario: A component's repository missing

- **WHEN** a registry that makes no repository on push holds the repository's own and not `lib` under it, and a publish of the component `lib` makes `lib:lib-v1.2.0`
- **THEN** the make is refused carrying the registry's words and naming `lib` under the repository's own, nothing is asked again, and the version's tag is not made

#### Scenario: A cache repository missing

- **WHEN** a registry that makes no repository on push holds the repository's own and not `cache/moon` under it, and a store bound for a push puts a cache entry's bytes, with the moon build profile bound
- **THEN** the put is refused carrying the registry's words and naming `cache/moon` under the repository's own, nothing is asked again, and the repository's own is not written

#### Scenario: The repository's own missing

- **WHEN** a registry that makes no repository on push does not hold the repository's own, and bytes are put
- **THEN** the put is refused carrying the registry's words verbatim and naming the repository's own, and nothing is written elsewhere

### Requirement: The store's scenarios pass against GitHub's registry

The suite's store group and the cache manifest's scenarios SHALL pass
against a repository of GitHub's registry the host names, in a check of
its own outside the gate, tagged `check-ghcr` and run by hand with the
host's credential, since it needs a credential a person signs in for.
The check SHALL refuse to run, naming the setting, on a host that names
no repository of `ghcr.io` (93, core.38).

#### Scenario: GitHub's registry, by hand

- **WHEN** `moon run ':#check-ghcr'` runs on a host whose `devenv.local.nix` names a repository of `ghcr.io`, with the host's Docker configuration holding a credential that writes its packages
- **THEN** every scenario of the group passes, attestations held through the referrers tag and each run's names apart from every earlier run's, and the group's refusal of a name moved from where it is not is answered not applicable

#### Scenario: A repository the push makes is private

- **WHEN** the check makes an artifact's name in a repository no earlier run wrote
- **THEN** GitHub's registry makes the repository on the push, the name resolves to its index, and a read of the index there with no credential is never answered the index

#### Scenario: No repository of GitHub's registry named

- **WHEN** the GHCR check starts on a host that names no repository, or one on another host than `ghcr.io`
- **THEN** it refuses to run naming the setting the host must give

#### Scenario: Each repository's packages under its own root

- **WHEN** the check makes a declared component's version and a cache entry for the repository its store is bound for, and an artifact's name at the version's index
- **THEN** the version's and the entry's tags are held in packages under `<root>/<owner>/<name>/`, the name's under `<root>/artifact/`, and a store bound for another repository reads the claims on the index from the name's package

## REMOVED Requirements

### Requirement: One declared repository holds the store

**Reason**: C.1, as amended at `boomdrivedev/blueprints` f412008, has
the binding declare a registry and a root, and holds every blob,
manifest and referrer a name reaches in that name's repository. No one
repository holds the store, since a package on GitHub's registry is
public or private as a whole (record 0040).

**Migration**: "The binding declares a registry and a root" and "A
repository's own root is its remote's owner and name" take its place.
A host's `[store] registry` stays as written, its path read as the
root.

### Requirement: Three spaces under the named repository

**Reason**: a repository's components and cache entries now hang under
its own `<root>/<owner>/<name>`, and artifacts' names stay one space
under the root (C.1 as amended at f412008; record 0040).

**Migration**: "Three spaces under the root" says where each space is
held.
