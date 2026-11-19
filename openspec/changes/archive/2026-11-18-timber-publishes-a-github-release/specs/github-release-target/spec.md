# Spec Delta

## Purpose

The target profile bound as `github_release` (B.7, C.12): the GitHub
release of a version's tag made or updated in the checkout's repository,
the version's notes as its text and the outputs the table names as its
assets, with GitHub's own CLI and the host's token; current the release
GitHub marks latest.

## ADDED Requirements

### Requirement: The GitHub release target is a target profile of its own

The GitHub release target SHALL implement the target seam for the
releases of one repository on GitHub, pushed to from the host that holds
its credential (B.7, C.12). It SHALL be bound under the profile name
`github_release`, among every target profile the binding holds, reached
through the target seam alone, and SHALL name GitHub and `gh` only under
`profiles/target/` (rules core.1, core.2).

#### Scenario: An environment names the GitHub release target

- **WHEN** the environment `github` writes `target.github_release = { credential = "flume-releases", assets = ["flume-x86_64-linux.tar.gz"] }`
- **THEN** apply reaches the GitHub release target with `github`'s settings

### Requirement: The table names the credential and the assets

The profile's table SHALL hold two required keys (C.10, C.12):
`credential`, a non-empty name with no whitespace; and `assets`, a list
of the component's output names, each once, possibly empty. Each name
SHALL be letters, digits, `.`, `-` and `_`, neither beginning nor ending
with `.`, since GitHub renames any other. A missing key or one of
another form SHALL make every operation unreadable naming the
environment and the key, with nothing run.

#### Scenario: An asset GitHub would rename

- **WHEN** the table names `assets = ["flume linux"]`
- **THEN** apply is unreadable naming `assets` and `flume linux`, and `gh` is not run

#### Scenario: An asset named twice

- **WHEN** the table names `assets = ["flume.tar.gz", "flume.tar.gz"]`
- **THEN** apply is unreadable naming `assets`, and `gh` is not run

#### Scenario: A release of notes alone

- **WHEN** the table names `assets = []`
- **THEN** the table reads, and apply attaches no asset

### Requirement: The table reads no other key

Any key of the table but the two SHALL make desired, current, apply and
report unreadable, naming the key and the two the table reads, with
nothing run and the key's value in no words. No token, path, host or
repository SHALL be read from the table (45, rule core.21).

#### Scenario: A token in the table

- **WHEN** the table names its two keys and `token = "ghp_abc"`
- **THEN** apply is unreadable naming `token` and the two keys, its words hold no `ghp_abc`, and `gh` is not run

### Requirement: The credential is the host's, read at the moment of use

The table's `credential` SHALL name a credential of the host's
`[credentials]` (118). current, apply and report SHALL read the file it
names when they run, and SHALL hold the token for that operation alone,
written nowhere and in no answer's words (64, rule core.21). A
credential the host's file does not name, a file that cannot be read, or
one holding nothing SHALL make them unreadable naming the credential,
with nothing run. desired SHALL read no credential.

#### Scenario: A credential the host does not name

- **WHEN** the table names `credential = "flume-releases"` and the bindings file's `[credentials]` names no `flume-releases`
- **THEN** apply is unreadable naming `flume-releases` and the bindings file's path, and `gh` is not run

#### Scenario: A file that cannot be read

- **WHEN** `[credentials]` maps `flume-releases` to a path that holds no file
- **THEN** apply is unreadable naming `flume-releases` and the path, with the operating system's words, and `gh` is not run

#### Scenario: An empty file

- **WHEN** the file `flume-releases` names holds only a newline
- **THEN** apply is unreadable naming `flume-releases`, and `gh` is not run

### Requirement: The token reaches each run of `gh` alone

Every run SHALL be `gh` as found on `PATH`, handed the token in
`GH_TOKEN` and in no argument, with `GH_PROMPT_DISABLED` set. The token
SHALL be handed to the operation's own runs of `gh` and to no other
program (record 0033, rule core.21).

#### Scenario: The token goes in the run's variable

- **WHEN** `[credentials]` maps `flume-releases` to a file holding `ghp_abc`, and apply runs against a stand-in `gh`
- **THEN** every run sees `GH_TOKEN` set to `ghp_abc`, and no argument of any run holds `ghp_abc`

