use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use serde_json::{json, Value};
use tempfile::TempDir;
use timber_fakes::FakeStore;
use timber_seams::{
    Artifact, ArtifactMeta, ChannelId, ChannelName, ComponentName, Ecosystem, EcosystemName,
    EnvironmentName, IndexMeta, Manifest, Version,
};

use super::*;
use crate::profiles::run::under_env;

const VERSION: &str = "0.4.0";
const COMPONENT: &str = "swancloud-srv";

/// A root of the machine's closure, under this host's store directory.
fn root() -> String {
    format!(
        "{}/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sbm-nixos-system-srv-25.05",
        nix_closure::store_dir()
    )
}

/// A second root, of another closure.
fn other_root() -> String {
    format!(
        "{}/9kn2wd8q1ycx0l6hzm4fjrb3gv5sap7i-hello-2.12.1",
        nix_closure::store_dir()
    )
}

fn cache(settings: Value) -> Environment {
    Environment {
        name: EnvironmentName::new("cache"),
        channel: ChannelId::new(
            Some(ComponentName::new(COMPONENT)),
            ChannelName::new("beta"),
        ),
        waits: false,
        settings,
    }
}

fn swancloud() -> Environment {
    cache(json!({ "attic": "fleet:swancloud", "closures": [COMPONENT] }))
}

/// A stand-in attic, cachix and nix-store, written by the test, each
/// recording the arguments it was given one to a line, and its name in
/// `ran` in the order they ran; attic and cachix record their
/// environment, and cachix whether the file its `--config` names exists
/// and what that file's folder holds; nix-store prints the paths of
/// `invalid` as `--print-invalid` does; and each fails with the words of
/// its `fails` where the test writes one.
struct Programs {
    dir: TempDir,
}

impl Programs {
    fn new() -> Programs {
        let dir = TempDir::new().unwrap();
        let at = dir.path().display().to_string();
        let attic = format!(
            "#!/bin/sh\n\
             echo attic >> '{at}/ran'\n\
             for arg in \"$@\"; do printf '%s\\n' \"$arg\"; done > '{at}/attic.args'\n\
             env > '{at}/attic.env'\n\
             if [ -f '{at}/attic.fails' ]; then cat '{at}/attic.fails' >&2; exit 1; fi\n"
        );
        let cachix = format!(
            "#!/bin/sh\n\
             echo cachix >> '{at}/ran'\n\
             for arg in \"$@\"; do printf '%s\\n' \"$arg\"; done > '{at}/cachix.args'\n\
             env > '{at}/cachix.env'\n\
             config=''; before=''\n\
             for arg in \"$@\"; do\n\
               if [ \"$before\" = --config ]; then config=\"$arg\"; fi; before=\"$arg\"\n\
             done\n\
             if [ -e \"$config\" ]; then echo held; else echo none; fi > '{at}/cachix.config'\n\
             ls -A \"$(dirname \"$config\")\" > '{at}/cachix.folder' 2>&1\n\
             if [ -f '{at}/cachix.fails' ]; then cat '{at}/cachix.fails' >&2; exit 1; fi\n"
        );
        let nix_store = format!(
            "#!/bin/sh\n\
             echo nix-store >> '{at}/ran'\n\
             for arg in \"$@\"; do printf '%s\\n' \"$arg\"; done > '{at}/nix-store.args'\n\
             if [ -f '{at}/nix-store.fails' ]; then cat '{at}/nix-store.fails' >&2; exit 1; fi\n\
             if [ -f '{at}/invalid' ]; then cat '{at}/invalid'; fi\n"
        );
        for (name, script) in [
            ("attic", attic),
            ("cachix", cachix),
            ("nix-store", nix_store),
        ] {
            let program = dir.path().join(name);
            fs::write(&program, script).unwrap();
            fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
        }
        Programs { dir }
    }

    fn program(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }

    fn target(&self, store: &FakeStore) -> NixCache {
        self.target_with(store, &Credentials::default())
    }

    /// The target over `store`, the host's credentials `credentials`.
    fn target_with(&self, store: &FakeStore, credentials: &Credentials) -> NixCache {
        NixCache::with_programs(
            Box::new(store.clone()),
            credentials,
            self.program("attic"),
            self.program("cachix"),
            self.program("nix-store"),
        )
    }

