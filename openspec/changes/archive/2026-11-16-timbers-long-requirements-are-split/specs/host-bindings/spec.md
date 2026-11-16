## MODIFIED Requirements

### Requirement: The file names the store, the landing and the signer

The file SHALL bind the store, the landing and the signer for the host,
and nothing the repository's authors bind (92): `[store]` naming a
layout by its path (C.2) or a registry by its repository (C.1);
`[landing]` naming sluice or a plain branch,
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

- **WHEN** the file names `[store] registry` with a repository such as `localhost:5000/timber`
- **THEN** publish writes into that repository and verify reads from it, neither makes a repository, and the file names no credential

#### Scenario: A registry named by something that is not a repository

- **WHEN** `[store] registry` names a value that is not a registry host followed by a repository path
- **THEN** publish and verify answer unreadable, naming the file, `[store] registry` and the value, and nothing is read or written

#### Scenario: A plain branch

- **WHEN** the file names `[landing] branch`
- **THEN** a revision is landed when the branch's history holds it, and publish attaches no verdicts (78, C.6)

#### Scenario: A relative path

- **WHEN** the file names a layout or a key by a path that is neither absolute nor begins with `~/`
- **THEN** publish and verify answer unreadable, naming the file and the key, and nothing is read or written

### Requirement: The repository binds the build and the projections

The binding SHALL take the build, the projections and the targets from
the repository, not the host (92): moon is bound as the build, its tasks
named per output in the policy file at the landed revision (C.3), and
the file and image projections are bound by their ecosystem names, the
policy file naming each output's (C.10, C.11). The policy seam SHALL be
bound to the policy file at the landed revision (C.10).

#### Scenario: The bindings file names no build

- **WHEN** the bindings file names a store, a landing and a signer and nothing else
- **THEN** publish asks moon to ensure the outputs the policy file declares, and packs each with the projection the policy file names for it

#### Scenario: A build named in the host's file

- **WHEN** the bindings file holds a `[build]` table
- **THEN** publish and verify answer unreadable naming the table, since the build is the repository's to bind

#### Scenario: The targets are the repository's

- **WHEN** the bindings file names a store, a landing and a signer and nothing else, and `timber apply` runs
- **THEN** the binding holds the target `nix_cache`, reading through a store binding of its own that is not made

### Requirement: The host trusts its own signer

For reading, the host SHALL trust its own signer, the one `[signing]`
names, verified by what its profile verifies it by: a host key by its
key's public half, the file beside the key that ssh-keygen names
`<key>.pub`, read when sync binds and never the key itself; a key in a
key service by its key ARN, which is its name. So a host consumes what
it published itself (118, C.10).

#### Scenario: The host's own signer trusted

- **WHEN** the file names `[signing]` for the signer `laptop` with a key whose public half lies beside it, and `[trust.gate]`
- **THEN** a sync's binding trusts `laptop`, verified by that public half, then `gate`, neither with a claim of its own, and the key itself was not read

#### Scenario: A key service's own signer trusted

- **WHEN** the file names `[signing] kms` with a key ARN and no `[trust]`
- **THEN** a sync's binding trusts that ARN's signer, verified by the ARN, with no claim of its own, and nothing is read or reached

#### Scenario: No trust named

- **WHEN** the file names `[store]`, `[landing]` and `[signing]` and no `[trust]`
- **THEN** a sync's binding trusts the host's own signer alone, and publish and verify answer as before

#### Scenario: Neither signing nor trust named

- **WHEN** the file names `[store]` and `[landing]`, no `[signing]` and no `[trust]`
- **THEN** a sync's binding trusts no signer

### Requirement: Every host binds every signer profile for verify

The binding SHALL bind both signer profiles on every host: the one
`[signing]` names first, signing as its signer, and the other verify-only,
so a claim made by either kind of signer is told on any host whatever it
signs with (80, 92). A host whose file names no `[signing]` SHALL bind
the given host key first, signing as the host's name, for publish and
promote, and both profiles verify-only for every other command.

#### Scenario: A host key host

- **WHEN** a host whose `[signing]` names a host key is bound for verify
- **THEN** the signers bound are the host key, signing as its signer, and then the key service signer, verify-only

#### Scenario: A key service host

