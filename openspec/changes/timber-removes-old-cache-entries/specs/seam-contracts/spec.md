## ADDED Requirements

### Requirement: The store lists and removes cache entries

The store seam SHALL list every cache entry it holds for the repository
and build profile it is bound for, each by its build identity with the
digest its name is at and its push moment where the store holds one,
and an entry it cannot read answered on its own. It SHALL remove one
entry by its build identity, only where its name is at the digest
given, and no other name or bytes another name reaches (145, 54, 100).

#### Scenario: An entry listed

- **WHEN** a store holds a cache entry made at some bytes
- **THEN** the listing names its build identity, the bytes' digest and its push moment

#### Scenario: A removal from where the name is not

- **WHEN** a cache entry's removal names a digest the entry's name is not at
- **THEN** it is refused naming the digest the name is at, and the entry is still listed

#### Scenario: A removal of no entry

- **WHEN** a removal names a build identity the store holds no entry for
- **THEN** it is absent naming the entry

### Requirement: The policy answers how long cache entries are kept

The policy seam SHALL answer, for a landed revision, how many days the
repository keeps its cache entries from their push, and whether that is
the givens' time, which is thirty days (145, C.10, rule core.43).

#### Scenario: No time written

- **WHEN** the policy at a revision writes no keeping time
- **THEN** it answers thirty days, given

### Requirement: A cache entry's store name is spelled once

A cache entry's store name SHALL be `cache/` and its build identity,
made by the seams' one function, which the core and every store profile
call (52, rule core.48).

#### Scenario: The name of an entry

- **WHEN** the seams spell the name of the cache entry of build identity `b7e1`
- **THEN** it is `cache/b7e1`
