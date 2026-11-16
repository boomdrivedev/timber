---
number: 37
title: Each open change is validated as it would archive, and the specs pass the strict check first
status: proposed
date: 2026-11-16
decision-makers:
- Chuck Swanberg, by boomdrivedev/blueprints principle 16 as amended on 2026-11-15
- boomdrive-design, by its ruling of 2026-11-16
- the code stage of the chore timbers-checks-catch-changes-that-wont-archive
---

# Each open change is validated as it would archive, and the specs pass the strict check first

Status: proposed, until the user's yes on it as a record. Point 1 is the user's rule, boomdrivedev/blueprints principle 16 as amended on 2026-11-15. Points 2 to 5 are boomdrivedev/boomdrive's record 0050, the family's one design for this check, which timber takes unchanged. Points 6 and 7 are boomdrive-design's ruling of 2026-11-16. Points 8 and 9 are the code stage's own, made on the rules written and cheap to change.

Deciders: the user, whose answer of 2026-11-15 on their card `req_278df7ab2cc34988a3f5` amended principle 16: "OpenSpec's own validation stays; the rule is about checks of what pages say". The principle as amended keeps "OpenSpec's `openspec validate` of a kit's specs and its open changes, each change validated as it would archive". The brief of the chore `timbers-checks-catch-changes-that-wont-archive`: "Timber's checks fail a change OpenSpec would refuse to archive, such as one with a requirement over 500 characters, so it's caught while the code is written instead of at the end (as Boomdrive's and Flume's checks do)". boomdrive-design's ruling of 2026-11-16, on the code stage's finding that timber's specs fail `--strict`: "fix timber's specs first, in this chore, then add the family's check unchanged. Do not build a timber variant." The code stage, for points 8 and 9.

Sources: boomdrivedev/blueprints `design/principles.md` 16 at `3a5c19d`; boomdrivedev/boomdrive record 0050 and core.61, and its `check-changes.sh` at `d9414da6`; boomdrivedev/flume record 0073 and core.59, and its `check-changes-cases.sh` at `cdc3ddf3`; boomdrivedev/sluice record 0128 and core.61; OpenSpec 1.14.3, the release `devenv.nix` pins, its `openspec archive` and `openspec validate`, read in scratch trees on 2026-11-16; the change `timbers-long-requirements-are-split`; rules core.23, core.25 and core.26.

## Context and problem statement

timber's gate ran no OpenSpec validation. Boomdrive, Flume and sluice each run `root:specs`, `openspec validate --specs --strict`, and `root:changes`, which holds each open change to archiving as it is.

`openspec archive` checks the specs it rebuilds without `--strict`. A change whose requirement runs past OpenSpec's 500 characters, or which adds a capability with no `## Purpose`, archives with a warning and nothing fails. timber's specs carried what such changes wrote: on 2026-11-16, 30 of its 49 specs failed `--strict`, with 93 requirements past 500 characters and 12 specs keeping the placeholder Purpose archive writes for a new capability.

The family's check holds every spec a change's archive writes to `--strict`. Over timber's specs as they were, it failed a change that added one short, valid requirement to `tools-layer`, on `tools-layer`'s older long requirement. A long requirement can be split by a MODIFIED delta. But OpenSpec 1.14.3 reads a delta's `## Purpose` only when it creates the capability, so no change could fix the 12 Purposes, and no change could touch those 12 specs at all.

## Considered options

For the specs that fail `--strict`:

1. **Fixed first, in the chore, and the family's check added unchanged** (the ruling).
2. A timber variant of the check that fails only what a change's archive adds.

For the 12 placeholder Purposes:

1. **Written straight into their specs, each from what its requirements say** (the ruling).
2. Each capability created again under a new name by a change whose delta carries its Purpose.

For proving the check in timber's own gate:

1. **Flume's cases, `root:changes-cases`, beside `root:changes`.**
2. `root:changes` alone, as Boomdrive and sluice run it.

## Decision outcome

1. **The gate holds the specs and each open change to OpenSpec's validation as it would archive (the user's).** `root:specs` runs `openspec validate --specs --strict`. `root:changes` runs `openspec validate --changes --strict`, then, for each open change, `openspec archive` in a scratch copy and `openspec validate --strict` of each spec that results. A change that fails is named with OpenSpec's words. `check-changes.sh` does it, and core.77 says so.
2. **Each change is applied alone to a scratch copy of the tree's specs (record 0050).** The copy holds the tree's specs, `openspec/config.yaml` and that change. Two open changes that conflict are found by the gate on the branch that lands second.
3. **The specs the archive writes pass `openspec validate --strict`, though `openspec archive` validates them without it (record 0050).**
4. **Only the specs the archive writes are validated (record 0050).** It knows which by comparing each with the tree's byte for byte. A spec the change leaves as it was is `root:specs`' to check.
5. **A task left unchecked fails nothing (record 0050).** A change is open while its tasks are worked.
6. **timber's specs are made to pass `--strict` before the check holds them (option 1, the ruling).** The 93 long requirements are split through the change `timbers-long-requirements-are-split`, which changes nothing any requirement asks for: each split keeps its name, its scenarios and its meaning, and the text cut away becomes ADDED requirements, each with a scenario that instances its text. Its design says how.
   - Not taken: option 2. It departs from record 0050 and leaves 30 specs failing `--strict` for good.
7. **The 12 placeholder Purposes are written straight into their specs (option 1, the ruling).** A Purpose states no requirement, and it is what sluice and Flume give a created spec in its archive commit. The rule that specs change only through OpenSpec holds for every requirement and scenario. From here on `root:changes` fails a change that creates a capability with no Purpose, so no placeholder lands again.
   - Not taken: option 2. It renames 12 capabilities to work around OpenSpec, and leaves each old name behind as a spec with no requirements.
8. **`check-changes.sh` is Boomdrive's at `d9414da6`, copied whole (the code stage's).** timber's gate runs files its tree holds, at versions it pins (core.26), as Flume's and sluice's do. Its header cites this record. A change to the design is made in Boomdrive's record 0050 first and carried here (CLAUDE.md, "The family").
9. **Flume's cases run as `root:changes-cases` (option 1, the code stage's).** Once this chore's change is archived, timber's tree holds no open change, so `root:changes` would check nothing until the next one. Rule core.25 rules out a task whose only verification is a command run by hand, and allows a test that writes Markdown into a scratch tree as data. The cases check the script's exit and words over five scratch trees: a change that does not validate, one archive refuses, a new capability with no Purpose, a MODIFIED requirement past 500 characters, and two valid changes with open tasks.
   - Not taken: option 2. It leaves the check proven only by a run over whatever changes are open.

### Consequences

- Good: a change that would not archive, or would archive into specs `root:specs` fails, fails its own branch's gate, named with OpenSpec's words, while it is written.
- Good: all 49 specs pass `openspec validate --strict`, and `root:specs` keeps them so.
- Good: a delta that adds a capability has to carry its own `## Purpose`.
- Bad: `root:changes` runs `openspec` once more for each open change and for each spec its archive writes, and `root:changes-cases` runs it over five trees.
- Bad: three copies of the script can drift; the design is record 0050's to change first.
- An open change on another branch that MODIFIES a split requirement with its old text fails `root:changes` there, and is cut as `timbers-long-requirements-are-split` cut it.
- Rule core.77 holds the decision.