- **WHEN** a host whose `[signing]` names a KMS key ARN is bound for publish
- **THEN** the binding's signer is that ARN, and the signers bound are the key service signer bound to that key, and then the host key, verify-only

#### Scenario: A host that names no signer, publishing

- **WHEN** a host whose file names no `[signing]` is bound for publish
- **THEN** the binding's signer is the host's name, and the signers bound are the given host key, signing as it, and then the key service signer, verify-only, and no key is read or made in binding them

#### Scenario: A host that signs nothing

- **WHEN** a host whose file names no `[signing]` is bound for verify
- **THEN** both signer profiles are bound verify-only, and nothing is read, made or reached in binding them

#### Scenario: A key service host asking status

- **WHEN** a host whose `[signing]` names a KMS key ARN is bound for status
- **THEN** both signer profiles are bound verify-only, the key service profile reaching no service, and the binding names no signer

### Requirement: The file names the ledger

The file SHALL bind the ledger for the host with `[ledger]`, naming its
profile by its one key (92, 118): `git_ref`, with an optional `remote`
holding the refs, the repository the command runs in holding them where
none is named (C.13); or `store`, holding the chain in the store the
host binds, through a binding of its own that never makes the store
(C.14). `[ledger]` naming no profile, two, or an option its profile does
not read SHALL be unreadable naming the table.

#### Scenario: A git ref ledger with a remote

- **WHEN** the file names `[ledger] git_ref = { remote = "origin" }`
- **THEN** a promotion pushes its claim to `refs/timber/ledger/<channel>` on `origin`, and the file names no credential

#### Scenario: A store ledger

- **WHEN** the file names `[store] layout` and `[ledger] store = {}`
- **THEN** a promotion's chain is walked from the channel's name in that layout, and the layout is not made by the ledger's binding

#### Scenario: A store ledger over the given layout

- **WHEN** the file names `[ledger] store = {}` and no `[store]`
- **THEN** the chain is held in the given layout, and the ledger's binding does not make it

#### Scenario: A ledger table naming two profiles

- **WHEN** `[ledger]` names both `git_ref` and `store`
- **THEN** every command bound by the file answers unreadable naming `[ledger]`, and nothing is read or written

#### Scenario: No ledger table

- **WHEN** the file names no `[ledger]` and `timber promote` runs, the landing bound to the remote `origin`
- **THEN** the promotion is pushed to `refs/timber/ledger/<channel>` on `origin`, after the chain is fetched from it

#### Scenario: Targets named in the host's file

- **WHEN** the bindings file holds a `[targets]` table
- **THEN** every command bound by the file answers unreadable naming `[targets]`, since the targets are the repository's to bind

### Requirement: Status is bound with nothing written

Where the host holds no bindings file, status SHALL be bound as 118
binds such a host, of what status asks: the landing, sluice where the
repository's own tree holds `.config/sluice.toml` and the plain branch
otherwise, and the git ref ledger, each with the remote `origin` where
the repository has one and no remote where it has none; the policy from
the repository; the given layout as the store; and the signer profiles
verify-only, reaching no service (60, 92, 118,
C.5, C.6, C.13).

#### Scenario: A clone with no bindings file

- **WHEN** status is bound on a host with no bindings file, in a clone with a remote `origin`
- **THEN** the landing is the plain branch read from `origin`, the ledger the git ref fetched from `origin`, and no file or directory is made

#### Scenario: A repository set up for sluice

- **WHEN** the repository's tree holds `.config/sluice.toml` and the host has no bindings file
- **THEN** status's landing is sluice's

#### Scenario: A host that never published

- **WHEN** status is bound on a host with no bindings file whose given layout is not there
- **THEN** the binding is made, the store answers every operation unreadable naming the layout, and no layout is made

### Requirement: Status needs only what it asks

Where the host's bindings file is there, status SHALL be bound to its
`[landing]`, its `[store]` or the given layout, opened for the
environments' reports and for a store ledger and never made, and, where
named, its `[ledger]`; it SHALL need no `[signing]`, and a file naming
no `[ledger]` SHALL bind a ledger whose every chain is absent naming
`[ledger]` in the file, so each channel is shown absent naming it (60,
118).

#### Scenario: A file naming a landing and no ledger

- **WHEN** status is bound by a file naming `[landing]` and no `[ledger]`, `[store]` or `[signing]`
- **THEN** the binding is made, and each channel's chain is absent naming `[ledger]` in the file

