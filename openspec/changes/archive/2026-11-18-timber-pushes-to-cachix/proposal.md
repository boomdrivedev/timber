## Why

C.12's row for a Nix binary cache names two caches, Attic and Cachix:
push the version's store paths to the cache, with the cache's own
credential (`boomdrivedev/blueprints` `design/timber/requirements.md`
at 863a511). timber's Nix cache target, `nix_cache`, pushes to Attic
alone. The architecture page has flume's and Boomdrive's releases pushed
to Cachix for three systems, which Boomdrive's own workflow does today
with `cachix push boomdrive` and a token in a variable, and it marks the
cache target as filling Attic only. This change builds the row's other
cache.

## What Changes

- **The Nix cache target pushes to Cachix as well as to Attic.** The
  `nix_cache` table names its cache once: `attic = "<server>:<cache>"`
  as now, or `cachix = "<name>"`, the cache's name on Cachix, with
  `credential`, the name of the host's credential. Both caches, neither,
  or a `credential` beside `attic` is unreadable naming the key (45).
  The closures' reading, the check that this host holds every root, and
  the answers of a cache that keeps no marker are the profile's, for
  both caches.
- **The push to Cachix** is `cachix --config <folder>/cachix.dhall push
  <name> <roots…>`, each root with its closure, skipping what the cache
  holds. cachix's own words on a failure are the answer, verbatim (59).
- **The Cachix token is the host's, read at the moment of use.** apply
  looks the credential up in the host's `[credentials]` before it reads
  the version. It reads the file only once every root is held, and
  hands the token to that one run of `cachix` alone in
  `CACHIX_AUTH_TOKEN`. Every `CACHIX_*` variable of the host's is
  removed, and `--config` names a file in an empty folder of the run's
  own, so the person's own `~/.config/cachix/cachix.dhall` is never read
  (45, 64, rule core.21; record 0044). current and report read no
  credential.
- **One reading of a token at use.** The GitHub release target's reading
  of its credential's file moves into a family module that both targets
  call, so the two read a token one way.
- **attic keeps its own configuration.** Its push and its token are as
  they were (record 0044 says why the two caches differ).
- **The suite runs the target group over the Cachix cache** with a
  stand-in `cachix` and `nix-store` in `check-fast` (93, 119).
- **The proof against Cachix is run by hand.** The tag `check-cachix`
  pushes a closure new to a throwaway public cache the operator names in
  `TIMBER_TOOLS_CACHIX`, realises it in a fresh Nix store from that
  cache alone, and runs the target group there. The run refuses without
  the setting, naming it. The token is a file a person makes, which no
  gate task reads (rule core.38). The devenv shell names nixpkgs'
  `cachix` in `TIMBER_TOOLS_CACHIX_CLI` (rule core.26).
- The site's targets page documents the Cachix cache. Record 0044 holds
  the choices, and rule core.21's details say how the token reaches
  `cachix`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `nix-cache-target`: the table names its cache by `attic` or by
  `cachix` with `credential`; the push through cachix; the token read at
  use and handed to one run alone, with nothing of the host's own cachix
  settings read; the proof against Cachix by hand.
- `conformance-suite`: the target group runs over the Nix cache target
  naming a Cachix cache.

## Out of scope, and what each waits on

- **This change builds on `timber-publishes-a-github-release`**, the
  bolt's other change: the run module's variables set for one run, the
  verified version's tag every target's tests pass, rule core.21 as
  record 0043 amends it, and the binding that opens no credential's
  file. Its build begins once the bolt holds that change (design D1).
- Boomdrive's and flume's adoption: each repository's
  `.config/timber.toml` naming its component, a closure output per
  system and a `target.nix_cache` environment per system on Cachix, the
  checks that build the closures (135), and the retirement of
  Boomdrive's `cache.yml` push. Each repository's own unit.
- The token and where `[credentials]` points on each host. swancloud
  declares it, and Boomdrive's timber module writes the host's file
  (record 0035).
- What runs `timber apply` for a cache environment: the host's, by C.16
  and rule core.52.
- A cache whose owner holds its signing key, which also wants
  `CACHIX_SIGNING_KEY`. A later key of the table, when a cache needs it.
- Attic's token through `[credentials]`. Its table would have to name the
  server's address, which attic's configuration holds today. A change of
  its own, if one way is to serve both caches.
- The architecture page's line that the cache target fills Attic only.
  It is blueprints', and changes when this change lands.

## Impact

- `crates/timber/src/profiles/target/nix_cache/` (the table, the push
  through cachix, the tests and the target group over a stand-in
  cachix), and `profiles/target/mod.rs`, which hands the Nix cache
  target the host's credentials.
- New: `crates/timber/src/profiles/target/token.rs`, the reading of a
  token at use, which the GitHub release target's reading moves into.
- New: `crates/timber/tests/cachix.rs` (by hand). `crates/timber/moon.yml`
  gains the `cachix` task, tagged `check-cachix`. `devenv.nix` gains
  `TIMBER_TOOLS_CACHIX_CLI`. No crate, dependency, language or file
  type is added.
- The site's targets page. Record 0044 in `docs/adr/`. Rule core.21's
  details in `docs/architecture/core.md`.
- Rules kept:
  - core.1: Cachix and `cachix` are named only under `profiles/target/`.
  - core.2: the closures are read through the family's `nix_closure`
    and `file_artifact`, the token through the family's `token`, and
    the store through the store seam.
  - core.4: the index and each closure are read by digest.
  - core.19: cachix's words are carried verbatim, and absent, refused
    and unreadable are kept apart.
  - core.21: the token is read at use, held for the push, handed to the
    one run of `cachix` alone, and written nowhere.
  - core.29: nothing is pushed before the core's judge.
  - core.38: the proof against Cachix runs by hand.
  - core.70: the closure's paths are the cache's, and the store holds
    the file naming its roots alone.
