## Context

C.12's row for a Nix binary cache, at `boomdrivedev/blueprints` 863a511,
reads: a Nix binary cache (Attic, Cachix); push: the version's store
paths pushed to the cache, with the cache's own credential. The
architecture page has flume's and Boomdrive's releases pushed to Cachix
for three systems, and marks the built cache target as filling Attic
only.

What is built that this change stands on:

- The Nix cache target, `profiles/target/nix_cache/`, bound as
  `nix_cache`. Its table, `table.rs`, names `attic = "<server>:<cache>"`
  and `closures`, and nothing else. apply reads the version's index and
  each closure's file through the store by digest (`nix_closure`,
  `file_artifact`), checks every root valid in this host's Nix store
  with `nix-store --check-validity --print-invalid`, and runs `attic
  push <server>:<cache> <roots…>`. attic reads the server's address and
  the token from its own configuration on the host. current and report
  are unread: a cache keeps no marker. Its design D7 left another cache
  to "a later key of the same table".
- The host's `[credentials]`: each key a credential's name, each value
  the path of a file holding only that credential. The family's `bound`
  hands them to the Dash0 target, and not yet to the Nix cache target.
- Record 0033 gives a target's credential: the host's `[credentials]`
  names a file, and the target hands its path to its own tool. Its
  consequences add that a tool taking a token only in a variable is
  handed it by the target, which reads the file at the moment of use for
  that one run (64).
- The bolt's other change, `timber-publishes-a-github-release`, planned
  on its own branch and built before this one (D1). It gives the run
  module variables set for one run, `Verified` its tag, and rule core.21
  the token read at use for a tool that takes no file (record 0043). Its
  GitHub release target reads its credential's file at use, trims it,
  and answers a name the host lacks, a file it cannot read and a file of
  whitespace alone as unreadable, each naming the credential.
- Boomdrive's workflow `cache.yml` builds every package for three
  systems on GitHub's runners and pushes them with `cachix push
  boomdrive`, its token in `CACHIX_AUTH_TOKEN`. The cache is signed by
  Cachix, so no signing key is handed.

Cachix's CLI, `cachix` 1.11.1 as nixpkgs gives it, read on 2026-11-18:

- It takes its token from `CACHIX_AUTH_TOKEN`, else from its
  configuration file: `~/.config/cachix/cachix.dhall`, or the file
  `-c`/`--config` names. It reads no token file. `cachix authtoken`
  writes the token into that file.
- `cachix --config <path> push <name> <paths…>` with no file at `<path>`
  and no `CACHIX_AUTH_TOKEN` refuses: "Neither auth token nor signing
  key are present." With the variable it pushes, and it makes no file
  at `<path>`.
- `push` pushes each path with its closure, skipping what the cache
  holds. A cache whose owner holds its signing key also wants
  `CACHIX_SIGNING_KEY`. Its errors redact the token.
- A cache unknown, or one the token may not write, is refused in
  cachix's words: "Binary cache <name> doesn't exist or you're not
  authorized to access it."
- A cache's name is the label of its address,
  `https://<name>.cachix.org`. Cachix's API answers a public cache's
  keys with no token: `GET https://cachix.org/api/v1/cache/<name>`
  gives `publicSigningKeys`.

## Goals / Non-Goals

**Goals:**

- A version's closures pushed to a Cachix cache from the host that holds
  them, with the cache's own token, as they are pushed to Attic (C.12).
- The token reaching one run of `cachix` alone, with nothing of the
  host's own Cachix settings read (45, 64, rule core.21).
- One reading of a token at use for every target whose tool takes no
  file.
- The table's refusals, the run's arguments and environment, and the
  target group checked in `check-fast`, with a proof against Cachix by
  hand.

**Non-Goals:**

- A cache whose owner holds its signing key.
- Attic's token through `[credentials]`.
- Any Cachix but `cachix.org`, and Cachix's own features beyond push:
  pins, deploys, `watch-store`.

## Decisions

### D1. This change is built after the GitHub release change

The bolt carves two changes, and both reach the same code: the run
module, the `Verified` every target's tests make, rule core.21, and the
host-bindings requirement that no binding opens a credential's file.
The GitHub release change, planned first, changes each of them. This
change takes them as that change leaves them and writes none of its
own: its first task holds the branch to the bolt with that change in,
and stops short where the bolt doesn't hold it.

