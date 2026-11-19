# registry-store Specification

## Purpose
The store seam bound to an OCI registry over the distribution protocol:
where a registry holds bytes, artifacts, indexes, attestations and
names, what each store operation sends and answers, and the registries
the profile is proven against (B.1, C.1).

## Requirements

### Requirement: Bytes are blobs, put once

put-bytes SHALL upload bytes as a blob under their own digest, after
asking the registry whether it holds that blob; when it does, nothing
SHALL be uploaded and the answer SHALL say held already (5, 66,
core.5). read SHALL answer a blob's bytes by digest, and bytes whose
digest is not the one asked for SHALL be refused naming both (2).

#### Scenario: The same bytes put twice

- **WHEN** the same bytes are put twice
- **THEN** both answers name the bytes' own digest, the second says held already, and the second put uploads nothing

#### Scenario: A registry that answers other bytes

- **WHEN** the registry answers a blob read with bytes whose digest is not the digest asked for
- **THEN** read is refused naming the digest asked for and the digest of the bytes answered

### Requirement: An artifact is a manifest in the layout's own bytes

put-artifact SHALL put an artifact as an OCI image manifest and an index
as an OCI image index, by digest alone, in exactly the bytes the layout
store writes for the same artifact, so one artifact has one digest in
every store (1, 67). A manifest the registry holds already SHALL be
answered held already and not put again (5).

#### Scenario: One artifact, one digest, in a layout and a registry

- **WHEN** the same artifact is put into a layout and into a registry
- **THEN** both answer the same digest, and the registry's manifest bytes are the layout's blob bytes

#### Scenario: An artifact and an index read back

- **WHEN** an artifact and an index naming it are put and each digest is read
- **THEN** read answers the artifact's ecosystem, name, version and layers in order, and the index's component, version and artifacts in order

#### Scenario: A manifest naming bytes not held

- **WHEN** an artifact naming a digest the repository does not hold is put
- **THEN** the put is absent naming that digest, and no manifest is put

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

### Requirement: The referrers tag where the registry has no referrers API

Where a registry answers no referrers API, attach SHALL record the
referrer in the referrers tag the OCI distribution specification
defines for the subject, adding it once and keeping every entry already
there, and claims-of SHALL read that tag (C.1). A referrers tag that
lost the entry to another writer SHALL be reported, never retried.

#### Scenario: Attestations on a registry with no referrers API

- **WHEN** two attestations are attached to one artifact in the reference distribution
- **THEN** the subject's referrers tag lists both, and claims-of answers both envelopes byte for byte

#### Scenario: An entry another writer removed

- **WHEN** the referrers tag read back after an attach no longer lists the attestation just pushed
- **THEN** attach is answered unreadable naming the referrers tag and the attestation, and nothing is pushed again

### Requirement: A name is a tag moved from where it is expected and read back

name SHALL read the tag first and refuse a move from anywhere but the
expected digest, naming where it is; after the put it SHALL read the
tag back, again while it answers the former place up to the profile's
bound, and refuse naming a third digest as a lost race, or the former
place once the bound passes, never putting it again (70, 71, 109, C.1,
core.6). A name SHALL be escaped into a valid tag, one name to one tag.

#### Scenario: A name made, moved and resolved

- **WHEN** a name is made from nothing at one manifest and moved from there to another
- **THEN** resolve answers the second digest, and the repository holds one tag for the name

#### Scenario: A move from where the name is not

- **WHEN** a name at one digest is moved with an expected digest it is not at
- **THEN** the move is refused naming the digest the name is at, and the tag is not written

#### Scenario: A race lost after the put

- **WHEN** another writer moves the tag between the store's put and its read back
- **THEN** the move is refused naming the other writer's digest, and the store puts the tag no second time

#### Scenario: A read back at the former place within the bound

- **WHEN** the tag's read back after the put answers the digest it was moved from, as a registry not yet showing the put answers it, and a read again within the profile's bound answers the digest it was moved to
- **THEN** the move is made and is no lost race, and the store puts the tag no second time

#### Scenario: A name no tag can carry as written

- **WHEN** a name holding characters a tag cannot hold, such as `/` and `+`, is made and resolved, and so is a name differing from it only in those characters
- **THEN** each resolves to its own digest, and the repository holds a valid tag for each

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

### Requirement: Absent, refused and unreadable kept apart

A digest or name the repository does not hold SHALL be absent; a
registry that refuses SHALL be refused with its own words verbatim; a
registry that cannot be reached or answers what cannot be read SHALL be
unreadable with the client's or the registry's words (58, 59,
core.19).

#### Scenario: A registry that cannot be reached

- **WHEN** the registry's address answers no connection and bytes are read
- **THEN** the read is unreadable carrying the client's words, and never absent

