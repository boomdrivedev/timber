## MODIFIED Requirements

### Requirement: The suite knows the clauses of Parts A and B

The suite SHALL hold the numbers of every clause of Parts A and B, by
section, with the statement's revision they were read at. A clause the
statement adds or removes SHALL change this list in the commit that
adopts it.

#### Scenario: The clauses listed

- **WHEN** the suite's list of clauses is read
- **THEN** it holds every number from 1 to 145 exactly once, each under its section of Part A or Part B as boomdrivedev/blueprints fecf4d9 places it, 141, 142 and 143 after 13 in A.2, 144 after 20 in A.3 and 145 after 54 in A.9, and names that revision as the one it was read at

## ADDED Requirements

### Requirement: The clause the statement added at 145 is cited

Clause 145 SHALL be cited by the scenarios that adopt it, in the commit
that adopts boomdrivedev/blueprints fecf4d9, and SHALL NOT be named in
the register (rule core.33).

#### Scenario: The coverage after fecf4d9 is adopted

- **WHEN** the coverage check runs after the commit that adopts boomdrivedev/blueprints fecf4d9
- **THEN** 145 is cited by the store group's cache scenarios and by prune's, it is not in the register, and the check passes
