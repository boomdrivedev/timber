//! Spec serving-process: the resident's life, each test over a `Host` of
//! temporary directories and `serve_on` or `serve_with` on a port of the
//! test's own, never the core's: the process naming itself, the page
//! named and the process started where none answers, over a scratch
//! remote bound to a plain branch landing and the git ref ledger; the
//! lock that keeps it one, the built binary's `timber serve` ending where
//! it finds the lock held, two askers, and its idle endings, which wait
//! while a layout is served. Each route's answer is a unit of the lib,
//! `src/commands/serve/serving_process.rs`. Needs git alone, so it is in
//! the fast layer.

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::scratch::{Scratch, POLICY_FILE};
use git2::{IndexAddOption, Repository, RepositoryInitOptions, Signature};
use serde_json::Value;
use tempfile::TempDir;
use timber::bindings::Teller;
use timber::commands::serve::host::{checkout_of, find_or_start, name, Host, SERVING};
use timber::commands::serve::{serve_on, serve_with, Binder, Served};
use timber::commands::status::page;
use timber::{Binding, REVISION};
use timber_core::{
    artifact_name, place, sync_checkout, CheckoutAsk, History, Layouts, RepositoryName,
};
use timber_fakes::{FakeProjection, FakeSharedHistory, FakeSigner, FakeStore, Fakes};
use timber_seams::{
    Answer, Artifact, ArtifactMeta, AskedOf, Audience, Branch, BuildId, Claim, ClaimKind,
    ComponentName, Contents, Ecosystem, EcosystemName, IndexMeta, Layout, Manifest, Miss,
    NamedSigner, Output, PackAt, Packed, Pin, Pinned, Projection, Range, Rendered, Request,
    Required, RequiredClaim, Requirement, Revision, Seam, Signer, SignerId, Source, Store, Version,
};

const MAIN: &str = "main";

/// The host the tests serve: a state directory, a bindings file, a home
/// and a data directory, all in `dir`.
fn host_in(dir: &Path) -> Host {
    Host {
        state: dir.join("state/timber"),
        config: dir.join("config/timber/config.toml"),
        home: Some(dir.join("home")),
        data: Some(dir.join("data")),
    }
}

/// A host of a temporary directory of its own, holding nothing.
struct Bare {
    dir: TempDir,
    host: Host,
}

impl Bare {
    fn new() -> Bare {
        let dir = TempDir::new().unwrap();
        let host = host_in(dir.path());
        Bare { dir, host }
    }

