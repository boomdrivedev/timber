//! The profile's runs of `gh`, GitHub's own CLI, behind a trait of the
//! profile's own, which is no seam: the target seam stays the one
//! interface (rule core.2; design D1, D4). [`Gh`] runs `gh` as found on
//! `PATH`, which where the host runs `timber apply` in the repository's
//! shell is the `gh` the repository pins. A test's scripted GitHub
//! answers in its place.
//!
//! `gh` reads no token file, so the target reads the credential's file
//! at the moment of use and hands the token to each run of the operation
//! in `GH_TOKEN` alone, never in an argument (record 0033, rule core.21;
//! design D4). Every run is given nothing of the host's: it runs in a
//! folder the operation makes under the system's temporary directory and
//! removes when it ends, holding an empty `config/`; every variable whose
//! name begins `GH_` or `GITHUB_` is removed from its environment; and it
//! is given
//!
//! ```text
//! GH_CONFIG_DIR=<run>/config GH_TOKEN=<token> GH_PROMPT_DISABLED=1 GH_NO_UPDATE_NOTIFIER=1
//! ```
//!
//! so no sign-in, host, alias or repository of the host's own `gh` is
//! read. Each run names the host: `--repo github.com/<owner>/<name>` for
//! `gh release`, and `--hostname github.com` for `gh api`. A run that
//! exits other than as its operation allows is unreadable, naming the
//! command run and carrying `gh`'s stderr verbatim (59, rule core.19); the
//! command holds no token, since none is an argument.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use timber_seams::{Answer, Digest, Not, OutputName, RepositoryName, Revision, Source};

use crate::profiles::run::{Ran, Tool};
use crate::profiles::target::token::Token;

/// The variables removed from every run's environment: every one of the
/// host's own `gh` settings and GitHub tokens.
const GH_ENV: &[&str] = &["GH_*", "GITHUB_*"];

/// The host every run names.
pub(super) const HOST: &str = "github.com";

/// What `gh release view` says of a tag with no release.
const RELEASE_NOT_FOUND: &str = "release not found";

/// What `gh api` says of a path GitHub holds nothing at.
const API_NOT_FOUND: &str = "gh: Not Found (HTTP 404)";

/// How many tag objects a tag is peeled through before its commit.
const PEELS: usize = 8;

/// Where an operation reaches GitHub: the checkout's repository on
/// github.com, and the token it is handed.
#[derive(Clone, Copy, Debug)]
pub(super) struct At<'a> {
    pub repository: &'a RepositoryName,
    pub token: &'a Token,
}

/// An asset a release holds, as GitHub answers it: its name, its state,
/// `uploaded` once its bytes are all there, and the digest of its bytes,
/// none where GitHub answers none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Asset {
    pub name: String,
    pub state: String,
    pub digest: Option<Digest>,
}

/// A tag's release, a draft or published, by the assets it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Release {
    pub assets: Vec<Asset>,
}

/// The release GitHub marks latest: its tag, and when GitHub says it was
/// last updated, where it says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Latest {
    pub tag: String,
    pub updated_at: Option<String>,
}

/// An asset to upload: the output it is, whose name it takes, and its
/// bytes.
#[derive(Clone, Copy, Debug)]
pub(super) struct Upload<'a> {
    pub output: &'a OutputName,
    pub bytes: &'a [u8],
}

