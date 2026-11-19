# Verify: timber-removes-old-cache-entries

The verify stage's report on the branch
`change/timber-removes-old-cache-entries` at 2973de3b, cut from
`bolt/timber-publishes-github-releases-and-to-cachix` (merge base
1adfecc7).

**Verdict: the work holds.** Both layers of the whole checks pass, every
task is ticked with the work it names on the branch, and every
requirement and scenario of the change's ten delta specs is met. The
change is archived with this report.

## The whole checks

Both layers the repository's instructions name, run once at 2973de3b
inside `devenv shell`, nothing fixed between them:

| layer | command | outcome |
|---|---|---|
| `check-fast` | `moon run ':#check-fast'` | passed: 28 tasks completed, 21 of them from moon's cache at the hashes this tree gives, in 1 m 5 s |
| `check-tools` | `moon run ':#check-tools'` | passed: `cli:tools` run, not cached, 107 tests passed and none failed or ignored, in 9 m 54 s; `a_prune_removes_an_entry_past_its_time_and_leaves_a_released_version` and the store group (`zot_keeps_the_stores_contract`, `the_reference_distribution_keeps_the_stores_contract`) passing on zot and the reference distribution |

`check-fast` ran `root:changes`, `root:format`, `root:hooks`,
`root:toolchain`, `root:compile-server`, `site:install` and
`site:toolchain` afresh, and took `seams:test`, `fakes:test`,
`core:test`, `cli:test`, `conformance:test`, `conformance:coverage`,
`root:specs`, `root:lint`, `root:changes-cases`, `root:versions`,
`root:release`, `root:marketplace`, `site:build`, the `nix` project's
three tasks and the plugins' tasks from moon's cache, whose hashes are
of this tree's inputs, so each stands for a pass at 2973de3b.
`root:changes` (rule core.77) archived this change alone into a scratch
copy of the specs and validated what it wrote.

The GHCR and ECR checks are outside the gate (rule core.38) and were not
run again; their runs by hand are tasks 8.2 and 8.3's, below.

`devenv shell -- openspec validate timber-removes-old-cache-entries
--strict` answers the change valid.

## Tasks

Every task of `tasks.md`, 1.1 to 9.1, is ticked, and each has the work
it names on the branch:

- Every test the tasks name, 72 of them, is in the module or test file
  the task names it in (`a_cache_name_is_cache_and_the_build_identity`
  in `timber-seams/src/keeper.rs`, beside `channel_name`).
- 1.1: `CACHE` and `cache_name` in `timber-seams/src/keeper.rs`,
  re-exported by `timber_core::cache`; `utc_seconds` in `answer.rs`.
- 1.2, 1.3: `Pushed`, `Store::cache_entries`,
  `Store::remove_cache_entry`, `Keeping` and `Policy::cache_kept` in the
  seams, every implementer given them.
- 2.1, 2.2: the fake store stamps, lists and removes cache entries and
  records `CacheEntries` and `RemoveCacheEntry`; the fake policy answers
  thirty days given unless `keeps` or `keeps_at` set a time.
- 3.1, 3.2: `givens.toml` writes `[cache] keep_days = 30`; the policy
  file reads `[cache]`; `InForce::cache` and `timber policy`'s line
  `cache entries are kept <n> days from their push (given|written)`.
- 4.1 to 4.4: `oci::cache(digest, size, name, at)` writes the two
  annotations; `cache_entries` and `remove_cache_entry` on `Registry`
  and on the layout; a delete asks the realm for
  `repository:<repository>:pull,delete` (`registry/client.rs`); on
  `ghcr.io` `delete_version` goes through GitHub's API.
- 5.1 to 5.3: `prune` in `timber-core/src/prune.rs`; `For::Prune` binds
  the store by `bind_for` for the build profile with `push` set,
  `NoKeeper`, no projections or targets, and the signers unreaching;
  `timber prune` in `commands/prune.rs`.
- 6.1 to 6.3: `READ_AT` is `boomdrivedev/blueprints fecf4d9`, A.9 lists
  145 after 54; the store group's four cache scenarios and the core's
  four prune scenarios.
