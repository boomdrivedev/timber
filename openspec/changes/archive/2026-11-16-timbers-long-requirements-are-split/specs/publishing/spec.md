## MODIFIED Requirements

### Requirement: A publish refuses a revision not landed, before anything else

A publish SHALL ask the landing seam whether the branch has landed the
revision before it reads the policy file, asks the build, packs, puts or
signs anything. A revision the branch has not landed SHALL be refused
naming the branch and the revision, and nothing SHALL be read or offered
(14, core.11).

#### Scenario: A revision the branch has not landed

- **WHEN** a revision the fake landing has not landed on the branch is published
- **THEN** the answer is refused naming the branch and the revision, and the fake build, projections, signer and store record no call

#### Scenario: The landing cannot be read

- **WHEN** the fake landing answers unreadable for the branch and revision
- **THEN** the publish answers that unreadable answer with the landing's words, and nothing is offered

#### Scenario: Not landed as last fetched

- **WHEN** the fake landing answers not landed from its copy as last fetched, with a time and the remote's words
- **THEN** the publish is refused naming the branch, the revision, that time and those words verbatim, saying to ask again, the command line exits as for any refusal, and the fake build, projections, signer and store record no call

#### Scenario: Landed as last fetched

- **WHEN** the fake landing answers landed from its copy as last fetched
- **THEN** the publish goes on as for a revision landed on the remote, and its answer equals that one's

### Requirement: A second publish of a revision ends at the same index

A publish of a revision whose version's name already names an index,
or for a declared component whose bare tag does, as a publish before
version names carried their component left it, SHALL compare each packed artifact's digest with the digest the index
names for that ecosystem name. All the same, it SHALL attach the
publisher's layout policy where the index lacks it, make each artifact's
name the store lacks at that index, write nothing else, and answer the
same index (5, 18, 113, 138, S2).

#### Scenario: The same revision again

- **WHEN** a revision is published twice with projections that pack the same
- **THEN** the second answer names the same index and digests, and the store, the history and the host's record hold after it exactly what they held before it

#### Scenario: A version named before its artifacts were

- **WHEN** the store holds a version's index and its tag but no artifact's name and no layout-policy claim, as a publish before this change left it, and the revision, whose file writes a policy for layouts, is published again
- **THEN** the layout-policy claim is attached to that index, each artifact's name is then made from nothing at it, nothing is put, and the answer names the same index

#### Scenario: A projection that packs differently

- **WHEN** a revision is published twice and the fake projection packs different bytes the second time
- **THEN** the second publish is refused naming the artifact, the digest the index names and the digest packed now, the store records no put and no name, and the version's name still names the first index

### Requirement: The index carries the publisher's layout policy

Where the publishing repository's file writes a policy for layouts at
the landed revision, a publish SHALL sign it as a layout-policy claim about
the index, in the identity of the party running the publish and naming
that revision, and attach it to the index after the index is put and
before any artifact's name or the version's name is made (113, 8, 138,
core.8, core.9).

#### Scenario: A publish under a policy for layouts
- **WHEN** a revision whose policy file declares a policy for layouts is published
- **THEN** the index carries one layout-policy claim by the publishing host naming the revision and the claims that policy requires, attached before the first artifact's name was made

#### Scenario: A publish with no layout line
- **WHEN** a revision whose policy file names a gate signer and no `[layout]` is published
- **THEN** the index carries no layout-policy claim, and the given is left for a consumer to lay beneath itself

#### Scenario: A publish with no policy file
- **WHEN** a revision whose tree holds no policy file is published
- **THEN** the index carries no layout-policy claim, and the given is left for a consumer to lay beneath itself, and the publish answers as before

#### Scenario: A repeat publish
- **WHEN** the same revision is published again and ends at the same index
- **THEN** the index still carries one layout-policy claim

## ADDED Requirements

### Requirement: A landing answered from the host's copy

Where the landing answered not landed from the host's copy as last
fetched, the refusal SHALL also name when the copy was fetched, carry
the remote's words and say to ask again once the remote answers (77). A
revision the landing answered landed from such a copy SHALL be
published as one it answered landed from its remote.

#### Scenario: Not landed while the remote cannot be reached

- **WHEN** the landing answers a revision not landed from the host's copy fetched at a time, with the remote's words that it cannot be reached
- **THEN** the publish is refused naming the branch, the revision, that time and the remote's words, and says to ask again once the remote answers

### Requirement: A second publish that packs differently is refused

Where an artifact the second publish packs differs from the digest the
index names for its ecosystem name, the publish SHALL be refused naming
the artifact and both digests, nothing SHALL be offered or named, and
the name SHALL stay with the first index (18, 87).

#### Scenario: One of two artifacts packed differently

- **WHEN** a revision of two outputs is published twice, and the second time one output's projection packs different bytes
- **THEN** the second publish is refused naming that artifact and both its digests, neither artifact is offered or named, and the version's name stays with the first index

### Requirement: A revision with no layout policy carries none, and none is attached twice

Where the publishing repository's file writes no policy for layouts at
the landed revision, or the revision holds no policy file, a publish
SHALL attach no layout-policy claim. A publish whose index already
carries that claim SHALL attach nothing more (113).

#### Scenario: Published twice with no layout line

- **WHEN** a revision whose policy file writes no `[layout]` is published twice
- **THEN** the index carries no layout-policy claim after either publish
