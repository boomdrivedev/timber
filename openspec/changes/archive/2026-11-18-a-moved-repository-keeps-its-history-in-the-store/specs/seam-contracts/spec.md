## ADDED Requirements

### Requirement: The binding answers the roots

The binding the core is handed SHALL carry the roots: the repository its
store is bound for, or none where the store places no name by
repository, and, asked for a root the repository left, the store and
the keeper bound for it (92, C.1). A binding that moves no repository
SHALL answer no root and bind no root left.

#### Scenario: No roots

- **WHEN** a binding with no roots is asked its root and a root left
- **THEN** it answers no root, and the root left absent

### Requirement: A root moved is a refusal of its own

A read or write of a root its repository left SHALL be refused with a
reason of its own naming the root and the root it moved to, which reads
"`<from>`'s history moved to `<to>`" and round-trips through JSON (58,
59, C.1).

#### Scenario: The words of a root moved

- **WHEN** a refusal names the root `acme/app` moved to `acme/site`
- **THEN** it reads `acme/app's history moved to acme/site`

### Requirement: The mark of a root left

The mark of a root left SHALL be the store name the seam's one function
makes, `moved`, which no version's, channel's, cache entry's or
artifact's name can be, at bytes that are the canonical JSON
`{"moved_to":"<owner>/<name>"}`, the root moved to lowercased (C.1,
core.48).

#### Scenario: The mark's bytes

- **WHEN** the mark naming `Acme/Site` is written and read back
- **THEN** its bytes are `{"moved_to":"acme/site"}`, and reading them answers `acme/site`
