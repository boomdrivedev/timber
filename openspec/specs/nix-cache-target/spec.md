# nix-cache-target Specification

## Purpose
The target profile bound as `nix_cache` (C.12): the closures an
environment's version names pushed, each root with its closure, to a Nix
binary cache through attic, only from the host that holds them and with
the environment's own credential; a cache keeps no marker, so current
and report answer unreadable (42, 45, 119, rule core.70).

## Requirements

### Requirement: The Nix cache target is a target profile of its own

The Nix cache target SHALL be a target profile under
`crates/timber/src/profiles/target/nix_cache/`, bound by the profile
name `nix_cache`, implementing the target interface and nothing of the
core (C.12, rules core.1, core.2). It SHALL read the version's index and
its artifacts through a store binding of its own, opened and never
made, and SHALL name attic, Cachix, Nix and store paths in no source
outside `profiles/target/`.

#### Scenario: Bound by its profile name

- **WHEN** a repository's environment names `target.nix_cache = { attic = "fleet:swancloud", closures = ["swancloud-srv"] }`
- **THEN** apply of that environment reaches the Nix cache target, handed the environment with those settings

#### Scenario: A Cachix cache bound by the same profile

- **WHEN** a repository's environment names `target.nix_cache = { cachix = "boomdrive", credential = "cachix-boomdrive", closures = ["boomdrive-x86_64-linux"] }`
- **THEN** apply of that environment reaches the same Nix cache target, handed the environment with those settings

### Requirement: The target's table names the cache and the closures, and nothing else

The environment's `nix_cache` table SHALL name its cache once, by
`attic`, the cache as attic's own command names it, `<server>:<cache>`,
or by `cachix` with `credential`, and `closures`, one or more output
names whose artifacts name closures. Any other key, both caches or
neither, an empty `closures`, or an `attic` that is not
`<server>:<cache>` SHALL be unreadable naming the environment and the
key, and nothing SHALL be pushed; so no credential sits in the table
(45, 58, rule core.21).

#### Scenario: A token in the table

- **WHEN** the environment's `nix_cache` table names `attic`, `closures` and `token`
- **THEN** apply answers unreadable naming the environment and `token`, and attic is not run

#### Scenario: A cache with no server

- **WHEN** the table names `attic = "swancloud"`
- **THEN** apply answers unreadable naming `attic` and saying it is `<server>:<cache>`

#### Scenario: Both caches named

- **WHEN** the table names `attic = "fleet:swancloud"`, `cachix = "boomdrive"`, `credential` and `closures`
- **THEN** apply answers unreadable naming `cachix` and saying the table names one cache, and neither attic nor cachix is run

#### Scenario: No cache named

- **WHEN** the table names `closures` alone
- **THEN** apply answers unreadable naming the environment's `nix_cache` table and saying it names no cache, by `attic` or by `cachix`

#### Scenario: A credential beside attic

- **WHEN** the table names `attic = "fleet:swancloud"`, `credential = "attic-fleet"` and `closures`
- **THEN** apply answers unreadable naming `credential` and saying attic reads its token from its own configuration on the host, and attic is not run

### Requirement: A closure's artifact names its roots

A closure SHALL be named by the artifact of an output its component
declares, packed by the file projection as the output's one file: one
absolute store path per line, each a root of the closure, as `nix build
--print-out-paths` prints it (C.11, C.12). The store SHALL hold the
artifact and no store path of the closure; the paths are the cache's
(section 10).

#### Scenario: A machine's closure

- **WHEN** the artifact of `swancloud-srv` at `0.4.0` holds one line, the machine's top-level store path
- **THEN** the closure's roots are that one store path

### Requirement: The closure's form is read in one place

The closure's form SHALL be read by one module the Nix target profiles
share, `profiles/target/nix_closure.rs`, so no target profile calls
another (rule core.2). A file with no line, an empty or repeated line,
or a line that is no store path, `<store dir>/<32 characters of Nix's
base32>-<name>`, SHALL be refused naming the artifact and the line.

#### Scenario: A line that is no store path

- **WHEN** a closure's artifact holds a line `result`
- **THEN** reading it is refused naming the artifact and `result`, and nothing is pushed

### Requirement: desired is the channel's head

`desired` SHALL answer the head the core hands it, and absent naming the
channel where the core answers the channel empty (88, 31).

#### Scenario: The head handed

- **WHEN** desired is asked with a head that answers index `i3`
- **THEN** it answers `i3`

