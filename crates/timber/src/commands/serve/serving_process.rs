//! Spec serving-process: each route of the host's one serving process,
//! asked of its own handling of a request (`respond_with`) with no
//! socket, each checkout bound to the fakes and its history made in the
//! test's own process with libgit2. The resident's life, its port, its
//! lock, its askers and its idle endings, is the end-to-end set's,
//! `tests/end_to_end/serve.rs`.

use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use clap::Parser;
use git2::{IndexAddOption, Oid, Repository, RepositoryInitOptions, Signature};
use serde_json::Value;
use tempfile::TempDir;
use timber_core::{
    answer_json, artifact_name, place, sync_checkout, CheckoutAsk, Entry, History, Kept, Layouts,
    Noted, OverviewAsk, Reported, RepositoryName,
};
use timber_fakes::{FakeKeeper, FakeProjection, FakeSharedHistory, FakeSigner, FakeStore, Fakes};
use timber_seams::{
    Artifact, ArtifactMeta, AskedOf, Audience, Branch, ChannelId, ChannelName, Claim, ClaimKind,
    ComponentName, DeclaredEnvironment, Digest, Ecosystem, EcosystemName, EnvironmentName,
    IndexMeta, Keeper as _, Manifest, Moved, NamedSigner, Not, Pinned, Promotion, Request,
    Required, RequiredClaim, Revision, Satisfied, Seam, Signer as _, SignerId, Source, Store as _,
    Version,
};

use super::host::{name, Host, REPOSITORIES};
use super::page::escape;
use super::{respond_with, Response};
use crate::bindings::{For, Teller};
use crate::commands::status::now;
use crate::{run, Binding, Cli};

const MAIN: &str = "main";

/// The policy file every world commits first: its one output, which
/// gives the repository's one component, and the channels the fake
/// policy declares.
const POLICY: &str = "outputs = [\"site\"]\n\n[projections]\nsite = \"file\"\n\n\
                      [channels.beta]\nrequire = []\n\n[channels.stable]\nbound_to = \"beta\"\n\
                      require = []\n";

/// The environments' four counts of none, as a page renders them.
const NONE: &str = "0 running what they should, 0 behind, 0 waiting on a person, 0 unreadable";

/// The fixed author, committing now.
fn author() -> Signature<'static> {
    Signature::now("A. Uthor", "author@example.com").unwrap()
}

/// A commit in the repository at `path` with `message` writing each
/// file, by the fixed author; its revision.
fn commit_at(path: &Path, message: &str, files: &[(&str, &str)]) -> Revision {
    let repo = Repository::open(path).unwrap();
    let mut index = repo.index().unwrap();
    for (file, contents) in files {
        let at = path.join(file);
        fs::create_dir_all(at.parent().unwrap()).unwrap();
        fs::write(&at, contents).unwrap();
        index
            .add_all([file], IndexAddOption::DEFAULT, None)
            .unwrap();
    }
    index.write().unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    let parent = repo.head().ok().map(|h| h.peel_to_commit().unwrap());
    let parents: Vec<&git2::Commit> = parent.iter().collect();
    let message = git2::message_prettify(message, None).unwrap();
    let made = repo
        .commit(
            Some("HEAD"),
            &author(),
            &author(),
            &message,
            &tree,
            &parents,
        )
        .unwrap();
    Revision::new(made.to_string())
}

/// The host the tests serve, in `dir`: a state directory, and a bindings
/// file, a home and a data directory none of which is there, since every
/// checkout is bound to the fakes.
fn host_in(dir: &Path) -> Host {
    Host {
        state: dir.join("state/timber"),
        config: dir.join("config/timber/config.toml"),
        home: None,
        data: None,
    }
}

/// A repository on `main` with the policy committed, made in the test's
/// own process; the host's record holding one entry; the fakes landing
/// its head on `main` and declaring `beta`, and `stable` bound to it,
/// each requiring nothing.
struct World {
    _root: TempDir,
    repo: PathBuf,
    host: Host,
    head: Revision,
    fakes: Fakes,
}

impl World {
    fn new() -> World {
        World::on(None)
    }

    /// A second repository, of its own, on `host`.
    fn beside(host: &Host) -> World {
        World::on(Some(host.clone()))
    }

    fn on(host: Option<Host>) -> World {
        let root = TempDir::new().unwrap();
        let repo = root.path().join("repo");
        Repository::init_opts(&repo, RepositoryInitOptions::new().initial_head(MAIN)).unwrap();
        let host = host.unwrap_or_else(|| host_in(root.path()));
        let fakes = Fakes::new();
        for (channel, bound_to) in [("beta", None), ("stable", Some("beta"))] {
            fakes.policy.require(
                Audience::Channel(ChannelId::new(None, ChannelName::new(channel))),
                Required::Channel {
                    claims: vec![],
                    bound_to: bound_to.map(ChannelName::new),
                    promoted_by: vec![],
                    approvers: vec![],
                },
            );
        }
        let head = commit_at(
            &repo,
            "feat: the channels",
            &[(".config/timber.toml", POLICY)],
        );
        fakes.landing.land(&Branch::new(MAIN), &head);
        let world = World {
            _root: root,
            repo,
            host,
            head,
            fakes,
        };
        Kept::at(&world.history())
            .write(&entry(&world.head, None))
            .unwrap();
        world
    }

    fn history(&self) -> History {
        History::reaching(&self.repo, std::sync::Arc::new(FakeSharedHistory)).unwrap()
    }

