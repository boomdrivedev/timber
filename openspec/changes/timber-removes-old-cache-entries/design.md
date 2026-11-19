## Context

Clause 145, added at `boomdrivedev/blueprints` fecf4d9 on 2026-11-18,
the design agent's:

> The store keeps a repository's cache entries for as long as its
> owner's settings say, thirty days from their push unless they say
> otherwise, and timber removes each entry once that time has passed,
> with a credential that may remove cache entries and nothing else. It
> never removes for the cache anything accepted or released (54), the
> record of versions claimed (22), or the ledger (34).

What timber has to build on:

- **A cache entry's name** is `cache/<build identity>`, spelled by
  `timber_core::cache::cache_name`, made once from nothing at the
  entry's bytes (52, rule core.6). No channel, index or policy names one
  (53, core.30).
- **On a registry** a cache entry is a tag of its build profile's cache
  repository, `<root>/<owner>/<name>/cache/<build>`, the build identity
  the tag, at a cache manifest whose one layer is the bytes and whose
  config is the empty descriptor (C.1). The manifest carries no
  annotation, so the same bytes always write the same manifest
  (`oci::cache`). A push's store reaches the cache repository alone and
  reads no mark (135, record 0042).
- **In a layout** a cache entry is an entry of `index.json` naming the
  bytes, its ref name `cache/<build>` (C.2).
- **The store seam removes nothing.** `FakeStore::forget` stands in for
  a store's own retention.