### Requirement: No run of `gh` reads the host's own settings

Every variable whose name begins `GH_` or `GITHUB_` in timber's own
environment SHALL be removed from each run's environment. Each run's
`GH_CONFIG_DIR` SHALL name an empty folder the operation made for it
alone and removes when it ends, so no sign-in, host or repository of the
host's own `gh` is read (record 0033, rule core.21).

#### Scenario: The host's own settings are not read

- **WHEN** timber's environment sets `GITHUB_TOKEN`, `GH_HOST`, `GH_REPO` and `GH_CONFIG_DIR`, and apply runs against a stand-in `gh`
- **THEN** no run sees `GITHUB_TOKEN`, `GH_HOST` or `GH_REPO`, and each sees a `GH_CONFIG_DIR` that is empty and is not the host's

### Requirement: The release is the checkout's repository's

Every run SHALL name the checkout's repository, by its remote's owner
and name, on github.com. apply SHALL refuse a version whose index names
another publishing repository, or none, or no revision, before `gh` runs
(138, rule core.60). An operation whose checkout's repository is unread
SHALL be unreadable with the reason.

#### Scenario: Another repository's index

- **WHEN** the checkout's repository is `boomdrivedev/flume` and the version's index names `boomdrivedev/sluice`
- **THEN** apply is refused naming both, and `gh` is not run

### Requirement: desired is the channel's head

The profile SHALL answer desired with the head the core hands it, once
its table reads, and absent where the core's head is absent (31, 88).

#### Scenario: An empty channel

- **WHEN** desired is asked of an environment whose channel's head the core answers absent
- **THEN** desired is absent, and `gh` is not run

### Requirement: Apply reads the notes and the assets by digest

apply SHALL read the version's index, its notes and each named output's
file artifact through its store binding by digest, checking each against
its digest, before `gh` runs (42, 134, rules core.4, core.5). An index
naming no notes, notes that are not UTF-8, and a named output with no
file artifact SHALL each be refused naming the version and what it
lacks.

#### Scenario: A version published before notes

- **WHEN** the version's index names no artifact of the ecosystem `notes`
- **THEN** apply is refused naming the version and its notes, and `gh` is not run

#### Scenario: An asset the version does not carry

- **WHEN** the table names `assets = ["flume-x86_64-linux.tar.gz"]` and the index names no `file` artifact of that output
- **THEN** apply is refused naming the output and the version, and `gh` is not run

### Requirement: Apply releases only the tag GitHub holds at the version's revision

apply SHALL read the version's tag in the repository on GitHub, peeled
to its commit, and SHALL refuse the version, running nothing more, where
GitHub holds no such tag or holds it at another revision than the
index's, naming the tag and each revision (22, rules core.6, core.79).
No run SHALL make, move or delete a tag.

#### Scenario: A tag GitHub does not hold

- **WHEN** GitHub answers `gh: Not Found (HTTP 404)` for the tag `v0.31.0`
- **THEN** apply is refused naming `v0.31.0` and the repository, and no release is read or made

#### Scenario: A tag at another revision

- **WHEN** GitHub holds `v0.31.0` at a revision other than the one the index names
- **THEN** apply is refused naming `v0.31.0` and both revisions, and no release is read or made

#### Scenario: A release made only on a tag held

- **WHEN** apply makes or publishes a release against a stand-in `gh`
- **THEN** each run that makes or publishes it is given `--verify-tag`

### Requirement: Apply makes a release the tag lacks as a draft

apply SHALL find the tag's release, published or a draft. Where there is
none, it SHALL make one as a draft, titled by the tag, holding the
version's notes as its text, before any asset is attached. An apply that
stopped after making a draft SHALL leave it for the next apply to
finish, and none SHALL make a second release of one tag.

#### Scenario: A first apply

- **WHEN** GitHub holds the tag `v0.31.0` and no release of it, and apply runs
- **THEN** a draft release of `v0.31.0` is made with the version's notes, then its assets are uploaded, then it is published

#### Scenario: A draft an earlier apply left

- **WHEN** GitHub holds a draft release of `v0.31.0`, and apply runs
- **THEN** no release is made, and that draft is given its assets and published

### Requirement: Apply attaches each asset the release does not hold

