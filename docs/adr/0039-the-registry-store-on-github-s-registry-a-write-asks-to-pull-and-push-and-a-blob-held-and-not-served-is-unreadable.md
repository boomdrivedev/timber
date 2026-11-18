---
number: 39
title: The registry store on GitHub's registry - a write asks to pull and push, and a blob held and not served is unreadable
status: proposed
date: 2026-11-17
decision-makers:
- Chuck Swanberg, who chose GitHub's registry under boomdrivedev for the family's store, and the repository and the login its check ran with
- the construct stage of the change timbers-store-works-on-githubs-container-registry, by its design D2
- the code stage of the same change, by its answer to a blob held and not served
---

# The registry store on GitHub's registry - a write asks to pull and push, and a blob held and not served is unreadable

Status: proposed, until the user's yes on it as a record. The write's scope is the change's design D2. The answer to a blob held and not served is the code stage's own, made on rule core.19 and cheap to change. What GitHub's registry answered is as it answered.

Deciders: the user chose GitHub's registry under boomdrivedev for the family's store on their card `req_02f6ed1d21734e7ba2da` of 2026-11-16. On 2026-11-17 they named the repository the check writes, `ghcr.io/boomdrivedev/timber-tools-check`, with Docker's login for `ghcr.io` as its credential (card `req_8897c0801d834cab9f78`). They then logged Docker in again with a token carrying `write:packages` (card `req_4d833561e50f4a898c37`). The change's construct stage chose the write's scope (design D2). Its code stage chose the answer to a blob held and not served when the check's first run failed on it (design D6).
Sources: statement clauses 1, 58, 59, 64, 66, 72, 73, 93, C.1; ADRs 0003, 0014, 0015, 0023; rules core.19, core.21, core.38; the OCI distribution specification (main), its sections on pulling blobs, on pushing manifests and on the referrers API; the distribution project's token authentication specification; the change's design D2, D5, D6 and D7; five runs of the GHCR check, `moon run ':#check-ghcr'`, against `ghcr.io/boomdrivedev/timber-tools-check` on 2026-11-17, and reads by hand against the same registry the same day.

## Context and problem statement

C.1 names GitHub's registry among the registries the store binds, and the profile speaks the protocol it speaks, but nothing had run the store's scenarios against it. ADR 0015 records what ECR answered; this record holds what GitHub's registry answered, and two questions its answers raised.

GitHub's registry challenges a request it wants a credential for with a `Bearer` challenge whose scope names `pull` alone, a write's among them. The profile asked the challenge's realm for the scope the challenge named (ADR 0015). On a registry whose tokens are scoped as asked, a write's token would pull and never push, and the write would be refused. What does a write ask the realm for?

In a package GitHub's registry makes on a push and that holds no manifest yet, it answers a `HEAD` of a blob put there `200` and a `GET` of it `404 BLOB_UNKNOWN`, for minutes, until any manifest is put there. The profile read the `404` as absent. In the check's first run with a working login, the store group's first scenario put bytes, put them again and was answered that they were held, and then read them as absent (66, 72). What does a read answer where a registry holds bytes and will not serve them?

## Considered options

For a write's token:

1. A write, any method but `GET` and `HEAD`, asks for `repository:<repository>:pull,push`, `<repository>` the one the request is for; a read asks for the scope the challenge names, as before.
2. `push` added only where the challenge names the request's repository with fewer actions.
3. The challenge's scope for every request, as before.

For a blob a registry holds and will not serve:

1. A `GET` of a blob answered `404` is asked after by a `HEAD`. Where the `HEAD` answers the blob held, the bytes are unreadable in the registry's words; where it answers `404` too, they are absent.
2. A manifest put beside every blob the store puts, so the registry serves it.
3. The `404` read as absent, as before.

## Decision outcome

