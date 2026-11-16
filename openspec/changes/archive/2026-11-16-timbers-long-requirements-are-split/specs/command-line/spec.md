## MODIFIED Requirements

### Requirement: Status needs no seam

`timber status` SHALL answer in any checkout of a repository, bound to
the landing, the policy and the ledger alone, by the host's bindings
file or, where the host has none, as 118 binds a host with no file, and
SHALL ask no store, signer or target (60). It SHALL take `--branch` (by
default `main`) for the channels, `--component` to narrow both parts,
`--revision` to narrow the record's part, and `--json`.

#### Scenario: Status from the installed binary

- **WHEN** the built binary runs `timber status --json` in a repository whose host record holds one entry
- **THEN** it prints that entry's component, version, revision and index's digest in the record's part, the channels in theirs, the environments' part unreadable, and exits 0

#### Scenario: A broken bindings file

- **WHEN** the built binary runs `timber status --json` on a host whose bindings file is not well formed
- **THEN** the channels' part is unreadable naming the file with the parser's words, the record's part answers, and the exit is 0

#### Scenario: Status in no repository

- **WHEN** the built binary runs `timber status` in a directory that is no git repository
- **THEN** it exits 3 naming the directory

### Requirement: A host with no bindings file is bound to the defaults

A host with no bindings file SHALL be bound as 118 binds it, to the
given store, landing and its remote, ledger, signer and teller
(host-bindings), for `publish`, `verify`, `sync`, `push`, `rebuild`,
`promote` and `status` alike. Where the file is there, each
command SHALL be bound to the seams it names and the given ones for the
tables it leaves out, and render a broken file's unreadable answer in
its own form, as text or with `--json` (58, 59).

#### Scenario: Publish with nothing bound

- **WHEN** the built binary runs `timber publish HEAD` in a repository on a host with no bindings file, whose branch has not landed `HEAD`
- **THEN** it exits 1 refusing `HEAD` as not landed, and no tag, layout, key, record or bindings file is written

#### Scenario: Verify with nothing written

- **WHEN** the built binary runs `timber verify 0.1.0 --json` on a host with no bindings file and no given layout yet
- **THEN** it exits 3 unreadable naming the given layout's path, and no directory is made

#### Scenario: Several remotes and no origin

- **WHEN** the built binary runs `timber publish HEAD` on a host with no bindings file, in a checkout whose remotes are `github` and `gitlab`
- **THEN** it exits 3 naming both remotes and `[landing]` in the bindings file, and nothing is written

#### Scenario: A broken file as JSON

- **WHEN** the built binary runs `timber verify 0.1.0 --json` on a host whose bindings file is not well formed
- **THEN** it prints one JSON document saying unreadable, naming the file and carrying the parser's words, and exits 3

#### Scenario: Status with no file

- **WHEN** the built binary runs `timber status --json` on a host with no bindings file, in a repository whose host record holds one entry and whose ledger refs hold a promotion
- **THEN** it prints that entry and the promoted channel, and exits 0

### Requirement: Sync fills the checkout's cache from a landed revision

`timber sync [revision]` SHALL take the revision whose verdicts name
the entries and whose policy judges the layouts, by default the newest
revision of the branch the checkout descends from, `--branch` (by
default `main`) and `--json`; it SHALL be bound as verify is, by the
host's bindings file or the defaults where it has none, trusting the
host's own signer and the signers the file trusts, never making the
store it is bound to (118).

#### Scenario: A sync with one entry gone
- **WHEN** `timber sync --json` runs over the fakes for a landed revision whose verdicts name two build identities, one of which the store holds no entry for
- **THEN** the printed JSON names one as rendered and the other as absent, and the exit is 0

#### Scenario: A sync on an unbound host

- **WHEN** the built binary runs `timber sync --json` on a host with no bindings file and no given layout yet
- **THEN** it answers unreadable naming the given layout's path, exits 3, and writes no cache, layout or bindings file

#### Scenario: A sync with a version refused
- **WHEN** `timber sync --json` runs over the fakes in a checkout whose lockfile pins one version the policy admits, one it refuses and one the store does not name
- **THEN** the printed JSON is the core's answer value for value, naming the first held, the second refused with its version and the claim found wanting, and the third absent, and the exit is 1

#### Scenario: A sync whose layout cannot be refreshed
- **WHEN** `timber sync` runs over the fakes while the store's resolve is unreadable with the words `connection refused`, in a checkout whose layout was rendered before
- **THEN** it prints what the layout holds and when it was rendered, prints `connection refused` verbatim, and exits 3

### Requirement: Promote takes a version and a channel