    /// A commit with `message` writing each file, by the fixed author;
    /// its revision.
    fn commit(&self, message: &str, files: &[(&str, &str)]) -> Revision {
        commit_at(&self.repo, message, files)
    }

    /// A lightweight tag `name` at `at`.
    fn tag(&self, name: &str, at: &Revision) {
        let repo = Repository::open(&self.repo).unwrap();
        let target = repo
            .find_object(Oid::from_str(at.as_str()).unwrap(), None)
            .unwrap();
        repo.tag_lightweight(name, &target, false).unwrap();
    }

    /// The fakes as the one binding the checkout is bound to.
    fn binding(&self) -> Binding {
        Binding {
            seams: self.fakes.seams(),
            store: Source::new(Seam::Store, "fake"),
            signer: None,
            gate: Teller::Job("the gate's job".into()),
            trust: vec![],
            shared: std::sync::Arc::new(FakeSharedHistory),
        }
    }

    /// The checkout named to the host as `o/r`.
    fn named(&self) -> RepositoryName {
        self.named_as("o", "r")
    }

    /// The checkout named to the host as `owner/name`.
    fn named_as(&self, owner: &str, name: &str) -> RepositoryName {
        let repository = RepositoryName {
            owner: owner.into(),
            name: name.into(),
        };
        self::name(&self.host, &repository, &self.repo).unwrap();
        repository
    }

    /// `method` of `path` asked of the process's own handling of a
    /// request, the checkout bound to the fakes for status.
    fn ask(&self, method: &str, path: &str) -> Response {
        among(&[self], method, path)
    }

    fn get(&self, path: &str) -> (u16, String) {
        let response = self.ask("GET", path);
        (response.status, text(&response))
    }

    fn json(&self, path: &str) -> (u16, Value) {
        let (status, body) = self.get(path);
        (
            status,
            serde_json::from_str(&body).expect("a JSON document"),
        )
    }

    /// The core's row of `repository` from the status answer in the
    /// checkout, as a program reads it.
    fn row(&self, repository: &RepositoryName) -> Value {
        let history = self.history();
        let overview = timber_core::overview(
            &self.binding().seams,
            &history,
            &Kept::at(&history),
            &OverviewAsk {
                branch: Branch::new(MAIN),
                component: None,
                revision: None,
                at: 0,
            },
        );
        answer_json(&Ok::<_, Not>(timber_core::row(repository, &overview)))
    }

    /// `channel` promoted in the fake keeper to each version in turn, from
    /// the head it is at, sealed as the fake signer seals; every claim's
    /// digest.
    fn promote(&self, channel: &str, versions: &[&str]) -> Vec<Digest> {
        let moves: Vec<Moved> = versions.iter().map(|v| moved(v)).collect();
        self.promote_to(channel, &moves)
    }

    /// `channel` promoted to each of `moves` in turn, as
    /// [`World::promote`] promotes it.
    fn promote_to(&self, channel: &str, moves: &[Moved]) -> Vec<Digest> {
        let channel = ChannelId::new(None, ChannelName::new(channel));
        let keeper = &self.fakes.keeper;
        let gate = FakeSigner::new();
        gate.hold(&SignerId::new("gate"));
        let mut head = timber_core::channel(keeper, &channel).unwrap();
        let mut claims = Vec::new();
        for to in moves {
            let (from, predecessor) = match &head {
                timber_core::Channel::At { head, chain, .. } => {
                    (Some(head.to.clone()), chain.last().map(|(d, _)| d.clone()))
                }
                timber_core::Channel::Empty { .. } => (None, None),
            };
            let promotion = Promotion {
                channel: channel.name.clone(),
                component: channel.component_name(),
                to: to.clone(),
                from,
                predecessor: predecessor.clone(),
                asked_by: SignerId::new("gate"),
                policy: Satisfied {
                    revision: self.head.clone(),
                    claims: vec![acceptance(to.version.as_str())],
                },
                at: 1_793_000_000 + claims.len() as u64,
            };
            let envelope = gate
                .sign(&promotion.claim(self.head.clone()), &SignerId::new("gate"))
                .unwrap();
            let claim = keeper
                .append(&channel, &envelope, predecessor.as_ref())
                .unwrap();
            claims.push(claim);
            head = timber_core::channel(keeper, &channel).unwrap();
        }
        claims
    }

    /// `timber status --json` in the checkout, over the fakes.
    fn status(&self) -> Value {
        let cli = Cli::try_parse_from(["timber", "status", "--json"]).unwrap();
        let mut out = Vec::new();
        assert_eq!(run(&cli, &Ok(self.binding()), &self.repo, &mut out), 0);
        serde_json::from_slice(&out).unwrap()
    }
}

/// `method` of `path` asked of the process's own handling of a request
/// on the host of `worlds`, each checkout bound for status to the fakes
/// of the world it is.
fn among(worlds: &[&World], method: &str, path: &str) -> Response {
    respond_with(
        asked_at(),
        &worlds[0].host,
        &|checkout, command| {
            assert_eq!(command, For::Status, "a route bound for another");
            let world = worlds
                .iter()
                .find(|w| w.repo == checkout)
                .unwrap_or_else(|| panic!("{} bound, which no world is", checkout.display()));
            Ok(world.binding())
        },
        method,
        path,
    )
}

/// The record's entry of 0.1.0 at `revision`, with `notes`.
fn entry(revision: &Revision, notes: Option<Noted>) -> Entry {
    Entry {
        component: ComponentName::new("repository"),
        version: Version::new("0.1.0"),
        tag: "v0.1.0".into(),
        revision: revision.clone(),
        branch: Branch::new(MAIN),
        index: Digest::of(b"0.1.0's index"),
        store: Source::new(Seam::Store, "layout /srv/timber/layout"),
        notes,
    }
}

