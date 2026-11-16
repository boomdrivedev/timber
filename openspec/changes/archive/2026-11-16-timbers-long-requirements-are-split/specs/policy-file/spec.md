## MODIFIED Requirements

### Requirement: An audience with no table is absent

Where the revision holds no policy file, or its file holds no table for
acceptance or for a channel asked, the answer SHALL be absent naming
the audience (82, 58), except for a channel the givens declare, a held
one the repository has switched on, or one an environment's words make,
whose policy SHALL be answered as given (C.10). A file that cannot be
read or parsed SHALL be unreadable naming the file and the revision,
with git's or the parser's own words (59).

#### Scenario: No layout policy

- **WHEN** the file holds `[acceptance]` and no `[layout]`
- **THEN** the policy for a layout is the given one, marked as given, never absent

#### Scenario: No policy for a channel

- **WHEN** the file holds `[acceptance]` and no `[channels.nightly]`
- **THEN** the policy for the channel `nightly` is absent, naming the audience

#### Scenario: No policy file

- **WHEN** the revision's tree holds no `.config/timber.toml`
- **THEN** the policy for a layout is absent, naming the audience

#### Scenario: A malformed file

- **WHEN** the file at the revision is not well formed
- **THEN** the answer is unreadable, naming the file and the revision, with the parser's words verbatim

#### Scenario: A given channel with no line

- **WHEN** the file holds no `[channels]` and the policy for `stable` is asked
- **THEN** it answers the gate's acceptance as the one claim required, `beta` as the channel it is bound to, the gate's signers as its promoters, and no approvers

#### Scenario: A channel neither given nor declared

- **WHEN** the policy for `edge` is asked and neither the file nor the fragment declares it and no environment makes it
- **THEN** the answer is absent naming the channel

### Requirement: The profile lists the channels the file declares

The profile SHALL answer channels with every channel the table in force
lists, the givens of every component among them, and each channel an
environment's words make, once each, ordered by name and then
component, each with its component and how it moves (131, C.10). Where
the revision holds no policy file the answer SHALL be the givens the
fragment declares, of the one component.

#### Scenario: Two components' channels

- **WHEN** the file declares `lib` and `web`, `[channels.lib.next]` bound to `beta`, and `[channels.web.edge]` with `on_each_publish = true`
- **THEN** the channels answer `beta` of `lib` and of `web` on each publish, `edge` of `web` on each publish, `next` of `lib` bound to `beta`, and `stable` of each bound to its `beta`, in that order

#### Scenario: No channel declared

- **WHEN** the file at the revision holds `[acceptance]` and no `[channels]` or `[environments]`, or the revision holds no policy file
- **THEN** the channels answer `beta` moving on each publish and `stable` bound to `beta`

#### Scenario: A checkout's channel is not read

- **WHEN** the working tree's policy file declares a channel the file at the revision does not
- **THEN** the channels at that revision do not list it (65, core.22)

### Requirement: approvers names who may ask

A channel's table MAY name `approvers`, a list of signers by their
names in `[signers]`, and the policy for the channel SHALL answer them
as the signers that may ask for it; for a channel a manual environment
makes, the policy SHALL answer the `approvers` the environment's table
names; where neither names `approvers`, the policy SHALL answer none,
any asker the gate verifies (129, C.10).

#### Scenario: Approvers under a given

- **WHEN** `[signers.ci]` and `[signers.desk]` are declared and `[channels.stable]` names `approvers = ["ci"]`
- **THEN** the policy for `stable` names `ci` as the one signer that may ask, and its promoters are still the gate's signers

#### Scenario: An approver no table names

- **WHEN** `[channels.stable]` names `approvers = ["nobody"]` and `[signers]` holds no `nobody`
- **THEN** the channels are unreadable naming `[channels.stable]` and `nobody`

### Requirement: The environments' words are read

An environment's keys SHALL be C.10's words and no others: its
`channel`, `policy` (`auto` or `manual`, `auto` where unwritten),
`after`, a list of environment names of the same component, `target`,
and, where the environment is manual, `approvers`, the signers that may
say yes, read as a channel's `approvers` is (C.10, as ruled at
boomdrivedev/blueprints 088430c).

#### Scenario: An environment naming its target

