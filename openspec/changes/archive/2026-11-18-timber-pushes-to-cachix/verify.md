# Verify: timber-pushes-to-cachix

The verify stage's report on the branch `change/timber-pushes-to-cachix`
at bac07a63, cut from
`bolt/timber-publishes-github-releases-and-to-cachix`. The code stage
archived the change as its own task 5.1 (bac07a63), so this report sits
in the archived change's directory and lands with it.

## The whole checks

Both layers the repository's instructions name, run once at bac07a63
inside `devenv shell`, nothing fixed between them:

| layer | command | outcome |
|---|---|---|
| `check-fast` | `moon run ':#check-fast'` | passed: 28 tasks, in 4 m 2 s |
| `check-tools` | `moon run ':#check-tools'` | passed: `cli:tools`, 14 test binaries, 105 tests passed and none failed or ignored, in 5 m 51 s |

`check-fast` ran `seams:test`, `fakes:test`, `core:test`,
`conformance:test`, `conformance:coverage`, `cli:test`, `root:format`,
`root:lint`, `root:hooks`, `root:toolchain`, `root:compile-server`,
`root:specs`, `root:changes`, `root:changes-cases`, `root:versions`,
`root:release`, `root:marketplace`, `nix:typecheck`, `nix:test`,
`site:install`, `site:toolchain`, `site:build` and the plugins' tasks.
Its tests: `seams` 95, `fakes` 94, `core` 541, `conformance` 70 and
`cli` 1000 passed, none failed. The one ignored,
`profiles::target::dash0::cli::tests::a_child_runs_every_operation`, is
the Dash0 target's and is run by another test in a child process. The
Nix cache target's 41 tests and the GitHub release target's 52 passed.
`check-tools` ran `tests/nix_cache.rs` among its binaries, the attic
proof over the profile as this change leaves it, which passed.

The proof against Cachix, `check-cachix`, is outside the gate (rule
core.38) and was not run against Cachix here. It needs a throwaway
public cache and a token file a person makes. Its run by hand is the
operator's, before the change lands (tasks, "Workflow follow-up").

## OpenSpec

- The change is archived, so `openspec validate timber-pushes-to-cachix`
  and `openspec instructions apply` answer "not found" on the branch.
  Both were run on a scratch copy of `openspec/` at 2919de6f, the commit
  before the archive. Validate answered "Change 'timber-pushes-to-cachix'
  is valid". The archive there reported 11 of 12 tasks done, the one
  left being 5.1, the archive itself. After the archive, `tasks.md` holds
  all 12 ticked and none open.
- On that scratch copy, `openspec archive timber-pushes-to-cachix --yes`
  wrote the same specs as bac07a63, byte for byte: `nix-cache-target`
  (5 requirements added, 4 modified) and `conformance-suite` (1 added).
  The only other difference from the branch is task 5.1's tick. So the
  archive is OpenSpec's own.
- On the branch, `openspec validate --specs --strict` answered 50
  passed, 0 failed, and `openspec validate nix-cache-target --type spec
  --strict` answered "Specification 'nix-cache-target' is valid", task
  5.1's check.

## Tasks

Every task of `tasks.md`, 1.1 to 5.1, is ticked, and the branch holds
the work each one names:

- 1.1: the branch holds the bolt with `timber-publishes-a-github-release`
  in (b73c9c3b, its archive under
  `openspec/changes/archive/2026-11-18-timber-publishes-a-github-release/`).
  `profiles::run::tests::a_run_is_handed_variables_of_its_own` passed in
  `cli:test`.
- 1.2: core.21's details in `docs/architecture/core.md` rule out
  `cachix` run with the host's `CACHIX_*` variables or its own
  configuration, and the token handed in `cachix authtoken` or written
  anywhere. They let the Nix cache target read its Cachix credential's
  file when apply pushes and set `CACHIX_AUTH_TOKEN` for that one run.
  `grep -c CACHIX_AUTH_TOKEN` prints 1 and `grep -c 0044` prints 1.
  Record 0044 is in `docs/adr/`, with D2 to D7's paths not taken.
- 2.1: `profiles/target/token.rs`, declared in `profiles/target/mod.rs`,
  whose comment says the GitHub release target and the Nix cache target
  read their tokens through it. `path` looks a credential up and opens
  no file; `read` reads it into `Token`, trimmed, whose `Debug` prints
  `Token(…)`. Its six tests:
  `a_path_is_looked_up_and_its_file_not_opened`,
  `a_credential_the_host_does_not_name_is_unreadable_naming_it_and_the_bindings_file`,
  `a_file_that_cannot_be_read_is_unreadable_naming_the_credential_and_the_path`,
  `a_file_of_whitespace_alone_is_unreadable_naming_the_credential`,
  `a_trailing_newline_is_trimmed` and
  `the_token_is_in_no_answers_words_nor_its_debug_print`. The GitHub
  release target's `token` calls the module, `gh.rs` takes `Token` from
  it, and its `credential` tests pass with their words unchanged.