    /// The built binary's `timber serve` over this host, its home and
    /// its configuration, data and state directories this host's, once
    /// it ends. It is waited for up to a limit that no load of the host
    /// reaches, and stopped and failed at that limit alone.
    fn serve(&self) -> std::process::Output {
        let dir = self.dir.path();
        let mut serve = Command::new(env!("CARGO_BIN_EXE_timber"))
            .arg("serve")
            .env("HOME", dir.join("home"))
            .env("XDG_CONFIG_HOME", dir.join("config"))
            .env("XDG_DATA_HOME", dir.join("data"))
            .env("XDG_STATE_HOME", dir.join("state"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the binary runs");
        let limit = Instant::now() + Duration::from_secs(60);
        while serve.try_wait().unwrap().is_none() {
            if Instant::now() > limit {
                serve.kill().unwrap();
                let _ = serve.wait();
                panic!("timber serve did not end");
            }
            thread::sleep(Duration::from_millis(50));
        }
        serve.wait_with_output().unwrap()
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// A port for one test's process, never the core's: below the range the
/// system hands to connections, so no connection is given it while the
/// process starts, and one of its own per test. Nothing binds it to see
/// it free: a git a test runs at that moment could inherit the socket
/// and hold the port after it is let go.
fn free() -> SocketAddr {
    use std::sync::atomic::{AtomicU16, Ordering};
    static NEXT: AtomicU16 = AtomicU16::new(0);
    let base = 20_000 + (std::process::id() % 400) as u16 * 20;
    let port = base + NEXT.fetch_add(1, Ordering::SeqCst);
    SocketAddr::from(([127, 0, 0, 1], port))
}

/// `GET` of `path` asked at `address`: the status and the body.
fn get(address: SocketAddr, path: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(address).unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut answered = String::new();
    stream.read_to_string(&mut answered).unwrap();
    let (head, body) = answered.split_once("\r\n\r\n").expect("a head and a body");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .expect("a status");
    (status, body.to_string())
}

fn json(address: SocketAddr, path: &str) -> (u16, Value) {
    let (status, body) = get(address, path);
    (
        status,
        serde_json::from_str(&body).expect("a JSON document"),
    )
}

/// `serve_on` over `host` at a port of the test's own, in a thread, once
/// it answers: its address and the thread.
fn serving(host: &Host, idle: Duration) -> (SocketAddr, JoinHandle<Answer<Served>>) {
    let address = free();
    let host = host.clone();
    let server = thread::spawn(move || serve_on(address, &host, idle));
    find_or_start(address, || Ok(())).expect("the process answers");
    (address, server)
}

// Requirement: One process answers a host at one fixed address

#[test]
fn the_process_names_itself_its_revision_and_its_process_id() {
    let bare = Bare::new();
    let (address, _) = serving(&bare.host, Duration::from_secs(60));
    let (status, named) = json(address, "/serving.json");
    assert_eq!(status, 200);
    assert_eq!(named["serving"], "timber");
    assert_eq!(named["revision"], REVISION);
    assert_eq!(named["pid"], std::process::id());
}

// Requirement: The first asker that finds none starts it, and one lock
// keeps it one

/// The policy the scratch remote lands: `beta`, and `stable` bound to it.
const POLICY: &str = "outputs = [\"site\"]\n\n[projections]\nsite = \"file\"\n\n\
                      [channels.beta]\nrequire = []\n\n[channels.stable]\nbound_to = \"beta\"\n\
                      require = []\n";

#[test]
fn the_page_is_named_and_the_process_started_where_none_answers() {
    let scratch = Scratch::new();
    let repo = &scratch.repo;
    repo.write(POLICY_FILE, POLICY);
    repo.commit("feat: the channels");
    repo.push();
    let host = host_in(&scratch.path());
    fs::create_dir_all(host.config.parent().unwrap()).unwrap();
    fs::write(
        &host.config,
        "[landing]\nbranch = { remote = \"origin\" }\n\n[ledger]\ngit_ref = { remote = \"origin\" }\n",
    )
    .unwrap();
    // The remote's address names the repository; git reads it from the
    // scratch remote.
    let address_of = "https://example.com/o/r.git";
    repo.git(&["remote", "set-url", "origin", address_of]);
    repo.git(&[
        "config",
        &format!("url.{}.insteadOf", scratch.remote.display()),
        address_of,
    ]);
    let address = free();
    let started = host.clone();
    let answered = page(&repo.path, &host, address, move || {
        thread::spawn(move || serve_on(address, &started, Duration::from_secs(60)));
        Ok(())
    });
    assert_eq!(answered, Ok(format!("http://{address}/o/r/status")));
    let named = RepositoryName {
        owner: "o".into(),
        name: "r".into(),
    };
    assert_eq!(checkout_of(&host, &named), Ok(repo.path.clone()));
    let (status, served) = json(address, "/o/r/status.json");
    assert_eq!(status, 200, "{served}");
    assert_eq!(
        served["value"]["channels"]["answer"], "ok",
        "the landing and ledger answer through the rewritten remote: {served}"
    );

    // Asked again, the process answers and nothing is started.
    let again = page(&repo.path, &host, address, || panic!("started a second"));
    assert_eq!(again, Ok(format!("http://{address}/o/r/status")));
}

#[test]
fn a_second_serve_with_the_lock_held_returns_at_once_saying_the_first_answers() {
    let bare = Bare::new();
    let (address, _) = serving(&bare.host, Duration::from_secs(60));
    let second = bare.serve();
    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert_eq!(
        String::from_utf8_lossy(&second.stdout),
        format!(
            "the host's timber serve answers at http://{}\n",
            timber::commands::serve::address()
        )
    );
    let (status, named) = json(address, "/serving.json");
    assert_eq!(status, 200);
    assert_eq!(named["pid"], std::process::id(), "the first answers");
}

#[test]
fn two_askers_that_find_none_each_start_one_and_one_serves_both() {
    let bare = Bare::new();
    let address = free();
    // Each start waits for the other, so both askers find none.
    let starts = Arc::new((Mutex::new(0usize), Condvar::new()));
    let ended = Arc::new(Mutex::new(Vec::new()));
    let askers: Vec<_> = (0..2)
        .map(|_| {
            let (host, starts, ended) = (bare.host.clone(), starts.clone(), ended.clone());
            thread::spawn(move || {
                find_or_start(address, move || {
                    let (count, both) = &*starts;
                    let mut count = count.lock().unwrap();
                    *count += 1;
                    both.notify_all();
                    let _ = both
                        .wait_timeout_while(count, Duration::from_secs(5), |c| *c < 2)
                        .unwrap();
                    thread::spawn(move || {
                        let served = serve_on(address, &host, Duration::from_secs(1));
                        ended.lock().unwrap().push(served);
                    });
                    Ok(())
                })
            })
        })
        .collect();
    let found: Vec<_> = askers
        .into_iter()
        .map(|asker| asker.join().unwrap().expect("each asker found one"))
        .collect();
    assert_eq!(*starts.0.lock().unwrap(), 2, "both askers started one");
    assert_eq!(found[0].pid, found[1].pid);
    let deadline = Instant::now() + Duration::from_secs(10);
    while ended.lock().unwrap().len() < 2 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(50));
    }
    let mut ended = ended.lock().unwrap().clone();
    ended.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(ended, [Ok(Served::Held), Ok(Served::Idle)]);
}

#[test]
fn an_idle_process_ends_and_lets_the_lock_go() {
    let bare = Bare::new();
    let (address, server) = serving(&bare.host, Duration::from_secs(1));
    let asked = Instant::now();
    assert_eq!(get(address, "/serving.json").0, 200);
    assert_eq!(server.join().unwrap(), Ok(Served::Idle));
    assert!(asked.elapsed() >= Duration::from_secs(1));
    let lock = fs::File::open(bare.host.state.join(SERVING)).unwrap();
    // A git another test spawns at that moment may hold a copy of the
    // lock's descriptor until it runs, a few milliseconds.
    let ended = Instant::now();
    while lock.try_lock().is_err() {
        assert!(
            ended.elapsed() < Duration::from_secs(2),
            "the lock was not let go"
        );
        thread::sleep(Duration::from_millis(10));
    }
    drop(lock);

    // The next asker starts one again.
    let again = free();
    let host = bare.host.clone();
    let found = find_or_start(again, move || {
        thread::spawn(move || serve_on(again, &host, Duration::from_secs(1)));
        Ok(())
    });
    assert_eq!(found.map(|s| s.pid), Ok(std::process::id()));
}

// Requirement: The process serves each served layout under its
// repository's path, and stays while one is served

const ECO: &str = "eco";

fn eco() -> Ecosystem {
    Ecosystem::new(ECO)
}

/// `pkg 1.2.0` published to `store` with its index named where a pin
/// resolves, accepted by the gate.
fn publish(store: &FakeStore) {
    let version = Version::new("1.2.0");
    let meta = ArtifactMeta::new(eco(), EcosystemName::new("pkg"), version.clone()).unwrap();
    let layer = store.put_bytes(b"pkg 1.2.0").unwrap();
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
    let index = IndexMeta::new(ComponentName::new("pkg"), version.clone(), vec![artifact]).unwrap();
    let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
    let name = artifact_name(&eco(), &EcosystemName::new("pkg"), &version);
    store.name(&name, &index, None).unwrap();
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

/// A checkout of `agentplot/consumer` made in the test's own process and
/// named to a host of its own, its layout of the ecosystem synced over
/// fakes from a lockfile pinning `pkg 1.2.0`, accepted as its policy for
/// layouts requires.
struct Consumer {
    _dir: TempDir,
    host: Host,
    repo: PathBuf,
    tree: PathBuf,
}

impl Consumer {
    fn new() -> Consumer {
        let dir = TempDir::new().unwrap();
        let host = host_in(dir.path());
        let repo = dir.path().join("consumer");
        let git =
            Repository::init_opts(&repo, RepositoryInitOptions::new().initial_head(MAIN)).unwrap();
        fs::write(repo.join("README"), "the consumer").unwrap();
        let mut index = git.index().unwrap();
        index
            .add_all(["README"], IndexAddOption::DEFAULT, None)
            .unwrap();
        let tree = git.find_tree(index.write_tree().unwrap()).unwrap();
        let author = Signature::now("A. Uthor", "author@example.com").unwrap();
        let head = git
            .commit(
                Some("HEAD"),
                &author,
                &author,
                "feat: the consumer\n",
                &tree,
                &[],
            )
            .unwrap();
        let head = Revision::new(head.to_string());
        let repository = RepositoryName {
            owner: "agentplot".into(),
            name: "consumer".into(),
        };
        name(&host, &repository, &repo).unwrap();

        let mut fakes = Fakes::new();
        let projection = fakes.projection(&eco());
        let main = Branch::new(MAIN);
        fakes.landing.land(&main, &head);
        fakes.policy.require_at(
            &head,
            Audience::Layout,
            Required::Layout {
                claims: vec![RequiredClaim {
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
                }],
                given: false,
                signers: vec![],
            },
        );
        publish(&fakes.store);
        let history = History::reaching(&repo, Arc::new(FakeSharedHistory)).unwrap();
        let tree = history.workdir().unwrap();
        projection.lockfile(
            &tree.display().to_string(),
            &[Pinned {
                name: EcosystemName::new("pkg"),
                version: Version::new("1.2.0"),
            }],
        );
        let synced = sync_checkout(
            &fakes.seams(),
            &Layouts::at(&history),
            &CheckoutAsk {
                branch: main,
                revision: head,
                tree: tree.clone(),
                when: now(),
                trusted: vec![],
                repository: Ok(repository),
            },
        )
        .unwrap();
        assert!(
            matches!(synced.served.get(&eco()), Some(Ok(_))),
            "{synced:?}"
        );
        Consumer {
            _dir: dir,
            host,
            repo,
            tree,
        }
    }

    /// The layout rendered: its place made, as a projection that writes
    /// files makes it, beside the record the sync wrote.
    fn rendered(&self) -> PathBuf {
        let at = place(&self.tree, &eco());
        fs::create_dir_all(&at).unwrap();
        at
    }
}

/// `seams` as the one binding a checkout is bound to.
fn binding(seams: timber_seams::Seams) -> Binding {
    Binding {
        seams,
        store: Source::new(Seam::Store, "fake"),
        signer: None,
        gate: Teller::Job("the gate's job".into()),
        trust: vec![],
        shared: Arc::new(FakeSharedHistory),
    }
}

/// A binding over fakes made anew on each call, on any thread: the
/// ecosystem's fake projection, serving or scripted to serve none.
fn fresh(serving: bool) -> Binder {
    Arc::new(move |_, _| {
        let mut fakes = Fakes::new();
        let projection = fakes.projection(&eco());
        if !serving {
            projection.serves_none();
        }
        Ok(binding(fakes.seams()))
    })
}

/// An idle of moments, which the tests wait out several times over.
const MOMENTS: Duration = Duration::from_millis(200);

/// `serve_with` over `host` at a port of the test's own, each checkout
/// bound by `bind`, in a thread, once it answers.
fn serving_with(host: &Host, bind: Binder) -> (SocketAddr, JoinHandle<Answer<Served>>) {
    let address = free();
    let host = host.clone();
    let server = thread::spawn(move || serve_with(address, &host, MOMENTS, bind));
    find_or_start(address, || Ok(())).expect("the process answers");
    (address, server)
}

/// The process's answer once it ends within a few seconds.
fn ends(server: JoinHandle<Answer<Served>>) -> Answer<Served> {
    let asked = Instant::now();
    while !server.is_finished() {
        assert!(
            asked.elapsed() < Duration::from_secs(5),
            "the process did not end"
        );
        thread::sleep(Duration::from_millis(10));
    }
    server.join().unwrap()
}

/// A projection answering every request it is asked with the base it was
/// handed, the fake answering all else.
struct Based(FakeProjection);

impl Projection for Based {
    fn pack(&self, output: &Output, at: &BuildId, version: Option<&PackAt<'_>>) -> Answer<Packed> {
        self.0.pack(output, at, version)
    }

    fn render(&self, contents: Contents<'_>, at: &str) -> Answer<Rendered> {
        self.0.render(contents, at)
    }

    fn serve(&self, _: &Layout, request: &Request, _: Miss<'_>) -> Answer<timber_seams::Response> {
        Ok(timber_seams::Response {
            status: 200,
            content_type: "text/plain; charset=utf-8".into(),
            body: request.base.clone().into_bytes(),
        })
    }

    fn pinned(&self, checkout: &str) -> Answer<Vec<Pin>> {
        self.0.pinned(checkout)
    }

    fn requires(&self, held: &Artifact) -> Answer<Vec<Requirement>> {
        self.0.requires(held)
    }

    fn satisfying(&self, range: &Range, versions: &[Version]) -> Answer<Vec<Version>> {
        self.0.satisfying(range, versions)
    }
}

#[test]
fn a_served_layout_is_handed_the_base_of_the_address_the_process_is_bound_at() {
    let consumer = Consumer::new();
    let bind: Binder = Arc::new(|_, _| {
        let mut seams = Fakes::new().seams();
        seams
            .projections
            .insert(eco(), Box::new(Based(FakeProjection::new(eco()))));
        Ok(binding(seams))
    });
    let (address, server) = serving_with(&consumer.host, bind);
    let (status, body) = get(address, "/agentplot/consumer/eco/pkg/1.2.0");
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        body,
        format!("http://{address}/agentplot/consumer/eco"),
        "the base names another address than the one the process is bound at"
    );
    assert_eq!(ends(server), Ok(Served::Idle));
}

#[test]
fn a_process_stays_past_its_idle_time_while_a_layout_is_served() {
    let consumer = Consumer::new();
    consumer.rendered();
    let (address, server) = serving_with(&consumer.host, fresh(true));
    let (_, first) = json(address, "/serving.json");
    thread::sleep(MOMENTS * 5);
    assert!(
        !server.is_finished(),
        "the process ended while a layout is served"
    );
    let (status, body) = get(address, "/agentplot/consumer/eco/pkg/1.2.0");
    assert_eq!(status, 200, "{body}");
    let (_, again) = json(address, "/serving.json");
    assert_eq!(again["pid"], first["pid"]);
    assert!(!server.is_finished());
}

#[test]
fn a_process_ends_at_its_next_idle_once_the_served_layout_is_removed() {
    let consumer = Consumer::new();
    let at = consumer.rendered();
    let (_, server) = serving_with(&consumer.host, fresh(true));
    thread::sleep(MOMENTS * 3);
    assert!(
        !server.is_finished(),
        "the process ended while a layout is served"
    );
    fs::remove_dir_all(&at).unwrap();
    assert_eq!(ends(server), Ok(Served::Idle));
    let lock = fs::File::open(consumer.host.state.join(SERVING)).unwrap();
    lock.try_lock().expect("the lock was let go");
}

#[test]
fn a_process_ends_at_its_next_idle_once_the_checkout_holding_it_is_removed() {
    let consumer = Consumer::new();
    consumer.rendered();
    let (_, server) = serving_with(&consumer.host, fresh(true));
    thread::sleep(MOMENTS * 3);
    assert!(
        !server.is_finished(),
        "the process ended while a layout is served"
    );
    fs::remove_dir_all(&consumer.repo).unwrap();
    assert_eq!(ends(server), Ok(Served::Idle));
}

#[test]
fn a_process_ends_at_its_idle_where_the_only_layout_is_of_a_projection_serving_none() {
    let consumer = Consumer::new();
    consumer.rendered();
    let (_, server) = serving_with(&consumer.host, fresh(false));
    assert_eq!(ends(server), Ok(Served::Idle));
}