`timber promote <version> --to <channel>` SHALL take the version,
bare of its component's prefix, the channel, `--from <version>` where
the asker expects the channel at a version, `--component` (needed only
where more than one is declared), `--branch` (by default `main`) and
`--json`; `timber promote --back --to <channel>` SHALL take no version
and promote the version the channel's head moved from.

#### Scenario: A promotion as JSON

- **WHEN** `timber promote 0.2.0 --to stable --json` runs over the fakes for an accepted version
- **THEN** the printed JSON is the core's answer value for value, naming the claim's digest, and the exit is 0

#### Scenario: A stale start

- **WHEN** `timber promote 0.3.0 --to stable --from 0.1.0` runs while `stable` is at 0.2.0
- **THEN** it exits 1 naming 0.2.0 as where `stable` is

#### Scenario: A version and --back together

- **WHEN** `timber promote 0.2.0 --back --to stable` is run
- **THEN** it exits 2 naming the arguments it cannot take together, and the core is not asked

#### Scenario: A promote with no ledger or signing written

- **WHEN** `timber promote` runs on a host whose file names no `[signing]`, no `[landing]` and no `[ledger]`, in a checkout whose one remote is `origin`
- **THEN** the promotion is signed by the given host key as the host's name and pushed to `refs/timber/ledger/<channel>` on `origin`

### Requirement: Policy shows the policy in force

`timber policy` SHALL take `--branch` (by default `main`) and
`--json`, and SHALL print, at the branch's landed head, the policy for
acceptance and for a layout and every channel the channels answer
lists, the givens beside the repository's own, each with its component,
how it moves, whether a following channel takes prereleases, the claims
its policy requires, its promoters and who may ask for it (C.10, 131,
82).

#### Scenario: The givens beside a line

- **WHEN** `timber policy --json` runs in a repository whose landed head holds a policy file with `[acceptance]` and `[channels.stable]` requiring a rebuild
- **THEN** the printed JSON is the core's answer value for value, listing `beta` moving on each publish with releases alone and `stable` bound to `beta` requiring the gate's acceptance and then the rebuild, and the exit is 0

#### Scenario: The givens held off, beneath the table

- **WHEN** `timber policy` runs at a head whose file writes no channel and declares no component
- **THEN** beneath `beta` and `stable` it shows `alpha`, each publish with prereleases too, switched on by `[channels.alpha]`, and `lts`, bound to `stable`, switched on by `[channels.lts]`, and with `--json` the same channels answer, held givens and all, value for value with the core's

#### Scenario: A given switched on leaves the held part

- **WHEN** `timber policy` runs at a head whose file holds `[channels.alpha]`
- **THEN** `alpha` is shown among the channels in force and not beneath them, and `lts` alone is shown held off

#### Scenario: alpha and its approvers shown

- **WHEN** `timber policy` runs at a head whose file holds `[channels.alpha]` and `[channels.stable]` naming `approvers = ["ci"]`
- **THEN** it shows `alpha` moving on each publish with prereleases, and `stable` with `ci` as who may ask

#### Scenario: No layout policy

- **WHEN** `timber policy` runs at a head whose file holds no `[layout]`
- **THEN** the layout is shown absent naming the audience, the channels are shown, and the exit is 0

#### Scenario: A head not landed

- **WHEN** the landing answers that the branch has not landed its head
- **THEN** it exits 1 naming the branch and the revision, and the policy is not asked

### Requirement: Publish tells the host's gate where the host binds it to

Where the bindings file binds `[gate] publish = {}`, or holds no
`[gate]` and so binds the host's default (118), `timber publish`
SHALL, after the core's publish has answered, tell the gate of each
version the answer offered, in the same run: the component, the version
and the index digest the answer names, told by a publish, the gate
signing as the host's `[signing]` signer (95, 108, 130).

#### Scenario: One host, beta moves in the same run

- **WHEN** the policy lets the host's signer claim both provenance and acceptance, `beta` follows each publish and requires acceptance from that signer, the file binds `[gate] publish = {}` and a ledger, and `timber publish` offers version 0.2.0
- **THEN** the same run prints 0.2.0 accepted by the host, the acceptance saying the host accepted what it offered, and `beta` promoted to 0.2.0's index; the ledger's chain for `beta` holds the promotion; and the exit code is 0

#### Scenario: A file with no gate table tells the gate the same way

- **WHEN** the policy and the file are as in the scenario above, except that the file holds no `[gate]`
- **THEN** the run's answer, its exit code, the acceptance attached and `beta`'s chain are the same as with `[gate] publish = {}`

#### Scenario: Two landings, beta follows both

