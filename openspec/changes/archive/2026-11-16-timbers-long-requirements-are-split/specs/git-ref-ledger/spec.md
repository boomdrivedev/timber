## MODIFIED Requirements

### Requirement: Append is a push that fast-forwards

Append SHALL hold the claim at the head only when the predecessor given
is the claim at the head as the remote holds it, or none and the remote
holds no ref; it SHALL write the claim's commit on the head's and push
it so that the remote takes only a fast-forward, never a forced update
(120, 122). Where no remote is bound, the repository's own ref SHALL
be the ledger, moved only from the commit it was read at.

#### Scenario: Two hosts append after one head

- **WHEN** two clones each read `stable` at one head and each append a promotion naming it
- **THEN** the first push is held, the second is refused naming the first's claim, and the remote's chain holds the first alone after the head

#### Scenario: A remote that refuses for its own reasons

- **WHEN** the remote refuses the push while its head is still the predecessor given
- **THEN** the append is refused carrying git's words verbatim, and the chain is unchanged

### Requirement: A remote that does not answer is read as last fetched

Where the remote cannot be reached, chain SHALL answer the claims of
the repository's own copy of the ref as last fetched or appended, saying
when it was last written and carrying git's words of why the remote did
not answer (60, C.13); with no copy of the channel at all, the chain
SHALL be unreadable with git's words. Append SHALL be unreadable with
git's words, and SHALL never append to the copy alone where a remote is
bound.

#### Scenario: The remote gone after a read

- **WHEN** a clone has read `stable`, and its remote's path is then removed
- **THEN** the chain answers the claims it read, the time of that read and git's words, and an append is unreadable

#### Scenario: An empty channel as last fetched

- **WHEN** a clone has read `beta` while its remote held no promotion of it, and its remote's path is then removed
- **THEN** the chain answers no claims, the time of that read and git's words, and not absent

#### Scenario: The keeper's group

- **WHEN** the keeper's conformance group runs over the git ref ledger with a bare remote in a temporary directory
- **THEN** every scenario passes

## ADDED Requirements

### Requirement: A push the remote refuses is refused naming the remote's head

A push the remote refuses SHALL be refused naming the claim at the
remote's head, read again after the refusal; where the head is still
the predecessor given, the refusal SHALL carry git's words verbatim
(59, 120).

#### Scenario: The remote's head read again after a refusal

- **WHEN** a clone appends a promotion naming the head it read, and the remote refuses the push because another clone's claim now follows that head
- **THEN** the append is refused naming the other clone's claim, read from the remote after the refusal

### Requirement: An empty channel is read as last fetched as any other chain

Where the remote cannot be reached and the copy last read the channel
holding no claim, chain SHALL answer it empty, saying when it was read
and carrying git's words, as any other chain as last fetched (60, as
amended at boomdrivedev/blueprints b64d367).

#### Scenario: An empty copy and no remote

- **WHEN** a clone's copy last read `beta` holding no claim, and its remote cannot be reached
- **THEN** the chain of `beta` is empty, with the time of that read and git's words, and not unreadable
