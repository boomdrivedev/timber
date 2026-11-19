## Context

C.12's row for a GitHub release, at `boomdrivedev/blueprints` fecf4d9,
reads: push: the release of the version's tag made or updated, its notes
(134) as the release's text and the version's artifacts the policy names
as its assets, with the target's own credential; current is the release
GitHub marks latest. Record 0033 gives every target's credential: the
host's `[credentials]` names a file, and the target hands its path to
its own tool. Its consequences add that a tool taking a token only in a
variable is handed it by the target, which reads the file at the moment
of use for that one run (64).

What is built that this target stands on:

- A publish claims the version's name as a lightweight tag,
  `<prefix><version>`, at the landed revision, and pushes it to the
  shared history without force (22, rule core.13). The store holds the
  version under `version_name(component, tag)` at its index's digest.
- The index names its component, version, artifacts, the revision and
  the publishing repository by its remote's owner and name (13, 138).
- The notes are an artifact of the ecosystem `notes`, named by the
  component at the version, of one layer, the rendered Markdown (134).
- Each output the file projection packs is an artifact of the ecosystem
  `file`, named by the output, of one layer, the file's own bytes. The
  family module `profiles/target/file_artifact.rs` reads the index, an
  output's file artifact and its bytes by digest for every target.
- The core hands a target `Verified { index, version }`. It reads the
  component's prefix from the policy at the landed head, and already
  names the tag `<prefix><version>` when it finds the revision a report
  claim names (`apply.rs`, `tagged`).
- `profiles/run.rs` runs a tool with variables removed, and sets none.
- The Dash0 target is the pattern: a backend trait private to the
  profile, a scripted backend, a stand-in program in `check-fast`, and a
  proof by hand under a tag of its own.

GitHub's CLI, `gh` 2.102.0 as nixpkgs gives it, read on 2026-11-18,
settles what can be built:

- `gh` takes a token from `GH_TOKEN`, then `GITHUB_TOKEN`, then the
  sign-in stored under `GH_CONFIG_DIR`. It reads no token file.
- `gh release view <tag> --json tagName,isDraft,isPrerelease,assets`
  finds a published release by its tag and a draft by the tag it will
  take. Each asset answers its `name`, `state` and `digest`, the
  SHA-256 of its bytes. A tag with no release fails with `release not
  found`.
- `gh release create <tag> --verify-tag` fails where the repository on
  GitHub holds no such tag, rather than making one at the default
  branch's head. `--draft` makes it a draft.
- `gh release upload <tag> --clobber <files>` replaces an asset of the
  same name.
- `gh release edit <tag> --draft=false --prerelease=false --latest`
  publishes a draft and marks the release latest. It takes
  `--verify-tag`, `--title` and `--notes-file` too.