Not taken: this change writing its own variables for a run, its own
`Verified` passes and its own amendment of core.21. Each would be
written twice, on two branches, and the second to land would meet the
first's in a conflict over every caller of the run module.

### D2. Cachix is a second cache of the one `nix_cache` profile

The profile keeps its name, its module and its binding. The table names
which cache it pushes to. The closures' reading, the check that this
host holds every root, and the unread current and report stay one
profile's, for both caches. Only the push and its credential differ.

Not taken: a `cachix` profile of its own. It would repeat the closures'
reading, the held check and the no-marker answers, or call the Nix cache
profile for them (rule core.2). C.12 gives both caches one row.

### D3. The table names its cache once

```toml
[environments.boomdrive.cachix-x86_64-linux]
channel = "beta"
target.nix_cache = { cachix = "boomdrive", credential = "cachix-boomdrive", closures = ["boomdrive-x86_64-linux"] }
```

The table names exactly one cache:

- `attic`, the cache as attic's own command names it,
  `<server>:<cache>`, as it is today; or
- `cachix`, the cache's name on Cachix: one or more lowercase letters,
  digits and `-`, beginning with no `-`, which cachix would read as an
  option. It is the name `cachix push` takes, never the cache's address.
  With it, `credential`: the name of the host's credential, non-empty,
  with no whitespace.

`closures` is as it is today. Each refusal is unreadable at every
operation that reads the table, naming the environment and the key, and
nothing runs (45):

- any key but `attic`, `cachix`, `credential` and `closures`, as
  `token` is today;
- both `attic` and `cachix`, naming `cachix`;
- neither, naming the environment's `nix_cache` table;
- a `credential` beside `attic`, saying attic reads its token from its
  own configuration on the host;
- a `cachix` with no `credential`;
- a `cachix` or `credential` of another form, saying which form it
  takes.

The table reads into a `Cache`: attic's `<server>:<cache>`, or Cachix's
name with the credential's name. A cache names itself in the answers as
`fleet:swancloud`, as today, or as `boomdrive on Cachix`.

Not taken:

- `cachix` naming the cache's address. The command takes the name, and
  the address is `cachix.org`'s to give.
- `credential` optional for Cachix, falling back to the host's own
  cachix configuration. That configuration is the person's (D4).
- A `server` key beside a `cache` key, one form for both caches. attic
  names its server by a name its own configuration gives, and Cachix has
  one server. The two would share a key and mean different things.

### D4. Attic keeps its own configuration, and Cachix takes the host's credential

attic names its server by a name its configuration gives, and that
configuration holds the server's address beside the token. It is the
environment's own place, as record 0033 accepted. It stays as it is.

Cachix's configuration, `~/.config/cachix/cachix.dhall`, is the
person's. `cachix authtoken` writes it, and the person's own `cachix`
and devenv read it. It holds one token, which writes every cache its
owner holds, not the cache's own. So a Cachix cache takes the host's
`[credentials]`, as record 0033 gives every target, by the name the
table's `credential` gives it: a file holding a token that may push to
that cache alone.

Not taken:

- Cachix's own configuration on the host, as attic's. It is the
  person's, with a token for every cache they own.
- A `cachix.dhall` the host's `[credentials]` names, handed by
  `--config`, so the tool alone reads it. The file holds more than the
  credential, in the tool's own form, where 118 names a file holding
  only it.
- Attic moved to `[credentials]` too, so both caches take a token one
  way. Its table would have to name the server's address, which attic's
  configuration holds today, and swancloud's hosts keep that
  configuration. It is a change of its own, if one way is to serve both.

### D5. The token reaches one run of `cachix` alone

`cachix` reads no token file. So, as record 0033's consequences and
record 0043 give it, apply reads the file the credential names and
hands the token to the push's one run of `cachix`, in order:

1. The table is read. The credential's name is looked up in the host's
   `[credentials]`, the file never opened. A name the host's file lacks
   is unreadable naming the credential and the bindings file, before the
   store is asked anything.
2. The index, the closures and their roots are read, and every root is
   checked held, as for attic.
3. The file is read and its trailing whitespace trimmed. A file that
   can't be read is unreadable naming the credential and the path, with
   the operating system's words. A file of whitespace alone is
   unreadable naming the credential.
