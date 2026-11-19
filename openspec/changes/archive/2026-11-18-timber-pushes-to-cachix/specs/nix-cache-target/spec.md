## MODIFIED Requirements

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

## ADDED Requirements

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
