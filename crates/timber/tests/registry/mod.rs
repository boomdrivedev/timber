//! The registries the registry store's tests run against (design D12): a
//! zot and a reference distribution, each started on a free loopback
//! port over a temporary directory and a config the test writes, by the
//! binary the devenv shell pins, waited on until `GET /v2/` answers, and
//! stopped when dropped; stores bound for a repository under a root of
//! one, `acme/site` where a test names no other (C.1); the store group's
//! rig over a fresh root of one; a proxy in front of one that keeps every request it passes and
//! alters a response body when told to; the cache manifest's scenarios a
//! registry plays through the store alone; and the one live rig the runs
//! by hand against a hosted registry, ECR's and GitHub's, share: a run's
//! prefix, a store whose every name carries it, bound for the repository
//! `timber-checks/store` under the root the host names, and the store
//! group run there (design D4; the change
//! each-repositorys-spaces-sit-under-its-own-root's design D7); and a
//! moved repository's history carried between two roots of one registry
//! (C.1; the change a-moved-repository-keeps-its-history-in-the-store's
//! design D8).

#![allow(dead_code)]

#[path = "../ports/mod.rs"]
mod ports;

use std::cell::Cell;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tempfile::TempDir;
use timber::profiles::store::registry::Registry;
use timber_conformance::seams::store::{self, settled, StoreRig};
use timber_conformance::{run, Rig};
use timber_seams::{
    Answer, Digest, Envelope, Held, Manifest, Name, Not, Put, RepositoryName, Store, Why,
};

/// The one user the registries that ask for a credential take, and the
/// bcrypt of its password, as `htpasswd -nbB` writes it.
pub const USER: &str = "tester";
pub const PASSWORD: &str = "secret";
const HTPASSWD: &str = "tester:$2y$05$Os.CNqLFyqk1SjpFlJuGueXa1/FJ6j6H62Lehi3JZe1iZIEuG4zPm\n";

/// How long a registry is given to answer once started.
const STARTING: Duration = Duration::from_secs(30);

/// The registries the tools layer starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// zot, with the referrers API.
    Zot,
    /// The reference distribution, with none.
    Distribution,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Zot => "zot",
            Kind::Distribution => "the reference distribution",
        }
    }

    /// The pinned binary the devenv shell names; a panic naming the
    /// variable where it names none.
    fn binary(self) -> PathBuf {
        let (variable, tool) = match self {
            Kind::Zot => ("TIMBER_TOOLS_ZOT", "zot"),
            Kind::Distribution => ("TIMBER_TOOLS_DISTRIBUTION", "the reference distribution"),
        };
        match std::env::var_os(variable) {
            Some(path) if !path.is_empty() => PathBuf::from(path),
            _ => panic!(
                "{tool} is not in the shell ({variable} is unset): the tools layer runs in the \
                 devenv shell, which pins it"
            ),
        }
    }
}

pub use ports::unanswered;