- **GitHub's registry** removes a package version through GitHub's REST
  API or its settings, and documents no other way. A workflow's
  `GITHUB_TOKEN` may delete a version where its repository holds the
  admin role on the package (GitHub's "Deleting and restoring a
  package"). Record 0039 says a package's untagged versions are never
  deleted, since artifacts' manifests and referrers are held untagged.
- **ECR** takes the distribution protocol's delete of a manifest by its
  digest: AWS's guide to removing artifacts deletes one with
  `oras manifest delete <repository>@<digest>`.
- **Times** are seconds since the Unix epoch, `u64`, and
  `timber_seams::utc` writes one as RFC 3339 in UTC. A command reads the
  host's clock and hands the core the moment (`promote`, `status`).

## Goals / Non-Goals

**Goals:**

- `timber prune` removes each of a repository's cache entries pushed
  longer ago than the keeping time the policy gives at a landed
  revision, thirty days by default.
- No code path can remove anything but a cache entry: the store seam's
  removal names a build identity, never a name.
- The credential prune needs removes cache entries and nothing else in
  the store, on GitHub's registry and on ECR, as the guide sets it up.
- The core names no registry (core.1); GitHub's API is the registry
  profile's alone.

**Non-Goals:**

- A root the repository left, a build profile the host does not bind,
  a schedule, the registries' own collection of bytes, untagged
  manifests, a dry run (the proposal's out of scope).

## Decisions

### D1. The keeping time is the policy file's `[cache] keep_days`, given thirty

"Its owner's settings" are the repository's policy file, the one policy
the owner keeps (C.17), read at a landed revision (65, core.22):

```toml
[cache]
keep_days = 14
```

- `keep_days` is a whole number of days from 0. With 0, each entry is
  removed at the next prune.
- The givens' fragment, `givens.toml`, gains `[cache] keep_days = 30`,
  laid beneath the repository's file as the given channels are. A
  repository that writes `keep_days` replaces it (C.10).
- Any other key under `[cache]`, and a `keep_days` that is no whole
  number from 0, is unreadable naming the key and the file, as an
  environment's words outside C.10 are.
- The table is the repository's, not a component's: a repository's
  cache entries sit under its own root, whatever component's tasks left
  them (C.1).
- The policy seam answers it, so nothing but the policy profile reads
  the key (core.43):

  ```rust
  pub struct Keeping {
      pub days: u32,
      /// The givens' time, where the file writes none.
      pub given: bool,
  }

  /// 145: how long the repository at `revision` keeps its cache
  /// entries from their push.
  fn cache_kept(&self, revision: &Revision) -> Answer<Keeping>;
  ```

  A revision whose tree holds no policy file answers the givens' time.
- `timber policy` shows it beneath what acceptance and a layout
  require: `cache entries are kept 30 days from their push (given)`, or
  `(written)`. Its JSON carries `cache`, `{"days":30,"given":true}`, or
  the answer that was not one.

The paths not taken:

- **The host's bindings file.** Two hosts publishing one repository
  would keep its cache for two times, and the setting would not travel
  with the repository.
- **The store's own setting, an ECR lifecycle rule.** 145 has timber
  remove, and GitHub's registry has no such rule.
- **A duration's text, `30d`.** A parser and its units for one number.
- **A line per component.** The entries are the repository's.

### D2. A cache manifest names its entry and when it was put

The cache manifest gains two annotations:

- `io.github.agentplot.timber.store-name`: the store name it was made
  for, `cache/<build>` for a cache entry, or `moved` for a root's mark;
- `org.opencontainers.image.created`: the moment the store put it, the
  host's clock, written by `utc` to the second.

So each name made at bytes has a cache manifest of its own. Deleting the
manifest by its digest removes that one name, which a registry's delete
and GitHub's version delete both do with every tag at the manifest.
Its config and its one layer are as before, and resolving the tag still
answers the layer's digest (C.1).

- `oci::cache(digest, size, name, at)` writes it. A new
  `oci::cache_pushed(bytes) -> Result<Option<u64>, String>` reads the
  moment: `None` for a manifest with no `created` annotation, written
  before this change, and the parser's words for one that is not
  `YYYY-MM-DDTHH:MM:SSZ`.
- The registry takes its clock from a hook, the system's by default,
  so its tests fix the moment (`Registry::with_clock`).
- Where name's check before its put finds the name held, no cache
  manifest is put (rule core.6), so a write that loses leaves no
  untagged manifest behind.
- A layout stamps the index entry of each name it makes or moves at
  bytes with `org.opencontainers.image.created`, beside its ref name.
- `timber_seams::utc_seconds(text) -> Option<u64>` reads back what `utc`
  writes, and nothing of another form.

The forms the registry profile writes change, so the GHCR and ECR checks
run by hand before the change lands (core.38, D9).

The paths not taken:

- **The store's own push time.** The distribution protocol carries
  none. ECR's and GitHub's are in their own APIs, and zot's and the
  reference distribution's in none, so the time would come from four
  places, two of them absent.
- **One manifest per bytes, with only the tag deleted.** A tag's delete
  is optional in the distribution protocol, and GitHub's API deletes a
  version with all its tags, so two entries of the same bytes, such as
  two empty outputs, would go together, the second before its time.
- **The time in the config.** C.1 makes the config the empty
  descriptor.
- **A referrer naming the time.** A cache entry carries no claim beyond
  the writing host's own (52), and a referrer is one more write per
  entry.

### D3. The store seam lists and removes cache entries

```rust
/// 145: a cache entry the store holds: the digest its name is at, and
/// when it was pushed, in seconds since the Unix epoch; `None` where
/// the store holds no moment for it.
pub struct Pushed {
    pub digest: Digest,
    pub at: Option<u64>,
}

/// 145: every cache entry the store holds for the repository and the
/// build profile it is bound for, each by its build identity, in no
/// order the core relies on; an entry the store cannot read answered
/// on its own; the empty list where it holds none, and unreadable only
/// where the listing cannot be read (rule core.19).
fn cache_entries(&self) -> Answer<Vec<(BuildId, Answer<Pushed>)>>;

/// 145, 54: the cache entry of `build` removed, only where its name is
/// at `at`; refused `Why::NameAt` naming where it is otherwise, and
/// nothing removed; absent where the store names no entry for it. No
/// other name, and no bytes another name reaches, are removed (100).
fn remove_cache_entry(&self, build: &BuildId, at: &Digest) -> Answer<()>;
```

- The cache name's spelling moves into the seams, `CACHE` and
  `cache_name(build) -> Name`, beside `channel_name`, so the store and
  the core share one function (as rule core.48 has for a channel).
  `timber_core::cache` re-exports both, and every caller is unchanged.
- Every implementer of `Store` gains both. A wrapper delegates. A store
  that holds no cache entries answers both absent naming cache entries.

The paths not taken:

- **A removal of any name.** A seam that removes a name could remove a
  version's, a channel's or an artifact's. 145 lets timber remove cache
  entries alone.
- **The cache manifest's digest in the seam.** The seam addresses bytes
  (C.1); the manifest is the registry profile's.

### D4. A registry lists its cache repository's tags and deletes the entry's manifest

The store bound for prune reaches the bound build profile's cache
repository alone, as a push's does (`bound_for(build, true)`). It asks
nothing of the repository's own and reads no mark, since cache entries
are no history (record 0042).

`cache_entries`:

1. The cache repository's tags, `GET /v2/<repository>/tags/list`, page
   by page as the registry's `Link` header leads. A repository the
   registry does not hold, `NAME_UNKNOWN`, holds no entries: the empty
   list.
2. Each tag's manifest. A cache manifest answers its layer's digest and
   `cache_pushed`'s moment. A tag at any other manifest, or one whose
   manifest cannot be read, is that entry's answer, unreadable naming
   the tag and the registry's or the parser's words. It is never
   removed.

`remove_cache_entry`:

1. The tag's manifest. A tag the repository does not hold is absent
   naming it. A manifest of another type is refused naming the tag.
   A cache manifest whose layer is not `at` is refused `NameAt` naming
   the layer.
2. `DELETE /v2/<repository>/manifests/<the manifest's digest>`. A `202`
   is done, a `404` is absent, and any other answer is refused or
   unreadable in the registry's words, as every request's is (58, 59).
   A delete asks the realm for `pull,delete` on the repository, as a
   write asks `pull,push` (record 0039).

Nothing outside the cache repository is listed or deleted, and no
untagged manifest is (record 0039). The layer's blob is the registry's
to collect once nothing names it.

A store bound for no build profile answers both refused naming it, as a
cache name's place does now.

### D5. GitHub's registry removes through GitHub's API

Where the registry's host is `ghcr.io`, step 2 of the removal goes
through GitHub's REST API. The listing stays on the distribution
protocol, which GitHub's registry serves (record 0039).

- The namespace is the root's first part, and the package the rest of
  the cache repository's path, each `/` written `%2F`: the root
  `ghcr.io/boomdrivedev` and the repository `boomdrivedev/timber` give
  the package `boomdrivedev%2Ftimber%2Fcache%2Fmoon` of `boomdrivedev`.
- `GET {api}/users/{namespace}` answers its `type`, read once in the
  store's life. An `Organization` reads
  `{api}/orgs/{namespace}/packages/container/{package}/versions`, and a
  `User` reads `{api}/users/{namespace}/packages/container/{package}/versions`.
- The versions are read page by page, `?per_page=100&page=<n>`, until
  the version whose `name` is the cache manifest's digest. Its `id` is
  deleted, `DELETE …/versions/{id}`, and a `204` is done.
- Each request sends `Accept: application/vnd.github+json`,
  `X-GitHub-Api-Version: 2022-11-28` and `Authorization: Bearer
  <token>`. The token is the one the host's Docker credential for
  `ghcr.io` holds, reached at use, as every request of the store reaches
  it (record 0015, rule core.21).
- A version not found, and a `404`, are absent naming the package and
  the digest. A `401` or `403` is refused in GitHub's words, and a `5xx`
  or a body that cannot be read is unreadable.
- `{api}` is `https://api.github.com`. Tests set it with
  `Registry::with_github_api(origin)`, so the stub plays GitHub's API
  and its answers stand in the tests verbatim.

The credential is the repository's own scheduled workflow's token, with
`permissions: packages: write`. The repository is given the admin role
on its cache packages, `<root>/<owner>/<name>/cache/<build>`, and
nothing on its component and artifact packages, which the publishing
host makes (record 0040). That token writes and removes cache entries
and nothing else in the store (145, 135).

The paths not taken:

- **The distribution protocol's delete on GitHub's registry.** GitHub
  documents removal only through its API and its settings.
- **A token file named in `[credentials]`.** The store's credential is
  the host's Docker login (record 0015), and one token reaches both.
- **GitHub's version `created_at` as the push moment.** It would give
  one registry a second source of time (D2).

### D6. A layout lists its `cache/` names and removes the name alone

- `cache_entries` answers each entry of `index.json` whose ref name
  begins `cache/`, with the digest it names and its `created`
  annotation's moment, `None` where it has none.
- `remove_cache_entry` removes the entry under the index's lock, only
  where it names `at`, refused `NameAt` naming where it is otherwise,
  and absent where there is none. The bytes stay on disk.
- The fake store does the same in memory, stamping each name made or
  moved under `cache/` from a clock a test may set, the system's by
  default, and keeping the bytes.

The path not taken: **removing the blob where nothing else reaches
it.** A layout keeps no count of what reaches a blob, and deciding by
reading every manifest it holds risks an artifact's bytes (54, rule
core.5). The disk is the host's own.

### D7. `timber prune`

The core's `prune::prune(seams, ask) -> Answer<Pruned>`, asked
`PruneAsk { branch, revision, now }`:

1. `Landed::check`: a revision the branch has not landed is refused
   naming both, before anything else is asked (14, rule core.11).
2. `cache_kept` at the landed revision. Any answer but a time ends
   prune with that answer, and the store is not asked: nothing is
   removed while the time is not known.
3. `cache_entries`.
4. Each entry, in the order listed:
   - an entry answered otherwise than listed is carried as answered,
     and nothing is removed;
   - an entry with no moment is removed: it was pushed before push
     moments were written;
   - an entry whose moment plus `days × 86 400` is at or before `now`
     is removed;
   - any other is kept, naming the moment its time passes.

   Each removal is answered on its own, and one never stops the next.
   A removal answered absent is gone already, which another prune or the
   store's own retention did (54).

```rust
pub struct Pruned {
    pub branch: Branch,
    pub revision: Revision,
    pub keeping: Keeping,
    pub entries: Vec<(BuildId, Answer<Fate>)>,
}

pub enum Fate {
    Removed { pushed: Option<u64> },
    GoneAlready { pushed: Option<u64> },
    Kept { pushed: u64, until: u64 },
}
```

Prune asks the landing, the policy and the store's two cache operations,
and nothing else. It signs nothing, keeps nothing (61) and reads and
writes no git ref.

The command, `timber prune [revision] [--branch main] [--json]`:

- the revision any name git reads for a commit, by default the local
  branch's head, as `push` takes it; `now` the host's clock;
- the text says the branch, the revision and the keeping time, then a
  line per entry: `<build> removed, pushed <utc>`, `<build> removed,
  pushed before push times were kept`, `<build> gone already`, `<build>
  kept until <utc>, pushed <utc>`, or the answer that was not one;
- `--json` prints the answer as one document;
- the exit is the worst of the entries' answers, as push's is.

Its binding, `For::Prune`, holds the landing and the policy as the file
names them or as given, and the store bound for the build profile's
cache repository as a push's is. A layout is opened and never made. The
keeper, the signers, the projections and the targets are each absent
naming the binding, and no signer is reached.

What runs it is the operator's (core.52, C.16): the guide's scheduled
workflow of the repository's own, or a host's own schedule, or a
person.

### D8. Nothing but a cache entry is removed

Each part keeps 145's last sentence by what it can reach:

- the seam removes a cache entry by its build identity, and has no
  operation that names any other name;
- a registry lists and deletes in the bound build profile's cache
  repository alone, and only a tagged cache manifest there;
- a layout removes an index entry under `cache/` and no blob;
- prune reads and writes no git ref, so the record of claimed versions
  (22) and the git ref ledger (34) are out of its reach, and a store
  ledger's chains and channels' names are outside the cache space.

The tests show each: the store group's scenarios over every store
profile, and a prune over the fakes after a version is published,
accepted and promoted, whose every name, digest and claim outside
`cache/` reads back as before.

### D9. The proof

- **The seams.** `utc_seconds` reading back `utc`, the cache name's
  spelling, `Keeping` and `Pushed` round-tripping through JSON.
- **The fakes and the policy file.** The fake store's listing, moments,
  removals and kept bytes; the fake policy's keeping time; the file's
  `[cache]`, the given, and each refusal.
- **The stores.** The cache manifest's annotations and `cache_pushed`;
  the registry's listing and removal over the stub, GitHub's API played
  by the stub; the layout's stamp, listing and removal.
- **Prune.** Over the fakes in `crates/timber-core/tests/prune.rs`, and
  the command over a layout and a branch's landing in
  `crates/timber/tests/prune.rs`.
- **The suite.** 145 read at fecf4d9. The store group's cache scenarios
  over the fake, the layout, the stub and every live registry, with
  each registry rig's store bound for the build profile `moon`. Prune's
  scenarios in the core's cache group over the fakes.
- **The tools layer.** The reference distribution started with deletes
  on. A prune over a registry store on zot and on the reference
  distribution: an entry past its time removed, one within kept, and a
  version, its claims and a channel's name left.
- **GitHub's registry and ECR, by hand, once each.** The store group's
  cache scenarios against each, removing through GitHub's API on one
  and the distribution protocol's delete on the other (core.38). The
  GHCR run needs a login whose token may delete the check's cache
  package's versions.

### D10. The record and the rule

Record 0045, status proposed, holds D1, D2, D4 to D7, each with its path
not taken, as this change's construct stage chose them on 145. They are
its own and cheap to change.

Rule core.80:

> Only prune removes from the store, and only cache entries: each by
> its build identity, at the digest it was listed at, once the keeping
> time the policy gives at a landed revision has passed since its push;
> no code path removes a name outside the cache space, bytes another
> name reaches, an untagged manifest, a git ref or a ledger's claim.

Its details rule out a seam operation removing any other name, a
registry delete outside the cache repository or of an untagged manifest
(record 0039), a layout's blob removed for the cache, a keeping time
read from a checkout or a host's file (core.22), an entry with a moment
removed before its time, prune reading or moving a git ref, and timber
starting prune (core.52). They leave out an entry with no moment
removed at any prune, a removal racing a push of the same build
identity, and a pool's push token on GitHub's registry removing cache
entries too where its repository holds the admin role on them (54).

## Risks / Trade-offs

- **The first prune removes every entry pushed before this change.**
  Those read with no moment. A landing whose entries are removed and
  whose publish has not run is refused naming the output (135,
  core.30); the next landing's run fills the cache.
- **A landing must be published within the keeping time.** A publish
  never builds where the landing has verdicts (135), so a landing whose
  entries are gone is not published. A landing is published straight
  after it lands, so thirty days leaves room.
- **One manifest read per entry.** A cache repository of many entries
  costs a request each to list. Prune runs on a schedule, not on a
  landing's path.
- **A cache manifest per name.** Two entries of the same bytes write two
  manifests where they wrote one, a few hundred bytes each.
- **GitHub's API for a token's deletes is in public preview**, by
  GitHub's own page. A change in it shows in the GHCR check by hand.
- **Clocks.** A push moment is the pushing host's clock and the time
  passed is the pruning host's. A skewed clock removes an entry early or
  late, and early costs a rebuild alone (54).
- **The registry collects the bytes, not timber.** The reference
  distribution keeps them until its operator runs its garbage
  collection; a layout keeps them.

## Migration Plan

None in the store. Entries pushed before this change are removed at the
first prune, and every push after writes its moment.

## Open Questions

None.
