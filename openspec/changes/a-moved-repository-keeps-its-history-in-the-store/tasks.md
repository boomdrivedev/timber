# Tasks

Every command runs from the place's root, inside `devenv shell`.
Commit by pathspec, Conventional Commits scoped to the part of the
tree, citing clauses by number, rules by id and this change's design
by D-number. Submit nothing: Flume submits the place once its last
stage delivers. The headings are for reading; each task's check is its
own.

Read these before starting:

- This change's `design.md`, whole.
- `crates/timber-seams/src/bind.rs`, `answer.rs` (`Why`) and
  `keeper.rs` (`channel_name`).
- `crates/timber-core/src/history.rs` (`tags`, `remote`, `push_tag`,
  `remote_tags`), `version.rs` (`claim`), `copy.rs`, `ledger.rs`
  (`mirror`, `channel`), `repair.rs` (`head`, `follow`) and
  `publish.rs` (`publish`, `Run::component`).
- `crates/timber/src/profiles/store/registry/mod.rs` (`for_repository`,
  `own`, `place`, `holders_of_index`) and `crates/timber/src/profiles/ledger/`.
- `bind` and `repository_named` in `crates/timber/src/bindings/mod.rs`.
- Statement C.1 as amended at e8700c8, on `boomdrivedev/blueprints`
  `design/timber-a-moved-repository-keeps-its-history`, and clauses 22,
  62, 103 and 105.
- Record 0040 as amended on 2026-11-18, and record 0041.

The OCI forms the registry profile writes, the layout store and every
digest stay as they are. A task that cannot be done without changing
one stops short with `flume needs`.

## 1. The seams and the binding's roots

- [x] 1.1 Add the types of a root to `crates/timber-seams` (design D2, D4; seam-contracts spec, "The binding answers the roots", "A root moved is a refusal of its own", "The mark of a root left").
  - `RepositoryName::lowercased(&self) -> RepositoryName`, the owner and name each lowercased, in `artifact.rs`.
  - `Why::RootMoved { from: RepositoryName, to: RepositoryName }` in `answer.rs`, citing C.1, reading "`<from>`'s history moved to `<to>`", in every match over `Why` the crate holds.
  - In a new `root.rs`, exported from `lib.rs`: `moved_name() -> Name`, `moved`, whose comment says why no version's, channel's, cache entry's or artifact's name can be it (core.48); and `MovedTo { to: RepositoryName }` with `to_bytes(&self) -> Vec<u8>`, the canonical JSON `{"moved_to":"<owner>/<name>"}` of the root lowercased, and `of_bytes(&[u8]) -> Option<MovedTo>`.
  - In `bind.rs`: the trait `Roots` and the struct `Left` as D2 writes them, and `NoRoots`, which answers `bound` with `Ok(None)` and `left` absent naming the root asked.
  - Tests in the modules they test: `a_root_moved_reads_where_the_history_went`, `a_root_moved_round_trips_through_json`, `the_marks_bytes_name_the_root_lowercased_and_read_back`, `bytes_that_are_no_mark_read_as_none`, `no_roots_answers_no_root_and_no_root_left`.
  - Verify with `devenv shell -- cargo test -p timber-seams`, where the new tests and every other test of the crate pass.
- [x] 1.2 Hand the roots to the core (design D2; seam-contracts spec, "The binding answers the roots").
  - `Seams` gains `pub roots: Box<dyn Roots>`, its comment citing 92 and C.1.
  - Every place that builds a `Seams` hands it `Box::new(NoRoots)`: `crates/timber-seams/src/lib.rs`, `crates/timber-fakes/src/lib.rs`, `crates/timber/src/bindings/mod.rs`, `crates/timber/tests/moon_cache.rs`, `crates/timber/tests/landings_end_to_end.rs`, `crates/timber-core/tests/gate.rs`, and `crates/timber-conformance/src/fakes.rs`, `world.rs` and `core/after.rs`. Tasks 1.3 and 3.3 replace the fakes' and the host's.
  - Verify with `devenv shell -- cargo check --workspace --all-targets --all-features`, which builds every place that builds a `Seams`.