A write's token is option 1 (design D2). The token authentication specification has the client ask for the scope it needs, and Docker and go-containerregistry ask `pull,push` for a push without reading it from the challenge. Asking it for every write costs nothing where a challenge already names `push`. `service` stays the challenge's, and the token answered is held under the repository as before, so a write's token replaces a read's and the reads after it use it (64, rule core.21). GitHub's registry turned out not to need it. Its realm answers a token that carries the account's role on the package, whatever scope is asked, and a write sent with a token asked for `pull` alone was taken whole. The profile asks it still, for a registry that scopes its tokens as asked. Option 2 reads more of the challenge's form for the same answer. Option 3 asks a registry that names `pull` for a write, and scopes its tokens as asked, for a token that cannot push.

A blob held and not served is option 1. The registry answers that it holds the bytes, so absent is not the answer: absent is a digest the store does not hold, and a caller reads it as bytes never put (rule core.19). The `GET`'s `404` carries the registry's words, and the answer carries them verbatim (59). It costs one `HEAD` more, and only for a blob read whose `GET` answered `404`. Option 2 writes a manifest per blob on every registry for one registry's window, and changes the forms the profile writes, which needs ECR's check run again by hand (rule core.38). Option 3 answers absent for bytes the store holds. Nothing timber does reads bytes in that window: it lasts from a package's first blob to its first manifest, and a publish puts its artifacts' manifests, and a push its cache manifest, straight after their bytes.

The fix changes no form the profile writes and not how name reads a tag, so no layout's or registry's digests changed and ECR's check needs no new run.

## What GitHub's registry answered

The store's conformance group, the cache manifest's scenarios and a repository made on push were run against `ghcr.io/boomdrivedev/timber-tools-check`, owner `boomdrivedev`, path `timber-tools-check`, on 2026-11-17, by hand with the host's Docker login for `ghcr.io` (`moon run ':#check-ghcr'`, design D5). The first run, at the change's commit 56fd31f9, from 19:51:15 to 19:51:31 UTC, was refused every write and every read. The login was then a token the realm refused even a read: `{"errors":[{"code":"DENIED","message":"denied"}]}`. That was the login's answer, and no form's. The second run, at the same commit with a login carrying `write:packages`, from 20:09:28 to 20:10:18 UTC, failed the store group's scenario `the same bytes put twice` on the blob the registry held and would not serve. Its other two tests passed. The three runs after the fix, at commit 70475946, from 20:19:43 to 20:20:16, 20:20:16 to 20:20:45 and 20:20:45 to 20:21:13 UTC, all passed, with no word from GitHub's registry and nothing refused. Each store group's report named its refusal of a name moved from where it is not as not applicable, and ran the repair of a name moved from a stale place in its stead. Those three runs are the proof the registry's acceptance rests on. The token's scope, a referrer's answer, the referrers API with a credential, a blob in a package holding no manifest and a tag read straight after its move were read by hand against the same registry, between 19:52 and 20:23 UTC the same day. The account is named in none of the words below.

