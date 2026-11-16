## MODIFIED Requirements

### Requirement: The types that cross a seam

Every value an operation takes or gives SHALL be a type defined with the
seams, in the statement's words, and nowhere else: those "The seams
define the types of bytes, builds and claims" and "The seams define the
types of policies, channels and ranges" name. Every such type SHALL be
readable by a program (57).

#### Scenario: A value written and read back

- **WHEN** any value of these types is written in its machine-readable form and read back
- **THEN** it is the same value

#### Scenario: A cache entry is no artifact

- **WHEN** a cache entry is written in its machine-readable form
- **THEN** it names a build identity and a digest and nothing else, and reading it as artifact metadata is rejected

#### Scenario: A requirement keeps the manifest's spelling

- **WHEN** a requirement naming a scoped name and a range with an operator and a prerelease is written in its machine-readable form
- **THEN** it names that name and that range, character for character, and reads back as the same requirement

#### Scenario: A declared channel names its component

- **WHEN** a channel a policy declares is written in its machine-readable form
- **THEN** it names the channel's name, its component or that the repository declares none, and how it moves, naming the channel it is bound to where it is bound and whether it takes prereleases where it moves on each publish

#### Scenario: A channel spelled

- **WHEN** the channel `beta` of no declared component and the channel `beta` of `lib` are spelled
- **THEN** they are `beta` and `lib/beta`, and the two are not equal

### Requirement: A claim is about one digest by one signer

A claim SHALL be about exactly one digest, SHALL name its kind and the
revision it was made for, and SHALL name the build identity where it
has one (7, 13). A signed envelope SHALL have a digest of its own by
which a promotion names its predecessor (105, 120). An envelope holding
a claim of timber's form SHALL hold exactly one claim and the
signature of exactly one signer, and one holding any other number of
signatures SHALL be read as no claim of timber's form (7).

#### Scenario: A claim with no build identity

- **WHEN** a claim is made with a revision and no build identity
- **THEN** it is a claim, and reading it back answers no build identity rather than an empty one

#### Scenario: An envelope's digest

- **WHEN** the same signed envelope is held in two places
- **THEN** both answer the same envelope digest

#### Scenario: A claim carried inside another

- **WHEN** a claim holds another party's signed envelope as found
- **THEN** reading the claim back answers that envelope byte for byte as it was made (12)

#### Scenario: Another party's envelope with two signatures

- **WHEN** a claim carries an envelope signed twice, each signature with its own key id
- **THEN** reading the claim back answers both signatures with their key ids, in the order made

#### Scenario: Timber's form with two signatures

- **WHEN** an envelope holding a claim of timber's form carries two signatures
- **THEN** it is read as no claim of timber's form

### Requirement: The store's contract

The store SHALL answer put-bytes with the digest and whether it held
the bytes already (66, 5); put-artifact with the artifact's digest
(67); attach with the attestation held beside its digest (68, 8);
claims-of with every attestation as attached, or absent (69); name
with the name moved, or refused naming where it is (70); resolve with
a digest, or absent (71); and read with what the digest holds, or
absent (72).

#### Scenario: A name moved only from where it is expected

- **WHEN** name is asked to move a name from a digest it is not at
- **THEN** the answer is refused, naming the digest the name is at, and the name does not move

#### Scenario: A name made from nothing

- **WHEN** name is asked to move a name that names nothing, from nothing
- **THEN** the name is made

#### Scenario: An artifact read back

- **WHEN** read is asked for the digest put-artifact answered
- **THEN** it answers the artifact's metadata and the digests it names

#### Scenario: A name's versions listed

- **WHEN** the store names `artifact/eco/site@1.0.0`, `artifact/eco/site@1.1.0` and `artifact/eco/site-cli@1.0.0`, and the versions of `eco` `site` and of `eco` `web` are asked
- **THEN** the first answer is 1.0.0 and 1.1.0 and no other, and the second is the empty list, neither absent nor refused

### Requirement: The build's contract

The build SHALL answer ensure, given a revision and every output
declared in one asking, with those outputs present, or refused naming
each output it could not make (74); and report with the build identity
an output came from, the build's outcome, and where its files are (75).

#### Scenario: Outputs that could not be made

- **WHEN** ensure is asked for three outputs and the build can make two
- **THEN** the answer is refused, naming the one it could not make

