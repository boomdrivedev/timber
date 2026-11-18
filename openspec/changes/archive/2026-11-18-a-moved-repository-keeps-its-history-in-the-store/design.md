## Context

Every command binds the store for the checkout's repository, the owner
and name its landing's remote's address ends in (114; record 0041's
D2). On a registry, that repository's own root, `<root>/<owner>/<name>`,
holds the repository's versions and channels, its components' and its
cache repositories, and every claim a publish writes before a name
reaches it. Artifact names are one space for the whole store,
`<root>/artifact/`, under no repository's own (C.1, 138; records 0040,
0041).

A repository renamed, or moved to another owner, gets a new address.
Every command in a checkout that follows it binds a new root, which
holds nothing. Statement C.1, as amended at `boomdrivedev/blueprints`
e8700c8 on the branch `design/timber-a-moved-repository-keeps-its-history`,
and record 0040, as amended on 2026-11-18 (b0956f3a, sluice #43), say
what must hold:

- the repository's record of claimed versions names the root it
  publishes under (22, 62), so a move is read from git alone;
- the first publish after a move carries the history to the new root
  before it publishes;
- each channel's chain goes on from its head (105), and every version
  the repository published stays promotable there;
- a read of a channel at the old root is answered with where it moved,
  never as current;
- no person runs anything, cache entries are not carried, and no other
  repository's history writes under a root one left.

What timber has to build on:

- **The record of claimed versions** is each component's tags in the
  shared history, lightweight, made without force and pushed without
  force (`version::claim`, `History::tags`).
- **A copy** of a version between two stores through the store seam
  alone, every attestation on its digests with it, the name made last
  (`copy::copy`, 103).
- **A chain brought to a store**: `ledger::mirror` attaches each claim
  of a chain the ledger answers and moves the channel's name to its
  head's index. The git ref ledger holds chains in the repository's
  refs, which move with the repository. The store ledger walks a chain
  from the channel's name in the store, so its chains are under the
  root (C.13, C.14).
- **The registry store** keeps its own repository as an answer, so a
  store bound for no repository answers every write and every name of
  its own spaces with the binding's answer, and still reads the artifact
  space (`Registry::for_repository`).

## Goals / Non-Goals

**Goals:**

- A publish in a checkout of a moved repository carries its versions,
  their claims and its channels to the new root, then publishes as
  before, with nothing asked of a person.
- After the carry, the gate promotes a version published before the
  move at the new root, each chain going on from its head.
- Every command bound for the old root, a stale checkout's or another
  repository's at the old address, is answered where the history went.
- The core names no registry, and reaches the root left only through
  the store and keeper seams (core.1, core.2).

**Non-Goals:**

- A move back to a root the repository left (D7).
- Carrying cache entries, retired channels, or a store written before
  each repository kept its own root.
- Making packages or giving access after a move on GitHub's registry
  (core.1). The guide says what the operator does.

## Decisions

### D1. The record names each root in `refs/timber/root`

The record of claimed versions gains one ref beside the tags,
`refs/timber/root`, in the shared history the tags are claimed in (the
remote `branch.<name>.remote` names). It is a chain of commits:

- each commit's tree holds one file, `root`, the root's owner and name
  as `<owner>/<name>`, lowercased, with a newline, and its message
  says the same;
- each commit's one parent is the commit before it, the first has
  none, and the author and committer are `timber <timber@localhost>`,
  as the git ref ledger's;
- the record's roots are the chain from the first, and its last root
  is where the repository's history is.

`History::roots(landed)` reads the record:

- where the branch names a remote, the remote's ref, fetched into the
  repository's copy, `refs/timber/root`, which only moves forward; a
  remote holding less than the copy is unreadable naming the commits it
  lost, and one that cannot be reached is unreadable with git's words,
  never the local copy alone;
- where it names none, the repository's own ref;
- an empty list where neither holds the ref.

`History::record_root(landed, root)` writes a commit naming `root`
after the record as read, pushes it to the remote without force, then
moves the copy to it. A push refused is read again. Where the record's
last root is then `root`, the write is done; otherwise it is refused
with git's words, and nothing is retried (core.6). With no remote, the
own ref is moved from the commit it was read at.

The root is written lowercased, as the store spells it, and two roots
are the same where their owners and names are the same lowercased. A
rename that only changes case moves nothing.

The paths not taken:

- **Annotated version tags naming the root.** A tag claimed again after
  a move keeps the root it was first claimed under, since a claim never
  moves a tag (22, core.6), so the newest tag does not say where the
  history is. A tag only the shared history holds would also have to be
  fetched to read its message.
- **A ref per root, `refs/timber/roots/<owner>/<name>`, at the
  revision first published under it.** Names are listed without a
  fetch, but two roots first published at one revision cannot be
  ordered, and a root reached twice has one ref.

### D2. The binding answers the store's root and binds a root left

`Seams` gains `roots: Box<dyn Roots>`:

```rust
pub trait Roots {
    /// C.1: the repository the store is bound for, whose root holds its
    /// spaces; none where the store places no name by repository.
    fn bound(&self) -> Answer<Option<RepositoryName>>;
    /// C.1: the store and the ledger bound for `from`, a root the
    /// repository left for the bound one, reading past the mark naming
    /// the bound one.
    fn left(&self, from: &RepositoryName) -> Answer<Left>;
}

pub struct Left {
    pub store: Box<dyn Store>,
    pub keeper: Box<dyn Keeper>,
    /// The store as a copy's answers name it, as `copy::End` takes it.
    pub source: Source,
}
```

`timber_seams::NoRoots` answers `bound` with none and `left` absent. It
is the layout's, and what every binding built in a test hands where the
test moves no repository.

The host's bindings answer for a registry store:

- `bound`: the repository the store is bound for, as `bind` answered it
  for the landing's remote (record 0041's D2), its absent or unreadable
  answer among them;