| issue | what the store sends | what GitHub's registry answered | what the profile does |
|---|---|---|---|
| timber's artifact manifest | an OCI image manifest, artifact type `application/vnd.agentplot.timber.artifact.<ecosystem>`, config the metadata as canonical JSON, layers the bytes | taken: every run's store group and artifact's name passed with it | writes it as on every registry |
| timber's image index | an OCI image index, its component and version in annotations | taken: an artifact's name made at one resolved to it in each run | writes it as on every registry |
| a referrer with a subject | an OCI image manifest typed by the envelope type, its subject the attested digest, its one layer the envelope | taken, `201 Created` with no `OCI-Subject` header | writes the subject's referrers tag `sha256-<hex>`, which the declared repository holds, and lists claims from it |
| the referrers API | `GET /v2/<repository>/referrers/<digest>` of a manifest the repository holds | `404` `{"errors":[{"code":"MANIFEST_UNKNOWN","message":"manifest unknown"}]}`, read with no credential and with the host's | reads the first page's `404` as no API, and claims-of reads the referrers tag (C.1) |
| a manifest put by digest alone | `PUT /v2/<repository>/manifests/<digest>`, no tag | taken, `201 Created` | puts every manifest by digest, and a tag only for a name |
| the cache manifest | an OCI image manifest, artifact type `application/vnd.agentplot.timber.cache.v1`, config the empty descriptor, one layer the bytes | taken: every run's cache manifest scenarios passed | names bytes through it as on every registry |
| a write's token | a write sent with no credential, then the realm asked for a token | the challenge names `scope="repository:<owner>/<path>:pull"` alone for a write as for a read. A token asked for `pull` alone took an upload's open, `202`, its put, `201`, and a manifest's put under a tag, `201` | asks `repository:<repository>:pull,push` for every write, which GitHub's registry does not need and a registry that scopes as asked does |
| a repository made on push | a blob, then a manifest, put into a repository under the declared one that nobody made, `artifact/file/<run>-site` | made on the push: each run's artifact's name made its own package, and a nested path of the declared one was never refused | holds an artifact's name in its own repository under the declared one, as on every registry (ADR 0023) |
| a repository's visibility | a read of the index in that repository with no credential: the challenge followed to its realm with none | the realm answered `401` `{"errors":[{"code":"UNAUTHORIZED","message":"authentication required"}]}`, so the package is private; before the declared repository was made, a read of it with none was answered `403` `{"errors":[{"code":"DENIED","message":"requested access to the resource is denied"}]}` | nothing: a package is made public by the operator, in its settings (design D7) |
| a cross-repository mount | `POST /v2/<artifact's repository>/blobs/uploads/?mount=<digest>&from=<declared>` | taken, `201`: two mounts of two in each of the three runs | mounts each blob from the declared repository, uploading only where a mount is declined (ADR 0023) |
| a blob in a package holding no manifest | `GET /v2/<repository>/blobs/<digest>` of bytes just put into a package the put made | `404` `{"errors":[{"code":"BLOB_UNKNOWN","message":"blob unknown to registry"}]}`, while a `HEAD` answered `200`; by hand, in eight packages made so, still at three minutes, and served at once, `307` to its blob store, after a manifest was put there | answers the bytes unreadable in the registry's words, never absent |
| a tag read after its put | `GET /v2/<repository>/manifests/<tag>` straight after the tag's put | the new manifest: in the three runs no read of a tag lagged, and by hand twenty moves of a tag, each read straight after its put, all answered the new manifest | reads the tag again while it answers the former place, up to 30 seconds, as on every registry (C.1, ADR 0015) |

GitHub's registry refused no form the profile writes, so `oci.rs` writes on GitHub's registry what it writes on ECR, zot and the reference distribution, and no layout's digests changed.

### Consequences

- A host binds GitHub's registry by naming its repository, `[store] registry = "ghcr.io/<owner>/<path>"`, as every registry; nothing of the profile names it (core.1, core.2).
- Every repository under the declared one is a package of the owner's. GitHub's registry makes each on its first push, private, and an administrator of the organization makes one public, one at a time, in its settings. GitHub documents that a public package is never made private again. Each new artifact's name, and each new component, is a new package, private until that is done.
- The declared repository holds every blob, artifact, index and claim a host publishes, so it is public only where everything the host publishes may be.
- The host's credential is a token with `write:packages` for a publishing host, or `read:packages` for a host that only reads a private store, kept by `docker login ghcr.io` in its Docker configuration and reached at use (64, core.21).
- A token's reach on GitHub's registry is its account's role on each package, not the scope asked, so a credential cannot be narrowed by the scope it asks for.
- A package's untagged versions are never to be deleted: an artifact's manifest, a referrer and a cache entry's bytes are held by digest, untagged, and named by an index, a referrers tag or a cache manifest.
- A store whose declared repository is a new package answers a read of bytes it holds unreadable, in GitHub's words, until the repository holds a manifest, which the first publish puts.
- Each run of the GHCR check leaves private packages under the declared repository: one per run's artifact's name, beside the tags and manifests it writes in the declared repository and in the cache repository `cache/moon`, which every run shares. The reads by hand left eight more, `run1794946435-lag-<n>`. The operator removes them in the organization's package settings; the check deletes nothing, since that would need `delete:packages` and a client of GitHub's API in timber's tests (design D5).
- GitHub's registry is proven by the runs this record names and by no run of the gate, which holds no person's sign-in (core.38). A later change to the forms `oci.rs` writes, or to how name reads a tag, is proven against GitHub's registry again by hand, as against ECR.