- [x] 1.3 Add `FakeRoots` to `crates/timber-fakes` (design D2; seam-fakes spec, "The fake roots bind a store and a keeper per root").
  - It holds the root it answers bound, none by default, and a fake store and a fake keeper per root left a test binds, handed out as clones so the test's handles see every asking. `left` answers absent for a root it binds none for. Every call is recorded, as the other fakes record theirs.
  - `Fakes` holds one, and `Fakes::seams()` hands a clone of it.
  - Tests in its module: `fake_roots_answer_the_root_a_test_binds`, `a_root_left_is_read_through_the_store_the_test_holds`, `a_root_left_never_bound_is_absent`.
  - Verify with `devenv shell -- cargo test -p timber-fakes`, where the new tests and every other test of the crate pass.

## 2. The record and the carry

- [x] 2.1 Read and write the record's roots in `crates/timber-core/src/history.rs` (design D1; publishing spec, "A publish reads its root before anything is built").
  - `History::roots(&self, landed: &Landed) -> Answer<Vec<RepositoryName>>` and `History::record_root(&self, landed: &Landed, root: &RepositoryName) -> Answer<()>`, as D1 says: the ref `refs/timber/root`, a chain of commits each holding the file `root`, `<owner>/<name>` lowercased and a newline, by `timber <timber@localhost>`; read from the remote the branch names, fetched into the repository's copy, which only moves forward; pushed without force and the copy moved after; a push refused read again and done where the last root is now `root`, refused with git's words otherwise; the own ref alone where the branch names no remote. Run every remote operation by the git command, as `push_tag` does, so the host's own credential helper answers (core.21).
  - Tests in `crates/timber-core/tests/history.rs`, each over a scratch repository and, where it says so, a bare remote: `the_root_record_is_empty_where_no_ref_is_held`, `the_root_record_holds_the_first_root_written_and_pushed`, `the_root_record_appends_a_second_root_after_the_first`, `the_root_record_written_lowercases_the_root`, `the_root_record_read_again_after_a_push_refused_is_done_where_it_names_the_root`, `the_root_record_refused_where_another_root_was_written_first_names_gits_words`, `the_root_record_is_unreadable_where_the_remote_holds_less_than_the_copy`, `the_root_record_is_unreadable_where_the_remote_cannot_be_reached`, `the_root_record_of_a_branch_with_no_remote_is_the_own_ref`.
  - Verify with `devenv shell -- cargo test -p timber-core --test history the_root_record`.
- [x] 2.2 Carry a history between two roots in a new `crates/timber-core/src/carry.rs`, exported from `lib.rs` (design D4, D5; publishing spec, "A moved repository's first publish carries its history", "Each carried chain goes on from its head", "A carry writes only what is missing").
  - `carry::history`, as D5 says: the mark first, through the root left's put-bytes and name; each version by `copy::copy`; each channel's chain read from the root left's keeper through `repair::head`'s refusal of a chain as last read, its name made in the bound store from nothing, a name elsewhere accepted only where the bound keeper's chain holds the head left and refused `Why::Fork` naming both heads otherwise, and the bound keeper's chain checked last. It reads no git and no policy.
  - `Carried`, serializable, says both roots, each version's name with `copied`, `held_already` or `not_held`, and each channel with its head and `made` or `held_already`.
  - Tests in a new `crates/timber-core/tests/carry.rs` over the fakes: `a_carry_marks_the_root_left_before_it_copies_anything`, `a_carry_copies_each_version_with_every_claim_on_it`, `a_carry_passes_over_a_version_the_root_left_does_not_hold`, `a_carry_makes_each_channels_name_at_its_heads_index`, `a_carry_passes_over_a_channel_with_no_chain`, `a_carry_of_a_chain_as_last_read_is_unreadable`, `a_carry_again_writes_nothing`, `a_carry_leaves_a_channel_moved_on_since_where_it_is`, `a_carry_finding_two_heads_is_refused_as_a_fork`, `a_carry_finding_a_mark_naming_another_root_is_refused_naming_it`, `a_carry_reads_and_writes_nothing_of_the_cache`.
  - Verify with `devenv shell -- cargo test -p timber-core --test carry`.