fn moved(version: &str) -> Moved {
    Moved {
        version: Version::new(version),
        index: Digest::of(format!("{version}'s index").as_bytes()),
    }
}

/// The digest `promote` names as the claim that satisfied the policy for
/// `version`'s promotion.
fn acceptance(version: &str) -> Digest {
    Digest::of(format!("{version}'s acceptance").as_bytes())
}

/// The address the process's own handling of a request is asked at in
/// these tests, bound by nothing and never the core's.
fn asked_at() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 7499))
}

fn text(response: &Response) -> String {
    String::from_utf8(response.body.clone()).unwrap()
}

/// `value` with each moment asked set to 0.
fn unasked(mut value: Value) -> Value {
    value["value"]["asked_at"] = Value::from(0);
    value
}

/// Every file under `dir` but git's own, with its contents.
fn files(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut all = Vec::new();
    if !dir.exists() {
        return all;
    }
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(at) = dirs.pop() {
        for entry in fs::read_dir(&at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == ".git") {
                    continue;
                }
                dirs.push(path);
            } else {
                all.push((path.clone(), fs::read(&path).unwrap()));
            }
        }
    }
    all.sort();
    all
}

// Requirement: A repository's page and its JSON are its status answer

#[test]
fn a_named_repositorys_json_is_its_status_answer_in_its_checkout() {
    let world = World::new();
    let repository = world.named();
    world.promote("stable", &["0.1.0", "0.2.0"]);
    let before = now();
    let (status, served) = world.json(&format!("{}/status.json", repository.path()));
    let after = now();
    assert_eq!(status, 200, "{served}");
    let asked = served["value"]["asked_at"].as_u64().unwrap();
    assert!((before..=after).contains(&asked), "{asked}");
    let printed = world.status();
    assert_eq!(unasked(served.clone()), unasked(printed));
    let channels = &served["value"]["channels"]["value"]["channels"];
    assert_eq!(channels[1]["channel"]["name"], "stable", "{served}");
    assert_eq!(channels[1]["at"]["value"]["head"]["to"]["version"], "0.2.0");
    assert_eq!(served["value"]["published"]["answer"], "ok", "{served}");
}

#[test]
fn the_page_names_every_channel_version_and_promotion_and_escapes_the_words() {
    let mut world = World::new();
    let repository = world.named();
    let claims = world.promote("stable", &["0.1.0", "0.2.0"]);
    let (_, served) = world.json("/o/r/status.json");
    let (status, page) = world.get(&format!("{}/status", repository.path()));
    assert_eq!(status, 200, "{page}");
    assert!(page.starts_with("<!doctype html>"), "{page}");
    assert!(
        page.contains("href=\"/\""),
        "the page links no root: {page}"
    );
    for channel in served["value"]["channels"]["value"]["channels"]
        .as_array()
        .unwrap()
    {
        let name = channel["channel"]["name"].as_str().unwrap();
        assert!(page.contains(&format!("<td>{name}</td>")), "{name}: {page}");
    }
    for version in ["0.1.0", "0.2.0"] {
        assert!(page.contains(version), "{version}: {page}");
    }
    for claim in &claims {
        assert!(page.contains(claim.as_str()), "{claim}: {page}");
    }
    assert!(
        page.contains(&format!(
            "<code>{}</code>, 2026-10-26T07:33:21Z; claim <code>{}</code> after <code>{}</code>",
            acceptance("0.2.0"),
            claims[1],
            claims[0]
        )),
        "each promotion names the claims that satisfied its policy and its predecessor: {page}"
    );
    assert!(
        page.contains(&format!(
            "under the policy at <code>{}</code> satisfied by <code>{}</code>",
            world.head,
            acceptance("0.1.0")
        )),
        "{page}"
    );
    // The policy declares no environment: four counts of none.
    assert!(page.contains(NONE), "{page}");
    assert!(
        page.contains("The policy declares no environment."),
        "{page}"
    );
    assert!(page.contains("v0.1.0"), "the record's part: {page}");

    // A ledger whose words hold markup: each channel's chain is
    // unreadable with them, which the page escapes.
    let keeper = FakeKeeper::new();
    keeper.unreachable("<script>alert(1)</script>");
    world.fakes.keeper = keeper;
    let (status, page) = world.get("/o/r/status");
    assert_eq!(status, 200, "{page}");
    assert!(
        page.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
        "{page}"
    );
    assert!(!page.contains("<script>"), "{page}");
}

#[test]
fn the_page_and_its_json_say_the_landing_was_read_as_last_fetched_as_the_command_line_does() {
    let world = World::new();
    let repository = world.named();
    world.promote("stable", &["0.1.0"]);
    let (_, first) = world.json(&format!("{}/status.json", repository.path()));
    assert_eq!(
        first["value"]["channels"]["value"]["landing"], "source",
        "{first}"
    );
    let (_, page) = world.get(&format!("{}/status", repository.path()));
    assert!(!page.contains("landing:"), "{page}");
    world
        .fakes
        .landing
        .unreachable("fatal: 'origin' does not appear to be a git repository");

    let (status, served) = world.json(&format!("{}/status.json", repository.path()));
    assert_eq!(status, 200, "{served}");
    let kept = &served["value"]["channels"]["value"]["landing"]["kept"];
    let at = kept["at"].as_u64().unwrap_or_else(|| panic!("{served}"));
    let said = kept["said"].as_str().unwrap();
    assert!(
        said.contains("does not appear to be a git repository"),
        "{said}"
    );
    assert_eq!(unasked(served.clone()), unasked(world.status()));
    let (status, page) = world.get(&format!("{}/status", repository.path()));
    assert_eq!(status, 200, "{page}");
    assert!(
        page.contains(&format!(
            "<p>landing: as last fetched {}</p>\n<pre>{}</pre>",
            crate::render::utc(at),
            escape(said)
        )),
        "{page}"
    );
    assert!(page.contains("<td>stable</td>"), "{page}");
}

