# Tasks

Every command runs from the place's root, inside `devenv shell`.
Commit by pathspec, Conventional Commits scoped to the part of the
tree, citing clauses by number, rules by id and this change's design
by D-number. Submit nothing: Flume submits the place once its last
stage delivers. The headings are for reading; each task's check is its
own.

Read these before starting:

- `crates/timber/src/profiles/store/registry/mod.rs`, `tag.rs` and
  `stub.rs`.
- `crates/timber/src/profiles/store/mod.rs`.
- `bind`, `landing_remote` and `repository_named` in
  `crates/timber/src/bindings/mod.rs`.
- Statement C.1 as amended at f412008, on
  `boomdrivedev/blueprints` `design/timber-a-root-per-repository`.
- Record 0040 as amended. It is on main and not on this bolt, so read
  it with
  `git show main:docs/adr/0040-each-repository-keeps-its-own-root-in-the-store-and-on-githubs-registry-the-pool-writes-only-its-cache.md`.

No seam trait, shared type, OCI form, core module or layout store
changes. A task that cannot be done without one stops short with
`flume needs`.

## 1. The repository's own root

- [x] 1.1 Place each name under the root and the repository's own, in `crates/timber/src/profiles/store/registry/mod.rs` and `tag.rs` (design D1, D3; registry-store spec, "The binding declares a registry and a root", "A repository's own root is its remote's owner and name", "A store bound for no repository reads the artifact space alone", "Three spaces under the root", and the modified placement requirements).
  - `Registry::new` reads its text as `<host>/<root>`, with the same refusals as before. Rename the field `path` to `root`, and the `declared` parameter of `tag::repository` to `root`.
  - Add `pub fn for_repository(self, repository: Answer<RepositoryName>) -> Registry`, keeping the answer. A store with no repository given answers as one given an absent answer naming the repository it was bound for.
  - The repository's own is `<root>/<owner>/<name>`, each part written by `tag::component`. An owner whose component is `artifact` is refused (`Why::Said`), naming the owner and `<root>/artifact/`, the artifact space it would share.
  - Every write, and every name `place` maps outside the artifact space, answers a clone of the binding's answer where it holds no repository, and writes nothing.
  - `place` maps names as D3's table does: `artifact/…` under `<root>/artifact/`, `cache/…` to `<own>/cache/<build>`, a declared component's version and channel to `<own>/<component>`, and every other name to the own repository itself.
  - `cache_repository` is `<own>/cache/<build>`.
  - Every other use of the declared repository becomes the repository's own: put-bytes, put-artifact, attach, claims-of, read, `hold`, `named_at`, `mount`'s source, and the referrers functions. Tasks 2.1 to 2.3 change where reads and claims go further.
  - Rewrite the module comment for the root, the repository's own and the three spaces (C.1 as amended at f412008; record 0040).
  - Give the module's tests a helper building a store for the repository `acme/site`, and build every test's store with it. Rename `an_entry_named_from_bytes_the_declared_repository_holds_is_held_in_the_cache_repository` to `an_entry_named_from_bytes_the_repositorys_own_holds_is_held_in_its_cache_repository`. Rename `a_repository_declaring_no_component_keeps_its_version_and_channel_in_the_declared_one` to `a_repository_declaring_no_component_keeps_its_version_and_channel_in_its_own`. Rename `a_declared_repository_that_is_not_there_is_refused_naming_it` to `a_repositorys_own_that_is_not_there_is_refused_naming_it`. Rename `a_cache_repository_missing_is_refused_naming_it_and_the_declared_one_is_not_written` to `a_cache_repository_missing_is_refused_naming_it_and_the_repositorys_own_is_not_written`. Move every other test's component and cache paths under `acme/site`.
  - Add `a_repositorys_components_and_cache_hang_under_its_lowercased_owner_and_name`. A store for `Acme/Site` holds `lib:lib-v1.2.0` in `<root>/acme/site/lib`, `cache/b7e1` in `<root>/acme/site/cache/moon`, and `artifact/eco/site@1.3.0` in `<root>/artifact/eco/site`.
  - Add `two_repositories_components_of_one_name_are_held_apart`.
  - Add `an_owner_named_artifact_is_refused_naming_it_and_nothing_written`, checked by the stub's writes.
  - Add `a_store_bound_for_no_repository_refuses_its_writes_and_resolves_an_artifacts_name`.
  - Verify with `devenv shell -- cargo test -p timber --lib profiles::store::registry::tests`, where the new tests and every other test of the module pass.
