## ADDED Requirements

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

## MODIFIED Requirements

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
