# Verify: timber-publishes-a-github-release

The verify stage's report on the branch
`change/timber-publishes-a-github-release` at af3ed093, cut from
`bolt/timber-publishes-github-releases-and-to-cachix`. The code stage
archived the change as its own task 5.1 (af3ed093), so this report sits
in the archived change's directory and lands with it.

## The whole checks

Both layers the repository's instructions name, run once at af3ed093
inside `devenv shell`, nothing fixed between them:

| layer | command | outcome |
|---|---|---|
| `check-fast` | `moon run ':#check-fast'` | passed: 28 tasks, none answered from moon's cache, in 2 m 55 s |
| `check-tools` | `moon run ':#check-tools'` | passed: `cli:tools` run, not cached, 105 tests passed and none failed or ignored, in 4 m 21 s |

`check-fast` ran `seams:test`, `fakes:test`, `core:test`,
`conformance:test`, `conformance:coverage`, `cli:test`, `root:format`,
`root:lint`, `root:hooks`, `root:toolchain`, `root:compile-server`,
`root:specs`, `root:changes`, `root:changes-cases`, `root:versions`,
`root:release`, `root:marketplace`, `nix:typecheck`, `nix:lint`,
`nix:test`, `site:install`, `site:toolchain`, `site:build` and the
plugins' tasks. Its tests: `seams` 95, `fakes` 94, `core` 541,
`conformance` 70 and `cli` 975 passed, none failed. The one ignored,
`profiles::target::dash0::cli::tests::a_child_runs_every_operation`, is
the Dash0 target's and is run by another test in a child process. The
GitHub release profile's 52 tests all passed.

The proof against GitHub, `check-github-release`, is outside the gate
(rule core.38) and was not run against GitHub here. It needs a throwaway
repository and a token file a person makes. Its run by hand is the
operator's, before the change lands (tasks, "Workflow follow-up").

## OpenSpec

- `openspec instructions apply --change timber-publishes-a-github-release --json`
  and `openspec validate timber-publishes-a-github-release --strict`
  answer "not found" on the branch, since the change is archived. Both
  were run on a scratch copy of `openspec/` at f2e0b2c8, the commit
  before the archive. Validate answered "Change
  'timber-publishes-a-github-release' is valid". Apply answered state
  `ready`, 16 of 17 tasks complete, the one left being 5.1, the archive
  itself. After the archive, `tasks.md` holds all 17 ticked.
- On that scratch copy, `openspec archive timber-publishes-a-github-release --yes`
  wrote the same five specs as af3ed093, byte for byte:
  `github-release-target` (19 requirements added), `conformance-suite`
  (1 added), and `environment-apply`, `host-bindings` and
  `seam-contracts` (1 modified each). The only other difference from the
  branch is task 5.1's tick. So the archive is OpenSpec's own.
- `openspec validate --specs --strict` on the branch: 50 passed, 0
  failed, `github-release-target` among them. That is task 5.1's check.

## Tasks

Every task of `tasks.md`, 1.1 to 5.1, is ticked, and the branch holds
the work each one names:

- 1.1 to 1.3: `Verified::new(index, version, tag)` and `Verified::tag()`
  are in `timber-seams/src/target.rs`. The core makes the tag through
  `apply.rs`'s `tag`, which `tagged` uses too. Every other maker passes
  one. `the_verified_version_names_the_components_tag`,
  `the_fake_records_the_tag_it_is_handed` and the seams round trip
  passed.
- 2.1, 2.2: `run.rs` sets a run's own variables after `env_drop`, tested
  by `a_run_is_handed_variables_of_its_own` and
  `a_variable_set_outlives_a_dropped_prefix`. `file_artifact.rs` has
  `NOTES` and `notes_of`.
- 3.1 to 3.8: `github_release/{mod,table,gh,scripted,tests}.rs`. It is
  bound under `github_release` in `profiles/target/mod.rs`, and
  `an_apply_binds_every_target_built_through_a_store_it_never_makes`
  names `["clan", "dash0", "github_release", "nix_cache"]`. The test
  submodules `credential`, `reads`, `refusals`, `applies` and `group`
  are there, with `gh.rs`'s stand-in `gh` tests.
- 3.9: the targets page has "A GitHub release" and the member row,
  built, `github_release`. The integrations index lists it, and
  `site:build` passed.
