# Verify: timber-removes-old-cache-entries

The verify stage's report on the branch
`change/timber-removes-old-cache-entries` at 82967d71, cut from
`bolt/timber-publishes-github-releases-and-to-cachix` (merge base
1adfecc7). The code stage archived the change as its own task 9.1
(82967d71), so this report sits in the archived change's directory.

**Verdict: the work falls short.** `check-fast` fails on
`conformance:test`: the store group's broken-profile test in
`crates/timber-conformance/tests/broken.rs` expects two failing
scenarios over a store holding other bytes under a digest, and this
change's new cache scenario fails there as a third. The change goes
back to its code stage. Every task is ticked, and the specs, the code
and the tests the tasks name are otherwise met, as below.

## The whole checks

Both layers the repository's instructions name, run once at 82967d71
inside `devenv shell`, nothing fixed between them:

| layer | command | outcome |
|---|---|---|
| `check-fast` | `moon run ':#check-fast'` | **failed**: `conformance:test` exited 101, one test failed |
| `check-tools` | `moon run ':#check-tools'` | passed: `cli:tools` run, not cached, 107 tests passed and none failed or ignored, in 8 m 19 s, `a_prune_removes_an_entry_past_its_time_and_leaves_a_released_version` passing on zot and the reference distribution |

### What failed in `check-fast`

`crates/timber-conformance/tests/broken.rs`,
`a_store_holding_other_bytes_under_a_digest_fails_naming_what_it_read`:

```
assertion `left == right` failed: the store group over the profile `a store holding other bytes under a digest`: 3 of 13 scenarios failed, 1 not applicable
- `the same bytes put twice` (clauses 1, 5, 66): read of the bytes put answered "other bytes under that digest"
- `bytes read back` (clauses 1, 72, 73): assertion `left == right` failed: the first read answered bytes whose digest is sha256:89279cb176e78788ee415e35ff7b4f2c2b59c7ec2b189b8b1ba796d269f3c98a, under sha256:3641be509b3d4c957dfd04f009a2acab95b9dc2c85cf00930b5322a0723563ed
- `a cache entry removed, its bytes' other names left` (clauses 145, 54, 100): the bytes read back are others
- `a name moved from a stale place, repaired from the ledger` (clauses 70, 109): not applicable: ...
  left: ["the same bytes put twice", "bytes read back", "a cache entry removed, its bytes' other names left"]
 right: ["the same bytes put twice", "bytes read back"]