    /// The programs that ran, in the order they ran.
    fn ran(&self) -> Vec<String> {
        fs::read_to_string(self.dir.path().join("ran"))
            .map(|ran| ran.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// The lines of `file` a stand-in wrote.
    fn lines(&self, file: &str) -> Vec<String> {
        fs::read_to_string(self.dir.path().join(file))
            .unwrap_or_else(|e| panic!("no {file}: {e}"))
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// The paths this host's store does not hold.
    fn lacks(&self, paths: &[String]) {
        let mut text = paths.join("\n");
        text.push('\n');
        fs::write(self.dir.path().join("invalid"), text).unwrap();
    }

    /// `program` fails from now, with `words` on its stderr.
    fn fails(&self, program: &str, words: &str) {
        fs::write(self.dir.path().join(format!("{program}.fails")), words).unwrap();
    }

    /// The arguments `program` was given, or `None` where it never ran.
    fn args(&self, program: &str) -> Option<Vec<String>> {
        fs::read_to_string(self.dir.path().join(format!("{program}.args")))
            .ok()
            .map(|args| args.lines().map(str::to_string).collect())
    }

    /// The names of the variables attic ran with.
    fn attic_env(&self) -> BTreeSet<String> {
        let env = fs::read_to_string(self.dir.path().join("attic.env")).unwrap();
        env.lines()
            .filter_map(|line| line.split_once('=').map(|(name, _)| name))
            .filter(|name| {
                !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            })
            .map(str::to_string)
            .collect()
    }
}

/// A version of `COMPONENT` published into `store`, each of `files` an
/// artifact of the ecosystem it names, as the file projection packs one:
/// a manifest naming one layer of the file's bytes; and the version
/// verified, as the core makes it.
fn publish(store: &FakeStore, files: &[(&str, &str, &str)]) -> Verified {
    let mut artifacts = Vec::new();
    for (ecosystem, name, text) in files {
        let layer = store.put_bytes(text.as_bytes()).unwrap().digest;
        let meta = ArtifactMeta::new(
            Ecosystem::new(*ecosystem),
            EcosystemName::new(*name),
            Version::new(VERSION),
        )
        .unwrap();
        let manifest = Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        };
        let digest = store.put_artifact(&manifest).unwrap().digest;
        artifacts.push(Artifact { digest, meta });
    }
    let index = IndexMeta::new(
        ComponentName::new(COMPONENT),
        Version::new(VERSION),
        artifacts,
    )
    .unwrap();
    let digest = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
    Verified::new(
        digest,
        Version::new(VERSION),
        format!("{COMPONENT}-v{VERSION}"),
    )
}

fn closure_of(roots: &[String]) -> String {
    let mut text = roots.join("\n");
    text.push('\n');
    text
}

fn refused_said(answer: Answer<()>) -> String {
    match answer {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, NixCache::source());
            assert_eq!(r.why, Why::Said);
            r.said.expect("the target's words")
        }
        other => panic!("answered {other:?}, not refused"),
    }
}

fn unreadable(answer: Answer<impl std::fmt::Debug>) -> (String, String) {
    match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, NixCache::source());
            (u.what, u.said.expect("the words"))
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
}

fn index(n: u8) -> Digest {
    Digest::of(&[b'i', n])
}

// Requirement: The target's table names the cache and the closures, and
// nothing else

#[test]
fn a_token_in_the_table_is_unreadable_at_every_operation_and_attic_is_not_run() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let target = programs.target(&store);
    let environment = cache(json!({
        "attic": "fleet:swancloud",
        "closures": [COMPONENT],
        "token": "eyJhbGciOi",
    }));
    let named = "setting token of environment cache of swancloud-srv";
    let (what, said) = unreadable(target.apply(&environment, &version));
    assert_eq!(what, named);
    assert!(said.contains("no setting token"), "{said}");
    assert_eq!(
        unreadable(target.desired(&environment, &|| Ok(index(3)))).0,
        named
    );
    assert_eq!(unreadable(target.current(&environment)).0, named);
    assert_eq!(unreadable(target.report(&environment)).0, named);
    assert_eq!(programs.args("attic"), None, "attic ran");
    assert_eq!(programs.args("nix-store"), None, "nix-store ran");
}