- 3.1: `nix_cache/table.rs` reads a `Cache`, `Attic` or `Cachix { name,
  credential }`, displayed `fleet:swancloud` or `boomdrive on Cachix`.
  The attic tests are kept, and the new ones cover Boomdrive's cache,
  every refusal of the task, `https://boomdrive.cachix.org`,
  `-boomdrive`, `Boomdrive`, an empty name, and a credential with a
  space, empty, or not a string.
- 3.2: `NixCache::new(store, credentials)` runs `attic`, `cachix` and
  `nix-store` from `PATH`; `with_programs` takes the cachix program.
  `bound` hands it the credentials, its comment saying so (record 0044).
  The attic tests and `tests/nix_cache.rs` pass empty credentials and a
  cachix program.
- 3.3: `apply` looks the credential's path up after the table, before
  the store; reads the closures and checks the roots; reads the token
  through `token.rs`; and `push_cachix` runs `cachix --config
  <folder>/cachix.dhall push <name> <roots…>` with `CACHIX_*` removed
  and `CACHIX_AUTH_TOKEN` set for the run alone, in a `tempfile` folder
  dropped when the run answers. The module comment says how. The `cachix`
  submodule's stand-in records its arguments, its environment, whether
  the config exists and what its folder holds, and fails with a test's
  words. Each item of the task has its test, listed under the
  requirements below.
- 3.4: `current_and_report_are_unread_naming_the_cachix_cache_and_read_no_credential`,
  over a host's file naming no credential, with nothing run.
- 3.5: the `group` submodule's `CachixRig`: no marker, `verified`
  publishes the closure's file into the fake store, `foreign_settings`
  adds `token`, and `unreadable_environment` answers the no-marker words
  naming `boomdrive on Cachix`.
  `the_target_group_passes_over_the_stand_in_cachix_holding_no_marker`
  asserts "converged and reported" alone not applicable.
- 3.6: the targets page's members row reads "a Nix binary cache, through
  Attic or Cachix", built, `nix_cache`. The section gains the `cachix`
  and `credential` keys, Boomdrive's example, the one-cache rule, the
  run of `cachix`, the token read when apply pushes and handed to that
  run alone, the host's `CACHIX_*` and `~/.config/cachix/cachix.dhall`
  never read, `timber apply` in the repository's shell, a cache whose
  owner holds its signing key not pushed to, and a cache environment per
  system. Attic's part is kept, under a heading of its own. `site:build`
  passed in `check-fast`.
- 4.1: `devenv.nix` sets `env.TIMBER_TOOLS_CACHIX_CLI` to
  `${pkgs.cachix}/bin/cachix`, with the comment the task asks for, and
  adds it to no package list. `"$TIMBER_TOOLS_CACHIX_CLI" --version`
  printed `cachix 1.12.1`, nixpkgs' at `devenv.lock`'s pin.
- 4.2: `crates/timber/tests/cachix.rs` behind `tools`, reading
  `TIMBER_TOOLS_CACHIX` as `<cache>#<token file>` and running the
  `cachix` `TIMBER_TOOLS_CACHIX_CLI` names, with D9's six steps.
  `crates/timber/moon.yml`'s `cachix` task is tagged `check-cachix`
  alone, with the `dash0` task's inputs and `$TIMBER_TOOLS_CACHIX`, and
  a comment citing core.38. `devenv shell -- env -u TIMBER_TOOLS_CACHIX
  moon run cli:cachix` failed, exit 101, panicking at `cachix.rs:103`
  with "the Cachix proof names no cache: set env.TIMBER_TOOLS_CACHIX in
  devenv.local.nix to `<cache>#<token file>`…", as the task's check
  says.
- 5.1: the change archived at bac07a63, as above.

## Requirements and scenarios

Each requirement of the change's two delta specs, with the code that
meets it and the tests that hold each scenario. The tests named are in
`nix_cache/tests.rs` and `nix_cache/table.rs`, all in `check-fast`.

### nix-cache-target

- **The Nix cache target is a target profile of its own** (modified).
  One `NixCache`, bound as `nix_cache`, serves both caches. Cachix is
  named in no source under `crates/*/src` outside `profiles/target/`;
  the crate's integration tests `tests/nix_cache.rs` and
  `tests/cachix.rs` name it as the profile's own tests. "Bound by its
  profile name" and "A Cachix cache bound by the same profile":
  `bindings::tests::an_apply_binds_every_target_built_through_a_store_it_never_makes`,
  and every `cachix` test reaching the target
  through the environment `{ cachix = "boomdrive", credential =
  "cachix-boomdrive", closures = [...] }`.
- **The target's table names the cache and the closures, and nothing
  else** (modified). `Table::read`. "A token in the table":
  `a_token_in_the_table_is_unreadable_at_every_operation_and_attic_is_not_run`
  and `a_key_the_target_does_not_read_beside_cachix_is_unreadable_naming_it`.
  "A cache with no server":
  `a_cache_with_no_server_is_unreadable_saying_it_is_server_and_cache`.
  "Both caches named":
  `both_caches_are_unreadable_naming_cachix_and_saying_the_table_names_one`.
  "No cache named":
  `no_cache_is_unreadable_naming_the_table_and_saying_it_names_none`.
  "A credential beside attic":
  `a_credential_beside_attic_is_unreadable_saying_attic_reads_its_own_configuration`.