- `left(from)`: a registry for `from` under the same registry and root,
  reading past a mark naming the bound repository (D4), and the ledger
  `[ledger]` names over it: the git ref ledger as bound, since its refs
  move with the repository; the store ledger over the root left's store,
  its chains kept apart from the bound root's, under
  `<common>/timber/left/<owner>/<name>`.

Statement 92 binds one store written. The root left is a repository of
that store, under the same registry and root, and C.1 has the carry
write its mark there.

The paths not taken:

- **A store operation that carries.** The registry profile would walk
  the chains, which are the ledger's, and B.1 would gain an operation
  the statement does not list.
- **The publish command binding the root left and passing it in the
  ask.** The command would read the record before the landing is
  checked (core.11).

### D3. A move is read once per publish, before anything is built

`publish` reads the move after the landing check and the policy file's
declaration, before any component is derived, built, claimed or
offered, once whatever components it names, and even where none is
touched. `carry::root(seams, history, landed, declared)` answers what
the publish's answer carries as `rooted`:

1. `seams.roots.bound()`: none ends it with nothing read or written,
   and an absent or unreadable answer is the publish's.
2. The destination is checked (D4): a mark at the bound root refuses
   the publish.
3. `History::roots(landed)`:
   - empty: the bound root is recorded, `Rooted::Recorded`;
   - its last root the bound one: nothing, `None`;
   - another: a move, carried (D5), `Rooted::Carried`.

A publish that cannot read or write the record, or whose carry is
refused, answers that and claims nothing.

The paths not taken:

- **Reading the move at the gate or at apply.** Neither writes a
  version, and both would carry at a reader's asking.
- **Reading it per component, before each claim.** One repository has
  one root, and a publish of one component carries every component's
  versions.

### D4. A root left is marked with where it went

The mark is one name of the repository's own spaces,
`timber_seams::moved_name()`, `moved`. A version's name is a prefix and
a version, a channel's is under `channel/`, a cache entry's under
`cache/` and an artifact's under `artifact/`, so none of them is
`moved`. Its bytes are `MovedTo { to }` as canonical JSON,
`{"moved_to":"<owner>/<name>"}`, the root moved to, lowercased.

The carry writes the mark into the root left before anything else. So
no publish, promotion or report is written there while the carry reads
it.

The registry store reads the mark once in its life, at its first
operation that reaches the repository's own, its components' or its
cache repositories. It resolves `moved` in its own repository, and,
where that names a mark naming another root than the one it carries to,
answers each such operation `Refused(Why::RootMoved { from, to })` and
writes nothing. A tag absent there is no mark. A mark it cannot read is
unreadable naming the tag. An artifact's name, and the digests it
reaches, are read as before. A push's store reads no mark: it asks
nothing of the repository's own (135, core.62), and the mark guards
history alone, a publish, a claim and a channel's move.

