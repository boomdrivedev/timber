# Tasks

Every command runs from the place's root, inside `devenv shell`.
Commit by pathspec, Conventional Commits scoped to the part of the
tree, citing clauses by number, rules by id and this change's design
by D-number. Submit nothing: Flume submits the place once its last
stage delivers. The headings are for reading; each task's check is its
own.

Read these before starting:

- This change's `design.md`, whole.
- Statement clauses 145, 52 to 54, 100, 135 and C.1, at
  `boomdrivedev/blueprints` fecf4d9, `design/timber/requirements.md`.
- `crates/timber-seams/src/store.rs`, `policy.rs`, `keeper.rs`
  (`channel_name`) and `answer.rs` (`utc`).
- `crates/timber-core/src/cache.rs`, `push.rs` and `in_force.rs`.
- `crates/timber/src/profiles/store/oci.rs` (`cache`, `cached`),
  `registry/mod.rs` (`bound_for`, `place`, `cache_repository`, `named_at`,
  `cache`), `registry/client.rs`, `registry/stub.rs` and `layout/`.
- `crates/timber/src/profiles/policy/file/mod.rs` and `givens.toml`.
- `bind` and `For` in `crates/timber/src/bindings/mod.rs`, and
  `crates/timber/src/commands/push.rs`.
- Records 0015, 0039, 0040 and 0042.

## 1. The seams

- [x] 1.1 Spell a cache entry's name in the seams, and read back a time (design D2, D3; seam-contracts spec, "A cache entry's store name is spelled once").
  - Move `CACHE` and `cache_name(build: &BuildId) -> Name` from `crates/timber-core/src/cache.rs` into `crates/timber-seams`, beside `channel_name`, exported from `lib.rs`, citing 52 and rule core.48. `timber_core::cache` re-exports both, so every caller compiles unchanged.
  - Add `utc_seconds(text: &str) -> Option<u64>` to `answer.rs`, reading `YYYY-MM-DDTHH:MM:SSZ` back to seconds since the Unix epoch, `None` for text of any other form.
  - Tests in the modules they test: `a_cache_name_is_cache_and_the_build_identity`, `utc_seconds_reads_back_what_utc_writes`, `utc_seconds_reads_nothing_of_another_form`.
  - Verify with `devenv shell -- cargo test -p timber-seams`.
- [x] 1.2 Add the store's two cache operations (design D3; seam-contracts spec, "The store lists and removes cache entries").
  - In `store.rs`: `Pushed { digest: Digest, at: Option<u64> }`, serializable; `Store::cache_entries(&self) -> Answer<Vec<(BuildId, Answer<Pushed>)>>` and `Store::remove_cache_entry(&self, build: &BuildId, at: &Digest) -> Answer<()>`, each with the comment design D3 gives it.
  - Give every implementer both, found by `grep -rln 'impl Store for' crates`. A wrapper delegates: `Unmade` in `profiles/store/mod.rs`, `Unopened` in `bindings/mod.rs` answering its own answer, and the test wrappers in `timber-core` and `timber-conformance`. Every other implementer answers both absent naming cache entries; tasks 2.1, 4.2 and 4.4 replace the fake store's, the registry's and the layout's.
  - A test in `store.rs`: `a_pushed_round_trips_through_json`.
  - Verify with `devenv shell -- cargo check --workspace --all-targets --all-features`, which builds every implementer.
- [ ] 1.3 Add the policy's keeping time (design D1; seam-contracts spec, "The policy answers how long cache entries are kept").
  - In `policy.rs`: `Keeping { days: u32, given: bool }`, serializable, and `Policy::cache_kept(&self, revision: &Revision) -> Answer<Keeping>`, its comment citing 145 and C.10.
  - Give every implementer it, found by `grep -rln 'impl Policy for' crates`: the test doubles answer thirty days given; tasks 2.2 and 3.1 replace the fake policy's and the policy file's.
  - A test in `policy.rs`: `a_keeping_round_trips_through_json`.
  - Verify with `devenv shell -- cargo check --workspace --all-targets --all-features`.

## 2. The fakes