- `gh api repos/<owner>/<name>/releases/latest` answers the latest
  release's `tag_name` and `updated_at`, and `gh api
  repos/<owner>/<name>/git/ref/tags/<tag>` the tag's object. A path
  GitHub holds nothing at fails with `gh: Not Found (HTTP 404)`.
- GitHub never marks a draft or a prerelease latest, and renames an
  asset whose name holds characters other than letters, digits, `.`,
  `-` and `_`, or begins or ends with `.`.
- flume's own release today is titled by its tag, its text git-cliff's
  notes, its assets `flume-<system>.tar.gz`.

## Goals / Non-Goals

**Goals:**

- A version's GitHub release made or updated under the tag its publish
  claimed, on the revision its index names, and never on another.
- The release's text the version's notes as the store holds them, and
  its assets the bytes the version's index names, each by digest.
- What an environment runs readable from GitHub with no timber process
  running: the release GitHub marks latest (20, 119).
- The token reaching `gh` for one operation's runs alone, and the host's
  own `gh` sign-in never read (rule core.21).
- The table's refusals, every run's arguments and environment, and the
  target group all checked in `check-fast`, with a proof by hand.

**Non-Goals:**

- GitHub Enterprise Server, and a release in any repository but the
  checkout's.
- GitHub's prerelease flag, a release's discussion, and an asset's label.
- Removing an asset, a release or a tag.

## Decisions

### D1. The profile is `github_release`, running `gh` behind a backend of its own

The profile lives in `profiles/target/github_release/`. Its `mod.rs`
holds the target, `table.rs` the table, `gh.rs` the runs of `gh` behind a
trait private to the profile, `scripted.rs` a GitHub that answers as a
test sets, and `tests.rs`. The family's `bound` binds it under
`github_release` beside `clan`, `dash0` and `nix_cache`, handed a store
binding of its own, the checkout's repository and the host's
credentials.

The trait's operations are:

- `tag`: the commit a tag names on GitHub, peeled, or none.
- `release`: a tag's release, a draft or published, with each asset's
  name, state and digest, or none.
- `create`: a draft release of a tag.
- `upload`: files attached to a tag's release, replacing those of the
  same names.
- `publish`: a tag's release published, marked latest, with its title
  and text.
- `latest`: the release GitHub marks latest, by its tag and its
  `updated_at`, or none.

Each is handed the repository and the token.

Not taken: GitHub's REST API spoken by timber through `ureq`. That makes
timber a second client of a service whose own tool exists, with its
upload protocol and its draft lookup written again. A stand-in server of
the tests' own would then prove timber against its own idea of GitHub.

### D2. The verified version names its tag

`Verified` gains `tag`, the version's name in the shared history:
`Verified::new(index, version, tag)` and `Verified::tag()`. The core's
apply makes it as `<prefix><version>`, the component's prefix as the
policy at the landed head declares it, with the one function its
`tagged` already uses. Every other maker, in the fakes, the suite and
the target profiles' tests, passes the tag its version would take. The
fake target records it. The Nix and Dash0 profiles ignore it.

B.7's apply is given "a version the reconciler has verified" (90). The
version's name is the tag the publish claimed (22). So the seam's
operations stay as they are, and only the value handed grows.

Not taken:

- The profile listing GitHub's tags and resolving each in the store as
  the component's version name, until one names the index. That costs
  a read per tag, and reads a forge to learn what the core knows.
- A `prefix` key in the table. It would restate the component's prefix
  and could drift from it.
- The profile reading the policy's components. Only the policy profile
  reads the policy file (rule core.43).

### D3. The table is `github_release = { credential, assets }`

```toml
[environments.flume.github]
channel = "stable"
target.github_release = { credential = "flume-releases", assets = ["flume-aarch64-darwin.tar.gz", "flume-aarch64-linux.tar.gz", "flume-x86_64-linux.tar.gz"] }
```

- `credential` is a non-empty name with no whitespace.
- `assets` is a list of the component's output names, each once, and
  may be empty, for a release of notes alone. Each name is letters,
  digits, `.`, `-` and `_`, neither beginning nor ending with `.`.

Any other key, a missing key, or one of another form is unreadable at
every operation, naming the environment, the key and the two the table
reads, and nothing runs (45). No token, path, host or repository can sit
in the table.

Each asset is the output's file artifact, uploaded under the output's
name. A repository whose release carries `flume-x86_64-linux.tar.gz`
names its output so. The name GitHub keeps must be the name timber
asked for, since a later apply compares assets by name. So a name GitHub
would rename is refused in the table rather than renamed on GitHub.

Not taken:

- A map from each asset's name to an output. That is a second name for
  one thing, and the output's name is the authors' to choose.
- Every `file` artifact of the version as an asset. A component that
  packs a file it never meant to release would release it.
- An optional `assets`. The key says what the release carries where a
  reviewer reads it.

### D4. The token is read at use and handed to each run of `gh` alone

`gh` reads no token file. So, as record 0033's consequences give it,
the profile reads the file the table's `credential` names in the host's
`[credentials]` when current, apply or report runs. It trims the
trailing whitespace, and holds the token in the operation's memory
until the operation ends. It writes the token nowhere and puts it in no
argument or answer.

Every run of an operation is `gh`, as found on `PATH`, in a folder the
operation makes under the system's temporary directory and removes
after, holding an empty `config/`. The run module gains variables set
for one run, applied after those it removes. Each run is given:

- every variable of timber's own environment beginning `GH_` or
  `GITHUB_` removed, so no `GITHUB_TOKEN`, `GH_HOST`, `GH_REPO` or
  `GH_ENTERPRISE_TOKEN` of the host's is read;
- `GH_CONFIG_DIR=<run>/config`, so no stored sign-in, host or alias of
  the host's own `gh` is read;
- `GH_TOKEN=<token>`, `GH_PROMPT_DISABLED=1` and
  `GH_NO_UPDATE_NOTIFIER=1`;
- the host by name in its arguments: `--repo github.com/<owner>/<name>`
  for `gh release`, and `--hostname github.com` for `gh api`.

A credential the host's file doesn't name is unreadable, naming it and
the bindings file. A file that can't be read is unreadable, naming the
credential and the path, with the operating system's words. A file
holding nothing but whitespace is unreadable, naming the credential.
Each runs nothing. desired reads no credential.

A run that exits other than as its operation allows is unreadable,
naming the command run and carrying `gh`'s stderr verbatim (59, rule
core.19). The command's words hold no token, since none is an argument.

This departs from rule core.21's "its path is handed to the target's own
tool, which alone reads it", as record 0033 foresaw for a tool that
takes no file. Rule core.21 is amended to say so (D11).

Not taken:

- `gh auth login --with-token` into the run's folder, fed the file. It
  writes the token to disk under the temporary directory.
- A shell, handed the path, that reads the file into `GH_TOKEN` and runs
  `gh`. A second program reads the file all the same, and the token
  then lives in a shell's variables and its command line.
- The host's own `gh` sign-in. It is the person's, with the person's
  scopes, and shared with every `gh` they run.

### D5. The release is the checkout's repository's, on github.com

The repository is the checkout's, by its remote's owner and name, which
the binding hands every target. apply refuses an index naming another
publishing repository, or none, as the clan and Dash0 profiles do, and
an index naming no revision. The release goes where the publish pushed
the tag.

A remote on another host than github.com would name a repository of
GitHub that doesn't hold the tag at the index's revision, so D6's tag
check refuses it before any release is read.

### D6. Apply's steps

apply follows these steps, and stops at the first that fails, answering
it:

1. Read the table (D3). Read the index by digest. Refuse an index of
   another repository, or none, or naming no revision (D5).
2. Read the notes, the artifact the index names in the ecosystem `notes`
   by the component's name, through the family module (D10). Refuse an
   index naming none, or notes that are not UTF-8. Read each named
   output's file artifact and its bytes by digest. Write the notes as
   `<run>/notes.md` and each asset as `<run>/assets/<output>`.
3. Read the credential (D4). Nothing has reached GitHub yet.
4. `tag`: read the tag `Verified::tag()` on GitHub, `gh api --hostname
   github.com repos/<owner>/<name>/git/ref/tags/<tag>`, and where its
   object is a tag object, that object's commit through
   `git/tags/<sha>`. Refuse the version where GitHub holds no such tag,
   naming the tag and the repository, or holds it at another commit than
   the index's revision, naming the tag and both (rule core.79).
5. `release`: `gh release view <tag> --repo github.com/<owner>/<name>
   --json tagName,isDraft,isPrerelease,assets`. `release not found` is
   none.
6. `create`, where there is none: `gh release create <tag> --repo …
   --verify-tag --draft --title <tag> --notes-file <run>/notes.md`.
7. `upload` the named assets the release lacks: each whose name the
   release holds in no asset of state `uploaded` with the digest of the
   output's bytes, in one `gh release upload <tag> --repo … --clobber
   <run>/assets/<output>…`. None where every one is held.
8. `publish`: `gh release edit <tag> --repo … --verify-tag
   --draft=false --prerelease=false --latest --title <tag> --notes-file
   <run>/notes.md`.

The release is made a draft and published last, so a reader of the
repository's latest release never sees one without its assets. That also
holds where the repository makes its releases immutable, which takes no
asset once a release is published. An apply that stops after step 6
leaves a draft, which the next apply's step 5 finds by its tag. So one
tag never takes two releases. An apply of a version whose release is
published already writes its text and title again and marks it latest,
which is how a move back converges (34).

No step makes, moves or deletes a tag. Step 4 refuses before any write,
and `--verify-tag` stops `gh` making one where a tag went in between.

### D7. An asset is held by its digest, and none is removed

An asset counts as held where the release has one of its name, uploaded,
with the SHA-256 of the output's bytes, which is the digest of the file
artifact's one layer. One of the name with another digest, none, or no
`uploaded` state is uploaded with `--clobber`. A `gh` that answers no
digest for an asset makes it count as not held, so it is uploaded again,
never skipped.

No asset is removed. A release is one version's, so an asset the
version doesn't name was put there by hand or by an apply under an
earlier `assets`. Either way it is not the version's to take away.

Not taken: a marker naming the assets timber uploaded, as the Dash0
target's names its dashboards. A release holds one version, so nothing
an earlier version applied lingers in it for a later one to remove.

### D8. The release is titled by its tag, never a prerelease, and marked latest

C.12's current is the release GitHub marks latest. GitHub never marks a
prerelease latest. So every release apply publishes is no prerelease and
is marked latest, whatever the version's form, and current reads it.
Its title is its tag, as flume's releases are titled today, and its text
the notes' bytes as the store holds them.

A channel whose versions are semantic prereleases, `1.2.4-pre.2`, makes
each a release marked latest. A release environment follows the channel
of releases the authors choose.

Not taken:

- A semantic prerelease made a GitHub prerelease. current could never
  read it, so the core would never sign its report (rule core.71), and
  every apply would run again.
- The notes' first heading as the title. That ties the title to the
  template the repository gives git-cliff.

### D9. current and report are read from the release GitHub marks latest

`current`:

1. Read the table and the credential.
2. `latest`: `gh api --hostname github.com
   repos/<owner>/<name>/releases/latest`. `gh: Not Found (HTTP 404)` is
   none.
3. Resolve `version_name(component, tag_name)` through the profile's
   store binding, the component being the environment's channel's (125).

It answers that digest. It is unread where GitHub marks no release
latest, naming the repository, and where the store resolves no version
of the component under the tag, naming the tag: a release made by hand,
or another component's (119). It never guesses.

`report` answers the same digest, with `converged_at` the latest
release's `updated_at` as GitHub answers it. As in the clan and Dash0
designs, that orders nothing: the core orders reports by the apply's
`at`. A release with no `updated_at` is unread.

`desired` reads the table and answers the head the core hands it (88).

Not taken: current read from the release of the head's tag. That says
whether a release exists, not which one GitHub serves as latest, which
is what a reader of the repository is given.

### D10. The notes are read in the family module

`file_artifact.rs` gains `notes_of(index, words, source)`: the artifact
the index names in the ecosystem `notes`, named by the index's
component, refused naming the version where it names none. Its bytes are
read with the module's `bytes`, which checks the one layer against its
digest. The module keeps its own `NOTES`, as it keeps `FILE`, read from
the index, so no profile asks the core anything (rule core.2).

### D11. Proofs

- **A scripted GitHub.** It holds tags, releases with their assets and
  digests, and which release is latest, in memory. It answers each
  operation of the private trait as a test sets, records each operation
  asked in order with the token it was handed, and fails any operation
  with words a test gives, or every operation handed a given token. The
  table's refusals, apply's steps and their order, current, report and
  the target group run over it in `check-fast`.
- **A stand-in `gh`.** The real trait's runs are checked in `check-fast`
  against a shell script given as the program. The script records its
  arguments, its environment and the contents of the `GH_CONFIG_DIR` it
  was handed, and prints what a test sets on stdout or stderr with the
  exit a test sets. This holds each operation's arguments, the removed
  `GH_*` and `GITHUB_*` variables, the empty folder of the run's own,
  the token in `GH_TOKEN` and in no argument, and the two not-found
  wordings.
- **Against GitHub, by hand.** `crates/timber/tests/github_release.rs`
  sits behind the `tools` feature. It runs under the `github-release`
  task of `crates/timber/moon.yml`, tagged `check-github-release`, with
  `$TIMBER_TOOLS_GITHUB_RELEASE` among its inputs. The setting is
  `<owner>/<name>#<token file>`: a throwaway repository, and a file
  holding a token that may write its contents. A run without the setting
  fails naming it. With it, the run takes these steps:
  1. Clone the repository into the test's directory with the host's own
     git credential. Publish three versions, A, B and C, of one
     component from three commits on a branch of the run's own, through
     the fakes' build, each with its notes and the outputs
     `timber-proof-a.txt` and `timber-proof-b.txt`, C's `-b` of other
     bytes. Each publish pushes its tag to the repository.
  2. Run the target group over the profile.
  3. Apply A, B, then A again, checking after each the releases' text,
     assets, digests and draft state, which one GitHub marks latest,
     `current` and `report`. The second apply of A uploads nothing.
  4. Make a draft release of C's tag with `gh`, as an apply that
     stopped after its draft leaves it. Apply C, and check that C's tag
     has one release, published, latest, with both assets.
  5. Remove every release, tag and branch the run made.

  The `gh` it runs is `TIMBER_TOOLS_GH_CLI`, nixpkgs' `gh` as
  `devenv.lock` pins it, which `devenv.nix` names without putting it on
  the shell's `PATH`. No gate task runs it, since the token is a file a
  person makes (rule core.38).