4. A folder of the operation's own is made under the system's temporary
   directory, empty, and removed when apply answers, whatever cachix
   answered.
5. `cachix` runs with every `CACHIX_*` variable of timber's own
   environment removed, `CACHIX_AUTH_TOKEN` set to the token for that
   run alone, and `--config <folder>/cachix.dhall`, a file that is never
   made. So no token, signing key or configuration of the host's is
   read, and the person's own `cachix.dhall` is never opened.

The token is held in the operation's memory until apply answers. It is
in no argument, no answer and no file, and its type prints none of it.
current, report and desired read no credential: a cache's current and
report answer that it keeps no marker, whatever the host's file names.

This is how rule core.21 already reads once the GitHub release change
has amended it. Its details gain `cachix`'s variables and configuration
among what is ruled out, and the Nix cache target's reading of its
Cachix token among what is not.

Not taken:

- `cachix authtoken --stdin` into the run's folder, fed the token. It
  writes the token to disk.
- The config file fed on standard input, `--config /dev/stdin`. The
  token would be written in Dhall by timber, and the push would rest on
  how cachix opens a path.
- The token read when apply starts. It would be held through the
  store's reads and the held check, and read for an apply refused before
  the push.

### D6. One reading of a token at use, in the target family

The reading in D5's steps 1 and 3 is the GitHub release target's too. It
moves into a family module, `profiles/target/token.rs`, beside
`file_artifact.rs` and `nix_closure.rs`, which both targets call, so
none calls another (rule core.2). It holds:

- the lookup of a credential's path, by its name, in the host's
  credentials, never opening the file;
- the read of the file at use into a `Token`, trimmed, whose `Debug`
  prints no token, with the three unreadable answers as D5 words them.

Each answer names the credential as the asking target names it, by the
environment and its component. The GitHub release target's answers are
unchanged by the move.

Not taken: the Nix cache target reading the file itself. It would write
the GitHub release target's reading a second time, with the same three
refusals in words that could drift apart.

### D7. The push to Cachix

```sh
cachix --config <folder>/cachix.dhall push boomdrive <root> <root>…
```

Every root of every closure the table names, once each, as attic is
handed them. cachix pushes each with its closure and skips what the
cache holds, so an apply of a version already pushed uploads nothing.
It runs at `cachix.org`, with cachix's own defaults for compression,
chunks and jobs. An exit other than 0 is unreadable, naming the command
run and carrying cachix's stderr verbatim (59, rule core.19). The
command's words hold no token, since none is an argument.

A cache whose owner holds its signing key refuses the push in cachix's
words. A later key of the table takes `CACHIX_SIGNING_KEY` when a cache
needs it.

current and report are unread as for attic, naming the cache as
`boomdrive on Cachix`: "the Nix binary cache boomdrive on Cachix holds
store paths by their hash and keeps no marker of what was pushed to it"
(119).

The target runs `cachix` as found on `PATH`, as it runs `attic`. Where a
host runs `timber apply` in a repository's shell, that is the
repository's own `cachix`.

Not taken:

- `nix copy --to https://<name>.cachix.org`. Cachix takes uploads only
  through its own API, which its client speaks.
- `--hostname` in the table. There is one Cachix.
- `--omit-deriver`, compression or job flags. Cachix's defaults are the
  cache's own preferred method, and nothing in C.12 asks otherwise.

### D8. The target's tests over a stand-in cachix

`nix_cache/tests.rs` already stands in for attic and nix-store with
shell scripts the test writes. A stand-in `cachix` joins them. It records
its arguments, its environment, and whether the file its `--config`
names exists and what its folder holds, and fails with words a test
writes. Over it, in `check-fast`:

- the table's refusals for the Cachix cache;
- the credential's lookup and read: a name the host lacks, a missing
  file, a file of whitespace, each before cachix runs, and the first
  before nix-store or the store is asked;
- the run: `--config`, `push`, the name and every root; the host's
  `CACHIX_AUTH_TOKEN` and `CACHIX_SIGNING_KEY` set in timber's
  environment and absent from the run's, or replaced; the token in
  `CACHIX_AUTH_TOKEN` and in no argument; no file at the config path,
  its folder empty and under the system's temporary directory, and gone
  after;