- [x] 2.3 Read a move and record the root, in `carry.rs` (design D3, D5, D7; publishing spec, "A publish reads its root before anything is built", "A publish under a root left is refused").
  - `carry::root(seams, history, landed, declared) -> Answer<Option<Rooted>>`, with `Rooted::Recorded { root }` and `Rooted::Carried(Carried)`, serializable: `seams.roots.bound()` first, none ending it with nothing read; the bound root's mark checked by resolving `moved_name()` in the bound store, a mark found read and refused `Why::RootMoved` naming the bound root and where it went; then `History::roots`; an empty record recording the bound root; the same root, compared lowercased, answering `None`; another binding the root left, listing the versions from each declared component's tags the landed revision reaches and the channels from the policy's channels in force and its environments' channels, calling `carry::history`, and recording the bound root after it answers.
  - Tests in `crates/timber-core/tests/carry.rs`: `no_root_bound_reads_nothing_and_writes_nothing`, `an_empty_record_records_the_bound_root`, `the_same_root_in_another_case_carries_nothing`, `a_moved_repository_is_carried_and_its_root_recorded_last`, `a_carry_refused_leaves_the_record_at_the_root_left`, `a_bound_root_holding_a_mark_is_refused_naming_where_it_went`, `a_carry_lists_every_declared_components_versions_and_every_channel_once`.
  - Verify with `devenv shell -- cargo test -p timber-core --test carry`.
- [x] 2.4 Carry before a publish builds anything, in `crates/timber-core/src/publish.rs` (design D3, D6; publishing spec, "A moved repository's first publish carries its history", "A publish answers the root it recorded or carried").
  - `publish` calls `carry::root` after `Declared::read` and before any component is derived, and its answer is the publish's where it is not an answer.
  - `Publish` gains `rooted: Option<Rooted>`, serialized as `rooted` and left out where `None`. Every place that builds a `Publish` sets it.
  - Tests in `crates/timber-core/tests/publish.rs`: `a_moved_repositorys_publish_carries_before_it_claims`, `a_publish_whose_carry_is_refused_claims_and_offers_nothing`, `a_publish_touching_no_component_still_records_its_root`, `a_publish_to_a_store_with_no_roots_answers_nothing_of_a_root`.
  - Verify with `devenv shell -- cargo test -p timber-core --test publish`.