Not taken: a GitHub of the tests' own in the tools layer. There is none
to run, and the stand-in program already proves what timber hands `gh`.

### D12. Records

Record 0043 holds D2 to D9, each with the path not taken, citing the
statement at fecf4d9 and record 0033. Rule core.21 gains the token read
at use for a tool that takes no file. Rule core.79 holds D6's bound: a
release only on the tag GitHub holds at the version's revision, and no
tag made, moved or deleted, and no asset removed.

## Risks / Trade-offs

- **Two components' releases in one repository.** GitHub marks one
  release latest for the whole repository, so a second component's
  release environment reads the first's release as current and is
  unread, and each apply takes latest from the other. → One component
  per repository takes a GitHub release. The site's page says so.
- **A channel of prereleases.** Each of its versions becomes a release
  marked latest (D8). → The release environment follows the channel the
  authors promote releases to.
- **`gh`'s wording of a missing release or path.** A release that
  rewords `release not found` or `gh: Not Found (HTTP 404)` makes the
  read unreadable rather than none. → Nothing is written before the
  reads, the stand-in pins the wordings, and the proof runs the `gh` the
  tree pins.
- **`gh`'s draft lookup by tag.** apply relies on `gh release view`
  finding a draft by the tag it will take. → The proof's step 4 makes a
  draft and checks one release results.
- **An immutable release whose assets differ.** GitHub refuses the
  upload. → apply is unreadable with GitHub's words. The bytes of a
  version never change, so only an asset added by hand under the
  version's name meets it.
- **The token in a run's environment.** A process of the same user can
  read it there while `gh` runs. → That user can read the token's file
  too. The token reaches no other program, no argument and no file.
- **A release edited by hand.** The next apply writes its text and title
  back. An asset added by hand stays.
- **A different `gh` on `PATH`.** → The host runs `timber apply` in the
  repository's shell, whose `gh` is its own pin.

## Migration Plan

Nothing published changes. `Verified` lives in memory alone, so its new
field changes no stored form. A bindings file and a policy file read as
before. flume and Boomdrive adopt the target in their own units: each
policy file declares the component, its binaries as outputs and a
`target.github_release` environment, and the host's `[credentials]`
names the token. flume's release job retires once its first release
comes through timber. Rollback is a promotion back and an apply, which
marks the earlier release latest again (34).
