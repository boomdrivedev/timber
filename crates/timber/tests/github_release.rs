//! Spec github-release-target, its run against one repository (design
//! D11): the throwaway repository the host names in
//! `env.TIMBER_TOOLS_GITHUB_RELEASE` of its `devenv.local.nix`, as
//! `<owner>/<name>#<token file>`, where `<owner>/<name>` is a repository on
//! github.com no one uses, and `<token file>` a file holding only a token
//! that may write its contents, which a person makes. The `gh` is the one
//! the tree pins, `TIMBER_TOOLS_GH_CLI`, never the `gh` on a person's
//! path.
//!
//! The repository is cloned into the test's directory with the host's own
//! git credential, and three versions, A, B and C, of one component are
//! published from three commits on a branch of the run's own, through
//! the core over the fakes' build and the file projection: each with its
//! notes and the outputs `timber-proof-a.txt` and `timber-proof-b.txt`,
//! C's `-b` of other bytes. Each publish pushes its tag to the
//! repository. The target group runs over the profile; then A, B and A
//! again are applied, each release's text, assets, digests and draft
//! state, the release GitHub marks latest, current and report checked
//! after each, and the second apply of A uploading nothing. A draft of
//! C's tag is made with `gh`, as an apply that stopped after its draft
//! leaves it, and C applied: C's tag has one release, published, latest,
//! holding both assets. Every release, tag and branch the run made is
//! removed at its end, through `gh` itself, whatever it answered.
//!
//! Its token is a file a person makes, so it runs by hand under
//! `check-github-release` and in no gate task (rule core.38), and it
//! refuses to run, naming the setting, on a host that names no
//! repository, rather than pass without running.

#![cfg(feature = "tools")]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output as Ran, Stdio};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tempfile::TempDir;
use timber::bindings::Credentials;
use timber::profiles::projection::file::FileProjection;
use timber::profiles::target::file_artifact::{self, Words};
use timber::profiles::target::github_release::{self, GithubRelease};
use timber_conformance::seams::target::{self, TargetRig};
use timber_conformance::Rig;
use timber_core::{publish, Ask, History, Kept, Published};
use timber_fakes::{FakeStore, Fakes};
use timber_seams::{
    version_name, Answer, Branch, BuildId, ChannelId, ChannelName, ComponentName, Digest,
    Ecosystem, EcosystemName, Envelope, Environment, EnvironmentName, Held, Manifest, Name,
    Outcome, OutputName, Projection, Put, Report, RepositoryName, Revision, Seam, SignerId, Source,
    Store, Target, Verified, Version,
};

/// The setting a host names its throwaway repository by.
const SETTING: &str = "TIMBER_TOOLS_GITHUB_RELEASE";

/// The setting naming the `gh` the tree pins.
const CLI: &str = "TIMBER_TOOLS_GH_CLI";

/// The component the versions are of, and its two outputs.
const COMPONENT: &str = "timber-proof";
const OUTPUT_A: &str = "timber-proof-a.txt";
const OUTPUT_B: &str = "timber-proof-b.txt";

/// The credential's name, as the environments' tables give it.
const CREDENTIAL: &str = "timber-proof";

/// A credential whose file is not there, which the unreadable
/// environment names.
const NO_TOKEN: &str = "timber-proof-no-token";

/// The identity the publishes sign as.
const HOST: &str = "host";

/// The repository the host names, and the token's file; a panic naming
/// the setting where it names none.
struct Setting {
    repository: RepositoryName,
    token: PathBuf,
}

fn setting() -> Setting {
    let named = std::env::var(SETTING).unwrap_or_default();
    let read = named
        .trim()
        .split_once('#')
        .and_then(|(repository, token)| {
            let (owner, name) = repository.split_once('/')?;
            let named = !owner.is_empty() && !name.is_empty() && !name.contains('/');
            (named && !token.is_empty()).then(|| Setting {
                repository: RepositoryName {
                    owner: owner.into(),
                    name: name.into(),
                },
                token: PathBuf::from(token),
            })
        });
    read.unwrap_or_else(|| {
        panic!(
            "the GitHub release run names no repository: set env.{SETTING} in devenv.local.nix \
             to `<owner>/<name>#<token file>`, a throwaway repository on github.com no one uses, \
             and a file holding only a token that may write its contents"
        )
    })
}

