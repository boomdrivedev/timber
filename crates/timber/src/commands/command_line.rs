//! Spec command-line: each command calls the one core and renders its
//! answer, asked through `run` in the test's own process over the fakes
//! (design D11, D12); the built binary's main paths are the end-to-end
//! set's, `tests/end_to_end/cli.rs`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::bindings::Teller;
use crate::{run, Binding, Cli};
use clap::Parser;
use serde_json::Value;
use tempfile::TempDir;
use timber_core::{
    answer_json, artifact_name, in_force, promote, publish, push, push_left, sync_checkout, write,
    Ask, CheckoutAsk, History, InForceAsk, Kept, Layouts, LeftAsk, PromoteAsk, PushAsk, Reported,
    RepositoryName, To,
};
use timber_fakes::{FakeSharedHistory, FakeTarget, Fakes, Fault, PolicyOp, StoreOp, Times};
use timber_seams::{
    Artifact, ArtifactMeta, AskedOf, Audience, Branch, BuildId, ChannelId, ChannelName, Check,
    Claim, ClaimKind, ComponentName, DeclaredEnvironment, Digest, Ecosystem, EcosystemName,
    EnvironmentName, Held, IndexMeta, Keeper, Left, Manifest, Moves, NamedSigner, Outcome, Output,
    OutputName, Pinned, Promotion, PushedRef, Report, Required, RequiredClaim, Revision, Seam,
    SharedHistory, Signer, SignerId, Source, Store, TargetProfile, TargetTable, Verdict, Version,
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

    /// `args` asked through `run` over the fakes: the exit code and what
    /// it printed.
    fn run(&self, args: &[&str]) -> (i32, String) {
        let cli = Cli::try_parse_from(std::iter::once("timber").chain(args.iter().copied()))
            .expect("the arguments parse");
        let mut out = Vec::new();
        let code = run(&cli, &Ok(self.binding()), &self.repo, &mut out);
        (code, String::from_utf8(out).unwrap())
    }

    /// The scratch directory the repository is in.
    fn root(&self) -> PathBuf {
        self.repo.parent().unwrap().to_path_buf()
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

    /// The day `at` was committed, in UTC, as a template's date filter
    /// prints its time.
    fn day(&self, at: &Revision) -> String {
        let repo = git2::Repository::open(&self.repo).unwrap();
        let commit = repo
            .find_commit(git2::Oid::from_str(at.as_str()).unwrap())
            .unwrap();
        let seconds = u64::try_from(commit.committer().when().seconds()).unwrap();
        timber_seams::utc(seconds)[..10].to_string()
    }

    fn calls(&self) -> usize {
        self.fakes.store.calls().len()
            + self.fakes.build.calls().len()
            + self.fakes.landing.calls().len()
            + self.fakes.signer.calls().len()
            + self.fakes.policy.calls().len()
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

#[test]
fn publish_through_the_command_line_prints_the_cores_answer_value_for_value() {
    let host = Host::new();
    let (code, printed) = host.run(&["publish", "HEAD", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let mut printed: Value = serde_json::from_str(&printed).unwrap();
    // The one thing beside the core's answer: the job that tells the
    // gate.
    let gate = printed["value"]["components"][0]
        .as_object_mut()
        .unwrap()
        .remove("gate")
        .expect("the offered version names what tells the gate");
    assert_eq!(gate, serde_json::json!({ "told_by": "job", "job": JOB }));

    let history = History::open(&host.repo).unwrap();
    let core = publish(
        &host.fakes.seams(),
        &history,
        &Kept::at(&history),
        &Ask {
            branch: Branch::new("main"),
            revision: host.head.clone(),
            component: None,
            signer: SignerId::new("host"),
            store: store(),
            tree: history.workdir().unwrap().display().to_string(),
        },
    );
    assert_eq!(printed, answer_json(&core));
    assert_eq!(
        printed["value"]["components"][0]["value"]["version"],
        "0.1.0"
    );
}

// Requirement: A publish answers what a program reads

#[test]
fn publish_prints_the_notes_beneath_the_versions_line_and_names_them_in_its_json() {
    let host = Host::new();
    let (code, printed) = host.run(&["publish", "HEAD"]);
    assert_eq!(code, 0, "{printed}");
    let (code, json) = host.run(&["publish", "HEAD", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let notes = &json["value"]["components"][0]["value"]["notes"];
    let text = notes["text"].as_str().expect("the notes' text");
    // The repository keeps no template, so git-cliff's own default
    // renders them, dated the day of the tagged revision.
    let day = host.day(&host.head);
    assert_eq!(
        text,
        format!("## [0.1.0] - {day}\n\n### 🚀 Features\n\n- The site\n")
    );

    let Held::Bytes(bytes) = host
        .fakes
        .store
        .read(&Digest::parse(notes["bytes"].as_str().unwrap()).unwrap())
        .unwrap()
    else {
        panic!("no bytes under {}", notes["bytes"])
    };
    assert_eq!(bytes, text.as_bytes());
    let envelope = notes["claims"][0]["envelope"].as_str().unwrap();
    let digest = Digest::parse(notes["digest"].as_str().unwrap()).unwrap();
    let attached = host.fakes.store.claims_of(&digest).unwrap();
    assert_eq!(
        attached
            .iter()
            .map(|e| e.digest().as_str().to_string())
            .collect::<Vec<_>>(),
        [envelope]
    );

    let lines: Vec<&str> = printed.lines().collect();
    let version = lines
        .iter()
        .position(|l| l.starts_with("repository 0.1.0 (v0.1.0)"))
        .expect(&printed);
    let at = lines
        .iter()
        .position(|l| *l == format!("  notes {digest}"))
        .expect(&printed);
    assert!(version < at, "{printed}");
    assert_eq!(
        lines[at + 1],
        format!("    provenance by host, envelope {envelope}")
    );
    let shown: Vec<String> = text
        .lines()
        .map(|l| format!("    {l}").trim_end().to_string())
        .collect();
    let end = at + 2 + shown.len();
    assert_eq!(lines[at + 2..end], shown, "{printed}");
    assert!(lines[end].starts_with("  gate: "), "{printed}");

    let artifact = &json["value"]["components"][0]["value"]["artifacts"][0];
    let store_name = artifact["store_name"].as_str().expect("the store name");
    assert_eq!(store_name, "artifact/eco/site@0.1.0");
    let line = format!(
        "  eco site {}, named {store_name}, from site by build site-build",
        artifact["digest"].as_str().unwrap()
    );
    assert!(lines.contains(&line.as_str()), "{printed}");
}

// Requirement: Publish fills the checkout it runs in

#[test]
fn publish_in_a_repository_with_no_working_tree_is_absent_naming_it_and_asks_the_store_nothing() {
    let host = Host::new();
    passed(&host, "site-build");
    host.push_main();
    let bare = host.root().join("shared.git");
    let cli = Cli::try_parse_from(["timber", "publish", "HEAD", "--json"]).unwrap();
    let mut out = Vec::new();
    let code = run(&cli, &Ok(host.binding()), &bare, &mut out);
    let printed: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(printed["answer"], "absent", "{printed}");
    let what = printed["absent"]["what"].as_str().unwrap();
    assert!(what.starts_with("working tree at "), "{what}");
    assert!(what.contains("shared.git"), "{what}");
    assert_eq!(code, crate::render::ABSENT);
    assert_eq!(host.fakes.store.calls(), [], "the store was asked");
    assert_eq!(host.fakes.landing.calls(), [], "the core was asked");
}

// Requirement: Sync fills the checkout's cache from a landed revision

/// A passing verdict of `test` on `build` at the host's head, signed as
/// the landing signs, recorded.
fn passed(host: &Host, build: &str) {
    host.fakes
        .landing
        .verdict(&host.head, verdict(&host.head, build));
}

/// A passing verdict of `test` on `build` at `head`, signed as the
/// landing signs.
fn verdict(head: &Revision, build: &str) -> Verdict {
    let id = SignerId::new("landing");
    let theirs = timber_fakes::FakeSigner::new();
    theirs.hold(&id);
    let claim = Claim {
        subject: Digest::of(build.as_bytes()),
        kind: ClaimKind::Verdict,
        revision: head.clone(),
        build: Some(BuildId::new(build)),
        body: serde_json::json!({ "check": "test", "outcome": Outcome::Pass }),
    };
    Verdict {
        check: Check::new("test"),
        build: BuildId::new(build),
        outcome: Outcome::Pass,
        signer: id.clone(),
        envelope: theirs.sign(&claim, &id).unwrap(),
        found: format!("the fake landing's record of test on {build}"),
    }
}

#[test]
fn sync_with_one_entry_gone_prints_one_rendered_and_one_absent_and_exits_0() {
    let host = Host::new();
    passed(&host, "b7e1");
    passed(&host, "c3d4");
    let site = Output {
        name: OutputName::new("site"),
        files: vec![],
    };
    write(&host.fakes.seams(), &site, &BuildId::new("c3d4")).unwrap();

    let (code, printed) = host.run(&["sync", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let entries = &printed["value"]["entries"];
    assert_eq!(entries[0]["build"], "b7e1");
    assert_eq!(entries[0]["answer"], "absent");
    assert_eq!(entries[1]["build"], "c3d4");
    assert_eq!(entries[1]["answer"], "ok");
    assert_eq!(printed["value"]["revision"], host.head.as_str());

    let history = History::open(&host.repo).unwrap();
    let core = sync_checkout(
        &host.fakes.seams(),
        &Layouts::at(&history),
        &CheckoutAsk {
            branch: Branch::new("main"),
            revision: host.head.clone(),
            tree: history.workdir().unwrap(),
            when: 0,
            trusted: vec![],
            repository: Ok(RepositoryName {
                owner: "agentplot".into(),
                name: "timber".into(),
            }),
        },
    );
    assert_eq!(printed, answer_json(&core));
    assert_eq!(printed["value"]["layouts"], serde_json::json!([]));

    let (code, printed) = host.run(&["sync"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert!(lines[0].starts_with("b7e1 absent: "), "{printed}");
    assert!(
        lines[1].starts_with("c3d4 rendered by eco, sha256:"),
        "{printed}"
    );
}

#[test]
fn sync_on_a_branch_the_repository_lacks_is_absent_and_one_that_did_not_land_it_refuses() {
    let host = Host::new();
    let (code, printed) = host.run(&["sync", "--branch", "release", "--json"]);
    assert_eq!(code, 4, "the repository holds no branch release: {printed}");
    let (code, printed) = host.run(&["sync", "HEAD", "--branch", "release", "--json"]);
    assert_eq!(code, 1, "{printed}");
    let json: Value = serde_json::from_str(&printed).unwrap();
    assert_eq!(json["refused"]["why"]["not_landed"]["branch"], "release");
}

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

#[test]
fn sync_with_a_version_refused_prints_the_cores_answer_and_exits_1() {
    let host = Host::new();
    pinning_one_of_each(&host);
    let (code, printed) = host.run(&["sync", "--json"]);
    assert_eq!(code, 1, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let layout = &printed["value"]["layouts"][0];
    assert_eq!(layout["ecosystem"], "eco");
    let pins = &layout["value"]["refreshed"]["pins"];
    let answers: Vec<(&str, &str)> = pins
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["name"].as_str().unwrap(), p["answer"].as_str().unwrap()))
        .collect();
    assert_eq!(
        answers,
        [("pkg", "ok"), ("lib", "refused"), ("left-pad", "absent")]
    );
    let wanting = &pins[1]["refused"]["why"]["version_wanting"];
    assert_eq!(wanting["version"]["version"], "2.0.0");
    assert_eq!(wanting["wanting"]["kind"], "acceptance");
    assert_eq!(wanting["checked"], serde_json::json!(["gate"]));

    let history = History::open(&host.repo).unwrap();
    let when = layout["value"]["refreshed"]["rendered_at"]
        .as_u64()
        .unwrap();
    let core = sync_checkout(
        &host.fakes.seams(),
        &Layouts::at(&history),
        &CheckoutAsk {
            branch: Branch::new("main"),
            revision: host.head.clone(),
            tree: history.workdir().unwrap(),
            when,
            trusted: vec![],
            repository: Ok(RepositoryName {
                owner: "agentplot".into(),
                name: "timber".into(),
            }),
        },
    );
    assert_eq!(printed, answer_json(&core));

    let (code, printed) = host.run(&["sync"]);
    assert_eq!(code, 1, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert!(
        lines[1].starts_with("layout eco at ")
            && lines[1].ends_with(&format!(", rendered under its own policy at {}", host.head)),
        "{printed}"
    );
    assert!(
        lines[2].starts_with("  pkg 1.1.0 held, sha256:"),
        "{printed}"
    );
    assert!(lines[3].starts_with("  lib 2.0.0: "), "{printed}");
    assert!(lines[3].contains("acceptance"), "{printed}");
    assert_eq!(lines[4], "  1 pin the store does not name", "{printed}");
}

#[test]
fn sync_whose_layout_cannot_be_refreshed_prints_what_it_holds_and_the_stores_words_and_exits_3() {
    let host = Host::new();
    pinning_one_of_each(&host);
    let (code, printed) = host.run(&["sync"]);
    assert_eq!(code, 1, "{printed}");
    host.fakes.store.fault(
        StoreOp::Resolve,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );
    let (code, printed) = host.run(&["sync"]);
    assert_eq!(code, 3, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert!(
        lines[1].contains(" left as it was, rendered 20")
            && lines[1].ends_with(&format!("Z under its own policy at {}", host.head)),
        "{printed}"
    );
    assert!(
        lines[2].starts_with("  pkg 1.1.0 held, sha256:"),
        "{printed}"
    );
    assert!(lines.contains(&"connection refused"), "{printed}");
    assert!(
        !printed.contains("lib 2.0.0"),
        "a refusal was printed: {printed}"
    );
}

// Requirement: Push takes a landed revision

/// `fakes` with the host's head landed, passing verdicts on `b7e1` and
/// `c3d4` there, and no archive of `b7e1` on the host.
fn landed_with_one_archive_missing(fakes: &mut Fakes, head: &Revision) {
    fakes.landing.land(&Branch::new("main"), head);
    for build in ["b7e1", "c3d4"] {
        fakes.landing.verdict(head, verdict(head, build));
    }
    fakes
        .projection(&Ecosystem::new("eco"))
        .no_archive(&BuildId::new("b7e1"));
}

#[test]
fn push_with_one_archive_missing_prints_one_written_and_one_absent_and_exits_0() {
    let mut host = Host::new();
    let head = host.head.clone();
    landed_with_one_archive_missing(&mut host.fakes, &head);

    let (code, printed) = host.run(&["push", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let entries = &printed["value"]["entries"];
    assert_eq!(entries[0]["build"], "b7e1");
    assert_eq!(entries[0]["answer"], "absent");
    assert_eq!(entries[1]["build"], "c3d4");
    assert_eq!(entries[1]["answer"], "ok");
    assert_eq!(entries[1]["value"]["held_at"], Value::Null);
    let digest = entries[1]["value"]["entry"]["digest"].as_str().unwrap();
    assert!(digest.starts_with("sha256:"), "{printed}");
    assert_eq!(printed["value"]["revision"], host.head.as_str());

    let mut twin = Fakes::new();
    landed_with_one_archive_missing(&mut twin, &head);
    let core = push(
        &twin.seams(),
        &PushAsk {
            branch: Branch::new("main"),
            revision: head,
        },
    );
    assert_eq!(printed, answer_json(&core));

    let (code, printed) = host.run(&["push"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(lines[0], "b7e1 absent: the archive of b7e1", "{printed}");
    assert_eq!(
        lines[1],
        format!("c3d4 held already at {digest}"),
        "{printed}"
    );
}

// Requirement: Push takes what the checkout left with --left

/// The two passing build identities a check job's run in `tree` left.
fn left_by_a_job(fakes: &Fakes, tree: &Path) {
    let left = |build: &str, task: &str| Left {
        build: BuildId::new(build),
        tasks: vec![task.into()],
        outcome: Outcome::Pass,
        files: vec![tree.join(format!("{task}.out")).display().to_string()],
    };
    fakes.build.left_in(
        &tree.display().to_string(),
        vec![left("b7e1", "web:build"), left("c3d4", "app:test")],
    );
}

#[test]
fn a_check_jobs_push_prints_the_cores_answer_and_asks_no_landing() {
    let host = Host::new();
    let tree = host.repo.canonicalize().unwrap();
    left_by_a_job(&host.fakes, &tree);

    let (code, printed) = host.run(&["push", "--left", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    assert_eq!(printed["value"]["at"], tree.display().to_string());
    let entries = &printed["value"]["entries"];
    for (i, build) in ["b7e1", "c3d4"].iter().enumerate() {
        assert_eq!(entries[i]["build"], *build, "{printed}");
        assert_eq!(entries[i]["answer"], "ok", "{printed}");
        assert_eq!(entries[i]["value"]["held_at"], Value::Null);
        let digest = entries[i]["value"]["entry"]["digest"].as_str().unwrap();
        assert!(digest.starts_with("sha256:"), "{printed}");
    }
    assert_eq!(host.fakes.landing.calls(), [], "the landing was asked");

    let twin = Host::new();
    left_by_a_job(&twin.fakes, &tree);
    let core = push_left(
        &twin.fakes.seams(),
        &LeftAsk {
            at: tree.display().to_string(),
        },
    );
    assert_eq!(printed, answer_json(&core));

    let (code, printed) = host.run(&["push", "--left"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert!(
        lines[0].starts_with("b7e1 held already at sha256:"),
        "{printed}"
    );
    assert!(
        lines[1].starts_with("c3d4 held already at sha256:"),
        "{printed}"
    );
}

#[test]
fn a_check_job_whose_run_left_nothing_pushes_nothing_and_exits_0() {
    let host = Host::new();
    let (code, printed) = host.run(&["push", "--left"]);
    assert_eq!(code, 0, "{printed}");
    assert!(printed.contains("left no build that passed"), "{printed}");
    assert_eq!(host.fakes.store.calls(), [], "the store was asked");
}

#[test]
fn a_revision_or_a_branch_beside_left_is_an_argument_it_cannot_read() {
    let host = Host::new();
    for (args, named) in [
        (&["timber", "push", "--left", "HEAD"][..], "REVISION"),
        (
            &["timber", "push", "--left", "--branch", "main"][..],
            "--branch",
        ),
    ] {
        let err = Cli::try_parse_from(args).expect_err("push took both");
        assert_eq!(err.exit_code(), 2);
        assert!(err.to_string().contains(named), "{err}");
    }
    assert_eq!(host.calls(), 0, "the core was asked");
}

// Requirement: Rebuild takes a landed revision

#[test]
fn rebuild_after_the_record_is_deleted_writes_its_entry_and_status_answers_as_before() {
    let host = Host::new();
    assert_eq!(host.run(&["publish", "HEAD"]).0, 0);
    let published = || {
        let (code, printed) = host.run(&["status", "--json"]);
        assert_eq!(code, 0, "{printed}");
        let json: Value = serde_json::from_str(&printed).unwrap();
        json["value"]["published"].clone()
    };
    let before = published();
    assert_eq!(before["answer"], "ok", "{before}");
    let history = History::open(&host.repo).unwrap();
    let record = Kept::at(&history).dir().to_path_buf();
    fs::remove_dir_all(&record).unwrap();
    assert_eq!(
        published()["answer"],
        "absent",
        "the record was not deleted"
    );

    let (code, printed) = host.run(&["rebuild", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let json: Value = serde_json::from_str(&printed).unwrap();
    assert_eq!(json["answer"], "ok");
    assert_eq!(json["value"]["record"], record.display().to_string());
    let entries = &json["value"]["entries"];
    assert_eq!(entries.as_array().map(Vec::len), Some(1), "{printed}");
    assert_eq!(entries[0]["answer"], "ok");
    assert_eq!(entries[0]["value"], "wrote");
    assert_eq!(entries[0]["entry"]["tag"], "v0.1.0");
    assert_eq!(entries[0]["entry"]["revision"], host.head.as_str());

    assert_eq!(
        published(),
        before,
        "status answers otherwise after the rebuild"
    );

    let (code, printed) = host.run(&["rebuild"]);
    assert_eq!(code, 0, "{printed}");
    assert_eq!(
        printed.trim_end(),
        format!("repository 0.1.0 (v0.1.0) from {}: held", host.head)
    );
}

// Requirement: Every answer is readable by a program on request

#[test]
fn a_refusal_is_one_json_document_naming_what_its_clause_says() {
    let host = Host::new();
    let (code, printed) = host.run(&["publish", "HEAD", "--branch", "release", "--json"]);
    assert_eq!(code, 1);
    let json: Value = serde_json::from_str(&printed).expect("one JSON document");
    assert_eq!(json["answer"], "refused");
    let refused = &json["refused"];
    assert_eq!(refused["source"]["seam"], "landing");
    assert_eq!(refused["why"]["not_landed"]["branch"], "release");
    assert_eq!(refused["why"]["not_landed"]["revision"], host.head.as_str());
}

// Requirement: Absent, refused and unreadable are three exits

#[test]
fn three_answers_exit_three_ways_and_the_sources_words_are_printed() {
    let host = Host::new();
    let (refused, _) = host.run(&["publish", "HEAD", "--branch", "release"]);
    let (absent, _) = host.run(&["verify", "9.9.9"]);
    assert_eq!(host.run(&["publish", "HEAD"]).0, 0);
    host.fakes.store.fault(
        StoreOp::ClaimsOf,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );
    let (unreadable, printed) = host.run(&["verify", "0.1.0"]);
    assert_eq!((refused, absent, unreadable), (1, 4, 3));
    assert!(
        printed.lines().any(|l| l == "connection refused"),
        "{printed}"
    );
}

// Requirement: Publish and verify take what the statement gives them

#[test]
fn a_version_given_to_publish_is_an_argument_it_cannot_read() {
    let host = Host::new();
    let err = Cli::try_parse_from(["timber", "publish", "HEAD", "--version", "1.0.0"])
        .expect_err("publish took a version");
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("--version"), "{err}");
    assert_eq!(host.calls(), 0, "the core was asked");
}

// Requirement: Status needs no seam

// Requirement: Verify shows the notes the index names

#[test]
fn verify_prints_the_notes_after_the_claims() {
    let host = Host::new();
    let (code, published) = host.run(&["publish", "HEAD", "--json"]);
    assert_eq!(code, 0, "{published}");
    let published: Value = serde_json::from_str(&published).unwrap();
    let notes = &published["value"]["components"][0]["value"]["notes"];
    let text = notes["text"].as_str().expect("the notes' text");

    let (code, printed) = host.run(&["verify", "0.1.0"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    let at = lines
        .iter()
        .position(|l| *l == format!("notes {}", notes["digest"].as_str().unwrap()))
        .expect(&printed);
    assert!(
        lines[1..at].iter().all(|l| l.starts_with("  ")) && at > 1,
        "the notes are not after the claims: {printed}"
    );
    let shown: Vec<String> = text
        .lines()
        .map(|l| format!("  {l}").trim_end().to_string())
        .collect();
    assert_eq!(lines[at + 1..], shown, "{printed}");

    let (code, json) = host.run(&["verify", "0.1.0", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(json["value"]["notes"]["value"]["text"], text);
}

// Requirement: Verify lists reports as it lists every claim

#[test]
fn verify_lists_a_report_with_its_signer_and_the_environment_it_reports() {
    let host = Host::new();
    let (code, published) = host.run(&["publish", "HEAD", "--json"]);
    assert_eq!(code, 0, "{published}");
    let published: Value = serde_json::from_str(&published).unwrap();
    let index = Digest::parse(
        published["value"]["components"][0]["value"]["index"]
            .as_str()
            .expect("the index"),
    )
    .unwrap();
    let srv = EnvironmentName::new("srv");
    let report = Reported {
        environment: srv.clone(),
        component: ComponentName::new("repository"),
        channel: ChannelName::new("stable"),
        version: Version::new("0.1.0"),
        index: index.clone(),
        runs: index.clone(),
        converged_at: "2026-11-05T09:00:00Z".into(),
        at: 1_793_000_000,
    };
    let envelope = host
        .fakes
        .signer
        .sign(&report.claim(host.head.clone()), &SignerId::new("host"))
        .unwrap();
    host.fakes.store.attach(&index, &envelope).unwrap();
    host.fakes.policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Report,
                check: None,
                environment: Some(srv),
                signers: vec![NamedSigner {
                    id: SignerId::new("host"),
                    verified_by: "the host's key".into(),
                    kinds: vec![ClaimKind::Report],
                    checks: vec![],
                }],
                asked_of: AskedOf::Every,
            }],
        },
    );

    let (code, printed) = host.run(&["verify", "0.1.0"]);
    assert_eq!(code, 0, "{printed}");
    let line = format!("  report of srv on {index}: made by host; counts");
    assert!(printed.lines().any(|l| l == line), "{printed}");

    let (code, json) = host.run(&["verify", "0.1.0", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let claims = json["value"]["claims"].as_array().expect("the claims");
    let told = claims
        .iter()
        .find(|c| c["kind"] == "report")
        .expect("no report listed");
    assert_eq!(told["environment"], "srv", "{told}");
    assert_eq!(told["made_by"], "host", "{told}");
    assert_eq!(told["counts"], true, "{told}");
    assert_eq!(told["envelope"], envelope.digest().as_str(), "{told}");
    assert!(
        claims
            .iter()
            .filter(|c| c["kind"] != "report")
            .all(|c| c.get("environment").is_none()),
        "a claim other than the report names an environment: {json}"
    );
}

// Requirement: The apply's host tells its own gate of a report

/// When the fake target's apply converges.
const CONVERGED: &str = "2026-11-06T09:00:00Z";

/// The host as the policy names it: it may accept, promote and report.
fn host_signer() -> NamedSigner {
    NamedSigner {
        id: SignerId::new("host"),
        verified_by: "the host's key".into(),
        kinds: vec![
            ClaimKind::Acceptance,
            ClaimKind::Promotion,
            ClaimKind::Report,
        ],
        checks: vec![],
    }
}

/// A channel's policy requiring the host's acceptance and promoted by
/// the host, bound to `bound_to` where given.
fn by_the_host(bound_to: Option<&str>) -> Required {
    Required::Channel {
        claims: vec![RequiredClaim {
            kind: ClaimKind::Acceptance,
            check: None,
            environment: None,
            signers: vec![host_signer()],
            asked_of: AskedOf::Every,
        }],
        bound_to: bound_to.map(ChannelName::new),
        promoted_by: vec![host_signer()],
        approvers: vec![],
    }
}

fn second() -> ChannelId {
    ChannelId::new(None, ChannelName::new("second"))
}

/// The host's head published as 0.1.0, its index carrying the host's
/// acceptance; `stable` promoted to it by the host; each of `declared`
/// declared on `stable`, `srv` with the fake target, converging at
/// [`CONVERGED`]; and `second`, which comes after `after`, bound to
/// `stable`. The index and the fake target.
fn applicable(host: &mut Host, declared: &[&str], after: &[&str]) -> (Digest, FakeTarget) {
    let (code, published) = host.run(&["publish", "HEAD", "--json"]);
    assert_eq!(code, 0, "{published}");
    let published: Value = serde_json::from_str(&published).unwrap();
    let index = Digest::parse(
        published["value"]["components"][0]["value"]["index"]
            .as_str()
            .expect("the index"),
    )
    .unwrap();
    let acceptance = Claim {
        subject: index.clone(),
        kind: ClaimKind::Acceptance,
        revision: host.head.clone(),
        build: None,
        body: serde_json::json!({}),
    };
    let envelope = host
        .fakes
        .signer
        .sign(&acceptance, &SignerId::new("host"))
        .unwrap();
    host.fakes.store.attach(&index, &envelope).unwrap();
    host.fakes
        .policy
        .require(Audience::Channel(stable()), by_the_host(None));
    let promoted = promote(
        &host.fakes.seams(),
        &History::open(&host.repo).unwrap(),
        &PromoteAsk {
            branch: Branch::new("main"),
            component: None,
            channel: ChannelName::new("stable"),
            to: To::Version(Version::new("0.1.0")),
            from: None,
            signer: SignerId::new("host"),
            at: 1_793_000_000,
        },
    );
    assert!(promoted.is_ok(), "{promoted:?}");
    for name in declared {
        host.fakes.policy.declares_at(
            &host.head,
            DeclaredEnvironment {
                name: EnvironmentName::new(*name),
                channel: stable(),
                waits: false,
                approvers: vec![],
                target: Some(TargetTable {
                    profile: TargetProfile::new("fake"),
                    settings: serde_json::json!({}),
                }),
            },
        );
    }
    let after: Vec<EnvironmentName> = after.iter().map(|e| EnvironmentName::new(*e)).collect();
    host.fakes.policy.after(
        &second(),
        &after,
        &[host_signer()],
        by_the_host(Some("stable")),
    );
    let target = host.fakes.target(&TargetProfile::new("fake"));
    target.at(CONVERGED);
    (index, target)
}

/// The host's binding with its own steps telling its gate (118).
fn own_gate(host: &Host) -> Binding {
    Binding {
        gate: Teller::Publish,
        ..host.binding()
    }
}

/// The promotions `channel`'s chain holds, none where it holds none.
fn promotions(fakes: &Fakes, channel: &ChannelId) -> Vec<Promotion> {
    match fakes.keeper.chain(channel) {
        Ok(chain) => chain
            .claims
            .iter()
            .map(|e| Promotion::of(&Claim::from_envelope(e).unwrap()).unwrap())
            .collect(),
        Err(_) => vec![],
    }
}

#[test]
fn srv_applied_on_a_host_that_is_its_gate_moves_the_channel_after_it_and_exits_0() {
    let mut host = Host::new();
    let (index, _target) = applicable(&mut host, &["srv"], &["srv"]);
    let (code, printed) = run_over(&host, own_gate(&host), &["apply", "srv", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let value = &printed["value"];
    assert_eq!(value["report"]["answer"], "ok", "{printed}");
    let after = &value["after"];
    assert_eq!(after["told_by"], "apply", "{printed}");
    assert_eq!(after["answer"], "ok", "{printed}");
    let channels = after["value"]["channels"].as_array().expect("the channels");
    assert_eq!(channels.len(), 1, "{printed}");
    assert_eq!(channels[0]["channel"], serde_json::json!(second()));
    assert_eq!(channels[0]["answer"], "ok", "{printed}");
    assert_eq!(channels[0]["value"]["after"], "promoted", "{printed}");
    assert_eq!(channels[0]["value"]["promoted"], "moved", "{printed}");
    assert_eq!(channels[0]["value"]["to"]["index"], index.as_str());

    let chain = promotions(&host.fakes, &second());
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].to.index, index);
    assert_eq!(chain[0].asked_by, SignerId::new("host"));
    let report = Digest::parse(value["report"]["value"].as_str().unwrap()).unwrap();
    assert!(chain[0].policy.claims.contains(&report), "{chain:?}");
}

#[test]
fn srv_applied_where_a_job_tells_the_gate_carries_the_jobs_words_and_moves_nothing() {
    let mut host = Host::new();
    applicable(&mut host, &["srv"], &["srv"]);
    let before = host.fakes.policy.calls().len();
    let (code, printed) = host.run(&["apply", "srv", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    assert_eq!(printed["value"]["report"]["answer"], "ok", "{printed}");
    assert_eq!(
        printed["value"]["after"],
        serde_json::json!({ "told_by": "job", "job": JOB })
    );
    assert!(
        promotions(&host.fakes, &second()).is_empty(),
        "second moved"
    );
    assert!(
        host.fakes.policy.calls()[before..]
            .iter()
            .all(|c| c.op != PolicyOp::Channels),
        "the channels were asked for a move"
    );

    let (code, printed) = host.run(&["apply", "srv"]);
    assert_eq!(code, 0, "{printed}");
    assert_eq!(
        printed.lines().last(),
        Some(format!("  after: the gate is told by \"{JOB}\", and no channel moved").as_str()),
        "{printed}"
    );
}

#[test]
fn a_report_the_target_answers_unread_asks_no_gate_and_exits_as_unreadable() {
    let mut host = Host::new();
    let (_, target) = applicable(&mut host, &["srv"], &["srv"]);
    target.holds_no_marker();
    let before = host.fakes.policy.calls().len();
    let (code, printed) = run_over(&host, own_gate(&host), &["apply", "srv", "--json"]);
    assert_eq!(code, 3, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    assert_eq!(printed["answer"], "ok", "{printed}");
    assert_eq!(
        printed["value"]["report"]["answer"], "unreadable",
        "{printed}"
    );
    assert!(printed["value"].get("after").is_none(), "{printed}");
    assert!(
        promotions(&host.fakes, &second()).is_empty(),
        "second moved"
    );
    assert!(
        host.fakes.policy.calls()[before..]
            .iter()
            .all(|c| c.op != PolicyOp::Channels),
        "the channels were asked for a move"
    );
}

#[test]
fn promote_of_a_channel_an_after_makes_exits_1_naming_it_and_its_environments() {
    let mut host = Host::new();
    applicable(&mut host, &["srv"], &["srv"]);
    let (code, printed) = host.run(&["promote", "0.1.0", "--to", "second"]);
    assert_eq!(code, 1, "{printed}");
    assert!(
        printed.contains(
            "second comes after srv, and the gate moves it on their reports; no asker may move it"
        ),
        "{printed}"
    );
    let (code, printed) = host.run(&["promote", "0.1.0", "--to", "second", "--json"]);
    assert_eq!(code, 1, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    assert_eq!(printed["answer"], "refused", "{printed}");
    assert_eq!(
        printed["refused"]["why"]["moves_on_reports"]["after"],
        serde_json::json!(["srv"]),
        "{printed}"
    );
    assert!(
        promotions(&host.fakes, &second()).is_empty(),
        "second moved"
    );
}

#[test]
fn a_channel_held_for_an_environment_that_has_not_reported_prints_each_and_exits_as_refused() {
    let mut host = Host::new();
    let (index, _target) = applicable(&mut host, &["srv", "eu"], &["srv", "eu"]);
    let (code, printed) = run_over(&host, own_gate(&host), &["apply", "srv"]);
    assert_eq!(code, 1, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    let at = lines
        .iter()
        .position(|l| *l == "  after:")
        .unwrap_or_else(|| panic!("no after part: {printed}"));
    assert_eq!(
        lines[at + 1],
        "    second held, its environments not at one index:"
    );
    assert_eq!(
        lines[at + 2],
        format!("      srv last reported 0.1.0 ({index})")
    );
    assert_eq!(lines[at + 3], "      eu:");
    assert!(
        lines[at + 4].contains("a counted report of environment eu"),
        "{printed}"
    );
    assert!(
        promotions(&host.fakes, &second()).is_empty(),
        "second moved"
    );
}

// Requirement: Status shows each version's notes from git

#[test]
fn status_prints_the_notes_beneath_the_versions_line_and_carries_them_in_its_json() {
    let host = Host::new();
    let (code, published) = host.run(&["publish", "HEAD", "--json"]);
    assert_eq!(code, 0, "{published}");
    let published: Value = serde_json::from_str(&published).unwrap();
    let carried = published["value"]["components"][0]["value"]["notes"]["text"]
        .as_str()
        .expect("the notes' text")
        .to_string();

    let (code, printed) = host.run(&["status"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.starts_with("  repository 0.1.0 (v0.1.0) from "))
        .expect(&printed);
    let shown: Vec<String> = carried
        .lines()
        .map(|l| format!("    {l}").trim_end().to_string())
        .collect();
    assert_eq!(lines[at + 1..], shown, "{printed}");

    let (code, json) = host.run(&["status", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let version = &json["value"]["published"]["value"]["components"][0]["versions"][0];
    assert_eq!(version["version"], "0.1.0");
    assert_eq!(version["notes"]["answer"], "ok");
    assert_eq!(version["notes"]["value"], carried.as_str());
}

// Requirement: An unbound host answers absent
/// The moment now, in seconds since the Unix epoch.
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// `channel` declared over `fakes`' policy in the one component, bound
/// to nothing.
fn declared(fakes: &Fakes, channel: &str) {
    fakes.policy.require(
        Audience::Channel(ChannelId::new(None, ChannelName::new(channel))),
        Required::Channel {
            claims: vec![],
            bound_to: None,
            promoted_by: vec![],
            approvers: vec![],
        },
    );
}

/// `channel` promoted over `fakes`' keeper to each version in turn, each
/// from the one before, signed by the host as the fake signer signs,
/// each satisfied by its version's acceptance; every claim's digest.
fn chained(host: &Host, channel: &str, versions: &[&str]) -> Vec<Digest> {
    let channel = ChannelId::new(None, ChannelName::new(channel));
    let moved = |v: &str| timber_seams::Moved {
        version: Version::new(v),
        index: Digest::of(format!("{v}'s index").as_bytes()),
    };
    let mut predecessor: Option<Digest> = None;
    let mut claims = Vec::new();
    for (i, version) in versions.iter().enumerate() {
        let promotion = Promotion {
            channel: channel.name.clone(),
            component: ComponentName::new("repository"),
            to: moved(version),
            from: i.checked_sub(1).map(|j| moved(versions[j])),
            predecessor: predecessor.clone(),
            asked_by: SignerId::new("host"),
            policy: timber_seams::Satisfied {
                revision: host.head.clone(),
                claims: vec![acceptance(version)],
            },
            at: 1_793_000_000 + i as u64,
        };
        let envelope = host
            .fakes
            .signer
            .sign(&promotion.claim(host.head.clone()), &SignerId::new("host"))
            .unwrap();
        let digest = timber_seams::Keeper::append(
            &host.fakes.keeper,
            &channel,
            &envelope,
            predecessor.as_ref(),
        )
        .unwrap();
        claims.push(digest.clone());
        predecessor = Some(digest);
    }
    claims
}

/// The digest `chained` names as the claim that satisfied the policy for
/// `version`'s promotion.
fn acceptance(version: &str) -> Digest {
    Digest::of(format!("{version}'s acceptance").as_bytes())
}

#[test]
fn status_through_the_command_line_prints_the_cores_overview_value_for_value() {
    let host = Host::new();
    assert_eq!(host.run(&["publish", "HEAD"]).0, 0);
    declared(&host.fakes, "beta");
    declared(&host.fakes, "stable");
    chained(&host, "stable", &["0.1.0", "0.2.0"]);
    let before = now();
    let (code, printed) = host.run(&["status", "--json"]);
    let after = now();
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let asked_at = printed["value"]["asked_at"].as_u64().expect("asked_at");
    assert!(
        (before..=after).contains(&asked_at),
        "asked at {asked_at}, not between {before} and {after}"
    );

    let history = History::open(&host.repo).unwrap();
    let core = timber_core::overview(
        &host.fakes.seams(),
        &history,
        &Kept::at(&history),
        &timber_core::OverviewAsk {
            branch: Branch::new("main"),
            component: None,
            revision: None,
            at: asked_at,
        },
    );
    assert_eq!(printed, answer_json(&Ok::<_, timber_seams::Not>(core)));
    let channels = &printed["value"]["channels"]["value"]["channels"];
    assert_eq!(channels[1]["at"]["value"]["head"]["to"]["version"], "0.2.0");
    assert_eq!(printed["value"]["published"]["answer"], "ok");
}

#[test]
fn status_as_text_names_each_channel_its_chain_newest_first_and_each_answer_apart() {
    let host = Host::new();
    for channel in ["edge", "preview", "stable"] {
        declared(&host.fakes, channel);
    }
    let claims = chained(&host, "stable", &["0.1.0", "0.2.0"]);
    // The first chain asked, edge's, cannot be read.
    host.fakes.keeper.fault(
        timber_fakes::KeeperOp::Chain,
        Fault::Unreadable("fatal: unable to access".into()),
        Times::Once,
    );
    let (code, printed) = host.run(&["status"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert!(
        lines[0].starts_with(&format!("main at {}, as of 20", host.head)),
        "{printed}"
    );
    let at = |starts: &str| {
        lines
            .iter()
            .position(|l| l.starts_with(starts))
            .unwrap_or_else(|| panic!("no line starts {starts:?}: {printed}"))
    };
    assert_eq!(lines[at("repository")], "repository");
    let edge = at("  edge ");
    assert!(lines[edge].contains("could not read"), "{printed}");
    assert_eq!(lines[edge + 1], "fatal: unable to access", "{printed}");
    assert!(lines[at("  preview ")].ends_with("empty"), "{printed}");
    let stable = at("  stable ");
    assert!(
        lines[stable].contains(&format!(
            "0.2.0 ({})",
            Digest::of("0.2.0's index".as_bytes())
        )),
        "{printed}"
    );
    assert_eq!(
        lines[stable + 1].trim_start(),
        format!(
            "0.2.0 from 0.1.0, asked by host, under the policy at {} satisfied by {}, \
             2026-10-26T07:33:21Z, claim {} after {}",
            host.head,
            acceptance("0.2.0"),
            claims[1],
            claims[0]
        ),
        "{printed}"
    );
    // Neither version was ever tagged here: each promotion's notes are
    // unreadable beneath it, two spaces in, naming the tag, with the
    // history's words on a line of their own.
    let indent = lines[stable + 1].len() - lines[stable + 1].trim_start().len();
    let unread = |version: &str| {
        [
            format!(
                "{}the history (git, local) could not read the notes of repository {version}",
                " ".repeat(indent + 2)
            ),
            format!(
                "the history holds no tag v{version} reached from {}",
                host.head
            ),
        ]
    };
    assert_eq!(lines[stable + 2..stable + 4], unread("0.2.0"), "{printed}");
    assert_eq!(
        lines[stable + 4].trim_start(),
        format!(
            "0.1.0, the first, asked by host, under the policy at {} satisfied by {}, \
             2026-10-26T07:33:20Z, claim {}",
            host.head,
            acceptance("0.1.0"),
            claims[0]
        ),
        "{printed}"
    );
    assert_eq!(lines[stable + 5..stable + 7], unread("0.1.0"), "{printed}");
    // The policy declares no environment: four counts of none.
    let environments = at("environments");
    assert_eq!(
        lines[environments + 1],
        "  0 running what they should, 0 behind, 0 waiting, 0 unreadable",
        "{printed}"
    );
    let published = at("published from this host");
    assert!(lines[published + 1].contains("holds no"), "{printed}");
}

// Requirement (environment-status): The environments on the command line

/// The fake store's index of `version`, which the fake chains move to.
fn index_of(version: &str) -> Digest {
    Digest::of(format!("{version}'s index").as_bytes())
}

/// The fakes' report of `srv`, which follows `stable`, running `version`,
/// converged at `converged_at`, from an apply at `at`, signed by the host
/// and attached to the version's index.
fn reported(host: &Host, version: &str, converged_at: &str, at: u64) {
    let index = index_of(version);
    let report = Reported {
        environment: EnvironmentName::new("srv"),
        component: ComponentName::new("repository"),
        channel: ChannelName::new("stable"),
        version: Version::new(version),
        index: index.clone(),
        runs: index.clone(),
        converged_at: converged_at.into(),
        at,
    };
    let envelope = host
        .fakes
        .signer
        .sign(&report.claim(host.head.clone()), &SignerId::new("host"))
        .unwrap();
    host.fakes.store.attach(&index, &envelope).unwrap();
}

/// Over the host's fakes: 0.2.0 tagged at the head, 0.3.0 and 0.4.0 each
/// at a feature of its own landed on `main` in turn; `stable` promoted
/// to each in turn, every index held in the store; `srv` declared at the
/// landed head following `stable`; and the policy letting the host
/// report, whose one report says `srv` runs 0.2.0. The landed head and
/// the revisions of 0.3.0 and 0.4.0.
fn srv_behind(host: &Host) -> (Revision, [Revision; 2]) {
    host.tag("v0.2.0", &host.head);
    let mut revisions = Vec::new();
    for version in ["0.3.0", "0.4.0"] {
        fs::write(
            host.repo.join("site/index.html"),
            format!("<p>hello, {version}</p>"),
        )
        .unwrap();
        let revision = host.commit_all(&format!("feat: the site at {version}"));
        host.tag(&format!("v{version}"), &revision);
        host.fakes.landing.land(&Branch::new("main"), &revision);
        revisions.push(revision);
    }
    let landed = revisions[1].clone();
    declared(&host.fakes, "stable");
    for version in ["0.2.0", "0.3.0", "0.4.0"] {
        let held = host
            .fakes
            .store
            .put_bytes(format!("{version}'s index").as_bytes())
            .unwrap();
        assert_eq!(held.digest, index_of(version));
    }
    chained(host, "stable", &["0.2.0", "0.3.0", "0.4.0"]);
    host.fakes.policy.declares_at(
        &landed,
        DeclaredEnvironment {
            name: EnvironmentName::new("srv"),
            channel: stable(),
            waits: false,
            approvers: vec![],
            target: None,
        },
    );
    host.fakes.policy.require(
        Audience::Layout,
        Required::Layout {
            claims: vec![],
            given: false,
            signers: vec![NamedSigner {
                id: SignerId::new("host"),
                verified_by: "the host's key".into(),
                kinds: vec![ClaimKind::Report],
                checks: vec![],
            }],
        },
    );
    reported(host, "0.2.0", "2026-11-05T09:00:00Z", 1_793_000_100);
    (landed, [revisions[0].clone(), revisions[1].clone()])
}

/// `srv`'s line among `lines`: where it is.
fn srv_line(lines: &[&str], printed: &str) -> usize {
    lines
        .iter()
        .position(|l| l.starts_with("  srv  "))
        .unwrap_or_else(|| panic!("no line for srv: {printed}"))
}

#[test]
fn an_environment_behind_prints_its_line_and_the_notes_between_and_its_json_is_the_cores() {
    let host = Host::new();
    let (_, revisions) = srv_behind(&host);

    let (code, json) = host.run(&["status", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let part = &json["value"]["environments"];
    assert_eq!(part["answer"], "ok", "{json}");
    for (count, n) in [
        ("converged", 0),
        ("behind", 1),
        ("waiting", 0),
        ("unreadable", 0),
    ] {
        assert_eq!(part["value"][count], n, "{count}: {json}");
    }
    let srv = &part["value"]["environments"][0];
    assert_eq!(srv["environment"], "srv");
    assert_eq!(srv["state"]["value"], "behind", "{json}");
    assert_eq!(srv["runs"]["value"]["version"], "0.2.0", "{json}");
    assert_eq!(srv["should"]["value"]["version"], "0.4.0", "{json}");
    let between = &srv["between"]["value"];
    assert_eq!(between["direction"], "ahead", "{json}");
    let versions = between["versions"]
        .as_array()
        .expect("the versions between");
    assert_eq!(versions.len(), 2, "{json}");
    let notes: Vec<String> = versions
        .iter()
        .zip(["0.3.0", "0.4.0"])
        .zip(&revisions)
        .map(|((v, version), revision)| {
            assert_eq!(v["version"], version, "{json}");
            assert_eq!(v["revision"], revision.as_str(), "{json}");
            assert_eq!(v["notes"]["answer"], "ok", "{json}");
            v["notes"]["value"].as_str().unwrap().to_string()
        })
        .collect();
    assert!(notes[0].contains("- The site at 0.3.0"), "{}", notes[0]);
    assert!(notes[1].contains("- The site at 0.4.0"), "{}", notes[1]);

    // The JSON's part is the core's, value for value.
    let history = History::open(&host.repo).unwrap();
    let core = timber_core::overview(
        &host.fakes.seams(),
        &history,
        &Kept::at(&history),
        &timber_core::OverviewAsk {
            branch: Branch::new("main"),
            component: None,
            revision: None,
            at: json["value"]["asked_at"].as_u64().unwrap(),
        },
    );
    assert_eq!(
        *part,
        answer_json(&Ok::<_, timber_seams::Not>(core))["value"]["environments"]
    );

    let (code, printed) = host.run(&["status"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    let environments = lines
        .iter()
        .position(|l| *l == "environments")
        .expect(&printed);
    assert_eq!(
        lines[environments + 1],
        "  0 running what they should, 1 behind, 0 waiting, 0 unreadable",
        "{printed}"
    );
    let srv = srv_line(&lines, &printed);
    assert_eq!(srv, environments + 2, "{printed}");
    assert_eq!(
        lines[srv],
        format!(
            "  srv  behind on repository/stable: runs 0.2.0 ({}) since 2026-11-05T09:00:00Z; \
             should run 0.4.0 ({})",
            index_of("0.2.0"),
            index_of("0.4.0")
        ),
        "{printed}"
    );
    assert_eq!(
        lines[srv + 1],
        "       ahead, the versions between:",
        "{printed}"
    );
    let mut beneath = srv + 2;
    for ((version, revision), text) in ["0.3.0", "0.4.0"].iter().zip(&revisions).zip(&notes) {
        assert_eq!(
            lines[beneath],
            format!("         {version} at {revision}"),
            "{printed}"
        );
        let shown: Vec<String> = text
            .lines()
            .map(|l| format!("           {l}").trim_end().to_string())
            .collect();
        assert_eq!(
            lines[beneath + 1..beneath + 1 + shown.len()],
            shown,
            "{printed}"
        );
        beneath += 1 + shown.len();
    }
    assert_eq!(lines[beneath], "", "{printed}");
}

// Requirement (environment-status): The copy answers where the store does
// not

#[test]
fn the_store_unreachable_after_a_fetch_prints_what_each_environment_ran_as_last_fetched() {
    let host = Host::new();
    srv_behind(&host);
    let (code, first) = host.run(&["status", "--json"]);
    assert_eq!(code, 0, "{first}");
    let first: Value = serde_json::from_str(&first).unwrap();
    let fetched = first["value"]["asked_at"].as_u64().unwrap();
    let srv = &first["value"]["environments"]["value"]["environments"][0];
    assert_eq!(srv["runs"]["value"]["read"], "source", "{first}");

    const WORDS: &str = "dial tcp 10.0.0.1:443: connect: connection refused";
    host.fakes.store.fault(
        StoreOp::ClaimsOf,
        Fault::Unreadable(WORDS.into()),
        Times::UntilHealed,
    );
    let (code, json) = host.run(&["status", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let again = &json["value"]["environments"]["value"]["environments"][0];
    assert_eq!(again["state"], srv["state"], "{json}");
    assert_eq!(again["between"], srv["between"], "{json}");
    for key in ["version", "index", "converged_at", "at", "claim", "signer"] {
        assert_eq!(
            again["runs"]["value"][key], srv["runs"]["value"][key],
            "{key}"
        );
    }
    assert_eq!(
        again["runs"]["value"]["read"],
        serde_json::json!({ "kept": { "at": fetched, "said": WORDS } }),
        "{json}"
    );

    let (code, printed) = host.run(&["status"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    let srv = srv_line(&lines, &printed);
    assert!(
        lines[srv].starts_with("  srv  behind on repository/stable: runs 0.2.0 ("),
        "{printed}"
    );
    assert!(
        lines[srv].ends_with(&format!(
            "; reports as last fetched {}",
            crate::render::utc(fetched)
        )),
        "{printed}"
    );
    assert_eq!(lines[srv + 1], WORDS, "{printed}");
    assert_eq!(
        lines[srv + 2],
        "       ahead, the versions between:",
        "{printed}"
    );
}

// Requirement (channel-status): A channel's changelog is its chain read
// with each promotion's notes

#[test]
fn status_prints_each_promotions_notes_beneath_it_and_carries_them_beside_the_channel() {
    let host = Host::new();
    let second = grown(&host);
    promotable(&host, &host.fakes, &second);
    for version in ["0.1.0", "0.2.0"] {
        let (code, printed) = host.run(&["promote", version, "--to", "stable"]);
        assert_eq!(code, 0, "{printed}");
    }

    let (code, json) = host.run(&["status", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let channel = &json["value"]["channels"]["value"]["channels"][0];
    assert_eq!(channel["channel"]["name"], "stable", "{json}");
    let notes = channel["notes"]
        .as_array()
        .expect("the notes beside the channel");
    assert_eq!(notes.len(), 2, "{json}");
    let published = &json["value"]["published"]["value"]["components"][0]["versions"];
    let mut texts = Vec::new();
    for (i, version) in ["0.1.0", "0.2.0"].into_iter().enumerate() {
        assert_eq!(notes[i]["version"], version, "{json}");
        assert_eq!(notes[i]["index"], published[i]["index"], "{json}");
        assert_eq!(notes[i]["notes"]["answer"], "ok", "{json}");
        // The same text as the notes of the version this host published.
        assert_eq!(
            notes[i]["notes"]["value"], published[i]["notes"]["value"],
            "{json}"
        );
        texts.push(notes[i]["notes"]["value"].as_str().unwrap().to_string());
    }
    assert_ne!(texts[0], texts[1]);

    let (code, printed) = host.run(&["status"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    let stable = lines
        .iter()
        .position(|l| l.starts_with("  stable "))
        .unwrap_or_else(|| panic!("no stable line: {printed}"));
    let mut at = stable + 1;
    for (version, text) in [("0.2.0", &texts[1]), ("0.1.0", &texts[0])] {
        assert!(
            lines[at].trim_start().starts_with(version) && lines[at].contains("asked by host"),
            "{printed}"
        );
        let indent = " ".repeat(lines[at].len() - lines[at].trim_start().len() + 2);
        let shown: Vec<String> = text
            .lines()
            .map(|l| format!("{indent}{l}").trim_end().to_string())
            .collect();
        assert_eq!(lines[at + 1..at + 1 + shown.len()], shown, "{printed}");
        at += 1 + shown.len();
    }
}

// Requirement: Status says when the landing was fetched

/// What the fake landing's remote says once it is cut off: git's words
/// over two lines, as a remote that cannot be reached answers.
const UNREACHABLE: &str = "fatal: unable to access 'https://forge.example/o/r.git/': \
                           Could not resolve host: forge.example\n\
                           fatal: Could not read from remote repository.\n";

#[test]
fn status_says_the_landing_was_read_as_last_fetched_and_shows_every_channel() {
    let host = Host::new();
    declared(&host.fakes, "beta");
    declared(&host.fakes, "stable");
    chained(&host, "stable", &["0.1.0", "0.2.0"]);
    let before = now();
    let (code, first) = host.run(&["status"]);
    let after = now();
    assert_eq!(code, 0, "{first}");
    assert!(
        !first.contains("landing:"),
        "a landing read from itself was named: {first}"
    );
    host.fakes.landing.unreachable(UNREACHABLE);

    let (code, printed) = host.run(&["status"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert!(
        lines[0].starts_with(&format!("main at {}, as of 20", host.head)),
        "{printed}"
    );
    let at = lines[1]
        .strip_prefix("  landing: as last fetched ")
        .unwrap_or_else(|| panic!("the landing's read is not under the branch: {printed}"));
    let fetched = (before..=after).map(crate::render::utc).find(|t| t == at);
    assert!(fetched.is_some(), "fetched at {at}, not when first read");
    assert!(
        printed.contains(&format!("{at}\n{UNREACHABLE}")),
        "the remote's words are not beneath, unchanged: {printed}"
    );
    let shown: Vec<&str> = first.lines().filter(|l| l.starts_with("  ")).collect();
    for line in shown {
        assert!(printed.contains(line), "{line:?} is not shown: {printed}");
    }
    assert!(!printed.contains("refused"), "{printed}");

    let (code, json) = host.run(&["status", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let channels = &json["value"]["channels"];
    assert_eq!(channels["answer"], "ok", "{json}");
    let kept = &channels["value"]["landing"]["kept"];
    assert_eq!(kept["said"], UNREACHABLE, "{json}");
    assert_eq!(
        kept["at"].as_u64().map(crate::render::utc).as_deref(),
        Some(at),
        "{json}"
    );
    assert_eq!(
        channels["value"]["channels"][1]["at"]["value"]["head"]["to"]["version"],
        "0.2.0"
    );
}

#[test]
fn status_prints_an_empty_channel_read_from_the_copy_as_last_read_with_its_words() {
    let host = Host::new();
    declared(&host.fakes, "preview");
    let before = now();
    assert_eq!(host.run(&["status"]).0, 0);
    let after = now();
    host.fakes.keeper.unreachable("fatal: unable to access");

    let (code, printed) = host.run(&["status"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    let preview = lines
        .iter()
        .position(|l| l.starts_with("  preview "))
        .unwrap_or_else(|| panic!("no preview line: {printed}"));
    let (said, when) = lines[preview]
        .split_once("empty, as last read ")
        .unwrap_or_else(|| panic!("preview is not empty as last read: {printed}"));
    assert!(said.trim() == "preview", "{printed}");
    assert!(when.starts_with("20") && when.ends_with('Z'), "{printed}");
    assert_eq!(
        lines[preview + 1].trim(),
        "fatal: unable to access",
        "{printed}"
    );

    let (code, printed) = host.run(&["status", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let at = &printed["value"]["channels"]["value"]["channels"][0]["at"]["value"];
    assert_eq!(at["state"], "empty", "{printed}");
    let kept = &at["read"]["kept"];
    assert_eq!(kept["said"], "fatal: unable to access", "{printed}");
    let read_at = kept["at"].as_u64().expect("the copy's time");
    assert!((before..=after).contains(&read_at), "{printed}");
}

// Requirement: Policy shows the policy in force

/// `id` as the policy names a signer, verified by a key of its name,
/// claiming `kinds` and, of verdicts, `checks`.
fn signer(id: &str, kinds: &[ClaimKind], checks: &[&str]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: checks.iter().map(|c| Check::new(*c)).collect(),
    }
}

/// `channel`'s policy scripted on the host's fake policy: the gate's
/// acceptance, bound to `bound_to` where it names a channel, promoted by
/// `promoters` and asked for by `approvers`.
fn channel_policy(
    host: &Host,
    channel: &str,
    bound_to: Option<&str>,
    promoters: &[&str],
    approvers: &[&str],
) -> ChannelId {
    let id = ChannelId::new(None, ChannelName::new(channel));
    host.fakes.policy.require(
        Audience::Channel(id.clone()),
        Required::Channel {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                environment: None,
                signers: vec![signer("gate", &[ClaimKind::Acceptance], &[])],
                asked_of: AskedOf::Every,
            }],
            bound_to: bound_to.map(ChannelName::new),
            promoted_by: promoters
                .iter()
                .map(|p| signer(p, &[ClaimKind::Acceptance], &[]))
                .collect(),
            approvers: approvers.iter().map(|a| signer(a, &[], &[])).collect(),
        },
    );
    id
}

/// The lines `timber policy` prints over the host's fakes, and its JSON
/// compared value for value with the core's answer.
fn policy_lines(host: &Host) -> (Vec<String>, Value) {
    let (code, printed) = host.run(&["policy"]);
    assert_eq!(code, 0, "{printed}");
    let (code, json) = host.run(&["policy", "--json"]);
    assert_eq!(code, 0, "{json}");
    let json: Value = serde_json::from_str(&json).unwrap();
    let history = History::open(&host.repo).unwrap();
    let core = in_force(
        &host.fakes.seams(),
        &history,
        &InForceAsk {
            branch: Branch::new("main"),
        },
    );
    assert_eq!(json, answer_json(&core));
    (printed.lines().map(str::to_string).collect(), json)
}

#[test]
fn policy_shows_a_verdict_asked_of_built_artifacts_alone() {
    let host = Host::new();
    host.fakes.policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![
                RequiredClaim {
                    kind: ClaimKind::Provenance,
                    check: None,
                    environment: None,
                    signers: vec![signer("laptop", &[ClaimKind::Provenance], &[])],
                    asked_of: AskedOf::Every,
                },
                RequiredClaim {
                    kind: ClaimKind::Verdict,
                    check: Some(Check::new("cli:test")),
                    environment: None,
                    signers: vec![signer("box", &[ClaimKind::Verdict], &["cli:test"])],
                    asked_of: AskedOf::Built,
                },
            ],
        },
    );

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

#[test]
fn policy_shows_the_givens_held_off_beneath_the_table_with_their_lines() {
    let host = Host::new();
    let beta = channel_policy(&host, "beta", None, &["gate"], &[]);
    host.fakes.policy.follows(&beta, false);
    channel_policy(&host, "stable", Some("beta"), &["gate"], &[]);
    host.fakes.policy.holds(
        &ChannelId::new(None, ChannelName::new("alpha")),
        Moves::EachPublish { prereleases: true },
        "[channels.alpha]",
    );
    host.fakes.policy.holds(
        &ChannelId::new(None, ChannelName::new("lts")),
        Moves::Bound {
            to: ChannelName::new("stable"),
            after: vec![],
        },
        "[channels.lts]",
    );

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
fn policy_shows_alpha_taking_prereleases_and_stables_approvers_value_for_value() {
    let host = Host::new();
    let alpha = channel_policy(&host, "alpha", None, &["gate"], &[]);
    host.fakes.policy.follows(&alpha, true);
    let beta = channel_policy(&host, "beta", None, &["gate"], &[]);
    host.fakes.policy.follows(&beta, false);
    channel_policy(&host, "stable", Some("beta"), &["ci", "gate"], &["ci"]);

    let (lines, json) = policy_lines(&host);
    let printed = lines.join("\n");
    let at = |line: &str| {
        lines
            .iter()
            .position(|l| l == line)
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

    let channels = json["value"]["channels"]["value"]["in_force"]
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

// Requirement: Status names the page

// Requirement: Serve is a command

#[test]
fn serve_takes_no_argument_that_would_move_it() {
    let err =
        Cli::try_parse_from(["timber", "serve", "--port", "8080"]).expect_err("serve took a port");
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("--port"), "{err}");
}

#[test]
fn a_moment_is_rendered_in_utc() {
    assert_eq!(crate::render::utc(0), "1970-01-01T00:00:00Z");
    assert_eq!(crate::render::utc(1_793_000_000), "2026-10-26T07:33:20Z");
}

// Requirement: A host with no bindings file is bound to the defaults

// Requirement: A host with no bindings file is bound to the defaults, a
// broken file

// Requirement: The host's bindings are one file beside the landing's own

// Requirement: Promote takes a version and a channel

fn stable() -> ChannelId {
    ChannelId::new(None, ChannelName::new("stable"))
}

/// A second feature committed on `main` after the host's head, and its
/// revision.
fn grown(host: &Host) -> Revision {
    fs::write(host.repo.join("site/index.html"), "<p>hello again</p>").unwrap();
    host.commit_all("feat: the site grows")
}

/// Over `fakes`, the host's head published as 0.1.0 and `second`, landed
/// and built, as 0.2.0, each index carrying the host's acceptance, and
/// the policy for `stable` requiring it of the host, which it names for
/// acceptance and promotion (108).
fn promotable(host: &Host, fakes: &Fakes, second: &Revision) {
    fakes.landing.land(&Branch::new("main"), second);
    fakes.build.made(
        second,
        &OutputName::new("site"),
        Report {
            build: BuildId::new("site-build-2"),
            outcome: Outcome::Pass,
            files: vec!["dist/site.tar".into()],
        },
    );
    let history = History::open(&host.repo).unwrap();
    for revision in [&host.head, second] {
        let published = publish(
            &fakes.seams(),
            &history,
            &Kept::at(&history),
            &Ask {
                branch: Branch::new("main"),
                revision: revision.clone(),
                component: None,
                signer: SignerId::new("host"),
                store: store(),
                tree: history.workdir().unwrap().display().to_string(),
            },
        )
        .unwrap();
        let Ok(timber_core::Published::Version { index, .. }) = &published.components[0].1 else {
            panic!("nothing was published: {published:?}")
        };
        let acceptance = Claim {
            subject: index.clone(),
            kind: ClaimKind::Acceptance,
            revision: revision.clone(),
            build: None,
            body: serde_json::json!({}),
        };
        let envelope = fakes
            .signer
            .sign(&acceptance, &SignerId::new("host"))
            .unwrap();
        timber_seams::Store::attach(&fakes.store, index, &envelope).unwrap();
    }
    let host_signer = NamedSigner {
        id: SignerId::new("host"),
        verified_by: "the host's key".into(),
        kinds: vec![ClaimKind::Acceptance, ClaimKind::Promotion],
        checks: vec![],
    };
    fakes.policy.require(
        Audience::Channel(stable()),
        Required::Channel {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                environment: None,
                signers: vec![host_signer.clone()],
                asked_of: AskedOf::Every,
            }],
            bound_to: None,
            promoted_by: vec![host_signer],
            approvers: vec![],
        },
    );
}

#[test]
fn promote_through_the_command_line_prints_the_cores_answer_value_for_value() {
    let host = Host::new();
    let second = grown(&host);
    promotable(&host, &host.fakes, &second);
    let (code, printed) = host.run(&["promote", "0.2.0", "--to", "stable", "--json"]);
    assert_eq!(code, 0, "{printed}");
    let printed: Value = serde_json::from_str(&printed).unwrap();

    let chain = timber_seams::Keeper::chain(&host.fakes.keeper, &stable()).unwrap();
    let head = chain.claims.last().unwrap();
    let asked = Promotion::of(&Claim::from_envelope(head).unwrap()).unwrap();
    assert_eq!(printed["value"]["claim"], head.digest().as_str());

    // The same question over an identical binding, at the moment the
    // command asked it.
    let other = fakes_at(&host.head);
    promotable(&host, &other, &second);
    let core = promote(
        &other.seams(),
        &History::open(&host.repo).unwrap(),
        &PromoteAsk {
            branch: Branch::new("main"),
            component: None,
            channel: stable().name,
            to: To::Version(Version::new("0.2.0")),
            from: None,
            signer: SignerId::new("host"),
            at: asked.at,
        },
    );
    assert_eq!(printed, answer_json(&core));
    assert_eq!(printed["value"]["to"]["version"], "0.2.0");
}

#[test]
fn promote_from_a_version_the_channel_has_left_exits_1_naming_where_it_is() {
    let host = Host::new();
    let second = grown(&host);
    promotable(&host, &host.fakes, &second);
    let (code, printed) = host.run(&["promote", "0.2.0", "--to", "stable"]);
    assert_eq!(code, 0, "{printed}");
    assert!(
        printed.starts_with("stable moved to 0.2.0 (sha256:")
            && printed.contains("its first promotion"),
        "{printed}"
    );
    let (code, printed) = host.run(&["promote", "0.1.0", "--to", "stable", "--from", "0.1.0"]);
    assert_eq!(code, 1, "{printed}");
    assert!(printed.contains("stable is at 0.2.0"), "{printed}");
    let (code, printed) = host.run(&["promote", "--back", "--to", "stable"]);
    assert_eq!(code, 4, "{printed}");
    assert!(printed.contains("a version stable moved from"), "{printed}");
}

#[test]
fn a_version_and_back_together_are_arguments_promote_cannot_take() {
    let host = Host::new();
    let err = Cli::try_parse_from(["timber", "promote", "0.2.0", "--back", "--to", "stable"])
        .expect_err("promote took a version and --back");
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("--back"), "{err}");
    let err = Cli::try_parse_from(["timber", "promote", "--to", "stable"])
        .expect_err("promote took neither a version nor --back");
    assert_eq!(err.exit_code(), 2);
    assert_eq!(host.calls(), 0, "the core was asked");
}

// Requirement: A prerelease moves only the channels that take prereleases

// Requirement: Who may ask for a channel

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

#[test]
fn policy_as_text_shows_the_layout_absent_beside_the_channels_and_exits_0() {
    let host = Host::new();
    declared(&host.fakes, "beta");
    host.fakes
        .policy
        .follows(&ChannelId::new(None, ChannelName::new("beta")), false);
    host.fakes.policy.require(
        Audience::Channel(stable()),
        Required::Channel {
            claims: vec![],
            bound_to: Some(ChannelName::new("beta")),
            promoted_by: vec![],
            approvers: vec![],
        },
    );
    let (code, printed) = host.run(&["policy"]);
    assert_eq!(code, 0, "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(lines[0], format!("main at {}", host.head), "{printed}");
    let at = |line: &str| {
        lines
            .iter()
            .position(|l| *l == line)
            .unwrap_or_else(|| panic!("no line {line:?}: {printed}"))
    };
    assert_eq!(
        lines[at("acceptance") + 1],
        "  requires no claim",
        "{printed}"
    );
    let layout = lines[at("layout") + 1];
    assert!(
        layout.starts_with("  ") && layout.contains("holds no") && layout.contains("layout"),
        "{printed}"
    );
    let channels = at("channels");
    assert_eq!(lines[channels + 1], "  beta, each publish", "{printed}");
    assert_eq!(lines[channels + 5], "  stable, bound to beta", "{printed}");
    assert_eq!(
        lines[channels + 7],
        "    promoted by no signer",
        "{printed}"
    );
    assert_eq!(
        lines[channels + 8],
        "    may be asked for by any asker",
        "{printed}"
    );
    assert!(
        !lines.contains(&"held off"),
        "a policy holding nothing off showed a held part: {printed}"
    );
}

#[test]
fn an_unreadable_channels_answer_is_shown_once_in_place_of_the_channels_and_the_held_part() {
    let host = Host::new();
    host.fakes.policy.holds(
        &ChannelId::new(None, ChannelName::new("canary")),
        timber_seams::Moves::Promotion,
        "[channels.canary]",
    );
    host.fakes.policy.fault(
        timber_fakes::PolicyOp::Channels,
        Fault::Unreadable("the channels' table cannot be read".into()),
        Times::Once,
    );
    let (code, printed) = host.run(&["policy"]);
    assert_eq!(code, 0, "{printed}");
    assert_eq!(
        printed
            .matches("the channels' table cannot be read")
            .count(),
        1,
        "{printed}"
    );
    assert!(printed.contains("\nchannels\n"), "{printed}");
    assert!(!printed.contains("held off"), "{printed}");
    assert!(!printed.contains("canary"), "{printed}");
    assert!(
        printed.contains("acceptance\n  requires no claim"),
        "{printed}"
    );
}

#[test]
fn policy_at_a_head_not_landed_exits_1_naming_the_branch_and_the_revision() {
    let host = Host::new();
    let second = grown(&host);
    let (code, printed) = host.run(&["policy"]);
    assert_eq!(code, 1, "{printed}");
    assert!(
        printed.contains("main") && printed.contains(second.as_str()),
        "{printed}"
    );
    assert!(
        host.fakes.policy.calls().is_empty(),
        "the policy was asked of a head not landed"
    );
}

// Requirement: The binary says its version, its revision and its build
