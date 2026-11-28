//! Spec command-line: the built binary's main paths, the policy command
//! over the policy file read with git, and a sync finding the host's
//! serving process (design D11, D12); each command's answers asked
//! through `run` over the fakes are the unit tests'
//! (`src/commands/command_line.rs`).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;
use serde_json::Value;
use tempfile::TempDir;
use timber::bindings::Teller;
use timber::commands::serve::host::{self as serving, Host as ServingHost};
use timber::commands::sync::{run_on as sync_on, SyncArgs};
use timber::profiles::policy::file::PolicyFile;
use timber::{run, Binding, Cli};
use timber_core::{
    accept, answer_json, artifact_name, in_force, publish, AcceptAsk, Ask, Entry, History,
    InForceAsk, Judged, Kept, Moved, RepositoryName, ToldBy,
};
use timber_fakes::{FakeSharedHistory, Fakes};
use timber_seams::{
    Answer, Artifact, ArtifactMeta, AskedOf, Audience, Branch, ChannelId, ChannelName, Claim,
    ClaimKind, ComponentName, Digest, Ecosystem, EcosystemName, IndexMeta, Manifest, NamedSigner,
    OutputName, Pinned, Promotion, PushedRef, Report, Required, RequiredClaim, Revision, Seam,
    SharedHistory, Signer, SignerId, Source, Store, Version,
};

/// The store as the host's bindings name it, which the record keeps.
fn store() -> Source {
    Source::new(Seam::Store, "layout /srv/timber/layout")
}

/// The job the host's bindings name as what tells the gate, so no
/// publish here asks it.
const JOB: &str = "the gate's job in another place";

/// The one component of the repository, its one output packed in `eco`.
const POLICY: &str = "outputs = [\"site\"]\n\n[projections]\nsite = \"eco\"\n";

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "A. Uthor")
        .env("GIT_AUTHOR_EMAIL", "author@example.com")
        .env("GIT_COMMITTER_NAME", "A. Uthor")
        .env("GIT_COMMITTER_EMAIL", "author@example.com")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .trim_end()
        .to_string()
}

/// A repository on `main` whose shared history is a bare one beside it,
/// with the policy and one feature committed, and the fakes bound to
/// publish its head.
struct Host {
    _root: TempDir,
    repo: PathBuf,
    head: Revision,
    fakes: Fakes,
}

impl Host {
    fn new() -> Host {
        let root = TempDir::new().unwrap();
        git2::Repository::init_opts(
            root.path().join("shared.git"),
            git2::RepositoryInitOptions::new()
                .bare(true)
                .initial_head("main"),
        )
        .unwrap();
        let repo = root.path().join("repo");
        // Made in the test's own process: a clone of the empty shared
        // history on `main`, following it, with the policy and one
        // feature committed by the fixed author.
        let cloned = git2::build::RepoBuilder::new()
            .clone(root.path().join("shared.git").to_str().unwrap(), &repo)
            .unwrap();
        cloned.set_head("refs/heads/main").unwrap();
        let mut config = cloned.config().unwrap();
        config.set_str("branch.main.remote", "origin").unwrap();
        config
            .set_str("branch.main.merge", "refs/heads/main")
            .unwrap();
        fs::create_dir_all(repo.join(".config")).unwrap();
        fs::write(repo.join(".config/timber.toml"), POLICY).unwrap();
        fs::create_dir_all(repo.join("site")).unwrap();
        fs::write(repo.join("site/index.html"), "<p>hello</p>").unwrap();
        let mut index = cloned.index().unwrap();
        for file in [".config/timber.toml", "site/index.html"] {
            index.add_path(Path::new(file)).unwrap();
        }
        index.write().unwrap();
        let tree = cloned.find_tree(index.write_tree().unwrap()).unwrap();
        let author = git2::Signature::now("A. Uthor", "author@example.com").unwrap();
        let made = cloned
            .commit(
                Some("HEAD"),
                &author,
                &author,
                "feat: the site\n",
                &tree,
                &[],
            )
            .unwrap();
        let head = Revision::new(made.to_string());
        let fakes = fakes_at(&head);
        Host {
            _root: root,
            repo,
            head,
            fakes,
        }
    }

    fn binding(&self) -> Binding {
        Binding {
            seams: self.fakes.seams(),
            store: store(),
            signer: Some(SignerId::new("host")),
            gate: Teller::Job(JOB.into()),
            trust: Vec::new(),
            shared: std::sync::Arc::new(timber_fakes::FakeSharedHistory),
        }
    }

    /// The scratch directory the repository is in.
    fn root(&self) -> PathBuf {
        self.repo.parent().unwrap().to_path_buf()
    }

    /// Where the built binary looks for the host's bindings file: under
    /// a configuration directory of the scratch directory's own.
    fn bindings_file(&self) -> PathBuf {
        self.root().join("config/timber/config.toml")
    }

    /// Where the built binary's given layout is: under a data directory
    /// of the scratch directory's own.
    fn given_layout(&self) -> PathBuf {
        self.root().join("data/timber/layout")
    }

    /// Where the built binary's given host key is: under the same data
    /// directory.
    fn given_key(&self) -> PathBuf {
        self.root().join("data/timber/host/signing.key")
    }

    /// `text` as the host's bindings file.
    fn bind(&self, text: &str) {
        let file = self.bindings_file();
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
    }

    /// The built binary run in the repository, its home and its
    /// configuration, data and state directories in the scratch
    /// directory, so it reads no bindings file and makes no layout or
    /// record of the host running the tests.
    fn binary(&self, args: &[&str]) -> std::process::Output {
        let home = self.root().join("home");
        fs::create_dir_all(&home).unwrap();
        Command::new(env!("CARGO_BIN_EXE_timber"))
            .args(args)
            .current_dir(&self.repo)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("HOME", &home)
            .env("XDG_CONFIG_HOME", self.root().join("config"))
            .env("XDG_DATA_HOME", self.root().join("data"))
            .env("XDG_STATE_HOME", self.root().join("state"))
            .output()
            .expect("the binary runs")
    }

