---
number: 43
title: A GitHub release target runs gh with the token read at use, releases only the tag GitHub holds at the version's revision, and current is the release marked latest
status: accepted
date: 2026-11-18
decision-makers:
- the construct stage of timber-publishes-a-github-release
---

# A GitHub release target runs gh with the token read at use, releases only the tag GitHub holds at the version's revision, and current is the release marked latest

Status: accepted on 2026-11-18 as the construct stage's own choice, made on the rules written and cheap to change: the statement and record 0033 fix what the GitHub release target does, and this record fixes how the profile does it.

Deciders: the construct stage of the change `timber-publishes-a-github-release`, writing its design (D2 to D9).

Sources: `boomdrivedev/blueprints:design/timber/requirements.md` C.12, 22, 90 and 134, at fecf4d9; record 0033 and its consequences; rule core.21; GitHub's CLI, `gh` 2.102.0 as nixpkgs gives it, its help and its answers read against `boomdrivedev/flume`'s releases on 2026-11-18; flume's release job in `.github/workflows/ci.yml` at its main.

## Context and problem statement

C.12's GitHub release row says what the target does. push makes or updates the release of the version's tag, with its notes (134) as the release's text and the version's artifacts the policy names as its assets, with the target's own credential. current is the release GitHub marks latest.

The row doesn't say six things: which tool runs, how the token reaches it, how the target learns the version's tag, which artifacts are assets and under what names, how a release is made and finished, and what a release is titled and marked.

`gh` settles what can be built. It takes a token only in `GH_TOKEN` or `GITHUB_TOKEN`, or from the sign-in under its configuration folder, and reads no token file. `gh release create --verify-tag` refuses a tag GitHub lacks, where without it GitHub makes the tag at the default branch's head. `gh release view <tag>` finds a draft by the tag it will take, and answers each asset's digest. GitHub never marks a draft or a prerelease latest, and renames an asset whose name holds other characters than letters, digits, `.`, `-` and `_`.

## Considered options

**The tool.**

1. **`gh`, GitHub's own CLI, as found on `PATH` (chosen).** As attic, clan and `dash0` are found. The host runs `timber apply` in the repository's shell, whose `gh` is its pin.
2. **GitHub's REST API spoken by timber.** Not taken: timber would be a second client of a service whose own tool exists, writing its upload protocol and draft lookup again, and a stand-in server of the tests' own would prove timber against its own idea of GitHub.

**The token.**

1. **Read from the file `[credentials]` names when the operation runs, and handed to each of its `gh` runs alone in `GH_TOKEN` (chosen).** Every `GH_*` and `GITHUB_*` variable of the host's is removed, and `GH_CONFIG_DIR` is an empty folder of the run's own. This is the way record 0033's consequences give a tool that takes a token only in a variable.
2. **`gh auth login --with-token` into the run's folder.** Not taken: it writes the token to disk.
3. **A shell handed the path, reading it into `GH_TOKEN`.** Not taken: a second program reads the file all the same, and the token lives in its variables and command line.
4. **The host's own `gh` sign-in.** Not taken: it is the person's, with their scopes, shared with every `gh` they run.

**The tag.**

1. **The core hands it with the verified version (chosen).** `Verified` names the version's tag, `<prefix><version>`, which the core already names from the component's prefix. 90 hands a target "a version the reconciler has verified", and the version's name is its tag (22).
2. **GitHub's tags listed and each resolved in the store as the component's version.** Not taken: a read per tag, to learn from a forge what the core knows.
3. **A `prefix` in the table.** Not taken: it restates the component's prefix and can drift from it.

**The assets.** The table's `assets` names the component's outputs, each uploaded under its output's name, the name GitHub keeps, so a name GitHub would rename is refused in the table. `assets` may be empty. An asset counts as held where the release has one of its name, uploaded, with its bytes' digest; any other is uploaded with `--clobber`. None is removed. Not taken: a map from each asset's name to an output, a second name for one thing; every `file` artifact of the version, which would release a file the component never meant to; a marker of what was uploaded, since a release holds one version.

**How a release is made.**

1. **A draft, its assets uploaded, then published marked latest (chosen).** The tag is checked first: GitHub must hold it at the revision the index names. The draft is found by its tag by the next apply where one stopped. A reader of the latest release never sees one without its assets, and a repository with immutable releases takes the assets before they lock.
2. **Published first, then its assets.** Not taken: the latest release is briefly without its assets, and an immutable release takes none.

**Title and mark.** Every release is titled by its tag, as flume's are, is never a prerelease, and is marked latest, so current reads it. Not taken: a semantic prerelease made a GitHub prerelease, which GitHub never marks latest, so current could never read it, the core would never sign its report, and every apply would run again; the notes' first heading as the title, which ties the title to the repository's template.

**current.** The release GitHub marks latest, its tag resolved in the store as the environment's component's version name to the index digest. Unread where none is marked, or where the tag names no version of the component. report's `converged_at` is the release's `updated_at`, which orders nothing. Not taken: the release of the head's tag, which says a release exists and not which one GitHub serves.

## Decision outcome

Option 1 of each, as the change's design D2 to D9 gives them. Rule core.21 says how the token reaches a tool that takes no file, and rule core.79 holds the bound on the tag and the assets.

### Consequences

- Good: no run makes, moves or deletes a tag, so a release is only ever of the revision the version names.
- Good: the token reaches `gh` alone, for one operation, and the host's own `gh` sign-in is never read.
- Good: what an environment runs is readable on GitHub with no timber process running (20).
- Bad: GitHub marks one release latest per repository, so only one component of a repository can take a GitHub release.
- Bad: a version from a channel of prereleases becomes a release marked latest.
- Bad: the reads of a missing release or tag rest on `gh`'s wording, `release not found` and `gh: Not Found (HTTP 404)`. A rewording makes the read unreadable, before anything is written.
- Bad: a process of the same user can read the token in `gh`'s environment while it runs, as it can read the token's file.
