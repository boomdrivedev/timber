---
number: 41
title: A registry store is bound for the checkout's repository, reads a digest where its name was resolved, and holds each claim beside its subject
status: proposed
date: 2026-11-17
decision-makers:
- the construct stage of the change each-repositorys-spaces-sit-under-its-own-root, by its design D1 to D6 and D8
- the code stage of the same change, by two answers its design left open, under "What the code stage chose"
---

# A registry store is bound for the checkout's repository, reads a digest where its name was resolved, and holds each claim beside its subject

Status: proposed. Every choice here is the construct stage's own, made on statement C.1 as amended at `boomdrivedev/blueprints` f412008 and on record 0040, and each is cheap to change: no form the profile writes and no digest changes, and a store from before keeps its bytes.

Deciders: the construct stage of the change `each-repositorys-spaces-sit-under-its-own-root`, carrying out record 0040, boomdrive-design's, which carries out the user's choice of GitHub's registry under boomdrivedev as the family's store, public for the tools and private for swancloud (card `req_02f6ed1d21734e7ba2da`, 2026-11-16).

Sources: statement clauses 8, 49, 58, 72, 114, 118, 135, 138 and C.1 as amended at `boomdrivedev/blueprints` f412008; record 0040 as amended at 858b8fc6; ADRs 0003, 0023; rules core.19, core.20, core.27, core.32, core.60, core.76; the change's design D1 to D6 and D8; the OCI distribution specification (main), its sections on cross-repository mounts and the referrers API.

## Context and problem statement

C.1 as amended has the binding declare a registry and a root. A repository's components and its cache entries hang under `<root>/<owner>/<name>/`, its remote's owner and name lowercased, and artifacts' names stay one space for the whole store, `<root>/artifact/`. Every blob, manifest and referrer a name reaches is held in that name's repository.

The registry store held everything in one declared repository, so every digest had one home, and claims-of read every claim there. The store seam addresses by digest alone (B.1), and a registry resolves a digest only within a repository. With a root per repository there is no one home. Six questions follow: how the root is bound, which repository the store is bound for, where each name is held, where a digest is read, where a claim is written, and what a name's versions are once its repository holds a referrers tag. A seventh is what becomes of a store written before.

## Considered options

For the binding (design D1):

1. **`[store] registry = "<host>/<root>"` keeps its one key, its path read as the root (chosen).**
2. Two keys, `registry` and `root`. One value already declares both; two would change every host's file and the option timber's module names as the file names it (core.76), for no difference in what is bound.

For the repository the store is bound for (design D2):

1. **The checkout's, as `repository_named` answers it for the landing's remote, the given one's for `push --left`, the answer kept and not required (chosen).** The serving process names a repository by the same two functions (114), so the store and the served path have one owner. A store bound for no repository answers every write, and every name of the repository's own spaces, with the binding's answer, absent naming the landing's remote or unreadable naming its address, and reads the artifact space as any store does. A checkout with no remote still syncs, and a publish from it is refused before anything is written.
2. Refusing to bind a registry store with no repository. That stops a sync that needs none (49, 138).
3. Placing a version's names under the repository its index names. resolve has no index to read, and a publish's index and its checkout name one repository.

For where each name is held (design D3):

1. **`artifact/<ecosystem>/<name>@<version>` in `<root>/artifact/<ecosystem>/<name>`; `cache/<build identity>` in `<own>/cache/<build>`; a declared component's version and channel in `<own>/<component>`; every other name in `<own>`, the repository's own `<root>/<owner>/<name>`; an owner written `artifact` refused, naming it and the artifact space (chosen).** Owner `artifact` with name `npm` would make `<root>/artifact/npm/<component>` the repository of the npm name `<component>`.
2. A prefix on every repository's own, such as `<root>/repo/<owner>/<name>`. It lengthens every path to rule out one owner, and C.1 spells the root as `<root>/<owner>/<name>/`.

For where a digest is read (design D4):

1. **The store keeps, in memory for its life, the repositories each digest was reached in: a name it resolved or made, against the digest it names, and each digest a manifest read there names. read, claims-of, attach and name ask those first, in the order reached, then the repository's own, then the cache repository (chosen).** A consumer bound for `acme/app` that resolves `acme/site`'s pin reads the index, the artifacts, their bytes and their claims from the artifact's repository and asks its own nothing. What the store keeps is no record: the next process keeps it again from the names it resolves (core.20).
2. Reading the publisher from the index's `repository` annotation and asking its own repository. C.1 holds the claims beside the index in the artifact's repository, and a private publisher's own may hold builds a consumer of one name is given nothing of.
3. Asking every repository under the root. GitHub's registry serves no catalog, and 138 forbids a consumer needing the publisher.

For where a claim is written (design D5):