    /// Every tracked file's change committed with `message` by the fixed
    /// author, in the test's own process; answers the commit.
    fn commit_all(&self, message: &str) -> Revision {
        let repo = git2::Repository::open(&self.repo).unwrap();
        let mut index = repo.index().unwrap();
        index.update_all(["*"], None).unwrap();
        index.write().unwrap();
        let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        let author = git2::Signature::now("A. Uthor", "author@example.com").unwrap();
        let message = git2::message_prettify(message, None).unwrap();
        let made = repo
            .commit(Some("HEAD"), &author, &author, &message, &tree, &[&parent])
            .unwrap();
        Revision::new(made.to_string())
    }

    /// A lightweight tag `name` at `at`.
    fn tag(&self, name: &str, at: &Revision) {
        let repo = git2::Repository::open(&self.repo).unwrap();
        let target = repo
            .find_object(git2::Oid::from_str(at.as_str()).unwrap(), None)
            .unwrap();
        repo.tag_lightweight(name, &target, false).unwrap();
    }

    /// `main` pushed to the shared history, in the test's own process.
    fn push_main(&self) {
        let main = "refs/heads/main";
        let pushed = FakeSharedHistory.push(&self.repo, "origin", main, main, "main");
        assert_eq!(pushed, Ok(PushedRef::Held));
    }

    /// Every path of the working tree git would print as changed or
    /// untracked, read in the test's own process.
    fn changed(&self) -> Vec<String> {
        let repo = git2::Repository::open(&self.repo).unwrap();
        let mut options = git2::StatusOptions::new();
        options.include_untracked(true);
        let statuses = repo.statuses(Some(&mut options)).unwrap();
        statuses
            .iter()
            .filter_map(|entry| entry.path().ok().map(str::to_string))
            .collect()
    }

    /// Every tag the repository holds, by name.
    fn tags(&self) -> Vec<String> {
        let repo = git2::Repository::open(&self.repo).unwrap();
        let names = repo.tag_names(None).unwrap();
        names
            .iter()
            .flatten()
            .flatten()
            .map(str::to_string)
            .collect()
    }
}

/// The fakes bound to publish `head`: landed on `main`, its one output
/// built and passing, the host's identity at hand, and acceptance
/// requiring nothing.
fn fakes_at(head: &Revision) -> Fakes {
    let mut fakes = Fakes::new();
    fakes.landing.land(&Branch::new("main"), head);
    fakes.signer.hold(&SignerId::new("host"));
    fakes.projection(&Ecosystem::new("eco"));
    fakes.build.made(
        head,
        &OutputName::new("site"),
        Report {
            build: timber_seams::BuildId::new("site-build"),
            outcome: timber_seams::Outcome::Pass,
            files: vec!["dist/site.tar".into()],
        },
    );
    fakes.policy.require(
        Audience::Acceptance,
        Required::Acceptance { claims: vec![] },
    );
    fakes
}

// Requirement: Each command calls the one core and renders its answer

// Requirement: A publish answers what a program reads

// Requirement: Publish fills the checkout it runs in

// Requirement: Sync fills the checkout's cache from a landed revision

/// `version` of `name` in `eco` put in the host's store with its index,
/// the name a pin resolves through moved to it, and, where `accepted`,
/// the gate's acceptance attached to the index.
fn published(host: &Host, name: &str, version: &str, accepted: bool) {
    let store = &host.fakes.store;
    let meta = ArtifactMeta::new(
        Ecosystem::new("eco"),
        EcosystemName::new(name),
        Version::new(version),
    )
    .unwrap();
    let layer = store
        .put_bytes(format!("{name} {version}").as_bytes())
        .unwrap()
        .digest;
    let artifact = store
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    let index = IndexMeta::new(
        ComponentName::new(name),
        Version::new(version),
        vec![Artifact {
            digest: artifact,
            meta,
        }],
    )
    .unwrap();
    let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
    let pointer = artifact_name(
        &Ecosystem::new("eco"),
        &EcosystemName::new(name),
        &Version::new(version),
    );
    store.name(&pointer, &index, None).unwrap();
    if accepted {
        let gate = SignerId::new("gate");
        let theirs = timber_fakes::FakeSigner::new();
        theirs.hold(&gate);
        let claim = Claim {
            subject: index.clone(),
            kind: ClaimKind::Acceptance,
            revision: Revision::new("p1b11s4"),
            build: None,
            body: Value::Null,
        };
        store
            .attach(&index, &theirs.sign(&claim, &gate).unwrap())
            .unwrap();
    }
}

/// The host's checkout pinning `pkg 1.1.0`, which the gate accepted,
/// `lib 2.0.0`, which it did not, and `left-pad 1.3.0`, which the store
/// does not name, under a policy for layouts requiring the gate's
/// acceptance.
fn pinning_one_of_each(host: &Host) {
    pinning_one_of_each_served(host);
    // The fake stands for a projection whose clients read files, so the
    // sync names no address and starts no process on the host running
    // the test.
    host.fakes.projections[&Ecosystem::new("eco")].serves_none();
}

/// What [`pinning_one_of_each`] stages, the projection serving its
/// layout.
fn pinning_one_of_each_served(host: &Host) {
    published(host, "pkg", "1.1.0", true);
    published(host, "lib", "2.0.0", false);
    let tree = History::open(&host.repo).unwrap().workdir().unwrap();
    let pin = |name: &str, version: &str| Pinned {
        name: EcosystemName::new(name),
        version: Version::new(version),
    };
    host.fakes.projections[&Ecosystem::new("eco")].lockfile(
        &tree.display().to_string(),
        &[
            pin("pkg", "1.1.0"),
            pin("lib", "2.0.0"),
            pin("left-pad", "1.3.0"),
        ],
    );
    host.fakes.policy.require(
        Audience::Layout,
        Required::Layout {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                environment: None,
                signers: vec![NamedSigner {
                    id: SignerId::new("gate"),
                    verified_by: "the gate's key".into(),
                    kinds: vec![ClaimKind::Acceptance],
                    checks: vec![],
                }],
                asked_of: AskedOf::Every,
            }],
            given: false,
            signers: vec![],
        },
    );
}