#[test]
fn a_cache_with_no_server_is_unreadable_saying_it_is_server_and_cache() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let environment = cache(json!({ "attic": "swancloud", "closures": [COMPONENT] }));
    let (what, said) = unreadable(programs.target(&store).apply(&environment, &version));
    assert_eq!(what, "setting attic of environment cache of swancloud-srv");
    assert!(said.contains("<server>:<cache>"), "{said}");
    assert_eq!(programs.args("attic"), None, "attic ran");
}

#[test]
fn an_empty_closures_is_unreadable_naming_it() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let environment = cache(json!({ "attic": "fleet:swancloud", "closures": [] }));
    let (what, _) = unreadable(programs.target(&store).apply(&environment, &version));
    assert_eq!(
        what,
        "setting closures of environment cache of swancloud-srv"
    );
    assert_eq!(programs.args("attic"), None, "attic ran");
}

// Requirement: desired is the channel's head

#[test]
fn desired_answers_the_head_it_is_handed() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let target = programs.target(&store);
    assert_eq!(target.desired(&swancloud(), &|| Ok(index(3))), Ok(index(3)));
    let empty = || -> Answer<Digest> {
        Err(Not::absent(
            &Source::new(Seam::Keeper, "fake"),
            "the head of the chain of swancloud-srv/beta",
        ))
    };
    assert_eq!(target.desired(&swancloud(), &empty), empty());
}

// Requirement: current and report are unread, since a cache keeps no
// marker

#[test]
fn current_and_report_are_unread_naming_the_cache_even_after_an_apply() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let target = programs.target(&store);
    target.apply(&swancloud(), &version).unwrap();
    let applied = version.index().to_string();
    for (what, said) in [
        unreadable(target.current(&swancloud())),
        unreadable(target.report(&swancloud())),
    ] {
        assert_eq!(what, "what environment cache of swancloud-srv runs");
        assert_eq!(
            said,
            "the Nix binary cache fleet:swancloud holds store paths by their hash and keeps no \
             marker of what was pushed to it"
        );
        assert!(!said.contains(&applied) && !what.contains(&applied));
    }
}

// Requirement: Apply pushes each root with its closure through attic

#[test]
fn attic_is_given_the_cache_and_the_roots_and_nothing_else() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(
        &store,
        &[
            ("file", COMPONENT, &closure_of(&[root()])),
            ("file", "tools", &closure_of(&[other_root(), root()])),
        ],
    );
    let environment = cache(json!({
        "attic": "fleet:swancloud",
        "closures": [COMPONENT, "tools"],
    }));
    let (before, applied) = under_env(&[], || {
        let before: BTreeSet<String> = std::env::vars_os()
            .map(|(name, _)| name.to_string_lossy().into_owned())
            .collect();
        (
            before,
            programs.target(&store).apply(&environment, &version),
        )
    });
    assert_eq!(applied, Ok(()));
    assert_eq!(
        programs.args("nix-store").unwrap(),
        [
            "--check-validity",
            "--print-invalid",
            &root(),
            &other_root()
        ]
    );
    assert_eq!(
        programs.args("attic").unwrap(),
        ["push", "fleet:swancloud", &root(), &other_root()]
    );
    let set: Vec<String> = programs
        .attic_env()
        .into_iter()
        .filter(|name| !before.contains(name))
        .filter(|name| !["PWD", "OLDPWD", "SHLVL", "_"].contains(&name.as_str()))
        .collect();
    assert!(set.is_empty(), "timber set {set:?} for attic");
}

#[test]
fn attic_failing_is_unreadable_in_its_own_words() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let words = "Error: Unauthorized: You are not authorized to push to cache swancloud";
    programs.fails("attic", &format!("{words}\n"));
    let (what, said) = unreadable(programs.target(&store).apply(&swancloud(), &version));
    assert_eq!(said, words);
    assert!(
        what.ends_with(&format!("attic push fleet:swancloud {}", root())),
        "{what}"
    );
}

// Requirement: Apply pushes only from the host that holds the closure