#### Scenario: What a checkout left

- **WHEN** left is asked of a checkout whose build state holds the outputs of two build identities
- **THEN** the answer names both, each with its tasks, its outcome and where its files are, and names no build identity the checkout's build state does not hold

#### Scenario: A checkout that ran nothing

- **WHEN** left is asked of a checkout whose build has recorded no run
- **THEN** the answer is an empty list, which is neither absent nor unreadable

### Requirement: The landing's contract

The landing SHALL answer landed with whether a branch has landed a
revision (77), and verdicts-of with every verdict recorded for a
revision, each naming its check, its build identity, its outcome and
its signer, with its signed envelope as the landing recorded it, and
where the landing found it, in the landing's own words (12, 78).

#### Scenario: A revision with no verdicts

- **WHEN** verdicts-of is asked for a revision the landing recorded no verdict for
- **THEN** it answers an empty set of verdicts, which is not absent and not unreadable

#### Scenario: Where a verdict was found

- **WHEN** verdicts-of answers a verdict
- **THEN** the verdict says where the landing found it

#### Scenario: An answer read from the remote

- **WHEN** landed or verdicts-of is answered while the landing's remote answers
- **THEN** the answer says it was read from the landing itself

#### Scenario: An answer read as last fetched

- **WHEN** landed or verdicts-of is answered while the landing's remote does not answer, from a copy the host fetched before
- **THEN** the answer says it was read from the copy, with the time the copy was fetched and the remote's words verbatim, and it is neither refused nor unreadable

### Requirement: The policy's contract

The policy SHALL answer policy-for, given a repository at a landed
revision and an audience of acceptance, a channel with its component,
or a layout, with the claims required, each with the signers that may
make it and what each may claim (82), or absent where no policy is
declared. The policy for acceptance SHALL name no channel (83). The
policy seam SHALL have no operation for the held givens apart from
channels (136).

#### Scenario: Acceptance names no channel

- **WHEN** policy-for is asked for acceptance
- **THEN** its answer names no channel, and requirements for acceptance cannot be made naming one

#### Scenario: The channels and policy-for agree on a binding

- **WHEN** channels answers a channel bound to another, and policy-for is asked for that channel at the same revision
- **THEN** both name the same channel it is bound to

#### Scenario: A channel's promoters

- **WHEN** policy-for is asked for a channel
- **THEN** it names the signers whose promotion of that channel counts, each with what it may claim

#### Scenario: A channel's approvers

- **WHEN** policy-for is asked for a channel whose policy names who may ask for it, and for one whose policy names none
- **THEN** the first names those signers, and the second names none, which is any asker the gate verifies

#### Scenario: A given held off

- **WHEN** the channels are asked at a revision whose policy holds a given the repository has not switched on
- **THEN** the one answer carries the channels in force and, apart from them, the held given marked held, naming its component, how it would move and the line that would switch it on, and no channel is both in force and held

#### Scenario: A channels answer written and read back

- **WHEN** a channels answer holding a channel in force and a given held off is written in its machine-readable form and read back
- **THEN** it is the same value, the held given naming the channel, its component or that the repository declares none, how it would move and its line, apart from the channels in force

### Requirement: A projection's contract

A projection SHALL answer pack, given an output at a build identity and,
where the bytes are an artifact's, the version, the component the output
belongs to, the component's declared outputs as the build left them, and
a way to ask whether the component's paths hold a path of the
repository, with the ecosystem's own bytes, their metadata only where a
version was given, and whether the same outputs and version, or the same
build identity, pack to the same bytes (84, 87, 100, 110, 123).

#### Scenario: A projection that cannot promise the same bytes
- **WHEN** pack answers bytes it cannot promise to make again
- **THEN** its answer says so

#### Scenario: An output packed as an artifact
- **WHEN** pack is given an output, a build identity and a version
- **THEN** its answer carries metadata naming the ecosystem, the name and that version

#### Scenario: An output packed as a cache entry
- **WHEN** a projection that packs cache entries is given an output and a build identity and no version
- **THEN** its answer carries the bytes and no artifact metadata, so nothing about it makes an offer

#### Scenario: A layout of cache entries
- **WHEN** a projection that packs cache entries renders a layout from cache entries
- **THEN** the answer names the build identities the layout holds, and no version