/// A port for a registry: below the ephemeral ranges every host hands a
/// listener on port 0 from (32768 on Linux, 49152 on macOS), so no
/// listener of a test takes it before the registry binds it.
fn server_port() -> u16 {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    loop {
        let n = NEXT.fetch_add(1, Ordering::Relaxed) + u64::from(std::process::id()) * 131;
        let port = 30000 + (n % 2700) as u16;
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
}

/// A registry started for a test, stopped when dropped.
pub struct Server {
    pub kind: Kind,
    pub port: u16,
    child: Child,
    dir: TempDir,
}

impl Server {
    /// `kind` started with no credential asked.
    pub fn start(kind: Kind) -> Server {
        Server::started(kind, false)
    }

    /// `kind` started asking every request for [`USER`]'s credential, by
    /// `Basic`.
    pub fn asking(kind: Kind) -> Server {
        Server::started(kind, true)
    }

    fn started(kind: Kind, asks: bool) -> Server {
        let dir = TempDir::new().expect("a temporary directory");
        let root = dir.path().join("root");
        fs::create_dir_all(&root).unwrap();
        let htpasswd = dir.path().join("htpasswd");
        fs::write(&htpasswd, HTPASSWD).unwrap();
        for _ in 0..5 {
            let port = server_port();
            match Server::spawn(kind, asks, port, &dir) {
                Ok(child) => {
                    return Server {
                        kind,
                        port,
                        child,
                        dir,
                    }
                }
                Err(log) if log.contains("address already in use") => continue,
                Err(log) => panic!("{} did not start: {log}", kind.name()),
            }
        }
        panic!("{} found no free port in five tries", kind.name());
    }

    /// `kind` started on `port`, once `GET /v2/` answers anything at all;
    /// its log when it exits first or does not answer within
    /// [`STARTING`].
    fn spawn(kind: Kind, asks: bool, port: u16, dir: &TempDir) -> Result<Child, String> {
        let root = dir.path().join("root");
        let htpasswd = dir.path().join("htpasswd");
        let (config, args): (PathBuf, Vec<&str>) = match kind {
            Kind::Zot => {
                let mut http =
                    serde_json::json!({"address": "127.0.0.1", "port": port.to_string()});
                if asks {
                    http["auth"] = serde_json::json!({"htpasswd": {"path": htpasswd}});
                }
                let config = serde_json::json!({
                    "distSpecVersion": "1.1.1",
                    "storage": {"rootDirectory": root, "gc": false, "dedupe": false},
                    "http": http,
                    "log": {"level": "error"},
                });
                let path = dir.path().join("config.json");
                fs::write(&path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
                (path, vec!["serve"])
            }
            Kind::Distribution => {
                let auth = if asks {
                    format!(
                        "auth:\n  htpasswd:\n    realm: timber\n    path: {}\n",
                        htpasswd.display()
                    )
                } else {
                    String::new()
                };
                // Deletes on: a cache entry is removed by deleting its
                // cache manifest, which the reference distribution
                // answers UNSUPPORTED while deletes are off, its default
                // (145).
                let config = format!(
                    "version: 0.1\nlog:\n  level: error\n  accesslog:\n    disabled: true\n\
                     storage:\n  filesystem:\n    rootdirectory: {}\n  delete:\n    enabled: true\n\
                     {auth}http:\n  addr: 127.0.0.1:{port}\n",
                    root.display()
                );
                let path = dir.path().join("config.yml");
                fs::write(&path, config).unwrap();
                (path, vec!["serve"])
            }
        };
        let log_path = dir.path().join("log");
        let log = fs::File::create(&log_path).unwrap();
        let mut child = Command::new(kind.binary())
            .args(&args)
            .arg(&config)
            .stdin(Stdio::null())
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn()
            .unwrap_or_else(|e| panic!("{} could not be started: {e}", kind.name()));
        let log = || fs::read_to_string(&log_path).unwrap_or_default();
        let start = Instant::now();
        let url = format!("http://127.0.0.1:{port}/v2/");
        while start.elapsed() < STARTING {
            if let Ok(Some(status)) = child.try_wait() {
                return Err(format!("it exited {status}: {}", log()));
            }
            if let Ok(_) | Err(ureq::Error::StatusCode(_)) = ureq::get(&url).call() {
                return Ok(child);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = child.kill();
        let _ = child.wait();
        Err(format!(
            "it did not answer {url} within {STARTING:?}: {}",
            log()
        ))
    }

    /// The registry's host, as a repository names it.
    pub fn host(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    /// A root no other asking has been given, as the host's
    /// `[store] registry` names one.
    pub fn repository(&self) -> String {
        static MADE: AtomicU64 = AtomicU64::new(0);
        format!(
            "{}/store-{}",
            self.host(),
            MADE.fetch_add(1, Ordering::Relaxed)
        )
    }

    /// The server's one root, which [`Server::store`] and
    /// [`Server::store_for`] share.
    pub fn root(&self) -> String {
        format!("{}/store", self.host())
    }

    /// A registry store under the server's root, bound for the repository
    /// `acme/site`, whose own is `<root>/acme/site`.
    pub fn store(&self) -> Registry {
        self.store_for("acme", "site")
    }

    /// A registry store under the server's root, bound for the repository
    /// `owner/name`.
    pub fn store_for(&self, owner: &str, name: &str) -> Registry {
        store_under(&self.root(), owner, name)
    }
}

/// A registry store under `root`, `<host>/<path>`, bound for the
/// repository `owner/name` (C.1).
pub fn store_under(root: &str, owner: &str, name: &str) -> Registry {
    Registry::new(root)
        .expect("a registry's host and root")
        .for_repository(Ok(RepositoryName {
            owner: owner.into(),
            name: name.into(),
        }))
}

/// A registry store under `root`, bound for the repository `acme/site`.
pub fn site(root: &str) -> Registry {
    store_under(root, "acme", "site")
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The store group's rig: a registry store under a fresh root of a server
/// the test started, bound for the repository `acme/site` and the build
/// profile `moon`, whose cache repository holds its cache entries, and one
/// at a port nothing listens on, which [`StoreRig::unreachable`] makes
/// the rig's store (145).
pub struct RegistryRig {
    _server: Rc<Server>,
    live: Registry,
    dead: Registry,
    down: Cell<bool>,
}

impl RegistryRig {
    pub fn new(server: &Rc<Server>) -> RegistryRig {
        RegistryRig {
            live: site(&server.repository()).bound_for("moon", false),
            dead: site(&format!("127.0.0.1:{}/store", unanswered())).bound_for("moon", false),
            _server: server.clone(),
            down: Cell::new(false),
        }
    }
}

impl Rig for RegistryRig {
    fn profile(&self) -> String {
        self.live.source().profile.clone()
    }
}

impl StoreRig for RegistryRig {
    fn store(&self) -> &dyn Store {
        if self.down.get() {
            &self.dead
        } else {
            &self.live
        }
    }

    /// The rig's store pointed at a port nothing listens on: every
    /// operation answers with what a client asking that port is told.
    fn unreachable(&self) -> String {
        self.down.set(true);
        ureq::get(&format!("{}tags/list", self.dead.base()))
            .call()
            .expect_err("nothing listens on the port")
            .to_string()
    }

    /// The registry's bound: its tag reads may answer a moved name's
    /// former place for that long (C.1, ADR 0015).
    fn settles_within(&self) -> Duration {
        self.live.bound()
    }

    /// The registry's word: its names take no condition (C.1, ADR 0015).
    fn names_conditional(&self) -> bool {
        self.live.names_conditional()
    }
}

/// The repository a live rig's stores are bound for under the root the
/// host names, `<root>/timber-checks/store` its own (C.1).
pub const LIVE: (&str, &str) = ("timber-checks", "store");

/// A registry store under the root `root` the host names, bound for the
/// live rig's repository, [`LIVE`].
pub fn live(root: &str) -> Registry {
    store_under(root, LIVE.0, LIVE.1)
}

/// A prefix no earlier run's names carry.
pub fn run_prefix() -> String {
    let started = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("a clock after 1970")
        .as_secs();
    format!("run{started}-{}", std::process::id())
}

/// A registry store whose every name carries the run's prefix in the
/// space it is in: an artifact's on its name, so each run lists a
/// repository of its own, a cache entry's on its build identity, and
/// every other name before it.
pub struct Prefixed {
    pub registry: Registry,
    pub prefix: String,
}

impl Prefixed {
    fn named(&self, name: &Name) -> Name {
        if let Some((ecosystem, artifact, version)) = name.as_artifact() {
            return timber_seams::artifact_name(&ecosystem, &self.artifact(&artifact), &version);
        }
        match name.as_str().strip_prefix("cache/") {
            Some(identity) => Name::new(format!("cache/{}{identity}", self.prefix)),
            None => Name::new(format!("{}{name}", self.prefix)),
        }
    }

    fn artifact(&self, name: &timber_seams::EcosystemName) -> timber_seams::EcosystemName {
        timber_seams::EcosystemName::new(format!("{}{name}", self.prefix))
    }
}

impl Store for Prefixed {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        self.registry.put_bytes(bytes)
    }
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        self.registry.put_artifact(manifest)
    }
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        self.registry.attach(subject, attestation)
    }
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        self.registry.claims_of(digest)
    }
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        self.registry.name(&self.named(name), to, expected)
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.registry.resolve(&self.named(name))
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.registry.read(digest)
    }
    fn versions(
        &self,
        ecosystem: &timber_seams::Ecosystem,
        name: &timber_seams::EcosystemName,
    ) -> Answer<Vec<timber_seams::Version>> {
        self.registry.versions(ecosystem, &self.artifact(name))
    }
    /// The entries whose build identity carries the run's prefix, listed
    /// without it, so a scenario sees its own entries alone.
    fn cache_entries(&self) -> Answer<Vec<(timber_seams::BuildId, Answer<timber_seams::Pushed>)>> {
        Ok(self
            .registry
            .cache_entries()?
            .into_iter()
            .filter_map(|(build, entry)| {
                let own = build.as_str().strip_prefix(&self.prefix)?;
                Some((timber_seams::BuildId::new(own), entry))
            })
            .collect())
    }
    fn remove_cache_entry(&self, build: &timber_seams::BuildId, at: &Digest) -> Answer<()> {
        self.registry.remove_cache_entry(
            &timber_seams::BuildId::new(format!("{}{build}", self.prefix)),
            at,
        )
    }
}

/// The store group's rig under the root of a hosted registry the host
/// names, bound for the repository [`LIVE`] and the build profile `moon`,
/// a scenario's names apart from every other's by the run's prefix, and a
/// store at a loopback port nothing listens on, which
/// [`StoreRig::unreachable`] makes the rig's (145).
pub struct LiveRig {
    live: Prefixed,
    dead: Registry,
    down: Cell<bool>,
}

impl LiveRig {
    pub fn new(repository: &str, run: &str, scenario: usize) -> LiveRig {
        LiveRig {
            live: Prefixed {
                registry: live(repository).bound_for("moon", false),
                prefix: format!("{run}-{scenario}-"),
            },
            dead: live(&format!("127.0.0.1:{}/store", unanswered())).bound_for("moon", false),
            down: Cell::new(false),
        }
    }
}

impl Rig for LiveRig {
    fn profile(&self) -> String {
        self.live.registry.source().profile.clone()
    }
}

impl StoreRig for LiveRig {
    fn store(&self) -> &dyn Store {
        if self.down.get() {
            &self.dead
        } else {
            &self.live
        }
    }

    fn unreachable(&self) -> String {
        self.down.set(true);
        ureq::get(&format!("{}tags/list", self.dead.base()))
            .call()
            .expect_err("nothing listens on the port")
            .to_string()
    }

    /// The registry's bound: its tag reads may answer a moved name's
    /// former place for that long (C.1, ADR 0015).
    fn settles_within(&self) -> Duration {
        self.live.registry.bound()
    }

    /// The registry's word: its names take no condition, ECR's and
    /// GitHub's tag put among them (C.1, ADR 0015).
    fn names_conditional(&self) -> bool {
        self.live.registry.names_conditional()
    }
}

/// The store group run under the root `repository`, a fresh [`LiveRig`]
/// for each scenario under one run's prefix: every scenario passes and
/// ran, the report names the registry and the root, and only the refusal of a name moved
/// from where it is not is not applicable, since no registry the profile
/// binds takes a conditional tag put (C.1, ADR 0015).
pub fn store_group_against(repository: &str) {
    let run_prefix = run_prefix();
    let scenario = Cell::new(0);
    let report = run(store::GROUP, &store::scenarios(), &|| {
        scenario.set(scenario.get() + 1);
        Box::new(LiveRig::new(repository, &run_prefix, scenario.get()))
    });
    report.assert();
    assert_eq!(
        report.profile,
        format!("OCI registry {repository}"),
        "the report names {}",
        report.profile
    );
    assert_eq!(report.ran, store::scenarios().len());
    let not: Vec<&str> = report.not_applicable.iter().map(|n| n.scenario).collect();
    assert_eq!(not, ["a name moved from where it is not"], "{report}");
}

/// One request a [`Proxy`] passed: its method, its path, and the status
/// the registry answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Passed {
    pub method: String,
    pub path: String,
    pub status: u16,
}

