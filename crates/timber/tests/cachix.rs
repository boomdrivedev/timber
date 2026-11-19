//! Spec nix-cache-target, the proof against Cachix by hand (the Cachix
//! change's design D9): the throwaway public cache the host names in
//! `env.TIMBER_TOOLS_CACHIX` of its `devenv.local.nix`, as
//! `<cache>#<token file>`, where `<cache>` is a cache's name on Cachix no
//! one uses, and `<token file>` a file holding only a token that may push
//! to it, which a person makes. The `cachix` is the one the tree pins,
//! `TIMBER_TOOLS_CACHIX_CLI`, never the `cachix` on a person's path.
//!
//! The cache's public signing key is read from Cachix's API with no
//! token. A closure of two store paths is built with the host's Nix, a
//! root whose output names a leaf, the leaf holding a nonce of the run's
//! own so the cache holds neither; the root's line is packed by the file
//! projection into a layout store with its index, and the version
//! verified as the core makes it. The test's own environment is given
//! wrong `CACHIX_AUTH_TOKEN` and `CACHIX_SIGNING_KEY` values, the host's
//! credentials name `cachix-proof` at the token file, and the version is
//! applied through the Nix cache target. A fresh Nix store in the test's
//! directory, the cache's address its only source and its key the only
//! one trusted, realises the root and holds the leaf after. A second
//! apply converges, and the target group runs over a rig of the profile
//! naming the same cache (C.12, 42, 45, 64, 90, 119; record 0044).
//!
//! Its token is a file a person makes, and it reaches a hosted service,
//! so it runs by hand under `check-cachix` and in no gate task (rule
//! core.38), and it refuses to run, naming the setting, on a host that
//! names no cache, rather than pass without running. The paths it pushed
//! stay in the cache until Cachix's own retention collects them.

#![cfg(feature = "tools")]

use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output as Ran, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tempfile::TempDir;
use timber::bindings::Credentials;
use timber::profiles::projection::file::FileProjection;
use timber::profiles::store::layout::Layout;
use timber::profiles::target::nix_cache::{self, NixCache};
use timber_conformance::seams::target::{self, TargetRig};
use timber_conformance::Rig;
use timber_core::paths_hold;
use timber_seams::{
    Artifact, BuildId, ChannelId, ChannelName, Component, ComponentName, Digest, Environment,
    EnvironmentName, IndexMeta, Manifest, Output, OutputName, PackAt, Projection, Store, Target,
    Verified, Version,
};

/// The setting a host names its throwaway cache by.
const SETTING: &str = "TIMBER_TOOLS_CACHIX";

/// The setting naming the `cachix` the tree pins.
const CLI: &str = "TIMBER_TOOLS_CACHIX_CLI";

/// The credential's name, as the environments' tables give it.
const CREDENTIAL: &str = "cachix-proof";

/// The closure's component, its one output, a closure file, and the
/// version published.
const COMPONENT: &str = "timber-cachix-proof";
const VERSION: &str = "0.1.0";

/// A closure of two store paths built with the host's Nix and nothing
/// else: a leaf holding the run's nonce, and a root whose output names
/// it, so the root refers to the leaf. `/bin/sh` is every builder's, the
/// sandbox's among them.
const CLOSURE: &str = r#"let
  sh = name: script: derivation {
    inherit name;
    system = builtins.currentSystem;
    builder = "/bin/sh";
    args = [ "-c" script ];
  };
  leaf = sh "timber-cachix-leaf" "echo @NONCE@ > $out";
in
sh "timber-cachix-root" "echo ${leaf} > $out"
"#;

/// The cache the host names, and the token's file; a panic naming the
/// setting where it names none, or names one of another form.
struct Setting {
    cache: String,
    token: PathBuf,
}

fn setting() -> Setting {
    let named = std::env::var(SETTING).unwrap_or_default();
    let read = named.trim().split_once('#').and_then(|(cache, token)| {
        let is_name = !cache.is_empty()
            && !cache.starts_with('-')
            && cache
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        (is_name && !token.is_empty()).then(|| Setting {
            cache: cache.into(),
            token: PathBuf::from(token),
        })
    });
    read.unwrap_or_else(|| {
        panic!(
            "the Cachix proof names no cache: set env.{SETTING} in devenv.local.nix to \
             `<cache>#<token file>`, a throwaway public cache's name on Cachix no one uses, and \
             a file holding only a token that may push to it"
        )
    })
}