#### Scenario: A projection that packs no cache entries
- **WHEN** a projection that packs no cache entries is asked to pack an output with no version, or to render cache entries
- **THEN** each answer is absent naming cache entries, never refused and never unreadable

#### Scenario: A checkout with no lockfile
- **WHEN** pinned is asked of a checkout holding no lockfile of the projection's ecosystem and no manifest declaring a range
- **THEN** the answer is absent naming the lockfile, never refused and never an empty list of pins

#### Scenario: A pin written and read back
- **WHEN** a pin, an ecosystem name and a version, and a range, an ecosystem name and a range's text, are each written in its machine-readable form and read back
- **THEN** each is the same as written

#### Scenario: What a held version requires
- **WHEN** requires is asked of an artifact whose manifest requires two names the layout serves, each with a range
- **THEN** the answer is those two names, each with its range as the manifest spells it

#### Scenario: Versions that satisfy a range, newest first
- **WHEN** satisfying is given a range and versions listed in neither the ecosystem's order nor spelling order
- **THEN** the answer is the versions that satisfy the range, newest first by the ecosystem's ordering, and no other

#### Scenario: An ecosystem with no manifest
- **WHEN** a projection whose ecosystem has no manifest is asked requires of an artifact, or satisfying of a range
- **THEN** each answer is none, an empty list, never absent, refused or unreadable

#### Scenario: An output packed with its component
- **WHEN** pack is given an output with the version, the component the output belongs to, the component's declared outputs and a way to ask whether the component's paths hold a path
- **THEN** the file, image and moon cache projections each answer the same bytes and metadata they answer given that version alone

#### Scenario: A request served
- **WHEN** a projection whose clients read no files is asked to serve a request for a version its layout holds
- **THEN** it answers the response its client reads, and asks the core for nothing

#### Scenario: A request for a version the layout lacks
- **WHEN** a projection is asked to serve a request for a version its layout lacks
- **THEN** it asks the core for that one version, by its ecosystem name and version, and answers the core's answer in its client's protocol: the version's response where the core rendered it, and otherwise a response carrying the refusal, the absence or the unreadable answer in the source's own words

#### Scenario: A projection whose clients read files
- **WHEN** a projection whose ecosystem's clients read files is asked to serve any request
- **THEN** the answer is absent naming serving, and the core is asked for nothing

#### Scenario: A request and a response written and read back
- **WHEN** a request and a response are written in their machine-readable form and read back
- **THEN** each is the same, a response's bytes included

### Requirement: A target's contract

A target SHALL answer desired with the digest an environment's channel
names, or absent when the channel is empty (88), given the head the core
read from the channel's chain, which a target that is pushed to answers
and a target that pulls answers by its own reading (44, rule core.50);
current with the digest the environment runs, or unreadable with the
reason (89); and apply, given a version the reconciler has verified,
with the environment converged or what stopped it (90).

#### Scenario: An environment that cannot be read

- **WHEN** current is asked of an environment whose state cannot be read
- **THEN** the answer is unreadable with the reason, never a digest

#### Scenario: A push target answers the head it is handed

- **WHEN** desired is asked of a target that is pushed to, with a head that answers a digest
- **THEN** it answers that digest, and with a head that answers the channel empty it answers absent

#### Scenario: A target that holds no marker

- **WHEN** a version is applied to an environment whose target can hold no marker, and current and report are asked
- **THEN** each answers unreadable with the reason, and neither names the digest applied

### Requirement: The keeper's contract

The keeper SHALL answer append, given a channel with its component, a
promotion claim and the digest of the promotion it names as its
predecessor, with the claim held at the head of that channel's chain,
or refused naming the claim at the head (120); and chain, given a
channel with its component, with every promotion claim of it in order
from the first, or absent when none is held (121). Two components'
channels of one name SHALL be two chains (125).

#### Scenario: An append whose predecessor is not the head

- **WHEN** append is given a predecessor that is not the head of the channel's chain
- **THEN** it is refused naming the claim at the head, and the chain is unchanged

#### Scenario: A chain read as last read

- **WHEN** the source holding a chain the host has read cannot be reached, and the chain is asked
- **THEN** the answer is the claims as last read, the time of that read and the source's words, and an append is unreadable

#### Scenario: An empty chain read as last read