- 4.1: `devenv.nix` sets `env.TIMBER_TOOLS_GH_CLI` to nixpkgs' `gh`, not
  on the shell's `PATH`. Its `--version` printed `gh version 2.100.0
  (nixpkgs)`.
- 4.2: `crates/timber/tests/github_release.rs` is behind `tools`.
  `cli:github-release` is tagged `check-github-release` alone. Run with
  `env -u TIMBER_TOOLS_GITHUB_RELEASE`, it failed naming the setting,
  as the task's check says.
- Record 0043 is in `docs/adr/`. Rules core.21 and core.79 are in
  `docs/architecture/core.md`.

## Requirements and scenarios

Each requirement of the change's five delta specs, with the code that
meets it and the tests that hold each scenario.

### github-release-target

- **A target profile of its own.** `GithubRelease` implements `Target`
  and is bound as `github_release`. GitHub and `gh` are named only
  under `profiles/target/`. "An environment names the GitHub release
  target": the binding test above and the `group` submodule.
- **The table names the credential and the assets.** `table.rs`. "An
  asset GitHub would rename":
  `an_asset_with_a_space_is_unreadable_naming_assets_and_the_asset`,
  `an_asset_github_would_rename_is_unreadable_naming_its_character`,
  `an_asset_beginning_or_ending_with_a_dot_is_unreadable_naming_it`. "An
  asset named twice": `a_repeated_asset_is_unreadable_naming_assets`. "A
  release of notes alone": `an_empty_assets_is_a_release_of_notes_alone`,
  `a_release_of_notes_alone_uploads_nothing`.
- **The table reads no other key.** Every operation reads `Table` first.
  "A token in the table":
  `a_token_in_the_table_is_unreadable_naming_it_and_the_two_and_never_its_value`,
  and the group's `foreign_settings`.
- **The credential is read at the moment of use.** `GithubRelease::token`
  runs in current, apply and report, and not in desired. Each of the
  three scenarios, "A credential the host does not name", "A file that
  cannot be read" and "An empty file", is held by the `credential`
  submodule's test of that name. Each asserts that apply, current and
  report are unreadable with nothing asked of GitHub. With them:
  `desired_reads_no_credential`,
  `the_token_is_handed_with_its_trailing_newline_trimmed` and
  `the_token_is_in_no_answers_words`.
- **The token reaches each run of `gh` alone.** `Gh::ran` sets
  `GH_TOKEN`, `GH_PROMPT_DISABLED`, `GH_NO_UPDATE_NOTIFIER` and
  `GH_CONFIG_DIR` for the run alone, and no argument holds the token. "The
  token goes in the run's variable": `of_their_own`, asserted after every
  stand-in test in `gh.rs`. `Token`'s debug print holds no token.
- **No run of `gh` reads the host's own settings.** `GH_*` and
  `GITHUB_*` are dropped, and each operation runs in a temporary folder
  of its own, removed after, with an empty `config/`. "The host's own
  settings are not read": `the_hosts_own_gh_settings_reach_no_run`.
- **The release is the checkout's repository's.** `GithubRelease::index`
  refuses before `gh` runs, and every run names
  `github.com/<owner>/<name>`. "Another repository's index":
  `another_repositorys_index_is_refused_naming_both`, with
  `an_index_naming_no_revision_is_refused` and
  `an_unread_checkout_repository_is_the_answer_with_nothing_asked`.
- **desired is the channel's head.** "An empty channel":
  `desired_is_the_head_and_an_empty_channel_is_absent_with_nothing_asked`.
- **Apply reads the notes and the assets by digest.** `notes` and
  `assets` read through `file_artifact` before the token. "A version
  published before notes":
  `a_version_published_before_notes_is_refused_naming_its_notes`. "An
  asset the version does not carry":
  `an_asset_the_version_does_not_carry_is_refused_naming_the_output_and_the_version`.
  With them: `notes_that_are_not_utf8_are_refused`.
- **Apply releases only the tag GitHub holds at the version's revision.**
  `GithubRelease::tagged` and `Gh::tag`, which peels tag objects. "A tag
  GitHub does not hold":
  `a_tag_github_does_not_hold_is_refused_naming_it_and_the_repository`.
  "A tag at another revision":
  `a_tag_at_another_revision_is_refused_naming_it_and_both_revisions`.
  "A release made only on a tag held":
  `a_draft_is_made_only_on_the_tag_titled_by_it_with_the_notes` and
  `a_release_is_published_on_the_tag_not_a_prerelease_and_marked_latest`,
  both asserting `--verify-tag`. No run makes, moves or deletes a tag.
- **Apply makes a release the tag lacks as a draft.** "A first apply":
  `a_first_apply_makes_a_draft_uploads_its_assets_and_publishes_it_latest`.
  "A draft an earlier apply left":
  `a_draft_an_earlier_apply_left_is_finished_with_no_second_create`.
- **Apply attaches each asset the release does not hold.** One `gh
  release upload --clobber` run, each file named by its output. "Assets
  held already": `every_asset_held_uploads_nothing`. "An asset of other
  bytes": `only_the_assets_not_held_are_uploaded_and_none_is_removed`,
  with `an_asset_github_answers_with_no_digest_is_uploaded_again` and
  `assets_are_uploaded_in_one_run_each_under_its_outputs_name`.
- **Apply publishes the release last, marked latest.** `gh release edit
  --verify-tag --draft=false --prerelease=false --latest --title <tag>
  --notes-file`. "A move back":
  `a_move_back_marks_the_earlier_release_latest_again_with_its_notes`.
- **current is the version of the release GitHub marks latest.**
  `GithubRelease::running` resolves `version_name(component, tag)`
  through the store. "After a converged apply":
  `a_converged_release_is_current_and_reported_with_when_github_updated_it`.
- **current is unread where no release names a version.** "A repository
  with no release": `no_release_marked_latest_is_unread_naming_the_repository`.
  "A release made by hand":
  `a_latest_release_whose_tag_names_no_version_is_unread_naming_the_tag`.
  With them: `another_components_version_under_the_same_tag_is_not_taken`.
- **report is the latest release's version and when GitHub updated it.**
  "Reported after a converged apply":
  `a_converged_release_is_current_and_reported_with_when_github_updated_it`.
  With it: `a_release_with_no_updated_at_is_current_and_its_report_unread`.
- **GitHub's words are the answer.** `Gh::run` and `Gh::found` answer
  `Tool::failed` with `gh`'s stderr verbatim, the command named and no
  token in it. "A token that may not write":
  `a_failure_at_each_step_is_the_answer_in_ghs_words_and_nothing_runs_after`,
  where nothing runs after a failed `create`, and
  `another_failure_is_unreadable_naming_the_command_and_carrying_ghs_words`.
- **The profile passes the target group.** "The target group over a
  scripted GitHub":
  `the_target_group_passes_over_the_scripted_github_holding_a_marker`,
  in `check-fast`.
- **Against GitHub, by hand.** `cli:github-release` under
  `check-github-release`. "Run without the setting" failed at
  `github_release.rs:102` naming `TIMBER_TOOLS_GITHUB_RELEASE`, before
  any run of `gh`. The run against a repository is the operator's, as
  above.

### environment-apply

- **Apply hands the target the verified version.** `apply.rs` makes
  `Verified` with `tag(component, &version)` after the claims are
  judged. "The channel's head applied" is held by the existing apply
  tests, which passed. "The version's tag handed over":
  `the_verified_version_names_the_components_tag`, which hands over
  `web-v1.2.0`.

### seam-contracts

- **A target's contract.** `Target::apply` is handed `Verified` with its
  index, version and tag. The three scenarios this change leaves as they
  were are held by the existing tests, which passed. "The verified
  version names its tag": `the_fake_records_the_tag_it_is_handed` and
  `target::tests::every_type_round_trips_through_json`.

### host-bindings

- **The file names the credentials a target's tool reads.** The binding
  keeps each path and opens none. "A credential named by its path":
  `credentials_a_home_path_is_expanded_and_the_file_at_it_never_opened`.
  "No credentials table":
  `credentials_no_table_names_none_and_publish_binds_as_before`.

### conformance-suite

- **The GitHub release target runs the target group.** "The scripted
  GitHub": the `group` submodule's test, as above. The run by hand is
  `check-github-release`'s.

## Verdict

The work holds. Both layers pass, every task is ticked and done, and
every requirement and scenario of the five delta specs is met by code
and held by a test or the suite. OpenSpec's own archive writes the specs
the branch holds. The change stands archived, as 5.1 archived it, with
this report beside it. No requirement was found wanting, and nothing
goes back to the code stage. One step is left before landing: the
operator's run of `moon run ':#check-github-release'` against a
throwaway repository, its answer in the landing's description.
