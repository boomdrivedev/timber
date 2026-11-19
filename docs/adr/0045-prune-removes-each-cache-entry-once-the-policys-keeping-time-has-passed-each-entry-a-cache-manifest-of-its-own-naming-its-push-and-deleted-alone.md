---
number: 45
title: Prune removes each cache entry once the policy's keeping time has passed, each entry a cache manifest of its own naming its push and deleted alone
status: proposed
date: 2026-11-18
decision-makers:
- the construct of the change timber-removes-old-cache-entries, by its design D1, D2 and D4 to D7, on statement clause 145
- the change's code stage, by its answers under "What the code stage chose"
---

# Prune removes each cache entry once the policy's keeping time has passed, each entry a cache manifest of its own naming its push and deleted alone

Status: proposed. Every choice here is the construct's own, made on statement clause 145 as added at `boomdrivedev/blueprints` fecf4d9 on 2026-11-18, or the code stage's, and each is cheap to change: a keeping time is one line of the policy file, a removal costs a rebuild and nothing more (54), and only the cache manifest's form changes in the store.

Deciders: the construct of the change `timber-removes-old-cache-entries`, its design and its code stage, carrying out clause 145: the store keeps a repository's cache entries for as long as its owner's settings say, thirty days from their push unless they say otherwise, and timber removes each entry once that time has passed, with a credential that may remove cache entries and nothing else, and never anything accepted or released (54), the record of versions claimed (22) or the ledger (34).

Sources: statement clauses 22, 34, 52 to 54, 100, 135, 145 and C.1 at `boomdrivedev/blueprints` fecf4d9; records 0015, 0039, 0040 and 0042; rules core.6, core.19, core.21, core.22, core.30, core.43, core.48, core.52; the change's design D1 to D10; GitHub's "Deleting and restoring a package" and its REST API for package versions; AWS's guide to removing artifacts from ECR with `oras manifest delete`.

## Context and problem statement

Every landing's run pushes the cache entries of the tasks it ran, and nothing removed them, so a store on GitHub's registry or ECR grew with every landing for as long as the repository lived. Before this change a cache entry was a tag of its build profile's cache repository at a cache manifest that carried no annotation, so the same bytes always wrote the same manifest and nothing said when an entry was pushed; and the store seam removed nothing.

145 says what must hold. Six questions follow: where the owner's keeping time is written, how an entry says when it was pushed, how a registry removes one entry, how GitHub's registry does, how a layout does, and what prune does with each entry it lists.

## Considered options

For where the keeping time is written (design D1):

1. **The policy file's `[cache] keep_days`, a whole number of days from 0, read through the policy seam at a landed revision; the givens' fragment gives `keep_days = 30` beneath the file, as it gives the channels (chosen).** The owner's settings are the repository's one policy (C.17), read at a landed revision (65, core.22), and nothing but the policy profile reads the key (core.43). The table is the repository's, not a component's, since a repository's cache entries sit under its own root whatever component's tasks left them (C.1). Another key under `[cache]`, and a `keep_days` that is no whole number from 0, is unreadable naming the key and the file.
2. The host's bindings file. Two hosts publishing one repository would keep its cache for two times, and the setting would not travel with the repository.
3. The store's own setting, an ECR lifecycle rule. 145 has timber remove, and GitHub's registry has no such rule.
4. A duration's text, `30d`. A parser and its units for one number.
5. A line per component. The entries are the repository's.

For how an entry says when it was pushed (design D2):

1. **The cache manifest carries `io.github.agentplot.timber.store-name`, the name it was made for, and `org.opencontainers.image.created`, the moment the store put it by the host's clock, so each name made at bytes has a cache manifest of its own; a layout stamps the index entry of a name made or moved at bytes with the same `created` (chosen).** Deleting a manifest by its digest then removes that one name, which a registry's delete and GitHub's version delete both do with every tag at it. Its config and its one layer are as before, so resolving the tag still answers the layer's digest (C.1). Where name's check before its put finds the name held, no cache manifest is put (core.6).
2. The store's own push time. The distribution protocol carries none; ECR's and GitHub's are in their own APIs, and zot's and the reference distribution's in none, so the time would come from four places, two of them absent.
3. One manifest per bytes, with only the tag deleted. A tag's delete is optional in the distribution protocol, and GitHub's API deletes a version with all its tags, so two entries of the same bytes, such as two empty outputs, would go together, the second before its time.
4. The time in the config. C.1 makes the config the empty descriptor.
5. A referrer naming the time. A cache entry carries no claim beyond the writing host's own (52), and a referrer is one more write per entry.

For how a registry removes one entry (design D4):

1. **The bound build profile's cache repository listed page by page, each tag's cache manifest read for its layer's digest and its moment; a removal reading the tag's manifest, checking its layer is the digest listed, and deleting the manifest by its digest, the realm asked for `pull,delete` (chosen).** A tag at any other manifest, or one that cannot be read, is that entry's answer and never removed; a repository the registry does not hold lists none. Neither operation reads the mark of a root left nor asks anything of the repository's own, since cache entries are no history (record 0042). Nothing outside the cache repository, and no untagged manifest (record 0039), is deleted, and the layer's blob is the registry's to collect.
2. A seam operation removing any name. It could remove a version's, a channel's or an artifact's; 145 lets timber remove cache entries alone.
3. The cache manifest's digest in the seam. The seam addresses bytes (C.1), and the manifest is the registry profile's.