- **WHEN** the host last read a channel's chain holding no claim, and the source holding it then cannot be reached
- **THEN** the answer is a chain of no claims, the time of that read and the source's words, and not absent

#### Scenario: Two components' chains of one name

- **WHEN** a promotion is appended to `beta` of `lib` and the chain of `beta` of `web` is asked
- **THEN** the chain of `beta` of `web` is absent

### Requirement: The promotion claim's body

A promotion claim's body SHALL be one type defined with the ledger seam,
readable by a program and read back the same (57): the channel, the
component, the version and index it moves to, the version and index it
moves from or none for the channel's first, the digest of the promotion
it follows or none, who asked, the revision the channel's policy was
read at and the digests of the claims that satisfied it, and when, in
seconds since the Unix epoch (33, 39, 105).

#### Scenario: A promotion's body read back

- **WHEN** a promotion claim is made with a body and its envelope is read back
- **THEN** the body read is the body made, value for value, and an acceptance claim's envelope reads as no promotion

## ADDED Requirements

### Requirement: The seams define the types of bytes, builds and claims

The seams SHALL define digest, revision, branch, component, output,
build identity, version, ecosystem and ecosystem name, artifact metadata
and index metadata (4, 6), cache entry (52), claim and signed envelope
(7, 8), and verdict (78). A cache entry SHALL be named by its build
identity and its digest alone, and SHALL carry no ecosystem, name or
version (53).

#### Scenario: A verdict read back

- **WHEN** a verdict naming its check, a build identity and its outcome is written in its machine-readable form and read back
- **THEN** it is the same verdict

### Requirement: The seams define the types of policies, channels and ranges

The seams SHALL define signer, audience and the policy's requirements
(82), the channel a policy declares with how it moves (131), a given
channel held off with the line that would switch it on (C.10),
environment and channel, and a manifest's range and the requirement
that pairs an ecosystem name with one (140). A range and a requirement
SHALL carry the text as the manifest spells it, unchanged.

#### Scenario: A held given read back

- **WHEN** a given channel held off, with how it would move and the line that would switch it on, is written in its machine-readable form and read back
- **THEN** it is the same value

### Requirement: A channel is its component and its name

A channel SHALL be its component and its name together, its component
none where the repository declares no component (125). The name of the
one component of a repository that declares none (123) SHALL be defined
with the seams, so every profile and the core name it alike.

#### Scenario: The one component's name

- **WHEN** two profiles name the one component of a repository that declares none
- **THEN** both name it by the name the seams define

### Requirement: An envelope carried from another party is kept as made

An envelope carried from another party SHALL hold every signature it
was made with, each with its signer and its key id as made, and the
payload and its type byte for byte (12, C.5).

#### Scenario: Another party's payload type

- **WHEN** a claim carries another party's envelope whose payload type is not timber's
- **THEN** reading the claim back answers that payload type and that payload byte for byte

### Requirement: The store lists a name's versions

The store SHALL answer the listing, given an ecosystem and a name, with
every version the store names that name at, an empty list where it names
none, or unreadable where the listing cannot be read (138, core.19).

#### Scenario: A listing that cannot be read

- **WHEN** the listing is asked while the store cannot be read
- **THEN** it answers unreadable with the store's words, never an empty list

### Requirement: The build answers what a checkout left

The build SHALL answer left, given a checkout, with the build identities
whose outputs the checkout's own build state holds, each with its tasks,
its outcome and where its files are, read from the build's own records
and from nothing of timber's (139).

#### Scenario: left reads nothing of timber's

- **WHEN** left is asked of a checkout whose build state holds one build identity's outputs, and timber's own records name another
- **THEN** the answer names the first alone

### Requirement: Each landing answer says how it was read

Each answer of the landing SHALL say how it was read: from the landing's
remote, or the repository itself where no remote is bound, or from the
host's copy as last fetched, because the remote did not answer, with
when that copy was fetched and the remote's own words of why it did not
answer, the two ways a chain says it was read (59, 60, 61, 77, C.5,
C.6).

#### Scenario: An answer with no remote bound

- **WHEN** landed is answered by a landing bound to no remote
- **THEN** the answer says it was read from the repository itself

### Requirement: The policy for a channel names its promoters, its askers and its binding

