---
number: 40
title: Each repository keeps its own root in the store, and on GitHub's registry the pool writes only its cache
status: accepted
date: 2026-11-17
decision-makers: boomdrive-design (the record and every choice in it)
---

# Each repository keeps its own root in the store, and on GitHub's registry the pool writes only its cache

Status: accepted, boomdrive-design's. It carries out a choice the user already made: GitHub's registry under boomdrivedev as the family's store, public for the tools and private for swancloud (card `req_02f6ed1d21734e7ba2da`, 2026-11-16).

Deciders: boomdrive-design, on two questions batch 33's review left open, relayed by boomdrive-planner on 2026-11-17.

Sources: statement C.1 (the store's three spaces under one declared root) and 135 (a pool's credential writes cache entries and nothing else), 95 and 96 (no build host holds the accepting key); record 0039 (what GitHub's registry answered, proposed on the bolt `timber-publishes-from-githubs-registry`); GitHub's documentation on a package's access control and visibility, where each image name is a package with its own visibility and its own access list, and a repository's Actions may be given read, write or admin on a package.

## Context and problem statement

C.1 hangs every repository's spaces under one root that the host binds. A component's tags sit in a repository of their own, but cache entries share `cache/<build>` and artifact pointers share `artifact/`. On GitHub's registry each image name is a package, public or private as a whole. So swancloud's private cache entries would land in the same `cache/moon` package as the public tools' entries.

Clause 135 wants the pool's credential to write cache entries and nothing else. GitHub has no token scoped to some packages of an owner. A job's own token writes the packages its repository has been given, and can also make new ones.

## Considered options

1. **A root per repository, under the host's one store (chosen).** Each repository's component, cache and artifact spaces hang under `<root>/<repository>/`. Each package is public or private as its repository is, and one host still binds one store.
2. **A second store or a second host for private repositories.** Twice the bindings for one difference in visibility. It also breaks "a host binds one store".
3. **A repository names its own store in its policy.** The store is the host's binding (C.17), and a repository's file that named stores would follow the repository to every host that checks it out.

For the pool's credential:

1. **The job's own token, given write on its repository's cache packages alone (chosen).** The publishing host makes the repository's component and artifact packages before the first publish and gives the job's repository nothing on them. A package the job could still make on its own holds no name anyone trusts: every name a consumer follows must carry the gate's signed acceptance, which no build host can make (95, 96).
2. **A token of an account that may make no packages at all.** A second account to keep, for no gain over option 1 once acceptance is checked.
3. **No push from the pool.** Then the publish has nothing in the store to read (135).

## Decision outcome

Option 1 for both.

- **A repository's spaces hang under its own root.** The host binds a registry and a root. A repository's component tags, cache entries and artifact pointers live under `<root>/<repository>/`, so its packages' visibility is its own.
- **On GitHub's registry the pool writes only its cache.** The job pushes with its own token, which is given write on its repository's cache packages and nothing else. The publishing host makes the component and artifact packages first, so no job can take one over.
- **The guarantee rests on the claims as well.** Clause 135's aim is that a build can never offer, accept or move a name a consumer follows. On GitHub it holds because the packages that carry names grant the job nothing, and because a name without the gate's acceptance is refused wherever it is read.

### Consequences

- C.1's root and 135's reading on GitHub change in the statement. ECR's creation templates take the repository's root as their prefix.
- A cache entry is shared only within its repository. The family's repositories share none today.