`Why::RootMoved { from, to }` reads "`<from>`'s history moved to
`<to>`".

What a command bound for a root left answers:

- the gate, a promotion, apply, an `after` channel and status: their
  first read of a chain, a channel's name or a claim beside an index is
  `RootMoved`, the store ledger's walk among them;
- a pool's push: its cache entries written into the root's cache
  repository as at any root, never refused, since cache entries are no
  history and serve the builds of the revision they came from (record
  0040 as amended: they are not carried, and the next build fills
  them);
- a publish: refused at D3's step 2;
- a sync: as before, from the artifact space.

The paths not taken:

- **A mark per channel.** Another repository at the old address would
  still publish its own components and cache there.
- **Each core path checking the mark.** The gate, the ledger, apply,
  `after` and status each read the store their own way, and the store
  ledger's walk is a profile's.
- **Readers reading the record.** Another repository taking the old
  address holds no record of it.
- **A pool's push refused by a mark in its cache repository.** It
  refuses the builds of a revision from before the move, and leaves an
  old release branch uncached.
- **A pool's push reading the mark in the repository's own.** Record
  0040 gives the pool's credential write on its cache packages and
  nothing else.

### D5. What the carry carries, in order

`carry::root`, for a move, binds the root left,
`seams.roots.left(last)`, and lists what the record and the policy say
the repository has:

- **the versions:** for each component the policy file declares at the
  landed revision, the name of each tag carrying its prefix that the
  landed revision reaches, in version order;
- **the channels:** each channel the policy answers in force at the
  landed revision, and each environment's channel, once each.

It hands them to `carry::history`, given the root left's store with its
source and its keeper, the bound store with the publish's source and
its keeper, both roots, the versions and the channels. It reads no git
and no policy, so the tools layer runs it over registry stores. Then it records the bound root.
`carry::history`, in order:

1. **The mark.** Put `MovedTo { to }` as bytes into the root left and
   name `moved` at their digest from nothing. A mark already at the
   same digest is a carry again. One at another digest is refused
   `RootMoved` naming the root it names.
2. **The versions.** For each name: `copy` from the root left to the
   bound store. That carries every artifact and the index, and every
   claim on them: provenance, verdicts carried, the layout policy,
   acceptance, promotions and reports. A name the root left does not
   hold is passed over and listed so. Any other answer of a copy is the
   carry's.
3. **The channels.** For each channel:
   - read its chain from the root left's keeper; absent passes it over,
     and a chain answered as last read is unreadable, since the carry
     reads the source;
   - name `channel_name` at the head's index in the bound store from
     nothing; there at the head already is a carry again;
   - where the name is elsewhere, the bound keeper's chain must hold
     the head left, the channel having moved on after an earlier carry;
     otherwise it is refused as a fork naming both heads (105);
   - read the bound keeper's chain, whose head must be the head left or
     a claim after it, else unreadable naming both.

`carry::root` records the bound root, `History::record_root(landed,
bound)`, only once `carry::history` has answered.

Each step writes only what is missing. A carry that stops partway
leaves the record at the root left, so the next publish carries again
and writes only what the last one did not.

`Carried` says the roots, each version's name with whether it was
copied, held already or not held at the root left, and each channel
with its head and whether its name was made or held already.

The paths not taken:

- **Each channel's name copied from the root left.** A channel's head
  is read from its chain, never from its name (core.50).
- **Every tag the root left holds.** The store lists only an artifact
  name's versions (138), so the record of claimed versions is what
  says which versions there are.
- **Cache entries.** The statement says they are not carried.

### D6. The answer and the command's text

`Publish` gains `rooted: Option<Rooted>`, serialized as `rooted`, left
out where it is `None`. The text prints, before the components:

- `recorded the root <owner>/<name>`;
- `carried <from>'s history to <to>: <n> versions, <m> channels`, then
  a line per version passed over and per channel made.

### D7. A root the repository left, named again, is refused