/// A proxy on a loopback port in front of a registry: it passes each
/// request on a connection of its own, keeps its method and path, and
/// answers a `GET` of a path it is told to alter with the body's bytes
/// each flipped.
pub struct Proxy {
    pub port: u16,
    passed: Arc<Mutex<Vec<Passed>>>,
    alter: Arc<Mutex<Option<String>>>,
}

impl Proxy {
    pub fn before(server: &Server) -> Proxy {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let passed = Arc::new(Mutex::new(Vec::new()));
        let alter: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let upstream = server.port;
        let (kept, altered) = (passed.clone(), alter.clone());
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { return };
                let (kept, altered) = (kept.clone(), altered.clone());
                std::thread::spawn(move || {
                    let _ = pass(stream, upstream, &kept, &altered);
                });
            }
        });
        Proxy {
            port,
            passed,
            alter,
        }
    }

    /// A registry store under the root `repository`, reached through the
    /// proxy, bound for the repository `acme/site`.
    pub fn store(&self, repository: &str) -> Registry {
        self.store_for(repository, "acme", "site")
    }

    /// A registry store under the root `repository`, reached through the
    /// proxy, bound for the repository `owner/name`.
    pub fn store_for(&self, repository: &str, owner: &str, name: &str) -> Registry {
        let (_, path) = repository.split_once('/').unwrap();
        store_under(&format!("127.0.0.1:{}/{path}", self.port), owner, name)
    }

    /// Every request passed since the last asking, in order.
    pub fn passed(&self) -> Vec<Passed> {
        std::mem::take(&mut *self.passed.lock().unwrap())
    }

    /// Every `GET` of a path ending in `suffix` answered with other bytes.
    pub fn alter(&self, suffix: &str) {
        *self.alter.lock().unwrap() = Some(suffix.to_string());
    }
}