- **WHEN** `[environments.site]` names `channel = "stable"` and `target.nix_cache = { attic = "fleet:site" }`
- **THEN** the channels answer is read, and `site` makes no channel

#### Scenario: An environment with a target's own key

- **WHEN** `[environments.site]` names `channel = "stable"` and `bucket = "example-site"`
- **THEN** the channels are unreadable naming `[environments.site]` and `bucket`, and saying a target's settings sit in the target's table

#### Scenario: A target naming two profiles

- **WHEN** `[environments.site]` names `channel = "stable"` and a `target` table with the keys `nix_cache` and `bucket`
- **THEN** the channels and the environments are unreadable naming `[environments.site]` and `target`

#### Scenario: A target whose profile names no table

- **WHEN** `[environments.site]` names `target = { nix_cache = "fleet:site" }`
- **THEN** the channels and the environments are unreadable naming `[environments.site]` and `target`

#### Scenario: Approvers in a manual environment's table

- **WHEN** `[signers.ci]` is declared and `[environments.prod]` names `channel = "stable"`, `policy = "manual"` and `approvers = ["ci"]`
- **THEN** the channels answer lists `prod` bound to `stable`, and the policy for `prod` names `ci` as the one signer that may ask

#### Scenario: Approvers on an environment that moves on its own

- **WHEN** `[environments.site]` names `channel = "stable"` and `approvers = ["ci"]`, and no `policy`
- **THEN** the channels are unreadable naming `[environments.site]` and `approvers`

#### Scenario: An unknown policy word

- **WHEN** `[environments.prod]` names `policy = "sometimes"`
- **THEN** the channels are unreadable naming `[environments.prod]` and `sometimes`

### Requirement: An environment's channel is named for it

The channel a manual or after environment is given SHALL take the
environment's name, in its component, and a line under it SHALL add
claims after the gate's acceptance (C.10). A channel so made that
shares a name with a channel the fragment holds, held or not, or with
another made channel, or a line under it naming another binding or
`on_each_publish`, SHALL be unreadable naming the table (C.10).

#### Scenario: A manual environment named beta

- **WHEN** `[environments.beta]` names `channel = "stable"` and `policy = "manual"`
- **THEN** the channels are unreadable naming `[environments.beta]`

#### Scenario: A line that restates prod's binding

- **WHEN** the manual environment `prod` names `stable`, and `[channels.prod]` names `bound_to = "stable"` and a `require`
- **THEN** `prod` is listed bound to `stable`, and its policy requires the gate's acceptance and then the line's claims

#### Scenario: Approvers under a made channel

- **WHEN** the manual environment `prod` names `stable`, and `[channels.prod]` names `approvers = ["ci"]`
- **THEN** the channels are unreadable naming `[channels.prod]` and `approvers`, and saying who may ask for `prod` is written on `[environments.prod]`

#### Scenario: A manual environment named for a held given

- **WHEN** `[environments.lts]` names `channel = "stable"` and `policy = "manual"`, and the file writes no `[channels.lts]`
- **THEN** the channels are unreadable naming `[environments.lts]`

### Requirement: The table in force lays the fragment beneath the file

The profile SHALL read every answer from one table in force: the
repository's file as written, with the fragment's table of each channel
it holds laid beneath the repository's table of that channel in every
component, as `[channels.<name>]` where no component is declared and
`[channels.<component>.<name>]` where components are (C.10, 125).

#### Scenario: A line under one component's given

- **WHEN** the file declares `lib` and `web` and `[channels.lib.stable]` requires a rebuild from `verifier`
- **THEN** the policy for `stable` of `lib` requires the gate's acceptance and then the rebuild, and the policy for `stable` of `web` the gate's acceptance alone, both bound to their component's `beta`

#### Scenario: A line that restates how a given moves

- **WHEN** `[channels.stable]` names `bound_to = "beta"` and a `require`
- **THEN** `stable` is listed bound to `beta`, and its policy requires the gate's acceptance and then the line's claims

#### Scenario: A fragment that does not parse

- **WHEN** the profile is given a fragment that is not well formed, and the channels or any policy are asked
- **THEN** the answer is unreadable naming the givens, with the parser's words verbatim

#### Scenario: The shipped fragment

