## MODIFIED Requirements

### Requirement: The policy's contract

The policy's group SHALL hold a policy to 82, 83 and 131: for each
audience, the claims required with the signers that may make each and
the kinds and checks each may claim, binding no signer by output or
ecosystem (116); an audience no policy declares, as "The policy's group
asks an absence of a channel" says; and the channels, as "The policy's
group holds the channels a policy declares" says.

#### Scenario: The claims an audience requires
- **WHEN** the policy for acceptance is asked at a revision whose policy requires a provenance from one signer and a verdict for one check from another
- **THEN** it answers both requirements, each with its signers and what each may claim

#### Scenario: Acceptance names no channel
- **WHEN** the policy for acceptance and for a channel bound to another are asked
- **THEN** acceptance answers no channel, and the bound channel's answer names the channel it is bound to

#### Scenario: No policy for an audience
- **WHEN** the policy for the channel `nightly` is asked where no policy declares it
- **THEN** the answer is absent naming the audience, never refused

#### Scenario: The channels a policy declares

- **WHEN** the channels are asked at a revision declaring the components `lib` and `web`, `edge` of `lib` moving by promotion alone, `nightly` of `web` moving on each publish and `next` of `lib` bound to `edge`
- **THEN** the answer is those three channels and the ones the profile gives `lib` and `web`, once each by name and then component, each with its component and how it moves, and the policy for `next` names `edge` as the channel it is bound to

#### Scenario: One component has every channel

- **WHEN** the channels `edge` and `next` are declared, at one revision declaring no component and at another declaring the one component `lib`
- **THEN** both are of no declared component, spelled by name alone, at the first, and of `lib` at the second, and so is every channel the profile gives

#### Scenario: A policy declaring no channel

- **WHEN** the channels are asked at a revision whose policy declares acceptance and no channel
- **THEN** the answer is the channels the profile gives the one component, an empty list where it gives none, and not an absence

#### Scenario: Each way a channel moves

- **WHEN** the channels are asked at a revision declaring one channel of each way it moves, and a second channel moving on each publish with prereleases
- **THEN** the answer says each as declared, prereleases of the second alone, and policy-for of each agrees on the channel it is bound to

#### Scenario: The givens held off

- **WHEN** the channels are asked at a revision declaring the components `lib` and `web` and the channel `edge` of `lib`
- **THEN** the answer's held givens are what the rig says its profile holds off for `lib` and `web`, an empty list where it holds none, each with how it would move and a line, and none of them is among the answer's channels in force

#### Scenario: The channels of a policy that cannot be reached

- **WHEN** the rig makes the policy unreachable and the channels are asked
- **THEN** the answer is unreadable naming the policy, with its words unchanged, and not an empty list

### Requirement: A projection's contract

The projection's group SHALL hold a projection to 84 to 87, and to the
parts of 48 and 50 a projection keeps: a layout rendered again from the
same artifacts is the same layout, and holds the versions it was given
and no other. The rig SHALL name its ecosystem's request for a version's
bytes, since a request's form is the ecosystem's.

#### Scenario: Packed the same again

- **WHEN** one output is packed twice with one version
- **THEN** both answers carry the same bytes and metadata naming the ecosystem, name and version, or the projection says it cannot promise the same bytes

#### Scenario: A layout of the versions given

- **WHEN** a layout is rendered from two artifacts and rendered again from the same two
- **THEN** both renders hold those two versions and no other, and are the same layout

#### Scenario: Served where the clients read no files

- **WHEN** a projection whose clients read no files renders a layout and is asked to serve the rig's request for a version it holds
- **THEN** it answers a response that answers that request, and asks the core nothing

#### Scenario: A cache entry packed with no version

- **WHEN** a projection that packs cache entries packs one output twice at one build identity with no version
- **THEN** neither answer carries artifact metadata, and both carry the same bytes, or the projection says it cannot promise the same bytes

#### Scenario: A layout of the build identities given

- **WHEN** a projection that packs cache entries renders a layout from two cache entries, each named by a build identity and the digest of the bytes packed at it, and renders it again from the same two
- **THEN** both renders hold those two build identities and no other, and are the same layout

#### Scenario: A projection that packs no cache entries

- **WHEN** a projection that packs no cache entries is asked to pack an output with no version and to render a cache entry
- **THEN** both answers are absent, never refused and never unreadable

#### Scenario: The fakes in both cases

