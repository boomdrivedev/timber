## MODIFIED Requirements

### Requirement: The givens are a fragment beneath the file

The given channels SHALL be a policy fragment in the policy file's own
form, shipped with timber as data in the policy profile's directory
and laid beneath the repository's file, the two read as one table in
force by every answer the profile gives (C.10). The fragment SHALL hold
the channels "The given channels the fragment holds" names, and declare
those "Every component has the givens not held, with no line written"
names.

#### Scenario: A repository that writes no channel

- **WHEN** the policy file at a revision declares the components `lib` and `web` and no channel
- **THEN** the channels answer is `beta` of `lib` and of `web`, each moving on each publish, and `stable` of each, bound to its component's `beta`, and neither `alpha` nor `lts`

#### Scenario: A given's policy with no line written

- **WHEN** the policy file writes no channel and names `gate` as a signer that may claim acceptance, and the policy for `beta` and for `stable` is asked
- **THEN** each answers the gate's acceptance with `gate` as its promoter, `stable` naming `beta` as the channel it is bound to, and neither is absent

#### Scenario: A repository with no policy file

- **WHEN** the revision holds no policy file
- **THEN** the channels answer is `beta` and `stable` of the one component, `stable` bound to `beta`

#### Scenario: Another fragment

- **WHEN** the profile is given, in place of the shipped fragment, one holding only `edge` moving on each publish, and the revision holds no policy file
- **THEN** the channels answer is `edge` alone, and the gate moves `edge` on an accepted publish with nothing else changed

### Requirement: A held given is answered with the line that switches it on

The channels answer SHALL carry, apart from the channels in force,
every given the profile holds that the repository has not switched on,
for every component, marked held, with how it would move once switched
on and the line that would switch it on, read from the fragment:
`[channels.<name>]` where the repository declares no component and
`[channels.<component>.<name>]` where it declares components (C.10 as
amended at boomdrivedev/blueprints b1267d7, 131 as amended at 1ae1cc5).

#### Scenario: Nothing switched on

- **WHEN** the policy file writes no channel and declares no component
- **THEN** the channels answer holds off `alpha`, moving on each publish with prereleases, with the line `[channels.alpha]`, and `lts`, bound to `stable`, with the line `[channels.lts]`

#### Scenario: One component's alpha switched on

- **WHEN** the file declares `lib` and `web` and holds `[channels.lib.alpha]`
- **THEN** the channels answer holds off `alpha` of `web` with the line `[channels.web.alpha]` and `lts` of each with its component's line, and carries `alpha` of `lib` among the channels in force alone

#### Scenario: Another fragment's held given

- **WHEN** the profile is given, in place of the shipped fragment, one holding `edge` moving on each publish and `canary` held, bound to `edge`
- **THEN** the channels answer holds off `canary` with the line `[channels.canary]`, and its channels in force are `edge` alone

### Requirement: A manual environment has a channel of its own

An environment whose `policy` is `manual` SHALL give it a channel of
its own, named for the environment, in the component of the channel it
names and bound to that channel, moved only at an asking (41, 97,
C.10). An environment whose `policy` is `auto`, or unwritten, SHALL
make no channel.

#### Scenario: prod waits on stable

- **WHEN** the environment `prod` names the channel `stable` and `policy = "manual"`
- **THEN** the channels answer lists `prod`, bound to `stable`, beside `beta` and `stable`

#### Scenario: An environment that moves on its own

- **WHEN** the environment `site` names the channel `stable` and writes no `policy`
- **THEN** no channel named `site` is listed

#### Scenario: Who may say yes to prod

- **WHEN** the manual environment `prod` names `stable` and `approvers = ["ci"]`
- **THEN** `prod` is listed bound to `stable`, and its policy names `ci` as the one signer that may ask for it

### Requirement: An environment's channel is read as any channel

A channel an environment's words make SHALL be listed, judged and
promoted like a given channel: its policy requires the gate's
acceptance and its promotion is signed by the gate's signer (131,
C.10). A line under it SHALL add claims after the gate's acceptance;
one naming another way it moves SHALL be unreadable naming its table,
since the environment's words say how it moves (C.10).

#### Scenario: Promoting prod

- **WHEN** `prod` is made by a manual environment bound to `stable`, and the gate's signer promotes to `prod` a version `stable` has held
- **THEN** `prod` moves, and its chain holds the promotion

#### Scenario: A line under prod that rebinds it

- **WHEN** `[channels.prod]` names `bound_to = "beta"` while the manual environment `prod` names `stable`
- **THEN** the channels are unreadable naming `[channels.prod]`

## ADDED Requirements

### Requirement: The given channels the fragment holds

The fragment SHALL hold `beta`, moving on each accepted publish with
releases alone, and `stable`, bound to `beta`; and, held until the
repository switches them on, `alpha`, moving on each accepted publish
with prereleases too, and `lts`, bound to `stable` and moved by no
rule, so only an asking moves it (C.10).

#### Scenario: The shipped fragment's channels

- **WHEN** the revision's policy file declares no component and writes no channel
- **THEN** `beta` moves on each accepted publish with releases alone and `stable` is bound to `beta`, and `alpha`, moving with prereleases too, and `lts`, bound to `stable` and moved by no rule, are held

### Requirement: Every component has the givens not held, with no line written

The givens that are not held SHALL be declared, by the fragment: every
component SHALL have `beta` and `stable` with no line written, listed
like a channel declared by hand, with the policy for each answering
like any channel's, and never answered as not declared (C.10).

#### Scenario: A given's policy with no line, of one of two components

- **WHEN** the file declares `lib` and `web` and writes no channel, and the policy for `stable` of `lib` is asked
- **THEN** it answers as the policy of a channel declared by hand would, and not as a channel not declared

### Requirement: A repository that declares no channel still has the givens

A repository that declares no channel, as 102 lets it, SHALL be one
that writes no channel line: its versions are offered and accepted as
102 says, and it still has the givens, which the fragment declares (as
ruled at boomdrivedev/blueprints b64d367).

#### Scenario: A repository that declares no channel publishes

- **WHEN** a repository whose policy file writes no channel line publishes a version the gate accepts
- **THEN** the version is offered and accepted as 102 says, and `beta` and `stable` of its component are listed

### Requirement: A given switched on joins the channels in force

A given the repository switches on SHALL leave the held givens and join
the channels in force, and one it does not SHALL never be among the
channels in force (C.10).

#### Scenario: alpha switched on

- **WHEN** the file declares no component and holds `[channels.alpha]`
- **THEN** `alpha` is among the channels in force and not held, and `lts` alone is held

### Requirement: Who may ask for a manual environment's channel is named on the environment

Who may ask for a manual environment's channel SHALL be named by
`approvers` on the environment, `[environments.<environment>]`, which
the made channel takes as its own; never under the made channel, which
the repository does not write approvers on (C.10, as ruled at
boomdrivedev/blueprints 088430c).

#### Scenario: Two approvers on prod

- **WHEN** the manual environment `prod` names `stable` and `approvers = ["ci", "ops"]`
- **THEN** the channel `prod` takes `ci` and `ops` as the signers that may ask for it

### Requirement: A line under an environment's channel names no approvers

A line under a channel an environment's words make that names
`approvers` SHALL be unreadable naming its table, since who may ask for
it is written on the environment (C.10).

#### Scenario: A line under prod naming approvers

- **WHEN** `[channels.prod]` names `approvers = ["ci"]` while the manual environment `prod` names `stable`
- **THEN** the channels are unreadable naming `[channels.prod]`
