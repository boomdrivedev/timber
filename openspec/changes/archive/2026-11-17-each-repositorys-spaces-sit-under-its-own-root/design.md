## Context

The registry store (`crates/timber/src/profiles/store/registry/`)
binds the store seam to an OCI registry the host names as
`[store] registry = "<host>/<path>"`. Today that path is one declared
repository. It holds every blob, manifest and referrer, and the tags of
a repository that declares no component. Three spaces of repositories
hang under it: `<path>/<component>`, `<path>/artifact/<ecosystem>/<name>`
and `<path>/cache/<build>`. A name's repository gets the manifests and
blobs its tag reaches, mounted from the declared repository. Its claims
stay in the declared repository, where claims-of reads them (ADR 0023;
`hold`, `place`, `named_at`, `claims_of` in `registry/mod.rs`).

Statement C.1, as amended on `boomdrivedev/blueprints`
`design/timber-a-root-per-repository` at f412008, and timber record
0040, as amended at 858b8fc6 (on main through sluice #40 and #41, not
yet on this bolt), change that:

- The binding declares a registry and a root. Every name of the store
  is a tag of a repository under the root.
- A repository's components and cache entries hang under
  `<root>/<owner>/<name>/`, the owner and name its remote gives,
  lowercased (114, 138).
- An artifact's pointer is a tag of `<root>/artifact/<ecosystem>/<name>`,
  one space for the whole store and under no repository's own. A
  consumer still finds a pin by ecosystem, name and version alone, and
  reads the version's claims beside its index there (138, core.60).
- Every blob, manifest and referrer a name reaches is held in that
  name's repository. It is mounted from another repository under the
  root where the registry takes a mount, and copied where it does not.
- ECR's creation templates cover each repository's root and
  `artifact/`. On GitHub's registry the pool's token is the job's own,
  given write on its repository's cache packages alone. The publishing
  host makes the component and artifact packages first (135, 95, 96).

The store seam addresses by digest alone (B.1), and a registry resolves
a digest only within a repository. With one declared repository, every
digest had one home. With a root per repository there is none. The
design below says where a digest is found and where a claim is
written, so the seam's answers stay the same.

What a publish asks of the store, in order (`publish.rs`, `offer`):

1. put-bytes and put-artifact for each artifact, then attach its
   provenance;
2. put the notes and attach their claim;
3. put the index, held at once in a declared component's repository
   (ADR 0023);
4. attach the layout policy to the index;
5. name each artifact's pointer, then the version.

After a publish, the gate, the ledger and apply attach acceptance,
promotions and reports to the index and move channels (`gate.rs`,
`ledger.rs`, `apply.rs`). A consumer resolves an artifact's pointer and
reads the index, the artifact, its bytes and the claims on the index
and the artifact (`render.rs`).

## Goals / Non-Goals

**Goals:**

- One host's store holds the public tools and private swancloud side
  by side. Each package is public or private as its repository is.
- A consumer bound for any repository, or for none, resolves a pin and
  reads its claims from the artifact's repository alone.
- The gate, the ledger, apply and status read and write a repository's
  versions, channels, claims and cache entries under its own root.
- The seam, the core and the layout store are unchanged.

**Non-Goals:**

- Carrying a store written before this change (D8).
- Code that names GitHub, or makes packages, sets their visibility or
  gives a job access (core.1). These are the operator's (D9).
- A second store, or a store named by a repository's policy. Record
  0040 weighed both and took neither.

## Decisions

### D1. The binding's one value names the registry and the root

`[store] registry = "<host>/<root>"` keeps its key and its form.
`Registry::new` reads the path as the root, a non-empty path as
before. The record keeps the binding as written,
`registry <host>/<root>`, and every answer names the store's source,
`OCI registry <host>/<root>`, as today.

The path not taken was two keys, `registry` and `root`. One value
already declares both. Two keys would change every host's file and the
option timber's module names as the file names it (core.76), for no
difference in what is bound.

### D2. The store is bound for the checkout's repository, as the serving process names it

`bindings::bind` binds a registry store for
`repository_named(checkout, remote)`. `remote` is what
`landing_remote` answers: the remote `[landing]` names, or the given
landing remote where the file names no `[landing]`. That holds for
`push --left` too, whose landing is otherwise unbound. The serving
process names a repository by the same two functions (114), so the
store and the served path have one owner. `profiles::store::bind` and
`bind_for` take that answer, and a target's store is bound the same
way. The layout ignores it.