- [x] 1.2 Bind a registry store for the checkout's repository, in `crates/timber/src/profiles/store/mod.rs` and `crates/timber/src/bindings/mod.rs` (design D2; host-bindings spec, "A registry store is bound for the checkout's repository" and the two modified requirements).
  - `profiles::store::bind` and `bind_for` take the repository as an `Answer<RepositoryName>`, handed to `Registry::for_repository` and ignored by the layout.
  - `bindings::bind` computes it once as `repository_named(repository, remote)`. `remote` is the remote `[landing]` names, or the given landing remote where the file names no `[landing]`, `push --left` among them, as `landing_remote` answers. It passes the answer to every store it binds, the targets' among them.
  - Say so in both modules' comments, and say in the bindings file's example comment that `registry` names `<host>/<root>`.
  - In `crates/timber/src/bindings/tests.rs`, rename `a_pushs_store_puts_bytes_into_the_cache_repository_and_no_other_commands_does` to `a_pushs_store_puts_bytes_into_its_cache_repository_and_every_other_commands_into_its_own`. Its host's directory becomes a git checkout whose `origin` is `https://github.com/Acme/Site.git`, and it expects `acme/store/acme/site/cache/moon` for `Push` and `Left`, and `acme/store/acme/site` for `Publish` and `Sync`.
  - Add `a_check_jobs_store_is_bound_for_the_given_remotes_repository`: `For::Left`, a file naming `[store]` alone, and a checkout whose `origin` is `https://github.com/acme/site`.
  - Add `a_store_whose_landing_names_no_remote_resolves_an_artifacts_name_and_refuses_a_put`: `[landing] branch = {}`. Resolving `artifact/eco/site@1.3.0` asks only `acme/store/artifact/eco/site`, and put-bytes answers absent naming the landing's remote, with nothing asked of the stub.
  - Verify with `devenv shell -- cargo test -p timber --lib bindings::tests`, where the new tests and every other test of the module pass.
- [x] 1.3 In `nix/modules/options.nix`, say that the `registry` option names `<host>/<root>`, such as `ghcr.io/<owner>/timber`. Each repository's components and cache hang under the root at the owner and name its remote gives, and artifacts' names under `<root>/artifact/` (design D1; C.1). The option's name and type stay. Verify with `devenv shell -- moon run nix:typecheck`.
- [x] 1.4 In `site/src/content/docs/guides/self-hosting.mdx` (design D8, D9; record 0040):
  - Say `[store] registry` names a registry's host and root, with the examples it gives.
  - Rewrite "A registry's repositories" for the root, the repository's own, its components and cache under it, and the artifact space under the root. Say the repository is named by its landing remote's owner and name, lowercased, and that a checkout with none still syncs and cannot publish.
  - Say what ECR's creation templates and each credential cover, as D9 lists.
  - Rewrite "On GitHub's registry" with D9's setup, and drop "The declared repository holds everything".
  - Replace the upgrade notes on the declared repository with D8's: what a store from before still holds, what is read by nothing now, and what a host does before following anything there.
  - Verify with `devenv shell -- moon run site:build`.
- [x] 1.5 In `docs/architecture/core.md`, add to core.62's details a Not ruled out: on GitHub's registry, where no token is scoped to some of an owner's packages, the job's own token given write on its repository's cache packages and nothing else. That token may still make a package of its own, which holds no name the gate accepted. Cite C.1, 95, 96 and 135 as amended at `boomdrivedev/blueprints` f412008, and record 0040 (design D10). Verify with `devenv shell -- treefmt --ci docs/architecture`.