- **WHEN** the shipped fragment is read
- **THEN** it parses, holds only channels' tables, declares `beta` moving on each publish with releases alone and `stable` bound to `beta`, and holds `alpha` moving on each publish with prereleases and `lts` bound to `stable`, both marked held

### Requirement: A file naming no gate names its one provenance signer as the gate

Where `[signers]` at the revision asked names no signer that may claim
`acceptance` and exactly one that may claim `provenance`, the profile
SHALL answer that signer as one that may claim `acceptance` too, in
every audience's claims but a layout's, whose given gate is for
publishing, not reading: the repository's gate, and the promoter of its
given and made channels, as the one host 108 admits (118, C.10).

#### Scenario: A laptop's policy

- **WHEN** the file at the revision lists `[signers.laptop]` with `kinds = ["provenance"]`, `[acceptance]` requires provenance by `laptop`, and no channel is written
- **THEN** the policy for acceptance names `laptop` with `provenance` and `acceptance`, and the policies for `beta` and `stable` each require acceptance by `laptop` and are promoted by `laptop`

#### Scenario: The gate accepts as one host

- **WHEN** the gate signing as `laptop` judges, under that file, a version whose provenance `laptop` made
- **THEN** the version is accepted, its acceptance claim saying one host offered and accepted, naming `laptop`, and `beta` moves; the gate read `laptop`'s reach from the profile's answer alone (core.7)

#### Scenario: A file that names a gate

- **WHEN** the file lists `[signers.laptop]` for provenance and `[signers.gate]` with `kinds = ["acceptance"]`
- **THEN** the policy for acceptance names `laptop` with `provenance` alone, and a gate signing as `laptop` over a version `laptop` offered refuses it as offered by the gate

### Requirement: Several publishers and no gate is refused in words

Where `[signers]` at the revision asked names no signer that may claim
`acceptance` and two or more that may claim `provenance`, the profile
SHALL answer the policy for acceptance, and the policy for each given or
made channel, refused, wanting acceptance by no signer, with its own
words `several publishers and no gate, name an accepting signer`, since
108 admits no host (58, C.10).

#### Scenario: Two publishers and no gate

- **WHEN** the file lists `[signers.laptop]` and `[signers.desk]`, each with `kinds = ["provenance"]`, and no signer that may claim acceptance
- **THEN** the policy for acceptance, and for `beta` and `stable` with no line written, is refused wanting acceptance by no signer, carrying `several publishers and no gate, name an accepting signer` verbatim

#### Scenario: The gate accepts nothing

- **WHEN** a gate signing as `laptop` judges, under that file, a version whose provenance `laptop` made
- **THEN** nothing is accepted, the gate's answer carries the policy's refusal with those words, no channel moves, and the ledger is not written

#### Scenario: What else the file answers

- **WHEN** that file also has a `[layout]` table and `[channels.edge]` declared by hand
- **THEN** the policy for `[layout]` and for `edge` are answered as written, and the channels listed are `beta`, `edge` and `stable`

## ADDED Requirements

### Requirement: The policy for a layout with no table

Where the revision holds no policy file, the answer for a layout SHALL
be absent naming it, and where its file holds no `[layout]`, the answer
for a layout SHALL be the given policy for a layout (C.10).

#### Scenario: A file of channels alone

- **WHEN** the file holds `[channels.stable]` and no `[acceptance]` or `[layout]`, and the policy for a layout is asked
- **THEN** it is the given policy for a layout, marked as given

### Requirement: The channels of a revision or file that cannot be read

A revision the repository does not hold, or a file that cannot be read,
SHALL make the channels answer as policy-for answers it (58, 59).

#### Scenario: Channels at a revision the repository does not hold

- **WHEN** the channels are asked at a revision the repository does not hold
- **THEN** the answer is the one policy-for gives at that revision

### Requirement: An approvers list naming no signer the file holds is unreadable

A name in `approvers` that `[signers]` does not hold SHALL be unreadable
naming the table and the name, and an empty list SHALL be unreadable
naming the table, since it would read as naming no one (C.10).

#### Scenario: An empty approvers list

- **WHEN** `[channels.stable]` names `approvers = []`
- **THEN** the channels are unreadable naming `[channels.stable]`

### Requirement: An environment's target is one profile's table, carried as written