### Requirement: Apply reads each closure the version names by digest

`apply` SHALL read the verified version's index through the store by
digest, find for each output `closures` names the file projection's
artifact of that name, and read its bytes by digest (42, rule core.4);
an output the index carries no such artifact of SHALL be refused naming
the output and the version.

#### Scenario: A closure output the version lacks

- **WHEN** `closures` names `swancloud-srv` and the version's index carries no artifact of it
- **THEN** apply is refused naming `swancloud-srv` and the version, and attic is not run

### Requirement: Apply pushes only from the host that holds the closure

Before anything is pushed, every root SHALL be checked valid in this
host's Nix store; a root it does not hold SHALL refuse the apply naming
the root and the host, and nothing SHALL be pushed. Apply SHALL never
build, substitute or copy a root into this host's store (15, I3).

#### Scenario: A root this host does not hold

- **WHEN** the closure's root is not valid in this host's Nix store
- **THEN** apply is refused naming the root and the host, attic is not run, and nothing is built or fetched

### Requirement: Apply pushes each root with its closure through attic

Where the table names `attic`, once every root is held, apply SHALL run
`attic push <server>:<cache>` with every root, which pushes each root
with its closure; attic's own words on a failure SHALL be the answer,
unreadable, verbatim (59).

#### Scenario: Pushed with its closure

- **WHEN** the table names `attic`, and the root is valid in this host's store and refers to one other store path
- **THEN** after the apply the cache holds both store paths

### Requirement: The push uses the environment's own credential

Where the table names `attic`, the push SHALL be run with the credential
attic reads from its own configuration on the host, at the push, the
environment's own place (45). timber SHALL pass no token on attic's
command line or in its environment, SHALL read nothing of attic's
configuration, and SHALL keep nothing of it (rule core.21).

#### Scenario: The token stays attic's

- **WHEN** apply pushes to an attic cache whose token is held only in attic's own configuration
- **THEN** the push succeeds, and neither attic's command line, its environment as timber set it, nor anything timber wrote holds the token

### Requirement: current and report are unread, since a cache keeps no marker

A Nix binary cache holds store paths by their hash and keeps no name a
marker could be held under, so `current` and `report` SHALL answer
unreadable, in words that say so and name the cache, never a digest and
never converged (119, 89, 98).

#### Scenario: Current after an apply

- **WHEN** a version is applied and current is asked
- **THEN** current answers unreadable saying the cache keeps no marker of what was pushed, not the digest applied

### Requirement: The profile passes the target group

The profile SHALL pass the target group with a rig declaring that its
target holds no marker, its foreign settings a `token` key (93).

#### Scenario: The group over the profile

- **WHEN** the target group runs over the Nix cache target's rig
- **THEN** every scenario passes, and "converged and reported" is reported not applicable

### Requirement: A closure built on one host is realised on another from the cache alone

A tools-layer test SHALL push, through the profile, a closure of two
store paths built by the test's host and named by a published version's
artifact, with the token in attic's own configuration alone, to an attic
server started for the test; then a fresh Nix store, trusting only the
cache's key, SHALL realise the root and its leaf from that cache alone
(C.12).

#### Scenario: Realised from the cache alone

- **WHEN** the version is applied to the cache environment from the host that built it
- **THEN** a fresh Nix store with the cache as its only source realises the root, and holds the leaf it refers to

### Requirement: A Cachix cache is named by its name, with the host's credential

Where the table names `cachix`, it SHALL be the cache's name on Cachix,
one or more lowercase letters, digits and `-`, beginning with no `-`,
and `credential` SHALL name the host's credential, a non-empty name with
no whitespace, which the host's `[credentials]` maps to a file holding a
token that may push to that cache (118; record 0044). Either otherwise
SHALL be unreadable naming the environment and the key, and nothing
SHALL be pushed.

#### Scenario: Boomdrive's cache

- **WHEN** the table names `cachix = "boomdrive"`, `credential = "cachix-boomdrive"` and `closures = ["boomdrive-x86_64-linux"]`
- **THEN** the table is read as the Cachix cache `boomdrive`, the credential `cachix-boomdrive` and the one closure output

#### Scenario: A cache's address in place of its name

- **WHEN** the table names `cachix = "https://boomdrive.cachix.org"`
- **THEN** apply answers unreadable naming `cachix` and saying it is the cache's name on Cachix, and cachix is not run

#### Scenario: Cachix with no credential