## 2. Reads and claims where a name is

- [x] 2.1 Read a digest where its name was resolved, in `registry/mod.rs` (design D4; registry-store spec, "A digest is read where its name was resolved").
  - The store keeps, in a `RefCell` for its life, the repositories each digest was reached in, in the order reached. resolve and name keep the name's repository against the digest. Reading a manifest keeps its repository against each digest the manifest names, read by `oci::named`.
  - read, claims-of, attach's search for its subject, `named_at` and `hold` find a digest in its kept repositories, then the repository's own, then the cache repository, and mount a blob from the repository they found it in. A push's `named_at` still asks the cache repository alone (135).
  - Say so in the comments of `Store for Registry`.
  - Add `a_digest_is_read_where_its_name_was_resolved`. Store A for `acme/site` publishes `artifact/eco/site@1.3.0` with an attestation on the index and one on the artifact. Store B for `acme/app` resolves the name, reads the index, the artifact and its bytes, and asks the claims on both. Each answer is A's, and the stub records no request of B's to `acme/store/acme/app`.
  - Add `a_digest_no_name_reached_is_read_from_the_repositorys_own_then_its_cache`.
  - Verify with `devenv shell -- cargo test -p timber --lib profiles::store::registry::tests::a_digest`.
- [x] 2.2 Carry a manifest's claims wherever it is held, in `registry/mod.rs` (design D5; registry-store spec, "A claim is held beside its subject in every repository holding it", "A declared component's repository holds its indexes" and "An artifact name's repository holds what its index names").
  - `referrers_api`, `referrers_tag`, `tag_referrer` and `envelope` take the repository they read or write.
  - `hold` brings the referrers of timber's envelope type of each manifest it holds, from the repository it found the manifest in, into the target. Each referrer's config and layer are mounted, the referrer is put by its digest unless held, and it is added to the subject's referrers tag where the put answers no `OCI-Subject`. Another tool's referrers stay where they are.
  - Add `an_artifacts_name_holds_the_claims_on_its_index_and_artifacts`.
  - Add `a_components_repository_holds_the_claims_on_the_artifacts_its_index_names`.
  - Add `another_tools_referrer_is_not_carried_into_a_names_repository`.
  - Verify with `devenv shell -- cargo test -p timber --lib profiles::store::registry::tests`, where the new tests and every other test of the module pass.
- [x] 2.3 Write an attestation into each repository that holds its subject, in `registry/mod.rs` (design D5).
  - attach finds its subject as 2.1 does, absent with nothing written where it is found nowhere.
  - It writes into each of these that holds the subject, by a `HEAD` of the manifest: the repository's own, each repository the subject was reached in, and, for an index, its component's repository and each artifact's name's repository, from the index's own annotations.
  - It writes only where the attestation is missing, and answers held already only where every holder held it.
  - Add `a_claim_attached_after_the_names_is_held_in_each_repository_holding_its_subject`.
  - Add `an_attestation_held_in_part_is_written_where_it_is_missing`.
  - Verify with `devenv shell -- cargo test -p timber --lib profiles::store::registry::tests`, where the new tests and every other test of the module pass.
- [x] 2.4 Leave referrers tags out of a name's versions (design D6; registry-store spec, "A name's versions are its repository's tags"). Make `tag::is_referrers` `pub(super)`, and have `versions` skip every tag it names before `tag::read`. Add `a_names_versions_leave_out_its_referrers_tags`. Verify with `devenv shell -- cargo test -p timber --lib profiles::store::registry::tests::a_names_versions`.
- [x] 2.5 In `site/src/content/docs/guides/integrations/stores.mdx`, rewrite how a registry holds what it holds. The binding names a root. The repository's own and its components' and cache repositories hang under it, and the artifact space under the root. A name's repository holds what its tag reaches and the claims beside it. A digest is read where its name was resolved, and a claim is written into each repository holding its subject. Name record 0040 beside ADR 0023 (design D3 to D6). Verify with `devenv shell -- moon run site:build`.
- [x] 2.6 Write record 0041, or the next number free on main and on this branch, in `docs/adr/`. Use MADR form as `adrs.toml` sets and as record 0039 is laid out, status proposed (design D10). It holds design D1 to D6 and D8, each with its path not taken, as the construct stage of this change chose them on C.1 as amended at f412008 and record 0040. Verify with `devenv shell -- treefmt --ci docs/adr`.