- [ ] 2.1 List and remove cache entries in the fake store, `crates/timber-fakes/src/store.rs` (design D6; seam-fakes spec, "The fake store lists and removes cache entries with when each was pushed").
  - `StoreState` keeps a moment per name made or moved under `cache/`, from the store's clock: the system's, unless a test sets one with `FakeStore::clock(seconds)`. `FakeStore::pushed(build, Option<u64>)` sets an entry's moment, or none.
  - `cache_entries` lists every name under `cache/` with its digest and moment; `remove_cache_entry` removes the name where it is at the digest given, refused `NameAt` naming where it is otherwise, absent where there is none, and keeps the bytes.
  - `StoreOp` gains `CacheEntries` and `RemoveCacheEntry`, recorded and scriptable as the other operations are. `forget`'s comment says it stands in for a store's own retention beside `remove_cache_entry`.
  - Tests in the module: `the_fake_store_lists_a_cache_entry_with_its_moment`, `the_fake_store_removes_an_entry_and_keeps_its_bytes`, `the_fake_store_refuses_a_removal_from_where_the_name_is_not`, `the_fake_store_answers_a_removal_of_no_entry_absent`, `a_moment_a_test_sets_is_listed`, `the_fake_store_records_its_cache_operations`.
  - Verify with `devenv shell -- cargo test -p timber-fakes store`.
- [ ] 2.2 Answer the keeping time in the fake policy, `crates/timber-fakes/src/policy.rs` (design D1; seam-fakes spec, "The fake policy answers the keeping time a test sets"). `FakePolicy::keeps(days)` sets it for every revision and `keeps_at(revision, days)` for one, each answered written; otherwise thirty days given. Tests: `the_fake_policy_keeps_thirty_days_given`, `the_fake_policy_answers_a_time_set_at_one_revision_alone`. Verify with `devenv shell -- cargo test -p timber-fakes policy`.

## 3. The policy file

- [ ] 3.1 Read `[cache]` in `crates/timber/src/profiles/policy/file/` (design D1; policy-file spec, "The keeping time is read from `[cache]`, given thirty days", "A `[cache]` table outside its form is unreadable").
  - `givens.toml` gains `[cache]` with `keep_days = 30` and a comment citing 145, and `Givens` reads it beside the channels.
  - `File` reads `[cache]`. `cache_kept` answers the file's `keep_days`, written, or the fragment's, given, where the file writes none or the revision holds no file. Another key under `[cache]`, and a `keep_days` that is no whole number from 0 or past `u32`, is unreadable naming the key and the file.
  - Tests in `tests.rs`: `the_keeping_time_written_is_answered_written`, `no_cache_table_answers_the_givens_thirty_days`, `a_revision_with_no_file_answers_the_givens_thirty_days`, `keep_days_of_zero_is_answered`, `another_key_under_cache_is_unreadable_naming_it`, `keep_days_of_another_form_is_unreadable_naming_it`.
  - Verify with `devenv shell -- cargo test -p timber --lib profiles::policy::file`.
- [ ] 3.2 Show the keeping time in `timber policy` (design D1; command-line spec, "Policy shows how long cache entries are kept").
  - `InForce` in `crates/timber-core/src/in_force.rs` gains `cache: Answer<Keeping>`, asked of the policy at the same revision, serialized as `cache` as every part of the answer is.
  - `crates/timber/src/commands/policy.rs` prints, beneath acceptance and the layout, `cache entries are kept <n> days from their push (given)` or `(written)`, or the answer that was not one. Say so in the module's comment.
  - Tests: `the_keeping_time_is_in_force` in `crates/timber-core/tests/in_force.rs`, and a test module in `commands/policy.rs` with `the_text_says_how_long_cache_entries_are_kept`.
  - Verify with `devenv shell -- cargo test -p timber-core --test in_force the_keeping_time_is_in_force` and `devenv shell -- cargo test -p timber --lib commands::policy`.

## 4. The stores