The policy for a channel SHALL name the signers that may sign its
promotion (32) and the signers that may ask for it, none meaning any
asker the gate verifies (129, C.10), and the policy for a channel bound
to another SHALL name the channel it is bound to (83).

#### Scenario: A bound channel's policy

- **WHEN** policy-for is asked for `stable`, bound to `beta`
- **THEN** it names `beta` as the channel it is bound to, the signers that may sign its promotion and those that may ask for it

### Requirement: The policy answers the channels in force

The policy SHALL answer channels, given a repository at a landed
revision, with every channel its policy declares or gives, once each,
ordered by name and then by component, each with the one component it
belongs to (125) and how it moves: on each accepted publish, with
releases alone or prereleases too, by a promotion alone, or bound to a
channel of the same component (37, 83, 131, C.10).

#### Scenario: Channels ordered by name, then component

- **WHEN** the channels are asked at a revision declaring `lib` and `web`, each with `beta` and `stable`
- **THEN** the answer is `beta` of `lib`, `beta` of `web`, `stable` of `lib` and `stable` of `web`, in that order, once each

### Requirement: One component or none holds every channel

A repository that declares one component SHALL have every channel in it,
and one that declares none SHALL have every channel in that one, its
channels spelled by name alone (123).

#### Scenario: One component declared

- **WHEN** the channels are asked at a revision declaring the one component `lib`
- **THEN** every channel answered is of `lib`

### Requirement: The channels answer carries the held givens apart

The same channels answer SHALL carry, apart from the channels in force,
every channel the profile gives but holds until the repository switches
it on, and the repository has not switched on, marked held, once each,
ordered by name and then by component, each with its component, how it
would move once switched on, and the line that would switch it on, in
the profile's own words (131 as amended at boomdrivedev/blueprints
1ae1cc5, C.10).

#### Scenario: Held givens of two components

- **WHEN** the channels are asked of a profile holding `alpha` and `lts` back, at a revision declaring `lib` and `web` that switches on neither
- **THEN** the held givens are `alpha` of `lib`, `alpha` of `web`, `lts` of `lib` and `lts` of `web`, in that order, each marked held with how it would move and its line

### Requirement: The held givens are an empty list where none is held, and never a channel in force

The held givens SHALL be an empty list where the profile holds none
back, and SHALL never hold a channel in force (131 as amended at
boomdrivedev/blueprints 1ae1cc5, C.10).

#### Scenario: Nothing held back

- **WHEN** the channels are asked of a profile that holds no given back
- **THEN** the held givens are an empty list

### Requirement: A projection renders a layout

A projection SHALL answer render, given artifacts and their metadata or
cache entries by build identity, with a consumer layout and the versions
or build identities it holds (85).

#### Scenario: A layout of two artifacts

- **WHEN** a projection renders a layout from two artifacts
- **THEN** the answer names the two versions the layout holds

### Requirement: A projection serves its client's requests

A projection SHALL answer serve, given a layout, one request of the
ecosystem's own client with the base address it reached, and a way to
ask the core for a version the layout lacks, with the response to that
request in the client's own protocol, or absent naming serving, for
every request, from a projection whose ecosystem's clients read files
(86, 115).

#### Scenario: A request under its base

- **WHEN** a projection whose clients read no files is given a request with the base address it reached
- **THEN** the response answers the request in the client's own protocol

### Requirement: A projection answers what a checkout pins

A projection SHALL answer pinned, given a checkout, with the ecosystem
names and versions its lockfile pins as the lockfile spells them, and,
for a name its manifest declares and no lockfile pins yet, the
manifest's range as the manifest spells it, or absent where the checkout
holds no lockfile of that ecosystem and its manifest declares no such
range (137).

#### Scenario: A pin and a range

- **WHEN** pinned is asked of a checkout whose lockfile pins one name and whose manifest declares another that no lockfile pins
- **THEN** it answers the pin as the lockfile spells it and the range as the manifest spells it

### Requirement: A projection answers what a held version requires

A projection SHALL answer requires, given an artifact of its ecosystem a
layout holds, with every name its manifest requires that the layout
serves and the range it requires of it, as the manifest spells them, a
name the layout serves being one the consuming checkout's own
configuration of the ecosystem routes to the served address (140).

#### Scenario: A name the checkout does not route