#[test]
fn a_root_this_host_does_not_hold_is_refused_naming_it_and_the_host_and_attic_never_runs() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(
        &store,
        &[("file", COMPONENT, &closure_of(&[other_root(), root()]))],
    );
    programs.lacks(&[root()]);
    let said = refused_said(programs.target(&store).apply(&swancloud(), &version));
    assert!(
        said.starts_with(&format!("{} is not valid in the Nix store of ", root())),
        "{said}"
    );
    assert!(said.contains(&host()), "{said}");
    assert!(said.contains("building and fetching nothing"), "{said}");
    assert_eq!(programs.args("attic"), None, "attic ran");
}

#[test]
fn nix_store_failing_is_unreadable_in_its_own_words_and_attic_never_runs() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let words = "error: cannot connect to socket at '/nix/var/nix/daemon-socket/socket'";
    programs.fails("nix-store", words);
    let (_, said) = unreadable(programs.target(&store).apply(&swancloud(), &version));
    assert_eq!(said, words);
    assert_eq!(programs.args("attic"), None, "attic ran");
}

// Requirement: Apply reads each closure the version names by digest

#[test]
fn a_closure_output_the_version_lacks_is_refused_naming_it_and_the_version() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("eco", COMPONENT, "not a closure")]);
    let said = refused_said(programs.target(&store).apply(&swancloud(), &version));
    assert_eq!(
        said,
        "version 0.4.0 of swancloud-srv names no file artifact of the output swancloud-srv, so \
         it carries no closure of it"
    );
    assert_eq!(programs.args("nix-store"), None, "nix-store ran");
    assert_eq!(programs.args("attic"), None, "attic ran");
}

#[test]
fn a_line_that_is_no_store_path_is_refused_naming_it_and_nothing_runs() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = publish(&store, &[("file", COMPONENT, "result\n")]);
    let said = refused_said(programs.target(&store).apply(&swancloud(), &version));
    assert!(said.contains("`result`"), "{said}");
    assert!(
        said.contains("the closure file swancloud-srv@0.4.0"),
        "{said}"
    );
    assert_eq!(programs.args("nix-store"), None, "nix-store ran");
    assert_eq!(programs.args("attic"), None, "attic ran");
}

#[test]
fn a_closure_file_whose_bytes_are_not_its_layers_is_refused_and_nothing_runs() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let text = closure_of(&[root()]);
    let version = publish(&store, &[("file", COMPONENT, &text)]);
    let layer = Digest::of(text.as_bytes());
    store.corrupt(&layer, closure_of(&[other_root()]).as_bytes());
    match programs.target(&store).apply(&swancloud(), &version) {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, NixCache::source());
            assert!(
                matches!(&r.why, Why::OtherBytes { expected, .. } if expected == &layer),
                "{:?}",
                r.why
            );
        }
        other => panic!("answered {other:?}"),
    }
    assert_eq!(programs.args("nix-store"), None, "nix-store ran");
}

#[test]
fn an_index_the_store_does_not_hold_is_unreadable_naming_the_version() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let version = Verified::new(
        index(9),
        Version::new(VERSION),
        format!("{COMPONENT}-v{VERSION}"),
    );
    let (what, said) = unreadable(programs.target(&store).apply(&swancloud(), &version));
    assert_eq!(what, "the index of version 0.4.0");
    assert_eq!(said, format!("the store holds no {}", index(9)));
}

#[test]
fn an_index_of_another_version_than_the_one_verified_is_refused() {
    let (store, programs) = (FakeStore::new(), Programs::new());
    let published = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
    let version = Verified::new(
        published.index().clone(),
        Version::new("0.5.0"),
        format!("{COMPONENT}-v0.5.0"),
    );
    let said = refused_said(programs.target(&store).apply(&swancloud(), &version));
    assert!(
        said.ends_with("names version 0.4.0, and the version verified is 0.5.0"),
        "{said}"
    );
    assert_eq!(programs.args("nix-store"), None, "nix-store ran");
}

mod cachix {
    //! Requirements: Apply pushes each root with its closure through
    //! cachix; the Cachix token is the host's, read at the moment of use;
    //! the token reaches one run of cachix alone (record 0044; the Cachix
    //! change's design D5, D7, D8).

    use std::path::Path;

    use super::*;

    const TOKEN: &str = "eyJhbGciOiJIUzI1NiJ9.cachix-boomdrive";
    const CREDENTIAL: &str = "cachix-boomdrive";
    const BINDINGS: &str = "/home/fleet/.config/timber/config.toml";