- [ ] 4.1 Name each cache manifest's entry and moment, in `crates/timber/src/profiles/store/oci.rs` and `registry/mod.rs` (design D2; registry-store spec, "A cache entry's bytes are named through a cache manifest").
  - `oci::cache(digest, size, name, at)` writes the annotations `io.github.agentplot.timber.store-name` and `org.opencontainers.image.created`, the second by `utc`; its comment no longer says the same bytes write the same manifest. `oci::cache_pushed(bytes) -> Result<Option<u64>, String>` reads the moment as design D2 says.
  - The registry passes the name it makes and its clock's moment. `Registry::with_clock(Box<dyn Fn() -> u64>)` sets the clock, the system's by default. Where name's check before its put finds the name held, no cache manifest is put. The root's mark `moved` is written the same way.
  - Update the module tests that expected one cache manifest for two names of the same bytes. Add: `a_cache_manifest_names_its_entry_and_the_moment_it_was_put` and `a_cache_manifest_with_no_moment_reads_none` in `oci.rs`; `two_names_of_the_same_bytes_each_have_a_cache_manifest`, `a_name_found_held_puts_no_cache_manifest` and `the_mark_is_written_through_a_cache_manifest_naming_moved` in the registry's tests.
  - Verify with `devenv shell -- cargo test -p timber --lib profiles::store::oci` and `devenv shell -- cargo test -p timber --lib profiles::store::registry::tests`.
- [ ] 4.2 List and remove a registry's cache entries (design D4; registry-store spec, "A cache repository's entries are listed with their push moments", "A cache entry is removed by deleting its cache manifest").
  - `cache_entries` and `remove_cache_entry` on `Registry`, as design D4 says: the tags of the bound build profile's cache repository, page by page; each tag's manifest read; a repository the registry does not hold listing none; the removal checking the tag's manifest and its layer, then deleting the manifest by its digest; a delete asking the realm for `pull,delete`. Neither reads the mark nor asks anything of the repository's own. A store bound for no build profile answers both refused naming it.
  - The stub in `registry/stub.rs` answers a manifest's `DELETE` by digest, removing it and every tag at it, and pages a tag list where a test asks.
  - Say so in the module's comment and in `Store for Registry`'s, citing 145 and record 0039.
  - Tests over the stub: `a_cache_repositorys_entries_are_listed_with_their_moments`, `an_entry_written_before_moments_lists_none`, `a_tag_at_another_manifest_is_listed_unreadable_naming_it`, `a_cache_repository_never_made_lists_no_entries`, `the_listing_reads_every_page_of_tags`, `an_entry_removed_deletes_its_cache_manifest_alone`, `a_removal_at_another_digest_is_refused_naming_the_layer`, `a_removal_of_a_tag_not_held_is_absent`, `a_delete_the_registry_refuses_is_refused_in_its_words`, `a_removal_at_a_root_marked_moved_reads_no_mark`, `nothing_outside_the_cache_repository_is_listed_or_deleted`.
  - Verify with `devenv shell -- cargo test -p timber --lib profiles::store::registry::tests`.
- [ ] 4.3 Remove through GitHub's API on GitHub's registry (design D5; registry-store spec, "On GitHub's registry a cache entry is removed through GitHub's API").
  - Where the host is `ghcr.io`, or a test has set `Registry::with_github_api(origin)`, the removal's delete goes through GitHub's REST API as design D5 says: the namespace's type read once, the package's versions read page by page, the version named the cache manifest's digest deleted by its id, the headers and the bearer token as D5 gives them, the token from the client's Docker credential for the registry.
  - The stub plays GitHub's API where a test asks: `/users/<namespace>`, the organization's and the user's package versions, and the delete, recording each request with its headers.
  - Say so in the module's comment, naming GitHub's API as this profile's alone (rule core.1).
  - Tests over the stub: `ghcr_removes_an_organizations_package_version_by_the_manifests_digest`, `ghcr_removes_a_users_package_version`, `ghcr_reads_every_page_of_versions`, `ghcr_sends_the_docker_credentials_token_as_the_bearer`, `ghcr_answers_no_version_of_the_digest_absent`, `ghcr_answers_a_403_refused_in_githubs_words`, `ghcr_reads_the_namespaces_type_once`, `ghcr_escapes_the_package_name`.
  - Verify with `devenv shell -- cargo test -p timber --lib profiles::store::registry::tests::ghcr`.