#[test]
fn a_promotion_after_one_request_is_shown_by_the_next() {
    let world = World::new();
    world.named();
    world.promote("stable", &["0.1.0"]);
    let head = |served: &Value| {
        served["value"]["channels"]["value"]["channels"][1]["at"]["value"]["head"]["to"]["version"]
            .clone()
    };
    let (_, first) = world.json("/o/r/status.json");
    assert_eq!(head(&first), "0.1.0", "{first}");
    world.promote("stable", &["0.2.0"]);
    let (_, second) = world.json("/o/r/status.json");
    assert_eq!(head(&second), "0.2.0", "{second}");
    let (_, page) = world.get("/o/r/status");
    assert!(page.contains("0.2.0"), "{page}");
}

#[test]
fn the_changelog_beside_the_channel_shows_each_promotions_notes_escaped_or_why_they_cannot_be_read()
{
    let world = World::new();
    let feature = world.commit(
        "feat: <b>the</b> page & its notes",
        &[("site/index.html", "<p>the page</p>")],
    );
    world.fakes.landing.land(&Branch::new(MAIN), &feature);
    world.tag("v0.1.0", &feature);
    let repository = world.named();
    // 0.2.0 was never tagged here: its notes cannot be rendered.
    world.promote("stable", &["0.1.0", "0.2.0"]);
    let (_, served) = world.json(&format!("{}/status.json", repository.path()));
    assert_eq!(unasked(served.clone()), unasked(world.status()));
    let notes = &served["value"]["channels"]["value"]["channels"][1]["notes"];
    assert_eq!(notes[0]["version"], "0.1.0", "{served}");
    assert_eq!(notes[0]["notes"]["answer"], "ok", "{served}");
    let text = notes[0]["notes"]["value"].as_str().unwrap();
    assert!(text.contains("<b>the</b> page & its notes"), "{text}");
    assert_eq!(notes[1]["version"], "0.2.0", "{served}");
    assert_eq!(notes[1]["notes"]["answer"], "unreadable", "{served}");
    let said = notes[1]["notes"]["unreadable"]["said"].as_str().unwrap();
    assert!(said.contains("no tag v0.2.0"), "{said}");

    let (status, page) = world.get(&format!("{}/status", repository.path()));
    assert_eq!(status, 200, "{page}");
    // Inside each promotion's item, after who asked, the notes of the
    // version it moved to, the same text the JSON carries, escaped.
    assert!(
        page.contains(&format!(
            "asked by gate, under the policy at <code>{}</code> satisfied by <code>{}</code>, \
             2026-10-26T07:33:20Z; claim",
            world.head,
            acceptance("0.1.0")
        )),
        "{page}"
    );
    assert!(
        page.contains(&format!("<pre>{}</pre></li>", escape(text))),
        "{page}"
    );
    assert!(
        page.contains("&lt;b&gt;the&lt;/b&gt; page &amp; its notes"),
        "{page}"
    );
    assert!(!page.contains("<b>the</b>"), "{page}");
    // In the unreadable version's place, the answer with the history's
    // words.
    assert!(
        page.contains(&format!(
            "could not read the notes of repository 0.2.0</span><pre>{}</pre></li>",
            escape(said)
        )),
        "{page}"
    );
}

#[test]
fn the_notes_of_a_version_this_host_published_are_shown_beneath_its_row() {
    let world = World::new();
    let fix = world.commit(
        "fix: <i>the</i> record & its notes",
        &[("site/index.html", "<p>the record</p>")],
    );
    let kept = Kept::at(&world.history());
    fs::remove_dir_all(kept.dir()).unwrap();
    kept.write(&entry(
        &fix,
        Some(Noted {
            last: None,
            commits: vec![fix.clone(), world.head.clone()],
        }),
    ))
    .unwrap();
    let repository = world.named();
    let (_, served) = world.json(&format!("{}/status.json", repository.path()));
    let version = &served["value"]["published"]["value"]["components"][0]["versions"][0];
    assert_eq!(version["notes"]["answer"], "ok", "{served}");
    let text = version["notes"]["value"].as_str().unwrap();
    assert!(text.contains("<i>the</i> record & its notes"), "{text}");

    let (status, page) = world.get(&format!("{}/status", repository.path()));
    assert_eq!(status, 200, "{page}");
    let row = page
        .find(&format!("<td><code>{fix}</code></td>"))
        .unwrap_or_else(|| panic!("no row for the version: {page}"));
    let notes = format!(
        "</tr>\n<tr><td colspan=\"7\"><pre>{}</pre></td></tr>",
        escape(text)
    );
    assert_eq!(
        page[row..]
            .find(&notes)
            .map(|at| page[row..row + at].contains("</tr>")),
        Some(false),
        "the notes are not the row beneath the version's: {page}"
    );
    assert!(!page.contains("<i>the</i>"), "{page}");
}

