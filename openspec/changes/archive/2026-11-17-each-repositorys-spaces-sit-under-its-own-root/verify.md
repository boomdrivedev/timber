# Verify: each-repositorys-spaces-sit-under-its-own-root

Verified on 2026-11-17 at 893a56f8, on the branch
`change/each-repositorys-spaces-sit-under-its-own-root`, cut from
`bolt/timber-publishes-from-githubs-registry`. The code stage archived
the change at 893a56f8 (task 4.1). This report is written into the
archived change.

Verdict: the whole checks pass and the work holds. Every task is
ticked, and every requirement and scenario of the two delta specs is
met.

## The whole checks

Each layer ran once, as the repository's instructions name them,
inside `devenv shell`:

| layer | command | outcome |
|---|---|---|
| `check-fast` | `moon run ':#check-fast'` | passed, exit 0: 28 tasks, 17 of them from moon's cache because their inputs had not changed since their last passing run. `cli:test`, `root:lint`, `root:format`, `nix:test` and `nix:typecheck` ran fresh. `cli:test` ran 900 tests: none failed, 1 ignored. `root:specs`, `root:changes`, `root:changes-cases` and `site:build` passed. |
| `check-tools` | `moon run ':#check-tools'` | passed, exit 0: `cli:tools` ran 103 tests across 14 test binaries and none failed. `registry_store` ran 46 tests against zot and the reference distribution. |

The GHCR check, `check-ghcr`, is outside the gate (core.38) and was not
run here. Its proof is task 3.3's passing run, which record 0041
records: 2026-11-18, 01:59:48 to 02:01:05 UTC, at 2ea5f1d5, against
`ghcr.io/boomdrivedev/timber-tools-check`. The commits after 2ea5f1d5
touch only the change, the site and the record. The ECR check was not
owed, because the forms the profile writes and how `name` reads a tag
are unchanged (core.38, record 0041).

## Tasks

All 16 tasks of `tasks.md` are ticked `[x]`, and each is found in the
tree:

- **1.1** `Registry::new` reads `<host>/<root>` into the field `root`.
  `tag::repository` takes `root`. `Registry::for_repository` keeps the
  answer and refuses an owner written `artifact` (`Why::Said`), naming
  `<root>/artifact/`. `place` and `cache_repository` follow D3. The
  module's tests build their stores with `store_for`. The four renamed
  tests and the four new ones are there.
- **1.2** `profiles::store::bind` and `bind_for` take an
  `Answer<RepositoryName>`. `bindings::bind` computes it once, using
  the given landing remote for a `push --left` whose file names no
  `[landing]`, and passes it to every store, the targets' included.
  Both module comments and the file's example say so. The renamed test
  and the two new tests are in `bindings/tests.rs`.
- **1.3** The `registry` option in `nix/modules/options.nix` says it
  names `<host>/<root>`, such as `ghcr.io/<owner>/timber`. It names
  `<root>/<owner>/<name>/` and `<root>/artifact/`. Its name and type
  are unchanged. `nix:typecheck` passed.
- **1.4** `self-hosting.mdx` names a registry's host and root and
  rewrites "A registry's repositories" and "On GitHub's registry". It
  carries D8's upgrade notes. `site:build` passed.
- **1.5** core.62's details carry the Not ruled out for GitHub's
  registry, citing C.1, 95, 96 and 135 as amended at f412008, and
  record 0040.
- **2.1** `reached`, a `RefCell` kept for the store's life, records
  each digest's repositories. `resolve` and `name` keep each digest
  where its name is, and `manifest` keeps each digest a manifest
  names. `find` asks the reached repositories, then the repository's
  own, then the cache repository. A push's `named_at` asks only the
  cache repository. Both tests are there.
- **2.2** `hold` carries timber's referrers through `carry`, mounting
  each referrer's blobs and putting its manifest. It tags the subject's
  referrers where the put answers no `OCI-Subject`. All three tests are
  there.
- **2.3** `attach` writes into the repository's own, every repository
  the subject was reached in, an index's holders from its annotations,
  and the repository the read found it in. It writes into each by
  `HEAD`, only where the attestation is missing, and answers held
  already only where every holder held it. Both tests are there.
- **2.4** `tag::is_referrers` is `pub(super)`, and `versions` skips
  every tag it names. `a_names_versions_leave_out_its_referrers_tags`
  is there.
- **2.5** `stores.mdx` describes the root, the repository's own, the
  artifact space, reads where a name was resolved, and claims written
  into every holder. It names record 0040.