    /// The words Cachix refuses a push to a cache it holds none of in.
    const REFUSED: &str =
        "Binary cache boomdrive doesn't exist or you're not authorized to access it.";

    pub(super) fn boomdrive() -> Environment {
        cache(json!({
            "cachix": "boomdrive",
            "credential": CREDENTIAL,
            "closures": [COMPONENT],
        }))
    }

    /// The host's credentials, naming each of `named` at its path.
    pub(super) fn credentials(named: &[(&str, &Path)]) -> Credentials {
        Credentials {
            file: PathBuf::from(BINDINGS),
            paths: named
                .iter()
                .map(|(name, path)| (name.to_string(), path.to_path_buf()))
                .collect(),
        }
    }

    /// A host whose credential `cachix-boomdrive` is a file in `dir`
    /// holding the token and a newline.
    pub(super) fn host(dir: &TempDir) -> Credentials {
        let file = dir.path().join(CREDENTIAL);
        fs::write(&file, format!("{TOKEN}\n")).unwrap();
        credentials(&[(CREDENTIAL, &file)])
    }

    /// The folder of the file cachix's `--config` named.
    fn folder(programs: &Programs) -> PathBuf {
        let args = programs.args("cachix").expect("cachix ran");
        assert_eq!(args[0], "--config", "{args:?}");
        let config = Path::new(&args[1]);
        assert_eq!(config.file_name().unwrap(), "cachix.dhall", "{args:?}");
        config.parent().unwrap().to_path_buf()
    }

    /// The `CACHIX_*` variables cachix ran with, each `NAME=value`.
    fn cachix_variables(programs: &Programs) -> Vec<String> {
        let mut theirs: Vec<String> = programs
            .lines("cachix.env")
            .into_iter()
            .filter(|line| line.starts_with("CACHIX_"))
            .collect();
        theirs.sort();
        theirs
    }

    #[test]
    fn cachix_is_given_the_config_push_the_name_and_every_root_once_after_nix_stores_check() {
        let (store, programs, dir) = (FakeStore::new(), Programs::new(), TempDir::new().unwrap());
        let version = publish(
            &store,
            &[
                ("file", COMPONENT, &closure_of(&[root()])),
                ("file", "tools", &closure_of(&[other_root(), root()])),
            ],
        );
        let environment = cache(json!({
            "cachix": "boomdrive",
            "credential": CREDENTIAL,
            "closures": [COMPONENT, "tools"],
        }));
        let target = programs.target_with(&store, &host(&dir));
        assert_eq!(target.apply(&environment, &version), Ok(()));
        assert_eq!(programs.ran(), ["nix-store", "cachix"]);
        let folder = folder(&programs).display().to_string();
        assert_eq!(
            programs.args("cachix").unwrap(),
            [
                "--config",
                &format!("{folder}/cachix.dhall"),
                "push",
                "boomdrive",
                &root(),
                &other_root(),
            ]
        );
        assert_eq!(programs.args("attic"), None, "attic ran");
    }

    #[test]
    fn the_hosts_cachix_variables_are_removed_and_the_token_handed_to_the_run_alone() {
        let (store, programs, dir) = (FakeStore::new(), Programs::new(), TempDir::new().unwrap());
        let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
        let target = programs.target_with(&store, &host(&dir));
        let host_token = "the-hosts-own-token";
        let host_key = "the-hosts-own-signing-key";
        let (applied, after) = under_env(
            &[
                ("CACHIX_AUTH_TOKEN", host_token),
                ("CACHIX_SIGNING_KEY", host_key),
            ],
            || {
                let applied = target.apply(&boomdrive(), &version);
                let after = (
                    std::env::var("CACHIX_AUTH_TOKEN").unwrap(),
                    std::env::var("CACHIX_SIGNING_KEY").unwrap(),
                );
                (applied, after)
            },
        );
        assert_eq!(applied, Ok(()));
        assert_eq!(
            cachix_variables(&programs),
            [format!("CACHIX_AUTH_TOKEN={TOKEN}")]
        );
        assert_eq!(
            after,
            (host_token.to_string(), host_key.to_string()),
            "timber's own environment was changed"
        );
    }