An environment's `target` SHALL be a table of exactly one key, the
profile of the environment's target, whose value is a table, the
target's own settings, which the profile SHALL carry as written and read
nothing of (C.10, 136). A `target` naming no profile or two, or whose
profile's value is not a table, SHALL be unreadable naming the table and
`target`.

#### Scenario: A target's settings carried as written

- **WHEN** `[environments.site]` names `channel = "stable"` and `target.nix_cache = { attic = "fleet:site", closures = ["site"] }`
- **THEN** the environment's target is `nix_cache`, with those settings as written

#### Scenario: A target naming no profile

- **WHEN** `[environments.site]` names `channel = "stable"` and `target = {}`
- **THEN** the channels and the environments are unreadable naming `[environments.site]` and `target`

### Requirement: An environment's words outside C.10 are unreadable

Any `policy` but `auto` or `manual`, or a `channel` the file does not
declare, give or make, SHALL be unreadable naming the table.
`approvers` on an environment that is not manual SHALL be unreadable
naming the table and the key, since no one says yes to it. Any key but
`channel`, `policy`, `after`, `target` and `approvers` SHALL be unreadable naming the table and the key, since a target's own
settings sit in the target's table and nowhere else (C.10).

#### Scenario: A channel the file does not give

- **WHEN** `[environments.site]` names `channel = "nightly"`, which the file neither declares, gives nor makes
- **THEN** the channels are unreadable naming `[environments.site]`

### Requirement: A line under a made channel naming approvers is unreadable

A line under a made channel naming `approvers` SHALL be unreadable
naming the table, saying that who may ask for it is written on the
environment, since the made channel takes its environment's (C.10, as
ruled at boomdrivedev/blueprints 088430c).

#### Scenario: Approvers under qa's channel

- **WHEN** the manual environment `qa` names `beta`, and `[channels.qa]` names `approvers = ["ci", "desk"]`
- **THEN** the channels are unreadable naming `[channels.qa]`, saying who may ask for `qa` is written on `[environments.qa]`

### Requirement: A key the repository writes replaces the fragment's

A key the repository writes SHALL replace the fragment's;
`on_each_publish` and `bound_to` SHALL be replaced together where either
is written, and with them the fragment's `prereleases` unless the
repository writes its own (C.10).

#### Scenario: stable made to move on each publish

- **WHEN** `[channels.stable]` names `on_each_publish = true`
- **THEN** `stable` is listed moving on each publish, and bound to no channel

### Requirement: A file that names a gate, or no publisher, is answered as written

A file that names a signer that may claim `acceptance`, or none that may
claim `provenance`, SHALL be answered as written, and the profile SHALL
write nothing into any file (118, C.10).

#### Scenario: A file naming no publisher

- **WHEN** the file lists `[signers.verifier]` with `kinds = ["rebuild"]` alone
- **THEN** no signer is answered as one that may claim `acceptance`, and the file is unchanged

### Requirement: With several publishers and no gate, the rest is answered as written

Where several publishers and no gate leave acceptance refused, the
policy for `[layout]` and for a channel declared by hand SHALL be
answered as written, and every channel SHALL still be listed (C.10).

#### Scenario: The channels still listed

- **WHEN** the file lists two signers that may claim `provenance`, none that may claim acceptance, and `[channels.edge]` declared by hand
- **THEN** the channels listed are `beta`, `edge` and `stable`, and `edge`'s policy is answered as written

### Requirement: An environment is declared under its component

An environment SHALL be declared as `[environments.<name>]` where no
component is declared and `[environments.<component>.<name>]` where
components are (C.10).

#### Scenario: An environment of one of two components

- **WHEN** the file declares `lib` and `web`, and `[environments.web.site]` names `channel = "stable"`
- **THEN** `site` is an environment of `web`, following `stable` of `web`

### Requirement: A fragment that cannot be read makes every answer unreadable

A fragment that does not parse, or holds a table other than a
channel's, SHALL make every answer unreadable naming the givens; the
shipped fragment SHALL parse (C.10).

#### Scenario: A fragment holding another table

- **WHEN** the profile is given a fragment holding `[acceptance]` beside its channels' tables, and the channels are asked
- **THEN** the answer is unreadable naming the givens
