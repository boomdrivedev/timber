---
number: 44
title: The Nix cache target pushes to Cachix by the cache's name with the host's credential read at use, and to Attic with attic's own configuration
status: accepted
date: 2026-11-18
decision-makers:
- the construct stage of timber-pushes-to-cachix
---

# The Nix cache target pushes to Cachix by the cache's name with the host's credential read at use, and to Attic with attic's own configuration

Status: accepted on 2026-11-18 as the construct stage's own choice, made on the rules written and cheap to change. The statement fixes what the Nix binary cache target does, and this record fixes how it does it on Cachix.

Deciders: the construct stage of the change `timber-pushes-to-cachix`, writing its design (D1 to D7).

Sources: `boomdrivedev/blueprints:design/timber/requirements.md` C.12, 45, 64 and 118, at 863a511; the architecture page's "Where a release goes", at 863a511; record 0033 and its consequences; record 0043; rule core.21; the Nix cache target's change, `2026-11-05-the-nix-cache-target`, its design D7; Cachix's CLI, `cachix` 1.11.1 as nixpkgs gives it, its help and its answers read on 2026-11-18; Cachix's API answer for a public cache, `GET https://cachix.org/api/v1/cache/<name>`; Boomdrive's workflow `cache.yml` at its main.

## Context and problem statement

C.12's row for a Nix binary cache names two caches, Attic and Cachix: push the version's store paths to the cache, with the cache's own credential. The Nix cache target, `nix_cache`, pushes to Attic alone. Its design (D7) left Cachix to "a later key of the same table". The architecture page has flume's and Boomdrive's releases pushed to Cachix for three systems, as Boomdrive's workflow pushes them today with `cachix push boomdrive` and a token in `CACHIX_AUTH_TOKEN`.

What `cachix` 1.11.1 settles:

- It takes its token from `CACHIX_AUTH_TOKEN`, else from its configuration file, `~/.config/cachix/cachix.dhall`, or the file `--config` names. It reads no token file.
- With `--config` naming a file that does not exist, and no token in its variable, `cachix push` refuses with "Neither auth token nor signing key are present." With the variable, it pushes. It makes no file at that path.
- `cachix push <name> <paths…>` pushes each path with its closure, skipping what the cache holds.
- A cache whose owner holds its signing key also wants `CACHIX_SIGNING_KEY`. A cache Cachix signs wants the token alone.
- A cache is named on Cachix by a name of lowercase letters, digits and `-`, the label of its address, `https://<name>.cachix.org`.

## Considered options

**One profile or two.**

1. **A second cache of the one `nix_cache` profile (chosen).** The table names its cache by `attic` or by `cachix`. The closures' reading, the check that this host holds every root, and the answers of a cache that keeps no marker stay one profile's.
2. **A `cachix` profile of its own.** Not taken: it would repeat all of that, and C.12 has one row for both caches.

**Cachix's token.**

1. **The host's `[credentials]`, read when apply pushes and handed to that one run of `cachix` alone in `CACHIX_AUTH_TOKEN` (chosen).** The table's `credential` names it. Every `CACHIX_*` variable of the host's is removed, and `--config` names a file in an empty folder of the run's own. This is the way record 0033's consequences and record 0043 give a tool that takes a token only in a variable.
2. **Cachix's own configuration on the host.** Not taken: `~/.config/cachix/cachix.dhall` is the person's, written by `cachix authtoken` and read by their own `cachix` and devenv. It holds one token, which writes every cache its owner holds, not the cache's own.
3. **A `cachix.dhall` the host's `[credentials]` names, handed by `--config`.** Not taken: the file holds more than the credential, in the tool's own form, where 118 names a file holding only it.
4. **`cachix authtoken` into the run's folder.** Not taken: it writes the token to disk.

**Attic's token.**

1. **attic's own configuration on the host, as it is (chosen).** attic names its server by a name its configuration gives, with the server's address beside the token. The configuration is the environment's own place, as record 0033 accepted.
2. **Attic moved to `[credentials]` too.** Not taken here: the table would have to name the server's address, which attic's configuration holds today. It is a change of its own, if one cache's way is to serve both.

**The push.**

1. **`cachix --config <folder>/cachix.dhall push <name> <roots…>` (chosen)**, with cachix's own defaults for compression and jobs, at `cachix.org`.
2. **`nix copy --to https://<name>.cachix.org`.** Not taken: Cachix takes uploads only through its own API, which its client speaks.

**A cache's own signing key.** Not taken: a second credential for `CACHIX_SIGNING_KEY`. A cache whose owner holds its signing key refuses the push in cachix's words. A later key of the table takes it when a cache needs it.

## Decision outcome

Option 1 of each, as the change's design D2 to D7 give them. Rule core.21 says how the token reaches `cachix`.

### Consequences

- Good: the cache's own token, which writes that cache alone, reaches `cachix` for one push. The person's own `cachix` configuration and the host's `CACHIX_*` variables are never read.
- Good: an apply of a version already pushed uploads nothing, as attic's does.
- Good: one host can push the same version to both caches, through two environments of one channel.
- Bad: the two caches take their tokens two ways, attic from its configuration and Cachix from the host's file.
- Bad: a cache whose owner holds its signing key can't be pushed to.
- Bad: a process of the same user can read the token in `cachix`'s environment while it runs, as it can read the token's file.
- Bad: the proof against Cachix reaches a hosted service with a token a person makes, so it runs by hand (rule core.38).