- **WHEN** the projection group runs over the fake projection and over the fake projection scripted to pack no cache entries
- **THEN** every scenario passes or is not applicable to that case, and each cache-entry scenario passes in one of the two runs

### Requirement: A target's contract

The target's group SHALL hold a target to 88 to 91, to 45 and 119, and
to the parts of 31, 40 and 43 a target keeps: what an environment should
run is its channel's head as the core hands it, an empty channel is an
answer, what it runs is reported by digest with when it last converged
where the target holds a marker and is unread with the reason where it
holds none, and a target reads no setting but its own.

#### Scenario: What an environment should run

- **WHEN** desired is asked of one environment with a head that names a digest, and of another with a head that answers the channel empty
- **THEN** the first answers that digest and the second answers empty, not an error

#### Scenario: An environment that cannot be read

- **WHEN** what an environment runs cannot be read
- **THEN** the answer is unreadable with the reason, never a digest

#### Scenario: Converged and reported

- **WHEN** a verified version is applied to an environment whose target holds a marker
- **THEN** the environment's current digest and its report name that version's digest and when it converged

#### Scenario: Applied where no marker is held

- **WHEN** a verified version is applied to an environment whose target holds no marker
- **THEN** current and report each answer unreadable with the reason, and neither names the digest applied

#### Scenario: A table with a key the target does not read

- **WHEN** desired, current and apply are asked of an environment whose settings hold a key the target does not read
- **THEN** each answers unreadable naming that key, and nothing is applied

### Requirement: Promotion over the fakes

The ledger's group SHALL hold the core's promotion over the fakes: S6 as
far as the ledger goes, an approver promoting a version whose policy
for `stable` requires the gate's acceptance and a rebuild by a second
signer, the move in the ledger before the store, the claim attached to
the index and `stable`'s name at it (33, 39, 101); S7 (35, 105); and S8
(34, 38). Each scenario SHALL cite the clauses it exercises.

#### Scenario: S6 over the fakes

- **WHEN** an approver promotes an accepted version whose index carries a second signer's rebuild claim the policy for `stable` requires
- **THEN** the ledger's append comes before every store write, the promotion claim is attached to the index, and `stable`'s name resolves to it

#### Scenario: S7 over the fakes

- **WHEN** a promotion names `stable` at a version it has left
- **THEN** it is refused naming the version `stable` is at, and the ledger and the store are asked to write nothing

#### Scenario: S8 over the fakes

- **WHEN** `stable` is moved back to the version before
- **THEN** the ledger holds both moves, and the newer version's index and claims are still read by digest

#### Scenario: The register after this unit

- **WHEN** the coverage check runs
- **THEN** 32, 33, 37, 38, 39, 101 and 106 are cited and in no register entry, 41 and 97 are pending with the first target, and 125 is pending with `the-policy-seam-lists-the-channels`

### Requirement: The given channels over the fakes

The gate's and the ledger's groups SHALL hold the core to the channels
the policy gives and makes: the gate moving every channel the channels
answer says moves on each publish, of the component published and no
other, and no channel it reads from the policy file (95, 125, 131), a
prerelease moving only those that take prereleases (95, 131). Each
scenario SHALL cite the clauses it exercises.

#### Scenario: beta of one component

- **WHEN** the fake policy lists `beta` of `lib` and of `web`, both moving on each publish, and the gate accepts a version of `lib` a publish told it of
- **THEN** `beta` of `lib` moves and `beta` of `web` does not, and the ledger holds one chain

#### Scenario: stable promoted by the gate's signer

- **WHEN** the policy for `stable` names `gate` as the signer of its promotion, and in one world a host signing as `gate` and in another one signing as `laptop` promotes a version `beta` has held
- **THEN** the first moves `stable`, and the second is refused naming the promotion and `gate` with nothing appended

#### Scenario: A prerelease and the channels that follow

- **WHEN** the fake policy lists `edge` moving on each publish with prereleases and `nightly` moving on each publish with releases alone, and the gate accepts a prerelease a publish told it of
- **THEN** `edge` moves and `nightly` does not, and the version is accepted

#### Scenario: An asker the approvers do not name

- **WHEN** the policy for `next` names `gate` as its promoter and `ci` as the one signer that may ask, and a host signing as `gate` promotes a version
- **THEN** the promotion is refused naming `gate` as the asker and `ci`, with nothing signed or appended

### Requirement: The gate told by a publish's answer over the fakes

