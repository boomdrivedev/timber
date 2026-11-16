## MODIFIED Requirements

### Requirement: Status shows every channel the policy declares

Status SHALL answer, for the branch asked (by default `main`), the
landed revision at the branch's head that its policy was read at, and
every channel the policy at that revision declares, as the policy seam
lists them (131): each with the component it belongs to (125), how it
moves, and the channel it is bound to where it is one (37, 83), and
either the one version its chain's head moved it to, by version and
index, or that it is empty (31, 101).

#### Scenario: Two channels, one promoted

- **WHEN** the policy declares `beta` and `stable` of one component, and the ledger holds two promotions of `stable` and none of `beta`
- **THEN** status names both, `beta` empty and `stable` at the version and index of its second promotion, each with its component and how it moves

#### Scenario: A host that never published

- **WHEN** status is asked on a host whose record holds nothing, of a repository whose ledger holds a promotion
- **THEN** the channel is shown at its version, and the record's part of the answer is absent naming the record, apart from the channels

#### Scenario: A policy that declares no channel

- **WHEN** the policy at the landed head declares no channel
- **THEN** status answers an empty list of channels, never absent

#### Scenario: A head not landed

- **WHEN** the local branch's head is a revision the landing has not landed
- **THEN** the channels' part is refused naming the branch and the revision, and the record's part still answers

### Requirement: Every reader gets one answer as of when it read

Status SHALL say when it asked, in seconds since the Unix epoch and as
a person reads a time in UTC, and every reader of one ledger SHALL see
one version per channel (36). Where the ledger answers a chain as this
host last read it, status SHALL show that chain, saying when it was
read and carrying the remote's or store's own words verbatim, and SHALL
never show it as refused or as empty (36, 60, 61, C.13, C.14).

#### Scenario: Two machines, one answer

- **WHEN** one machine promotes and a second machine, which never published, asks status of the same repository
- **THEN** both answer the same version and chain for every channel

#### Scenario: The ledger's remote cannot be reached

- **WHEN** the landing answers the branch's head landed, and the ledger's remote cannot be reached after this host last fetched the chain
- **THEN** the channel is shown at the version of the chain as last fetched, with when it was fetched and the remote's words, and nothing is shown refused

#### Scenario: The landing cannot be read

- **WHEN** the landing answers unreadable with its source's words
- **THEN** the channels' part is unreadable carrying those words verbatim, never refused, the record's part answers, and status exits 0

### Requirement: A repository's row is made from its status

The core SHALL make one repository's row from its status answer alone:
the repository's owner and name, and for each channel shown its
component, its name and its head, the version it is at, empty, or that
channel's own answer as status gives it; for the environments, the
environments' part's four counts, how many run what they should, how
many are behind, how many wait on a person and how many cannot be read,
or the environments' part's own answer where it gives no counts (132,
58).

#### Scenario: A row of heads

- **WHEN** a repository's status names `beta` empty and `stable` at 0.2.0, with one environment running what it should and one behind
- **THEN** its row names `beta` empty, `stable` at 0.2.0 with its component, and the counts one, one, none and none

#### Scenario: A row whose channels cannot be read

- **WHEN** a repository's status answers its channels' part unreadable
- **THEN** its row carries that unreadable answer, with the source's words, in place of the heads

## ADDED Requirements

### Requirement: Status reads each channel from the ledger, and one component's when asked

A channel's version SHALL be read from the ledger alone, never from a
name in the store (101, 106). Asked for one component, status SHALL
show that component's channels alone.

#### Scenario: A name in the store behind the chain

- **WHEN** `stable`'s name in the store names 0.1.0's index while its chain's head moved it to 0.2.0
- **THEN** status shows `stable` at 0.2.0 and its index

#### Scenario: One component asked

- **WHEN** the repository declares `lib` and `web`, each with a channel `beta`, and status is asked for `lib`
- **THEN** status shows `lib`'s `beta` alone

### Requirement: Status takes the landing's answer as the landing seam gives it

Status SHALL take the landing's answer as the landing seam gives it:
where the landing cannot be read, the channels' part SHALL be unreadable
carrying the landing's words, never refused, and the record's part
SHALL still answer (58, 59, 77).

#### Scenario: An unreadable landing beside a record

- **WHEN** the landing answers unreadable with its source's words, and the host's record holds two versions
- **THEN** the channels' part is unreadable carrying those words, and the record's part answers both versions

### Requirement: A row carries the channels' own answer, and no surface makes one

Where the channels' part is not answered, the row SHALL carry that
answer in place of the heads. A surface SHALL compute no row of its own
(56, core.31).

#### Scenario: A row whose channels were refused

- **WHEN** a repository's status answers its channels' part refused, naming the branch and a revision the landing has not landed
- **THEN** its row carries that refusal in place of the heads

#### Scenario: The served root's row

- **WHEN** the serving process answers a repository's row at its root
- **THEN** the row is the core's row of that repository's status answer, value for value
