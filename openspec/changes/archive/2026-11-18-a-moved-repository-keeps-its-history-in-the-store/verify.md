# Verify: a-moved-repository-keeps-its-history-in-the-store

The verify stage's report on the branch
`change/a-moved-repository-keeps-its-history-in-the-store` at 08d9b14a,
cut from `bolt/timber-follows-a-moved-repository` (merge base
010d83d8). The code stage archived the change as its own task 6.1
(08d9b14a), so this report sits in the archived change's directory and
lands with it.

## The whole checks

Both layers the repository's instructions name, run once at 08d9b14a
inside `devenv shell`, nothing fixed between them:

| layer | command | outcome |
|---|---|---|
| `check-fast` | `moon run ':#check-fast'` | passed: 28 tasks, 14 of them answered from moon's cache at hashes this tree gives, in 2 m 2 s |
| `check-tools` | `moon run ':#check-tools'` | passed: `cli:tools` run, not cached, 105 tests passed and none failed or ignored, in 6 m 27 s |

`check-fast` ran `seams:test`, `fakes:test`, `core:test`,
`conformance:test`, `conformance:coverage`, `cli:test`, `root:format`,
`root:lint`, `root:hooks`, `root:toolchain`, `root:compile-server`,
`nix:typecheck`, `site:install` and `site:toolchain`, and took
`root:specs`, `root:changes`, `root:changes-cases`, `root:versions`,
`root:release`, `root:marketplace`, `site:build`, `nix:lint`, `nix:test`
and the plugins' tasks from moon's cache. Every test result it printed
was `ok`, none failed.

The GHCR check is outside the gate (core.38) and was not run again; its
one run by hand is task 5.3's, below.

## Tasks

Every task of `tasks.md`, 1.1 to 6.1, is ticked, and each has the work
it names on the branch:

- Every test the tasks name, 58 of them, is in the module or test file
  the task names it in.
- 1.2: every place that builds a `Seams` hands it roots: the seams
  crate's own test `NoRoots`, the fakes `FakeRoots` (none bound by
  default, which answers as `NoRoots` does), the host `HostRoots`; the
  tests and the suite that build a `Seams` take it from
  `Fakes::seams()`.
- 2.6, 2.7: record 0042 is in `docs/adr/`, status proposed, holding D1
  to D5 and D7 with their paths not taken, and the code stage's own
  choices under "What the code stage chose", among them D2's `Left`
  named `RootLeft` since the build seam's `Left` holds the name. Rule
  core.78 is in `docs/architecture/core.md` under "Acceptance and
  channels".
- 3.4, 3.5: the stores page says what a registry holds after a move,
  naming record 0042 beside 0040 and 0041; the self-hosting guide has
  "When a repository moves" and the paragraph on GitHub's registry
  after a move.
- 5.3, 5.4: the GHCR check ran by hand once, by boomdrive/ops at
  6f9e46ed under `ghcr.io/boomdrivedev/timber-tools-check` (card
  `req_8897c0801d834cab9f78`), on 2026-11-18 from 16:47:30 to 16:49:23
  UTC, `cli:tools-ghcr` passing its five tests, the carry among them
  (81b531f8). Record 0042 and the stores page's GitHub row carry the
  date and times (8ad923e8).
- 6.1: `openspec archive` wrote the five specs (08d9b14a), and
  `root:specs` and `root:changes` passed in `check-fast`.

## Requirements and scenarios

Each requirement of the change's five delta specs, with the code that
meets it and the tests that hold each scenario.

### seam-contracts

- **The binding answers the roots.** `Seams::roots: Box<dyn Roots>`,
  `Roots::{bound, left}` and `NoRoots` in `timber-seams/src/bind.rs`.
  "No roots": `no_roots_answers_no_root_and_no_root_left`.
- **A root moved is a refusal of its own.** `Why::RootMoved { from, to }`
  in `answer.rs`, reading "`<from>`'s history moved to `<to>`". "The
  words of a root moved": `a_root_moved_reads_where_the_history_went`,
  `a_root_moved_round_trips_through_json`.
- **The mark of a root left.** `moved_name()` and `MovedTo` in
  `root.rs`, the bytes the canonical JSON of the root lowercased. "The
  mark's bytes": `the_marks_bytes_name_the_root_lowercased_and_read_back`,
  `bytes_that_are_no_mark_read_as_none`.

### seam-fakes

- **The fake roots bind a store and a keeper per root.** `FakeRoots` in
  `timber-fakes/src/roots.rs`, each call recorded, a root left handed
  out as clones of the test's store and keeper. "A root left asked":
  `a_root_left_is_read_through_the_store_the_test_holds`, with
  `fake_roots_answer_the_root_a_test_binds` and
  `a_root_left_never_bound_is_absent`.

### host-bindings

- **Every binding answers the root its store is bound for.**
  `HostRoots::bound` in `bindings/mod.rs`: the checkout's repository
  answer for a registry, its absent or unreadable answer among them,
  none for a layout. "A registry's root":
  `a_registrys_binding_answers_the_checkouts_root`; "A layout's root":
  `a_layouts_binding_answers_no_root`; with
  `a_binding_with_no_landing_remote_answers_its_root_absent`.