apply SHALL upload each named output's bytes under the output's name
where the release holds no uploaded asset of that name with the bytes'
digest, replacing one of that name, and SHALL upload none it holds so
(C.12). No asset SHALL be removed, so one the version does not name
stays as it is.

#### Scenario: Assets held already

- **WHEN** the release of `v0.31.0` holds every named asset with its bytes' digest
- **THEN** apply uploads nothing

#### Scenario: An asset of other bytes

- **WHEN** the release holds `flume-x86_64-linux.tar.gz` with another digest than the version's bytes
- **THEN** apply uploads the version's bytes under that name in its place, and removes no other asset

### Requirement: Apply publishes the release last, marked latest

apply SHALL last write the release's text as the version's notes and its
title as the tag, and SHALL publish it, not a prerelease, marked the
release GitHub calls latest, so current reads it (C.12, 134). A release
published already SHALL be written so again.

#### Scenario: A move back

- **WHEN** apply of `v0.30.0` runs while `v0.31.0`'s release is marked latest
- **THEN** `v0.30.0`'s release is published marked latest, and its text is `v0.30.0`'s notes

### Requirement: current is the version of the release GitHub marks latest

current SHALL read the release GitHub marks latest in the repository and
SHALL answer the digest the store resolves the environment's component's
version name of its tag to (89, rule core.4).

#### Scenario: After a converged apply

- **WHEN** GitHub marks `v0.31.0`'s release latest and the store resolves the component's version `v0.31.0` to the index `A`
- **THEN** current answers `A`

### Requirement: current is unread where no release names a version

current SHALL answer unread with the reason, never a digest, where
GitHub marks no release latest, naming the repository, and where the
latest release's tag names no version of the component the store holds,
naming the tag (119).

#### Scenario: A repository with no release

- **WHEN** GitHub answers `gh: Not Found (HTTP 404)` for the latest release
- **THEN** current is unread naming the repository, and is no digest

#### Scenario: A release made by hand

- **WHEN** the latest release's tag is `nightly`, which names no version of the component in the store
- **THEN** current is unread naming `nightly`, and is no digest

### Requirement: report is the latest release's version and when GitHub updated it

report SHALL answer the digest current answers, with `converged_at` the
latest release's `updated_at` as GitHub answers it (91). A release with
no `updated_at` SHALL be unread. Where current is unread, report SHALL be
unread with the same reason (119).

#### Scenario: Reported after a converged apply

- **WHEN** apply converges and GitHub answers the latest release with `updated_at` `2026-11-18T20:22:59Z`
- **THEN** report answers the applied index and `2026-11-18T20:22:59Z`

### Requirement: GitHub's words are the answer

A run of `gh` that exits other than as its operation allows SHALL make
that operation unreadable, naming the command run and carrying `gh`'s
stderr verbatim, never the token, and nothing after it SHALL run (59,
rule core.19).

#### Scenario: A token that may not write

- **WHEN** GitHub answers the draft's making with `HTTP 403: Resource not accessible by personal access token`
- **THEN** apply is unreadable carrying those words, and no asset is uploaded

### Requirement: The profile passes the target group

The GitHub release target SHALL pass the conformance suite's target
group, declaring that it holds a marker, in its own tests against a
scripted GitHub that answers each operation of the profile as `gh`
would, in `check-fast` (93, rule core.33).

#### Scenario: The target group over a scripted GitHub

- **WHEN** the GitHub release target's tests run the target group over the scripted GitHub
- **THEN** every scenario of the group that applies to a target holding a marker passes

### Requirement: Against GitHub, by hand

A test under its own tag, `check-github-release`, SHALL run the target
group and applies of three versions against a repository the operator
names in `TIMBER_TOOLS_GITHUB_RELEASE`, with the `gh` the tree pins,
checking each release's text, assets and mark. It SHALL refuse to run
without the setting, naming it, and no gate task SHALL run it, since its
token is a file a person makes (rule core.38).

#### Scenario: Run without the setting

- **WHEN** `moon run cli:github-release` runs with `TIMBER_TOOLS_GITHUB_RELEASE` unset
- **THEN** it fails naming `TIMBER_TOOLS_GITHUB_RELEASE`, and nothing reaches GitHub