#### Scenario: A tag the registry will not move

- **WHEN** the registry refuses a tag put, as an ECR repository with immutable tags does
- **THEN** name is refused carrying the registry's error body verbatim, and nothing is retried

#### Scenario: Bytes the registry holds and will not serve

- **WHEN** a registry answers a blob's `GET` `404` and its `HEAD` `200`, as GitHub's registry does in a package that holds no manifest yet, and the bytes are read
- **THEN** the read is unreadable carrying the registry's words, and never absent

### Requirement: The credential is the host's, reached at use

The registry store SHALL reach a registry's credential at the moment of
use through the host's Docker configuration and the credential helper
it names, and SHALL keep no credential, write none into any file, and
name none in the bindings file (64, core.21).

#### Scenario: A registry behind a credential helper

- **WHEN** the host's Docker configuration names a credential helper for the registry and an artifact is put
- **THEN** the store asks the helper for the credential when the registry asks for one, and no file timber writes holds it

#### Scenario: No credential for a registry that asks for one

- **WHEN** the registry answers that a credential is required and the host has none for it
- **THEN** the operation is refused carrying the registry's words, and names the registry

### Requirement: The store's conformance scenarios pass against real registries

The suite's store group SHALL pass against zot and against the
reference distribution in the tools layer, which the gate requires,
and against a real ECR repository in a check of its own outside the
gate, tagged `check-ecr` and run by hand with a live session as the
bolt's acceptance, since it needs a credential a person signs in for
(93, core.38). The ECR
check SHALL refuse to run, naming the setting it lacks, on a host that
names no ECR repository, rather than pass without running.

#### Scenario: zot

- **WHEN** the tools layer runs the store group against a zot started for the test
- **THEN** every scenario of the group passes

#### Scenario: The reference distribution

- **WHEN** the tools layer runs the store group against a reference distribution started for the test
- **THEN** every scenario of the group passes, attestations held through the referrers tag

#### Scenario: ECR, by hand, as the bolt's acceptance

- **WHEN** `moon run ':#check-ecr'` runs on a host whose `devenv.local.nix` names an ECR repository, with a live session for it
- **THEN** every scenario of the group passes against that repository, each run's names apart from every earlier run's

#### Scenario: No ECR repository named

- **WHEN** the ECR check starts on a host that names no ECR repository
- **THEN** it refuses to run naming the setting the host must give

### Requirement: What ECR answered is recorded

An ADR SHALL record what a push against a real ECR repository
answered on each open issue the profile rests on: timber's artifact
manifest and image index, a referrer with a subject, the referrers
list filtered by artifact type, a manifest put by digest alone, the
cache manifest, a media type with a parameter, and tag immutability
(ADR 0003, ADR 0014).

#### Scenario: The record read

- **WHEN** a reader opens the ADR
- **THEN** each open issue names what ECR answered, the date and the region, and what the profile does because of it

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

### Requirement: A write asks to pull and push its repository

Where a registry answers a write, a put, an upload or a mount, with a
`Bearer` challenge, the registry store SHALL ask the challenge's realm
for a token scoped to pull and push the repository the write is for,
whatever actions the challenge names. A read SHALL ask the scope the
challenge names (64, C.1).

#### Scenario: A write challenged for pull alone

- **WHEN** a registry answers a manifest put with a `Bearer` challenge whose scope names only `pull` on the repository
- **THEN** the store asks the realm for `repository:<repository>:pull,push`, and sends the put once more with the token answered

#### Scenario: A read challenged for pull

- **WHEN** a registry answers a tag read with a `Bearer` challenge whose scope names `pull` on the repository
- **THEN** the store asks the realm for the scope the challenge names

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

### Requirement: What GitHub's registry answered is recorded

A record SHALL say what GitHub's registry answered on each issue the
profile rests on: timber's artifact manifest and image index, a
referrer with a subject, the referrers API, a manifest put by digest
alone, the cache manifest, a write's token, a repository made on push
and its visibility, a cross-repository mount, and a tag read after its
put (C.1, ADR 0015).

#### Scenario: The record read

- **WHEN** a reader opens the record
- **THEN** each issue names what GitHub's registry answered, the date, and what the profile does because of it

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

### Requirement: A pool's push reads no mark

A push's store, which asks nothing of the repository's own, SHALL read
no mark and SHALL write its cache entries at a root left as at any root,
cache entries being no history and serving the builds of the revision
they came from (135, C.1).

#### Scenario: A push at the old root

- **WHEN** a push's store bound for `acme/app`, whose own holds a mark naming `acme/site`, puts a cache entry's bytes and names it
- **THEN** the entry is held in `acme/app`'s cache repository, and `moved` is never asked for

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