- **WHEN** the table names `cachix = "boomdrive"` and `closures`, and no `credential`
- **THEN** apply answers unreadable naming `credential`, and cachix is not run

### Requirement: Apply pushes each root with its closure through cachix

Where the table names `cachix`, once every root is held and the token is
read, apply SHALL run `cachix --config <folder>/cachix.dhall push
<name>` with every root, which pushes each root with its closure,
skipping what the cache holds; cachix's own words on a failure SHALL be
the answer, unreadable, verbatim (59; record 0044).

#### Scenario: Pushed to Cachix

- **WHEN** the table names `cachix = "boomdrive"`, and every root is valid in this host's store
- **THEN** cachix is run once, after nix-store's check, with `--config`, `push`, `boomdrive` and every root, and apply answers converged

#### Scenario: Cachix refuses the push

- **WHEN** cachix fails with the words `Binary cache boomdrive doesn't exist or you're not authorized to access it.`
- **THEN** apply answers unreadable carrying those words verbatim

### Requirement: The Cachix token is the host's, read at the moment of use

Apply SHALL look up the table's `credential` in the host's
`[credentials]` before it reads the version, and SHALL read the file it
names only once every root is held, just before the push, trimming
trailing whitespace. A name the host's file lacks, a file that cannot
be read, or one of whitespace alone SHALL be unreadable naming the
credential, and nothing SHALL be pushed. current and report SHALL read
no credential (45, 64; record 0044).

#### Scenario: A credential the host does not name

- **WHEN** the table names `credential = "cachix-boomdrive"`, and the host's bindings file names no such credential
- **THEN** apply answers unreadable naming `cachix-boomdrive` and the bindings file, and neither the store, nix-store nor cachix is asked

#### Scenario: A token file that cannot be read

- **WHEN** the host's `[credentials]` names `cachix-boomdrive` at a path where no file is
- **THEN** apply answers unreadable naming `cachix-boomdrive` and the path, with the operating system's words, and cachix is not run

#### Scenario: A token file of whitespace alone

- **WHEN** the file the credential names holds only a newline
- **THEN** apply answers unreadable naming `cachix-boomdrive`, and cachix is not run

#### Scenario: current reads no credential

- **WHEN** the host's bindings file names no `cachix-boomdrive`, and current is asked of the environment
- **THEN** current answers unreadable saying the cache keeps no marker of what was pushed to it

### Requirement: The token reaches one run of cachix alone

The token SHALL be handed to the push's run of `cachix` alone, in
`CACHIX_AUTH_TOKEN`, held in the operation's memory and written
nowhere, in no argument and no answer. The run SHALL have every
`CACHIX_*` variable of timber's own environment removed, and `--config`
naming a file in an empty folder the operation makes and removes after,
so no token, signing key or configuration of the host's is read (rule
core.21; record 0044).

#### Scenario: The host's own cachix settings

- **WHEN** timber's environment holds `CACHIX_AUTH_TOKEN` and `CACHIX_SIGNING_KEY` of the host's, and apply pushes to `boomdrive`
- **THEN** the run's `CACHIX_AUTH_TOKEN` is the token the credential's file holds, the run has no `CACHIX_SIGNING_KEY`, its `--config` names a file that does not exist in an empty folder under the system's temporary directory, and no argument holds the token

#### Scenario: The folder is removed

- **WHEN** apply pushes to `boomdrive`, and cachix succeeds or fails
- **THEN** the folder its `--config` named no longer exists once apply answers

### Requirement: A closure is pushed to Cachix and realised from it alone, by hand

A test behind the `tools` feature, run by hand under `check-cachix`,
SHALL push through the profile a closure of two store paths new to a
throwaway public cache the operator names in `TIMBER_TOOLS_CACHIX`, its
token in a file the host's credentials name; then a fresh Nix store,
trusting only the cache's key, SHALL realise the root and its leaf from
that cache alone. It SHALL refuse to run without the setting, naming it
(C.12; rule core.38).

#### Scenario: Realised from Cachix alone

- **WHEN** the version is applied to the Cachix environment from the host that built it, with wrong `CACHIX_AUTH_TOKEN` and `CACHIX_SIGNING_KEY` values in the test's own environment
- **THEN** a fresh Nix store with the cache as its only source realises the root, and holds the leaf it refers to

#### Scenario: No cache named

- **WHEN** `moon run cli:cachix` runs with `TIMBER_TOOLS_CACHIX` unset
- **THEN** it fails naming `TIMBER_TOOLS_CACHIX`, and nothing is built or pushed