#[test]
fn a_repository_none_named_is_404_with_its_absent_answer() {
    let world = World::new();
    world.named();
    let (status, served) = world.json("/agentplot/timber/status.json");
    assert_eq!(status, 404, "{served}");
    assert_eq!(served["answer"], "absent", "{served}");
    let what = served["absent"]["what"].as_str().unwrap();
    assert!(what.contains("agentplot/timber"), "{what}");
    assert!(what.contains("has named it"), "{what}");
    let (status, page) = world.get("/agentplot/timber/status");
    assert_eq!(status, 404);
    assert!(page.contains("has named it"), "{page}");
}

#[test]
fn another_path_is_404_and_another_method_405() {
    let world = World::new();
    world.named();
    for path in ["/nothing", "/o/r", "//r/status", "/o//npm/x"] {
        assert_eq!(world.get(path).0, 404, "{path}");
    }
    assert_eq!(world.ask("POST", "/serving.json").status, 405);
    assert_eq!(world.ask("HEAD", "/o/r/status").status, 405);
}

#[test]
fn no_request_writes_a_file_of_the_record_or_the_repository() {
    let world = World::new();
    world.named();
    world.promote("stable", &["0.1.0"]);
    let record = Kept::at(&world.history()).dir().to_path_buf();
    let repositories = world.host.state.join(REPOSITORIES);
    let before = (
        files(&world.repo),
        files(&record),
        fs::read(&repositories).unwrap(),
    );
    for path in [
        "/serving.json",
        "/",
        "/status.json",
        "/o/r/status.json",
        "/o/r/status",
        "/x/y/status.json",
        "/elsewhere",
    ] {
        world.get(path);
    }
    assert_eq!(
        (
            files(&world.repo),
            files(&record),
            fs::read(&repositories).unwrap()
        ),
        before,
        "a request wrote a file of the record or the repository"
    );
}

// Requirement: A repository's page and its JSON are its status answer,
// the notes between beside an environment that is behind

/// On `world`: a commit whose subject holds markup, tagged 0.2.0 over
/// 0.1.0 at the world's head and landed; at it, `srv` declared following
/// `beta`, and a deployer whom the policy lets claim a report; both
/// versions' indexes in the fake store; `beta` promoted to each in turn;
/// and the deployer's one report, attached to 0.1.0's index, saying `srv`
/// runs it. The revision of 0.2.0.
fn srv_behind(world: &World) -> Revision {
    let second = world.commit(
        "feat: <script>alert(1)</script> for srv",
        &[("site/index.html", "<p>srv</p>")],
    );
    world.tag("v0.1.0", &world.head);
    world.tag("v0.2.0", &second);
    world.fakes.landing.land(&Branch::new(MAIN), &second);
    let beta = ChannelId::new(None, ChannelName::new("beta"));
    world.fakes.policy.declares_at(
        &second,
        DeclaredEnvironment {
            name: EnvironmentName::new("srv"),
            channel: beta,
            waits: false,
            approvers: vec![],
            target: None,
        },
    );
    let deployer = SignerId::new("deployer");
    world.fakes.policy.require(
        Audience::Layout,
        Required::Layout {
            claims: vec![],
            given: false,
            signers: vec![NamedSigner {
                id: deployer.clone(),
                verified_by: "the deployer's key".into(),
                kinds: vec![ClaimKind::Report],
                checks: vec![],
            }],
        },
    );
    let moves: Vec<Moved> = ["0.1.0", "0.2.0"]
        .iter()
        .map(|version| {
            let meta = IndexMeta::new(
                ComponentName::new("repository"),
                Version::new(*version),
                vec![],
            )
            .unwrap();
            Moved {
                version: Version::new(*version),
                index: world
                    .fakes
                    .store
                    .put_artifact(&Manifest::Index(meta))
                    .unwrap()
                    .digest,
            }
        })
        .collect();
    world.promote_to("beta", &moves);
    let runs = &moves[0].index;
    let report = Reported {
        environment: EnvironmentName::new("srv"),
        component: ComponentName::new("repository"),
        channel: ChannelName::new("beta"),
        version: Version::new("0.1.0"),
        index: runs.clone(),
        runs: runs.clone(),
        converged_at: "2026-11-05T09:00:00Z".into(),
        at: now(),
    };
    let signer = FakeSigner::new();
    signer.hold(&deployer);
    let envelope = signer
        .sign(&report.claim(world.head.clone()), &deployer)
        .unwrap();
    world.fakes.store.attach(runs, &envelope).unwrap();
    second
}