#### Scenario: A store that cannot be opened

- **WHEN** status is bound by a file whose `[store]` names a layout that is not there and whose `[ledger]` is the git ref
- **THEN** the binding is made, the channels are read from the git ref, and the store answers every operation unreadable naming the layout

### Requirement: The file names what tells the gate

The bindings file SHALL name what tells the host's gate of an offered
version in a `[gate]` table, by one key, in the form every other table
of the file takes (118, 130, C.16): `publish = {}`, the host's own
publish step tells the host's own gate; or `job = "<words>"`, a job
outside timber, woken by a store's or a forge's event, tells a gate
elsewhere, and the words say which. The core SHALL name neither (130).
timber SHALL run nothing for `job`, and SHALL carry its words as
written.

#### Scenario: The host's publish tells its own gate

- **WHEN** the bindings file holds `[gate]` with `publish = {}`
- **THEN** the binding says the host's publish step tells the host's gate

#### Scenario: A job tells a separated gate

- **WHEN** the bindings file holds `[gate]` with `job = "the release job a push to main wakes"`
- **THEN** the binding says a job tells the gate, carrying those words as written, and nothing of timber starts or reaches that job

#### Scenario: Two tellers

- **WHEN** `[gate]` holds both `publish = {}` and `job = "…"`
- **THEN** publish and verify answer unreadable naming `[gate]` and both keys, and the layout the file names is not made

#### Scenario: A teller the file does not know

- **WHEN** `[gate]` holds `webhook = "…"`
- **THEN** the answer is unreadable naming `[gate]`, `webhook`, and the keys the table reads, `publish` and `job`

#### Scenario: A publish teller with options

- **WHEN** `[gate]` holds `publish = { now = true }`
- **THEN** the answer is unreadable naming `[gate] publish`, which takes no options

#### Scenario: A job with no words

- **WHEN** `[gate]` holds `job = ""`
- **THEN** the answer is unreadable naming `[gate] job`

### Requirement: A table the file leaves out is bound to its given profile

Each of `[store]`, `[landing]`, `[ledger]`, `[signing]` and `[gate]` the
file names SHALL bind what it names, exactly as written; each it leaves
out SHALL bind its given binding (118): the store the given layout
(C.2); the ledger the git ref on the
landing's remote, written or given, or the repository's own refs where
it has none (C.13); the signer the given host key at
`timber/host/signing.key` under that layout's data directory (C.8); the gate
told by the host's own publish (130).

#### Scenario: A registry and nothing else

- **WHEN** the file names `[store] registry` and no other table
- **THEN** publish writes into that registry, reads the landing the repository is set up for, signs with the given host key, tells the host's own gate, and the gate's promotions are appended to `refs/timber/ledger/...` on the given landing remote

#### Scenario: A key service and nothing else

- **WHEN** the file names `[signing] kms` and no other table
- **THEN** publish signs through that key and writes into the given layout, and no host key is made

#### Scenario: The data directory set

- **WHEN** `XDG_DATA_HOME` names a directory and the file names no `[store]` and no `[signing]`
- **THEN** the store is the layout at `timber/layout` under that directory, and a publish's given host key is `timber/host/signing.key` under it

#### Scenario: A promote with no ledger table

- **WHEN** the file names no `[ledger]` and `timber promote` runs in a checkout whose one remote is `origin`
- **THEN** the channel's chain is fetched from `origin` and the promotion is pushed to the channel's git ref there, fast-forward only

#### Scenario: A checkout with no remote and no ledger table

- **WHEN** the file names no `[ledger]` and `timber promote` runs in a checkout with no remote
- **THEN** the promotion is appended to the channel's git ref in the repository the command runs in, and nothing is pushed

#### Scenario: A written landing with no remote

- **WHEN** the file names `[landing] branch = {}` and no `[ledger]`, in a checkout whose remote is `origin`
- **THEN** the landing reads the repository's own branches, the ledger is the repository's own refs, and `origin` is not read

#### Scenario: A written landing's remote carries the given ledger

- **WHEN** the file names a `[landing]` whose `remote` is `upstream`, and no `[ledger]`
- **THEN** each channel's chain is fetched from and pushed to `upstream`, never `origin`

## ADDED Requirements

### Requirement: `[signing]` names a host key or a key service's key

