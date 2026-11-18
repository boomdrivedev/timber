---
number: 40
title: Each repository keeps its own root in the store, and on GitHub's registry the pool writes only its cache
status: accepted
date: 2026-11-17
decision-makers: boomdrive-design (the record and every choice in it)
---

# Each repository keeps its own root in the store, and on GitHub's registry the pool writes only its cache

Status: accepted, boomdrive-design's. It carries out a choice the user already made: GitHub's registry under boomdrivedev as the family's store, public for the tools and private for swancloud (card `req_02f6ed1d21734e7ba2da`, 2026-11-16).

Amended 2026-11-17, boomdrive-design's, on the construct of `each-repositorys-spaces-sit-under-its-own-root`: artifact pointers stay one space for the whole store, since a consumer finds a pin by ecosystem, name and version alone (138, core.60); only a repository's components and cache entries move under its root, spelled by its remote's owner and name.

Deciders: boomdrive-design, on two questions batch 33's review left open, relayed by boomdrive-planner on 2026-11-17.

Sources: statement C.1 (the store's three spaces under one declared root) and 135 (a pool's credential writes cache entries and nothing else), 95 and 96 (no build host holds the accepting key); record 0039 (what GitHub's registry answered, proposed on the bolt `timber-publishes-from-githubs-registry`); GitHub's documentation on a package's access control and visibility, where each image name is a package with its own visibility and its own access list, and a repository's Actions may be given read, write or admin on a package.

## Context and problem statement

C.1 hangs every repository's spaces under one root that the host binds. A component's tags sit in a repository of their own, but cache entries share `cache/<build>` and artifact pointers share `artifact/`. On GitHub's registry each image name is a package, public or private as a whole. So swancloud's private cache entries would land in the same `cache/moon` package as the public tools' entries.

Clause 135 wants the pool's credential to write cache entries and nothing else. GitHub has no token scoped to some packages of an owner. A job's own token writes the packages its repository has been given, and can also make new ones.

## Considered options

1. **A root per repository, under the host's one store (chosen).** Each repository's components and cache entries hang under `<root>/<owner>/<name>/`. Each package is public or private as its repository is, and one host still binds one store.
2. **A second store or a second host for private repositories.** Twice the bindings for one difference in visibility. It also breaks "a host binds one store".
3. **A repository names its own store in its policy.** The store is the host's binding (C.17), and a repository's file that named stores would follow the repository to every host that checks it out.

For artifact pointers:

1. **One `artifact/<ecosystem>/<name>` space for the whole store (chosen).** A consumer resolves what its lockfile pins by ecosystem, name and version alone and reads the version's claims beside its index there, knowing nothing of the publisher (138, core.60). Each name is already a package of its own, so its visibility is its publisher's without a root.
2. **Under the publishing repository's root.** A consumer would need the publisher to find a pin, which 138 forbids, and GitHub's registry serves no catalog to search for it.

For the pool's credential:

1. **The job's own token, given write on its repository's cache packages alone (chosen).** The publishing host makes the repository's component and artifact packages before the first publish and gives the job's repository nothing on them. A package the job could still make on its own holds no name anyone trusts: every name a consumer follows must carry the gate's signed acceptance, which no build host can make (95, 96).
2. **A token of an account that may make no packages at all.** A second account to keep, for no gain over option 1 once acceptance is checked.
3. **No push from the pool.** Then the publish has nothing in the store to read (135).

## Decision outcome

Option 1 for both.

- **A repository's components and cache hang under its own root.** The host binds a registry and a root. A repository's component tags and cache entries live under `<root>/<owner>/<name>/`, its remote's owner and name lowercased (114, 138), so its packages' visibility is its own.
- **Artifact pointers stay store-wide.** `<root>/artifact/<ecosystem>/<name>` holds a name's pointers, its index and the claims beside it, one package per name, public or private as its publisher's repository is. The publishing host makes it before the name's first publish and gives no job anything on it.
- **On GitHub's registry the pool writes only its cache.** The job pushes with its own token, which is given write on its repository's cache packages and nothing else. The publishing host makes the component and artifact packages first, so no job can take one over.
- **The guarantee rests on the claims as well.** Clause 135's aim is that a build can never offer, accept or move a name a consumer follows. On GitHub it holds because the packages that carry names grant the job nothing, and because a name without the gate's acceptance is refused wherever it is read.

### Consequences

- C.1's root and 135's reading on GitHub change in the statement. ECR's creation templates cover each repository's root and `artifact/`.
- A cache entry is shared only within its repository. The family's repositories share none today.
