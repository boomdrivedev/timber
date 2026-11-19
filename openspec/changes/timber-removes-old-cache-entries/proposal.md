## Why

A repository's cache entries pile up in the store. Every landing's run
pushes the entries of the tasks it ran, and nothing removes them, so a
store on GitHub's registry or ECR grows with every landing for as long
as the repository lives.

Clause 145, added to the statement at `boomdrivedev/blueprints` fecf4d9
on 2026-11-18, says what must hold. The store keeps a repository's cache
entries for as long as its owner's settings say, thirty days from their
push unless they say otherwise. timber removes each entry once that time
has passed, with a credential that may remove cache entries and nothing
else. It never removes for the cache anything accepted or released (54),
the record of versions claimed (22), or the ledger (34). A build whose
entry is gone builds again and is no worse for it (54).

## What Changes

- **The keeping time is the policy's.** The policy file gains
  `[cache] keep_days`, a whole number of days. The givens' fragment
  gives `keep_days = 30`, laid beneath the file as the given channels
  are. It is read through the policy seam at a landed revision, and
  `timber policy` shows it beside the rest of the policy in force.
- **A cache entry says when it was pushed.** The registry's cache
  manifest carries the name it was made for and the moment the host put
  it, so each entry has a manifest of its own. A layout stamps the index
  entry of a name made at bytes with the moment it was made.
- **The store lists and removes cache entries.** The store seam gains
  two operations: every cache entry the store holds for its repository
  and build profile, with when each was pushed; and one entry removed
  by its build identity, only where its name is at the digest listed.
  Neither can name anything outside the cache space.
- **Each store removes its own way.** A registry deletes the entry's
  cache manifest by its digest in the cache repository, through the
  distribution protocol, as ECR, zot and the reference distribution
  take it. On GitHub's registry it deletes the package version through
  GitHub's API, with the token the host's Docker login holds. A layout
  removes the name from its index and leaves the bytes.
- **`timber prune` removes what has passed its time.** It checks that
  the branch landed the revision, reads the keeping time from the policy
  there, lists the store's cache entries, and removes each one pushed
  longer ago than the keeping time. An entry with no push moment, pushed
  before this change, is removed. An entry it cannot read is answered
  and left. It is bound to the landing, the policy and the store's cache
  repository alone, and signs, keeps and starts nothing.
- **The suite reads 145.** The statement's list moves to fecf4d9. The
  store group gains the listing and removal of cache entries, run over
  every store profile, and the core's cache group gains prune over the
  fakes.
- **The proof on registries.** The registry store's unit tests over the
  stub, GitHub's API among them; prune over zot and the reference
  distribution in the tools layer; and the GHCR and ECR checks by hand,
  since the cache manifest's form changes (core.38).
- **The record, the rule and the pages.** Record 0045 holds this
  change's choices with the paths not taken. Rule core.80 says what
  prune may remove. The stores page and the self-hosting guide say how
  entries are kept and removed, and how to scope the credential that
  removes them on GitHub's registry and on ECR.

Out of scope, and what each waits on:

- Cache entries at a root the repository left. A pool's push may still
  write there (record 0042), and prune binds the checkout's root alone.
  Removing them waits on a later change that binds prune at each root
  the record names; none is planned, since a root left takes entries
  only from a stale checkout.
- A cache repository of a build profile the host does not bind. Every
  repository binds moon today. A second build profile's cache waits on
  the change that binds one.
- Running prune on a schedule. timber starts nothing (core.52, C.16).
  The guide gives the repository's own scheduled workflow, and a host
  runs it as its operator chooses.
- Bytes nothing names any more. A registry collects them as it does:
  GitHub's registry and ECR on their own, zot by its own collection, and
  the reference distribution by its operator's garbage collection. A
  layout keeps them (design D6).
- Untagged cache manifests a lost race leaves. Record 0039 says a
  package's untagged versions are never deleted, so prune deletes none.
- A dry run. Removing an entry early costs a rebuild and nothing more
  (54).