## 3. The proof on registries

- [x] 3.1 Run the tools layer's registry tests under the root (design D7).
  - In `crates/timber/tests/registry/mod.rs`, `Server::store()` and `RegistryRig` bind the repository `acme/site`, and add `Server::store_for(owner, name)`. `tags_of`, `holds` and `cache_of` in `crates/timber/tests/registry_store.rs` read under the root.
  - Rewrite `artifact_names_are_tags_of_their_own_repositories`, `a_declared_components_versions_and_channels_are_tags_of_its_own_repository` and `cache_entries_are_tags_of_the_build_profiles_cache_repository` for D3's places.
  - Add `a_pins_claims_are_read_from_its_artifacts_repository_by_a_store_of_another_repository` to `on_both!`. One store publishes and attaches an acceptance after naming. A store for another repository resolves the artifact's name and reads every claim on the index and the artifact.
  - Verify with `devenv shell -- cargo test -p timber --features tools --test registry_store`, where every test passes on zot and on the reference distribution.
- [x] 3.2 Bind the live rigs, and add the GHCR check's test of the places (design D7; registry-store spec, "Each repository's packages under its own root").
  - `LiveRig` binds the repository `timber-checks/store` under the root the host names, and keeps each run's names apart by its prefix, as before.
  - Add `ghcr_keeps_each_repositorys_packages_under_its_own_root` to `crates/timber/tests/registry_ghcr.rs`. It makes a declared component's version and a cache entry for that repository, and an artifact's name at the version's index, each under the run's prefix. It asserts that each tag is held where D3 says, and that a store for `timber-checks/reader` reads the index's claims from the name's package.
  - Update both files' module comments.
  - Verify with `devenv shell -- cargo test -p timber --features tools --test registry_ecr --test registry_ghcr --no-run`, which compiles both, and with `devenv shell -- env -u TIMBER_TOOLS_GHCR cargo test -p timber --features tools --test registry_ghcr`, where each test fails naming `env.TIMBER_TOOLS_GHCR`.
- [ ] 3.3 Run the GHCR check by hand once (design D7). It needs the place's `devenv.local.nix` to set `env.TIMBER_TOOLS_GHCR` to a root under boomdrivedev, such as `ghcr.io/boomdrivedev/timber-tools-check`, and the host's Docker configuration to hold a credential for `ghcr.io` that writes the organization's packages. Where either is missing, tick nothing and stop short with `flume needs --to ops`, naming both and saying the run is the change's acceptance on GitHub's registry. Where GitHub's registry refuses a package's nested path, stop short with `flume needs --found --to design`, quoting the refusal. Where it refuses anything else, fix it in the profile for every registry with a unit test over the stub holding its answer verbatim, and run again. Keep the date, the run's start and end in UTC, and the places test's printed words, for 3.4. Verify with one passing run of `devenv shell -- moon run ':#check-ghcr' --force`.
- [ ] 3.4 In `site/src/content/docs/guides/integrations/stores.mdx`, say GitHub's registry's row was proven under a root per repository by hand on 3.3's date, and add that date and the run's times in UTC to record 0041. Verify with `devenv shell -- moon run site:build` and `devenv shell -- treefmt --ci docs/adr`.

## 4. The archive

- [ ] 4.1 Archive the change once groups 1 to 3 are done: `devenv shell -- openspec archive each-repositorys-spaces-sit-under-its-own-root --yes`. Then commit the archived change and the two specs it writes. Verify with `devenv shell -- moon run root:specs root:changes`, where `root:specs` passes every spec and `root:changes` finds no open change.
