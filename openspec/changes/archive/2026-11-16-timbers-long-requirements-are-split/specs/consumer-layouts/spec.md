## MODIFIED Requirements

### Requirement: A layout holds the lockfile's versions and no other

For each bound projection whose pinned answers pins or ranges for the
checkout, sync SHALL render that ecosystem's layout from the pinned
versions the policy admits, the version held for each range, and the
versions the family requires of what is held, and no other, and SHALL
ask the store only for what the pins name, and for the versions of a
name only where a range is not met by what the layout holds (50, S4,
137).

#### Scenario: S4, a lockfile's versions and no other
- **WHEN** a checkout whose lockfile pins two family versions that require nothing of the family is synced, the store holds a third version of the same name, and the policy admits all three
- **THEN** the layout holds exactly the two pinned versions, the store was asked nothing about the third, and the layout's place names no host

#### Scenario: No lockfile, no layout
- **WHEN** a checkout is synced and a bound projection answers pinned absent
- **THEN** no layout of that ecosystem is rendered, recorded or named in the answer as refused or unreadable

### Requirement: A range is held at the newest admitted version within it

For a range pinned answers, sync SHALL list the name's versions in the
store, keep those the projection answers satisfy the range, and judge
each under the layout's policy, newest first by the projection's
ordering, until one is admitted, and hold that one; none where the listing is empty or none
within it is admitted (137, 138, 140, 23).

#### Scenario: The newest admitted within a range
- **WHEN** a checkout's manifest declares `site` by a range within which the store names 1.0.0 and 1.1.0, which the policy admits, and 1.2.0, which it refuses, and names 2.0.0 outside it
- **THEN** the layout holds `site` 1.1.0 alone, sync's answer names the range holding 1.1.0 and 1.2.0 refused naming the claim found wanting, 2.0.0 was never judged, and the sync answers no refusal

#### Scenario: A range the store names nothing within
- **WHEN** a checkout's manifest declares a range of a name the store's listing answers with the empty list
- **THEN** the layout holds no version of it, and sync's answer names the range holding none

### Requirement: What a held version requires is held through every level

For every version a layout holds, sync SHALL ask the projection what it
requires of the family and hold each range so answered by the same rule,
level by level, until nothing held requires a range the layout does not
meet; a range SHALL be met where the layout holds a version of its name
the projection answers satisfies it (137, 140). Where the projection
answers requires absent, the version SHALL require nothing.

#### Scenario: A pinned version's dependencies, two levels down
- **WHEN** a checkout's lockfile pins `app` 1.0.0, which requires `lib` by a range the store names 1.0.0 and 1.1.0 within, and `lib` 1.1.0 requires `base` by a range the store names 0.3.0 within
- **THEN** the layout holds `app` 1.0.0, `lib` 1.1.0 and `base` 0.3.0, and sync's answer names `lib`'s range as required by `app` 1.0.0 and `base`'s by `lib` 1.1.0

#### Scenario: A range met by what the layout holds
- **WHEN** a checkout's lockfile pins `app` 1.0.0 and `lib` 1.0.0, and `app` 1.0.0 requires `lib` by a range 1.0.0 is within
- **THEN** the layout holds `app` 1.0.0 and `lib` 1.0.0, and the store was asked for no listing of `lib`

### Requirement: A version a served layout lacks is rendered on the request

When a served layout's client asks for a version the layout lacks, the
core SHALL resolve that one version as a pin is resolved, judge it under
the policy "The policy a version a served layout lacks is judged under"
names, and, where the policy admits it, hold that version and the
family packages it requires, by the loop a sync's render runs through
every level, and render the layout holding what it held and those, and
record them (115, 50, 137).

#### Scenario: A miss the policy admits
- **WHEN** a layout holding `1.4.0` of a name is asked for `1.3.0`, which the store names and the policy admits
- **THEN** the layout holds `1.3.0` and `1.4.0`, the record names both, and the store was asked about `1.3.0` alone