- **WHEN** two revisions land one after the other and `timber publish` runs after each on that one host
- **THEN** `beta` is at the second version, its chain holds two promotions, the second naming the first as its predecessor, and no other process ran

#### Scenario: The gate refuses what the host offered

- **WHEN** the file binds `[gate] publish = {}` and the policy does not let the host's signer claim acceptance
- **THEN** the version is offered, the gate's answer refuses it naming the first artifact and the claim found wanting, nothing is attached, `beta` does not move, and the exit code is that of a refusal

#### Scenario: A channel held

- **WHEN** the gate accepts the version and `beta`'s own policy requires a claim the version does not carry
- **THEN** the version is accepted, the answer for `beta` is held naming the claim, and the exit code is that of a refusal

#### Scenario: Accepted with no ledger bound

- **WHEN** the file binds `[gate] publish = {}` and no `[ledger]`, and `beta` follows each publish
- **THEN** the version is accepted and the answer for `beta` is absent naming `[ledger]` in the file

#### Scenario: Unchanged is not told

- **WHEN** the publish answers a component unchanged since its last release
- **THEN** the gate is not asked, and that component's answer carries no gate answer

#### Scenario: The JSON answer carries the gate's

- **WHEN** `timber publish --json` tells the gate of 0.2.0 and the gate accepts it
- **THEN** the component's entry holds the publish's answer as before and a `gate` member holding the gate's answer, value for value as the core's gate answers it

### Requirement: Sync makes a served layout reachable

After the layouts are rendered, `timber sync` SHALL answer, for each
layout whose projection serves it, the address its clients read it at,
and, where there is one, SHALL name the checkout's repository to the
host and find the host's serving process at the core's address or start
it where none answers, as `timber status --page` does, and answer that
it serves or why it cannot (114, 132). A layout whose projection serves
none SHALL name no address and start nothing.

#### Scenario: A sync of an npm layout
- **WHEN** `timber sync --json` runs in a checkout of `agentplot/consumer` whose npm projection rendered a layout, with no process answering at the address
- **THEN** the answer names the layout's address `http://127.0.0.1:7420/agentplot/consumer/npm/`, the host's file names the checkout for `agentplot/consumer`, one process was started, and a second sync starts none

#### Scenario: A sync with no served layout
- **WHEN** `timber sync` runs in a checkout whose only rendered layouts are of projections that serve none
- **THEN** the answer names no address, the host's file is unchanged, and no process is started

#### Scenario: A remote that names no owner
- **WHEN** `timber sync` renders an npm layout in a checkout whose landing remote is `/srv/consumer.git`
- **THEN** the layout is rendered, the answer is unreadable naming the remote and its address in place of the address, and the exit is 3

## ADDED Requirements

### Requirement: Status answers each part on its own, and exits 0 whenever it answers

A binding status cannot make SHALL be the channels' answer, in its own
form, and the record's part SHALL still answer. Status SHALL exit 0
whenever it answers, each part carrying its own answer, and exit with
the code of its answer only where it cannot read the repository at all.

#### Scenario: A ledger that cannot be bound

- **WHEN** `timber status --json` runs on a host whose bindings file names a `[ledger]` that cannot be bound, in a repository whose host record holds one entry
- **THEN** the channels' part is that binding's answer, the record's part names the entry, and the exit is 0

### Requirement: A default that cannot be made binds nothing

The installed binary SHALL answer absent or unreadable for the binding
only where a default cannot be made: absent naming `XDG_CONFIG_HOME`
and `HOME` where neither is set, and unreadable naming the repository's
remotes and `[landing]` in the bindings file where the checkout has
several and none is `origin`; it SHALL then offer, read or write
nothing (118).

#### Scenario: Neither home set

- **WHEN** the built binary runs `timber publish HEAD` with neither `XDG_CONFIG_HOME` nor `HOME` set
- **THEN** it answers absent naming both, and offers, reads and writes nothing

### Requirement: Sync's answer and its exit

Sync SHALL answer, as one answer of the core, each build identity as
rendered, absent, or the refusal or unreadable answer its render gave,
and each layout as refreshed, naming every pin held, refused with its
claim found wanting, or absent, or as what it still holds and when it
was rendered beside what could not be read. It SHALL exit 0 when the
only entries and pins not rendered or held are absent, and otherwise
with the worst code among them (51, 54, 57, 99).

#### Scenario: A sync whose misses are all absent

- **WHEN** `timber sync` runs over the fakes for a revision whose verdicts name one build identity the store holds no entry for, in a checkout whose lockfile pins one version the store names nothing of
- **THEN** it answers the entry and the pin absent, and exits 0

### Requirement: Promote is bound by the host's signing and ledger, or the givens