#[test]
fn an_environment_behind_shows_the_notes_between_escaped_and_its_json_is_status() {
    let world = World::new();
    let repository = world.named();
    let second = srv_behind(&world);

    let (status, served) = world.json(&format!("{}/status.json", repository.path()));
    assert_eq!(status, 200, "{served}");
    assert_eq!(unasked(served.clone()), unasked(world.status()));
    let srv = &served["value"]["environments"]["value"]["environments"][0];
    assert_eq!(srv["state"]["value"], "behind", "{served}");
    let between = &srv["between"]["value"]["versions"];
    assert_eq!(between[0]["version"], "0.2.0", "{served}");
    assert_eq!(between[0]["revision"], second.as_str(), "{served}");
    let notes = between[0]["notes"]["value"]
        .as_str()
        .expect("0.2.0's notes");
    assert!(notes.contains("<script>alert(1)</script>"), "{notes}");

    let (status, page) = world.get(&format!("{}/status", repository.path()));
    assert_eq!(status, 200, "{page}");
    assert!(
        page.contains("0 running what they should, 1 behind, 0 waiting on a person, 0 unreadable"),
        "{page}"
    );
    let runs = srv["runs"]["value"]["index"].as_str().unwrap();
    let should = srv["should"]["value"]["index"].as_str().unwrap();
    assert!(
        page.contains(&format!(
            "<tr><td>srv</td><td>repository/beta</td><td>behind</td>\
             <td>0.1.0 <code>{runs}</code> since 2026-11-05T09:00:00Z</td>\
             <td>0.2.0 <code>{should}</code></td><td></td></tr>"
        )),
        "{page}"
    );
    assert!(
        page.contains(&format!(
            "ahead, the versions between:<ol class=\"chain\"><li>0.2.0 at <code>{second}</code>\
             <pre>{}</pre></li></ol>",
            escape(notes)
        )),
        "{page}"
    );
    assert!(
        page.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
        "{page}"
    );
    assert!(!page.contains("<script"), "{page}");

    // The root's row carries the four counts as counted.
    let (status, root) = world.json("/status.json");
    assert_eq!(status, 200, "{root}");
    assert_eq!(
        root["value"]["repositories"][0]["value"]["environments"],
        serde_json::json!({
            "answer": "ok",
            "value": { "converged": 0, "behind": 1, "waiting": 0, "unreadable": 0 }
        }),
        "{root}"
    );
    let (status, page) = world.get("/");
    assert_eq!(status, 200, "{page}");
    assert!(
        page.contains(
            "</td><td>0 running what they should, 1 behind, 0 waiting on a person, \
             0 unreadable</td></tr>"
        ),
        "{page}"
    );
}

// Requirement: The root answers every repository it knows together

#[test]
fn two_repositories_named_are_answered_together_at_the_root() {
    let first = World::new();
    let second = World::beside(&first.host);
    let two = second.named_as("b", "two");
    let one = first.named_as("a", "one");
    first.promote("stable", &["0.1.0", "0.2.0"]);
    second.promote("beta", &["0.3.0"]);
    let both = [&first, &second];
    let before = now();
    let response = among(&both, "GET", "/status.json");
    let after = now();
    let served: Value = serde_json::from_slice(&response.body).unwrap();
    assert_eq!(response.status, 200, "{served}");
    assert_eq!(served["answer"], "ok", "{served}");
    let asked = served["value"]["asked_at"].as_u64().unwrap();
    assert!((before..=after).contains(&asked), "{asked}");
    let rows = served["value"]["repositories"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "{served}");
    assert_eq!(rows[0], first.row(&one));
    assert_eq!(rows[1], second.row(&two));
    assert_eq!(rows[0]["value"]["repository"]["owner"], "a");

    let response = among(&both, "GET", "/");
    let page = text(&response);
    assert_eq!(response.status, 200, "{page}");
    for linked in ["href=\"/a/one/status\"", "href=\"/b/two/status\""] {
        assert!(page.contains(linked), "{linked}: {page}");
    }
    for head in [
        "repository/beta empty",
        "repository/stable 0.2.0",
        "repository/beta 0.3.0",
        "repository/stable empty",
    ] {
        assert!(page.contains(head), "{head}: {page}");
    }
    assert_eq!(page.matches(NONE).count(), 2, "{page}");
}

#[test]
fn a_row_whose_checkout_is_gone_is_unreadable_naming_the_path_beside_the_other() {
    let first = World::new();
    let second = World::beside(&first.host);
    first.named_as("a", "one");
    second.named_as("b", "two");
    let gone = second.repo.clone();
    fs::remove_dir_all(&gone).unwrap();
    let (status, served) = first.json("/status.json");
    assert_eq!(status, 200, "{served}");
    let rows = &served["value"]["repositories"];
    assert_eq!(rows[0]["answer"], "ok", "{served}");
    assert_eq!(rows[1]["answer"], "unreadable", "{served}");
    let what = rows[1]["unreadable"]["what"].as_str().unwrap();
    assert!(what.contains(&gone.display().to_string()), "{what}");
    let (_, page) = first.get("/");
    assert!(page.contains("href=\"/a/one/status\""), "{page}");
    assert!(page.contains(&gone.display().to_string()), "{page}");
}

#[test]
fn a_host_that_named_nothing_answers_an_empty_list_and_says_how_to_name_one() {
    let world = World::new();
    let (status, served) = world.json("/status.json");
    assert_eq!(status, 200, "{served}");
    assert_eq!(served["value"]["repositories"], serde_json::json!([]));
    let (status, page) = world.get("/");
    assert_eq!(status, 200);
    assert!(page.contains("timber status --page"), "{page}");
}

#[test]
fn a_file_of_repositories_that_does_not_parse_is_the_roots_unreadable_answer() {
    let world = World::new();
    world.named();
    fs::write(
        world.host.state.join(REPOSITORIES),
        "[\"o/r\"\ncheckout = 3\n",
    )
    .unwrap();
    let (status, served) = world.json("/status.json");
    assert_eq!(status, 500, "{served}");
    assert_eq!(served["answer"], "unreadable", "{served}");
    let said = served["unreadable"]["said"].as_str().unwrap();
    assert!(said.contains("TOML parse error"), "{said}");
    let (status, page) = world.get("/");
    assert_eq!(status, 500);
    assert!(page.contains("TOML parse error"), "{page}");
}

// Requirement: The process serves each served layout under its repository's path

const ECO: &str = "eco";

fn eco() -> Ecosystem {
    Ecosystem::new(ECO)
}

fn pinned(version: &str) -> Pinned {
    Pinned {
        name: EcosystemName::new("pkg"),
        version: Version::new(version),
    }
}

