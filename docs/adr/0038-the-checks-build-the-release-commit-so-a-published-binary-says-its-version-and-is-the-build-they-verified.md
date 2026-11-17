---
number: 38
title: The checks build the release commit, so a published binary says its version and is the build they verified
status: rejected
date: 2026-11-16
decision-makers: boomdrive-design (the record and every choice in it)
---

# The checks build the release commit, so a published binary says its version and is the build they verified

Status: rejected by the user on 2026-11-17, in boomdrive-design's pane: "This seems completely contradictory to how I have been told that Timber is supposed to work." It breaks timber's core, clause 24: the build never knows its version, so the checks build once and a publish names those same bytes with a version, never building them again. Where timber publishes a component, a compiled binary carries its revision and its build identity, and its version is the name timber's index gives those bytes (24, C.11). Nothing below binds.

Deciders: boomdrive-design, on blueprints-a4's question of 2026-11-16. The user said that day, in blueprints-a4's session, that timber is used in every repository of the family, flume and Boomdrive among them. That makes timber the publisher of the family's own binaries, which record 0031 left to "that unit's design".

Sources: record 0031 and `core.74` (main says `0.0.0`; a release's tag names a release commit off main that writes the version; a build takes its version from its tree alone; the binary says its version); boomdrivedev/blueprints `design/principles.md` 14; boomdrivedev/blueprints `design/timber/requirements.md` 21, 22, 24, 135 and C.11.

## Context and problem statement

Two rules meet when timber publishes a compiled tool such as flume or timber itself. Record 0031, after principle 14, has a released binary say its version, read from a release commit's tree. Clauses 24 and 135 have a publish offer the very build the landing's checks ran, and clause 24 has that build never know its version. Checks build main's tree, which says `0.0.0`, so the build they verify can't say the release's version. Either the published binary isn't the one checked, or it can't say what release it is.

## Considered options

1. **The checks build the release commit (chosen).** For a landing that makes a version, the check builds the release commit record 0031 makes, at the version the rule derives for that landing. The verified build says its version, and the tag names that same commit.
2. **The checked build reckons its version from the tags as they stand.** Two landings in the queue at once both reckon from the same newest tag and both claim one version, since the first one isn't tagged until after it lands and publishes.
3. **Two builds.** The check verifies main's `0.0.0` build, and a second build of the release commit is what's published. The published bytes are then ones no check verified, against clause 135 and I3.
4. **The binary says only its revision, and timber says the version.** Record 0031 ruled this out under principle 14: a tool's binary says its own version.

## Decision outcome

Option 1.

- **A landing's version is a function of its history alone.** Counting from the component's newest tag, each landing on the branch's first-parent line that touches the component's paths, and that the rule bumps, is a version of its own, tagged yet or not. A candidate in the queue therefore derives the same version before and after it lands, two candidates never derive one version, and no version waits on an earlier publish having run.
- **The check builds the release commit.** For a candidate that makes a version, the gate's build task makes the release commit record 0031 makes (one parent, the candidate; one tree; fixed identities and dates, so one id everywhere) and builds that. Its verdict names that build. A candidate that makes no version builds its own tree, as now.
- **The publish tags the commit the check built.** It offers the verified build's outputs, never builds them again, and tags the release commit off main. Main's tree still says `0.0.0` (`core.74`).
- **A build still takes its version from its tree alone.** The release commit's tree names the version, so the same tree gives the same build whatever host builds it. That includes a consumer building the tag, such as Boomdrive's set, which then hits the cache the check filled.

### Consequences

- Clause 24 and C.11's sentence on compiled binaries change: a build knows a version only where its tree names one, and a release commit's tree does. Clause 21's rule names the per-landing count.
- A candidate that fails and is queued again in another position may derive another version, so its build is made again. A version is claimed only at publish (22), so nothing is claimed twice.
- A landing outside a queue, one GitHub's job publishes on a push, has no verdicts, so its publish builds the release commit through the build seam, as 135 has it.
