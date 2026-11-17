# Verify: timbers-store-works-on-githubs-container-registry

Verified on 2026-11-17 at 6094b4ff, on the branch
`change/timbers-store-works-on-githubs-container-registry`, cut from
`bolt/timber-publishes-from-githubs-registry`. The code stage archived
the change at 6094b4ff (task 6.1); this report is written into the
archived change.

Verdict: the whole checks pass and the work holds. Every task is
ticked and every requirement and scenario of the two delta specs is
met.

## The whole checks

Run once each, as the repository's instructions name them, inside
`devenv shell`:

| layer | command | outcome |
|---|---|---|
| `check-fast` | `moon run ':#check-fast'` | passed, exit 0: 28 tasks, 19 from moon's cache at inputs unchanged since their last passing run; `cli:test` ran 1644 tests, none failed; `root:specs`, `root:changes` and `root:changes-cases` passed |
| `check-tools` | `moon run ':#check-tools'` | passed, exit 0: `cli:tools`, 101 tests, none failed, the store group against zot and the reference distribution among them |

The GHCR check, `check-ghcr`, is outside the gate (core.38) and was not
run here. Its proof is task 3.1's three passing runs, recorded in
record 0039.

## Tasks

All 11 tasks of `tasks.md` are ticked `[x]`, and each is found in the
tree:

- **1.1** `Client::authorize` takes the method and the repository; a
  `Bearer` write, any method but `GET` and `HEAD`, asks
  `repository:<repository>:pull,push`, a read the challenge's scope.
  The module comment says so. The unit test
  `a_write_asks_to_pull_and_push_its_repository` asserts both queries
  and the put's `200` (`client.rs`).
- **2.1** `run_prefix`, `Prefixed`, `LiveRig` and
  `store_group_against` are in `tests/registry/mod.rs`;
  `registry_ecr.rs` uses them.
- **2.2** `tests/registry_ghcr.rs`, behind `tools`, reads
  `TIMBER_TOOLS_GHCR`, panics naming `env.TIMBER_TOOLS_GHCR`, the form
  `ghcr.io/<owner>/<path>` and the `write:packages` credential where it
  is unset or names another host, and holds the three named tests. The
  third reads the index anonymously and fails only on a `200`.
- **2.3** `tools-ghcr` in `crates/timber/moon.yml`, tagged `check-ghcr`
  alone, its inputs `tools-ecr`'s with `$TIMBER_TOOLS_GHCR`, with the
  comment the task asks for.
- **3.1, 3.2** Record 0039 holds five runs: one refused on the login,
  one failing on a blob held and not served, and three passing in a row
  at 70475946, 20:19:43 to 20:21:13 UTC. The fix (70475946) answers a
  blob whose `GET` is `404` and whose `HEAD` holds it unreadable in the
  registry's words, with the stub answering GitHub's words verbatim and
  the unit test
  `a_blob_held_and_not_served_is_unreadable_in_the_registrys_words_never_absent`.
  It changes no form the profile writes and not how name reads a tag,
  so no run of the ECR check was owed (core.38).
- **4.1** Record 0039, proposed, in MADR form, with design D2's options,
  the section "What GitHub's registry answered" and its table over
  every issue the spec lists, and the consequences of D5 and D7.
- **4.2** core.38's details name the GHCR check beside ECR's, citing
  record 0039 beside ADR 0015.
- **5.1** `stores.mdx` marks GitHub Container Registry built and names
  the GHCR check, record 0039 and a write's scope; `self-hosting.mdx`
  has "On GitHub's registry" and `ghcr.io/<owner>/timber` among the
  examples.
- **6.1** Archived; `openspec/specs/registry-store/spec.md` and
  `openspec/specs/tools-layer/spec.md` hold every added and modified
  requirement, and `root:specs` and `root:changes` pass.

## Requirements and scenarios

### registry-store

| requirement | scenario | met by |
|---|---|---|
| A write asks to pull and push its repository | A write challenged for pull alone | `a_write_asks_to_pull_and_push_its_repository`: a `PUT` asks `scope=repository%3Aone%3Apull%2Cpush` and is answered `200` with the token |
| | A read challenged for pull | the same test: a `GET` from a fresh client asks `scope=repository%3Aone%3Apull` |
| The store's scenarios pass against GitHub's registry | GitHub's registry, by hand | `ghcr_keeps_the_stores_contract` over `store_group_against`, which asserts every scenario ran and only "a name moved from where it is not" is not applicable; `ghcr_names_a_cache_entry_through_a_cache_manifest`; three passing runs in record 0039 |
| | A repository the push makes is private | `ghcr_makes_an_artifacts_repository_on_push_and_keeps_it_private`; record 0039 gives the realm's `401` to the anonymous read |
| | No repository of GitHub's registry named | `repository()` in `registry_ghcr.rs` panics naming `env.TIMBER_TOOLS_GHCR` where it is unset, empty or not `ghcr.io` |
| What GitHub's registry answered is recorded | The record read | record 0039's table: artifact manifest, image index, referrer with a subject, referrers API, put by digest, cache manifest, write's token, repository made on push, its visibility, mount, blob held and not served, tag read after its put; each with the date and what the profile does |
| Absent, refused and unreadable kept apart (modified) | Bytes the registry holds and will not serve | `Registry::blob` asks a `HEAD` after a `GET` `404`, and answers unreadable carrying the registry's words where it is held; the unit test proves it over the stub, with absent kept for bytes never put |
| | A registry that cannot be reached; A tag the registry will not move | unchanged by this change, and passing in `cli:test` and `cli:tools` |

### tools-layer

| requirement | scenario | met by |
|---|---|---|
| A check that needs a person's sign-in runs outside the gate (modified) | The gate runs no check that needs a sign-in | `tools-ghcr` is tagged `check-ghcr` alone, so neither `#check-fast` nor `#check-tools` runs it |
| The GHCR check's repository and credential are the host's | The GHCR check's repository | the check reads the repository from `TIMBER_TOOLS_GHCR` alone and reaches the credential through the host's Docker configuration; no tracked file sets either for the check |

## Observation

The tools-layer scenario says no file the tree tracks names the
repository or the credential. Record 0039 names
`ghcr.io/boomdrivedev/timber-tools-check` as the repository its runs
wrote, as task 4.1 and design D8 ask, and as ADR 0015 names ECR's
`timber-tools`. That names where the proof ran, as history. No tracked
file configures the check with it, and no credential is named
anywhere. The requirement cites 64 and core.21, and the check takes its
repository and credential from the host alone, so the requirement
holds. A later change could make the scenario's words say "configures"
rather than "names".