/// Acceptance by the gate's signer, `gate`.
fn accepted_by_the_gate() -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Acceptance,
        check: None,
        environment: None,
        signers: vec![NamedSigner {
            id: SignerId::new("gate"),
            verified_by: "gate's key".into(),
            kinds: vec![ClaimKind::Acceptance],
            checks: vec![],
        }],
        asked_of: AskedOf::Every,
    }
}

/// `version` of `pkg` published to `store` with its index named where a
/// pin resolves, accepted by the gate where `accepted`.
fn publish(store: &FakeStore, version: &str, accepted: bool) {
    let meta = ArtifactMeta::new(eco(), EcosystemName::new("pkg"), Version::new(version)).unwrap();
    let layer = store
        .put_bytes(format!("pkg {version}").as_bytes())
        .unwrap();
    let artifact = Artifact {
        digest: store
            .put_artifact(&Manifest::Artifact {
                meta: meta.clone(),
                layers: vec![layer.digest],
            })
            .unwrap()
            .digest,
        meta,
    };
    let index = IndexMeta::new(
        ComponentName::new("pkg"),
        Version::new(version),
        vec![artifact],
    )
    .unwrap();
    let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
    let name = artifact_name(&eco(), &EcosystemName::new("pkg"), &Version::new(version));
    store.name(&name, &index, None).unwrap();
    if accepted {
        let gate = SignerId::new("gate");
        let signer = FakeSigner::new();
        signer.hold(&gate);
        let claim = Claim {
            subject: index.clone(),
            kind: ClaimKind::Acceptance,
            revision: Revision::new("p1b11s4"),
            build: None,
            body: Value::Null,
        };
        store
            .attach(&index, &signer.sign(&claim, &gate).unwrap())
            .unwrap();
    }
}

/// A checkout of `agentplot/consumer` named to the host, over the
/// world's fakes binding the ecosystem's fake projection, whose policy
/// for layouts requires acceptance at its landed head; `pkg 1.2.0` and
/// `1.1.0` published and accepted and `1.0.0` published unaccepted; the
/// layout synced from a lockfile pinning `1.2.0`.
struct Consumer {
    world: World,
    projection: FakeProjection,
    tree: PathBuf,
}

impl Consumer {
    fn new() -> Consumer {
        let mut world = World::new();
        world.named_as("agentplot", "consumer");
        let projection = world.fakes.projection(&eco());
        let main = Branch::new(MAIN);
        world.fakes.policy.require_at(
            &world.head,
            Audience::Layout,
            Required::Layout {
                claims: vec![accepted_by_the_gate()],
                given: false,
                signers: vec![],
            },
        );
        publish(&world.fakes.store, "1.2.0", true);
        publish(&world.fakes.store, "1.1.0", true);
        publish(&world.fakes.store, "1.0.0", false);
        let history = world.history();
        let tree = history.workdir().unwrap();
        projection.lockfile(&tree.display().to_string(), &[pinned("1.2.0")]);
        let synced = sync_checkout(
            &world.fakes.seams(),
            &Layouts::at(&history),
            &CheckoutAsk {
                branch: main,
                revision: world.head.clone(),
                tree: tree.clone(),
                when: now(),
                trusted: vec![],
                repository: Ok(RepositoryName {
                    owner: "agentplot".into(),
                    name: "consumer".into(),
                }),
            },
        )
        .unwrap();
        assert_eq!(
            synced.served.get(&eco()),
            Some(&Ok(
                "http://127.0.0.1:7420/agentplot/consumer/eco/".to_string()
            ))
        );
        Consumer {
            world,
            projection,
            tree,
        }
    }

    /// `GET` of `path` asked of the process's own handling of a request,
    /// the checkout bound to the fakes, for sync alone.
    fn get(&self, path: &str) -> Response {
        self.ask(path, "GET")
    }

    /// `method` of `path`, asked as [`Consumer::get`] asks, of the process
    /// at [`asked_at`].
    fn ask(&self, path: &str, method: &str) -> Response {
        respond_with(
            asked_at(),
            &self.world.host,
            &|checkout, command| {
                assert_eq!(command, For::Sync, "a served layout bound for another");
                assert_eq!(checkout, self.world.repo, "another checkout bound");
                Ok(self.world.binding())
            },
            method,
            path,
        )
    }

    /// The versions the layout's record holds.
    fn held(&self) -> Vec<String> {
        Layouts::at(&self.world.history())
            .read(&eco())
            .unwrap()
            .held
            .iter()
            .map(|h| h.version.to_string())
            .collect()
    }
}

#[test]
fn a_served_layout_is_its_projections_response_from_the_checkout_that_named_it() {
    let consumer = Consumer::new();
    let response = consumer.get("/agentplot/consumer/eco/pkg/1.2.0");
    assert_eq!(response.status, 200, "{}", text(&response));
    assert_eq!(response.content_type, "application/json");
    let served: Value = serde_json::from_slice(&response.body).unwrap();
    assert_eq!(served["name"], "pkg", "{served}");
    assert_eq!(served["version"], "1.2.0", "{served}");
    assert_eq!(
        consumer.projection.requests(),
        [
            Request {
                method: "GET".into(),
                path: String::new(),
                base: "http://127.0.0.1:7420/agentplot/consumer/eco".into(),
            },
            Request {
                method: "GET".into(),
                path: "pkg/1.2.0".into(),
                base: "http://127.0.0.1:7499/agentplot/consumer/eco".into(),
            }
        ],
        "sync's probe at the core's address, then the request below the layout's prefix \
         at the address the process was asked at"
    );
    assert_eq!(consumer.projection.misses(), []);
    let rendered: Vec<String> = consumer
        .projection
        .renders()
        .into_iter()
        .map(|(at, _)| at)
        .collect();
    assert_eq!(
        rendered,
        [place(&consumer.tree, &eco()).display().to_string()],
        "served from a layout the checkout's sync did not render"
    );
}