/// The `gh` the tree pins; a panic naming its setting where the shell
/// names none.
fn cli() -> PathBuf {
    match std::env::var_os(CLI) {
        Some(cli) if !cli.is_empty() => PathBuf::from(cli),
        _ => panic!("{CLI} names no gh: run the proof in the repository's devenv shell"),
    }
}

/// git run in `dir` with the host's own credential and no prompt,
/// answering what it printed; a panic with its words where it fails.
fn git(dir: &Path, args: &[&str]) -> String {
    let ran = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_AUTHOR_NAME", "timber's proof")
        .env("GIT_AUTHOR_EMAIL", "proof@timber.invalid")
        .env("GIT_COMMITTER_NAME", "timber's proof")
        .env("GIT_COMMITTER_EMAIL", "proof@timber.invalid")
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("git could not be started: {e}"));
    assert!(
        ran.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&ran.stderr)
    );
    String::from_utf8_lossy(&ran.stdout).trim().to_string()
}

/// A store that answers a read of a digest it was told stands for
/// another with what that other holds, so the suite can hand the target
/// a version at a digest of its own choosing.
#[derive(Clone)]
struct Aliased {
    store: FakeStore,
    aliases: Rc<RefCell<BTreeMap<Digest, Digest>>>,
}

impl Store for Aliased {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        self.store.put_bytes(bytes)
    }
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        self.store.put_artifact(manifest)
    }
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        self.store.attach(subject, attestation)
    }
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        self.store.claims_of(digest)
    }
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        self.store.name(name, to, expected)
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.store.resolve(name)
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        let to = self.aliases.borrow().get(digest).cloned();
        self.store.read(to.as_ref().unwrap_or(digest))
    }
    fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>> {
        self.store.versions(ecosystem, name)
    }
}