- C.1's table and C.10's list in the statement. Neither yet names a
  removal or the keeping time's line; this change rests on 145 alone,
  as the store's listing of versions rested on 138. Folding the lines
  into C.1 and C.10 is the design's, named in the tasks' follow-up.

## Capabilities

### New Capabilities

- `cache-keeping`: `timber prune` reads the keeping time from the policy
  at a landed revision and removes each cache entry pushed longer ago,
  answering each entry on its own and removing nothing but cache
  entries.

### Modified Capabilities

- `seam-contracts`: the store lists and removes cache entries, the
  policy answers the keeping time, and a cache entry's store name is
  spelled by the seams' one function.
- `seam-fakes`: the fake store lists and removes cache entries with
  their push moments, and the fake policy answers the keeping time a
  test sets.
- `policy-file`: `[cache] keep_days`, given thirty days.
- `registry-store`: the cache manifest names its entry and its push
  moment; a cache repository's entries are listed and removed; GitHub's
  registry removes through GitHub's API.
- `layout-store`: a name made at bytes is stamped; the layout lists its
  cache entries and removes a name alone.
- `host-bindings`: prune's binding.
- `command-line`: `timber prune`, and the keeping time in
  `timber policy`.
- `conformance-suite`: the store group's cache scenarios and prune over
  the fakes.
- `clause-coverage`: the suite reads the statement at fecf4d9, through
  145.

## Impact

- `crates/timber-seams`: `store.rs` (`Pushed`, `cache_entries`,
  `remove_cache_entry`), `policy.rs` (`Keeping`, `cache_kept`), the
  cache name's spelling, `answer.rs` (`utc_seconds`), and every
  implementer of `Store` and `Policy` in the workspace.
- `crates/timber-core`: `cache.rs` (the name re-exported), a new
  `prune.rs`, `in_force.rs`, and their tests.
- `crates/timber-fakes`: `store.rs`, `policy.rs`.
- `crates/timber`: `profiles/store/oci.rs`, `profiles/store/registry/`,
  `profiles/store/layout/`, `profiles/policy/file/` and `givens.toml`,
  `bindings/mod.rs`, a new `commands/prune.rs`, `commands/policy.rs`,
  the command list, and the tests under `crates/timber/tests/`.
- `crates/timber-conformance`: `statement.rs`, `seams/store.rs`,
  `core/cache.rs`, and the registry rigs.
- `docs/adr/0045-…`, rule core.80 in `docs/architecture/core.md`, and
  the stores page and the self-hosting guide under `site/`.
- The cache manifest's form changes. Entries pushed before this change
  read with no push moment and are removed at the first prune; nothing
  else in the store changes.

## Touches

```
crates/timber-seams/src/**
crates/timber-core/src/cache.rs
crates/timber-core/src/prune.rs
crates/timber-core/src/in_force.rs
crates/timber-core/src/lib.rs
crates/timber-core/src/copy.rs
crates/timber-core/src/repair.rs
crates/timber-core/tests/**
crates/timber-fakes/src/**
crates/timber-conformance/src/**
crates/timber-conformance/tests/**
crates/timber/src/profiles/store/**
crates/timber/src/profiles/policy/file/**
crates/timber/src/profiles/target/**
crates/timber/src/bindings/mod.rs
crates/timber/src/bindings/tests.rs
crates/timber/src/commands/**
crates/timber/src/main.rs
crates/timber/src/lib.rs
crates/timber/tests/**
docs/adr/*
docs/architecture/core.md
site/src/content/docs/guides/integrations/stores.mdx
site/src/content/docs/guides/self-hosting.mdx
openspec/changes/timber-removes-old-cache-entries/**
openspec/specs/cache-keeping/spec.md
openspec/specs/seam-contracts/spec.md
openspec/specs/seam-fakes/spec.md
openspec/specs/policy-file/spec.md
openspec/specs/registry-store/spec.md
openspec/specs/layout-store/spec.md
openspec/specs/host-bindings/spec.md
openspec/specs/command-line/spec.md
openspec/specs/conformance-suite/spec.md
openspec/specs/clause-coverage/spec.md
```
