# cache-keeping Specification

## Purpose
Removes a repository's cache entries once the keeping time its policy
gives has passed since their push, thirty days unless the policy says
otherwise, through `timber prune`, and never anything accepted or
released, the record of versions claimed or the ledger (145, 54, 22,
34).

## Requirements

### Requirement: Prune reads the keeping time at a landed revision

Prune SHALL ask the landing first and refuse a revision its branch has
not landed, naming both, before the policy or the store is asked (14).
It SHALL then read the keeping time from the policy at that revision
(65, rule core.22). Any answer but a time SHALL end prune with that
answer, the store not asked and nothing removed.

#### Scenario: A revision not landed

- **WHEN** prune is asked for a revision its branch has not landed
- **THEN** it is refused naming the branch and the revision, and neither the policy nor the store was asked

#### Scenario: A policy that cannot be read

- **WHEN** the policy at the landed revision answers the keeping time unreadable
- **THEN** prune answers that, and the store was asked nothing

### Requirement: An entry past its keeping time is removed

Prune SHALL remove each cache entry whose push moment, plus the keeping
time in days, is at or before the moment it runs, and SHALL keep every
other, naming when its time passes (145).

#### Scenario: One entry past its time and one within

- **WHEN** the policy gives thirty days, one entry was pushed thirty-one days before prune runs and another twenty-nine days before
- **THEN** the first is answered removed, the second kept naming the moment a day after, and only the first is gone from the store

#### Scenario: A keeping time the file writes

- **WHEN** the policy file writes `keep_days = 7` and an entry was pushed eight days before prune runs
- **THEN** that entry is removed, and the answer says the keeping time is seven days and written

#### Scenario: A keeping time of none

- **WHEN** the policy file writes `keep_days = 0`
- **THEN** prune removes every entry the store lists

### Requirement: An entry with no push moment is removed

Prune SHALL remove a cache entry the store lists with no push moment,
since it was pushed before push moments were written, and SHALL answer
it removed with no moment (145, 54).

#### Scenario: An entry from before

- **WHEN** the store lists an entry with no push moment
- **THEN** prune removes it and answers it removed, pushed before push times were kept

### Requirement: Each entry is answered on its own

An entry the store answers otherwise than listed SHALL be carried as
answered and not removed. Each removal SHALL be answered on its own,
one never stopping the next, and a removal answered absent SHALL be
gone already (54, rule core.19).

#### Scenario: One entry that cannot be read

- **WHEN** the store lists two entries past their time and a third it answers unreadable
- **THEN** the two are removed, the third is answered unreadable with the store's words, and no removal of it was asked

#### Scenario: A removal refused

- **WHEN** the store refuses the removal of one of two entries past their time
- **THEN** that entry is answered with the store's refusal, and the other is removed

#### Scenario: An entry gone before its removal

- **WHEN** an entry the store listed is gone by the time prune removes it
- **THEN** it is answered gone already, and prune goes on

### Requirement: Prune removes cache entries and nothing else

Prune SHALL ask the store only to list and remove cache entries, and
SHALL ask nothing of the keeper, a signer, a projection or a target.
Every name, digest and claim outside the cache space SHALL read back as
before, and prune SHALL keep nothing (145, 54, 22, 34, 61).

#### Scenario: A released version beside old entries

- **WHEN** a version is published, accepted and promoted, its cache entries are past their time, and prune runs
- **THEN** the entries are removed, every name the store held outside `cache/` resolves to the digest it did, every claim on those digests is listed as before, and the keeper, the signers, the projections and the targets were asked nothing