- [ ] 4.4 Stamp, list and remove in the layout, `crates/timber/src/profiles/store/layout/` (design D2, D6; layout-store spec, "A name made at bytes says when it was made", "The layout lists its cache entries and removes a name alone").
  - `name` stamps the entry of a name it makes or moves at bytes with `org.opencontainers.image.created`, from a clock the layout takes as the registry does, the system's by default.
  - `cache_entries` and `remove_cache_entry` as design D6 says, the removal under the index's lock and the blob left on disk.
  - Tests in the module: `a_name_made_at_bytes_is_stamped`, `the_layout_lists_its_cache_entries_with_their_moments`, `an_entry_from_before_lists_no_moment`, `a_removed_entry_leaves_its_blob`, `a_removal_at_another_digest_is_refused_naming_where_it_is`, `a_removal_of_no_entry_is_absent`.
  - Verify with `devenv shell -- cargo test -p timber --lib profiles::store::layout`.

## 5. Prune

- [ ] 5.1 Prune in a new `crates/timber-core/src/prune.rs`, exported from `lib.rs` (design D7, D8; cache-keeping spec, every requirement).
  - `prune(seams, ask: &PruneAsk) -> Answer<Pruned>`, with `PruneAsk { branch, revision, now }`, `Pruned` and `Fate` as design D7 gives them, serializable as push's answer is, each entry's answer beside its build identity.
  - The module's comment cites 145, 54, 22, 34 and 61, and says prune asks the landing, the policy and the store's two cache operations alone.
  - Tests in a new `crates/timber-core/tests/prune.rs` over the fakes: `a_revision_not_landed_is_refused_before_anything_is_asked`, `a_keeping_time_not_read_asks_the_store_nothing`, `an_entry_past_its_time_is_removed_and_one_within_kept`, `an_entry_at_its_time_exactly_is_removed`, `a_written_keeping_time_is_used`, `a_keeping_time_of_zero_removes_every_entry`, `an_entry_with_no_moment_is_removed`, `an_entry_listed_unreadable_is_carried_and_not_removed`, `a_removal_refused_stops_no_other`, `an_entry_gone_before_its_removal_is_gone_already`, `prune_asks_nothing_of_the_keeper_the_signers_the_projections_or_the_targets`, `a_released_version_reads_back_whole_after_its_entries_are_removed`.
  - Verify with `devenv shell -- cargo test -p timber-core --test prune`.
- [ ] 5.2 Bind prune, in `crates/timber/src/bindings/mod.rs` (design D7; host-bindings spec, "Prune binds the landing, the policy and the store's cache alone").
  - `For::Prune`: the landing and the policy as for push; the store bound by `profiles::store::bind_for` for the build profile with `push` set, so it reaches the cache repository alone, a layout opened and never made; the keeper, projections and targets absent naming the binding, and the signers unreaching. Say so in `For`'s and `bind`'s comments.
  - Tests in `bindings/tests.rs`: `prunes_registry_store_reaches_the_cache_repository_alone`, `prune_makes_no_layout`, `prune_binds_no_keeper_and_reaches_no_signer`.
  - Verify with `devenv shell -- cargo test -p timber --lib bindings::tests::prune`.
- [ ] 5.3 Add the command, `crates/timber/src/commands/prune.rs` (design D7; command-line spec, "Prune takes a landed revision and removes what has passed its time").
  - `PruneArgs { revision, branch, json }` as push's are; the revision resolved as push resolves it; `now` from the host's clock; the text and the JSON as design D7 gives them; the exit the worst of the entries' answers, as push's `worst`.
  - Register it in `commands/mod.rs`, in `Command` and its dispatch in `crates/timber/src/lib.rs`, and in `main.rs` as `For::Prune`. Its module comment cites 145 and says timber never starts it (core.52).
  - Tests in a new `crates/timber/tests/prune.rs`, over a scratch repository whose branch's landing is the git branch, a host's file naming a layout, and a policy file writing `keep_days`: `prune_removes_an_entry_past_its_time_and_keeps_one_within`, `prune_prints_its_answer_as_json`, `prune_of_a_revision_not_landed_is_refused`, `prune_exits_unreadable_where_an_entry_is`.
  - Verify with `devenv shell -- cargo test -p timber --test prune`.

## 6. The suite

