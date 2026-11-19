## Why

flume and Boomdrive are each to publish their versions through timber,
and each keeps a GitHub release under its version's tag: flume's binaries
for three systems, with git-cliff's notes as the release's text, made
today by a GitHub workflow of flume's own (the architecture page, "What
flume and Boomdrive need before step 2 can run"). The statement now has
the row: C.12 gains a GitHub release as a push target, the release of the
version's tag made or updated, its notes (134) as the release's text and
the artifacts the policy names as its assets, with the target's own
credential, and current the release GitHub marks latest
(`boomdrivedev/blueprints` `design/timber/requirements.md` at fecf4d9).
timber binds three targets, and none of them is a forge. This change
builds the row.

## What Changes

- **The verified version names its tag.** The core hands a target the
  version's name in the shared history beside its index and version: the
  component's prefix and the version, the tag the publish claimed (22,
  42, 90, 123). Every target and fake takes it, and the Nix and Dash0
  profiles ignore it.
- **The GitHub release target**, `crates/timber/src/profiles/target/github_release/`,
  bound as `github_release` (C.12's GitHub release row):
  - Its table is `target.github_release = { credential, assets }`.
    `credential` names the host's credential, a token that may write the
    repository's releases. `assets` names the component's outputs whose
    file artifacts are attached, each under its output's name, and may
    be empty. Any other key is unreadable, naming it (45).
  - The release is made in the checkout's repository on GitHub. An index
    naming another publishing repository, or none, or no revision, is
    refused before anything reaches GitHub.
  - The runs are `gh`'s, GitHub's own CLI, as found on `PATH`. `gh` takes
    a token only in a variable. So the target reads the credential's file
    at the moment of use and sets `GH_TOKEN` for each run of that
    operation alone, as record 0033's consequences give it. Every `GH_*`
    and `GITHUB_*` variable of the host's is removed, and each run's
    `GH_CONFIG_DIR` is an empty folder of its own, so no sign-in of the
    host's own `gh` is read (45, 64, rule core.21).
  - `apply` reads the index, the version's notes and each named output's
    file by digest (42, rule core.4). It refuses the version unless
    GitHub holds the tag at the revision the index names. It finds the
    tag's release, a draft among them, or makes one as a draft. It
    uploads each asset the release lacks or holds with another digest.
    Last, it publishes the release, the version's notes as its text and
    the tag as its title, not a prerelease, and marked latest. No run
    makes or moves a tag, and no asset is removed.
  - `current` and `report` read the release GitHub marks latest. They
    answer the index of the component's version its tag names in the
    store, and for report, when GitHub says the release was last updated.
    They answer unread where there is none, or where its tag names no
    version of the component (89, 91, 119).
- **The version's notes are read in the family module.** The reading of
  a version's files gains the reading of its notes, the artifact the
  index names in the ecosystem `notes` (134).
- **A tool's run may be handed variables of its own.** The run module
  sets variables for one run, beside those it removes.
- **The binding's credentials are reached by the targets.** A target
  reads a credential's file itself only where its tool takes no file,
  and the binding opens none (record 0033).
- **The suite runs the target group over the GitHub release target**
  with a scripted GitHub, declaring that it holds a marker (93).
- **The proof against GitHub is run by hand.** The tag
  `check-github-release` publishes three versions to a throwaway
  repository the operator names in `TIMBER_TOOLS_GITHUB_RELEASE`, and
  applies them. The run refuses without the setting, naming it. The
  token is a file a person makes, which no gate task reads (rule
  core.38). The devenv shell names nixpkgs' `gh` in
  `TIMBER_TOOLS_GH_CLI` (rule core.26).
- The site's targets page documents the target. Record 0043 holds the
  profile's choices. Rule core.21 says how a token reaches a tool that
  takes no file, and rule core.79 binds the release to the tag GitHub
  holds at the version's revision.

## Capabilities

### New Capabilities

- `github-release-target`: the GitHub release target profile: its
  table, the token read at use and handed to each run alone, every run's
  flags and folder, apply's reads, tag check, draft, uploads and
  publish, current and report read from the release GitHub marks latest,
  and its proofs.

### Modified Capabilities

- `environment-apply`: the verified version the core hands the target
  names the version's tag.
- `seam-contracts`: a target's contract is handed the verified version
  with its tag.
- `host-bindings`: the binding opens no credential's file, and a target
  whose tool takes no file reads it at the moment of use.
- `conformance-suite`: the target group runs over the GitHub release
  target.

## Out of scope, and what each waits on

- flume's and Boomdrive's adoption: each repository's
  `.config/timber.toml` naming its component, its binaries as outputs,
  and a `target.github_release` environment, the checks that build the
  binaries (135), and the retirement of flume's release workflow. Each
  repository's own unit, after the policy file the architecture page
  lists as missing.
- The token and where `[credentials]` points on deneb. swancloud
  declares it, and Boomdrive's timber module writes the host's file
  (record 0035).
- What runs `timber apply` for a release environment: the host's, by
  C.16 and rule core.52. timber starts nothing.
- The Nix cache target on Cachix, the other half of this bolt. Its own
  change.
- GitHub Enterprise Server. The release is made on github.com. A later
  key of the table takes another host when a repository needs one.
- A GitHub prerelease. A prerelease is never marked latest, so C.12's
  current could not read it. A statement change takes it if a channel of
  prereleases is to keep releases.

## Impact

- New: `crates/timber/src/profiles/target/github_release/` (the profile,
  its table, its `gh` runs and the scripted GitHub),
  `crates/timber/tests/github_release.rs` (by hand).
- `crates/timber-seams/src/target.rs`: `Verified` names its tag, and
  every maker of one in `timber-core`, `timber-fakes`,
  `timber-conformance` and the target profiles' tests passes it.
  `crates/timber-core/src/apply.rs` makes it from the component's prefix
  and the version.
- `crates/timber/src/profiles/target/file_artifact.rs` reads the notes.
  `profiles/target/mod.rs` binds `github_release`.
  `crates/timber/src/profiles/run.rs` sets a run's own variables.
- `crates/timber/moon.yml` gains the `github-release` task, tagged
  `check-github-release`. `devenv.nix` gains `TIMBER_TOOLS_GH_CLI`. No
  crate, dependency, language or file type is added.
- The site's targets page and its integrations index. Record 0043 in
  `docs/adr/`. Rules core.21 and core.79 in `docs/architecture/core.md`.
- Rules kept:
  - core.1: GitHub and `gh` are named only under `profiles/target/`.
  - core.2: the files and the notes are read through the family module,
    and the store through the store seam.
  - core.4: the index, the notes and each asset are read by digest, and
    current resolves the latest release's tag to a digest.
  - core.6 and core.13: no run makes or moves a tag (rule core.79).
  - core.19: GitHub's words are carried verbatim, and absent, refused
    and unreadable are kept apart.
  - core.21: the token is read at use, held for the operation, handed to
    each `gh` run alone, and written nowhere.
  - core.29: nothing is applied before the core's judge.
  - core.38: the proof against GitHub runs by hand.
  - core.71: the core signs the report once the latest release names
    the index.