1. **Two writes keep the claims beside a manifest wherever it is held (chosen).** hold, which brings a manifest into a name's repository, brings timber's referrers of each manifest it holds too, from the repository it found the manifest in, by the referrers API or the referrers tag; another tool's referrers stay where they are. attach writes into the repository's own, each repository the subject was reached in, and for an index its component's repository and each of its artifacts' names' repositories, read from the index's own annotations, each where a `HEAD` finds the subject and the attestation is missing; it answers held already only where every holder held it, so an attach again after one that failed partway writes what the first left out.
2. Claims in the repository's own alone. C.1 holds every referrer a name reaches in that name's repository, and a consumer reads only the artifact's.
3. Copying claims when they are read. A reader holds no write, and a read writes nothing (60, core.32).

For a name's versions (design D6):

1. **The listing leaves out every tag in the referrers tag's shape, `sha256-<hex>` (chosen).** On a registry with no referrers API, an artifact's repository now holds its index's referrers tag. `tag::of` refuses a name in that shape, so no version's tag is lost, and any other tag that does not read back stays unreadable (core.19).
2. Reading the referrers tag back as a version, or answering the listing unreadable because of it. Either answers a version nobody published, or none, for a tag the store wrote itself.

For a store written before (design D8):

1. **Not carried (chosen).** It keeps every byte. Its undeclared component's tags in the root, its declared components' repositories `<root>/<component>`, its cache `<root>/cache/<build>`, and the claims held in the root alone are read by nothing after this; its artifact names are where they were. A host binds a new root, or publishes and promotes again under the old one, before it follows anything there.
2. A carry: a command, or reads falling back to the old places. No host binds a registry store whose versions anything follows yet; the family's store on GitHub's registry is bound after this bolt, and the ECR and GHCR checks' runs are throwaway. A carry waits on a store that needs one, and the old bytes stay for it.

## Decision outcome

Option 1 for each, as the change's design D1 to D6 and D8 say.

### Consequences

- An attach after a publish writes into two repositories plus one per artifact name, each one manifest, and on a registry with no referrers API a referrers tag rewritten in each.
- An attach that fails partway is answered with the failure unchanged; until the next attach writes what is missing, a reader of the name that lacks it reads less. A gate's move waits on its own attach's answer (core.45).
- A digest of another repository's that no name in the process reached is absent, which is what 138 asks: a consumer reaches another's versions through their names.
- The forms the profile writes and how name reads a tag are unchanged, so ECR's check asks for no new run (core.38); its next run checks the new places.

## The run on GitHub's registry

The GHCR check, `moon run ':#check-ghcr' --force`, ran by hand against the root `ghcr.io/boomdrivedev/timber-tools-check`, the repository the user approved for timber's GHCR runs (card `req_8897c0801d834cab9f78`), with the host's Docker login for `ghcr.io`, a classic token with `write:packages`, on 2026-11-18, at the change's commit 2ea5f1d5. Its live rig's stores are bound for the repository `timber-checks/store`, so its own is the package `timber-tools-check/timber-checks/store`, which no earlier run had written.

- **The first run**, from 01:58:13 to 01:59:20 UTC, passed three of its four tests, the places test among them, and failed one scenario of the store group, `the same bytes put twice`: its read was answered unreadable, `the registry answers a HEAD of the blob that it holds it, and its GET 404: {"errors":[{"code":"BLOB_UNKNOWN","message":"blob unknown to registry"}]}`. That is the window record 0039 recorded, of a package GitHub's registry made on push and that holds no manifest yet, answered as 0039 decided, and the scenario was the first write into the new package. Nothing in the profile changed.
- **The passing run**, from 01:59:48 to 02:01:05 UTC, passed all four tests, the store group, the cache manifest's scenarios, an artifact's repository made on push and kept private, and the places test. It is the proof the change's acceptance on GitHub's registry rests on.
- **The places test's words**, from a run of it alone with its output shown, from 02:01:22 to 02:02:05 UTC: it wrote into `boomdrivedev/timber-tools-check/artifact/file/run1794967285-22366-lib`, `boomdrivedev/timber-tools-check/timber-checks/store/cache/moon` and `boomdrivedev/timber-tools-check/timber-checks/store/run1794967285-22366-lib`; a store of `timber-checks/reader` resolving the component's version and the cache entry was answered absent, `name run1794967285-22366-lib:run1794967285-22366-lib-v1.0.0` and `name cache/run1794967285-22366-b7e1`; and that store read the index's and the artifact's claims from the name's package. GitHub's registry took each nested package path, four deep under the owner, on push.

## What the code stage chose

Two answers the design left open, the code stage's own, each cheap to change:

- **A digest no name reached, read by a store bound for no repository, is answered as the binding answered the repository**, absent naming the landing's remote or unreadable naming its address, since the repository's own and its cache repository are the next places asked and the store has neither. The path not taken was absent naming the digest, which reads as bytes never put where the store could not ask (core.19).
- **attach writes into the repository a read found its subject in as well**, beside the repository's own, the repositories the subject was reached in and an index's holders. That repository is one of those but where the subject is held only in the cache repository, so a claim is never answered absent for a subject a read finds. The path not taken was answering absent there, which tells a gate a manifest it can read is not held.