- **Apply pushes each root with its closure through attic** (modified).
  `push_attic`, unchanged in its run. "Pushed with its closure": the
  attic tests, and `tests/nix_cache.rs` in the tools layer.
- **The push uses the environment's own credential** (modified, attic
  alone). attic is handed no token and no variable:
  `attic_is_given_the_cache_and_the_roots_and_nothing_else`. "The token
  stays attic's": `tests/nix_cache.rs`.
- **A Cachix cache is named by its name, with the host's credential.**
  `cachix_of` and `cachix_problem`. "Boomdrive's cache":
  `boomdrives_cache_is_read_as_the_cachix_cache_its_credential_and_its_closure`.
  "A cache's address in place of its name":
  `a_cachix_that_is_no_caches_name_is_unreadable_saying_it_is_the_name_on_cachix`.
  "Cachix with no credential":
  `cachix_with_no_credential_is_unreadable_naming_credential`. With
  them: `a_credential_that_is_no_credentials_name_is_unreadable_saying_its_form`.
- **Apply pushes each root with its closure through cachix.**
  `push_cachix`. "Pushed to Cachix":
  `cachix_is_given_the_config_push_the_name_and_every_root_once_after_nix_stores_check`,
  which asserts the runs `nix-store` then `cachix`, the arguments, each
  root once over two closures sharing one, and attic never run. "Cachix
  refuses the push": `cachix_failing_is_unreadable_in_its_own_words`,
  carrying "Binary cache boomdrive doesn't exist or you're not
  authorized to access it." verbatim, the command named and no token in
  it. With them: `a_root_this_host_does_not_hold_is_refused_and_cachix_never_runs`.
- **The Cachix token is the host's, read at the moment of use.**
  `push_of` looks the path up before `nix_closure::index`; `token::read`
  runs after `held`. "A credential the host does not name":
  `a_credential_the_host_lacks_is_unreadable_and_neither_the_store_nix_store_nor_cachix_asked`,
  over a version the store does not hold, so a store asked would answer
  otherwise, and nothing run. "A token file that cannot be read" and "A
  token file of whitespace alone":
  `a_missing_token_file_or_one_of_whitespace_is_unreadable_after_nix_store_and_cachix_never_runs`,
  nix-store run and cachix not. "current reads no credential":
  `current_and_report_are_unread_naming_the_cachix_cache_and_read_no_credential`.
- **The token reaches one run of cachix alone.** "The host's own cachix
  settings":
  `the_hosts_cachix_variables_are_removed_and_the_token_handed_to_the_run_alone`,
  where the run's only `CACHIX_*` is `CACHIX_AUTH_TOKEN` holding the
  file's token and timber's own variables are as they were after;
  `the_token_is_in_no_argument`; and
  `the_config_names_no_file_in_an_empty_folder_under_the_temporary_directory_gone_after`.
  "The folder is removed": that last test, run once with cachix
  succeeding and once failing.
- **A closure is pushed to Cachix and realised from it alone, by hand.**
  `tests/cachix.rs`: the wrong `CACHIX_AUTH_TOKEN` and
  `CACHIX_SIGNING_KEY` set in the test's environment, the apply, `nix
  copy` into a fresh store with no substituter but the cache and its key
  alone trusted, both paths' bytes checked, a second apply, then the
  target group. "Realised from Cachix alone" is the operator's run by
  hand. "No cache named": the run above, which failed naming
  `TIMBER_TOOLS_CACHIX` in its first step, before anything was built or
  pushed.

### conformance-suite

- **The Nix cache target on Cachix runs the target group.** "The
  stand-in Cachix":
  `the_target_group_passes_over_the_stand_in_cachix_holding_no_marker`,
  with "converged and reported" alone not applicable, and
  `the_rig_pushes_through_the_stand_in_cachix`. The run by hand is
  `check-cachix`'s, whose test runs the same group over the cache.

## Seen beside the change

Inside `devenv shell`, a `cachix` 1.12.1 is on `PATH` from a store
folder of its own, not the shell's profile; outside the shell there is
none. This change adds `cachix` to no package list (task 4.1), and the
proof runs `TIMBER_TOOLS_CACHIX_CLI`, not that one. The design's
reading of `cachix` names 1.11.1; the pin gives 1.12.1. Neither bears on
a requirement.

## Verdict

The work holds. Both layers pass, every task is ticked and done, and
every requirement and scenario of the two delta specs is met by code
and held by a test or the suite. OpenSpec's own archive writes the specs
the branch holds. The change stands archived, as 5.1 archived it, with
this report beside it. No requirement was found wanting, and nothing
goes back to the code stage. One step is left before landing: the
operator's run of `moon run ':#check-cachix'` against a throwaway public
cache, its answer in the landing's description.