- [ ] 6.1 Read the statement through 145, in `crates/timber-conformance/src/statement.rs` (clause-coverage spec, "The suite knows the clauses of Parts A and B"). `READ_AT` names `boomdrivedev/blueprints fecf4d9`, A.9 lists 145 after 54, and the test becomes `the_parts_hold_every_clause_from_1_to_145_once_in_20_sections`. Nothing goes in the register: tasks 6.2 and 6.3 cite 145 in this same commit, so commit 6.1 to 6.3 together. Verify with `devenv shell -- cargo test -p timber-conformance --lib statement`.
- [ ] 6.2 Add the store group's cache scenarios below the line of `crates/timber-conformance/src/seams/store.rs` (design D9; conformance-suite spec, "The store's group lists and removes cache entries").
  - "a cache entry listed with when it was pushed", citing 145 and 52; "a cache entry removed, its bytes' other names left", citing 145, 54 and 100; "a cache entry's removal from where it is not", citing 145; "a removal of no cache entry", citing 145. Each makes its entries through `cache_name`.
  - Bind each registry rig's store for the build profile `moon`, `bound_for("moon", false)`: the stub's rig in the registry's tests, `RegistryRig` and `LiveRig` in `crates/timber/tests/registry/mod.rs`, and the rigs of `crates/timber-conformance/tests/`. The layout's and the fake's rigs need nothing more.
  - Verify with `devenv shell -- cargo test -p timber-conformance --test fakes the_fake_store_keeps_the_stores_contract`, `devenv shell -- cargo test -p timber --test layout_store` and `devenv shell -- cargo test -p timber --lib profiles::store::registry::tests`, where the group passes over the fake, the layout and the stub.
- [ ] 6.3 Add prune's scenarios below the line of `crates/timber-conformance/src/core/cache.rs` (design D9; conformance-suite spec, "Prune over the fakes"): "an entry past its keeping time removed, one within kept", citing 145; "an entry with no push moment removed", citing 145 and 54; "the keeping time read at the landed revision", citing 145 and 22; "a released version read back whole after prune", citing 145, 54, 22 and 34. Verify with `devenv shell -- cargo test -p timber-conformance --test fakes the_core_keeps_the_cache_over_the_fakes`, and with `devenv shell -- cargo run -q -p timber-conformance --bin coverage`, which passes naming 145 among the clauses cited and not as pending.

## 7. The record, the rule and the pages

- [ ] 7.1 Write record 0045, or the next number free on main, on `origin/bolt/timber-publishes-github-releases-and-to-cachix` and on the bolt's other changes (`change/timber-publishes-a-github-release` takes 0043 and `change/timber-pushes-to-cachix` 0044), in `docs/adr/`, in MADR form as `adrs.toml` sets and record 0042 is laid out, status proposed (design D10). It holds design D1, D2 and D4 to D7, each with its path not taken, as this change's construct stage chose them on 145. Verify with `devenv shell -- treefmt --ci docs/adr`.
- [ ] 7.2 Add rule core.80, or the next number free on main and the bolt's changes (`change/timber-publishes-a-github-release` takes core.79), to `docs/architecture/core.md` under "Answers and records", with its details as design D10 writes them, citing 145, 54, 22, 34, 100, C.1, records 0039, 0040, 0042 and the record of 7.1. Verify with `devenv shell -- treefmt --ci docs/architecture`.
- [ ] 7.3 In `site/src/content/docs/guides/integrations/stores.mdx`, say how cache entries are kept and removed: the keeping time the policy gives, thirty days unless `[cache] keep_days` says otherwise; the cache manifest's name and moment; each member's removal, the distribution protocol's delete and GitHub's API on GitHub's registry; that an entry pushed before this change is removed at the first prune; and that the bytes are the registry's to collect, and a layout's stay. Name the record of 7.1 beside 0040 to 0042 (design D2, D4 to D6). Verify with `devenv shell -- moon run site:build`.
- [ ] 7.4 In `site/src/content/docs/guides/self-hosting.mdx`, add "### Removing old cache entries" under "After a landing" (design D1, D5, D7): the `[cache] keep_days` line; that timber starts nothing, so a scheduled workflow of the repository's own runs `timber prune` with a full checkout of the branch and the landing's records, as a publish workflow has; on ECR, an IAM policy allowing `ecr:BatchGetImage`, `ecr:ListImages` and `ecr:BatchDeleteImage` on `<root>/<owner>/<name>/cache/*` and `ecr:GetAuthorizationToken` on `*`, and nothing else; on GitHub's registry, the workflow's `permissions: packages: write`, `docker login ghcr.io` with the job's token, and the repository given the admin role on its cache packages alone. Under "On GitHub's registry", say the admin role on the cache packages lets the pool's push token remove cache entries too, and that untagged versions are still to be left alone. Verify with `devenv shell -- moon run site:build`.