#[test]
fn a_version_the_layout_lacks_is_rendered_into_it_by_the_core_on_the_request() {
    let consumer = Consumer::new();
    let response = consumer.get("/agentplot/consumer/eco/pkg/1.1.0");
    assert_eq!(response.status, 200, "{}", text(&response));
    assert_eq!(consumer.projection.misses(), [pinned("1.1.0")]);
    assert_eq!(consumer.held(), ["1.2.0", "1.1.0"]);
    let renders: Vec<Vec<String>> = consumer
        .projection
        .renders()
        .into_iter()
        .map(|(at, artifacts)| {
            assert_eq!(at, place(&consumer.tree, &eco()).display().to_string());
            artifacts
                .iter()
                .map(|a| a.meta.version().to_string())
                .collect()
        })
        .collect();
    assert_eq!(renders, [vec!["1.2.0"], vec!["1.2.0", "1.1.0"]]);
}

#[test]
fn a_version_the_policy_refuses_is_the_projections_refusal_and_the_layout_is_unchanged() {
    let consumer = Consumer::new();
    let response = consumer.get("/agentplot/consumer/eco/pkg/1.0.0");
    assert_eq!(response.status, 403, "{}", text(&response));
    let words = text(&response);
    assert!(words.contains("pkg 1.0.0"), "{words}");
    assert!(words.contains("acceptance"), "{words}");
    assert_eq!(consumer.held(), ["1.2.0"]);
    assert_eq!(consumer.projection.renders().len(), 1, "rendered again");
}

#[test]
fn a_served_layouts_method_is_its_projections_to_answer() {
    let consumer = Consumer::new();
    let response = consumer.ask("/agentplot/consumer/eco/pkg/1.2.0", "HEAD");
    assert_eq!(response.status, 200, "{}", text(&response));
    let asked = consumer.projection.requests();
    assert_eq!(asked.last().map(|r| r.method.as_str()), Some("HEAD"));
    let mut written = Vec::new();
    response.write_head(&mut written).unwrap();
    let head = String::from_utf8(written).unwrap();
    assert!(
        head.ends_with("\r\n\r\n")
            && head.contains(&format!("\r\nContent-Length: {}\r\n", response.body.len())),
        "{head}"
    );
    let posted = consumer.ask("/agentplot/consumer/eco/pkg/1.2.0", "POST");
    assert_eq!(
        consumer
            .projection
            .requests()
            .last()
            .map(|r| r.method.clone()),
        Some("POST".to_string()),
        "the process answered {} itself",
        posted.status
    );
}

#[test]
fn a_response_is_written_with_its_bytes_exactly_and_their_length() {
    let response = Response {
        status: 200,
        content_type: "application/octet-stream".into(),
        body: (0..=255).collect(),
    };
    let mut written = Vec::new();
    response.write(&mut written).unwrap();
    let blank = written
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("a head");
    let head = String::from_utf8(written[..blank].to_vec()).unwrap();
    assert!(head.starts_with("HTTP/1.1 200 OK\r\n"), "{head}");
    assert!(head.contains("\r\nContent-Length: 256\r\n"), "{head}");
    assert!(
        head.contains("\r\nContent-Type: application/octet-stream\r\n"),
        "{head}"
    );
    assert_eq!(&written[blank + 4..], response.body.as_slice());
    for (status, reason) in [(403, "Forbidden"), (502, "Bad Gateway")] {
        let mut written = Vec::new();
        Response {
            status,
            ..response.clone()
        }
        .write(&mut written)
        .unwrap();
        let line = format!("HTTP/1.1 {status} {reason}\r\n");
        assert!(written.starts_with(line.as_bytes()), "{status}");
    }
}

#[test]
fn an_ecosystem_nothing_serves_is_404_naming_it_and_nothing_is_rendered() {
    let mut consumer = Consumer::new();
    let file = consumer.world.fakes.projection(&Ecosystem::new("file"));
    file.serves_none();
    let response = consumer.get("/agentplot/consumer/file/notes.txt");
    assert_eq!(response.status, 404, "{}", text(&response));
    assert!(text(&response).contains("file"), "{}", text(&response));
    assert!(file.renders().is_empty());
    let response = consumer.get("/agentplot/consumer/other/x");
    assert_eq!(response.status, 404);
    assert!(text(&response).contains("other"), "{}", text(&response));
    assert_eq!(consumer.projection.misses(), []);
}

#[test]
fn a_repository_none_named_and_a_checkout_that_cannot_be_bound_are_their_own_answers() {
    let consumer = Consumer::new();
    let response = consumer.get("/agentplot/timber/eco/pkg/1.2.0");
    assert_eq!(response.status, 404);
    assert!(
        text(&response).contains("has named it"),
        "{}",
        text(&response)
    );
    let unbound = Not::unreadable(
        &Source::new(Seam::Host, "bindings"),
        "the bindings file",
        Some("expected `=`".into()),
    );
    let response = respond_with(
        asked_at(),
        &consumer.world.host,
        &|_, _| Err(unbound.clone()),
        "GET",
        "/agentplot/consumer/eco/pkg/1.2.0",
    );
    assert_eq!(response.status, 500);
    assert_eq!(text(&response), format!("{unbound}\n"));
    assert!(consumer.projection.requests().len() == 1, "served unbound");
}