Where the record's last root is another and the bound root holds a
mark, the publish is refused `RootMoved` naming where the bound root
went (D3's step 2), before anything is written. That covers:

- another repository at an address one left;
- a checkout whose remote still names the old address, which a
  forge's redirect may still serve;
- a repository moved back to a root it left.

From git alone the last two cannot be told apart. Carrying back would
have a stale checkout's publish send the history to the old address,
the next publish from an updated checkout send it back, and readers'
answers flip each time. A refusal names where the history is, and the
person corrects the remote. A move back stays refused until a later
statement says how one is told from a stale address. This is the
construct stage's choice, cheap to change.

### D8. The proof

- **The seams.** `Why::RootMoved`'s words and JSON, `MovedTo`'s bytes,
  `moved_name`, and `NoRoots`.
- **The record.** `History::roots` and `record_root` against a scratch
  repository and a bare remote: the first root written and pushed, a
  second appended, a push refused and read again, a remote holding less
  than the copy, a remote that cannot be reached, a branch with no
  remote.
- **The carry over the fakes.** In `crates/timber-core/tests/carry.rs`:
  nothing for no roots; a first root recorded; nothing for the same
  root; a move carried in order with the mark first and the record
  last; a carry again writing nothing; the destination marked; a mark
  elsewhere at the root left; a chain answered as last read; a channel
  moved on after an earlier carry; a fork.
- **The publish.** A moved repository's publish carries before it
  claims, and a refused carry claims nothing.
- **The suite.** A publish scenario where a repository publishes and
  promotes at one root, moves, publishes at the next, which carries,
  and the gate promotes the version published before the move there,
  its predecessor the head left (22, 62, 103, 105).
- **The registry stub.** A store bound for a marked root refuses its
  own spaces' operations and reads an artifact's name; a store reading
  past a mark naming its destination reads; the mark read once.
- **The bindings.** The roots a registry and a layout answer, and a
  root left's store and ledger.
- **The tools layer.** `carry::history` over two registry stores on zot
  and the reference distribution, each with the store ledger over it:
  versions with their claims and a channel carried, the chain walked at
  the new root, the old root answering `RootMoved`, and an artifact's
  name still read.
- **GitHub's registry, by hand, once.** The GHCR check gains the same
  carry between two fixed roots, `timber-checks/carried-from` and
  `timber-checks/carried-to`, every version and channel under the run's
  prefix; a later run finds the mark naming the same root and carries
  again. One passing run is the acceptance on the family's store.
- **ECR is not run.** The forms the profile writes and how name reads a
  tag are unchanged (core.38).

### D9. The record and the rule

Record 0042, status proposed, holds D1 to D5 and D7, each with its path
not taken, as the construct stage chose them on C.1 as amended at
e8700c8 and record 0040 as amended on 2026-11-18. They are its own and
cheap to change.

Rule core.78:

> A move is read from the record of claimed versions and the root the
> store is bound for alone; a publish that finds them apart marks the
> root left with where it went before it carries anything, and records
> the new root last; a root marked left answers every read and write of
> its own spaces with where it went, and only a carry reads past it.

Its details rule out a carry that writes before the mark, a record
written before the versions and channels are carried, a record ref
forced or moved back (core.47), a channel's name carried from the root
left's name (core.50), a cache entry carried, a write of history under
a root marked left, a publish, a claim or a channel's move, and a mark
read by a pool's push or by any core path but a publish's root and its
carry. They leave out a carry again writing what the last one missed,
and a pool's push of cache entries into a root marked left.

## Risks / Trade-offs

- **A carry reads every version's bytes.** `copy` reads each blob from
  the root left and puts it into the new one, through the seam, with no
  cross-repository mount. A repository with many large images pays that
  once, on its first publish after a move.
- **The mark is one more read.** Every registry store reads `moved` in
  its own repository once in its life, before its first operation
  there. A repository's first publish reads it before its packages
  exist, as it resolves its version's name today.
- **A carry stops partway.** The root left is marked, the bound root
  holds part of the history, and the record names the root left. Until
  the next publish, readers at the old root are told where it went and
  readers at the new one read part of the history. The next publish
  carries again (D5).
- **A move back is refused (D7).**
- **The first publish after a move may be refused for its cache.** A
  landing whose run pushed entries under the old root names entries the
  new root lacks, and the publish is refused naming the output after
  the carry is done (core.30). The next landing's run fills the new
  root.
- **GitHub's registry makes the new root's packages private.** Until an
  administrator sets each package as its repository is, a public
  repository's new packages are private, and a job's repository has no
  write on its new cache package. The guide says so.

## Migration Plan

None in the store. A repository's first publish after this change
records the root it binds. A repository that moved before that publish
starts its record at the new root, with nothing carried.

## Open Questions

None.
