## MODIFIED Requirements

### Requirement: A promotion moves a channel to an accepted version

A promotion SHALL name one channel, one version of one component and
the branch whose landed head holds the policy. It SHALL move the channel only when the version's index carries an
acceptance claim that counts under the channel's policy (31, 9) and, in
the order the policy lists them, every claim the channel's policy
requires counts; otherwise it SHALL be refused naming the first claim
found wanting, and nothing SHALL be signed, appended or written (58,
core.7).

#### Scenario: A version with every claim the channel requires

- **WHEN** a version's index carries the gate's acceptance and a rebuild claim by a second signer, and the policy for `stable` requires both
- **THEN** the promotion moves `stable` to that version, and the answer names the claims that satisfied the policy

#### Scenario: A version not accepted

- **WHEN** a version's index carries no acceptance claim by a signer the channel's policy names for acceptance
- **THEN** the promotion is refused naming acceptance as the claim found wanting, and the ledger and the store are not written

#### Scenario: A claim the channel requires is wanting

- **WHEN** the policy for `stable` requires a rebuild claim and the index carries none that counts
- **THEN** the promotion is refused naming the rebuild claim and the signers that may make it

#### Scenario: A version never published

- **WHEN** the version's name resolves to nothing in the store
- **THEN** the answer is absent naming the version, and nothing is written

#### Scenario: A channel of another component

- **WHEN** the repository declares `lib` and `web`, only `web` has a channel `edge`, and `edge` is promoted for `lib`
- **THEN** the answer is absent naming `edge` and `lib`, and nothing is read of the version or written

### Requirement: The ledger holds the move before the store

A promotion SHALL append its claim to the channel's chain, naming the
head it read as predecessor, before it attaches the claim to the index
or moves the channel's name in the store; a promotion the ledger does
not hold SHALL write nothing to the store (33, 39, core.15). A store
write that fails after the append SHALL be reported beside the
promotion, which stands.

#### Scenario: The order of the writes

- **WHEN** a promotion is made over the fakes
- **THEN** the ledger's append comes before the store's attach, and the attach before the name's move

#### Scenario: The store cannot be written after the append

- **WHEN** the store's attach is unreadable after the ledger held the promotion
- **THEN** the answer is the promotion, with the store's unreadable answer beside it, and the chain holds the claim

### Requirement: A channel moves only from where it is

A promotion SHALL read the chain's head as the version the channel moves
from. Given the version it expects the channel to be at, a promotion
whose expectation is not the head's version SHALL be refused naming the
version and index the channel is at, or that it is empty (35). A chain
that holds a fork SHALL refuse the promotion naming the fork, and
nothing SHALL be appended or written (105).

#### Scenario: S7, a promotion from a version the channel has left

- **WHEN** a promotion names `stable` as at 0.1.0 and `stable` has since moved to 0.2.0
- **THEN** it is refused naming 0.2.0 and its index, nothing is appended, and nothing is retried

#### Scenario: Another promotion got there first

- **WHEN** another promotion is appended to the chain between this promotion's read of the head and its append
- **THEN** the answer is the ledger's refusal naming the new head, the ledger is asked to append once, and the store is not written

#### Scenario: A forked chain

- **WHEN** the channel's chain holds two claims naming one predecessor
- **THEN** the promotion is refused naming both claims, and nothing is appended or written

#### Scenario: A promotion to where the channel is

- **WHEN** a promotion names the version the channel's head is already at
- **THEN** the answer says the channel is there, nothing is appended, and the store is brought to the ledger

### Requirement: A channel is read from the ledger alone

The channel's state SHALL be answered from its chain through the ledger
seam alone: its head's version and index, the version it moved from,
and every promotion in order with what its claim records; a head that
holds no promotion claim of timber's form SHALL be unreadable naming
it. Reading a channel SHALL ask the store nothing and write nothing
(101, 60, core.32).

#### Scenario: A promotion read on another machine

- **WHEN** a promotion is made on one machine through a git ref ledger, and a second clone with an empty store reads the channel
- **THEN** the second clone answers the same head and the same chain, and its store is asked nothing

#### Scenario: A channel never promoted

- **WHEN** a channel whose chain is empty is read
- **THEN** the answer is that the channel is empty, which is an answer and not an error (31)

#### Scenario: An empty channel read as last read

- **WHEN** the ledger answers a channel's chain as last read, holding no claim, with the time of that read and its words
- **THEN** the answer is that the channel is empty, with that time and those words, and status shows it `empty, as last read` at that time with the words

## ADDED Requirements

### Requirement: A promotion finds its channel and resolves its version first

A promotion SHALL find the channel among the channels the policy lists
at the branch's landed head for its component, absent naming the
channel and the component where none is listed (125, 131); it SHALL
resolve the version's name in the store to its index before anything
else is read of it, and SHALL read the channel's policy at the branch's
landed head (3, 65, core.4, core.22).

#### Scenario: The version resolved before it is read

- **WHEN** `stable` is promoted to 0.2.0, whose name resolves to index `D`
- **THEN** the promotion resolves 0.2.0's name before reading anything of it, reads its claims from `D`, and reads `stable`'s policy at the landed head

### Requirement: The channel's name in the store is moved after the append

After the append, the channel's name in the store SHALL be moved from
the version the channel moved from to the version it moved to, a race
lost on it reported and repaired from the ledger (109).

#### Scenario: The name follows the chain

- **WHEN** `stable` is promoted from 0.1.0 to 0.2.0
- **THEN** after the ledger's append, `stable`'s name in the store is moved from 0.1.0's index to 0.2.0's

### Requirement: An append the ledger refuses is the answer, never retried

An append the ledger refuses as not after the head SHALL be the answer,
naming the head, and SHALL never be retried (core.6, core.34).

#### Scenario: The ledger refuses the append

- **WHEN** the ledger refuses a promotion's append as not after the head, naming the head
- **THEN** the promotion answers that refusal naming the head, and asks the ledger to append nothing more

### Requirement: A channel read as last read says when

Where the ledger answered the chain as last read, a channel's state
SHALL say when and carry the source's words, an empty channel as any
other (60 as amended at boomdrivedev/blueprints b64d367).

#### Scenario: A promoted channel read as last read

- **WHEN** the ledger answers `stable`'s chain as last read, holding two claims, with the time of that read and its words
- **THEN** the answer is `stable` at its head's version and index, with that time and those words
