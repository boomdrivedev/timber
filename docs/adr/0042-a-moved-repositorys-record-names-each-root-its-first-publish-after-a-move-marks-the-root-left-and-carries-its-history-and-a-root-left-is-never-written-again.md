---
number: 42
title: A moved repository's record names each root, its first publish after a move marks the root left and carries its history, and a root left is never written again
status: proposed
date: 2026-11-18
decision-makers:
- the construct stage of the change a-moved-repository-keeps-its-history-in-the-store, by its design D1 to D5 and D7
- the code stage of the same change, by the answers its design left open, under "What the code stage chose"
---

# A moved repository's record names each root, its first publish after a move marks the root left and carries its history, and a root left is never written again

Status: proposed. Every choice here is the construct stage's own, made on statement C.1 as amended at `boomdrivedev/blueprints` e8700c8, on the branch `design/timber-a-moved-repository-keeps-its-history`, and on record 0040 as amended on 2026-11-18 (b0956f3a), and each is cheap to change: no form the registry profile writes, no layout store and no digest changes, and a store keeps every byte it held, a root left gaining one tag.

Deciders: the construct stage of the change `a-moved-repository-keeps-its-history-in-the-store`, carrying out record 0040's amendment of 2026-11-18, boomdrive-design's, on batch 35's review: a repository that is renamed or moves owner keeps one history in the store, carried to its new root by its first publish after the move.

Sources: statement clauses 22, 62, 92, 103, 105, 121 and C.1 as amended at `boomdrivedev/blueprints` e8700c8; records 0040 as amended on 2026-11-18 and 0041; rules core.6, core.30, core.47, core.48, core.50; the change's design D1 to D5 and D7.

## Context and problem statement

A repository's components and cache entries hang under its own root in the store, `<root>/<owner>/<name>/`, the owner and name its remote gives (C.1, record 0040). A repository renamed or moved to another owner gets a new remote address, so every command in a checkout that follows it binds a new root, which holds nothing: each channel's chain would start again there, no version published before the move could be promoted, and a target still reading a channel at the old root would read it as current.

C.1 as amended says what must hold: the record of claimed versions names the root the repository publishes under (22, 62), so a move is read from git alone; the first publish after a move carries the history to the new root before it publishes; each channel's chain goes on from its head (105), and every version stays promotable there; a read of a channel at the old root is answered with where it moved; no person runs anything, cache entries are not carried, and no other repository's history writes under a root one left. Six questions follow: where the record names the root, how the core reaches the root left, when a publish reads a move, how a root left answers, what a carry carries and in what order, and what becomes of a root the repository left named again.

## Considered options

For where the record names the root (design D1):

1. **`refs/timber/root`, beside the version tags in the shared history, a chain of commits each holding one file, `root`, the root's owner and name lowercased, pushed without force and read into a copy that only moves forward (chosen).** Its last root is where the history is, two roots are one where they are the same lowercased, and a push refused is read again, done where the record now names the root and refused with git's words otherwise (core.6, core.47).
2. Annotated version tags naming the root. A tag claimed again after a move keeps the root it was first claimed under, since a claim never moves a tag (22, core.6), so the newest tag does not say where the history is; and a tag only the shared history holds would have to be fetched to read its message.
3. A ref per root, `refs/timber/roots/<owner>/<name>`, at the revision first published under it. Names are listed without a fetch, but two roots first published at one revision cannot be ordered, and a root reached twice has one ref.

For how the core reaches the root left (design D2):

1. **The binding gains its roots: the repository the store is bound for, none for a store that places nothing by repository, as a layout does; and, asked for a root left, that root's store under the same registry and root, reading past the mark naming the bound root, and the ledger the file names over it (chosen).** The git ref ledger is bound as it is, since its refs move with the repository, and the store ledger over the root left's store keeps its chains under `timber/left/<owner>/<name>` in the common git directory, apart from the bound root's. Statement 92 binds one store written; the root left is a repository of that store, and C.1 has the carry write its mark there.
2. A store operation that carries. The registry profile would walk the chains, which are the ledger's, and B.1 would gain an operation the statement does not list.
3. The publish command binding the root left and passing it in the ask. The command would read the record before the landing is checked (core.11).

For when a publish reads a move (design D3):

1. **Once per publish, after the landing check and the policy file's declaration and before any component is derived, built, claimed or offered, whatever components it names and where none is touched (chosen).** No root bound reads and writes nothing; a bound root holding a mark refuses the publish; an empty record is given the bound root; a record whose last root is the bound one writes nothing; and a record whose last root is another is a move, carried. A publish that cannot read or write the record, or whose carry is refused, answers that and claims nothing.
2. At the gate or at apply. Neither writes a version, and both would carry at a reader's asking.
3. Per component, before each claim. One repository has one root, and a publish of one component carries every component's versions.

For how a root left answers (design D4):