    #[test]
    fn the_token_is_in_no_argument() {
        let (store, programs, dir) = (FakeStore::new(), Programs::new(), TempDir::new().unwrap());
        let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
        let target = programs.target_with(&store, &host(&dir));
        assert_eq!(target.apply(&boomdrive(), &version), Ok(()));
        let args = programs.args("cachix").unwrap();
        assert!(!args.iter().any(|a| a.contains(TOKEN)), "{args:?}");
    }

    #[test]
    fn the_config_names_no_file_in_an_empty_folder_under_the_temporary_directory_gone_after() {
        for fails in [false, true] {
            let (store, programs, dir) =
                (FakeStore::new(), Programs::new(), TempDir::new().unwrap());
            let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
            if fails {
                programs.fails("cachix", REFUSED);
            }
            let applied = programs
                .target_with(&store, &host(&dir))
                .apply(&boomdrive(), &version);
            assert_eq!(applied.is_ok(), !fails, "{applied:?}");
            assert_eq!(
                programs.lines("cachix.config"),
                ["none"],
                "a config was made"
            );
            assert_eq!(
                programs.lines("cachix.folder"),
                Vec::<String>::new(),
                "the run's folder held something"
            );
            let folder = folder(&programs);
            assert!(
                folder.starts_with(std::env::temp_dir()),
                "{} is not under the temporary directory",
                folder.display()
            );
            assert!(!folder.exists(), "{} was left behind", folder.display());
        }
    }

    #[test]
    fn a_credential_the_host_lacks_is_unreadable_and_neither_the_store_nix_store_nor_cachix_asked()
    {
        let (store, programs) = (FakeStore::new(), Programs::new());
        let unheld = Verified::new(
            index(9),
            Version::new(VERSION),
            format!("{COMPONENT}-v{VERSION}"),
        );
        let target = programs.target_with(&store, &credentials(&[]));
        let (what, said) = unreadable(target.apply(&boomdrive(), &unheld));
        assert_eq!(
            what,
            "credential cachix-boomdrive of environment cache of swancloud-srv"
        );
        assert_eq!(
            said,
            format!(
                "the bindings file {BINDINGS} names no cachix-boomdrive in its [credentials], \
                 the path of the file holding the token the nix_cache target hands cachix"
            )
        );
        assert_eq!(programs.ran(), Vec::<String>::new());
    }

    #[test]
    fn a_missing_token_file_or_one_of_whitespace_is_unreadable_after_nix_store_and_cachix_never_runs(
    ) {
        let dir = TempDir::new().unwrap();
        let gone = dir.path().join("no-such-file");
        let blank = dir.path().join("blank");
        fs::write(&blank, "\n").unwrap();
        for (path, named, words) in [
            (
                &gone,
                format!("credential cachix-boomdrive at {}", gone.display()),
                "No such file or directory (os error 2)".to_string(),
            ),
            (
                &blank,
                "credential cachix-boomdrive of environment cache of swancloud-srv".to_string(),
                format!(
                    "the file {} holds no token, only whitespace",
                    blank.display()
                ),
            ),
        ] {
            let (store, programs) = (FakeStore::new(), Programs::new());
            let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
            let target = programs.target_with(&store, &credentials(&[(CREDENTIAL, path)]));
            let (what, said) = unreadable(target.apply(&boomdrive(), &version));
            assert_eq!(what, named);
            assert_eq!(said, words);
            assert_eq!(programs.ran(), ["nix-store"]);
        }
    }

    #[test]
    fn a_root_this_host_does_not_hold_is_refused_and_cachix_never_runs() {
        let (store, programs, dir) = (FakeStore::new(), Programs::new(), TempDir::new().unwrap());
        let version = publish(
            &store,
            &[("file", COMPONENT, &closure_of(&[other_root(), root()]))],
        );
        programs.lacks(&[root()]);
        let target = programs.target_with(&store, &host(&dir));
        let said = refused_said(target.apply(&boomdrive(), &version));
        assert!(
            said.starts_with(&format!("{} is not valid in the Nix store of ", root())),
            "{said}"
        );
        assert_eq!(programs.ran(), ["nix-store"]);
    }