## 8. The proof on registries

- [ ] 8.1 Prune on zot and the reference distribution in the tools layer (design D9; registry-store spec, "Removal passes against real registries").
  - Start the reference distribution in `crates/timber/tests/registry/mod.rs` with `storage: delete: enabled: true`, saying why in a comment.
  - Add `a_prune_removes_an_entry_past_its_time_and_leaves_a_released_version` to `on_both!` in `crates/timber/tests/registry_store.rs`: under `acme/site`, a store bound for `moon` with a fixed clock writes one entry forty days ago and one a day ago; a version is published, a claim attached and a channel's name made through a store bound for the same repository; `prune` over that store and the fake landing and policy removes the first entry alone, and the version, its claim and the channel's name read as before.
  - Verify with `devenv shell -- cargo test -p timber --features tools --test registry_store a_prune_removes`, where it passes on zot and on the reference distribution, and `devenv shell -- cargo test -p timber --features tools --test registry_store keeps_the_stores_contract`, where the store group's cache scenarios pass on both.
- [ ] 8.2 Run the GHCR check by hand once (design D9; rule core.38). It needs the place's `devenv.local.nix` to set `env.TIMBER_TOOLS_GHCR` to a root under boomdrivedev, such as `ghcr.io/boomdrivedev/timber-tools-check`, and the host's Docker login for `ghcr.io` to hold a token that may delete the versions of the check's cache package, a classic token with `delete:packages` or one whose account administers the package. Where either is missing, tick nothing and stop short with `flume needs --to ops`, naming both and saying the run is the change's acceptance on GitHub's registry. Where GitHub refuses anything, fix it in the profile with a unit test over the stub holding GitHub's answer verbatim, and run again. Keep the date and the run's start and end in UTC for 8.4. Verify with one passing run of `devenv shell -- moon run ':#check-ghcr' --force`.
- [ ] 8.3 Run the ECR check by hand once, since the cache manifest's form changed (design D9; rule core.38). It needs `env.TIMBER_TOOLS_ECR` in the place's `devenv.local.nix` and a live AWS session for the profile it names. Where the session has lapsed, tick nothing and stop short with `flume needs --to ops`, asking the operator to run `aws sso login --sso-session willdan`; where the setting is missing, name it. Where ECR refuses the delete, fix it in the profile with a unit test over the stub holding ECR's answer verbatim, and run again. Keep the date and the run's times in UTC for 8.4. Verify with one passing run of `devenv shell -- moon run ':#check-ecr' --force`.
- [ ] 8.4 In `site/src/content/docs/guides/integrations/stores.mdx`, say GitHub's registry's and ECR's rows were proven removing cache entries by hand on 8.2's and 8.3's dates, and add those dates and the runs' times in UTC to the record of 7.1. Verify with `devenv shell -- moon run site:build` and `devenv shell -- treefmt --ci docs/adr`.

## 9. The specs

- [ ] 9.1 Archive the change once groups 1 to 8 are done: `devenv shell -- openspec archive timber-removes-old-cache-entries --yes`. Then commit the archived change and the ten specs it writes. Verify with `devenv shell -- openspec validate cache-keeping --type spec --strict`.

## Workflow follow-up

- The design folds this change's lines into the statement, once it lands: C.1's table gains the removal of a cache entry, the cache manifest's two annotations and GitHub's API on GitHub's registry; C.10 lists `[cache] keep_days`, given thirty days (record of 7.1).
- Each repository on the family's store adds the scheduled prune workflow, and an administrator gives each repository the admin role on its cache packages, as the guide says.
