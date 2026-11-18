## Why

A repository's components and cache entries hang under its own root in
the store, `<root>/<owner>/<name>/`, the owner and name its remote
gives (C.1, record 0040). A repository renamed or moved to another
owner gets a new remote address, so every command in it binds a new
root. Today that root holds nothing: each channel's chain starts again
there, no version published before the move can be promoted, and a
target still reading a channel at the old root keeps reading it as
current, with no word that it moved.

Statement C.1, as amended at `boomdrivedev/blueprints` e8700c8, and
record 0040, as amended on 2026-11-18 (b0956f3a), settle it. The
repository's record of claimed versions names the root it publishes
under (22, 62), so a move is read from git alone. The first publish
after a move carries the repository's history to its new root before it
publishes. Each channel's chain goes on from its head (105), every
version it published stays promotable there, and a read of a channel at
the old root is answered with where it moved, never as current. No
person runs anything for it, cache entries are not carried, and no
other repository's history writes under a root one left.

## What Changes

- **The record names each root.** `refs/timber/root` in the shared
  history, beside the version tags, is a chain of commits, each naming
  one root, `<owner>/<name>` lowercased, in the order the repository
  published under them. It is pushed without force and its copy only
  moves forward, as a ledger ref's is (core.47).
- **The binding answers the store's root.** Each binding the core is
  handed carries the roots: the repository the store is bound for, none
  for a store that places nothing by repository, as a layout does; and
  the store and ledger bound for a root the repository left, reading
  past the mark that says where it went.
- **A publish reads a move and carries.** After the landing check and
  the policy file's declaration, before any component is built, claimed
  or offered, a publish reads the record's last root and the bound one.
  An empty record records the bound root. A record whose last root is
  another is a move. The publish marks the old root with where it went,
  copies there every version the record claims with every claim on it,
  makes each channel's name at its chain's head, checks each chain goes
  on from that head, and records the new root last. Its answer says
  what it recorded or carried.
- **A root left answers where it went.** The mark is one name of the
  repository's own, `moved`, made by the seam's one function, at bytes
  naming the new root. A registry store bound for a root whose mark
  names another refuses every read and write of the repository's own,
  its components' and its cache repositories with a new refusal,
  `RootMoved`, naming both roots. It reads the artifact space as before.
  Only a carry's binding reads past the mark naming where it carries.
- **A root left is never written again.** A publish whose bound root
  holds a mark is refused naming where that root went, before anything
  is written: another repository at an address one left, a checkout
  whose remote still names the old address, and a repository moved back
  to a root it left alike.
- **The fakes and the suite.** A fake roots binding per world, and a
  conformance scenario where a moved repository's first publish carries
  its history and a version published before the move is promoted at
  the new root.
- **The proof on registries.** The registry store's unit tests over the
  stub, a carry over two registry stores on zot and the reference
  distribution in the tools layer, and the GHCR check by hand once with
  a carry under boomdrivedev.
- **The record, the rule and the pages.** Record 0042 holds this
  change's choices with the paths not taken. Rule core.78 says what a
  carry and a root left must keep. The stores page and the self-hosting
  guide say what a move does and what an operator sets up on GitHub's
  registry after one.

Out of scope, and what each waits on:

- A repository moved back to a root it left. timber cannot tell it from
  a checkout whose remote still names the old address, from git alone,
  so both are refused naming where the root went. A later change that
  tells them apart waits on a statement that says how.
- Carrying cache entries. The statement says they are not carried; the
  pool's next run fills the new root's cache, and a publish whose
  verdicts name entries only the old root holds is refused naming the
  output, as any publish whose entry is gone (54, 135, core.30).
- A store written before each repository kept its own root, and a
  repository that moved before its record named a root. Neither is
  carried: the record's first root is the one its first publish after
  this change binds (record 0041's D8).
- Retired channels. A chain the policy at the landed revision no longer
  declares is not carried; it stays readable by digest at the old root.
- A repository published to two stores. The record is the repository's,
  and a move is carried into the store of the host that publishes after
  it. A repository's publishers share one store.
- Making packages, setting their visibility and giving a job's
  repository access on GitHub's registry after a move. These are the
  operator's, in GitHub's settings, as the guide says (core.1).
- Running the ECR check by hand. The forms the profile writes and how
  name reads a tag are unchanged (core.38).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `publishing`: a publish reads the record's root against the bound
  one, records a first root, and carries a moved repository's history
  before it builds, claims or offers anything; a publish whose root was
  left is refused naming where it went.
- `registry-store`: a root left is marked with where it went, every
  operation reaching the repository's own spaces there is refused
  naming it, and a carry's binding reads past the mark naming where it
  carries.
- `host-bindings`: every binding answers the root its store is bound
  for, and binds the store and the ledger for a root left.
- `seam-contracts`: the roots a binding answers, the root left's
  refusal, and the mark's name and bytes.
- `seam-fakes`: a fake roots binding with a store and a keeper per
  root.

## Impact

- `crates/timber-seams`: `bind.rs` (`Roots`, `Left`, `NoRoots`,
  `Seams::roots`), `answer.rs` (`Why::RootMoved`), `keeper.rs` or a new
  `root.rs` (`moved_name`, `MovedTo`).
- `crates/timber-core`: `history.rs` (the root record), a new
  `carry.rs`, `publish.rs` (the carry before the components, the
  answer's `rooted`), and their tests in `crates/timber-core/tests/`.
- `crates/timber-fakes`: `FakeRoots`, and `seams()` handing it.
- `crates/timber`: `profiles/store/registry/mod.rs` (the mark),
  `profiles/store/mod.rs` and `profiles/ledger/mod.rs` (a root left's
  store and ledger), `bindings/mod.rs` (the roots), the publish
  command's text, and the tests under `crates/timber/tests/`.
- `crates/timber-conformance`: the world's roots and a publish
  scenario.
- Every place that builds a `Seams`.
- `docs/adr/0042-…`, rule core.78 in `docs/architecture/core.md`, and
  the stores page and self-hosting guide under `site/`.
- No digest, OCI form or layout store changes. A store keeps every byte
  it held; a root left gains one tag.

## Touches

```
crates/timber-seams/src/**
crates/timber-core/src/history.rs
crates/timber-core/src/carry.rs
crates/timber-core/src/lib.rs
crates/timber-core/src/publish.rs
crates/timber-core/tests/**
crates/timber-fakes/src/**
crates/timber-conformance/src/**
crates/timber-conformance/tests/fakes.rs
crates/timber/src/profiles/store/mod.rs
crates/timber/src/profiles/store/registry/**
crates/timber/src/profiles/ledger/mod.rs
crates/timber/src/bindings/mod.rs
crates/timber/src/bindings/tests.rs
crates/timber/src/commands/publish.rs
crates/timber/tests/**
docs/adr/*
docs/architecture/core.md
site/src/content/docs/guides/integrations/stores.mdx
site/src/content/docs/guides/self-hosting.mdx
openspec/changes/a-moved-repository-keeps-its-history-in-the-store/**
openspec/specs/publishing/spec.md
openspec/specs/registry-store/spec.md
openspec/specs/host-bindings/spec.md
openspec/specs/seam-contracts/spec.md
openspec/specs/seam-fakes/spec.md
```
