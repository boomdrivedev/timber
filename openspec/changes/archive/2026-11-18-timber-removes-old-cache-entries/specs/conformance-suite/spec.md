## ADDED Requirements

### Requirement: The store's group lists and removes cache entries

The store group SHALL hold, below its line, scenarios citing 145, 54
and 100: an entry listed with its push moment between the moments
before and after it was made; an entry removed, another entry and an
artifact of the same bytes left; a removal from where the name is not
refused; and a removal of no entry absent. Every store rig SHALL bind
its store for a build profile, the registry's for `moon` (rule core.33).

#### Scenario: The group over the fake store

- **WHEN** the store group runs over the fake store
- **THEN** every cache scenario passes and names 145 among its clauses

### Requirement: Prune over the fakes

The core's cache group SHALL hold, below its line, scenarios citing
145, 22, 34 and 54: an entry past its time removed and one within kept;
an entry with no moment removed; the keeping time read from the policy
at the landed revision; and a version published, accepted and promoted
read back whole after its entries are removed.

#### Scenario: The cache group over the fakes

- **WHEN** the core's cache group runs over the fakes
- **THEN** every prune scenario passes, and none cites a clause outside Parts A and B
