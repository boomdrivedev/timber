## Why

The family's store is GitHub's registry under boomdrivedev, public for
the tools and private for swancloud (card `req_02f6ed1d21734e7ba2da`,
2026-11-16). On GitHub's registry each repository of a registry is a
package, and a package is public or private as a whole. Today the
registry store holds every blob, manifest, claim and undeclared
component's tag in the one repository a host binds, and every
repository's cache entries in one `cache/<build>` beneath it, so one
host's store cannot keep swancloud private beside the public tools.

Statement C.1, as amended at `boomdrivedev/blueprints` f412008, and
timber record 0040, as amended at 858b8fc6 (sluice #40 and #41), settle
it. The binding declares a registry and a root. A repository's
components and its cache entries hang under `<root>/<owner>/<name>/`,
the owner and name its remote gives, lowercased (114, 138). Artifact
pointers stay one space for the whole store, `<root>/artifact/`, so a
consumer still finds a pin by ecosystem, name and version alone (138,
core.60). Every blob, manifest and referrer a name reaches is held in
that name's repository, mounted or copied. On GitHub's registry the
pool's token is the job's own, given write on its repository's cache
packages and nothing else (135, 95, 96).

## What Changes

- **The binding's root.** `[store] registry = "<host>/<root>"` keeps
  its one key. Its path is read as the root every repository of the
  store is under (C.1). A host's file and timber's module stay as
  written.
- **The store is bound for the checkout's repository.** Every command
  binds a registry store for the repository the checkout is of. That
  is the owner and name its landing's remote's address ends in, the
  same answer the serving process uses (114, 118). A checkout with no
  such remote binds all the same. Its store answers that absence for
  every write and name of the repository's own spaces, and still reads
  every artifact's name.
- **Where each name is held.** A repository's own repository is
  `<root>/<owner>/<name>`. Under it sit each declared component's
  repository, holding that component's versions, channels and indexes,
  and each build profile's cache repository, `cache/<build>`. The own
  repository itself holds the undeclared component's versions and
  channels, and what a publish writes before any name reaches it.
  `artifact/<ecosystem>/<name>@<version>` is a tag of
  `<root>/artifact/<ecosystem>/<name>`, under no repository's own. An
  owner written `artifact` is refused, since that path is the artifact
  space.
- **A digest is read where its name was resolved.** The store
  remembers, for its life only, the repository each name it resolved
  was in, and the digests a manifest read there names. read, claims-of
  and name ask those repositories first, then the repository's own,
  then the cache repository. A consumer bound for one repository then
  reads another's pin, its index, its artifacts and their claims from
  the artifact's repository.
- **A claim is held in every repository that holds its subject.**
  Holding a manifest in a name's repository holds timber's attestations
  on it and on each manifest it names there too. attach writes an
  attestation into each repository that holds its subject: the
  repository's own, each repository it was reached in, and for an
  index, its component's and its artifacts' names' repositories. A
  consumer reads acceptance, promotions and reports beside the index
  in the artifact's repository.
- **A name's versions leave out referrers tags.** On a registry with no
  referrers API, an artifact's repository now holds its index's
  referrers tag, which the listing of its versions skips.
- **The proof.** The registry store's unit tests over the stub, the
  tools layer's tests against zot and the reference distribution, and
  the GHCR check by hand once against a root under boomdrivedev.
- **The record, the rule and the pages.** Record 0041 holds this
  change's choices with the paths not taken. Rule core.62's details say
  how the pool's token reads on GitHub's registry (record 0040). The
  module's option, the stores page and the self-hosting guide say where
  a repository's packages are, what ECR's creation templates and a
  pool's credential cover, and what an operator sets up on GitHub's
  registry.

Out of scope, and what each waits on:

- Carrying a store written before this change. Its undeclared
  component's tags, `<root>/<component>` and `<root>/cache/<build>`,
  and its claims held in the root alone are read by nothing after it.
  Its artifact names stay where they are. No host binds a registry
  store whose versions anything follows yet. The family's store is
  bound on GitHub's registry after this bolt, and the checks' runs are
  throwaway. The guide says so. A carry waits on a store that needs one.
- Making packages, setting their visibility and giving a job's
  repository access on GitHub's registry. These are the operator's, in
  GitHub's settings, as record 0040 and the guide say. timber names no
  forge (core.1).
- Running the ECR check by hand. The forms the profile writes and how
  name reads a tag are unchanged (core.38). Its next run checks the new
  places.
- Binding deneb's store and publishing the family's builds there: the
  bolt's later units.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `registry-store`: the binding declares a registry and a root. A
  repository's components and cache entries hang under its own
  `<root>/<owner>/<name>`, and artifacts' names under the root alone. A
  digest is read in the repository its name was resolved in. A claim is
  held beside its subject in each repository holding it. A name's
  versions leave out referrers tags. The GHCR check proves the places.
- `host-bindings`: `[store] registry` names a host and a root, and
  every command binds a registry store for the repository the
  checkout's landing remote names, the given remote for a push before
  the landing.

## Impact

- `crates/timber/src/profiles/store/registry/mod.rs` and `tag.rs`:
  where each name is held, the remembered repositories, claims copied
  and attached into each holder, and the listing; their unit tests.
- `crates/timber/src/profiles/store/mod.rs` and
  `crates/timber/src/bindings/mod.rs`: the store bound for the
  checkout's repository; their tests in
  `crates/timber/src/bindings/tests.rs`.
- `crates/timber/tests/registry/mod.rs`, `registry_store.rs`,
  `registry_ecr.rs` and `registry_ghcr.rs`.
- `docs/adr/0041-…`, core.62's details in `docs/architecture/core.md`,
  `nix/modules/options.nix`'s `registry` option, and the stores page
  and self-hosting guide under `site/`.
- No seam trait, shared type or OCI form changes. No digest changes.
  The layout store is untouched. A store from before keeps its bytes,
  and its old names are read by nothing.

## Touches

```
crates/timber/src/profiles/store/mod.rs
crates/timber/src/profiles/store/registry/**
crates/timber/src/bindings/mod.rs
crates/timber/src/bindings/tests.rs
crates/timber/tests/registry/mod.rs
crates/timber/tests/registry_store.rs
crates/timber/tests/registry_ecr.rs
crates/timber/tests/registry_ghcr.rs
docs/adr/*
docs/architecture/core.md
nix/modules/options.nix
site/src/content/docs/guides/integrations/stores.mdx
site/src/content/docs/guides/self-hosting.mdx
openspec/changes/each-repositorys-spaces-sit-under-its-own-root/**
openspec/specs/registry-store/spec.md
openspec/specs/host-bindings/spec.md
```