/// One version published: as the core verified it, its notes, and each
/// output's bytes.
struct Made {
    verified: Verified,
    notes: String,
    outputs: Vec<(&'static str, Vec<u8>)>,
}

/// The repository the run stands on: the setting and its token, the
/// clone, the three versions published, the target, and the branch and
/// tags the run made, removed with their releases when it is dropped.
struct Proof {
    dir: TempDir,
    setting: Setting,
    token: String,
    branch: String,
    versions: [Made; 3],
    store: Aliased,
    target: GithubRelease,
    /// The versions the group's rig hands out in turn, A first.
    handed: Cell<usize>,
}

impl Proof {
    fn new() -> Proof {
        let setting = setting();
        let gh = cli();
        let token = fs::read_to_string(&setting.token)
            .unwrap_or_else(|e| panic!("{SETTING} names {}: {e}", setting.token.display()))
            .trim_end()
            .to_string();
        let dir = TempDir::new().unwrap();
        let run = format!(
            "{:x}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let branch = format!("timber-proof/{run}");
        let prefix = format!("timber-proof-{run}-v");
        let clone = dir.path().join("clone");
        let address = format!("https://github.com/{}.git", setting.repository);
        git(
            dir.path(),
            &["clone", "-q", &address, clone.to_str().unwrap()],
        );
        git(&clone, &["checkout", "-q", "-b", &branch]);

        let fakes = Fakes::new();
        fakes.signer.hold(&SignerId::new(HOST));
        let store = Aliased {
            store: fakes.store.clone(),
            aliases: Rc::default(),
        };
        let mut published = Vec::new();
        for (n, b) in ["one", "one", "two"].into_iter().enumerate() {
            published.push(Proof::publish(
                &fakes,
                &clone,
                &branch,
                &prefix,
                n,
                b,
                dir.path(),
            ));
        }
        let versions: [Made; 3] = published.try_into().ok().unwrap();

        let credentials = Credentials {
            file: dir.path().join("config.toml"),
            paths: BTreeMap::from([
                (CREDENTIAL.to_string(), setting.token.clone()),
                (NO_TOKEN.to_string(), dir.path().join("no-token")),
            ]),
        };
        let target = GithubRelease::with_program(
            Box::new(store.clone()),
            Ok(setting.repository.clone()),
            &credentials,
            gh,
        );
        Proof {
            dir,
            setting,
            token,
            branch,
            versions,
            store,
            target,
            handed: Cell::new(0),
        }
    }

    /// The `n`-th version published through the core from a commit of its
    /// own on `branch`, its `-b` output holding `b`: the policy file
    /// declares the component on the first, each commit is a feature of
    /// the component pushed to the repository, and the publish pushes the
    /// version's tag there (22).
    fn publish(
        fakes: &Fakes,
        clone: &Path,
        branch: &str,
        prefix: &str,
        n: usize,
        b: &str,
        dir: &Path,
    ) -> Made {
        if n == 0 {
            let policy = format!(
                "[[component]]\nname = \"{COMPONENT}\"\n\
                 outputs = [\"{OUTPUT_A}\", \"{OUTPUT_B}\"]\n\
                 paths = [\"timber-proof/**\"]\nprefix = \"{prefix}\"\n\
                 projections = {{ \"{OUTPUT_A}\" = \"file\", \"{OUTPUT_B}\" = \"file\" }}\n"
            );
            fs::create_dir_all(clone.join(".config")).unwrap();
            fs::write(clone.join(".config/timber.toml"), policy).unwrap();
        }
        fs::create_dir_all(clone.join("timber-proof")).unwrap();
        fs::write(
            clone.join(format!("timber-proof/{n}.txt")),
            format!("commit {n}\n"),
        )
        .unwrap();
        git(clone, &["add", "-A"]);
        git(
            clone,
            &[
                "commit",
                "-q",
                "-m",
                &format!("feat: timber's proof grows {n}"),
            ],
        );
        git(clone, &["push", "-q", "-u", "origin", branch]);
        let revision = Revision::new(git(clone, &["rev-parse", "HEAD"]));

        let outputs = vec![
            (OUTPUT_A, b"a, made by timber's proof\n".to_vec()),
            (
                OUTPUT_B,
                format!("b {b}, made by timber's proof\n").into_bytes(),
            ),
        ];
        let built = dir.join(format!("built-{n}"));
        fs::create_dir_all(&built).unwrap();
        for (output, bytes) in &outputs {
            let file = built.join(output);
            fs::write(&file, bytes).unwrap();
            fakes.build.made(
                &revision,
                &OutputName::new(*output),
                Report {
                    build: BuildId::new(format!("{output}@{}", &revision.as_str()[..12])),
                    outcome: Outcome::Pass,
                    files: vec![file.display().to_string()],
                },
            );
        }
        let branch = Branch::new(branch);
        fakes.landing.land(&branch, &revision);
        let mut seams = fakes.seams();
        seams.projections = BTreeMap::from([(
            Ecosystem::new("file"),
            Box::new(FileProjection::new()) as Box<dyn Projection>,
        )]);
        let history = History::open(clone).expect("the clone opens");
        let ask = Ask {
            branch,
            revision,
            component: Some(ComponentName::new(COMPONENT)),
            signer: SignerId::new(HOST),
            store: Source::new(Seam::Store, timber_fakes::PROFILE),
            tree: clone.display().to_string(),
        };
        let mut answered = publish(&seams, &history, &Kept::at(&history), &ask)
            .expect("the publish answered")
            .components;
        assert_eq!(answered.len(), 1, "{answered:?}");
        let Ok(Published::Version {
            version,
            tag,
            index,
            ..
        }) = answered.remove(0).1
        else {
            panic!("version {n} was not published")
        };
        assert!(tag.starts_with(prefix), "{tag}");
        let verified = Verified::new(index, version, tag);
        let notes = notes(&fakes.store, &verified);
        Made {
            verified,
            notes,
            outputs,
        }
    }

    fn repo(&self) -> String {
        format!("github.com/{}", self.setting.repository)
    }

    /// The `gh` the tree pins run with `args` as the profile runs it: an
    /// empty configuration folder of its own, no `GH_*` or `GITHUB_*`
    /// variable of the host's, and the token in `GH_TOKEN`.
    fn gh(&self, args: &[&str]) -> Ran {
        self.try_gh(args)
            .unwrap_or_else(|e| panic!("{} could not be started: {e}", cli().display()))
    }

    /// [`Proof::gh`], answering why where `gh` could not be started.
    fn try_gh(&self, args: &[&str]) -> std::io::Result<Ran> {
        let config = tempfile::TempDir::new()?;
        let mut command = Command::new(cli());
        for (name, _) in std::env::vars_os() {
            let text = name.to_string_lossy();
            if text.starts_with("GH_") || text.starts_with("GITHUB_") {
                command.env_remove(&name);
            }
        }
        command
            .args(args)
            .env("GH_CONFIG_DIR", config.path())
            .env("GH_TOKEN", &self.token)
            .env("GH_PROMPT_DISABLED", "1")
            .env("GH_NO_UPDATE_NOTIFIER", "1")
            .stdin(Stdio::null())
            .output()
    }

    /// What `gh` printed, as JSON, where it exits 0; a panic with its
    /// words otherwise.
    fn gh_json(&self, args: &[&str]) -> Value {
        let ran = self.gh(args);
        assert!(
            ran.status.success(),
            "gh {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&ran.stderr)
        );
        serde_json::from_slice(&ran.stdout).unwrap()
    }

    /// The release of `tag` as GitHub holds it.
    fn release(&self, tag: &str) -> Value {
        self.gh_json(&[
            "release",
            "view",
            tag,
            "--repo",
            &self.repo(),
            "--json",
            "tagName,name,body,isDraft,isPrerelease,assets",
        ])
    }

    /// The tag of the release GitHub marks latest, where it marks one.
    fn latest(&self) -> Option<String> {
        let path = format!("repos/{}/releases/latest", self.setting.repository);
        let ran = self.gh(&["api", "--hostname", "github.com", &path]);
        if !ran.status.success() {
            let words = String::from_utf8_lossy(&ran.stderr);
            assert!(words.contains("Not Found (HTTP 404)"), "{words}");
            return None;
        }
        let latest: Value = serde_json::from_slice(&ran.stdout).unwrap();
        latest["tag_name"].as_str().map(str::to_string)
    }

    /// How many releases, drafts among them, the repository holds of
    /// `tag`, or `gh`'s words where it cannot say.
    fn releases_of(&self, tag: &str) -> Result<usize, String> {
        let path = format!("repos/{}/releases?per_page=100", self.setting.repository);
        let filter = format!(".[] | select(.tag_name == \"{tag}\") | .id");
        let args = [
            "api",
            "--hostname",
            "github.com",
            "--paginate",
            &path,
            "--jq",
            &filter,
        ];
        let ran = self.try_gh(&args).map_err(|e| e.to_string())?;
        if !ran.status.success() {
            return Err(String::from_utf8_lossy(&ran.stderr).into_owned());
        }
        Ok(String::from_utf8_lossy(&ran.stdout).lines().count())
    }

    /// The ids of the assets of `tag`'s release, by name.
    fn asset_ids(&self, tag: &str) -> BTreeMap<String, Value> {
        let release = self.release(tag);
        release["assets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| (a["name"].as_str().unwrap().to_string(), a["id"].clone()))
            .collect()
    }

    /// The environment `name`, following the component's channel, its
    /// release carrying both outputs, with the token `credential` names.
    fn environment(&self, name: &str, credential: &str) -> Environment {
        Environment {
            name: EnvironmentName::new(name),
            channel: ChannelId::new(
                Some(ComponentName::new(COMPONENT)),
                ChannelName::new("stable"),
            ),
            waits: false,
            settings: json!({ "credential": credential, "assets": [OUTPUT_A, OUTPUT_B] }),
        }
    }

    /// The store names version `n` at `digest`, as the suite hands it.
    fn named_at(&self, n: usize, digest: &Digest) {
        let verified = &self.versions[n].verified;
        let name = version_name(&ComponentName::new(COMPONENT), verified.tag());
        let at = self.store.resolve(&name).unwrap();
        self.store.name(&name, digest, Some(&at)).unwrap();
    }

    /// Version `n`'s release is published, not a prerelease, titled by
    /// its tag, its text the version's notes, holding each output under
    /// its name with its bytes' digest, and GitHub marks it latest;
    /// current and report answer its index.
    fn released(&self, environment: &Environment, n: usize, after: &str) {
        let version = &self.versions[n];
        let tag = version.verified.tag();
        let release = self.release(tag);
        assert_eq!(release["isDraft"], false, "{after}: {release}");
        assert_eq!(release["isPrerelease"], false, "{after}: {release}");
        assert_eq!(release["name"], tag, "{after}: {release}");
        let body = release["body"].as_str().unwrap().replace("\r\n", "\n");
        assert_eq!(body.trim_end(), version.notes.trim_end(), "{after}");
        let assets = release["assets"].as_array().unwrap();
        for (output, bytes) in &version.outputs {
            let asset = assets
                .iter()
                .find(|a| a["name"] == *output)
                .unwrap_or_else(|| panic!("{after}: the release holds no {output}: {release}"));
            assert_eq!(asset["state"], "uploaded", "{after}: {asset}");
            assert_eq!(
                asset["digest"],
                Digest::of(bytes).as_str(),
                "{after}: {asset}"
            );
        }
        assert_eq!(self.latest().as_deref(), Some(tag), "{after}");
        let index = version.verified.index();
        assert_eq!(
            self.target.current(environment),
            Ok(index.clone()),
            "{after}"
        );
        let running = self
            .target
            .report(environment)
            .unwrap_or_else(|not| panic!("report after {after}: {not:?}"));
        assert_eq!(&running.digest, index, "{after}");
        assert!(!running.converged_at.is_empty(), "{after}");
    }
}

/// The notes `verified`'s index names, as the store holds them.
fn notes(store: &FakeStore, verified: &Verified) -> String {
    let source = GithubRelease::source();
    let words = Words {
        file: "notes",
        carries: "notes",
    };
    let index = file_artifact::index(store, verified, &source).unwrap();
    let artifact = file_artifact::notes_of(&index, words, &source).unwrap();
    String::from_utf8(file_artifact::bytes(store, artifact, words, &source).unwrap()).unwrap()
}

impl Drop for Proof {
    /// Every release, tag and branch the run made removed, through `gh`
    /// itself, whatever the run answered.
    fn drop(&mut self) {
        let repo = self.repo();
        let repository = self.setting.repository.to_string();
        for version in &self.versions {
            let tag = version.verified.tag();
            let releases = self.releases_of(tag).unwrap_or_else(|words| {
                eprintln!("the releases of {tag} were not read: {words}");
                0
            });
            for _ in 0..releases {
                self.removed(
                    &format!("the release of {tag}"),
                    &["release", "delete", tag, "--repo", &repo, "--yes"],
                );
            }
            let path = format!("repos/{repository}/git/refs/tags/{tag}");
            self.removed(
                &path,
                &["api", "--hostname", "github.com", "-X", "DELETE", &path],
            );
        }
        let path = format!("repos/{repository}/git/refs/heads/{}", self.branch);
        self.removed(
            &path,
            &["api", "--hostname", "github.com", "-X", "DELETE", &path],
        );
    }
}

impl Proof {
    /// `what` removed with `gh` run with `args`, saying so where it was
    /// not, and never panicking, since a drop runs while a failed run
    /// unwinds.
    fn removed(&self, what: &str, args: &[&str]) {
        match self.try_gh(args) {
            Ok(ran) if ran.status.success() => {}
            Ok(ran) => eprintln!(
                "{what} was not removed: {}",
                String::from_utf8_lossy(&ran.stderr)
            ),
            Err(e) => eprintln!("{what} was not removed: gh could not be started: {e}"),
        }
    }
}

/// The target group's rig over the profile and the repository: each
/// environment is one of its own, the unreadable one names a credential
/// whose file is not there, and the foreign settings add a `token`; each
/// version handed is the next of A and B, so no scenario reads another's
/// release as its own.
struct GithubRig {
    proof: Rc<Proof>,
    environments: Cell<u32>,
}

impl GithubRig {
    fn next(&self, credential: &str) -> Environment {
        self.environments.set(self.environments.get() + 1);
        self.proof
            .environment(&format!("group-{}", self.environments.get()), credential)
    }
}

impl Rig for GithubRig {
    fn profile(&self) -> String {
        github_release::PROFILE.into()
    }
}

impl TargetRig for GithubRig {
    fn target(&self) -> &dyn Target {
        &self.proof.target
    }

    /// The next of A and B, read at `digest`, the store naming its version
    /// at `digest` as it names a version at its index.
    fn verified(&self, digest: &Digest) -> Verified {
        let n = self.proof.handed.get();
        assert!(n < 2, "the group asked for a third version");
        self.proof.handed.set(n + 1);
        let version = &self.proof.versions[n].verified;
        self.proof
            .store
            .aliases
            .borrow_mut()
            .insert(digest.clone(), version.index().clone());
        self.proof.named_at(n, digest);
        Verified::new(digest.clone(), version.version().clone(), version.tag())
    }

    fn environment(&self, _head: Option<&Digest>) -> Environment {
        self.next(CREDENTIAL)
    }

    fn unreadable_environment(&self) -> (Environment, String) {
        let missing = self.proof.dir.path().join("no-token");
        let words = fs::read_to_string(&missing).unwrap_err().to_string();
        (self.next(NO_TOKEN), words)
    }

    fn foreign_settings(&self) -> (Environment, String) {
        let mut environment = self.next(CREDENTIAL);
        environment.settings["token"] = json!("ghp_not_a_token");
        (environment, "token".into())
    }
}

#[test]
fn the_target_group_passes_and_a_repository_releases_a_b_a_and_finishes_cs_draft() {
    let proof = Rc::new(Proof::new());

    let report = timber_conformance::run(target::GROUP, &target::scenarios(), &|| {
        Box::new(GithubRig {
            proof: Rc::clone(&proof),
            environments: Cell::new(0),
        }) as Box<dyn TargetRig>
    });
    report.assert();
    let inapplicable: Vec<&str> = report.not_applicable.iter().map(|n| n.scenario).collect();
    assert_eq!(
        inapplicable,
        ["applied where no marker is held"],
        "{report}"
    );
    // The store names each version at its own index again.
    for n in 0..2 {
        proof.named_at(n, &proof.versions[n].verified.index().clone());
    }

    let environment = proof.environment("proof", CREDENTIAL);
    let mut held = BTreeMap::new();
    for (step, n) in [0, 1, 0].into_iter().enumerate() {
        let version = &proof.versions[n].verified;
        let named = format!("apply {} of {}", step + 1, version.version());
        proof
            .target
            .apply(&environment, version)
            .unwrap_or_else(|not| panic!("{named}: {not:?}"));
        proof.released(&environment, n, &named);
        if step == 0 {
            held = proof.asset_ids(version.tag());
        }
    }
    assert_eq!(
        proof.asset_ids(proof.versions[0].verified.tag()),
        held,
        "the second apply of A uploaded its assets again"
    );

    // A draft of C's tag, as an apply that stopped after making it leaves
    // it, finished by C's apply.
    let c = &proof.versions[2].verified;
    let made = proof.gh(&[
        "release",
        "create",
        c.tag(),
        "--repo",
        &proof.repo(),
        "--verify-tag",
        "--draft",
        "--title",
        c.tag(),
        "--notes",
        "a draft an apply left",
    ]);
    assert!(
        made.status.success(),
        "the draft of {}: {}",
        c.tag(),
        String::from_utf8_lossy(&made.stderr)
    );
    proof
        .target
        .apply(&environment, c)
        .unwrap_or_else(|not| panic!("the apply of C: {not:?}"));
    proof.released(&environment, 2, "the apply of C over its draft");
    assert_eq!(
        proof.releases_of(c.tag()),
        Ok(1),
        "C's tag has another release"
    );
}