1. **One name of the repository's own, `moved`, made by the seam's one function, at the canonical JSON `{"moved_to":"<owner>/<name>"}` of the root moved to, lowercased; written by the carry before anything else; read by the registry store once in its life, at its first operation reaching the repository's own, its components' or its cache repositories, every such operation then refused `RootMoved` naming both roots, and the artifact space read as before (chosen).** No version's, channel's, cache entry's or artifact's name can be `moved` (core.48). A command bound for a root left is answered at its first read of a chain, a channel's name or a claim beside an index; a pool's push is refused with nothing written; a sync reads the artifact space as before.
2. A mark per channel. Another repository at the old address would still publish its own components and cache there.
3. Each core path checking the mark. The gate, the ledger, apply, `after` and status each read the store their own way, and the store ledger's walk is a profile's.
4. Readers reading the record. Another repository taking the old address holds no record of it.

For what a carry carries and in what order (design D5):

1. **The mark first; then each version the record claims, every tag carrying a declared component's prefix that the landed revision reaches, in version order, copied with every claim on its digests (103); then each channel the policy answers in force and each environment's channel, once each, its chain read from the root left as its source holds it, its name made in the bound store at its head's index from nothing, a name elsewhere accepted only where the bound chain holds the head left and refused as a fork naming both heads otherwise (105), and the bound chain checked last; and the bound root recorded only once the carry answers (chosen).** Each step writes only what is missing, so a carry that stops partway leaves the record at the root left and the next publish carries again, writing only what the last one did not. The carry reads no git and no policy, so the tools layer runs it over registry stores.
2. Each channel's name copied from the root left. A channel's head is read from its chain, never from its name (core.50).
3. Every tag the root left holds. The store lists only an artifact name's versions (138), so the record of claimed versions is what says which versions there are.
4. Cache entries. The statement says they are not carried; the pool's next run fills the new root, and a publish whose verdicts name entries only the old root holds is refused naming the output, as any publish whose entry is gone (54, 135, core.30).

For a root the repository left, named again (design D7):

1. **Refused naming where the bound root went, before anything is written: another repository at an address one left, a checkout whose remote still names the old address, and a repository moved back to a root it left alike (chosen).** From git alone the last two cannot be told apart. The refusal names where the history is, and the person corrects the remote.
2. Carrying back. A stale checkout's publish would send the history to the old address, the next publish from an updated checkout would send it back, and readers' answers would flip each time. A move back stays refused until a later statement says how one is told from a stale address.

## Decision outcome

Option 1 for each, as the change's design D1 to D5 and D7 say.

### Consequences

- A carry reads every version's bytes from the root left and puts them into the new root through the seam, with no cross-repository mount; a repository with many large images pays that once, on its first publish after a move.
- Every registry store reads `moved` in its own repository once in its life, before its first operation there.
- A carry that stops partway leaves the root left marked, the bound root holding part of the history and the record naming the root left; until the next publish, readers at the old root are told where it went and readers at the new one read part of it.
- The first publish after a move may be refused for its cache, where the landing's run pushed entries under the old root; the next landing's run fills the new root.
- On GitHub's registry the carry makes the new root's packages private, and a job's repository has no write on its new cache package, until an administrator sets each as the self-hosting guide says (record 0040).
- A repository that moved before its first publish under this record starts its record at the new root, with nothing carried; a store written before each repository kept its own root is not carried (record 0041's D8).

## What the code stage chose

Answers the design left open, the code stage's own, each cheap to change:

- **The struct D2 calls `Left` is `timber_seams::RootLeft`.** The build seam's `Left`, what a checkout's run left (139), already holds the name, and the seams crate exports both at its root. The path not taken was a module path, `bind::Left`, which every use would have to spell out.
- **A commit of the record is written at the landed revision's committer time.** The core reads no clock (`PromoteAsk` carries the moment for the same reason), and two hosts recording one root on one parent at one revision write one commit. The path not taken was the clock's time, which makes two such commits differ for nothing.
- **A carry answers each channel's name `made`, `held_already` or `moved_on`.** A name the bound root holds past the head left, after an earlier carry, is neither made nor held at the head, and saying so tells the reader why the carry left it. The path not taken was answering it held already.
- **A fork the carry finds names the last claim the two chains share as its predecessor, none where they share none, and the two heads as its claims.** The path not taken was always naming none, which reads as two first claims.
- **Rule core.78's details name the reads of the mark the core keeps beside the store's own profile:** a publish's root reading the bound root's mark through the store seam, and a carry writing the root left's mark and reading it there, as design D3's step 2 and D5's step 1 have them. The path not taken was design D9's "a mark read by any but the store's own profile", which those two steps would break.
- **A push's store reads no mark, for now.** It asks nothing of the repository's own, which a pool's credential may not read (135, rule core.62; record 0040), so it cannot read `moved` there, and a pool's push bound for a root left is not refused as design D4 says it is. How a push learns where a root went is asked of the design. The path not taken was reading the mark in the repository's own, which refuses every pool's push of a private repository whose job is given write on its cache package alone.
- **The conformance world's root moved to is bound with a fresh store and a keeper holding the chains the root left's keeper held**, as the git ref ledger's refs move with the repository; a fake keeper reads no store, so a keeper holding nothing would answer no chain the carry could go on from.