The gate's group SHALL hold a scenario in which the core's publish
offers a version and answers, with nothing accepted and nothing asked
of the ledger, and the gate, told of that answer with the index it
names, accepts that index and moves each channel that follows each
publish; and a scenario in which the told index is not what the
version's name resolves to, answered unreadable with nothing written
(95, 130).

#### Scenario: The core starts nothing after an offer

- **WHEN** the suite publishes a version over the fakes and then tells the gate of the publish's answer
- **THEN** the fake keeper and the fake store record no acceptance, no promotion and no append before the gate is told, and after it the acceptance is attached to the index the publish's answer named and `beta`'s chain holds the promotion

#### Scenario: A told index the name does not resolve to

- **WHEN** the suite tells the gate of a publish's answer whose index differs from the one the version's name resolves to
- **THEN** the gate's answer is unreadable naming both digests, and the fake store and the fake keeper record no write

#### Scenario: 130 cited and not registered

- **WHEN** the coverage check runs
- **THEN** 130 is cited by the gate's group and is not in the register

### Requirement: The projection's group holds serve

The projection's group SHALL hold a projection to serve, below its
marker, over the rig's own request for a version, which the rig names
since a request's form is the ecosystem's: a version the layout holds is
answered and asks the core nothing (86, 46); a version it lacks is asked
of the core once, and answered once the core renders it (115, 50); a
refusal the core answers is carried in the response in its own words
(99, 59).

#### Scenario: A version held, served
- **WHEN** a projection that serves renders two of the rig's artifacts and is asked to serve the rig's request for one
- **THEN** the response answers it, and the core was asked nothing

#### Scenario: A version lacked, asked of the core
- **WHEN** a projection that serves renders one of two artifacts and is asked for the other, the core rendering it on the ask
- **THEN** the core was asked once for that version's ecosystem name and version, and the response answers it

#### Scenario: A refusal in the response
- **WHEN** a projection that serves is asked for a version it lacks and the core answers it refused with the words `acceptance wanting`
- **THEN** the response is no answer of the version and carries `acceptance wanting` verbatim

#### Scenario: One base, two places
- **WHEN** one set of artifacts is rendered at two places and the same request is served from each under one base
- **THEN** the two responses are the same, and neither names either place

#### Scenario: Every projection runs it
- **WHEN** the projection group runs over the fake projection, the npm projection, and the file, image and moon cache profiles
- **THEN** the serve scenarios pass over the fake and npm, and are not applicable to the three profiles, each of which answers absent naming serving

## ADDED Requirements

### Requirement: The policy's group asks an absence of a channel

Where the repository declares no policy for an audience, the policy's
group SHALL hold an absence naming the audience, asked of a channel no
policy declares, since the policy file answers a layout with no table by
the given (C.10), the revision beside it being verify's to name, not the
policy's (82).

#### Scenario: An absence that names no revision

- **WHEN** the policy for the channel `nightly` is asked at a revision where no policy declares it
- **THEN** the absence names the audience, and not the revision

### Requirement: The policy's group holds the channels a policy declares

The policy's group SHALL hold the channels the policy declares, each
with the one component it belongs to and how it moves, on each publish
with releases alone or prereleases too, by promotion alone, or bound
within its component, one component holding every channel where the
repository declares one or none (125, 37), listed beside the channels
the profile gives with nothing declared and like them (131).

#### Scenario: A declared channel beside the given ones

- **WHEN** the channels are asked at a revision declaring no component and the channel `edge` moving by promotion alone
- **THEN** the answer lists `edge`, moving by promotion alone, beside the channels the profile gives, each of the one component

### Requirement: The policy's rig declares what its profile gives and holds off

The group's rig SHALL declare components and channels, each with how it
moves, at a landed revision as well as policies for audiences, and
SHALL answer the channels its profile gives a set of components with
nothing declared, and the channels it holds off. The group SHALL hold
the givens a profile holds off to the channels answer, carried apart
from the channels in force, each naming how it would move and the line
that would switch it on (131, C.10).

#### Scenario: A profile that holds nothing off

- **WHEN** the channels are asked of a profile whose rig says it holds no given off
- **THEN** the answer's held givens are an empty list

### Requirement: The policy's group names no given channel

The policy's group SHALL hold a profile to the three ways a channel
moves and SHALL name no channel a profile gives, its declared channels
bearing names no profile gives, so a profile whose givens bear other
names passes alike (C.10).

