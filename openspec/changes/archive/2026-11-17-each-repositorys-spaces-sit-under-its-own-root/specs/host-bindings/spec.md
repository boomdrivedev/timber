## ADDED Requirements

### Requirement: A registry store is bound for the checkout's repository

Every command SHALL bind a registry store for the repository the
checkout is of, by the owner and name the address of its landing's
remote ends in, the landing as the file names it or as given, as the
serving process names the repository (114, 118, C.1). Where that
answer is absent or unreadable, the store SHALL be bound all the same
and SHALL answer it for every write and every name of the repository's
own spaces.

#### Scenario: A checkout of a forge's repository

- **WHEN** the file names `[store] registry = "<host>/store"` and `[landing] branch = { remote = "origin" }`, the checkout's `origin` is `https://github.com/Acme/Site.git`, and publish puts bytes
- **THEN** they are put into the repository `store/acme/site`

#### Scenario: A push's cache entries

- **WHEN** `timber push` puts a cache entry's bytes in the same checkout, with the moon build profile bound
- **THEN** they are put into the repository `store/acme/site/cache/moon`

#### Scenario: A check job's file with no landing table

- **WHEN** `timber push --left` binds a file naming `[store]` alone in a checkout whose `origin` is `https://github.com/acme/site`
- **THEN** its entries are put into the repository `store/acme/site/cache/moon`

#### Scenario: A checkout whose landing names no remote

- **WHEN** the file names `[store] registry` and `[landing] branch = {}`, and sync resolves an artifact's name and publish puts bytes
- **THEN** sync reads the name from `store/artifact/…`, and the put answers absent naming the landing's remote, with nothing written

## MODIFIED Requirements

### Requirement: The file names the store, the landing and the signer

The file SHALL bind the store, the landing and the signer for the host,
and nothing the repository's authors bind (92): `[store]` naming a
layout by its path (C.2) or a registry by its host and root,
`<host>/<root>` (C.1); `[landing]` naming sluice or a plain branch,
each with an optional remote (C.5, C.6); and `[signing]` naming the
signer, as "`[signing]` names a host key or a key service's key" says.
A path SHALL be absolute or begin with `~/`.

#### Scenario: A layout, sluice and a host key

- **WHEN** the file names `[store] layout`, `[landing] sluice` with remote `origin`, and `[signing]` with a signer and a key path beginning `~/`
- **THEN** publish writes into that layout, reads sluice's records from `origin`, and signs as that signer with the key under the home directory

#### Scenario: A key in a key service

- **WHEN** the file's `[signing]` names `kms` with a KMS key ARN and nothing else
- **THEN** publish signs as that ARN through the key service signer, and the file names no credential

#### Scenario: A key service key beside a host key

- **WHEN** the file's `[signing]` names `kms` together with `signer` or `key`
- **THEN** publish and verify answer unreadable, naming the file and `[signing]`, and saying a key service signer's name is its key's and the table names `kms` alone

#### Scenario: A registry

- **WHEN** the file names `[store] registry` with a host and a root such as `localhost:5000/timber`
- **THEN** publish writes under that root and verify reads from under it, neither makes a repository, and the file names no credential

#### Scenario: A registry named by something that is not a repository

- **WHEN** `[store] registry` names a value that is not a registry host followed by a root's path
- **THEN** publish and verify answer unreadable, naming the file, `[store] registry` and the value, and nothing is read or written

#### Scenario: A plain branch

- **WHEN** the file names `[landing] branch`
- **THEN** a revision is landed when the branch's history holds it, and publish attaches no verdicts (78, C.6)

#### Scenario: A relative path

- **WHEN** the file names a layout or a key by a path that is neither absolute nor begins with `~/`
- **THEN** publish and verify answer unreadable, naming the file and the key, and nothing is read or written

### Requirement: A push before the landing binds with a store alone

`timber push --left` SHALL bind from the host's file with `[store]`
alone: a file naming no `[landing]` and no `[signing]` SHALL bind, the
landing answering absent naming its binding for every operation, and a
registry store bound for the repository the given landing remote
names; it SHALL make the layout the file names as push does. Every
other command SHALL still answer absent naming `[landing]` where the
file names none.

#### Scenario: A check job's file

- **WHEN** the file names `[store] registry` and nothing else, and `timber push --left` runs
- **THEN** the push binds and writes the checkout's entries into the cache repository under the root and the owner and name of the checkout's `origin`, and the file names no credential

#### Scenario: The same file for a publish

- **WHEN** the same file is read for `timber publish`
- **THEN** publish answers absent naming `[signing]`, and nothing is read or written