`timber promote` SHALL be bound by the host's bindings file, its
`[signing]` and `[ledger]` or, for each it leaves out or where there is
no file, the given host key and the git ref on the landing's remote, or
in the repository where the landing has none (118), never making the
store it is bound to.

#### Scenario: A promote with a ledger and a signer written

- **WHEN** `timber promote` runs on a host whose file names a `[signing]` signer and a `[ledger]`
- **THEN** the promotion is signed by that signer and appended to that ledger, and the store it is bound to is not made

### Requirement: Promote prints the move

`timber promote` SHALL print the channel, the version and index moved to
and from, the promotion claim's digest and its predecessor's, and how
the store followed, with the store's own answer beside where it was not
written (33, 57).

#### Scenario: A move the store did not follow

- **WHEN** `timber promote 0.2.0 --to stable` moves `stable` from 0.1.0 over the fakes, and the store's attach is unreadable
- **THEN** it prints `stable`, 0.2.0 and 0.1.0 with their indexes, the claim's digest and its predecessor's, and the store's unreadable answer beside

### Requirement: Policy shows the givens held off beneath the table

Beneath the table of channels, `timber policy` SHALL print each given
held off, with how it would move and the line that would switch it on,
as the same channels answer carries them apart from the channels in
force, so a reader sees what one line would give them (C.10 as amended
at boomdrivedev/blueprints b1267d7, 131 as amended at 1ae1cc5).

#### Scenario: One component's givens held off

- **WHEN** `timber policy` runs at a head whose file declares `lib` and `web` and holds `[channels.lib.alpha]`
- **THEN** beneath the table it shows `alpha` of `web`, switched on by `[channels.web.alpha]`, and `lts` of each with its component's line

### Requirement: Each part of policy's answer is its own

Each part of `timber policy`'s answer SHALL carry its own answer, so an
absent layout, a channel whose policy cannot be read, or an unreadable
channels answer leaves the rest shown, the last shown once in place of
the channels and the givens held off (58).

#### Scenario: An unreadable channels answer

- **WHEN** `timber policy` runs at a head whose channels answer is unreadable
- **THEN** that answer is shown once in place of the channels and the givens held off, and the policy for acceptance and for a layout is shown

### Requirement: Policy is bound as status is, writes nothing, and exits 0 when it answers

`timber policy` SHALL be bound as status is, asking the landing and the
policy alone, and SHALL write nothing (60, core.32). It SHALL exit 0
whenever an answer is made, and with the answer's own code where the
branch's head cannot be resolved or has not landed.

#### Scenario: Policy asks nothing else

- **WHEN** `timber policy` runs on a host whose bindings file names a store and a signer
- **THEN** it asks neither, writes nothing, and exits 0

### Requirement: The core's publish reaches nothing of the gate

The core's publish SHALL reach nothing of the gate: the command calls
the gate after the publish has answered (130, core.10).

#### Scenario: The gate asked after the publish answers

- **WHEN** `timber publish` runs over the fakes on a host whose file binds `[gate] publish = {}`
- **THEN** the core's publish answers before the gate is asked anything, and asks nothing of the gate itself

### Requirement: The gate's answer is printed beside the publish's, and counts in the exit

The gate's answer for each version SHALL be printed beside that
version's, and in the JSON answer SHALL be the version's `gate`, in the
core's one form (57). The exit code SHALL be the worst of each
component's publish answer and of the gate's, a version the gate
refused or a following channel it held counting as refused.

#### Scenario: The gate refuses one of two components

- **WHEN** `timber publish` offers `lib` and `web`, and the gate accepts `lib` and refuses `web`
- **THEN** each gate answer is printed beside its version, and the exit code is that of a refusal

### Requirement: A remote that names no owner and name is answered beside the layouts

Where the checkout's landing remote names no owner and name, `timber
sync` SHALL answer unreadable naming the remote and its address beside
the layouts, and name and start nothing. The exit SHALL count that
answer as it counts a layout's (57).

#### Scenario: Nothing named or started

- **WHEN** `timber sync` renders an npm layout in a checkout whose landing remote is `/srv/consumer.git`, and no process answers at the core's address
- **THEN** the host's file is unchanged, and no process is started

### Requirement: A component the publish did not offer tells the gate nothing

A component the publish left unchanged, or answered with absent,
refused or unreadable, SHALL tell the gate nothing (95, 130).

#### Scenario: A refused component is not told

- **WHEN** `timber publish` offers `lib` and answers `web` refused, on a host whose file binds `[gate] publish = {}`
- **THEN** the gate is told of `lib` alone, and `web`'s answer carries no gate answer