`[signing]` SHALL name either the host's signer and the path of its key
file, in sluice's `[signing]` shape (C.8), or a key in a key service by
its key ARN alone, `kms = "<key ARN>"`, whose ARN is the signer's name
(C.7).

#### Scenario: A host key's signing table

- **WHEN** the file's `[signing]` names the signer `laptop` and a key path beginning `~/`
- **THEN** publish signs as `laptop` with the key at that path under the home directory

### Requirement: Every target profile is bound by its name

Every target profile built SHALL be bound by its profile name, each
environment's `target` in the policy file picking among them (C.10,
C.12); a target that reads the store SHALL read it through a store
binding of its own, opened and never made.

#### Scenario: An environment picking its target

- **WHEN** the policy file's environment `fleet` names `target.clan = { machine = "m1", closure = "system" }`
- **THEN** an apply of `fleet` is handed to the target profile bound as `clan`

### Requirement: Sync alone carries the trusted signers

The binding SHALL carry the trusted signers to sync alone, the host's
own before those `[trust]` names; a file naming no `[signing]` and no
`[trust]` SHALL trust no signer (118).

#### Scenario: Trust carried to sync alone

- **WHEN** the file names `[signing]` for the signer `laptop` and `[trust.gate]`, and verify and sync are bound
- **THEN** verify's binding carries no trusted signer, and sync's carries `laptop` and then `gate`

### Requirement: Status binds both signer profiles verify-only

For status, both signer profiles SHALL be bound verify-only and the key
service profile bound to reach no service, whatever `[signing]` names,
since status reaches no signer (60).

#### Scenario: A host key host asking status

- **WHEN** a host whose `[signing]` names a host key is bound for status
- **THEN** both signer profiles are bound verify-only, and the binding names no signer

### Requirement: A file with no ledger table binds the git ref on the landing's remote

A file that names no `[ledger]` SHALL bind the git ref on the remote the
landing is bound to, written or given, or with no remote where the
landing has none (118).

#### Scenario: No ledger table, the landing on upstream

- **WHEN** the file names no `[ledger]`, and its `[landing]` names the remote `upstream`
- **THEN** the ledger is the git ref on `upstream`

### Requirement: The targets are not the host's file's to name

A `[targets]` table, or any table naming a target, SHALL be unreadable
naming it, since the targets are the repository's to bind (92).

#### Scenario: A table naming a target profile

- **WHEN** the bindings file holds a table naming the target profile `nix_cache`
- **THEN** every command bound by the file answers unreadable naming that table

### Requirement: Status's given layout is opened and never made

Where the host holds no bindings file, status SHALL be bound to the
given layout as the store, opened and never made, and a layout that is
not there SHALL be bound as a store whose every operation answers unreadable with the
opening's words. Status SHALL write nothing to bind the given bindings
(60).

#### Scenario: A clone with no remote and no layout

- **WHEN** status is bound on a host with no bindings file and no given layout, in a clone with no remote
- **THEN** the landing is the plain branch and the ledger the git ref, each with no remote, and no file or directory is made

### Requirement: A store status cannot open answers unreadable

A store status cannot open SHALL be bound as a store whose every
operation answers unreadable with the opening's words, so the channels
are read and each environment is answered from its copy (61).

#### Scenario: Environments answered from the copy

- **WHEN** status is bound by a file whose `[store]` names a layout that is not there, and the host's copy holds an environment's reports
- **THEN** that environment is answered from the copy, and the channels are read

### Requirement: A gate table timber cannot read is unreadable

A `[gate]` naming no key, two keys, a key other than these, a `publish`
that is not an empty table, or a `job` that is not a string with words
in it SHALL be unreadable naming the file and the table, and SHALL
leave nothing written for every command that reads the file (58, 59,
core.19).

#### Scenario: A gate table naming no key

- **WHEN** `[gate]` holds no key
- **THEN** publish and verify answer unreadable naming the file and `[gate]`, and nothing is written

### Requirement: The given store is the layout in the data directory

Where the file leaves out `[store]`, the store SHALL be the layout at
`$XDG_DATA_HOME/timber/layout`, or `~/.local/share/timber/layout` (C.2).

#### Scenario: No data directory set

- **WHEN** `XDG_DATA_HOME` is unset and the file names no `[store]`
- **THEN** the store is the layout at `~/.local/share/timber/layout`
