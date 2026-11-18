## ADDED Requirements

### Requirement: A publish reads its root before anything is built

A publish SHALL, after the landing check and the policy file's
declaration and before any component is derived, built, claimed or
offered, read the root its store is bound for and the last root the
record of claimed versions names (22, 62, C.1). Where the store places
no name by repository it SHALL read and write neither. An empty record
SHALL be written the bound root, and a record whose last root is the
bound one SHALL be written nothing.

#### Scenario: A first root recorded

- **WHEN** a repository whose record names no root publishes to a store bound for `acme/site`
- **THEN** the record names `acme/site` before the version's name is claimed, and the answer says the root was recorded

#### Scenario: The same root

- **WHEN** a repository whose record's last root is `acme/site` publishes to a store bound for `Acme/Site`
- **THEN** nothing is carried or recorded, and the answer says nothing of the root

#### Scenario: A layout

- **WHEN** a repository publishes to a layout
- **THEN** the record is neither read nor written

### Requirement: A moved repository's first publish carries its history

Where the record's last root is another than the bound one, the publish
SHALL mark the root left with the bound one, then copy into the bound
root every version the record claims, with every claim on its digests,
then make each channel's name there at its chain's head, then record
the bound root, all before any component is built (C.1, 103, 105). No
cache entry SHALL be carried.

#### Scenario: A move carried

- **WHEN** a repository that published `v1.2.0` and moved `beta` to it under `acme/app` publishes from a checkout whose remote now names `acme/site`
- **THEN** `acme/app` holds the mark naming `acme/site`, `acme/site` holds `v1.2.0` with its acceptance and promotion and `beta` at its index, and the record's last root is `acme/site`, each before the new version is claimed

#### Scenario: A version published before the move promoted after it

- **WHEN** after the carry the gate promotes `v1.2.0` to `stable` at `acme/site`, `stable` being at `v1.1.0` under `acme/app`
- **THEN** the promotion is appended after the head `stable` had under `acme/app`

#### Scenario: Cache entries stay

- **WHEN** a move is carried from a root whose cache repository holds entries
- **THEN** nothing of the cache is read or written

### Requirement: Each carried chain goes on from its head

A carry SHALL read each channel's chain from the root left as the
source holds it, a chain answered as last read being unreadable. Where
the channel's name at the bound root is elsewhere than the head left,
the bound chain SHALL hold that head, or the carry SHALL be refused as
a fork naming both heads (105). After the carry, each bound chain's head
SHALL be the head left or a claim after it.

#### Scenario: A chain as last read

- **WHEN** the root left's ledger answers `beta`'s chain as last read
- **THEN** the carry is unreadable with its words, and the record still names the root left

#### Scenario: A channel moved on after an earlier carry

- **WHEN** a carry runs again after `beta` was promoted at the bound root past the head left
- **THEN** `beta`'s name is left where it is, and the carry goes on

#### Scenario: Two heads

- **WHEN** `beta`'s name at the bound root is at an index whose chain does not hold the head left
- **THEN** the carry is refused as a fork naming both heads, and the record still names the root left

### Requirement: A carry writes only what is missing

A carry SHALL write only what the bound root lacks, so a carry run again
writes nothing, and a carry that stopped partway, which leaves the
record at the root left, SHALL be run again by the next publish and
write what the last one did not (core.6).

#### Scenario: A carry again

- **WHEN** a carry runs again after one that finished all but recording the root
- **THEN** the mark, every version and every channel's name are answered held already, and the root is recorded

### Requirement: A publish under a root left is refused

Where the root the store is bound for holds a mark naming another root,
the publish SHALL be refused naming the bound root and where it moved,
before anything is carried, recorded, claimed or offered. This SHALL
hold for another repository at an address one left, a checkout whose
remote still names the old address, and a repository moved back alike.

#### Scenario: A stale address

- **WHEN** a checkout whose remote still names `acme/app` publishes after the history moved to `acme/site`
- **THEN** the publish is refused naming `acme/app` and `acme/site`, and nothing is written or claimed

### Requirement: A publish answers the root it recorded or carried

A publish's answer SHALL say the root it recorded, or the roots it
carried between with each version's name, whether it was copied, held
already or not held at the root left, and each channel with its head
and whether its name was made, and SHALL say nothing of the root where
it wrote nothing (57).

#### Scenario: A carry in the answer

- **WHEN** a publish carries two versions and one channel from `acme/app` to `acme/site`
- **THEN** its answer names both roots, both versions as copied and the channel with its head