- **2.6** Record 0041 is proposed and in MADR form. It holds D1 to D6
  and D8 with the paths not taken, and the code stage's two answers.
- **3.1** `Server::store()` and `RegistryRig` bind `acme/site`, and
  `Server::store_for` is there. `tags_of`, `holds` and `cache_of` read
  under the root. The three tests are rewritten for D3, and
  `a_pins_claims_are_read_from_its_artifacts_repository_by_a_store_of_another_repository`
  is in `on_both!`. It passed on both registries in `check-tools`.
- **3.2** `LiveRig` binds `timber-checks/store`.
  `ghcr_keeps_each_repositorys_packages_under_its_own_root` is in
  `registry_ghcr.rs`. Both files' module comments say so.
- **3.3** Record 0041 holds two runs. The first was a failing
  first-write window of record 0039, with nothing changed in the
  profile. The second passed all four tests. It also holds the places
  test's printed words: GitHub's registry took each nested package
  path, four deep.
- **3.4** `stores.mdx` says GitHub's registry was proven under a root
  per repository on 2026-11-18. Record 0041 carries the run's times in
  UTC.
- **4.1** Archived. `openspec/specs/registry-store/spec.md` and
  `openspec/specs/host-bindings/spec.md` hold every added and modified
  requirement and drop the two removed ones. `root:specs` and
  `root:changes` passed.

## Requirements and scenarios

### registry-store