/// What the GitHub release target asks of GitHub (design D1).
pub(super) trait Backend {
    /// The commit `tag` names in the repository on GitHub, peeled through
    /// any tag object; none where GitHub holds no such tag.
    fn tag(&self, at: &At<'_>, tag: &str) -> Answer<Option<Revision>>;

    /// `tag`'s release, a draft or published; none where it has none.
    fn release(&self, at: &At<'_>, tag: &str) -> Answer<Option<Release>>;

    /// A draft release of `tag`, titled by it, holding `notes` as its
    /// text, made only where GitHub holds the tag.
    fn create(&self, at: &At<'_>, tag: &str, notes: &[u8]) -> Answer<()>;

    /// `assets` attached to `tag`'s release, each under its output's name,
    /// replacing one of that name, in one run.
    fn upload(&self, at: &At<'_>, tag: &str, assets: &[Upload<'_>]) -> Answer<()>;

    /// `tag`'s release published, not a prerelease, marked latest, titled
    /// by the tag and holding `notes` as its text.
    fn publish(&self, at: &At<'_>, tag: &str, notes: &[u8]) -> Answer<()>;

    /// The release GitHub marks latest; none where it marks none.
    fn latest(&self, at: &At<'_>) -> Answer<Option<Latest>>;
}

/// GitHub's own CLI.
pub(super) struct Gh {
    gh: Tool,
    source: Source,
}

/// The folder of one operation, removed when it is dropped.
struct Folder(tempfile::TempDir);

impl Folder {
    fn path(&self) -> &Path {
        self.0.path()
    }

    fn config(&self) -> PathBuf {
        self.path().join("config")
    }
}

/// What `gh release view --json tagName,isDraft,isPrerelease,assets`
/// prints, by the assets the profile reads of it; whether it is a draft
/// or a prerelease, the publish writes either way.
#[derive(Deserialize)]
struct Viewed {
    #[serde(default)]
    assets: Vec<ViewedAsset>,
}

#[derive(Deserialize)]
struct ViewedAsset {
    name: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    digest: Option<String>,
}

/// What `gh api …/git/ref/tags/<tag>` and `…/git/tags/<sha>` print: the
/// object the tag names.
#[derive(Deserialize)]
struct Referred {
    object: Object,
}

#[derive(Deserialize)]
struct Object {
    sha: String,
    #[serde(rename = "type")]
    kind: String,
}

/// What `gh api …/releases/latest` prints.
#[derive(Deserialize)]
struct Latested {
    tag_name: String,
    #[serde(default)]
    updated_at: Option<String>,
}

impl Gh {
    /// `program`, `gh` as found on `PATH` or a path to it, answering as
    /// `source`.
    pub fn new(program: impl Into<PathBuf>, source: &Source) -> Gh {
        Gh {
            gh: Tool::new(program, source.clone()),
            source: source.clone(),
        }
    }

    /// Unreadable naming `what`, for `said`.
    fn unreadable(&self, what: impl Into<String>, said: String) -> Not {
        Not::unreadable(&self.source, what, Some(said))
    }

    /// A folder of the operation's own under the system's temporary
    /// directory, holding an empty `config/`.
    fn folder(&self) -> Answer<Folder> {
        let made = tempfile::Builder::new()
            .prefix("timber-gh-")
            .tempdir()
            .map_err(|e| {
                self.unreadable(
                    format!(
                        "a folder for a run of gh under {}",
                        std::env::temp_dir().display()
                    ),
                    e.to_string(),
                )
            })?;
        let folder = Folder(made);
        fs::create_dir(folder.config())
            .map_err(|e| self.unreadable(folder.config().display().to_string(), e.to_string()))?;
        Ok(folder)
    }

    /// `bytes` written at `path`.
    fn write(&self, path: &Path, bytes: &[u8]) -> Answer<()> {
        fs::write(path, bytes)
            .map_err(|e| self.unreadable(path.display().to_string(), e.to_string()))
    }

    /// The notes written into the operation's folder as `notes.md`, and
    /// that file.
    fn notes(&self, folder: &Folder, notes: &[u8]) -> Answer<PathBuf> {
        let file = folder.path().join("notes.md");
        self.write(&file, notes)?;
        Ok(file)
    }

    /// `--repo github.com/<owner>/<name>`, as every `gh release` names the
    /// repository.
    fn repo(at: &At<'_>) -> [OsString; 2] {
        ["--repo".into(), format!("{HOST}/{}", at.repository).into()]
    }

    /// A run in `folder` with no `GH_*` or `GITHUB_*` variable of the
    /// host's, handed the token and the folder's configuration, answering
    /// what it printed whatever it exits with.
    fn ran(&self, folder: &Folder, at: &At<'_>, args: &[OsString]) -> Answer<Ran> {
        let config = folder.config();
        let set: [(&str, &OsStr); 4] = [
            ("GH_CONFIG_DIR", config.as_os_str()),
            ("GH_TOKEN", OsStr::new(at.token.secret())),
            ("GH_PROMPT_DISABLED", OsStr::new("1")),
            ("GH_NO_UPDATE_NOTIFIER", OsStr::new("1")),
        ];
        self.gh.ran(Some(folder.path()), args, GH_ENV, &set)
    }

    /// A run in `folder` that must exit 0, answering what it printed.
    fn run(&self, folder: &Folder, at: &At<'_>, args: &[OsString]) -> Answer<Ran> {
        let ran = self.ran(folder, at, args)?;
        if ran.code == Some(0) {
            return Ok(ran);
        }
        Err(self.gh.failed(args, &ran))
    }

    /// A run in `folder` that must exit 0, or answers none where its
    /// stderr holds the line `missing`, as `gh` says GitHub holds nothing
    /// there.
    fn found(
        &self,
        folder: &Folder,
        at: &At<'_>,
        args: &[OsString],
        missing: &str,
    ) -> Answer<Option<Ran>> {
        let ran = self.ran(folder, at, args)?;
        if ran.code == Some(0) {
            return Ok(Some(ran));
        }
        let said = String::from_utf8_lossy(&ran.stderr);
        if said.lines().any(|line| line.trim() == missing) {
            return Ok(None);
        }
        Err(self.gh.failed(args, &ran))
    }

    /// `gh api --hostname github.com <path>`'s JSON, read as `T`; none
    /// where GitHub holds nothing at `path`.
    fn api<T: for<'de> Deserialize<'de>>(
        &self,
        folder: &Folder,
        at: &At<'_>,
        path: String,
    ) -> Answer<Option<T>> {
        let args: Vec<OsString> = ["api", "--hostname", HOST]
            .into_iter()
            .map(OsString::from)
            .chain([OsString::from(&path)])
            .collect();
        let Some(ran) = self.found(folder, at, &args, API_NOT_FOUND)? else {
            return Ok(None);
        };
        serde_json::from_slice(&ran.stdout).map(Some).map_err(|e| {
            self.unreadable(
                format!("what gh api --hostname {HOST} {path} printed"),
                format!("it is not the JSON GitHub answers there: {e}"),
            )
        })
    }
}

impl Backend for Gh {
    fn tag(&self, at: &At<'_>, tag: &str) -> Answer<Option<Revision>> {
        let folder = self.folder()?;
        let repository = at.repository;
        let path = format!("repos/{repository}/git/ref/tags/{tag}");
        let Some(referred) = self.api::<Referred>(&folder, at, path)? else {
            return Ok(None);
        };
        let mut object = referred.object;
        for _ in 0..PEELS {
            match object.kind.as_str() {
                "commit" => return Ok(Some(Revision::new(object.sha))),
                "tag" => {
                    let path = format!("repos/{repository}/git/tags/{}", object.sha);
                    let peeled = self.api::<Referred>(&folder, at, path)?.ok_or_else(|| {
                        self.unreadable(
                            format!("the tag object {} of {tag} in {repository}", object.sha),
                            format!("GitHub holds no tag object {}", object.sha),
                        )
                    })?;
                    object = peeled.object;
                }
                other => {
                    return Err(self.unreadable(
                        format!("the tag {tag} in {repository}"),
                        format!(
                            "it names a {other} {}, and a tag names a commit",
                            object.sha
                        ),
                    ))
                }
            }
        }
        Err(self.unreadable(
            format!("the tag {tag} in {repository}"),
            format!("it names more than {PEELS} tag objects before a commit"),
        ))
    }

    fn release(&self, at: &At<'_>, tag: &str) -> Answer<Option<Release>> {
        let folder = self.folder()?;
        let mut args: Vec<OsString> = vec!["release".into(), "view".into(), tag.into()];
        args.extend(Gh::repo(at));
        args.extend(["--json", "tagName,isDraft,isPrerelease,assets"].map(OsString::from));
        let Some(ran) = self.found(&folder, at, &args, RELEASE_NOT_FOUND)? else {
            return Ok(None);
        };
        let viewed: Viewed = serde_json::from_slice(&ran.stdout).map_err(|e| {
            self.unreadable(
                format!("the release of {tag} in {}", at.repository),
                format!("gh release view printed no release in JSON: {e}"),
            )
        })?;
        Ok(Some(Release {
            assets: viewed
                .assets
                .into_iter()
                .map(|asset| Asset {
                    name: asset.name,
                    state: asset.state,
                    digest: asset.digest.and_then(|d| Digest::parse(&d).ok()),
                })
                .collect(),
        }))
    }

    fn create(&self, at: &At<'_>, tag: &str, notes: &[u8]) -> Answer<()> {
        let folder = self.folder()?;
        let notes = self.notes(&folder, notes)?;
        let mut args: Vec<OsString> = vec!["release".into(), "create".into(), tag.into()];
        args.extend(Gh::repo(at));
        args.extend(
            ["--verify-tag", "--draft", "--title", tag, "--notes-file"].map(OsString::from),
        );
        args.push(notes.into());
        self.run(&folder, at, &args).map(drop)
    }

    fn upload(&self, at: &At<'_>, tag: &str, assets: &[Upload<'_>]) -> Answer<()> {
        let folder = self.folder()?;
        let dir = folder.path().join("assets");
        fs::create_dir(&dir)
            .map_err(|e| self.unreadable(dir.display().to_string(), e.to_string()))?;
        let mut args: Vec<OsString> = vec!["release".into(), "upload".into(), tag.into()];
        args.extend(Gh::repo(at));
        args.push("--clobber".into());
        for asset in assets {
            let file = dir.join(asset.output.as_str());
            self.write(&file, asset.bytes)?;
            args.push(file.into());
        }
        self.run(&folder, at, &args).map(drop)
    }

    fn publish(&self, at: &At<'_>, tag: &str, notes: &[u8]) -> Answer<()> {
        let folder = self.folder()?;
        let notes = self.notes(&folder, notes)?;
        let mut args: Vec<OsString> = vec!["release".into(), "edit".into(), tag.into()];
        args.extend(Gh::repo(at));
        args.extend(
            [
                "--verify-tag",
                "--draft=false",
                "--prerelease=false",
                "--latest",
                "--title",
                tag,
                "--notes-file",
            ]
            .map(OsString::from),
        );
        args.push(notes.into());
        self.run(&folder, at, &args).map(drop)
    }

    fn latest(&self, at: &At<'_>) -> Answer<Option<Latest>> {
        let folder = self.folder()?;
        let path = format!("repos/{}/releases/latest", at.repository);
        Ok(self
            .api::<Latested>(&folder, at, path)?
            .map(|latest| Latest {
                tag: latest.tag_name,
                updated_at: latest.updated_at,
            }))
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use tempfile::TempDir;
    use timber_seams::Seam;

    use super::*;
    use crate::profiles::run::under_env;

    const TOKEN: &str = "ghp_0123456789secret";
    const TAG: &str = "v0.31.0";
    const REPO: &str = "github.com/boomdrivedev/flume";
    const COMMIT: &str = "4f2a9c1d8e0b7a6c5f4e3d2c1b0a998877665544";
    const LINUX: &str = "flume-x86_64-linux.tar.gz";
    const DARWIN: &str = "flume-aarch64-darwin.tar.gz";

    fn source() -> Source {
        Source::new(Seam::Target, "github_release")
    }

    fn repository() -> RepositoryName {
        RepositoryName {
            owner: "boomdrivedev".into(),
            name: "flume".into(),
        }
    }

    /// A stand-in `gh`, written by the test, which records each run's
    /// arguments, its environment, the folder it ran in, the
    /// configuration folder it was handed and what that held, and each
    /// file an argument names, each run in a folder of its own numbered
    /// from 0; and answers the n-th run with what the test sets for it,
    /// printing nothing and exiting 0 where it sets nothing.
    struct StandIn {
        dir: PathBuf,
    }

    /// What one run of the stand-in was handed.
    #[derive(Debug)]
    struct Run {
        args: Vec<String>,
        env: Vec<String>,
        cwd: String,
        config_dir: String,
        config_held: String,
        files: String,
    }

    impl StandIn {
        fn at(dir: &Path) -> StandIn {
            let at = dir.display();
            let script = format!(
                "#!/bin/sh\n\
                 d='{at}'\n\
                 n=$(ls \"$d/runs\" | wc -l | tr -d ' ')\n\
                 r=\"$d/runs/$n\"\n\
                 mkdir \"$r\"\n\
                 for arg in \"$@\"; do printf '%s\\n' \"$arg\"; done > \"$r/args\"\n\
                 env > \"$r/env\"\n\
                 pwd -P > \"$r/cwd\"\n\
                 printf '%s' \"$GH_CONFIG_DIR\" > \"$r/config-dir\"\n\
                 ls -A \"$GH_CONFIG_DIR\" > \"$r/config-held\" 2>&1\n\
                 : > \"$r/files\"\n\
                 for arg in \"$@\"; do\n\
                 if [ -f \"$arg\" ]; then printf '%s=%s\\n' \"$(basename \"$arg\")\" \"$(cat \"$arg\")\" >> \"$r/files\"; fi\n\
                 done\n\
                 if [ -f \"$d/answers/$n.out\" ]; then cat \"$d/answers/$n.out\"; fi\n\
                 if [ -f \"$d/answers/$n.err\" ]; then cat \"$d/answers/$n.err\" >&2; fi\n\
                 if [ -f \"$d/answers/$n.code\" ]; then exit \"$(cat \"$d/answers/$n.code\")\"; fi\n\
                 exit 0\n"
            );
            for folder in ["runs", "answers"] {
                fs::create_dir_all(dir.join(folder)).unwrap();
            }
            let program = dir.join("gh");
            fs::write(&program, script).unwrap();
            fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
            StandIn {
                dir: dir.to_path_buf(),
            }
        }

        fn gh(&self) -> Gh {
            Gh::new(self.dir.join("gh"), &source())
        }

        /// Run `n` prints `out`, says `err` on its stderr and exits with
        /// `code`.
        fn answers(&self, n: usize, out: &str, err: &str, code: i32) {
            let answers = self.dir.join("answers");
            fs::write(answers.join(format!("{n}.out")), out).unwrap();
            fs::write(answers.join(format!("{n}.err")), err).unwrap();
            fs::write(answers.join(format!("{n}.code")), code.to_string()).unwrap();
        }

        /// Every run, in order.
        fn runs(&self) -> Vec<Run> {
            let read = |run: &Path, name: &str| fs::read_to_string(run.join(name)).unwrap();
            let count = fs::read_dir(self.dir.join("runs")).unwrap().count();
            (0..count)
                .map(|n| {
                    let run = self.dir.join("runs").join(n.to_string());
                    Run {
                        args: read(&run, "args").lines().map(str::to_string).collect(),
                        env: read(&run, "env").lines().map(str::to_string).collect(),
                        cwd: read(&run, "cwd").trim_end().to_string(),
                        config_dir: read(&run, "config-dir"),
                        config_held: read(&run, "config-held"),
                        files: read(&run, "files"),
                    }
                })
                .collect()
        }
    }

    fn stand_in() -> (TempDir, StandIn) {
        let dir = TempDir::new().unwrap();
        let stand_in = StandIn::at(dir.path());
        (dir, stand_in)
    }

    fn token() -> Token {
        Token::new(TOKEN)
    }

    fn at<'a>(repository: &'a RepositoryName, token: &'a Token) -> At<'a> {
        At { repository, token }
    }

    /// The folder `run` was handed its configuration folder in, which the
    /// operation made.
    fn folder_of(run: &Run) -> PathBuf {
        let config = Path::new(&run.config_dir);
        assert_eq!(config.file_name().unwrap(), "config", "{run:?}");
        config.parent().unwrap().to_path_buf()
    }

    /// `run`'s arguments, its folder's paths written `<run>`.
    fn args(run: &Run) -> Vec<String> {
        let folder = folder_of(run).display().to_string();
        run.args
            .iter()
            .map(|a| a.replace(&folder, "<run>"))
            .collect()
    }

    /// Each of `runs` ran in a folder of its own under the temporary
    /// directory, removed once its operation ended, handed an empty
    /// configuration folder there, the token in `GH_TOKEN` and in no
    /// argument, no prompt and no update notice, and no `GH_*` or
    /// `GITHUB_*` variable of the host's.
    fn of_their_own(runs: &[Run]) {
        let temp = fs::canonicalize(std::env::temp_dir()).unwrap();
        for run in runs {
            assert_eq!(
                run.config_held, "",
                "a run's folder held something: {run:?}"
            );
            let folder = folder_of(run);
            assert!(
                folder.starts_with(std::env::temp_dir()),
                "{} is not under the temporary directory",
                folder.display()
            );
            assert!(
                Path::new(&run.cwd).starts_with(&temp),
                "the run ran in {}",
                run.cwd
            );
            assert!(!folder.exists(), "{} was left behind", folder.display());
            let mut theirs: Vec<&str> = run
                .env
                .iter()
                .map(String::as_str)
                .filter(|l| l.starts_with("GH_") || l.starts_with("GITHUB_"))
                .collect();
            theirs.sort();
            assert_eq!(
                theirs,
                [
                    format!("GH_CONFIG_DIR={}", run.config_dir).as_str(),
                    "GH_NO_UPDATE_NOTIFIER=1",
                    "GH_PROMPT_DISABLED=1",
                    format!("GH_TOKEN={TOKEN}").as_str(),
                ],
                "{run:?}"
            );
            assert!(!run.args.iter().any(|a| a.contains(TOKEN)), "{run:?}");
        }
    }

    fn repo_args(subcommand: &[&str], after: &[&str]) -> Vec<String> {
        let mut args: Vec<String> = subcommand.iter().map(|a| a.to_string()).collect();
        args.extend(["--repo", REPO].map(str::to_string));
        args.extend(after.iter().map(|a| a.to_string()));
        args
    }

    fn api_args(path: &str) -> Vec<String> {
        ["api", "--hostname", "github.com", path]
            .map(str::to_string)
            .to_vec()
    }

    const NOT_FOUND_API: &str = "gh: Not Found (HTTP 404)\n";

    fn ref_of(sha: &str, kind: &str) -> String {
        format!(
            "{{\"ref\":\"refs/tags/{TAG}\",\"object\":{{\"sha\":\"{sha}\",\"type\":\"{kind}\",\
             \"url\":\"https://api.github.com/x\"}}}}"
        )
    }

    // Requirement: Apply releases only the tag GitHub holds at the
    // version's revision

    #[test]
    fn a_lightweight_tag_is_read_by_its_ref_on_github_com() {
        let (_dir, stand_in) = stand_in();
        stand_in.answers(0, &ref_of(COMMIT, "commit"), "", 0);
        let (repository, token) = (repository(), token());
        let read = stand_in.gh().tag(&at(&repository, &token), TAG);
        assert_eq!(read, Ok(Some(Revision::new(COMMIT))));
        let runs = stand_in.runs();
        assert_eq!(runs.len(), 1);
        assert_eq!(
            args(&runs[0]),
            api_args("repos/boomdrivedev/flume/git/ref/tags/v0.31.0")
        );
        of_their_own(&runs);
    }

    #[test]
    fn a_tag_object_is_peeled_to_its_commit() {
        let (_dir, stand_in) = stand_in();
        let object = "1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d";
        stand_in.answers(0, &ref_of(object, "tag"), "", 0);
        let peeled = format!(
            "{{\"sha\":\"{object}\",\"object\":{{\"sha\":\"{COMMIT}\",\"type\":\"commit\"}}}}"
        );
        stand_in.answers(1, &peeled, "", 0);
        let (repository, token) = (repository(), token());
        let read = stand_in.gh().tag(&at(&repository, &token), TAG);
        assert_eq!(read, Ok(Some(Revision::new(COMMIT))));
        let runs = stand_in.runs();
        assert_eq!(runs.len(), 2);
        assert_eq!(
            args(&runs[1]),
            api_args(&format!("repos/boomdrivedev/flume/git/tags/{object}"))
        );
        of_their_own(&runs);
    }

    #[test]
    fn a_tag_github_does_not_hold_is_none() {
        let (_dir, stand_in) = stand_in();
        let body = "{\"message\":\"Not Found\",\"status\":\"404\"}";
        stand_in.answers(0, body, NOT_FOUND_API, 1);
        let (repository, token) = (repository(), token());
        assert_eq!(stand_in.gh().tag(&at(&repository, &token), TAG), Ok(None));
    }

    // Requirement: Apply makes a release the tag lacks as a draft

    #[test]
    fn a_release_is_viewed_by_its_tag_with_each_assets_name_state_and_digest() {
        let (_dir, stand_in) = stand_in();
        let digest = Digest::of(b"linux");
        let viewed = serde_json::json!({
            "tagName": TAG,
            "isDraft": true,
            "isPrerelease": false,
            "assets": [
                { "name": LINUX, "state": "uploaded", "digest": digest.as_str(), "size": 5 },
                { "name": DARWIN, "state": "starter", "digest": null },
                { "name": "checksums.txt", "state": "uploaded" },
            ],
        });
        stand_in.answers(0, &viewed.to_string(), "", 0);
        let (repository, token) = (repository(), token());
        let release = stand_in.gh().release(&at(&repository, &token), TAG);
        let asset = |name: &str, state: &str, digest: Option<Digest>| Asset {
            name: name.into(),
            state: state.into(),
            digest,
        };
        assert_eq!(
            release,
            Ok(Some(Release {
                assets: vec![
                    asset(LINUX, "uploaded", Some(digest)),
                    asset(DARWIN, "starter", None),
                    asset("checksums.txt", "uploaded", None),
                ],
            }))
        );
        let runs = stand_in.runs();
        assert_eq!(
            args(&runs[0]),
            repo_args(
                &["release", "view", TAG],
                &["--json", "tagName,isDraft,isPrerelease,assets"]
            )
        );
        of_their_own(&runs);
    }

    #[test]
    fn a_tag_with_no_release_is_none() {
        let (_dir, stand_in) = stand_in();
        stand_in.answers(0, "", "release not found\n", 1);
        let (repository, token) = (repository(), token());
        assert_eq!(
            stand_in.gh().release(&at(&repository, &token), TAG),
            Ok(None)
        );
    }

    #[test]
    fn a_draft_is_made_only_on_the_tag_titled_by_it_with_the_notes() {
        let (_dir, stand_in) = stand_in();
        let (repository, token) = (repository(), token());
        let notes = "## 0.31.0\n\n- flume grows\n";
        let made = stand_in
            .gh()
            .create(&at(&repository, &token), TAG, notes.as_bytes());
        assert_eq!(made, Ok(()));
        let runs = stand_in.runs();
        assert_eq!(
            args(&runs[0]),
            repo_args(
                &["release", "create", TAG],
                &[
                    "--verify-tag",
                    "--draft",
                    "--title",
                    TAG,
                    "--notes-file",
                    "<run>/notes.md"
                ]
            )
        );
        assert_eq!(runs[0].files, format!("notes.md={}\n", notes.trim_end()));
        of_their_own(&runs);
    }

    // Requirement: Apply attaches each asset the release does not hold

    #[test]
    fn assets_are_uploaded_in_one_run_each_under_its_outputs_name() {
        let (_dir, stand_in) = stand_in();
        let (repository, token) = (repository(), token());
        let (linux, darwin) = (OutputName::new(LINUX), OutputName::new(DARWIN));
        let uploads = [
            Upload {
                output: &linux,
                bytes: b"linux bytes",
            },
            Upload {
                output: &darwin,
                bytes: b"darwin bytes",
            },
        ];
        let uploaded = stand_in
            .gh()
            .upload(&at(&repository, &token), TAG, &uploads);
        assert_eq!(uploaded, Ok(()));
        let runs = stand_in.runs();
        assert_eq!(runs.len(), 1);
        assert_eq!(
            args(&runs[0]),
            repo_args(
                &["release", "upload", TAG],
                &[
                    "--clobber",
                    &format!("<run>/assets/{LINUX}"),
                    &format!("<run>/assets/{DARWIN}"),
                ]
            )
        );
        assert_eq!(
            runs[0].files,
            format!("{LINUX}=linux bytes\n{DARWIN}=darwin bytes\n")
        );
        of_their_own(&runs);
    }

    // Requirement: Apply publishes the release last, marked latest

    #[test]
    fn a_release_is_published_on_the_tag_not_a_prerelease_and_marked_latest() {
        let (_dir, stand_in) = stand_in();
        let (repository, token) = (repository(), token());
        let notes = "## 0.31.0\n";
        let published = stand_in
            .gh()
            .publish(&at(&repository, &token), TAG, notes.as_bytes());
        assert_eq!(published, Ok(()));
        let runs = stand_in.runs();
        assert_eq!(
            args(&runs[0]),
            repo_args(
                &["release", "edit", TAG],
                &[
                    "--verify-tag",
                    "--draft=false",
                    "--prerelease=false",
                    "--latest",
                    "--title",
                    TAG,
                    "--notes-file",
                    "<run>/notes.md"
                ]
            )
        );
        assert_eq!(runs[0].files, "notes.md=## 0.31.0\n");
        of_their_own(&runs);
    }

    // Requirement: current is the version of the release GitHub marks
    // latest

    #[test]
    fn the_latest_release_is_read_with_its_tag_and_when_it_was_updated() {
        let (_dir, stand_in) = stand_in();
        let latest =
            "{\"tag_name\":\"v0.31.0\",\"updated_at\":\"2026-11-18T20:22:59Z\",\"draft\":false}";
        stand_in.answers(0, latest, "", 0);
        stand_in.answers(1, "{\"tag_name\":\"v0.30.0\"}", "", 0);
        stand_in.answers(2, "{\"message\":\"Not Found\"}", NOT_FOUND_API, 1);
        let (repository, token) = (repository(), token());
        let gh = stand_in.gh();
        let at = at(&repository, &token);
        assert_eq!(
            gh.latest(&at),
            Ok(Some(Latest {
                tag: TAG.into(),
                updated_at: Some("2026-11-18T20:22:59Z".into()),
            }))
        );
        assert_eq!(
            gh.latest(&at),
            Ok(Some(Latest {
                tag: "v0.30.0".into(),
                updated_at: None,
            }))
        );
        assert_eq!(gh.latest(&at), Ok(None));
        let runs = stand_in.runs();
        assert_eq!(
            args(&runs[0]),
            api_args("repos/boomdrivedev/flume/releases/latest")
        );
        of_their_own(&runs);
    }

    // Requirement: GitHub's words are the answer

    #[test]
    fn another_failure_is_unreadable_naming_the_command_and_carrying_ghs_words() {
        let (_dir, stand_in) = stand_in();
        let words = "HTTP 403: Resource not accessible by personal access token \
                     (https://api.github.com/repos/boomdrivedev/flume/releases)";
        stand_in.answers(0, "", &format!("{words}\n"), 1);
        stand_in.answers(1, "", "gh: Bad credentials (HTTP 401)\n", 1);
        // Each wording of a missing thing is none only where its operation
        // answers it.
        stand_in.answers(2, "", NOT_FOUND_API, 1);
        stand_in.answers(3, "", "release not found\n", 1);
        let (repository, token) = (repository(), token());
        let gh = stand_in.gh();
        let at = at(&repository, &token);
        let program = stand_in.dir.join("gh").display().to_string();
        for (answer, said, command) in [
            (
                gh.create(&at, TAG, b"notes"),
                words,
                "release create v0.31.0",
            ),
            (
                gh.tag(&at, TAG).map(drop),
                "gh: Bad credentials (HTTP 401)",
                "api --hostname github.com",
            ),
            (
                gh.release(&at, TAG).map(drop),
                "gh: Not Found (HTTP 404)",
                "release view v0.31.0",
            ),
            (
                gh.latest(&at).map(drop),
                "release not found",
                "api --hostname github.com",
            ),
        ] {
            match answer {
                Err(Not::Unreadable(u)) => {
                    assert_eq!(u.source, source());
                    assert_eq!(u.said.as_deref(), Some(said));
                    assert!(
                        u.what.starts_with(&format!("{program} {command}")),
                        "{}",
                        u.what
                    );
                    assert!(!u.what.contains(TOKEN), "{}", u.what);
                }
                other => panic!("answered {other:?}, not unreadable"),
            }
        }
    }

    // Requirement: No run of `gh` reads the host's own settings

    #[test]
    fn the_hosts_own_gh_settings_reach_no_run() {
        let (dir, stand_in) = stand_in();
        let hosts = dir.path().join("hosts-gh");
        fs::create_dir_all(&hosts).unwrap();
        fs::write(
            hosts.join("hosts.yml"),
            "github.com:\n  oauth_token: gho_host\n",
        )
        .unwrap();
        stand_in.answers(0, &ref_of(COMMIT, "commit"), "", 0);
        stand_in.answers(1, "", "release not found\n", 1);
        stand_in.answers(5, "", NOT_FOUND_API, 1);
        let host_config = hosts.display().to_string();
        let set = [
            ("GITHUB_TOKEN", "ghp_host"),
            ("GH_ENTERPRISE_TOKEN", "ghe_host"),
            ("GH_HOST", "github.example.com"),
            ("GH_REPO", "someone/else"),
            ("GH_CONFIG_DIR", host_config.as_str()),
        ];
        let (repository, token) = (repository(), token());
        let gh = stand_in.gh();
        let output = OutputName::new(LINUX);
        under_env(&set, || {
            let at = at(&repository, &token);
            gh.tag(&at, TAG).unwrap();
            gh.release(&at, TAG).unwrap();
            gh.create(&at, TAG, b"notes").unwrap();
            gh.upload(
                &at,
                TAG,
                &[Upload {
                    output: &output,
                    bytes: b"bytes",
                }],
            )
            .unwrap();
            gh.publish(&at, TAG, b"notes").unwrap();
            gh.latest(&at).unwrap();
        });
        let runs = stand_in.runs();
        assert_eq!(runs.len(), 6, "{runs:?}");
        of_their_own(&runs);
        for run in &runs {
            assert_ne!(run.config_dir, host_config, "{run:?}");
            for theirs in ["ghp_host", "ghe_host", "github.example.com", "someone/else"] {
                assert!(
                    !run.env.iter().any(|l| l.contains(theirs)),
                    "the run saw {theirs}: {run:?}"
                );
            }
        }
    }

    #[test]
    fn a_tokens_debug_print_holds_no_token() {
        assert_eq!(format!("{:?}", token()), "Token(…)");
    }
}