#### Scenario: A miss that requires a name the layout lacks
- **WHEN** a layout holding `@family/widget` `1.4.0` is asked for `@family/widget` `1.3.0`, which the policy admits and which requires `@family/base` at `^2.0.0`, of which the store names `2.0.0` and `2.2.0`, both admitted
- **THEN** the layout and its record hold `@family/widget` `1.3.0` and `1.4.0` and `@family/base` `2.2.0`, and the store was asked about `@family/widget` `1.3.0` and the versions of `@family/base` alone

#### Scenario: A miss the policy refuses
- **WHEN** a layout is asked for a version whose index lacks the claim the policy requires first
- **THEN** the answer names the version and that claim as wanting, and the layout and its record are unchanged

#### Scenario: A miss the store does not name
- **WHEN** a layout is asked for a version the store holds no name for
- **THEN** the answer is absent naming the name, never refused, and the layout is unchanged

#### Scenario: A miss while the store is unreachable
- **WHEN** a layout is asked for a version while the store cannot be reached
- **THEN** the answer is unreadable carrying the store's own words, and the layout is unchanged

## ADDED Requirements

### Requirement: A projection with no pins is passed over

A projection whose pinned answers absent SHALL be passed over, and no
layout of it SHALL be rendered or recorded.

#### Scenario: Two projections, one with no pins

- **WHEN** a checkout binds two projections, one whose pinned answers its lockfile's pins and one whose pinned answers absent, and is synced
- **THEN** the first's layout alone is rendered and recorded, and the answer names no layout of the second

### Requirement: Each version judged within a range is answered as a pin is

Each version judged for a range SHALL be answered as a pin is, held or
refused naming the claim found wanting, and a refused one SHALL count
against the sync only where its range holds no version (99).

#### Scenario: A refused version above the one held

- **WHEN** a range within which the store names 1.1.0, which the policy admits, and 1.2.0, which it refuses, is synced
- **THEN** 1.2.0 is answered refused naming the claim found wanting, 1.1.0 is held, and the refusal does not count against the sync

#### Scenario: Every version within a range refused

- **WHEN** a range within which the policy refuses every version the store names is synced
- **THEN** each is answered refused naming the claim found wanting, and the refusals count against the sync

### Requirement: Every version is judged before anything is written, and no manifest is read by the core

Every version SHALL be judged before anything is written, in a fixed
order, so the same store under the same policy holds the same versions
(48). The core SHALL never read a manifest itself (core.1).

#### Scenario: The same store synced twice

- **WHEN** one checkout is synced twice against the same store under the same policy
- **THEN** both syncs hold the same versions, and each judged every version before it wrote anything

### Requirement: The policy a version a served layout lacks is judged under

A version a served layout lacks SHALL be judged under the policy
`timber sync` with no argument would use in the checkout the layout
belongs to, found as a sync finds it: the consuming repository's own
where its file writes one, else the publisher's layout policy the
version's index carries, else the given (113, C.10).

#### Scenario: A miss judged under the publisher's layout policy

- **WHEN** a served layout of a checkout whose file writes no layout policy is asked for a version whose index carries the publisher's layout-policy claim
- **THEN** the version is judged under the publisher's layout policy, as `timber sync` would judge it in that checkout

### Requirement: A version a served layout lacks is answered alone, and leaves the layout where it fails

The answer to a version a served layout lacks SHALL be the one version
held, refused naming the claim found wanting, absent, or unreadable
with the source's own words (47, 51, 99). A version refused, absent or
unreadable SHALL leave the layout and its record as they were. Only the
one version asked, and the names it requires that the layout lacks,
SHALL be read from the store.

#### Scenario: A miss held that requires nothing

- **WHEN** a served layout is asked for a version the store names and the policy admits, which requires nothing of the family
- **THEN** the answer is that version held, and the store was asked about that version alone