// Requirement: Push takes a landed revision

#[test]
fn the_built_binary_on_a_host_with_no_bindings_file_pushes_through_the_given_landing() {
    let host = Host::new();
    let file = host.bindings_file();
    let out = host.binary(&["push", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["answer"], "refused");
    assert_eq!(
        json["refused"]["why"]["not_landed"]["revision"],
        host.head.as_str()
    );
    assert_eq!(host.changed(), Vec::<String>::new());
    let history = History::open(&host.repo).unwrap();
    assert!(
        !Kept::at(&history).dir().exists(),
        "the push kept something"
    );
    assert!(!file.exists(), "the push wrote a bindings file");
    assert!(!host.given_key().exists(), "the push made a key");
}

// Requirement: Push takes what the checkout left with --left

// Requirement: Rebuild takes a landed revision

// Requirement: Every answer is readable by a program on request

// Requirement: Absent, refused and unreadable are three exits

// Requirement: Publish and verify take what the statement gives them

// Requirement: Status needs no seam

#[test]
fn the_built_binary_answers_status_from_the_hosts_record() {
    let host = Host::new();
    let index = Digest::of(b"the index");
    let history = History::open(&host.repo).unwrap();
    Kept::at(&history)
        .write(&Entry {
            component: ComponentName::new("repository"),
            version: Version::new("0.1.0"),
            tag: "v0.1.0".into(),
            revision: host.head.clone(),
            branch: Branch::new("main"),
            index: index.clone(),
            store: store(),
            notes: None,
        })
        .unwrap();

    assert!(!host.bindings_file().exists());
    let out = host.binary(&["status", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["answer"], "ok");
    let published = &json["value"]["published"];
    assert_eq!(published["answer"], "ok", "{json}");
    let entry = &published["value"]["components"][0]["versions"][0];
    assert_eq!(entry["component"], "repository");
    assert_eq!(entry["version"], "0.1.0");
    assert_eq!(entry["revision"], host.head.as_str());
    assert_eq!(entry["index"], index.as_str());
    assert!(
        json["value"]["channels"]["answer"].is_string(),
        "no channels' part: {json}"
    );
    // The head is not landed: the environments carry the landing's
    // refusal, as the channels do.
    assert_eq!(json["value"]["environments"]["answer"], "refused", "{json}");
    assert_eq!(
        json["value"]["environments"], json["value"]["channels"],
        "{json}"
    );
}

// Requirement: Verify shows the notes the index names

// Requirement: Verify lists reports as it lists every claim

// Requirement: The apply's host tells its own gate of a report

// Requirement: Status shows each version's notes from git

// Requirement: An unbound host answers absent
/// The digest `chained` names as the claim that satisfied the policy for
/// `version`'s promotion.
fn acceptance(version: &str) -> Digest {
    Digest::of(format!("{version}'s acceptance").as_bytes())
}

// Requirement (environment-status): The environments on the command line

// Requirement (environment-status): The copy answers where the store does
// not

// Requirement (channel-status): A channel's changelog is its chain read
// with each promotion's notes

// Requirement: Status says when the landing was fetched

#[test]
fn a_broken_bindings_file_is_the_channels_answer_and_the_record_still_answers() {
    let host = Host::new();
    let history = History::open(&host.repo).unwrap();
    Kept::at(&history)
        .write(&Entry {
            component: ComponentName::new("repository"),
            version: Version::new("0.1.0"),
            tag: "v0.1.0".into(),
            revision: host.head.clone(),
            branch: Branch::new("main"),
            index: Digest::of(b"the index"),
            store: store(),
            notes: None,
        })
        .unwrap();
    host.bind("[landing\n");
    let out = host.binary(&["status", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    let channels = &json["value"]["channels"];
    assert_eq!(channels["answer"], "unreadable", "{json}");
    let file = host.bindings_file().display().to_string();
    assert!(
        channels["unreadable"]["what"]
            .as_str()
            .is_some_and(|w| w.starts_with(&file)),
        "{json}"
    );
    assert!(
        channels["unreadable"]["said"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        "{json}"
    );
    assert_eq!(json["value"]["published"]["answer"], "ok", "{json}");
}

#[test]
fn status_in_a_directory_that_is_no_repository_exits_3_naming_it() {
    let host = Host::new();
    let elsewhere = host.root().join("elsewhere");
    fs::create_dir_all(&elsewhere).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_timber"))
        .args(["status"])
        .current_dir(&elsewhere)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CEILING_DIRECTORIES", host.root())
        .env("HOME", host.root().join("home"))
        .env("XDG_CONFIG_HOME", host.root().join("config"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let printed = String::from_utf8_lossy(&out.stdout);
    assert!(printed.contains("elsewhere"), "{printed}");
}

// Requirement: Status names the page

#[test]
fn status_page_where_the_remote_names_no_owner_and_name_is_unreadable_and_names_nothing() {
    let host = Host::new();
    let out = host.binary(&["status", "--page", "--json"]);
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["answer"], "unreadable", "{json}");
    let what = json["unreadable"]["what"].as_str().unwrap();
    assert!(
        what.contains("remote origin") && what.contains("shared.git"),
        "{what}"
    );
    assert!(!host.root().join("state").exists(), "an entry was written");
}

// Requirement: Serve is a command

// Requirement: A host with no bindings file is bound to the defaults

/// The store's unreadable answer in `out`, once it names `layout`.
fn unreadable_layout(out: &std::process::Output, layout: &Path) {
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["answer"], "unreadable", "{json}");
    assert_eq!(json["unreadable"]["source"]["seam"], "store", "{json}");
    assert!(
        json["unreadable"]["what"]
            .as_str()
            .is_some_and(|what| what.starts_with(&layout.display().to_string())),
        "{json}"
    );
}

#[test]
fn the_built_binary_on_a_host_with_no_bindings_file_is_bound_to_the_given_profiles() {
    let host = Host::new();
    let file = host.bindings_file();
    let layout = host.given_layout();
    let data = host.root().join("data");
    unreadable_layout(&host.binary(&["verify", "0.1.0", "--json"]), &layout);
    assert!(!data.exists(), "verify made {}", data.display());

    let out = host.binary(&["publish", "HEAD"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&format!("main has not landed {}", host.head)),
        "{stdout}"
    );
    assert_eq!(host.tags(), Vec::<String>::new());
    assert_eq!(host.changed(), Vec::<String>::new());
    let history = History::open(&host.repo).unwrap();
    assert!(!Kept::at(&history).dir().exists());
    assert!(
        !layout.exists(),
        "the refused publish made the given layout"
    );
    assert!(!file.exists(), "the publish wrote a bindings file");
    assert!(
        !host.given_key().exists(),
        "a publish that signed nothing made a key"
    );
    assert!(
        !data.exists(),
        "the refused publish made {}",
        data.display()
    );
}

#[test]
fn the_built_binary_in_a_checkout_with_several_remotes_and_no_origin_names_them_and_writes_nothing()
{
    let host = Host::new();
    git(&host.repo, &["remote", "rename", "origin", "github"]);
    git(
        &host.repo,
        &["remote", "add", "gitlab", "/nowhere/gitlab.git"],
    );
    let out = host.binary(&["publish", "HEAD"]);
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("github, gitlab"), "{stdout}");
    assert!(
        stdout.contains(&format!(
            "[landing] remote of {}",
            host.bindings_file().display()
        )),
        "{stdout}"
    );
    assert_eq!(host.tags(), Vec::<String>::new());
    assert_eq!(host.changed(), Vec::<String>::new());
    let history = History::open(&host.repo).unwrap();
    assert!(!Kept::at(&history).dir().exists());
    assert!(
        !host.root().join("data").exists(),
        "a layout or a key was made"
    );
    assert!(!host.bindings_file().exists());
}

#[test]
fn the_built_binary_with_no_home_and_no_configuration_directory_answers_absent_naming_both() {
    let host = Host::new();
    let data = host.root().join("data");
    let state = host.root().join("state");
    let out = Command::new(env!("CARGO_BIN_EXE_timber"))
        .args(["publish", "HEAD"])
        .current_dir(&host.repo)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env_remove("HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env("XDG_DATA_HOME", &data)
        .env("XDG_STATE_HOME", &state)
        .output()
        .expect("the binary runs");
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("neither XDG_CONFIG_HOME nor HOME is set"),
        "{stdout}"
    );
    assert_eq!(host.tags(), Vec::<String>::new());
    assert_eq!(host.changed(), Vec::<String>::new());
    let history = History::open(&host.repo).unwrap();
    assert!(!Kept::at(&history).dir().exists());
    assert!(!data.exists(), "a layout or a key was made");
    assert!(!state.exists(), "a record or a lock was made");
    assert!(!host.bindings_file().exists());
}

#[test]
fn the_built_binary_on_a_host_with_no_bindings_file_shows_status_from_the_record_and_the_ledger() {
    let host = Host::new();
    host.push_main();
    let index = Digest::of(b"the index");
    let history = History::open(&host.repo).unwrap();
    Kept::at(&history)
        .write(&Entry {
            component: ComponentName::new("repository"),
            version: Version::new("0.1.0"),
            tag: "v0.1.0".into(),
            revision: host.head.clone(),
            branch: Branch::new("main"),
            index: index.clone(),
            store: store(),
            notes: None,
        })
        .unwrap();
    // One promotion of stable on origin's ledger refs, where the given
    // ledger is kept, and none in the checkout's own.
    let promotion = Promotion {
        channel: ChannelName::new("stable"),
        component: ComponentName::new("repository"),
        to: timber_seams::Moved {
            version: Version::new("0.1.0"),
            index: index.clone(),
        },
        from: None,
        predecessor: None,
        asked_by: SignerId::new("host"),
        policy: timber_seams::Satisfied {
            revision: host.head.clone(),
            claims: vec![acceptance("0.1.0")],
        },
        at: 1_793_000_000,
    };
    let envelope = host
        .fakes
        .signer
        .sign(&promotion.claim(host.head.clone()), &SignerId::new("host"))
        .unwrap();
    let origin = timber::profiles::ledger::git_ref::GitRef::new(&host.repo, Some("origin".into()));
    timber_seams::Keeper::append(&origin, &stable(), &envelope, None).unwrap();
    git(
        &host.repo,
        &["update-ref", "-d", "refs/timber/ledger/stable"],
    );

    let out = host.binary(&["status", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    let entry = &json["value"]["published"]["value"]["components"][0]["versions"][0];
    assert_eq!(entry["version"], "0.1.0", "{json}");
    assert_eq!(entry["index"], index.as_str());
    let channels = &json["value"]["channels"];
    assert_eq!(channels["answer"], "ok", "{json}");
    let stable = channels["value"]["channels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["channel"]["name"] == "stable")
        .unwrap_or_else(|| panic!("no stable: {json}"));
    assert_eq!(
        stable["at"]["value"]["head"]["to"]["version"], "0.1.0",
        "{json}"
    );
    assert!(!host.bindings_file().exists());
    assert!(
        !host.root().join("data").exists(),
        "status made a layout or a key"
    );
}

#[test]
fn the_built_binary_on_a_host_with_no_bindings_file_syncs_from_the_given_layout_and_makes_none() {
    let host = Host::new();
    let file = host.bindings_file();
    let layout = host.given_layout();
    unreadable_layout(&host.binary(&["sync", "--json"]), &layout);
    assert_eq!(host.changed(), Vec::<String>::new());
    assert!(!host.repo.join(".moon").exists(), "the sync wrote a cache");
    assert!(!layout.exists(), "the sync made the given layout");
    assert!(!file.exists(), "the sync wrote a bindings file");
}

// Requirement: A host with no bindings file is bound to the defaults, a
// broken file

#[test]
fn a_bindings_file_that_is_not_toml_is_one_json_document_in_the_parsers_words() {
    let host = Host::new();
    let text = "[store\nlayout = \"/layout\"\n";
    host.bind(text);
    let words = toml::from_str::<toml::Table>(text).unwrap_err().to_string();
    let out = host.binary(&["verify", "0.1.0", "--json"]);
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(json["answer"], "unreadable");
    let unreadable = &json["unreadable"];
    assert_eq!(unreadable["source"]["seam"], "host");
    assert_eq!(
        unreadable["what"],
        host.bindings_file().display().to_string()
    );
    assert_eq!(unreadable["said"], words);
}

#[test]
fn a_bindings_file_naming_two_landings_is_unreadable_naming_the_table() {
    let host = Host::new();
    host.bind(&format!(
        "[store]\nlayout = \"{}\"\n\n[landing]\nsluice = {{}}\nbranch = {{}}\n\n\
         [signing]\nsigner = \"host\"\nkey = \"~/.config/timber/signing\"\n",
        host.root().join("layout").display()
    ));
    let out = host.binary(&["publish", "HEAD"]);
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&format!("{}, [landing]", host.bindings_file().display())),
        "{stdout}"
    );
    assert!(
        stdout.contains("[landing] names branch and sluice"),
        "{stdout}"
    );
    assert!(!host.root().join("layout").exists(), "the layout was made");
    assert_eq!(host.tags(), Vec::<String>::new());
}

// Requirement: The host's bindings are one file beside the landing's own

#[test]
fn with_no_configuration_directory_set_the_file_under_the_home_binds_verify() {
    let host = Host::new();
    let home = host.root().join("home");
    let layout = host.root().join("no-layout");
    let file = home.join(".config/timber/config.toml");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        &file,
        format!(
            "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n",
            layout.display()
        ),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_timber"))
        .args(["verify", "0.1.0", "--json"])
        .current_dir(&host.repo)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("HOME", &home)
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("the binary runs");
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["answer"], "unreadable", "{json}");
    assert_eq!(json["unreadable"]["source"]["seam"], "store", "{json}");
    let what = json["unreadable"]["what"].as_str().unwrap();
    assert!(what.starts_with(&layout.display().to_string()), "{json}");
    assert!(!layout.exists(), "verify made the layout");
}

// Requirement: Promote takes a version and a channel

fn stable() -> ChannelId {
    ChannelId::new(None, ChannelName::new("stable"))
}

// Requirement: A prerelease moves only the channels that take prereleases

/// The version published over the fakes at a commit with `subject`,
/// tagged after `v0.1.0`, whose policy file switches `line` on, and what
/// the gate accepting it over that file moved, each channel having moved
/// to its index, and the fakes.
fn accepted_with(line: &str, subject: &str) -> (Version, Vec<(ChannelId, Answer<Moved>)>, Fakes) {
    let host = Host::new();
    host.tag("v0.1.0", &host.head);
    fs::write(
        host.repo.join(".config/timber.toml"),
        format!(
            "{POLICY}\n[signers.host]\nssh = \"ssh-ed25519 HHHH\"\nkinds = [\"acceptance\"]\n\n\
             [acceptance]\nrequire = []\n\n{line}\n"
        ),
    )
    .unwrap();
    let head = host.commit_all(subject);
    let fakes = fakes_at(&head);
    let history = History::open(&host.repo).unwrap();
    let published = publish(
        &fakes.seams(),
        &history,
        &Kept::at(&history),
        &Ask {
            branch: Branch::new("main"),
            revision: head.clone(),
            component: None,
            signer: SignerId::new("host"),
            store: store(),
            tree: history.workdir().unwrap().display().to_string(),
        },
    )
    .unwrap();
    let Ok(timber_core::Published::Version { version, index, .. }) = &published.components[0].1
    else {
        panic!("nothing was published: {published:?}")
    };

    let mut seams = fakes.seams();
    seams.policy = Box::new(PolicyFile::new(&host.repo));
    let judged = accept(
        &seams,
        &history,
        &AcceptAsk {
            branch: Branch::new("main"),
            component: None,
            version: version.clone(),
            told: ToldBy::Publish {
                index: index.clone(),
            },
            gate: SignerId::new("host"),
            when: 1_793_000_000,
        },
    );
    let Ok(Judged::Accepted { channels, .. }) = judged else {
        panic!("{version} was not accepted: {judged:?}")
    };
    for (channel, moved) in &channels {
        assert!(
            matches!(moved, Ok(Moved::Promoted { to, .. }) if to == index),
            "{channel} did not move to {version}: {channels:?}"
        );
    }
    (version.clone(), channels, fakes)
}

/// Whether the fake keeper holds a chain of `name`.
fn holds_chain(fakes: &Fakes, name: &str) -> bool {
    !matches!(
        timber_seams::Keeper::chain(&fakes.keeper, &ChannelId::new(None, ChannelName::new(name))),
        Err(timber_seams::Not::Absent(_))
    )
}

#[test]
fn a_prerelease_accepted_with_alpha_written_moves_alpha_and_not_beta() {
    let (version, channels, fakes) = accepted_with("[channels.alpha]", "chore: alpha switched on");
    assert!(
        version.as_str().contains("-pre."),
        "{version} is no prerelease"
    );
    let alpha = ChannelId::new(None, ChannelName::new("alpha"));
    assert_eq!(
        channels.iter().map(|(c, _)| c).collect::<Vec<_>>(),
        [&alpha],
        "{channels:?}"
    );
    assert!(!holds_chain(&fakes, "beta"));
}

// Requirement: Who may ask for a channel

#[test]
fn lts_written_is_moved_by_no_publish_and_beta_is() {
    let (version, channels, fakes) = accepted_with("[channels.lts]", "feat: lts switched on");
    assert!(!version.as_str().contains('-'), "{version} is no release");
    let beta = ChannelId::new(None, ChannelName::new("beta"));
    assert_eq!(
        channels.iter().map(|(c, _)| c).collect::<Vec<_>>(),
        [&beta],
        "{channels:?}"
    );
    assert!(holds_chain(&fakes, "beta"));
    assert!(!holds_chain(&fakes, "lts"), "a publish moved lts");
}

// Requirement: Policy shows the policy in force

/// `args` asked through `run` over `binding`: the exit code and what it
/// printed.
fn run_over(host: &Host, binding: Binding, args: &[&str]) -> (i32, String) {
    let cli = Cli::try_parse_from(std::iter::once("timber").chain(args.iter().copied()))
        .expect("the arguments parse");
    let mut out = Vec::new();
    let code = run(&cli, &Ok(binding), &host.repo, &mut out);
    (code, String::from_utf8(out).unwrap())
}

/// The host's fakes with the policy file of its repository in place of
/// the fake policy.
fn over_the_file(host: &Host) -> Binding {
    let mut seams = host.fakes.seams();
    seams.policy = Box::new(PolicyFile::new(&host.repo));
    Binding {
        seams,
        store: store(),
        signer: None,
        gate: Teller::Job(JOB.into()),
        trust: Vec::new(),
        shared: std::sync::Arc::new(timber_fakes::FakeSharedHistory),
    }
}

#[test]
fn policy_through_the_command_line_prints_the_givens_beside_a_line_value_for_value() {
    let host = Host::new();
    fs::write(
        host.repo.join(".config/timber.toml"),
        format!(
            "{POLICY}\n[signers.gate]\nssh = \"ssh-ed25519 GGGG\"\nkinds = [\"acceptance\"]\n\n\
             [signers.verifier]\nssh = \"ssh-ed25519 VVVV\"\nkinds = [\"rebuild\"]\n\n\
             [acceptance]\nrequire = []\n\n\
             [channels.stable]\nrequire = [{{ kind = \"rebuild\", signers = [\"verifier\"] }}]\n"
        ),
    )
    .unwrap();
    let head = host.commit_all("chore: the policy");
    host.fakes.landing.land(&Branch::new("main"), &head);

    let (code, printed) = run_over(&host, over_the_file(&host), &["policy", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let history = History::open(&host.repo).unwrap();
    let core = in_force(
        &over_the_file(&host).seams,
        &history,
        &InForceAsk {
            branch: Branch::new("main"),
        },
    );
    assert_eq!(printed, answer_json(&core));

    let value = &printed["value"];
    assert_eq!(value["revision"], head.as_str());
    let channels = value["channels"]["value"]["in_force"]
        .as_array()
        .expect("the channels");
    let beta = &channels[0];
    assert_eq!(
        (&beta["channel"]["name"], &beta["moves"]["by"]),
        (&Value::from("beta"), &Value::from("each_publish"))
    );
    let stable = &channels[1];
    assert_eq!(stable["channel"]["name"], "stable");
    assert_eq!(
        stable["moves"],
        serde_json::json!({ "by": "bound", "to": "beta" })
    );
    let policy = &stable["policy"]["value"]["channel"];
    assert_eq!(policy["bound_to"], "beta");
    assert_eq!(policy["promoted_by"][0]["id"], "gate");
    let claims = &policy["claims"];
    let kinds: Vec<&str> = claims
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["acceptance", "rebuild"]);
    assert_eq!(claims[0]["signers"][0]["id"], "gate");
    assert!(claims[0].get("asked_of").is_none(), "{claims}");
    assert_eq!(claims[1]["asked_of"], "built", "{claims}");
    assert_eq!(channels.len(), 2, "{printed}");
}

#[test]
fn policy_shows_a_verdict_asked_of_built_artifacts_alone() {
    let host = Host::new();
    fs::write(
        host.repo.join(".config/timber.toml"),
        format!(
            "{POLICY}\n[signers.laptop]\nssh = \"ssh-ed25519 AAAA\"\nkinds = [\"provenance\"]\n\n\
             [signers.box]\nssh = \"ssh-ed25519 BBBB\"\nkinds = [\"verdict\"]\nchecks = [\"cli:test\"]\n\n\
             [acceptance]\nrequire = [\n  {{ kind = \"provenance\", signers = [\"laptop\"] }},\n  \
             {{ kind = \"verdict\", check = \"cli:test\", signers = [\"box\"] }},\n]\n"
        ),
    )
    .unwrap();
    let head = host.commit_all("chore: the policy");
    host.fakes.landing.land(&Branch::new("main"), &head);

    let (lines, json) = policy_lines(&host);
    let printed = lines.join("\n");
    let acceptance = lines
        .iter()
        .position(|l| l == "acceptance")
        .unwrap_or_else(|| panic!("no acceptance: {printed}"));
    assert_eq!(
        lines[acceptance + 1..acceptance + 3],
        [
            "  provenance by laptop",
            "  verdict of cli:test by box, of built artifacts alone",
        ],
        "{printed}"
    );
    let claims = &json["value"]["acceptance"]["value"]["acceptance"]["claims"];
    assert!(claims[0].get("asked_of").is_none(), "{claims}");
    assert_eq!(claims[1]["asked_of"], "built", "{claims}");
}

/// The host's repository with `channels` written beneath an acceptance
/// requiring nothing, committed and landed.
fn landed_with(host: &Host, channels: &str) {
    fs::write(
        host.repo.join(".config/timber.toml"),
        format!(
            "{POLICY}\n[signers.gate]\nssh = \"ssh-ed25519 GGGG\"\nkinds = [\"acceptance\"]\n\n\
             [acceptance]\nrequire = []\n\n{channels}"
        ),
    )
    .unwrap();
    let head = host.commit_all("chore: the policy");
    host.fakes.landing.land(&Branch::new("main"), &head);
}

/// The lines `timber policy` prints over the host's policy file, and
/// its JSON compared value for value with the core's answer.
fn policy_lines(host: &Host) -> (Vec<String>, Value) {
    let (code, printed) = run_over(host, over_the_file(host), &["policy"]);
    assert_eq!(code, 0, "{printed}");
    let (code, json) = run_over(host, over_the_file(host), &["policy", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let history = History::open(&host.repo).unwrap();
    let core = in_force(
        &over_the_file(host).seams,
        &history,
        &InForceAsk {
            branch: Branch::new("main"),
        },
    );
    assert_eq!(json, answer_json(&core));
    (printed.lines().map(str::to_string).collect(), json)
}

#[test]
fn policy_shows_the_givens_held_off_beneath_the_table_with_their_lines() {
    let host = Host::new();
    landed_with(&host, "");
    let (lines, json) = policy_lines(&host);
    let printed = lines.join("\n");
    let at = |line: &str| {
        lines
            .iter()
            .position(|l| l == line)
            .unwrap_or_else(|| panic!("no line {line:?}: {printed}"))
    };
    let held = at("held off");
    assert!(at("  beta, each publish") < held, "{printed}");
    assert!(at("  stable, bound to beta") < held, "{printed}");
    assert_eq!(
        lines[held + 1..],
        [
            "  alpha, each publish, prereleases too, switch on with [channels.alpha]",
            "  lts, bound to stable, switch on with [channels.lts]",
        ],
        "{printed}"
    );

    let channels = &json["value"]["channels"]["value"];
    assert_eq!(
        channels["held"],
        serde_json::json!([
            {
                "channel": { "component": null, "name": "alpha" },
                "moves": { "by": "each_publish", "prereleases": true },
                "switch": "[channels.alpha]",
            },
            {
                "channel": { "component": null, "name": "lts" },
                "moves": { "by": "bound", "to": "stable" },
                "switch": "[channels.lts]",
            },
        ])
    );
}

#[test]
fn a_given_switched_on_leaves_the_held_part() {
    let host = Host::new();
    landed_with(&host, "[channels.alpha]\n");
    let (lines, json) = policy_lines(&host);
    let printed = lines.join("\n");
    let held = lines
        .iter()
        .position(|l| l == "held off")
        .unwrap_or_else(|| panic!("no held part: {printed}"));
    assert!(
        lines[..held]
            .iter()
            .any(|l| l == "  alpha, each publish, prereleases too"),
        "{printed}"
    );
    assert_eq!(
        lines[held + 1..],
        ["  lts, bound to stable, switch on with [channels.lts]"],
        "{printed}"
    );
    let held: Vec<&Value> = json["value"]["channels"]["value"]["held"]
        .as_array()
        .expect("the held givens")
        .iter()
        .map(|h| &h["channel"]["name"])
        .collect();
    assert_eq!(held, [&Value::from("lts")]);
}

#[test]
fn policy_shows_alpha_taking_prereleases_and_stables_approvers_value_for_value() {
    let host = Host::new();
    fs::write(
        host.repo.join(".config/timber.toml"),
        format!(
            "{POLICY}\n[signers.gate]\nssh = \"ssh-ed25519 GGGG\"\nkinds = [\"acceptance\"]\n\n\
             [signers.ci]\nssh = \"ssh-ed25519 CCCC\"\nkinds = [\"acceptance\"]\n\n\
             [channels.alpha]\n\n\
             [channels.stable]\napprovers = [\"ci\"]\n"
        ),
    )
    .unwrap();
    let head = host.commit_all("chore: the policy");
    host.fakes.landing.land(&Branch::new("main"), &head);

    let (code, printed) = run_over(&host, over_the_file(&host), &["policy"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    let at = |line: &str| {
        lines
            .iter()
            .position(|l| *l == line)
            .unwrap_or_else(|| panic!("no line {line:?}: {printed}"))
    };
    let alpha = at("  alpha, each publish, prereleases too");
    assert_eq!(
        lines[alpha + 3],
        "    may be asked for by any asker",
        "{printed}"
    );
    let stable = at("  stable, bound to beta");
    assert_eq!(lines[stable + 2], "    promoted by ci, gate", "{printed}");
    assert_eq!(lines[stable + 3], "    may be asked for by ci", "{printed}");

    let (code, printed) = run_over(&host, over_the_file(&host), &["policy", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let history = History::open(&host.repo).unwrap();
    let core = in_force(
        &over_the_file(&host).seams,
        &history,
        &InForceAsk {
            branch: Branch::new("main"),
        },
    );
    assert_eq!(printed, answer_json(&core));
    let channels = printed["value"]["channels"]["value"]["in_force"]
        .as_array()
        .expect("the channels");
    assert_eq!(
        channels[0]["moves"],
        serde_json::json!({ "by": "each_publish", "prereleases": true })
    );
    let stable = channels
        .iter()
        .find(|c| c["channel"]["name"] == "stable")
        .expect("stable");
    assert_eq!(
        stable["policy"]["value"]["channel"]["approvers"][0]["id"],
        "ci"
    );
}

/// A host of the test's own for the serving process: its state, its
/// bindings file and its home under the scratch directory, so a sync
/// reads and writes no file of the host running the test.
fn serving_host(host: &Host) -> ServingHost {
    ServingHost {
        state: host.root().join("state/timber"),
        config: host.bindings_file(),
        home: Some(host.root().join("home")),
        data: None,
    }
}

/// An address of the test's own that nothing listens on: below the
/// ranges a system hands to connections, apart for each test by `k`.
fn unanswered(k: u16) -> std::net::SocketAddr {
    let port = 28_000 + (std::process::id() % 500) as u16 * 4 + k;
    std::net::SocketAddr::from(([127, 0, 0, 1], port))
}

/// A start that, once run, answers at `address` as the host's serving
/// process does, counting each time it was run in `started`.
fn starting(
    address: std::net::SocketAddr,
    started: &std::cell::Cell<u32>,
) -> impl FnOnce() -> Answer<()> + '_ {
    move || {
        started.set(started.get() + 1);
        let listener = std::net::TcpListener::bind(address).unwrap();
        std::thread::spawn(move || {
            let body = r#"{"serving":"timber","revision":"r","pid":7}"#;
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut asked = [0u8; 1024];
                let _ = std::io::Read::read(&mut stream, &mut asked);
                let _ = std::io::Write::write_all(
                    &mut stream,
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                );
            }
        });
        Ok(())
    }
}

/// `timber sync --json` over the fakes on `serving`, its process found
/// at `address` or started by `start`: the exit code and what it printed.
fn synced_on(
    host: &Host,
    serving: &ServingHost,
    address: std::net::SocketAddr,
    start: impl FnOnce() -> Answer<()>,
) -> (i32, Value) {
    let args = SyncArgs {
        revision: None,
        branch: "main".into(),
        json: true,
    };
    let mut out = Vec::new();
    let code = sync_on(
        &args,
        &Ok(host.binding()),
        &host.repo,
        &mut out,
        &Ok(serving.clone()),
        address,
        start,
    );
    (code, serde_json::from_slice(&out).unwrap())
}

#[test]
fn a_sync_of_a_served_layout_names_its_address_and_finds_or_starts_the_hosts_process() {
    let host = Host::new();
    git(
        &host.repo,
        &[
            "remote",
            "set-url",
            "origin",
            "https://example.com/agentplot/consumer.git",
        ],
    );
    pinning_one_of_each_served(&host);
    let serving = serving_host(&host);
    let address = unanswered(0);
    let started = std::cell::Cell::new(0);

    let (code, printed) = synced_on(&host, &serving, address, starting(address, &started));
    assert_eq!(code, 1, "the refused pin is not the exit: {printed}");
    assert_eq!(
        printed["value"]["layouts"][0]["served"],
        serde_json::json!({
            "answer": "ok",
            "value": "http://127.0.0.1:7420/agentplot/consumer/eco/"
        })
    );
    assert_eq!(printed["serving"]["answer"], "ok", "{printed}");
    assert_eq!(printed["serving"]["value"]["pid"], 7);
    assert_eq!(started.get(), 1);
    let consumer = RepositoryName {
        owner: "agentplot".into(),
        name: "consumer".into(),
    };
    let tree = History::open(&host.repo).unwrap().workdir().unwrap();
    assert_eq!(serving::checkout_of(&serving, &consumer), Ok(tree));

    let (_, printed) = synced_on(&host, &serving, address, || {
        panic!("started a second process")
    });
    assert_eq!(printed["serving"]["answer"], "ok", "{printed}");
    assert_eq!(started.get(), 1);
}

#[test]
fn a_sync_with_no_served_layout_names_nothing_and_starts_nothing() {
    let host = Host::new();
    git(
        &host.repo,
        &[
            "remote",
            "set-url",
            "origin",
            "https://example.com/agentplot/consumer.git",
        ],
    );
    pinning_one_of_each(&host);
    let serving = serving_host(&host);
    let (code, printed) = synced_on(&host, &serving, unanswered(1), || {
        panic!("a process was started")
    });
    assert_eq!(code, 1, "{printed}");
    assert_eq!(printed["value"]["layouts"][0]["served"], Value::Null);
    assert!(printed.get("serving").is_none(), "{printed}");
    assert!(
        !serving.state.join(serving::REPOSITORIES).exists(),
        "the host's file was written"
    );
}

#[test]
fn a_sync_in_a_checkout_whose_remote_names_no_owner_is_unreadable_naming_it() {
    let host = Host::new();
    git(
        &host.repo,
        &["remote", "set-url", "origin", "/srv/consumer.git"],
    );
    pinning_one_of_each_served(&host);
    let serving = serving_host(&host);
    let (code, printed) = synced_on(&host, &serving, unanswered(2), || {
        panic!("a process was started")
    });
    assert_eq!(code, 3, "{printed}");
    let layout = &printed["value"]["layouts"][0];
    assert_eq!(
        layout["answer"], "ok",
        "the layout was not rendered: {printed}"
    );
    let served = &layout["served"];
    assert_eq!(served["answer"], "unreadable", "{printed}");
    let what = served["unreadable"]["what"].as_str().unwrap();
    assert!(
        what.contains("origin") && what.contains("/srv/consumer.git"),
        "{what}"
    );
    assert!(printed.get("serving").is_none(), "{printed}");
    assert!(
        !serving.state.join(serving::REPOSITORIES).exists(),
        "the host's file was written"
    );
}

// Requirement: The binary says its version, its revision and its build

#[test]
fn the_binary_says_its_version_its_revision_and_its_build() {
    let out = Command::new(env!("CARGO_BIN_EXE_timber"))
        .arg("--version")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!(
            "timber {}, revision {}, built by {}\n",
            env!("CARGO_PKG_VERSION"),
            timber::REVISION,
            timber::BUILD
        )
    );
}
