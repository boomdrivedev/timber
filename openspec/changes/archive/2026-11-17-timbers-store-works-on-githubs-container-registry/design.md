## Context

The registry store (`crates/timber/src/profiles/store/registry/`)
binds the store seam to any OCI registry the host names as
`[store] registry = "<host>/<path>"` (C.1, ADR 0003). It is proven in
the tools layer against zot and the reference distribution, and against
Amazon ECR by a run by hand outside the gate, whose answers ADR 0015
records. The user chose GitHub's registry under boomdrivedev for the
family's store on 2026-11-16 (card `req_02f6ed1d21734e7ba2da`). Nothing
has yet run the store's scenarios against it.

What is known of GitHub's registry before the run, from reads of a
public repository with no credential on 2026-11-17:

- A request it challenges is answered `401` with
  `Bearer realm="https://ghcr.io/token",service="ghcr.io",scope="repository:<owner>/<name>:pull"`.
- `GET /v2/<name>/referrers/<digest>` of a manifest it holds is answered
  `404` with `{"errors":[{"code":"MANIFEST_UNKNOWN",…}]}`: it has no
  referrers API.
- `GET /v2/<name>/tags/list?n=2` pages with a `Link: <…>; rel="next"`
  header.

What the profile already does with each:

- It answers a `Bearer` challenge by asking the realm, with the host's
  Docker credential as `Basic`, for the scope the challenge names, and
  holds the token in memory under the repository the request is for
  (client.rs, design D4 of the registry-store change).
- It reads a `404` on the referrers API's first page as no API, and
  keeps attestations in the referrers tag the distribution
  specification defines (ADR 0015's design D6). The reference
  distribution proves that path in the gate.
- It follows `Link` headers on the tag list and the referrers list.

The run against ECR asked its own questions. The ones GitHub's
registry may answer differently are the scope of a write's token, the
nested repositories C.1's three spaces need (`<declared>/artifact/…`,
`<declared>/cache/<build>`, `<declared>/<component>`), which GitHub's
registry makes as packages on push, the visibility it gives them, and
a cross-repository mount.

## Goals / Non-Goals

**Goals:**

- The store's conformance group and the cache manifest's scenarios pass
  against a repository of GitHub's registry, by hand, three runs in a
  row.
- A write is answered with a token that may push.
- A record says what GitHub's registry answered, and the stores page
  marks it built.
- An operator reading the self-hosting guide can set a store up on
  GitHub's registry.

**Non-Goals:**

- A profile of its own for GitHub's registry, or any code that names
  it. It is one more registry the one profile binds (core.1, core.2).
- Changing an OCI form the profile writes. If the run finds one
  refused, the fix is the run's (D6), and every registry's digests
  move with it.
- Binding deneb's store, choosing the family's declared repository, or
  publishing anything into it.
- Making packages public, or removing them. Both are the operator's,
  in GitHub's settings.

## Decisions

### D1. GitHub's registry is bound as every registry

A host binds it with
`[store] registry = "ghcr.io/<owner>/<repository>"`. The profile has no
branch for it, and nothing in `crates/timber/src/` names it. A
repository path is lowercased by the host as the distribution protocol
requires, and a path GitHub's registry refuses is refused in its words
(58, 59).

The alternative was a profile `ghcr` with GitHub's settings, such as a
package's visibility through GitHub's REST API. It was not taken: C.1
binds a registry through the distribution protocol alone, and
visibility is the operator's (D7).

### D2. A write asks to pull and push its repository

In `Client::authorize`, the scope asked of a `Bearer` realm depends on
the request:

- A write, any method but `GET` and `HEAD`, asks for
  `repository:<repository>:pull,push`. `<repository>` is the repository
  the caller names for the request, the one its token is held under.
- A read asks for the scope the challenge names, or none where the
  challenge names none, as today.

`service` is still the challenge's. The token answered is held under
the repository as today, so a write's token replaces a read's, and the
reads after it use it.

The distribution token specification has the client ask the scope it
needs. Docker and go-containerregistry ask `pull,push` for a push
themselves, and never read it from the challenge. GitHub's registry
names `pull` in the challenge it gives a read. If it names `pull` for a
write too, the profile asks a token that cannot push, the write is sent
once more with it, and it is refused. Asking `pull,push` for every
write costs nothing where the challenge already names it.