fn pass(
    client: TcpStream,
    upstream: u16,
    passed: &Mutex<Vec<Passed>>,
    alter: &Mutex<Option<String>>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(client.try_clone()?);
    let mut head = Vec::new();
    let mut length = 0usize;
    let mut first = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }
        if first.is_empty() {
            first = line.clone();
        }
        if line == "\r\n" {
            break;
        }
        let lower = line.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("content-length:") {
            length = value.trim().parse().unwrap_or(0);
        }
        if lower.starts_with("connection:") {
            continue;
        }
        head.extend_from_slice(line.as_bytes());
    }
    head.extend_from_slice(b"connection: close\r\n\r\n");
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    let mut parts = first.split_whitespace();
    let (method, path) = (
        parts.next().unwrap_or_default().to_string(),
        parts.next().unwrap_or_default().to_string(),
    );
    let mut server = TcpStream::connect(("127.0.0.1", upstream))?;
    server.write_all(&head)?;
    server.write_all(&body)?;
    let mut answer = Vec::new();
    server.read_to_end(&mut answer)?;
    let status = String::from_utf8_lossy(&answer[..answer.len().min(16)])
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    passed.lock().unwrap().push(Passed {
        method: method.clone(),
        path: path.clone(),
        status,
    });
    let altering = alter.lock().unwrap().as_ref().is_some_and(|suffix| {
        method == "GET"
            && path
                .split('?')
                .next()
                .unwrap_or("")
                .ends_with(suffix.as_str())
    });
    if altering {
        if let Some(at) = answer.windows(4).position(|w| w == b"\r\n\r\n") {
            for byte in &mut answer[at + 4..] {
                *byte = !*byte;
            }
        }
    }
    let mut client = client;
    client.write_all(&answer)?;
    client.flush()
}