```

Task 6.2 added "a cache entry removed, its bytes' other names left" to
the store group (`crates/timber-conformance/src/seams/store.rs`), which
reads the entry's bytes back after the removal. `OtherBytesRig` answers
other bytes on every read, so the scenario rightly fails over it, and
the test's list of the scenarios it expects to fail
(`failed(&report, OTHER, &["the same bytes put twice", "bytes read
back"])`) was not brought along. Task 6.2's own verification ran
`--test fakes` and the registry's and layout's groups, never `--test
broken`. What the code stage must do: make the broken-profile test
expect the new scenario among the failures, with the words it names,
or otherwise reconcile the scenario with that test; then run
`check-fast` whole.

When `conformance:test` failed, moon stopped the tasks still running
with it, so `seams:test`, `fakes:test`, `core:test`, `cli:test`,
`conformance:coverage`, `root:lint` and `nix:typecheck` were cut off
unfinished at 82967d71 and gave no outcome. `root:specs` (50 passed),
`root:changes`, `root:format`, `root:hooks`, `root:toolchain`,
`root:compile-server`, `site:install` and `site:toolchain` passed;
`site:build`, `nix:lint`, `nix:test`, `root:versions`, `root:release`,
`root:marketplace`, `root:changes-cases` and the plugins' tasks were
taken from moon's cache at hashes this tree gives.

The GHCR and ECR checks are outside the gate (core.38) and were not
run again; their runs by hand are tasks 8.2 and 8.3's, below.

## The archive

OpenSpec's own `openspec archive timber-removes-old-cache-entries
--yes`, run on a scratch copy of `openspec/` at 82967d71's parent,
writes the ten specs the branch holds byte for byte: `cache-keeping`
new, and `seam-contracts`, `seam-fakes`, `policy-file`,
`registry-store`, `layout-store`, `host-bindings`, `command-line`,
`conformance-suite` and `clause-coverage` (24 added, 2 modified).

## Tasks

Every task of `tasks.md`, 1.1 to 9.1, is ticked, and each has the work
it names on the branch:

- Every test the tasks name, 72 of them, is in the module or test file
  the task names it in (`a_cache_name_is_cache_and_the_build_identity`
  in `timber-seams/src/keeper.rs`, beside `channel_name`).
- 1.1: `CACHE` and `cache_name` in `timber-seams/src/keeper.rs`;
  `utc_seconds` in `answer.rs`.
- 1.2, 1.3: `Pushed`, `Store::cache_entries`,
  `Store::remove_cache_entry`, `Keeping` and `Policy::cache_kept` in the
  seams, every implementer given them.
- 4.2: a delete asks the realm for `repository:<repository>:pull,delete`
  (`registry/client.rs`).
- 5.2: `For::Prune` binds the store by `bind_for` for the build profile
  with `push` set, `NoKeeper`, no projections or targets, and the
  signers unreaching.
- 7.1, 7.2: record 0045 in `docs/adr/`, status proposed; rule core.80
  in `docs/architecture/core.md` under "Answers and records".
- 7.3, 7.4: the stores page says the keeping time, the cache manifest's
  two annotations, each member's removal and the first prune; the
  self-hosting guide has "### Removing old cache entries", the ECR IAM
  policy, GitHub's `packages: write` and admin role, and untagged
  versions left alone.
- 8.1: the reference distribution starts with `delete: enabled: true`,
  and `a_prune_removes_an_entry_past_its_time_and_leaves_a_released_version`
  is in `on_both!` in `crates/timber/tests/registry_store.rs`.
- 8.2, 8.3, 8.4: record 0045 holds both runs by hand, on 2026-11-19:
  ECR from 02:06:24 to 02:08:00 UTC (`cli:tools-ecr`, hash cd8bac63,
  three tests passing) after one failed run from 02:03:08 to 02:04:52
  UTC that commit c6e45d3d's fixture fixed; GitHub's registry from
  02:08:06 to 02:09:57 UTC (`cli:tools-ghcr`, hash e6e22e32, five tests
  passing), against `ghcr.io/boomdrivedev/timber-tools-check`. The
  stores page's two rows carry the date (ee811a69). This stage did not
  run them again.
- 9.1: the change archived and the ten specs written (82967d71).

Task 6.2's check is met only in part: the group passes over the fake,
the layout and the stub, but the gate's broken-profile test of the same
group fails, above.

## Requirements and scenarios

Each requirement of the change's ten delta specs, with the code that
meets it and the tests that hold its scenarios.

### cache-keeping

- **Prune reads the keeping time at a landed revision.** `prune` in
  `timber-core/src/prune.rs` asks `Landed::check` first, then
  `policy.cache_kept` at the landed revision, then the store, returning
  at the first answer that is not one. "A revision not landed":
  `a_revision_not_landed_is_refused_before_anything_is_asked`; "A policy
  that cannot be read": `a_keeping_time_not_read_asks_the_store_nothing`.
- **An entry past its keeping time is removed.** An entry is kept only
  where its push moment plus the days exceeds `now`, otherwise removed
  at the digest listed. "One past and one within":
  `an_entry_past_its_time_is_removed_and_one_within_kept`,
  `an_entry_at_its_time_exactly_is_removed`; "A keeping time the file
  writes": `a_written_keeping_time_is_used`; "A keeping time of none":
  `a_keeping_time_of_zero_removes_every_entry`.
- **An entry with no push moment is removed.** `Fate::Removed { pushed:
  None }`, printed "pushed before push times were kept". "An entry from
  before": `an_entry_with_no_moment_is_removed`.
- **Each entry is answered on its own.** A listed entry's own answer is
  carried through `and_then`, and a removal's `Absent` is
  `Fate::GoneAlready`. Scenarios:
  `an_entry_listed_unreadable_is_carried_and_not_removed`,
  `a_removal_refused_stops_no_other`,
  `an_entry_gone_before_its_removal_is_gone_already`.
- **Prune removes cache entries and nothing else.** Prune calls the
  landing, the policy and the store's two cache operations alone. "A
  released version beside old entries":
  `a_released_version_reads_back_whole_after_its_entries_are_removed`,
  `prune_asks_nothing_of_the_keeper_the_signers_the_projections_or_the_targets`.

### seam-contracts

- **The store lists and removes cache entries.** `Store::cache_entries`
  and `Store::remove_cache_entry` in `store.rs`. "An entry listed", "A
  removal from where the name is not", "A removal of no entry": the
  store group's "a cache entry listed with when it was pushed", "a cache
  entry's removal from where it is not" and "a removal of no cache
  entry", over the fake, the layout and the registry stub.
- **The policy answers how long cache entries are kept.**
  `Policy::cache_kept` and `Keeping { days, given }`. "No time written":
  `the_fake_policy_keeps_thirty_days_given`,
  `no_cache_table_answers_the_givens_thirty_days`.
- **A cache entry's store name is spelled once.** `cache_name` in
  `keeper.rs`, re-exported by `timber_core::cache`. "The name of an
  entry": `a_cache_name_is_cache_and_the_build_identity`.

### seam-fakes

- **The fake store lists and removes cache entries with when each was
  pushed.** "A removed entry's bytes":
  `the_fake_store_removes_an_entry_and_keeps_its_bytes`,
  `the_fake_store_records_its_cache_operations`; "A moment a test sets":
  `a_moment_a_test_sets_is_listed`,
  `the_fake_store_lists_a_cache_entry_with_its_moment`.
- **The fake policy answers the keeping time a test sets.** "A time set
  for one revision":
  `the_fake_policy_answers_a_time_set_at_one_revision_alone`.

### policy-file

- **The keeping time is read from `[cache]`, given thirty days.**
  `givens.toml` writes `[cache] keep_days = 30`; `keep_days` in
  `profiles/policy/file/mod.rs`. "A time written":
  `the_keeping_time_written_is_answered_written`; "No time written":
  `no_cache_table_answers_the_givens_thirty_days`,
  `a_revision_with_no_file_answers_the_givens_thirty_days`; "A time of
  none": `keep_days_of_zero_is_answered`.
- **A `[cache]` table outside its form is unreadable.** "Another key":
  `another_key_under_cache_is_unreadable_naming_it`; "A time of another
  form": `keep_days_of_another_form_is_unreadable_naming_it`.

### registry-store

- **A cache entry's bytes are named through a cache manifest.**
  `oci::cache(digest, size, name, at)` writes
  `io.github.agentplot.timber.store-name` and
  `org.opencontainers.image.created`. "Named and resolved":
  `a_cache_manifest_names_its_entry_and_the_moment_it_was_put`, the
  tools layer's `bytes_are_named_through_a_cache_manifest`; "The same
  bytes named twice": `two_names_of_the_same_bytes_each_have_a_cache_manifest`;
  "A name found held before the put":
  `a_name_found_held_puts_no_cache_manifest`; "Bytes not held": the
  tools layer's same scenario; the move and the no-artifact scenarios
  stand as before over the store group.
- **A cache repository's entries are listed with their push moments.**
  `cache_entries` on `Registry`, page by page. "Entries of both forms":
  `a_cache_repositorys_entries_are_listed_with_their_moments`,
  `an_entry_written_before_moments_lists_none`; "A tag at another
  manifest": `a_tag_at_another_manifest_is_listed_unreadable_naming_it`;
  "A cache repository never made":
  `a_cache_repository_never_made_lists_no_entries`.
- **A cache entry is removed by deleting its cache manifest.**
  `remove_cache_entry` checks the tag's manifest's layer, then
  `delete_manifest` by digest. "An entry removed":
  `an_entry_removed_deletes_its_cache_manifest_alone`,
  `nothing_outside_the_cache_repository_is_listed_or_deleted`; "A
  registry that refuses the delete":
  `a_delete_the_registry_refuses_is_refused_in_its_words`; "A root
  marked moved": `a_removal_at_a_root_marked_moved_reads_no_mark`.
- **On GitHub's registry a cache entry is removed through GitHub's
  API.** `delete_version`: the namespace's type read once, versions page
  by page, the version deleted by id, the Docker credential's token as
  the bearer. "An organization's package version deleted":
  `ghcr_removes_an_organizations_package_version_by_the_manifests_digest`,
  `ghcr_sends_the_docker_credentials_token_as_the_bearer`; "GitHub
  refuses the delete": `ghcr_answers_a_403_refused_in_githubs_words`;
  "No version of that digest":
  `ghcr_answers_no_version_of_the_digest_absent`.
- **Removal passes against real registries.** "A prune on both
  registries":
  `a_prune_removes_an_entry_past_its_time_and_leaves_a_released_version`
  in the tools layer; ECR and GitHub's registry by hand (tasks 8.2,
  8.3).

### layout-store

- **A name made at bytes says when it was made.** The layout stamps
  `org.opencontainers.image.created` by its clock. "A cache entry's
  index entry": `a_name_made_at_bytes_is_stamped`.
- **The layout lists its cache entries and removes a name alone.** "An
  entry removed and its bytes kept": `a_removed_entry_leaves_its_blob`;
  "An entry from before": `an_entry_from_before_lists_no_moment`.

### host-bindings

- **Prune binds the landing, the policy and the store's cache alone.**
  "A registry's binding for prune":
  `prunes_registry_store_reaches_the_cache_repository_alone`,
  `prune_binds_no_keeper_and_reaches_no_signer`; "A host with no
  layout": `prune_makes_no_layout`.

### command-line

- **Prune takes a landed revision and removes what has passed its
  time.** `commands/prune.rs`, its exit the worst of the entries'
  answers. "The text of a prune":
  `prune_removes_an_entry_past_its_time_and_keeps_one_within`; "An
  entry that cannot be read": `prune_exits_unreadable_where_an_entry_is`,
  `prune_prints_its_answer_as_json`.
- **Policy shows how long cache entries are kept.** `InForce::cache` and
  `commands/policy.rs`'s `cache`. "The given time shown":
  `the_text_says_how_long_cache_entries_are_kept`, with
  `the_keeping_time_is_in_force`.

### conformance-suite

- **The store's group lists and removes cache entries.** Four scenarios
  in `seams/store.rs`, citing 145 with 52, 54 and 100; every registry
  rig bound for `moon`. "The group over the fake store": met over the
  fake; but the same group run over the broken store of `broken.rs`
  fails the gate, above.
- **Prune over the fakes.** Four scenarios in `core/cache.rs`, citing
  145 with 22, 34 and 54. "The cache group over the fakes": met.

### clause-coverage

- **The suite knows the clauses of Parts A and B.** `READ_AT` is
  `boomdrivedev/blueprints fecf4d9`, A.9 lists 145 after 54, and
  `the_parts_hold_every_clause_from_1_to_145_once_in_20_sections`.
- **The clause the statement added at 145 is cited.** 145 is cited by
  the store group's and prune's scenarios and is in no register.
  `conformance:coverage` was cut off unfinished in this run, so the
  coverage check's own pass at 82967d71 is not shown here.
