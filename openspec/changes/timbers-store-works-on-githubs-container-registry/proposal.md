## Why

The family's store is GitHub's container registry, under the
boomdrivedev organization: the user's choice on their card
`req_02f6ed1d21734e7ba2da` of 2026-11-16, public for the tools and
private packages for swancloud. Clause C.1 names GitHub's registry
among the registries the store binds, and timber's registry store
speaks the protocol it speaks, but nothing has run the store's
scenarios against it: the stores page marks it designed, and the
design book says the store is "designed for GHCR and not proven".
Amazon ECR, zot and the reference distribution are each proven, ECR by
a run by hand outside the gate (ADR 0015) and the other two in the
tools layer. This change proves the store on GitHub's registry the
same way, and fixes what that registry answers that the profile does
not yet take.

## What Changes

- **A write asks to pull and push its repository.** GitHub's registry
  challenges a read with a `Bearer` challenge whose scope names `pull`
  alone, as read on 2026-11-17. The profile asks the realm for the
  scope the challenge names, so where the registry names `pull` alone
  for a write too, the token would pull and never push. A write, a put,
  an upload or a mount, now asks for `repository:<repository>:pull,push`,
  the scope Docker and go-containerregistry ask for a push. A read asks
  the challenge's scope as before (64, C.1).
- **The check against GitHub's registry.** A new test,
  `crates/timber/tests/registry_ghcr.rs`, runs the suite's store group
  and the cache manifest's scenarios against the repository the host
  names in `env.TIMBER_TOOLS_GHCR` of its `devenv.local.nix`. It
  reaches the credential through the host's Docker configuration, and
  each run's names are apart from every earlier run's. A third test
  makes an artifact's name in a repository the push makes, reports the
  mounts the registry took, and checks that a read of that repository
  with no credential is never answered the index. The task
  `cli:tools-ghcr` runs it, tagged `check-ghcr` and outside the gate,
  since it needs a credential a person signs in for (93, core.38). It
  refuses to run, naming the setting, on a host that names no
  repository of `ghcr.io`.
- **One live rig for every hosted registry.** The run prefix, the
  prefixed store and the store group's rig over a hosted registry move
  from `registry_ecr.rs` into `crates/timber/tests/registry/mod.rs`.
  The ECR check and the GHCR check share them.
- **The proof.** The GHCR check runs by hand three times in a row
  against a repository under boomdrivedev, the change's acceptance.
  What GitHub's registry refuses is fixed in the profile for every
  registry and proven over the stub with its answer verbatim.
- **The record.** Record 0039 holds the write's scope and what GitHub's
  registry answered on each issue the profile rests on, beside ADR
  0015's for ECR. Rule core.38's details name the GHCR check beside
  ECR's for a change to the forms the profile writes or to how name
  reads a tag.
- **The site.** The stores page marks GitHub's registry built. The
  self-hosting guide says how a store sits on GitHub's registry:
  packages made on push and private, made public one at a time and
  never private again, the host's token, and untagged versions left
  alone.

Out of scope, and what each waits on:

- Binding a host's store to GitHub's registry, deneb's among them:
  the host's bindings file, written by timber's module or by hand, once
  this change lands. A publish of the family's builds there waits on
  the bolt's later units.
- Which declared repository the tools and swancloud publish into, and
  how one host's one store keeps the tools public and swancloud
  private. The declared repository holds every blob and claim a host
  publishes, and GitHub's registry makes a package public or private
  whole. The design answers this in the statement (design, Open
  Questions).
- A push credential that writes cache entries alone on GitHub's
  registry (135, core.62): the design, beside the question above.
- Removing what the check's runs leave: the operator's, in the
  organization's package settings. The store removes nothing.
- Harbor: it stays designed until something runs the suite against it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `registry-store`: a write's token is asked to pull and push its
  repository; the store's scenarios pass against GitHub's registry in a
  check by hand outside the gate; a record says what that registry
  answered.
- `tools-layer`: the GHCR check is one of the checks that need a
  person's sign-in and run outside the gate, against a repository and
  a credential the host names.

## Impact

- `crates/timber/src/profiles/store/registry/client.rs`: the scope a
  write asks for, and its unit test.
- `crates/timber/tests/registry/mod.rs`, `registry_ecr.rs` and a new
  `registry_ghcr.rs`.
- `crates/timber/moon.yml`: the `tools-ghcr` task.
- `docs/adr/0039-…`, and core.38's details in `docs/architecture/core.md`.
- `site/src/content/docs/guides/integrations/stores.mdx` and
  `site/src/content/docs/guides/self-hosting.mdx`.
- No seam trait, shared type, OCI form or binding changes. No layout's
  or registry's digests change.

## Touches

```
crates/timber/moon.yml
crates/timber/src/profiles/store/registry/**
crates/timber/tests/registry/mod.rs
crates/timber/tests/registry_ecr.rs
crates/timber/tests/registry_ghcr.rs
crates/timber/tests/registry_store.rs
docs/adr/*
docs/architecture/core.md
site/src/content/docs/guides/integrations/stores.mdx
site/src/content/docs/guides/self-hosting.mdx
openspec/changes/timbers-store-works-on-githubs-container-registry/**
openspec/specs/registry-store/spec.md
openspec/specs/tools-layer/spec.md
```