- **A root left is bound with the host's store and ledger.**
  `HostRoots::left`, `profiles::store::left` (the same registry and
  root, `carrying_to` the bound repository; absent for a layout) and
  `profiles::ledger::left` (the store ledger kept under
  `timber/left/<owner>/<name>`, the git ref ledger as bound). "With the
  store ledger": `a_root_lefts_store_ledger_keeps_its_chains_apart`,
  `a_root_left_is_bound_under_the_same_registry_and_root`; "with the git
  ref ledger": `a_root_lefts_git_ref_ledger_reads_the_repositorys_refs`.

### publishing

- **A publish reads its root before anything is built.** `publish`
  calls `carry::root` after `Landed::check` and `Declared::read` and
  before any component is derived; `carry::root` returns at once where
  no root is bound, records the bound root on an empty record, and
  answers `None` where the last root, lowercased, is the bound one. The
  record is `History::roots` and `History::record_root` over
  `refs/timber/root`, fetched into a copy that only moves forward and
  pushed without force. "A first root recorded":
  `an_empty_record_records_the_bound_root`,
  `a_publish_touching_no_component_still_records_its_root`; "The same
  root": `the_same_root_in_another_case_carries_nothing`; "A layout":
  `no_root_bound_reads_nothing_and_writes_nothing`,
  `a_publish_to_a_store_with_no_roots_answers_nothing_of_a_root`; the
  record's own nine `the_root_record_*` tests.
- **A moved repository's first publish carries its history.**
  `carry::history` marks the root left first, copies each version by
  `copy::copy`, makes each channel's name at its head's index, and
  `carry::root` records the bound root only after it answers. "A move
  carried": `a_moved_repository_is_carried_and_its_root_recorded_last`,
  `a_moved_repositorys_publish_carries_before_it_claims`, and the
  suite's "a moved repository's first publish carries its history"; "A
  version published before the move promoted after it": the suite's "a
  version published before a move is promoted after it"; "Cache entries
  stay": `a_carry_reads_and_writes_nothing_of_the_cache`.
- **Each carried chain goes on from its head.** `carry_channel` reads
  the chain through `repair::from_source`, refusing one answered as last
  read, and checks the bound chain holds the head left, refusing a name
  moved elsewhere as `Why::Fork` naming both heads. "A chain as last
  read": `a_carry_of_a_chain_as_last_read_is_unreadable`,
  `a_carry_refused_leaves_the_record_at_the_root_left`; "A channel moved
  on after an earlier carry":
  `a_carry_leaves_a_channel_moved_on_since_where_it_is`; "Two heads":
  `a_carry_finding_two_heads_is_refused_as_a_fork`.
- **A carry writes only what is missing.** The mark, each copy and each
  channel's name answer held already where they are. "A carry again":
  `a_carry_again_writes_nothing`.
- **A publish under a root left is refused.** `carry::root` resolves
  `moved` in the bound store before reading the record, and refuses
  `RootMoved` naming the bound root and where it went. "A stale
  address": `a_bound_root_holding_a_mark_is_refused_naming_where_it_went`,
  `a_publish_whose_carry_is_refused_claims_and_offers_nothing`, and the
  suite's "a publish under a root left is refused".
- **A publish answers the root it recorded or carried.**
  `Publish::rooted`, serialized as `rooted` and left out where `None`;
  `Carried` names both roots, each version's `copied`, `held_already` or
  `not_held`, and each channel's head and name; the command prints it
  before the components. "A carry in the answer":
  `a_moved_repository_is_carried_and_its_root_recorded_last`,
  `the_text_says_the_history_carried`, `the_text_says_the_root_recorded`.

### registry-store

- **A root left answers where it went.** `Registry::own` reads the mark
  once in the store's life (`unmarked`, `read_mark`) and refuses every
  operation of the repository's own, its components' and its cache
  repositories `RootMoved`; the artifact space is read as before. "A
  channel read at the old root":
  `a_root_marked_moved_refuses_every_operation_of_its_own_spaces_naming_where_it_went`,
  `a_root_marked_moved_writes_nothing`; "A pin read at the old root":
  `a_root_marked_moved_still_reads_an_artifacts_name_and_its_claims`;
  "The mark read once": `the_mark_is_read_once_in_a_stores_life`; with
  `a_mark_that_cannot_be_read_is_unreadable_naming_it` and
  `the_mark_written_is_a_tag_of_the_repositorys_own_through_a_cache_manifest`.
- **A pool's push reads no mark.** `own` skips the mark for a push's
  store. "A push at the old root":
  `a_pushs_store_at_a_root_marked_moved_writes_its_cache_and_reads_no_mark`.
- **A carry's store reads past the mark naming where it carries.**
  `Registry::carrying_to`. "Reading the root left":
  `a_store_carrying_to_the_root_a_mark_names_reads_and_writes_past_it`;
  "A mark naming a third root":
  `a_store_carrying_elsewhere_than_the_mark_names_is_refused`.
- **A carry between two roots on real registries.**
  `a_moved_repositorys_history_is_carried_to_its_new_root` in
  `on_both!`, over `registry::carried_between`, in `cli:tools`.
  "A carry on zot and the reference distribution": passed in this
  run's `check-tools`, `zot::` and `distribution::` alike.
- **A carry on GitHub's registry.**
  `ghcr_carries_a_moved_repositorys_history_to_its_new_root` over the
  same helper. "The check's carry": the run by hand of task 5.3, which
  passed.

## Verdict

The work holds. Both layers pass, every task is ticked and done, and
every requirement and scenario of the five delta specs is met by code
and held by a test or the suite. The change stands archived, as 6.1
archived it, with this report beside it. No requirement was found
wanting and nothing goes back to the code stage.