For how GitHub's registry removes one (design D5):

1. **Through GitHub's REST API: the root's namespace read once as an organization's or a user's, the cache repository's package, its path past the namespace with each `/` written `%2F`, read page by page for the version named the cache manifest's digest, and that version deleted by its id, with the token the host's Docker credential for `ghcr.io` holds, reached at use (chosen).** The listing stays on the distribution protocol, which GitHub's registry serves. The credential is the repository's own scheduled workflow's token with `packages: write`, the repository given the admin role on its cache packages and nothing on its component and artifact packages, so it writes and removes cache entries and nothing else (145, 135; record 0040).
2. The distribution protocol's delete. GitHub documents removal only through its API and its settings.
3. A token file named in `[credentials]`. The store's credential is the host's Docker login (record 0015), and one token reaches both.
4. GitHub's version `created_at` as the push moment. It would give one registry a second source of time.

For how a layout removes one (design D6):

1. **Each index entry whose ref name begins `cache/` listed with the digest it names and its `created` moment, none where it carries none; a removal taking the entry out of `index.json` under the index's lock where it names the digest given, and leaving the blob on disk (chosen).** The fake store does the same in memory, from a clock a test may set.
2. Removing the blob where nothing else reaches it. A layout keeps no count of what reaches a blob, deciding by reading every manifest risks an artifact's bytes (54, core.5), and the disk is the host's own.

For what prune does (design D7):

1. **`timber prune [revision] [--branch main] [--json]`: the landing asked first and a revision not landed refused; the keeping time read from the policy there, and the store asked nothing where it cannot be; then each entry listed, in order: one answered otherwise carried and never removed, one with no moment removed, one whose moment plus the keeping time is at or before the host's clock removed at the digest listed, and any other kept naming when its time passes; each removal answered on its own, an absent one gone already (chosen).** It asks the landing, the policy and the store's two cache operations alone, signs nothing, keeps nothing (61) and reads and writes no git ref, so the record of claimed versions (22) and the ledger (34) are out of its reach. Its binding holds the store bound for the build profile's cache repository as a push's is, a layout opened and never made, and no keeper, projection, target or reaching signer. What runs it is the operator's (core.52, C.16).
2. Prune inside a publish or a push. timber starts nothing after an offer or a landing (core.52), and a removal has no place on a landing's path.
3. A dry run. Removing an entry early costs a rebuild and nothing more (54).

## Decision outcome

Option 1 for each, as the change's design D1, D2 and D4 to D7 say.

### Consequences

- The first prune removes every entry pushed before this change, since those read with no moment. A landing whose entries are removed and whose publish has not run is refused naming the output (135, core.30); the next landing's run fills the cache.
- A landing must be published within the keeping time, since a publish never builds where the landing has verdicts (135).
- Listing costs a request per entry, on a schedule and off a landing's path.
- Two entries of the same bytes write two cache manifests where they wrote one, a few hundred bytes each.
- GitHub's API for a token's deletes is in public preview by GitHub's own page; a change in it shows in the GHCR check by hand.
- A push moment is the pushing host's clock and the time passed the pruning host's; a skewed clock removes an entry early or late, and early costs a rebuild alone (54).
- The reference distribution keeps a removed entry's bytes until its operator runs its garbage collection, and a layout keeps them.

## What the code stage chose

Answers the design left open, the code stage's own, each cheap to change:

- **A fragment that gives no `[cache] keep_days` leaves the keeping time unreadable naming the fragment, and every other operation reads it as before.** A test's fragment of other channels still binds. The path not taken was refusing such a fragment whole, which would make every channel answer unreadable for a line only prune reads.
- **Prune's binding answers a store that cannot be opened at each operation with the opening's words, as status's does.** A host whose layout is not there binds prune, and prune answers that the layout is not there, as the host-bindings spec says. The path not taken was failing the binding, as verify's does.
- **An entry the listing answers absent, its tag gone between the listing and its read, counts as an answer in prune's exit, as push counts an archive not on the host.** The path not taken was exiting absent for an entry already gone.
- **On a registry, a tag's manifest that cannot be read is that entry's answer unreadable naming the tag, whether the registry refused or failed, and a tag no name could have written lists the tag's text as its build identity.** The listing then carries every tag it found. The path not taken was answering the registry's refusal as refused, which reads as a refusal of the listing.
- **A delete through the distribution protocol is done on any 2xx, and GitHub's versions are read until a page lists none.** The specification answers 202, and a registry answering 200 has done it as well; GitHub's API pages by number and its `Link` header carries more than the next page. The paths not taken were 202 alone and following the `Link` header.
- **Every `Store` that wraps another delegates the two cache operations, the test wrappers of the timber crate among them.** The path not taken was each answering absent, which would hide the wrapped store's entries from a scenario that lists them through the wrapper.
- **The store group runs over the registry stub too, in the registry's unit tests**, beside the fake, the layout and the live registries, so the group's cache scenarios run in the gate's fast layer.