/// The moment a store whose clock is fixed for the cache manifest's
/// scenarios puts each cache manifest at.
pub const PUT_AT: u64 = 1_793_000_000;

/// The cache manifest the registry store names `bytes` through for the
/// store name `name`, put at `at`, written out here apart from the
/// profile's own form, so a test that finds these bytes in a registry
/// proves the form (145; design D14).
pub fn cache_manifest(bytes: &[u8], name: &str, at: u64) -> Vec<u8> {
    format!(
        r#"{{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","artifactType":"application/vnd.agentplot.timber.cache.v1","config":{{"mediaType":"application/vnd.oci.empty.v1+json","digest":"{}","size":2}},"layers":[{{"mediaType":"application/octet-stream","digest":"{}","size":{}}}],"annotations":{{"io.github.agentplot.timber.store-name":"{name}","org.opencontainers.image.created":"{}"}}}}"#,
        Digest::of(b"{}"),
        Digest::of(bytes),
        bytes.len(),
        timber_seams::utc(at)
    )
    .into_bytes()
}

/// The cache manifest's scenarios a registry plays through the store
/// alone (spec registry-store, design D14): bytes named from nothing
/// resolve to their digest, and so do two names at the same bytes; a
/// name at bytes moved with their digest as expected to other bytes; the
/// cache manifest's own digest read as bytes, never named, and the bytes'
/// digest with no claims; a digest held neither way absent. A moved
/// name is read until it settles within `within`, the registry's bound,
/// as the store group reads one (C.1, 36). A move from bytes the name has
/// left is refused only by the check's best effort, so it is proven
/// where every read shows a move at once, in `registry_store.rs`, and
/// never against ECR (C.1, ADR 0015). The store's clock answers
/// [`PUT_AT`], and it writes each cache entry's name with `prefix` before
/// its build identity, as [`Prefixed`] does, so the cache manifest's
/// own digest is known (145).
pub fn cache_entries(store: &dyn Store, within: Duration, prefix: &str) {
    let (entry, other): (&[u8], &[u8]) = (b"a cache entry", b"another cache entry");
    let bytes = store.put_bytes(entry).unwrap().digest;
    let moved = store.put_bytes(other).unwrap().digest;
    let (one, two) = (Name::new("cache/one"), Name::new("cache/two"));
    for name in [&one, &two] {
        store.name(name, &bytes, None).unwrap();
    }
    let resolved =
        |name: &Name, to: &Digest, was: &[Option<&Digest>]| settled(store, within, name, to, was);
    for name in [&one, &two] {
        assert_eq!(
            resolved(name, &bytes, &[None]).as_ref(),
            Ok(&bytes),
            "{name}"
        );
    }

    store.name(&two, &moved, Some(&bytes)).unwrap();
    let was = [None, Some(&bytes)];
    assert_eq!(resolved(&two, &moved, &was).as_ref(), Ok(&moved));

    let manifest = cache_manifest(entry, &format!("cache/{prefix}one"), PUT_AT);
    let own = Digest::of(&manifest);
    assert_eq!(store.read(&own), Ok(Held::Bytes(manifest)));
    assert_eq!(store.claims_of(&bytes), Ok(vec![]));
    match store.name(&one, &own, Some(&bytes)) {
        Err(Not::Refused(r)) => {
            assert_eq!(r.why, Why::Said);
            assert!(r.said.unwrap().contains(bytes.as_str()));
        }
        answer => panic!("a name at a cache manifest's own digest answered {answer:?}"),
    }
    assert_eq!(resolved(&one, &bytes, &[None]).as_ref(), Ok(&bytes));

    let missing = Digest::of(b"bytes no name names");
    let three = Name::new("cache/three");
    match store.name(&three, &missing, None) {
        Err(Not::Absent(a)) => assert_eq!(a.what, missing.to_string()),
        answer => panic!("a name at a digest not held answered {answer:?}"),
    }
    assert!(matches!(store.resolve(&three), Err(Not::Absent(_))));
}