- cachix's words carried on a failure;
- current and report unread naming the Cachix cache, with no credential
  named;
- the target group over a rig of the profile naming a Cachix cache, its
  target holding no marker, its foreign settings a `token` key, the
  version published into the fake store.

### D9. The proof against Cachix, by hand

`crates/timber/tests/cachix.rs` sits behind the `tools` feature. It runs
under the `cachix` task of `crates/timber/moon.yml`, tagged
`check-cachix`, with `$TIMBER_TOOLS_CACHIX` among its inputs. The
setting is `<cache>#<token file>`: a throwaway public cache on Cachix,
and a file holding a token that may push to it. A run without the
setting, or with one of another form, fails naming it. With it, the run
takes these steps:

1. Read the cache's public signing key from Cachix's API, `GET
   https://cachix.org/api/v1/cache/<cache>`, with no token. A cache that
   answers no key fails the run, saying the proof's cache is public.
2. Build, with the host's Nix, a closure of two store paths, a root
   whose output names a leaf, the leaf holding a nonce of the run's own
   so the cache holds neither, as `tests/nix_cache.rs` builds its
   closure. Publish it into a layout store, the root's line packed by
   the file projection, and make `Verified` as the core makes it.
3. Give the test's own environment wrong `CACHIX_AUTH_TOKEN` and
   `CACHIX_SIGNING_KEY` values. Bind credentials naming `cachix-proof`
   at the token file, and apply through the profile, running the
   `cachix` that `TIMBER_TOOLS_CACHIX_CLI` names.
4. In a fresh Nix store in the test's directory, with the cache's
   address as its only source and its key as the only key trusted,
   realise the root, and check that the store holds the leaf.
5. Apply again, and check it converges.
6. Run the target group over a rig of the profile naming the same
   cache.

The paths the run pushed stay in the cache until Cachix's own retention
collects them. They are two small paths per run, and the cache is the
operator's throwaway.

`TIMBER_TOOLS_CACHIX_CLI` is nixpkgs' `cachix` as `devenv.lock` pins it,
which `devenv.nix` names without putting it on the shell's `PATH`. No
gate task runs the proof, since its token is a file a person makes and
it reaches a hosted service (rule core.38).

Not taken: a Cachix of the tests' own in the tools layer. There is none
to run, and the stand-in program proves what timber hands `cachix`.

### D10. Records

Record 0044 holds D2 to D5 and D7, each with the path not taken, citing
the statement at 863a511 and records 0033 and 0043. Rule core.21's
details gain D5: `cachix` run with the host's `CACHIX_*` variables or
its own configuration ruled out, and the Nix cache target reading its
Cachix token's file when apply pushes, for that one run, not ruled out.
The rule is amended once the bolt holds the GitHub release change's
amendment (D1), so the two land on one text.

## Risks / Trade-offs

- **The token in a run's environment.** A process of the same user can
  read it there while `cachix` runs. → That user can read the token's
  file too. The token reaches no other program, no argument and no
  file.
- **cachix's wording.** Nothing is parsed from cachix's words: a failure
  is unreadable with them, whatever they say.
- **A cache Cachix collects by age.** A released closure Cachix has
  collected is built again by the machine that misses it (54). The
  cache's retention is its owner's.
- **One host holds one system's closures.** apply pushes only what the
  host that applies holds (the Nix cache target's design D8). → A
  repository releasing for three systems declares a cache environment
  per system, each applied from a host of that system.
- **The spec's Purpose names attic alone.** A delta cannot change a
  spec's Purpose, and rule core.77 lets only a placeholder Purpose be
  written over directly. The requirements say Cachix; the Purpose stays
  until a ruling says otherwise.
- **The bolt's order.** Where the bolt doesn't hold the GitHub release
  change when this change's build begins, the build stops short at its
  first task rather than writing that change's work again.

## Migration Plan

Boomdrive adopts it in its own unit: its policy file declares the
component with a closure output per system, and a cache environment per
system naming `cachix = "boomdrive"` and the host's credential; each
system's host applies its own. Its `cache.yml` push retires once
`timber apply` pushes the same paths. flume follows the same way. The
host's `[credentials]` gains the cache's token through Boomdrive's
timber module (record 0035). Nothing in timber migrates: every `attic`
table reads as it did.

## Open Questions

None.