- **WHEN** requires is asked of an artifact whose manifest requires one name the consuming checkout routes to the served address and one it does not
- **THEN** the answer is the first alone, with its range as the manifest spells it

### Requirement: A projection answers which versions satisfy a range

A projection SHALL answer satisfying, given a range and versions, with
those versions that satisfy the range by the ecosystem's own rules,
newest first by the ecosystem's own ordering, never by spelling or by
the order given (23, 140).

#### Scenario: No version within the range

- **WHEN** satisfying is given a range and versions none of which satisfy it
- **THEN** the answer is an empty list

### Requirement: An ecosystem with no manifest requires nothing

A projection whose ecosystem has no manifest SHALL answer requires and
satisfying with none, an empty list, reading nothing, and never absent
(140).

#### Scenario: The file projection asked what an artifact requires

- **WHEN** the file projection is asked requires of an artifact
- **THEN** it answers none, an empty list, and reads nothing

### Requirement: The core holds ranges and reads no manifest

pinned's range and requires' ranges SHALL be one type, so the core
holds a checkout's range and a held version's alike and reads neither.
The core SHALL learn what a manifest requires and how versions order
only through requires and satisfying, and SHALL never read a manifest or
order versions itself (140, core.1).

#### Scenario: A range from pinned and from requires

- **WHEN** a range pinned answers and a range requires answers, each of one name and one text, are compared
- **THEN** they are values of one type, and equal

### Requirement: A projection reaches the store, the policy and the paths only through what it is handed

A projection SHALL never reach the store but through the store it is
handed, nor the policy or a version's claims at all, nor match a
component's paths itself: a version a served layout lacks is asked of
the core, which judges it, and whether the component's paths hold a path
is asked of the core through pack's input (core.2).

#### Scenario: A served miss judged by the core

- **WHEN** a projection serving a layout is asked for a version the layout lacks
- **THEN** it asks the core for it, reads no claim or policy of the version itself, and reads the store only through the store it was handed

### Requirement: A target reports, and one that holds no marker reports nothing it applied

A target SHALL answer report with what it runs and when it last
converged (91). A target that can hold no marker of the digest it
applied SHALL answer current and report unreadable with the reason,
never a digest and never converged (119).

#### Scenario: A report after a converged apply

- **WHEN** a version is applied to an environment whose target holds a marker, and report is asked
- **THEN** it answers the digest applied and when it converged

### Requirement: A target reads its own settings alone

The environment a target is handed SHALL carry the target's own settings
as the policy answered them, and a target SHALL read no setting but its
own, answering unreadable naming a key it does not read (45, C.10).

#### Scenario: A key the target does not read

- **WHEN** a target is handed an environment whose settings hold a key it does not read, and desired is asked
- **THEN** it answers unreadable naming that key

### Requirement: A chain with a fork is refused

A chain that holds two claims naming one predecessor SHALL be refused
as a fork, naming both, never answered by choosing one (105, 122).

#### Scenario: Two claims after one

- **WHEN** chain is asked of a channel whose claims include two naming one predecessor
- **THEN** it is refused as a fork naming both

### Requirement: A chain answered from the copy says when it was read

A chain answered from the copy the host last read, because the source
holding the ledger did not answer, SHALL say when that copy was read and
carry the source's words of why it did not answer, a channel the copy
holds no claim of answered as a chain of no claims saying so, never as
an absence (60 as amended at boomdrivedev/blueprints b64d367, 61, C.13,
C.14).

#### Scenario: A copy of two claims

- **WHEN** the host last read `stable`'s chain holding two claims, and the source holding it then cannot be reached
- **THEN** chain answers the two claims, with when the copy was read and the source's words

### Requirement: A chain read from its source says so, and no copy holds an append

A chain read from the source itself SHALL say so. An append SHALL never
be held by the copy the host last read alone (60 as amended at
boomdrivedev/blueprints b64d367, 61, C.13, C.14).

#### Scenario: A chain read from its source

- **WHEN** chain is asked while the source holding the ledger answers
- **THEN** the answer says it was read from the source

### Requirement: A claim of another form reads as no promotion

Reading a claim's body SHALL answer none for a claim that is not a
promotion of timber's form.

#### Scenario: A report's body read

- **WHEN** the body of a report claim is read as a promotion's
- **THEN** it answers none
