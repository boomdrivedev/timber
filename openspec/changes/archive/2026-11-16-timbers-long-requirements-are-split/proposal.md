# timber's long requirements are split

## Why

timber's gate holds no spec to `openspec validate --strict`, and 30 of
its 49 specs failed it on 2026-11-16: 93 requirements ran past
OpenSpec's 500 characters, and 12 specs kept the placeholder Purpose
`openspec archive` writes for a new capability. `openspec archive`
checks the specs it rebuilds without `--strict`, so each change that
wrote them archived cleanly, and nothing failed.

Boomdrive, Flume and sluice hold their specs to `--strict` in
`root:specs` and each open change to archiving as it is in
`root:changes` (boomdrivedev/boomdrive record 0050, core.61). The
family's check validates every spec a change's archive writes with
`--strict`, so over timber's specs as they were it would fail any change
touching one of those 30 specs, for text the change did not write.

The ruling of boomdrive-design of 2026-11-16, on the chore
`timbers-checks-catch-changes-that-wont-archive`: timber's specs are
fixed first, in the chore, and then the family's check is added
unchanged, with no timber variant. The 93 requirements are split
through this change, which changes nothing any requirement asks for;
the 12 Purposes are written straight into their specs, since a Purpose
states no requirement (record 0037).

## What Changes

- **Each requirement over 500 characters is split, its meaning kept.**
  Its MODIFIED block keeps its name, its place in the spec and every one
  of its scenarios word for word, its text cut to 500 characters or
  fewer at a sentence or a clause. The text cut away moves into one or
  more ADDED requirements, each stating one behaviour in the original's
  words, with a scenario of its own that instances its text and asks
  nothing more. A MODIFIED text that names behaviour an ADDED one now
  states points to it by name.
- 93 requirements of 25 specs are split, into 93 MODIFIED and 137 ADDED
  requirements. Once archived, all 49 specs pass
  `openspec validate --specs --strict`.
- **The gate holds the specs and the open changes.** `root:specs` runs
  `openspec validate --specs --strict`, and `root:changes` runs
  `check-changes.sh`, boomdrivedev/boomdrive's at d9414da6 copied whole:
  each open change passes `openspec validate --strict`, and archived
  alone into a scratch copy of the specs, writes specs that pass it.
  `root:changes-cases` runs Flume's cases of the script at cdc3ddf3.
  Record 0037 and rule core.77 say so.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

Each with its long requirements split, nothing any requirement asks for
changed: `acceptance`, `branch-landing`, `cargo-projection`,
`channel-movement`, `channel-status`, `command-line`,
`conformance-suite`, `consumer-layouts`, `environment-reports`,
`git-ref-ledger`, `herdr-plugin`, `host-bindings`, `host-key-signer`,
`host-record`, `moon-build`, `npm-projection`, `policy-file`,
`promotion`, `publishing`, `registry-store`, `seam-contracts`,
`seam-fakes`, `serving-process`, `store-ledger` and `tools-layer`.

## Impact

- Specs: the 25 above, rewritten by the archive.
- New: `check-changes.sh`, `check-changes-cases.sh`,
  `docs/adr/0037-…md`.
- Changed: `moon.yml` (`root:specs`, `root:changes`,
  `root:changes-cases`), `docs/architecture/core.md` (core.77).
- No code changes, and no test changes: the new scenarios instance what
  their requirements already said.
- Sources: boomdrivedev/blueprints `design/principles.md` 16 as amended
  on 2026-11-15; the ruling of boomdrive-design of 2026-11-16;
  boomdrivedev/boomdrive record 0050 and core.61; boomdrivedev/flume
  record 0073 and core.59; boomdrivedev/sluice record 0128 and core.61;
  OpenSpec 1.14.3, the release `devenv.nix` pins; rules core.23,
  core.25, core.26.

### Out of scope

- **The 12 placeholder Purposes.** OpenSpec 1.14.3 reads a delta's
  `## Purpose` only when it creates the capability, so no change can
  write them. They are written straight into their specs in a commit of
  their own, on the ruling (record 0037).
- **A test of each new scenario.** Each instances what its requirement
  already said, which the crates' tests and the conformance suite hold
  now; a scenario the suite does not yet run waits on the unit that
  builds or changes that behaviour, as any scenario does.
- **The other tools' checks.** Boomdrive's record 0050 owns the design;
  this change copies it.