- [x] 2.5 Print the root in the publish command's text, in `crates/timber/src/commands/publish.rs` (design D6). Before the components it prints `recorded the root <owner>/<name>`, or `carried <from>'s history to <to>: <n> versions, <m> channels` with a line per version passed over and per channel made. Its JSON carries `rooted` as the core serializes it. Add a test module with `the_text_says_the_root_recorded` and `the_text_says_the_history_carried`. Verify with `devenv shell -- cargo test -p timber --lib commands::publish::tests`.
- [x] 2.6 Write record 0042, or the next number free on main and on this branch, in `docs/adr/`, in MADR form as `adrs.toml` sets and as record 0041 is laid out, status proposed (design D9). It holds design D1 to D5 and D7, each with its path not taken, as the construct stage of this change chose them on C.1 as amended at e8700c8 and record 0040 as amended on 2026-11-18. Verify with `devenv shell -- treefmt --ci docs/adr`.
- [x] 2.7 Add rule core.78 to `docs/architecture/core.md` under "Acceptance and channels", with its details, as design D9 writes them, citing C.1 as amended at e8700c8, 22, 62, 103, 105, record 0040 as amended on 2026-11-18 and record 0042. Verify with `devenv shell -- treefmt --ci docs/architecture`.

## 3. The registry and the host's bindings

- [x] 3.1 Answer a root left where it went, in `crates/timber/src/profiles/store/registry/mod.rs` (design D4; registry-store spec, "A root left answers where it went", "A carry's store reads past the mark naming where it carries").
  - `Registry::carrying_to(self, to: RepositoryName) -> Registry`, a store that reads past a mark naming `to`.
  - At its first operation reaching the repository's own, its components' or its cache repositories, the store resolves `moved` in its own repository, once in its life. A mark naming a root other than the one it carries to makes every such operation `Refused(Why::RootMoved)` naming the repository it is bound for and the root the mark names, with nothing written. A tag absent is no mark, and one it cannot read is unreadable naming it. The artifact space is read as before. A push's store, which asks nothing of the repository's own (135, core.62), resolves no mark and writes its cache entries at a root left as at any root, as boomdrive-design ruled on the code stage's question (record 0042).
  - Naming `moved` at a digest held as bytes writes it through the cache manifest, as any name at bytes is.
  - Say so in the module's comment and in `Store for Registry`'s, citing C.1 as amended at e8700c8.
  - Tests in its module over the stub: `a_root_marked_moved_refuses_every_operation_of_its_own_spaces_naming_where_it_went`, `a_root_marked_moved_still_reads_an_artifacts_name_and_its_claims`, `a_root_marked_moved_writes_nothing`, `the_mark_is_read_once_in_a_stores_life`, `a_store_carrying_to_the_root_a_mark_names_reads_and_writes_past_it`, `a_store_carrying_elsewhere_than_the_mark_names_is_refused`, `the_mark_written_is_a_tag_of_the_repositorys_own_through_a_cache_manifest`, `a_mark_that_cannot_be_read_is_unreadable_naming_it`, `a_pushs_store_at_a_root_marked_moved_writes_its_cache_and_reads_no_mark`.
  - Verify with `devenv shell -- cargo test -p timber --lib profiles::store::registry::tests`, where the new tests and every other test of the module pass.
- [x] 3.2 Bind a root left's store and ledger (design D2; host-bindings spec, "A root left is bound with the host's store and ledger").
  - In `crates/timber/src/profiles/store/mod.rs`: `left(table, from, to) -> Answer<Box<dyn Store>>`, a registry under the same registry and root bound for `from` and carrying to `to`; absent for a layout, naming it, since a layout places no name by repository.
  - In `crates/timber/src/profiles/ledger/mod.rs`: `left(table, repository, open, from)`, binding the ledger `[ledger]` names, or the given, over the root left: the git ref as `bind` and `given` bind it, and the store ledger over `open`'s store, kept under `timber/left/<owner>/<name>` in the common git directory.
  - Tests in a new test module of `profiles/ledger/mod.rs`: `a_root_lefts_store_ledger_keeps_its_chains_apart`, `a_root_lefts_git_ref_ledger_reads_the_repositorys_refs`.
  - Verify with `devenv shell -- cargo test -p timber --lib profiles::ledger::tests`.
- [x] 3.3 Bind the host's roots, in `crates/timber/src/bindings/mod.rs` (design D2; host-bindings spec, "Every binding answers the root its store is bound for", "A root left is bound with the host's store and ledger").
  - A roots binding holding the store's table, the checkout's repository answer and what binds the ledger: `bound` answers that repository for a registry and none for a layout; `left(from)` binds 3.2's store and ledger for `from` carrying to the bound repository, with the store's source. Every command's `Seams` gets it in place of `NoRoots`.
  - Say so in `bind`'s comment.
  - Tests in `crates/timber/src/bindings/tests.rs`: `a_registrys_binding_answers_the_checkouts_root`, `a_layouts_binding_answers_no_root`, `a_binding_with_no_landing_remote_answers_its_root_absent`, `a_root_left_is_bound_under_the_same_registry_and_root`.
  - Verify with `devenv shell -- cargo test -p timber --lib bindings::tests`, where the new tests and every other test of the module pass.
- [x] 3.4 In `site/src/content/docs/guides/integrations/stores.mdx`, say what a registry holds after a repository moves: the record names each root, the first publish after a move marks the old root, carries the versions, their claims and the channels, and records the new root; the old root answers where it went, its artifact names still read; cache entries are not carried; a root left is never written again, and a publish naming it is refused. Name record 0042 beside 0040 and 0041 (design D3 to D5, D7). Verify with `devenv shell -- moon run site:build`.
- [x] 3.5 In `site/src/content/docs/guides/self-hosting.mdx`, add "When a repository moves" under "A registry's repositories": what the first publish after a move does, that a checkout whose remote still names the old address is refused until its remote is set, that a move back is refused, and that the first publish may be refused for its cache until the next run fills it. Under "On GitHub's registry", say that the carry makes the new root's packages private, that an administrator sets each as its repository is, and that the job's repository is given write on the new root's cache package alone (design D7; record 0040). Verify with `devenv shell -- moon run site:build`.

## 4. The suite

- [x] 4.1 Give the conformance world a second root, in `crates/timber-conformance/src/world.rs` (design D8). The world binds `FakeRoots`, none bound by default. Add a way to move the world's repository: the roots bound to a new root, the world's store and keeper bound as the root left, and a fresh fake store and keeper as the bound ones, with handles to both. Verify with `devenv shell -- cargo test -p timber-conformance --lib`.
- [x] 4.2 Add the scenarios below the line of `crates/timber-conformance/src/core/publish.rs` and `core/ledger.rs` (design D8):
  - publish group, "a moved repository's first publish carries its history", citing 22, 62, 103 and 105: a version published, accepted and promoted at the first root, the repository moved, and the next publish answers the history carried, the new root holding the version with its claims and the channel's name at its head, the old root holding the mark, and the record naming the new root;
  - publish group, "a publish under a root left is refused", citing 22 and 58: the bound root holds a mark, and the publish is refused naming where it went with nothing claimed;
  - ledger group, "a version published before a move is promoted after it", citing 35 and 105: after the carry, the gate promotes the version published before the move, and the promotion follows the head left.
  - Verify with `devenv shell -- cargo test -p timber-conformance --test fakes the_core_publishes_over_the_fakes` and `devenv shell -- cargo test -p timber-conformance --test fakes the_core_follows_the_ledger_over_the_fakes`.

## 5. The proof on registries

- [x] 5.1 Carry between two roots in the tools layer (design D8; registry-store spec, "A carry between two roots on real registries"). Add `a_moved_repositorys_history_is_carried_to_its_new_root` to `on_both!` in `crates/timber/tests/registry_store.rs`. Under `acme/app`, it publishes a version, attaches an acceptance and appends a promotion through the store ledger over that store. It then runs `carry::history` from a store for `acme/app` carrying to `acme/site` into a store for `acme/site`, each with the store ledger over it. It asserts that the version resolves under `acme/site` with every claim, the channel's chain walks to the same head there, a store for `acme/app` is refused `RootMoved` naming `acme/site` for the channel, and that store still reads the artifact's name. Verify with `devenv shell -- cargo test -p timber --features tools --test registry_store a_moved_repositorys_history`, where it passes on zot and on the reference distribution.
- [x] 5.2 Add the GHCR check's carry (design D8; registry-store spec, "A carry on GitHub's registry"). Add `ghcr_carries_a_moved_repositorys_history_to_its_new_root` to `crates/timber/tests/registry_ghcr.rs`, carrying as 5.1 does between `timber-checks/carried-from` and `timber-checks/carried-to` under the root the host names, every version and channel under the run's prefix, and update the module's comment. Verify with `devenv shell -- cargo test -p timber --features tools --test registry_ghcr --no-run`, which compiles it, and with `devenv shell -- env -u TIMBER_TOOLS_GHCR cargo test -p timber --features tools --test registry_ghcr ghcr_carries`, where it fails naming `env.TIMBER_TOOLS_GHCR`.
- [x] 5.3 Run the GHCR check by hand once (design D8). It needs the place's `devenv.local.nix` to set `env.TIMBER_TOOLS_GHCR` to a root under boomdrivedev, such as `ghcr.io/boomdrivedev/timber-tools-check`, and the host's Docker configuration to hold a credential for `ghcr.io` that writes the organization's packages. Where either is missing, tick nothing and stop short with `flume needs --to ops`, naming both and saying the run is the change's acceptance on GitHub's registry. Where GitHub's registry refuses anything, fix it in the profile for every registry with a unit test over the stub holding its answer verbatim, and run again. Keep the date and the run's start and end in UTC for 5.4. Verify with one passing run of `devenv shell -- moon run ':#check-ghcr' --force`.
- [ ] 5.4 In `site/src/content/docs/guides/integrations/stores.mdx`, say GitHub's registry's row was proven carrying a moved repository's history by hand on 5.3's date, and add that date and the run's times in UTC to record 0042. Verify with `devenv shell -- moon run site:build` and `devenv shell -- treefmt --ci docs/adr`.

## 6. The archive

- [ ] 6.1 Archive the change once groups 1 to 5 are done: `devenv shell -- openspec archive a-moved-repository-keeps-its-history-in-the-store --yes`. Then commit the archived change and the five specs it writes. Verify with `devenv shell -- moon run root:specs root:changes`, where `root:specs` passes every spec and `root:changes` finds no open change.