| requirement | scenario | met by |
|---|---|---|
| The binding declares a registry and a root | Everything under the root | `a_repositorys_components_and_cache_hang_under_its_lowercased_owner_and_name` asserts that the stub's repositories are only those under `acme/store`. Every repository path is built by `tag::repository` from the root. |
| A repository's own root is its remote's owner and name | An owner and a name lowercased | the same test: `Acme/Site` holds `lib-v1.2.0` in `…/acme/site/lib` and the index in `…/acme/site` |
| | An owner named artifact | `an_owner_named_artifact_is_refused_naming_it_and_nothing_written`. `for_repository` refuses `tag::component(owner) == "artifact"`. |
| A store bound for no repository reads the artifact space alone | A consumer with no remote; A write with no remote | `a_store_bound_for_no_repository_refuses_its_writes_and_resolves_an_artifacts_name`; `a_store_whose_landing_names_no_remote_resolves_an_artifacts_name_and_refuses_a_put` |
| A digest is read where its name was resolved | Another repository's pin read where its name is | `a_digest_is_read_where_its_name_was_resolved` (stub), and the tools test of 3.1, which asserts that the reader asked nothing outside `artifact/npm/site/` |
| | A digest no name reached | `a_digest_no_name_reached_is_read_from_the_repositorys_own_then_its_cache` |
| A claim is held beside its subject in every repository holding it | Claims carried to an artifact's name | `an_artifacts_name_holds_the_claims_on_its_index_and_artifacts` |
| | A claim attached after the names are made | `a_claim_attached_after_the_names_is_held_in_each_repository_holding_its_subject` |
| | An attestation held in part | `an_attestation_held_in_part_is_written_where_it_is_missing` |
| | Another tool's referrer is not carried | `another_tools_referrer_is_not_carried_into_a_names_repository` |
| Three spaces under the root | A component's version, an artifact's name and a cache entry | `a_repositorys_components_and_cache_hang_under_its_lowercased_owner_and_name`; the three tools tests rewritten in 3.1 |
| | Two repositories' components of one name | `two_repositories_components_of_one_name_are_held_apart` |
| An attestation is a referrer of its subject (modified) | Two attestations read back; The same attestation attached twice; Another tool's referrer | `the_same_attestation_attached_twice_is_held_once` and `another_tools_referrer_is_not_answered_and_is_left` in `registry_store`, on both registries, and the store group's attestation scenarios. `attach` and `claims_of` answer absent for a subject `find` finds nowhere. |
| A declared component's versions and channels are tags of its own repository (modified) | A declared component's version; Two components' channels of one name; A repository that declares no component; A channel moved from where it is not | `a_declared_components_version_is_a_tag_of_its_own_repository`, `two_components_channels_of_one_name_are_tags_of_their_own_repositories`, `a_repository_declaring_no_component_keeps_its_version_and_channel_in_its_own`, `a_channel_moved_from_where_it_is_not_is_refused_naming_where_it_is_in_its_repository`; the tools test `a_declared_components_versions_and_channels_are_tags_of_its_own_repository` |
| A declared component's repository holds its indexes (modified) | An index published; An index put before this change | `a_declared_components_index_is_held_in_its_repository_untagged_from_the_put`, `a_components_repository_holds_the_claims_on_the_artifacts_its_index_names`, `an_index_its_components_repository_lacks_is_held_there_before_a_channels_tag` |
| An artifact's name is a tag of a repository of its own (modified) | A scoped package's name; A version a tag cannot hold as written; A name moved from where it is not, in its own repository; One name for every repository | `a_scoped_packages_name_is_escaped_into_one_path_component`, `a_names_versions_are_its_repositorys_tags_every_page_read_back`, `a_name_made_again_from_nothing_in_its_repository_is_refused_naming_where_it_is`, `a_digest_is_read_where_its_name_was_resolved`; the tools test `artifact_names_are_tags_of_their_own_repositories`. A path past 255 characters is covered by `a_name_whose_repository_runs_past_255_characters_is_refused_and_nothing_written`. |
| An artifact name's repository holds what its index names (modified) | A mount declined | `a_declined_mount_uploads_the_bytes_before_the_manifests_and_the_tag` |
| A name's versions are its repository's tags (modified) | Versions listed by tags; A name the registry never held; A referrers tag beside the versions | `a_names_versions_are_its_repositorys_tags_every_page_read_back`, `a_names_versions_leave_out_its_referrers_tags`, and the store group's listing scenarios on both registries |
| A cache entry is a tag of its build profile's cache repository (modified) | An entry named from bytes the declared repository holds | `an_entry_named_from_bytes_the_repositorys_own_holds_is_held_in_its_cache_repository` |
| A push writes into the cache repository alone (modified) | A push's cache entry; An entry's bytes read back | `a_pushs_cache_entry_is_held_in_the_cache_repository_alone`; the tools test `cache_entries_are_tags_of_the_build_profiles_cache_repository` |
| A repository nobody made is refused naming it (modified) | An artifact name's repository missing; A component's repository missing; A cache repository missing; The repository's own missing | `an_artifact_names_repository_missing_is_refused_naming_it_and_no_tag_put`, `a_components_repository_missing_is_refused_naming_it_and_never_asked_again`, `a_cache_repository_missing_is_refused_naming_it_and_the_repositorys_own_is_not_written`, `a_repositorys_own_that_is_not_there_is_refused_naming_it` |
| The store's scenarios pass against GitHub's registry (modified) | GitHub's registry, by hand; A repository the push makes is private; No repository of GitHub's registry named; Each repository's packages under its own root | `ghcr_keeps_the_stores_contract`, `ghcr_names_a_cache_entry_through_a_cache_manifest`, `ghcr_makes_an_artifacts_repository_on_push_and_keeps_it_private`, `ghcr_keeps_each_repositorys_packages_under_its_own_root`; the passing run in record 0041 |
| One declared repository holds the store; Three spaces under the named repository (removed) | — | gone from `openspec/specs/registry-store/spec.md`. Their migrations are the added requirements above. |

### host-bindings

| requirement | scenario | met by |
|---|---|---|
| A registry store is bound for the checkout's repository | A checkout of a forge's repository; A push's cache entries | `a_pushs_store_puts_bytes_into_its_cache_repository_and_every_other_commands_into_its_own`: `origin` `https://github.com/Acme/Site.git` gives `acme/store/acme/site` for `Publish` and `Sync`, and `…/cache/moon` for `Push` and `Left` |
| | A check job's file with no landing table | `a_check_jobs_store_is_bound_for_the_given_remotes_repository` |
| | A checkout whose landing names no remote | `a_store_whose_landing_names_no_remote_resolves_an_artifacts_name_and_refuses_a_put` |
| The file names the store, the landing and the signer (modified) | A registry; A registry named by something that is not a repository; the other five, unchanged | `a_registry_binds_a_store_named_by_its_host_and_root`, `a_registry_named_by_what_is_no_repository_is_unreadable_naming_the_key_and_the_value`, `a_text_naming_no_host_or_no_path_is_refused_naming_it`; the rest pass in `cli:test` as before |
| A push before the landing binds with a store alone (modified) | A check job's file; The same file for a publish | `a_registry_alone_binds_a_push_before_a_landing`, `a_push_before_a_landing_binds_with_a_store_alone`, `a_check_jobs_store_is_bound_for_the_given_remotes_repository` |