- 6.4: the store group's "a cache entry removed, its bytes' other names
  left" checks that the read after the removal answers the bytes the
  read before it did, so a store holding other bytes under a digest
  fails only "the same bytes put twice" and "bytes read back"
  (`crates/timber-conformance/tests/broken.rs`, whose expected failures
  stand as they were, passing);
  `crates/timber-conformance/tests/lagging.rs` lists the three cache
  scenarios that read an entry's name once made among the lagging
  store's failures, with their words; and
  `the_policy_in_force_asks_the_landing_and_the_policy_alone_and_writes_nothing`
  allows `PolicyOp::CacheKept` and counts seven asks. Its check,
  `check-fast` whole, passes above.
- 7.1, 7.2: record 0045 in `docs/adr/`, status proposed; rule core.80
  in `docs/architecture/core.md` under "Answers and records".
- 7.3, 7.4: the stores page says the keeping time, the cache manifest's
  two annotations, each member's removal, the first prune and the
  bytes left to the registry or the layout; the self-hosting guide has
  "### Removing old cache entries" with the `[cache] keep_days` line,
  the scheduled workflow, the ECR IAM policy, GitHub's `packages:
  write` and the admin role on the cache packages, and, under "On
  GitHub's registry", the pool's push token removing cache entries too
  and untagged versions left alone.
- 8.1: the reference distribution starts with `delete: enabled: true`,
  and `a_prune_removes_an_entry_past_its_time_and_leaves_a_released_version`
  is in `on_both!` in `crates/timber/tests/registry_store.rs`, passing
  in this run's `check-tools`.
- 8.2, 8.3, 8.4: record 0045 holds both runs by hand, on 2026-11-19
  UTC: ECR from 02:06:24 to 02:08:00 UTC after one failed run from
  02:03:08 to 02:04:52 UTC that commit c6e45d3d's fixture fixed;
  GitHub's registry from 02:08:06 to 02:09:57 UTC, against
  `ghcr.io/boomdrivedev/timber-tools-check`. The stores page's two rows
  carry the date (ee811a69). This stage did not run them again.
- 9.1: ticked, and met by this stage's archive, which applies the ten
  delta specs to `openspec/specs/`.

## Requirements and scenarios

Each requirement of the change's ten delta specs, with the code that
meets it and the tests that hold its scenarios.

### cache-keeping

- **Prune reads the keeping time at a landed revision.** `prune` asks
  `Landed::check` first, then `policy.cache_kept` at the landed
  revision, then the store, returning at the first answer that is not
  one. "A revision not landed":
  `a_revision_not_landed_is_refused_before_anything_is_asked`; "A policy
  that cannot be read": `a_keeping_time_not_read_asks_the_store_nothing`.
- **An entry past its keeping time is removed.** An entry is kept only
  where its push moment plus the days is after `now`, otherwise removed
  at the digest listed. "One past and one within":
  `an_entry_past_its_time_is_removed_and_one_within_kept`,
  `an_entry_at_its_time_exactly_is_removed`; "A keeping time the file
  writes": `a_written_keeping_time_is_used`; "A keeping time of none":
  `a_keeping_time_of_zero_removes_every_entry`.
- **An entry with no push moment is removed.** `Fate::Removed { pushed:
  None }`. "An entry from before": `an_entry_with_no_moment_is_removed`.
- **Each entry is answered on its own.** A listed entry's own answer is
  carried through `and_then` with no removal asked, and a removal's
  `Absent` is `Fate::GoneAlready`. Scenarios:
  `an_entry_listed_unreadable_is_carried_and_not_removed`,
  `a_removal_refused_stops_no_other`,
  `an_entry_gone_before_its_removal_is_gone_already`.
- **Prune removes cache entries and nothing else.** Prune calls the
  landing, the policy and the store's two cache operations alone. "A
  released version beside old entries":
  `a_released_version_reads_back_whole_after_its_entries_are_removed`,
  `prune_asks_nothing_of_the_keeper_the_signers_the_projections_or_the_targets`.

### seam-contracts

- **The store lists and removes cache entries.** "An entry listed", "A
  removal from where the name is not", "A removal of no entry": the
  store group's "a cache entry listed with when it was pushed", "a cache
  entry's removal from where it is not" and "a removal of no cache
  entry", over the fake, the layout, the registry stub, and zot and the
  reference distribution in the tools layer.
- **The policy answers how long cache entries are kept.** "No time
  written": `the_fake_policy_keeps_thirty_days_given`,
  `no_cache_table_answers_the_givens_thirty_days`.
- **A cache entry's store name is spelled once.** "The name of an
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

- **The keeping time is read from `[cache]`, given thirty days.** "A
  time written": `the_keeping_time_written_is_answered_written`; "No
  time written": `no_cache_table_answers_the_givens_thirty_days`,
  `a_revision_with_no_file_answers_the_givens_thirty_days`; "A time of
  none": `keep_days_of_zero_is_answered`.
- **A `[cache]` table outside its form is unreadable.** "Another key":
  `another_key_under_cache_is_unreadable_naming_it`; "A time of another
  form": `keep_days_of_another_form_is_unreadable_naming_it`.

### registry-store

- **A cache entry's bytes are named through a cache manifest.** "Named
  and resolved": `a_cache_manifest_names_its_entry_and_the_moment_it_was_put`
  and the tools layer's `bytes_are_named_through_a_cache_manifest`; "The
  same bytes named twice":
  `two_names_of_the_same_bytes_each_have_a_cache_manifest`; "A name
  found held before the put": `a_name_found_held_puts_no_cache_manifest`;
  the move, no-artifact and bytes-not-held scenarios over the store
  group and the tools layer.
- **A cache repository's entries are listed with their push moments.**
  "Entries of both forms":
  `a_cache_repositorys_entries_are_listed_with_their_moments`,
  `an_entry_written_before_moments_lists_none`; "A tag at another
  manifest": `a_tag_at_another_manifest_is_listed_unreadable_naming_it`;
  "A cache repository never made":
  `a_cache_repository_never_made_lists_no_entries`.
- **A cache entry is removed by deleting its cache manifest.**
  `remove_cache_entry` checks the tag's manifest's layer, then deletes
  by digest. "An entry removed":
  `an_entry_removed_deletes_its_cache_manifest_alone`,
  `nothing_outside_the_cache_repository_is_listed_or_deleted`; "A
  registry that refuses the delete":
  `a_delete_the_registry_refuses_is_refused_in_its_words`; "A root
  marked moved": `a_removal_at_a_root_marked_moved_reads_no_mark`.
- **On GitHub's registry a cache entry is removed through GitHub's
  API.** "An organization's package version deleted":
  `ghcr_removes_an_organizations_package_version_by_the_manifests_digest`,
  `ghcr_sends_the_docker_credentials_token_as_the_bearer`; "GitHub
  refuses the delete": `ghcr_answers_a_403_refused_in_githubs_words`;
  "No version of that digest":
  `ghcr_answers_no_version_of_the_digest_absent`.
- **Removal passes against real registries.** "A prune on both
  registries":
  `a_prune_removes_an_entry_past_its_time_and_leaves_a_released_version`
  in this run's tools layer; ECR and GitHub's registry by hand (tasks
  8.2, 8.3).

### layout-store

- **A name made at bytes says when it was made.** "A cache entry's
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
  time.** "The text of a prune":
  `prune_removes_an_entry_past_its_time_and_keeps_one_within`; "An
  entry that cannot be read": `prune_exits_unreadable_where_an_entry_is`,
  `prune_prints_its_answer_as_json`.
- **Policy shows how long cache entries are kept.** "The given time
  shown": `the_text_says_how_long_cache_entries_are_kept`, with
  `the_keeping_time_is_in_force`.

### conformance-suite

- **The store's group lists and removes cache entries.** Four scenarios
  in `seams/store.rs`, citing 145 with 52, 54 and 100; every registry
  rig bound for `moon`. "The group over the fake store": met, and the
  broken and lagging stores' tests of the same group pass.
- **Prune over the fakes.** Four scenarios in `core/cache.rs`, citing
  145 with 22, 34 and 54. "The cache group over the fakes": met.

### clause-coverage

- **The suite knows the clauses of Parts A and B.** `READ_AT` is
  `boomdrivedev/blueprints fecf4d9`, A.9 lists 145 after 54, and
  `the_parts_hold_every_clause_from_1_to_145_once_in_20_sections`.
- **The clause the statement added at 145 is cited.** 145 is cited by
  the store group's and prune's scenarios and is in no register;
  `conformance:coverage` passes in this run's `check-fast`.