/// The `cachix` the tree pins; a panic naming its setting where the
/// shell names none.
fn cli() -> PathBuf {
    match std::env::var_os(CLI) {
        Some(cli) if !cli.is_empty() => PathBuf::from(cli),
        _ => panic!("{CLI} names no cachix: run the proof in the repository's devenv shell"),
    }
}

/// `program` run with `args`, its output whatever its exit; a panic
/// naming it where it cannot be started.
fn run(program: &mut Command) -> Ran {
    program
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("{program:?} could not be started: {e}"))
}

/// `program`'s stdout where it exits 0, else a panic with its words.
fn succeeds(program: &mut Command) -> String {
    let ran = run(program);
    assert!(
        ran.status.success(),
        "{program:?} exited {}: {}{}",
        ran.status,
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr)
    );
    String::from_utf8(ran.stdout).expect("the output is UTF-8")
}

/// The public signing keys Cachix's API answers for the public cache
/// `cache`, read with no token; a panic saying the proof's cache is
/// public where it answers none.
fn public_keys(cache: &str) -> Vec<String> {
    let url = format!("https://cachix.org/api/v1/cache/{cache}");
    let mut answered = ureq::get(&url).call().unwrap_or_else(|e| {
        panic!(
            "{url} answered {e}: the proof's cache is a public cache on Cachix, whose keys \
             Cachix answers with no token"
        )
    });
    let body = answered
        .body_mut()
        .read_to_string()
        .unwrap_or_else(|e| panic!("{url} could not be read: {e}"));
    let answer: Value =
        serde_json::from_str(&body).unwrap_or_else(|e| panic!("{url} answered no JSON: {e}"));
    let keys: Vec<String> = answer["publicSigningKeys"]
        .as_array()
        .map(|keys| {
            keys.iter()
                .filter_map(|key| key.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    assert!(
        !keys.is_empty(),
        "{url} answered no public signing key: the proof's cache is a public cache on Cachix, \
         signed by Cachix: {body}"
    );
    keys
}

/// A nonce of the run's own, so the cache holds none of its closure.
fn nonce() -> String {
    let at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("timber-cachix-{}-{}", at.as_nanos(), std::process::id())
}

/// The closure built with the host's Nix under `dir`, its root held by a
/// link there; the root's store path and the leaf's, which it refers to.
fn build(dir: &Path) -> (String, String) {
    fs::create_dir_all(dir).unwrap();
    fs::write(
        dir.join("closure.nix"),
        CLOSURE.replace("@NONCE@", &nonce()),
    )
    .unwrap();
    succeeds(
        Command::new("nix-build")
            .arg(dir.join("closure.nix"))
            .arg("--out-link")
            .arg(dir.join("result")),
    );
    let root = fs::read_link(dir.join("result"))
        .expect("nix-build linked its output")
        .display()
        .to_string();
    let references = succeeds(Command::new("nix-store").args(["--query", "--references", &root]));
    let references: Vec<&str> = references.lines().collect();
    let [leaf] = references.as_slice() else {
        panic!("the root {root} refers to {references:?}, not to its leaf alone");
    };
    assert!(leaf.ends_with("-timber-cachix-leaf"), "{leaf}");
    (root, leaf.to_string())
}

/// The closure file naming `root`, written at `file` as a build task
/// leaves it, packed by the file projection at [`VERSION`] and put in a
/// layout store made at `layout` as a publish puts it: its bytes, its
/// artifact and the version's index naming it; the version verified, as
/// the core makes it after its verify (42).
fn publish(layout: &Path, file: &Path, root: &str) -> Verified {
    fs::write(file, format!("{root}\n")).unwrap();
    let store = Layout::create(layout).expect("a layout made");
    let output = Output {
        name: OutputName::new(COMPONENT),
        files: vec![file.display().to_string()],
    };
    let component = Component {
        name: ComponentName::new(COMPONENT),
        outputs: vec![output.name.clone()],
        paths: vec!["proof/".into()],
        prefix: format!("{COMPONENT}-v"),
    };
    let holds = paths_hold(&component.paths).unwrap();
    let version = Version::new(VERSION);
    let packed = FileProjection::new()
        .pack(
            &output,
            &BuildId::new("c4c1"),
            Some(&PackAt {
                version: &version,
                component: &component,
                outputs: std::slice::from_ref(&output),
                holds: &holds,
            }),
        )
        .unwrap_or_else(|not| panic!("the closure file packed: {not}"));
    let meta = packed.meta.expect("the file projection names its artifact");
    let layer = store.put_bytes(&packed.bytes).unwrap().digest;
    let digest = store
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    let index = IndexMeta::new(
        component.name.clone(),
        version.clone(),
        vec![Artifact { digest, meta }],
    )
    .unwrap();
    let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
    let tag = format!("{}{version}", component.prefix);
    Verified::new(index, version, tag)
}

/// `nix` run as another host runs it: none of this host's configuration
/// read, its cache of what binary caches hold its own under `at`, so the
/// only key it trusts and the only source it reads are its arguments'.
fn fresh_nix(at: &Path) -> Command {
    let conf = at.join("fresh-conf");
    fs::create_dir_all(&conf).unwrap();
    fs::write(conf.join("nix.conf"), "").unwrap();
    let mut nix = Command::new("nix");
    nix.env_remove("NIX_CONFIG")
        .env("NIX_CONF_DIR", &conf)
        .env("NIX_USER_CONF_FILES", conf.join("nix.conf"))
        .env("XDG_CACHE_HOME", at.join("fresh-cache"))
        .args(["--extra-experimental-features", "nix-command"]);
    nix
}

/// What the proof stands up: the cache and its keys, the host's
/// credentials naming the token, the closure built and the version
/// publishing it.
struct Proof {
    dir: TempDir,
    cache: String,
    keys: Vec<String>,
    credentials: Credentials,
    root: String,
    leaf: String,
    layout: PathBuf,
    verified: Verified,
}

impl Proof {
    fn new(setting: Setting) -> Proof {
        let keys = public_keys(&setting.cache);
        let dir = TempDir::new().expect("a temporary directory");
        // Nix refuses a store under a path through a link, as macOS's
        // temporary directory is.
        let at = fs::canonicalize(dir.path()).unwrap();
        let credentials = Credentials {
            file: at.join("config.toml"),
            paths: [(CREDENTIAL.to_string(), setting.token)].into(),
        };
        let (root, leaf) = build(&at.join("build"));
        let layout = at.join("layout");
        let verified = publish(&layout, &at.join("build/closure"), &root);
        Proof {
            dir,
            cache: setting.cache,
            keys,
            credentials,
            root,
            leaf,
            layout,
            verified,
        }
    }

    fn at(&self) -> PathBuf {
        fs::canonicalize(self.dir.path()).unwrap()
    }

    /// The Nix cache target over a store binding of its own, opened and
    /// never made, running the cachix the tree pins and the host's
    /// nix-store.
    fn target(&self) -> NixCache {
        let store = Layout::open(&self.layout).expect("the layout the test published into");
        NixCache::with_programs(
            Box::new(store),
            &self.credentials,
            "attic",
            cli(),
            "nix-store",
        )
    }

    /// The cache environment `name` of the closure's component, following
    /// `beta`, its table naming the cache, the credential and the closure.
    fn environment(&self, name: &str) -> Environment {
        Environment {
            name: EnvironmentName::new(name),
            channel: ChannelId::new(
                Some(ComponentName::new(COMPONENT)),
                ChannelName::new("beta"),
            ),
            waits: false,
            settings: json!({
                "cachix": self.cache,
                "credential": CREDENTIAL,
                "closures": [COMPONENT],
            }),
        }
    }

    /// The cache's address, as a Nix store names a binary cache.
    fn address(&self) -> String {
        format!("https://{}.cachix.org", self.cache)
    }
}

/// The proof: the version applied through the Nix cache target from the
/// host that built its closure, with the host's own cachix settings
/// wrong in timber's environment; then a fresh Nix store realises the
/// root and its leaf from the cache alone, and a second apply converges
/// (C.12, 42, 45, 64, 90).
fn realised_from_the_cache_alone(proof: &Proof) {
    let target = proof.target();
    let environment = proof.environment("cachix");
    // A token Cachix refuses and a key that is none: a run that read
    // either would fail.
    std::env::set_var("CACHIX_AUTH_TOKEN", "not-the-caches-token");
    std::env::set_var("CACHIX_SIGNING_KEY", "not-a-signing-key");
    target
        .apply(&environment, &proof.verified)
        .unwrap_or_else(|not| panic!("the apply: {not}"));

    let at = proof.at();
    let fresh = at.join("fresh");
    succeeds(
        fresh_nix(&at)
            .args(["copy", "--option", "substituters", ""])
            .args(["--option", "trusted-public-keys", &proof.keys.join(" ")])
            .args(["--from", &proof.address()])
            .arg("--to")
            .arg(format!("local?root={}", fresh.display()))
            .arg(&proof.root),
    );
    for path in [&proof.root, &proof.leaf] {
        let realised = fresh.join(path.trim_start_matches('/'));
        assert_eq!(
            fs::read(&realised).unwrap_or_else(|e| panic!("the fresh store holds no {path}: {e}")),
            fs::read(path).unwrap(),
            "the fresh store realised other bytes at {path}"
        );
    }

    target
        .apply(&environment, &proof.verified)
        .unwrap_or_else(|not| panic!("the second apply: {not}"));
}

/// The target group's rig over the Nix cache target and the proof's
/// cache: the target holds no marker, every environment names the
/// closure the version published, and its foreign settings add a `token`
/// (119, 45; design D9).
struct CachixRig {
    target: NixCache,
    verified: Verified,
    cache: String,
    settings: Value,
    environments: Cell<u32>,
}

impl CachixRig {
    fn new(proof: &Proof) -> CachixRig {
        CachixRig {
            target: proof.target(),
            verified: proof.verified.clone(),
            cache: proof.cache.clone(),
            settings: proof.environment("cachix").settings,
            environments: Cell::new(0),
        }
    }

    fn next(&self) -> Environment {
        self.environments.set(self.environments.get() + 1);
        Environment {
            name: EnvironmentName::new(format!("cachix-{}", self.environments.get())),
            channel: ChannelId::new(
                Some(ComponentName::new(COMPONENT)),
                ChannelName::new("beta"),
            ),
            waits: false,
            settings: self.settings.clone(),
        }
    }
}

impl Rig for CachixRig {
    fn profile(&self) -> String {
        nix_cache::PROFILE.into()
    }
}

impl TargetRig for CachixRig {
    fn target(&self) -> &dyn Target {
        &self.target
    }

    /// The version the test published, whatever `digest` the suite
    /// names: a store holds an index only at its own digest.
    fn verified(&self, _digest: &Digest) -> Verified {
        self.verified.clone()
    }

    /// The cache is pushed to, handed the head at desired, so `head` is
    /// left unread.
    fn environment(&self, _head: Option<&Digest>) -> Environment {
        self.next()
    }

    /// No environment of a cache can be read, for the cache keeps no
    /// marker of what was pushed to it.
    fn unreadable_environment(&self) -> (Environment, String) {
        (
            self.next(),
            format!(
                "the Nix binary cache {} on Cachix holds store paths by their hash and keeps no \
                 marker of what was pushed to it",
                self.cache
            ),
        )
    }

    fn holds_marker(&self) -> bool {
        false
    }

    fn foreign_settings(&self) -> (Environment, String) {
        let mut environment = self.next();
        environment.settings["token"] = json!("a credential");
        (environment, "token".into())
    }
}

#[test]
fn a_closure_built_here_is_realised_from_cachix_alone_and_the_target_group_passes() {
    let setting = setting();
    cli();
    let proof = Proof::new(setting);
    realised_from_the_cache_alone(&proof);

    let report = timber_conformance::run(target::GROUP, &target::scenarios(), &|| {
        Box::new(CachixRig::new(&proof)) as Box<dyn TargetRig>
    });
    report.assert();
    let inapplicable: Vec<&str> = report.not_applicable.iter().map(|n| n.scenario).collect();
    assert_eq!(inapplicable, ["converged and reported"], "{report}");
}