/// C.1, 103, 105: a moved repository's history carried by
/// `carry::history` from `from` to `to`, two roots under `root`, each
/// with the store ledger over its own store, every name carrying
/// `prefix` (the change a-moved-repository-keeps-its-history-in-the-store's
/// design D8). Under `from`, through a store carrying to `to`, so a mark
/// an earlier run left is read past, a version of `<prefix>web` is
/// published, named by its artifact `<prefix>site`, accepted and promoted
/// to `<prefix>web/beta`; carried, the version resolves under `to` with
/// every claim and the chain walks to the same head there, and a store
/// for `from` is refused naming `to` for the channel while it still reads
/// the artifact's name and its claims.
pub fn carried_between(root: &str, from: (&str, &str), to: (&str, &str), prefix: &str) {
    use serde_json::json;
    use timber::profiles::ledger::store::InStore;
    use timber_core::carry::{self, ChannelCarried, Root, VersionCarried};
    use timber_seams::{
        artifact_name, channel_name, Artifact, ArtifactMeta, ChannelId, ChannelName, Claim,
        ClaimKind, ComponentName, Ecosystem, EcosystemName, IndexMeta, Keeper, Moved, Promotion,
        Revision, Satisfied, Seam, Signature, SignerId, Source, Version, IN_TOTO,
    };

    let repository = |(owner, name): (&str, &str)| RepositoryName {
        owner: owner.into(),
        name: name.into(),
    };
    let (from, to) = (repository(from), repository(to));
    let at = |r: &RepositoryName| store_under(root, &r.owner, &r.name);
    let left = || at(&from).carrying_to(to.clone());
    let kept = TempDir::new().unwrap();
    let signed = |payload: Vec<u8>, sig: u8| {
        Envelope::signed(
            IN_TOTO,
            payload,
            Signature {
                signer: SignerId::new("gate"),
                keyid: String::new(),
                sig: vec![sig],
            },
        )
    };

    let (component, site) = (format!("{prefix}web"), format!("{prefix}site"));
    let meta = ArtifactMeta::new(
        Ecosystem::new("file"),
        EcosystemName::new(&site),
        Version::new("1.2.0"),
    )
    .unwrap();
    let publisher = left();
    let layer = publisher
        .put_bytes(format!("{prefix} the site at 1.2.0").as_bytes())
        .unwrap()
        .digest;
    let artifact = publisher
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    let index = publisher
        .put_artifact(&Manifest::Index(
            IndexMeta::new(
                ComponentName::new(&component),
                Version::new("1.2.0"),
                vec![Artifact {
                    digest: artifact,
                    meta: meta.clone(),
                }],
            )
            .unwrap(),
        ))
        .unwrap()
        .digest;
    let version = Name::new(format!("{component}:{component}-v1.2.0"));
    let pin = artifact_name(meta.ecosystem(), meta.name(), meta.version());
    publisher.name(&version, &index, None).unwrap();
    publisher.name(&pin, &index, None).unwrap();
    let acceptance = signed(
        Claim {
            subject: index.clone(),
            kind: ClaimKind::Acceptance,
            revision: Revision::new("4f2a9c1"),
            build: None,
            body: json!({}),
        }
        .payload(),
        4,
    );
    publisher.attach(&index, &acceptance).unwrap();
    let beta = ChannelId::new(
        Some(ComponentName::new(&component)),
        ChannelName::new("beta"),
    );
    let promotion = signed(
        Promotion {
            channel: ChannelName::new("beta"),
            component: ComponentName::new(&component),
            to: Moved {
                version: Version::new("1.2.0"),
                index: index.clone(),
            },
            from: None,
            predecessor: None,
            asked_by: SignerId::new("gate"),
            policy: Satisfied {
                revision: Revision::new("4f2a9c1"),
                claims: vec![acceptance.digest()],
            },
            at: 1_793_000_000,
        }
        .claim(Revision::new("4f2a9c1"))
        .payload(),
        5,
    );
    InStore::new(Box::new(left()), kept.path().join("published"))
        .append(&beta, &promotion, None)
        .unwrap();

    let (left_store, bound_store) = (left(), at(&to));
    let left_keeper = InStore::new(Box::new(left()), kept.path().join("left"));
    let bound_keeper = InStore::new(Box::new(at(&to)), kept.path().join("bound"));
    let (left_source, bound_source) = (
        Source::new(Seam::Store, format!("registry {root}, {from}")),
        Source::new(Seam::Store, format!("registry {root}, {to}")),
    );
    let carried = carry::history(
        Root {
            repository: &from,
            store: &left_store,
            source: &left_source,
            keeper: &left_keeper,
        },
        Root {
            repository: &to,
            store: &bound_store,
            source: &bound_source,
            keeper: &bound_keeper,
        },
        std::slice::from_ref(&version),
        std::slice::from_ref(&beta),
    )
    .unwrap();
    assert_eq!(carried.versions[0].carried, VersionCarried::Copied);
    assert_eq!(carried.channels[0].name, ChannelCarried::Made);
    assert_eq!(carried.channels[0].head, promotion.digest());

    let reader = at(&to);
    assert_eq!(reader.resolve(&version), Ok(index.clone()));
    let mut claims: Vec<Digest> = reader
        .claims_of(&index)
        .unwrap()
        .iter()
        .map(Envelope::digest)
        .collect();
    claims.sort();
    let mut expected = vec![acceptance.digest(), promotion.digest()];
    expected.sort();
    assert_eq!(claims, expected, "the version's claims were not carried");
    assert_eq!(reader.resolve(&channel_name(&beta)), Ok(index.clone()));
    let walked = InStore::new(Box::new(at(&to)), kept.path().join("walked"))
        .chain(&beta)
        .unwrap();
    assert_eq!(walked.claims, vec![promotion]);

    match at(&from).resolve(&channel_name(&beta)) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::RootMoved {
                from: from.lowercased(),
                to: to.lowercased(),
            }
        ),
        other => panic!("the channel at the root left answered {other:?}"),
    }
    let stale = at(&from);
    assert_eq!(stale.resolve(&pin), Ok(index.clone()));
    assert_eq!(stale.claims_of(&index).map(|c| c.len()), Ok(2));
}