The alternative was adding `push` only where the challenge names the
request's repository with fewer actions. It was not taken: it reads
more of the challenge's form for the same answer. A mount asks only for
the target repository, as today. A registry that declines the mount for
want of `pull` on `from` opens an upload, and the profile uploads the
bytes under their digest (ADR 0023). The run reports how many mounts
were taken.

### D3. Attestations on GitHub's registry are kept in the referrers tag

Nothing changes. GitHub's registry answers the referrers API's first
page `404`, which the profile reads as no API. attach then lists the
referrer in the subject's `sha256-<hex>` tag, and claims-of reads it
(C.1). The referrer itself still carries its subject. The run proves
the path, and the record says what the referrer's push answered,
`OCI-Subject` or none.

### D4. One live rig for every hosted registry

`registry_ecr.rs` holds `Prefixed`, `EcrRig` and `run_prefix`. They
move to `crates/timber/tests/registry/mod.rs`, which every registry
test includes, as:

- `run_prefix()`, unchanged.
- `Prefixed { registry, prefix }`, the store whose every name carries
  the run's prefix in its space, unchanged.
- `LiveRig::new(repository, run, scenario)`, `EcrRig` renamed. Its
  `unreachable()` points at a loopback port nothing listens on, and its
  bound and conditional names are the registry's.
- `store_group_against(repository)`, which runs the store group over a
  fresh `LiveRig` per scenario, and asserts the report: the profile
  names the repository, every scenario ran, and only
  `a name moved from where it is not` is not applicable, since no
  registry the profile binds takes a conditional tag put (ADR 0015).

`registry/mod.rs` already allows dead code, so `registry_store.rs`
includes it unchanged. `registry_ecr.rs` keeps its three tests and its
setting, and calls the shared items.

### D5. The GHCR check

`crates/timber/tests/registry_ghcr.rs`, behind the `tools` feature,
reads `TIMBER_TOOLS_GHCR`. The setting is the repository the host names
in its `devenv.local.nix`, `ghcr.io/<owner>/<path>`. Where it is unset,
empty, or names a host other than `ghcr.io`, every test panics naming
`env.TIMBER_TOOLS_GHCR` and what it must hold. The panic also says the
host's Docker configuration must hold a credential for `ghcr.io`, a
token with `write:packages`, as `docker login ghcr.io` keeps it. Its
tests:

- `ghcr_keeps_the_stores_contract`: `store_group_against` over the
  repository.
- `ghcr_names_a_cache_entry_through_a_cache_manifest`: `cache_entries`
  over a `Prefixed` registry bound for `moon`, as ECR's.
- `ghcr_makes_an_artifacts_repository_on_push_and_keeps_it_private`:
  makes an artifact and an index under the run's prefix, then makes the
  artifact's name, so the push makes the name's repository. The name
  resolves to the index, and the mounts asked and taken are printed. A
  read of the index's manifest in that repository is then sent with no
  credential: the anonymous challenge is followed to its realm with no
  `authorization`, and the manifest is asked with whatever token that
  answers. It must not be answered `200` with the index. Any other
  answer passes and is printed. A `200` fails, naming the repository:
  GitHub's registry made it public on push, which would publish a
  private store's artifacts.

The task `tools-ghcr` in `crates/timber/moon.yml` runs
`cargo test -p timber --features tools --test registry_ghcr`. It is
tagged `check-ghcr` alone, with the `tools-ecr` task's inputs and
`$TIMBER_TOOLS_GHCR` in place of `$TIMBER_TOOLS_ECR`. It runs outside
the gate (core.38).

A run leaves its packages, all private, under the declared repository.
The operator removes them in the organization's package settings. The
alternative was the check deleting what it made through GitHub's REST
API. It was not taken: that needs `delete:packages` and a GitHub API
client in timber's tests for a check's tidiness, and ECR's runs leave
theirs too.

### D6. The proof is three runs in a row, by hand

The acceptance is `moon run ':#check-ghcr'` passing three times in a
row. A tag's read lag shows in some runs only, as ECR's did (ADR 0015).
The runs need a repository under boomdrivedev and a credential that
writes it. The person or the ops role names both: the repository in the
place's `devenv.local.nix`, the credential in the host's Docker
configuration. The build stage runs the check where both are there.
Where either is missing, it stops short to ops and ticks nothing.

Where a run fails on something GitHub's registry answers:

- The fix goes in the profile for every registry.
- A unit test over the profile's stub reproduces the answer verbatim.
- The registry store's tools test against zot and the distribution
  passes again.
- The three runs start over.

A fix to a form the profile writes, or to how name reads a tag, also
needs a run of the ECR check by hand (core.38). It moves every
registry's digests, a layout's among them.

### D7. What an operator sets up on GitHub's registry

The self-hosting guide gains a section on GitHub's registry:

- Every repository under the declared one is a package. GitHub's
  registry makes each on its first push, private.
- A package is made public one at a time, in its settings, by an
  administrator of the organization. GitHub documents that a public
  package is never made private again. Each new artifact name, and
  each new component, is a new package, private until that is done.
- The declared repository holds every blob, artifact, index and claim a
  host publishes. It is public only where everything the host publishes
  may be.
- The host's credential is a token with `write:packages` for a
  publishing host, kept by `docker login ghcr.io`. A host that only
  reads a private store needs `read:packages`.
- A package's untagged versions are never deleted. An artifact's
  manifest, a referrer and a cache entry's bytes are held by digest,
  untagged, and named by an index, a referrers tag or a cache manifest.
- What the GHCR check leaves is removed in the package settings.

### D8. Records

- Record 0039, proposed: D2, with its path not taken, and what GitHub's
  registry answered on each issue of the registry-store spec's record
  requirement, with the date and each run's times, in ADR 0015's form.
  D5's leaving of packages and D7's operator setup go in its
  Consequences.
- Rule core.38's details: the change to the forms or to how name reads a
  tag that needs a run of the ECR check by hand now needs one of the
  GHCR check too. GitHub's registry is the family's store, and its
  proof holds only while the forms it took are the ones written.
- The stores page marks GitHub's registry built, with the date of the
  third run.

No clause changes. C.1 already names GitHub's registry, and its bound
and its referrers fallback hold there unchanged.

## Risks / Trade-offs

- **GitHub's registry refuses a form.** A nested repository, an artifact
  type, a manifest with a subject or the empty config may be refused.
  → D6: the run fails in the registry's words, the fix proves itself
  over the stub and on every registry, and the record says what
  changed. A refusal of nested repositories would mean C.1's three
  spaces cannot be kept on GitHub's registry. That is the statement's
  to change: the build stage reports it with
  `flume needs --found --to design`.
- **A tag reads back late.** GitHub's registry serves through a cache.
  → The profile's bound of 30 seconds holds every registry (C.1). Three
  runs in a row show whether it holds on GitHub's. A run past the bound
  fails naming the former place, and the record says so.
- **A write's token is refused for want of a package role.** The
  account behind the token may lack write on the organization's
  packages. → The token request or the write is refused in GitHub's
  words. The run names the repository, and the operator grants the
  role.
- **The check leaves packages.** Each run makes some private packages
  under the declared repository. → They are visible only to the
  organization's members, and the operator removes them (D5).
- **A package made public by mistake cannot be made private again.** →
  The guide says so before it says how. The check proves that what the
  store makes on push is private.

## Migration Plan

Nothing published changes. A host bound to ECR, zot or a layout reads
and writes as before. A host bound to a registry that names `push` in
its challenge to a write asks the same scope it asked before. A
registry whose token server refuses `pull,push` to an account that may
only pull refuses a write it would have refused anyway, in its own
words. Rollback is a revert of `client.rs`.

## Open Questions

For the design, in the statement. None blocks this change.

- **One host, one declared repository, two visibilities.** The user's
  choice keeps the tools public and swancloud private on GitHub's
  registry. A host's bindings name one store (118), its declared
  repository holds every blob, artifact, index and claim the host
  publishes (C.1), and GitHub's registry makes a package public or
  private whole. A host that publishes both keeps both in one package.
  Either each is published from a host, or a configuration, of its own,
  or the statement lets a repository name its store.
- **A push credential that writes cache entries alone.** C.1 and
  core.62 scope a pool job's push credential to `cache/`. On GitHub's
  registry a credential carries its account's role on each package,
  and an account that may create packages in the organization makes a
  new one, a new artifact name's or component's, on its first push. A
  push credential scoped to `cache/` alone holds only where the
  organization lets that account create no package, which also stops
  it making a new build profile's cache repository. Nothing runs a pool
  job on GitHub's registry yet.