#### Scenario: A profile whose givens bear other names

- **WHEN** the policy group runs over a profile whose only given channel is `canary`, a name none of the group's declared channels bears
- **THEN** each scenario passes or is not applicable as over a profile giving `beta` and `stable`, and none names `canary`

### Requirement: The projection's group holds each case of cache entries

The projection's group SHALL hold a projection whose rig says it packs
cache entries to their case of 84, 85 and 87, and one whose rig says it
packs none to answering absent for one; each case's scenarios SHALL be
not applicable to a projection of the other.

#### Scenario: A projection that packs none, run alone

- **WHEN** the projection group runs over a projection whose rig says it packs no cache entries
- **THEN** "a cache entry packed with no version" and "a layout of the build identities given" are not applicable, and "a projection that packs no cache entries" passes

### Requirement: A target's rig declares whether it holds a marker

The target group's rig SHALL declare whether its target holds a marker
of what it applied, as a store's rig declares whether its names take a
condition (rule core.49); "converged and reported" SHALL run where it
does, and "applied where no marker is held" where it does not, each not
applicable to the other.

#### Scenario: A rig declaring no marker

- **WHEN** the target group runs over a rig declaring that its target holds no marker
- **THEN** "converged and reported" is not applicable, and "applied where no marker is held" runs

### Requirement: Promotion's refusals, its store and its reading over the fakes

The ledger's group SHALL also hold over the fakes a bound channel taking
a version its bound channel held and since left, and refusing one it
never held (37, 83); a version found wanting, one not accepted, and a
host the policy does not let promote, each refused naming the claim and
writing nothing (9, 31, 32, 58); a forked chain refusing the promotion
(105); the store rewritten from the ledger (106); and a channel read
from the ledger alone, the store asked nothing (101, 60).

#### Scenario: A bound channel over the fakes

- **WHEN** `stable`, bound to `beta`, is promoted over the fakes to a version `beta` held and has since left, and then to one `beta` never held
- **THEN** the first moves `stable`, and the second is refused naming `beta`, with nothing written

#### Scenario: A forked chain over the fakes

- **WHEN** the fake ledger answers `stable`'s chain with two claims naming one predecessor, and `stable` is promoted
- **THEN** the promotion is refused naming both claims, with nothing appended

### Requirement: Who may promote a given channel, over the fakes

The ledger's group SHALL hold a promotion of a channel whose policy
names the gate's signer as its promoter, refusing any other signer (32,
9), and a promotion asked by a signer the channel's approvers do not
name refused (32, 33). Each scenario SHALL cite the clauses it
exercises.

#### Scenario: A host that is not the promoter

- **WHEN** the policy for `stable` names `gate` as its promoter, and a host signing as `desk` promotes a version `beta` held
- **THEN** the promotion is refused naming the promotion and `gate`, with nothing appended

### Requirement: The told gate's scenarios cite 95 and 130

Each of the gate's scenarios of a told publish SHALL cite 95 and 130,
and 130 SHALL leave the register in the commit that cites it (core.33).

#### Scenario: The citations of the told scenarios

- **WHEN** the suite's scenarios of the gate's group are listed with the clauses they cite
- **THEN** both scenarios of a told publish cite 95 and 130

### Requirement: Serve is not applicable where the clients read files

Each serve scenario SHALL be not applicable to a projection whose
clients read files, which SHALL answer absent naming serving. The
group's scenario "served where the clients read no files" SHALL ask
serve for the rig's request of a version the layout holds, in place of
an address, citing 86 as before.

#### Scenario: The file profile and serve

- **WHEN** the projection group runs over the file profile
- **THEN** each serve scenario is not applicable, and serve answers absent naming serving

### Requirement: The promotion's clauses leave the register

32, 33, 37, 38, 39, 101 and 106 SHALL leave the register in the commit
that cites them (core.33).

#### Scenario: None of the promotion's clauses registered

- **WHEN** the coverage check runs
- **THEN** no register entry names 32, 33, 37, 38, 39, 101 or 106

### Requirement: One layout served from two places answers the same

The projection's group SHALL hold a projection to serve, over the rig's
own request for a version, so that one layout served under one base from
two places answers the same and names neither place (49).

#### Scenario: The rig's request from two places

- **WHEN** a projection that serves renders one set of the rig's artifacts at two places, and serves the rig's request for one version from each under one base
- **THEN** the two responses are the same, and neither names either place