The answer is kept, not required, at binding. A registry store bound
for no repository answers every write, and every name of the
repository's own spaces, with the binding's answer: absent naming the
landing's remote, or unreadable naming its address. It writes nothing.
Its reads of the artifact space and of what those names reach work as
for any repository. So a checkout with no remote, a local clone among
them, still syncs, and a publish from it is refused before anything is
written.

The paths not taken:

- Refusing to bind a registry store with no repository. That would
  stop a sync that needs none (49, 138).
- Placing a version's names under the repository its index names.
  resolve has no index to read, and a publish's index and its checkout
  name one repository.

### D3. Where each name is held

The repository's own is `<root>/<owner>/<name>`. The owner and the
name are each written by `tag::component`, lowercased and escaped into
one path component (ADR 0023's D12). `place` maps each name:

| name | repository | tag |
|---|---|---|
| `artifact/<ecosystem>/<name>@<version>` | `<root>/artifact/<ecosystem>/<name>` | the version, escaped |
| `cache/<build identity>` | `<own>/cache/<build>`, `<build>` the bound build profile | the build identity |
| `<component>:<tag>`, `channel/<component>/<name>` | `<own>/<component>` | the tag, the channel's name |
| every other name: an undeclared component's version and `channel/<name>` among them | `<own>` | the name, escaped |

The repository's own also takes what put-bytes, put-artifact and
attach write before any name reaches it, as the declared repository
did. A store bound for a push puts bytes into its cache repository
alone, as before (135).

An owner whose path component is `artifact` is refused, naming it and
the artifact space, before anything is written. Owner `artifact` with
name `npm` would make `<root>/artifact/npm/<component>` the repository
of the npm name `<component>`. The other path was a prefix on every
repository's own, such as `<root>/repo/<owner>/<name>`. It lengthens
every path to rule out one owner, and C.1 spells the root as
`<root>/<owner>/<name>/`.

### D4. A digest is read where its name was resolved

The store keeps, in memory for its life, the repositories each digest
was reached in:

- each name it resolved or made, against the digest the name names;
- each digest a manifest it read there names, against the same
  repository.

read, claims-of, attach and name find a digest in those repositories
first, in the order reached, then in the repository's own, then in the
bound build profile's cache repository. A blob's bytes are mounted
from the repository it was found in. A consumer bound for `acme/app`
resolves `acme/site`'s `artifact/eco/site@1.3.0`. It then reads the
index, the artifact, its bytes and their claims from
`<root>/artifact/eco/site`, and asks `<root>/acme/app` nothing.

What the store keeps is no record. It is gone when the process ends,
and the next process rebuilds it from the names it resolves (core.20).

The paths not taken:

- Reading the publisher from the index's `repository` annotation and
  asking its own repository. C.1 holds the claims beside the index in
  the artifact's repository, and a private publisher's own repository
  may hold builds a consumer of one name is given nothing of.
- Asking every repository under the root. GitHub's registry serves no
  catalog, and 138 forbids a consumer needing the publisher.

### D5. A claim is held in every repository that holds its subject

Two writes keep the claims beside a manifest wherever it is held.

**hold** brings a manifest into a name's repository: the index, the
manifests it names, and their blobs. It now brings timber's referrers
of each manifest too. It lists them where the manifest was found,
through the referrers API, or the referrers tag where the registry has
none. It mounts each referrer's config and layer, puts the referrer by
its digest, and adds it to the subject's referrers tag where the put
answers no `OCI-Subject`. Another tool's referrers stay where they
are.

**attach** writes the attestation into each of these that holds the
subject:

- the repository's own;
- each repository the subject was reached in (D4);
- for an index, its component's repository, the repository's own for
  the undeclared component, and each of its artifacts' names'
  repositories, read from the index's own annotations.

It writes only where an attestation is missing. It answers held
already only where every holder held it, so a second attach after one
failed partway writes what the first left out.

In publish's order, the provenance and the notes' claim are written to
the repository's own. The component's repository takes them with the
index when the index is put. The layout policy goes to the own and
component repositories. Each artifact's name then holds the index, the
artifacts and every claim made so far. The gate's acceptance, each
promotion and each report are attached later, into all of them. The
ledger walks a chain from the channel's tag in the component's
repository and finds each claim there. A predecessor's index no name
reached is read from the repository's own, which holds every claim.

The paths not taken:

- Claims in the repository's own alone. C.1 holds every referrer a
  name reaches in that name's repository, and a consumer reads only
  the artifact's.
- Copying claims when they are read. A reader holds no write, and a
  read writes nothing (60, core.32).

### D6. A name's versions leave out referrers tags

On a registry with no referrers API, the reference distribution and
GitHub's registry among them, an artifact's repository now holds its
index's referrers tag, `sha256-<hex>`. `tag::read` would read that
back as a version. The listing leaves out every tag `tag::is_referrers`
names. `tag::of` already refuses a name in that shape, so no version's
tag is lost. Any other tag that does not read back stays unreadable
(core.19).

### D7. The proof

- **The stub.** Each placement, the remembered repositories, the
  claims carried and attached, and the listing, in
  `registry/mod.rs`'s unit tests. The stub has no referrers API, so
  these cover the referrers tag.
- **The tools layer.** zot, which has a referrers API, and the
  reference distribution, which has none. Each runs the placement
  tests, and a test where a store bound for another repository reads
  a pin's claims (`registry_store.rs`). The gate runs both
  (`check-tools`).
- **GitHub's registry, by hand, once.** The GHCR check runs the store
  group and a new test of the places. The test makes a component's
  version, a cache entry and an artifact's name for the rig's
  repository. It checks that each is held where D3 says, and that a
  store bound for another repository reads the index's claims from the
  name's package. The live rigs bind the repository
  `timber-checks/store` under the root the host names, and keep each
  run's names apart by the run's prefix, as before. One passing run is
  the acceptance on the family's registry. A refusal of a nested
  package path is the statement's (`flume needs --found --to design`).
- **ECR is not run.** The forms the profile writes and how name reads
  a tag are unchanged, so core.38 asks for no run. Its next run checks
  the new places.

### D8. A store from before is not carried

A store written before this change keeps everything it holds. After
it, these are read by nothing:

- its undeclared component's tags in the root;
- its declared components' repositories, `<root>/<component>`;
- its cache, `<root>/cache/<build>`;
- the claims held in the root alone.

Its artifact names are where they were. The guide says so, and says a
host binds a new root, or publishes and promotes again under the old
one, before it follows anything there.

The path not taken was a carry: a command, or reads falling back to
the old places. No host binds a registry store whose versions anything
follows yet. The family's store on GitHub's registry is bound after
this bolt, and the ECR and GHCR checks' runs are throwaway. A carry
waits on a store that needs one. This is cheap to change: the old bytes
stay.

### D9. What an operator sets up

The self-hosting guide carries record 0040's operator setup.

**On ECR**, creation templates apply on push to `<root>/artifact/` and
to each repository's `<root>/<owner>/<name>/`, or one template covers
`<root>/`. A pool's credential writes `<root>/<owner>/<name>/cache/`
alone. A publishing host writes the repository's own, its components'
repositories and `<root>/artifact/`.

**On GitHub's registry:**

- Every repository is a package of the owner's.
- The publishing host's first publish makes the repository's own, its
  components' and its artifacts' names' packages, before any job
  pushes there.
- The operator sets each package public or private as its repository
  is. A public package is never private again.
- The operator gives the job's repository write on
  `<root>/<owner>/<name>/cache/<build>` alone, and nothing on any other
  package.
- A package a job makes on its own holds no name the gate accepted, and
  is refused wherever it is read (95, 96).

### D10. The record and the rule

Record 0041, status proposed, holds D1 to D6 and D8, each with the path
not taken, as this change's construct stage chose them. They are its
own, on C.1 as amended and record 0040, and cheap to change.

Rule core.62's details gain a Not ruled out: on GitHub's registry,
where no token is scoped to some of an owner's packages, the job's own
token given write on its repository's cache packages and nothing else.
That token may still make a package of its own, which holds no name
the gate accepted (C.1, 95, 96, 135; record 0040). Record 0040 changed
135's reading in the statement, and the rule says how a reviewer reads
core.62 there.

## Risks / Trade-offs

- **Each claim is written into several repositories.** An attach after
  a publish writes into two repositories plus one per artifact. On a
  registry with no referrers API, that is a referrers tag rewritten in
  each. Each write is one manifest, and a version names a handful of
  artifacts.
- **An attach can fail partway.** An attestation written into some
  holders and not others is answered with the failure, unchanged.
  The next attach writes what is missing (D5). Until then a reader of
  the name that lacks it reads less. A gate's move waits on its own
  attach's answer (core.45).
- **The store remembers within one process.** A digest no name in this
  process reached is read from the repository's own. Every claim the
  repository's own holds is there, so a ledger walk or a report read
  finds them. A digest of another repository's that no name reached is
  absent, which is what 138 asks: a consumer reaches another's
  versions through their names.
- **GitHub's registry and nested packages.** Record 0039 saw nested
  packages made on push under a declared path. A run that is refused a
  four-deep path stops short to the design (D7).

## Migration Plan

None in the store (D8). A host's file stays as written. After
upgrading, a host bound to a registry writes under its repository's own
root. The guide says what a store from before still holds.

## Open Questions

None.