    #[test]
    fn cachix_failing_is_unreadable_in_its_own_words() {
        let (store, programs, dir) = (FakeStore::new(), Programs::new(), TempDir::new().unwrap());
        let version = publish(&store, &[("file", COMPONENT, &closure_of(&[root()]))]);
        programs.fails("cachix", &format!("{REFUSED}\n"));
        let target = programs.target_with(&store, &host(&dir));
        let (what, said) = unreadable(target.apply(&boomdrive(), &version));
        assert_eq!(said, REFUSED);
        assert!(
            what.ends_with(&format!("cachix.dhall push boomdrive {}", root())),
            "{what}"
        );
        assert!(!what.contains(TOKEN), "{what}");
    }

    // Requirement: The Cachix token is the host's, read at the moment of
    // use; current reads no credential

    #[test]
    fn current_and_report_are_unread_naming_the_cachix_cache_and_read_no_credential() {
        let (store, programs) = (FakeStore::new(), Programs::new());
        let target = programs.target_with(&store, &credentials(&[]));
        for (what, said) in [
            unreadable(target.current(&boomdrive())),
            unreadable(target.report(&boomdrive())),
        ] {
            assert_eq!(what, "what environment cache of swancloud-srv runs");
            assert_eq!(
                said,
                "the Nix binary cache boomdrive on Cachix holds store paths by their hash and \
                 keeps no marker of what was pushed to it"
            );
        }
        assert_eq!(programs.ran(), Vec::<String>::new());
    }
}

mod group {
    //! Requirement: The Nix cache target on Cachix runs the target group
    //! (93, 119; the Cachix change's design D8).

    use std::cell::Cell;

    use timber_conformance::seams::target::{self, TargetRig};
    use timber_conformance::Rig;

    use super::cachix::host;
    use super::*;

    /// The target group's rig over the Nix cache target naming a Cachix
    /// cache, with the stand-in cachix and nix-store: the target holds no
    /// marker, every environment names the closure the version
    /// published, and its foreign settings add a `token` (119, 45).
    struct CachixRig {
        store: FakeStore,
        programs: Programs,
        target: NixCache,
        _credentials: TempDir,
        environments: Cell<u32>,
    }

    impl CachixRig {
        fn new() -> CachixRig {
            let (store, programs, dir) =
                (FakeStore::new(), Programs::new(), TempDir::new().unwrap());
            let target = programs.target_with(&store, &host(&dir));
            CachixRig {
                store,
                programs,
                target,
                _credentials: dir,
                environments: Cell::new(0),
            }
        }

        fn next(&self) -> Environment {
            self.environments.set(self.environments.get() + 1);
            let mut environment = super::cachix::boomdrive();
            environment.name = EnvironmentName::new(format!("cachix-{}", self.environments.get()));
            environment
        }
    }

    impl Rig for CachixRig {
        fn profile(&self) -> String {
            PROFILE.into()
        }
    }

    impl TargetRig for CachixRig {
        fn target(&self) -> &dyn Target {
            &self.target
        }

        /// The closure's file published into the fake store, whatever
        /// `digest` the suite names: a store holds an index only at its
        /// own digest.
        fn verified(&self, _digest: &Digest) -> Verified {
            publish(&self.store, &[("file", COMPONENT, &closure_of(&[root()]))])
        }

        /// The cache is pushed to, handed the head at desired, so `head`
        /// is left unread.
        fn environment(&self, _head: Option<&Digest>) -> Environment {
            self.next()
        }

        /// No environment of a cache can be read, for the cache keeps no
        /// marker of what was pushed to it.
        fn unreadable_environment(&self) -> (Environment, String) {
            (
                self.next(),
                "the Nix binary cache boomdrive on Cachix holds store paths by their hash and \
                 keeps no marker of what was pushed to it"
                    .into(),
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
    fn the_target_group_passes_over_the_stand_in_cachix_holding_no_marker() {
        let report = timber_conformance::run(target::GROUP, &target::scenarios(), &|| {
            Box::new(CachixRig::new()) as Box<dyn TargetRig>
        });
        report.assert();
        let inapplicable: Vec<&str> = report.not_applicable.iter().map(|n| n.scenario).collect();
        assert_eq!(inapplicable, ["converged and reported"], "{report}");
    }

    #[test]
    fn the_rig_pushes_through_the_stand_in_cachix() {
        let rig = CachixRig::new();
        let version = rig.verified(&index(1));
        assert_eq!(rig.target.apply(&rig.environment(None), &version), Ok(()));
        assert_eq!(rig.programs.ran(), ["nix-store", "cachix"]);
    }
}
