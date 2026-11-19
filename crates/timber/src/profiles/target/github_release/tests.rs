//! The GitHub release target over a scripted GitHub: the token read at
//! use, its reads, its refusals, its applies and the target group (design
//! D11).

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::{json, Value};
use tempfile::TempDir;
use timber_fakes::FakeStore;
use timber_seams::{
    Artifact, ArtifactMeta, ChannelId, ChannelName, ComponentName, Ecosystem, EcosystemName,
    Envelope, EnvironmentName, Held as Holds, Manifest, Name, Put, Version,
};

use super::scripted::{Asked, Held, Scripted, CLOCK};
use super::*;

const COMPONENT: &str = "flume";
const CREDENTIAL: &str = "flume-releases";
const TOKEN: &str = "ghp_0123456789secret";
const REVISION: &str = "4f2a9c1d8e0b7a6c5f4e3d2c1b0a998877665544";
const LINUX: &str = "flume-x86_64-linux.tar.gz";
const DARWIN: &str = "flume-aarch64-darwin.tar.gz";
const BINDINGS: &str = "/home/a/.config/timber/config.toml";

fn repository() -> RepositoryName {
    RepositoryName {
        owner: "boomdrivedev".into(),
        name: "flume".into(),
    }
}

fn environment(name: &str, settings: Value) -> Environment {
    Environment {
        name: EnvironmentName::new(name),
        channel: ChannelId::new(
            Some(ComponentName::new(COMPONENT)),
            ChannelName::new("stable"),
        ),
        waits: false,
        settings,
    }
}

/// The table attaching the outputs `assets`.
fn table(assets: &[&str]) -> Value {
    json!({ "credential": CREDENTIAL, "assets": assets })
}

/// `github`, whose release carries the Linux and Darwin binaries.
fn github() -> Environment {
    environment("github", table(&[LINUX, DARWIN]))
}

/// The tag version `version` of the component was claimed under.
fn tag(version: &str) -> String {
    format!("v{version}")
}

/// The notes of version `version`.
fn notes(version: &str) -> String {
    format!("## {version}\n\n### Features\n\n- flume grows\n")
}

/// The bytes of the output `output` at version `version`.
fn binary(output: &str, version: &str) -> Vec<u8> {
    format!("{output} at {version}").into_bytes()
}

/// A store that answers a read of a digest it was told stands for another
/// with what that other holds, so the target group can hand the target a
/// version at a digest of its own choosing, as the core hands one after
/// its verify.
#[derive(Clone, Default)]
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
    fn read(&self, digest: &Digest) -> Answer<Holds> {
        let to = self.aliases.borrow().get(digest).cloned();
        self.store.read(to.as_ref().unwrap_or(digest))
    }
    fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>> {
        self.store.versions(ecosystem, name)
    }
}

/// What a test publishes of a version.
struct Publishing<'a> {
    version: &'a str,
    component: &'a str,
    files: Vec<(&'a str, &'a str, Vec<u8>)>,
    notes: Option<Vec<u8>>,
    revision: Option<&'a str>,
    repository: Option<RepositoryName>,
}

impl<'a> Publishing<'a> {
    /// Version `version` of the component as its publish makes it: the
    /// Linux and Darwin binaries as file artifacts, its notes, the
    /// revision and the checkout's repository.
    fn of(version: &'a str) -> Publishing<'a> {
        Publishing {
            version,
            component: COMPONENT,
            files: vec![
                ("file", LINUX, binary(LINUX, version)),
                ("file", DARWIN, binary(DARWIN, version)),
            ],
            notes: Some(notes(version).into_bytes()),
            revision: Some(REVISION),
            repository: Some(repository()),
        }
    }
}

/// A folder holding the token's file, a store, a scripted GitHub and the
/// target over them.
struct World {
    /// Holds the token's file for the world's life.
    _dir: TempDir,
    store: Aliased,
    github: Scripted,
    target: GithubRelease,
}

impl World {
    fn new() -> World {
        let dir = TempDir::new().unwrap();
        let token = dir.path().join("flume-releases");
        fs::write(&token, format!("{TOKEN}\n")).unwrap();
        World::in_dir(dir, Ok(repository()), &credentials(&[(CREDENTIAL, &token)]))
    }

    /// The world with the checkout's repository `repository` and the
    /// host's `credentials`.
    fn of(repository: Answer<RepositoryName>, credentials: &Credentials) -> World {
        World::in_dir(TempDir::new().unwrap(), repository, credentials)
    }

    fn in_dir(
        dir: TempDir,
        repository: Answer<RepositoryName>,
        credentials: &Credentials,
    ) -> World {
        let store = Aliased::default();
        let github = Scripted::new(&GithubRelease::source());
        let target = GithubRelease::with_backend(
            Box::new(store.clone()),
            repository,
            credentials,
            Box::new(github.clone()),
        );
        World {
            _dir: dir,
            store,
            github,
            target,
        }
    }

    /// `publishing` put into the store as a publish puts a version: each
    /// file and the notes an artifact of one layer, the index naming them,
    /// and the version's name at the index; and the version verified.
    fn publish(&self, publishing: Publishing<'_>) -> Verified {
        let store = &self.store.store;
        let version = Version::new(publishing.version);
        let notes = publishing
            .notes
            .map(|bytes| ("notes", publishing.component, bytes));
        let mut artifacts = Vec::new();
        for (ecosystem, name, bytes) in publishing.files.into_iter().chain(notes) {
            let layer = store.put_bytes(&bytes).unwrap().digest;
            let meta = ArtifactMeta::new(
                Ecosystem::new(ecosystem),
                EcosystemName::new(name),
                version.clone(),
            )
            .unwrap();
            let manifest = Manifest::Artifact {
                meta: meta.clone(),
                layers: vec![layer],
            };
            let digest = store.put_artifact(&manifest).unwrap().digest;
            artifacts.push(Artifact { digest, meta });
        }
        let component = ComponentName::new(publishing.component);
        let index = IndexMeta::new(component.clone(), version.clone(), artifacts)
            .unwrap()
            .with(
                publishing.revision.map(Revision::new),
                publishing.repository,
            );
        let digest = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
        let tag = tag(publishing.version);
        store
            .name(&version_name(&component, &tag), &digest, None)
            .unwrap();
        Verified::new(digest, version, tag)
    }

    /// Version `version` of the component published, and GitHub holding
    /// its tag at its revision, as its publish pushed it.
    fn version(&self, version: &str) -> Verified {
        let verified = self.publish(Publishing::of(version));
        self.github.holds_tag(verified.tag(), REVISION);
        verified
    }
}

/// The host's credentials, each name the path of its file.
fn credentials(paths: &[(&str, &Path)]) -> Credentials {
    Credentials {
        file: PathBuf::from(BINDINGS),
        paths: paths
            .iter()
            .map(|(name, path)| (name.to_string(), path.to_path_buf()))
            .collect(),
    }
}

/// The words of a refusal by the target.
fn refused_said<T: std::fmt::Debug>(answer: Answer<T>) -> String {
    match answer {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, GithubRelease::source());
            assert_eq!(r.why, Why::Said);
            r.said.expect("the target's words")
        }
        other => panic!("answered {other:?}, not refused"),
    }
}

/// What an unreadable answer by the target names, and its words.
fn unreadable<T: std::fmt::Debug>(answer: Answer<T>) -> (String, String) {
    match answer {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source, GithubRelease::source());
            (u.what, u.said.expect("the words"))
        }
        other => panic!("answered {other:?}, not unreadable"),
    }
}

/// An answer's words, whatever it is.
fn words<T: std::fmt::Debug>(answer: &Answer<T>) -> String {
    format!("{answer:?}")
}

mod credential {
    //! Requirement: The credential is the host's, read at the moment of
    //! use.

    use super::*;

    /// Every operation that reads the credential answered unreadable, with
    /// nothing asked of GitHub; what each named and said.
    fn each_unreadable(world: &World) -> Vec<(String, String)> {
        let version = world.version("0.31.0");
        let answers = vec![
            unreadable(world.target.apply(&github(), &version)),
            unreadable(world.target.current(&github())),
            unreadable(world.target.report(&github())),
        ];
        assert_eq!(world.github.ran(), Vec::<&str>::new(), "gh was asked");
        answers
    }

    #[test]
    fn a_credential_the_host_does_not_name_is_unreadable_naming_it_and_the_bindings_file() {
        let world = World::of(Ok(repository()), &credentials(&[]));
        for (what, said) in each_unreadable(&world) {
            assert_eq!(
                what,
                "credential flume-releases of environment github of flume"
            );
            assert!(
                said.starts_with(&format!(
                    "the bindings file {BINDINGS} names no flume-releases in its [credentials]"
                )),
                "{said}"
            );
        }
    }

    #[test]
    fn a_file_that_cannot_be_read_is_unreadable_naming_the_credential_and_the_path() {
        let dir = TempDir::new().unwrap();
        let gone = dir.path().join("no-such-file");
        let world = World::of(Ok(repository()), &credentials(&[(CREDENTIAL, &gone)]));
        for (what, said) in each_unreadable(&world) {
            assert_eq!(
                what,
                format!("credential flume-releases at {}", gone.display())
            );
            assert_eq!(said, "No such file or directory (os error 2)");
        }
    }

    #[test]
    fn a_file_of_whitespace_alone_is_unreadable_naming_the_credential() {
        let dir = TempDir::new().unwrap();
        for held in ["", "\n", " \t\n\n"] {
            let file = dir.path().join("flume-releases");
            fs::write(&file, held).unwrap();
            let world = World::of(Ok(repository()), &credentials(&[(CREDENTIAL, &file)]));
            for (what, said) in each_unreadable(&world) {
                assert_eq!(
                    what,
                    "credential flume-releases of environment github of flume"
                );
                assert_eq!(
                    said,
                    format!(
                        "the file {} holds no token, only whitespace",
                        file.display()
                    )
                );
            }
        }
    }

    #[test]
    fn the_token_is_handed_with_its_trailing_newline_trimmed() {
        let world = World::new();
        let version = world.version("0.31.0");
        world.target.apply(&github(), &version).unwrap();
        world.target.current(&github()).unwrap();
        world.target.report(&github()).unwrap();
        let tokens = world.github.tokens();
        assert_eq!(tokens.len(), 7, "{:?}", world.github.ran());
        assert!(tokens.iter().all(|t| t == TOKEN), "{tokens:?}");
    }

    #[test]
    fn desired_reads_no_credential() {
        let world = World::of(Ok(repository()), &credentials(&[]));
        let head = Digest::of(b"head");
        assert_eq!(
            world.target.desired(&github(), &|| Ok(head.clone())),
            Ok(head)
        );
        assert_eq!(world.github.ran(), Vec::<&str>::new());
    }

    #[test]
    fn the_token_is_in_no_answers_words() {
        let world = World::new();
        let version = world.version("0.31.0");
        let words_of_gh = "HTTP 403: Resource not accessible by personal access token";
        world.github.fails("create", words_of_gh);
        let applied = world.target.apply(&github(), &version);
        let (_, said) = unreadable(world.target.apply(&github(), &version));
        assert_eq!(said, words_of_gh);
        let current = world.target.current(&github());
        let report = world.target.report(&github());
        for answer in [words(&applied), words(&current), words(&report)] {
            assert!(!answer.contains(TOKEN), "{answer}");
        }
        assert!(!format!("{:?}", world.github.asked()).contains(TOKEN));
    }
}

mod reads {
    //! Requirements: desired is the channel's head; current is the
    //! version of the release GitHub marks latest; current is unread where
    //! no release names a version; report is the latest release's version
    //! and when GitHub updated it.

    use super::*;

    /// `version` applied and its release marked latest.
    fn converged(world: &World, version: &str) -> Verified {
        let verified = world.version(version);
        world.target.apply(&github(), &verified).unwrap();
        verified
    }

    #[test]
    fn desired_is_the_head_and_an_empty_channel_is_absent_with_nothing_asked() {
        let world = World::new();
        let head = Digest::of(b"head");
        assert_eq!(
            world.target.desired(&github(), &|| Ok(head.clone())),
            Ok(head)
        );
        let empty = || {
            Err(Not::absent(
                &Source::new(Seam::Keeper, "the test"),
                "the head of channel stable of flume",
            ))
        };
        assert!(matches!(
            world.target.desired(&github(), &empty),
            Err(Not::Absent(_))
        ));
        let foreign = environment("github", json!({ "token": TOKEN }));
        let (what, _) = unreadable(world.target.desired(&foreign, &|| Ok(Digest::of(b"x"))));
        assert_eq!(what, "setting token of environment github of flume");
        assert_eq!(world.github.ran(), Vec::<&str>::new());
    }

    #[test]
    fn no_release_marked_latest_is_unread_naming_the_repository() {
        let world = World::new();
        world.version("0.31.0");
        for (what, said) in [
            unreadable(world.target.current(&github())),
            unreadable(world.target.report(&github())),
        ] {
            assert_eq!(what, "what environment github of flume runs");
            assert_eq!(
                said,
                "github.com/boomdrivedev/flume marks no release latest, so no version is \
                 released there"
            );
        }
        assert_eq!(world.github.ran(), ["latest", "latest"]);
    }

    #[test]
    fn a_latest_release_whose_tag_names_no_version_is_unread_naming_the_tag() {
        let world = World::new();
        world.version("0.31.0");
        world.github.holds_tag("nightly", REVISION);
        world
            .github
            .holds_release("nightly", Held::of("nightly", "made by hand", false));
        world.github.marks_latest("nightly");
        for answer in [
            unreadable(world.target.current(&github())),
            unreadable(world.target.report(&github())),
        ] {
            let (what, said) = answer;
            assert_eq!(what, "what environment github of flume runs");
            assert!(
                said.starts_with(
                    "the release github.com/boomdrivedev/flume marks latest is of the tag \
                     nightly, which names no version of flume in the store"
                ),
                "{said}"
            );
        }
    }

    #[test]
    fn another_components_version_under_the_same_tag_is_not_taken() {
        let world = World::new();
        let theirs = world.publish(Publishing {
            component: "sluice",
            ..Publishing::of("0.31.0")
        });
        world.github.holds_tag(theirs.tag(), REVISION);
        world
            .github
            .holds_release(theirs.tag(), Held::of(theirs.tag(), "sluice's", false));
        world.github.marks_latest(theirs.tag());
        let (_, said) = unreadable(world.target.current(&github()));
        assert!(
            said.contains("of the tag v0.31.0, which names no version of flume"),
            "{said}"
        );
    }

    #[test]
    fn a_converged_release_is_current_and_reported_with_when_github_updated_it() {
        let world = World::new();
        let verified = converged(&world, "0.31.0");
        assert_eq!(
            world.target.current(&github()),
            Ok(verified.index().clone())
        );
        assert_eq!(
            world.target.report(&github()),
            Ok(Running {
                digest: verified.index().clone(),
                converged_at: CLOCK.into(),
            })
        );
    }

    #[test]
    fn a_release_with_no_updated_at_is_current_and_its_report_unread() {
        let world = World::new();
        let verified = world.version("0.31.0");
        let mut held = Held::of(verified.tag(), &notes("0.31.0"), false);
        held.updated_at = None;
        world.github.holds_release(verified.tag(), held);
        world.github.marks_latest(verified.tag());
        assert_eq!(
            world.target.current(&github()),
            Ok(verified.index().clone())
        );
        let (what, said) = unreadable(world.target.report(&github()));
        assert_eq!(what, "when environment github of flume converged");
        assert!(
            said.starts_with(
                "GitHub answered the latest release, of the tag v0.31.0, with no updated_at"
            ),
            "{said}"
        );
    }

    #[test]
    fn an_unread_checkout_repository_is_the_answer_with_nothing_asked() {
        let reason = Not::unreadable(
            &Source::new(Seam::Store, "the checkout"),
            "the repository of this checkout",
            Some("its remote origin names no owner and name".into()),
        );
        let world = World::of(Err(reason.clone()), &credentials(&[]));
        let version = world.version("0.31.0");
        assert_eq!(world.target.current(&github()), Err(reason.clone()));
        assert_eq!(
            world.target.report(&github()),
            Err::<Running, _>(reason.clone())
        );
        assert_eq!(world.target.apply(&github(), &version), Err(reason));
        assert_eq!(world.github.ran(), Vec::<&str>::new());
    }
}

mod refusals {
    //! Requirements: The release is the checkout's repository's; Apply
    //! reads the notes and the assets by digest; Apply releases only the
    //! tag GitHub holds at the version's revision.

    use super::*;

    /// The refusal `publishing`'s apply answers, once nothing but `ran`
    /// was asked of GitHub.
    fn refused_after(world: &World, verified: &Verified, ran: &[&str]) -> String {
        let said = refused_said(world.target.apply(&github(), verified));
        assert_eq!(world.github.ran(), ran, "{said}");
        said
    }

    #[test]
    fn another_repositorys_index_is_refused_naming_both() {
        let world = World::new();
        let sluice = RepositoryName {
            owner: "boomdrivedev".into(),
            name: "sluice".into(),
        };
        let verified = world.publish(Publishing {
            repository: Some(sluice),
            ..Publishing::of("0.31.0")
        });
        let said = refused_after(&world, &verified, &[]);
        assert!(
            said.contains("names the publishing repository boomdrivedev/sluice, and this checkout is of boomdrivedev/flume"),
            "{said}"
        );
        let none = world.publish(Publishing {
            repository: None,
            ..Publishing::of("0.32.0")
        });
        let said = refused_after(&world, &none, &[]);
        assert!(said.contains("names no publishing repository"), "{said}");
    }

    #[test]
    fn an_index_naming_no_revision_is_refused() {
        let world = World::new();
        let verified = world.publish(Publishing {
            revision: None,
            ..Publishing::of("0.31.0")
        });
        let said = refused_after(&world, &verified, &[]);
        assert!(
            said.contains(
                "of version 0.31.0 names no revision, so the tag v0.31.0 cannot be checked"
            ),
            "{said}"
        );
    }

    #[test]
    fn a_version_published_before_notes_is_refused_naming_its_notes() {
        let world = World::new();
        let verified = world.publish(Publishing {
            notes: None,
            ..Publishing::of("0.31.0")
        });
        let said = refused_after(&world, &verified, &[]);
        assert_eq!(
            said,
            "version 0.31.0 of flume names no notes artifact of flume, so it carries no notes"
        );
    }

    #[test]
    fn notes_that_are_not_utf8_are_refused() {
        let world = World::new();
        let verified = world.publish(Publishing {
            notes: Some(vec![b'#', 0xff, b'\n']),
            ..Publishing::of("0.31.0")
        });
        let said = refused_after(&world, &verified, &[]);
        assert!(
            said.starts_with("the notes of version 0.31.0 are not UTF-8"),
            "{said}"
        );
    }

    #[test]
    fn an_asset_the_version_does_not_carry_is_refused_naming_the_output_and_the_version() {
        let world = World::new();
        let verified = world.publish(Publishing {
            files: vec![("file", LINUX, binary(LINUX, "0.31.0"))],
            ..Publishing::of("0.31.0")
        });
        let said = refused_after(&world, &verified, &[]);
        assert_eq!(
            said,
            format!(
                "version 0.31.0 of flume names no file artifact of the output {DARWIN}, so it \
                 carries no asset of it"
            )
        );
        let image = world.publish(Publishing {
            files: vec![
                ("file", LINUX, binary(LINUX, "0.32.0")),
                ("image", DARWIN, binary(DARWIN, "0.32.0")),
            ],
            ..Publishing::of("0.32.0")
        });
        assert!(refused_after(&world, &image, &[]).contains(DARWIN));
    }

    #[test]
    fn a_tag_github_does_not_hold_is_refused_naming_it_and_the_repository() {
        let world = World::new();
        let verified = world.publish(Publishing::of("0.31.0"));
        let said = refused_after(&world, &verified, &["tag"]);
        assert!(
            said.starts_with("github.com/boomdrivedev/flume holds no tag v0.31.0"),
            "{said}"
        );
        assert_eq!(world.github.asked(), [Asked::Tag("v0.31.0".into())]);
    }

    #[test]
    fn a_tag_at_another_revision_is_refused_naming_it_and_both_revisions() {
        let world = World::new();
        let verified = world.publish(Publishing::of("0.31.0"));
        let other = "9e8d7c6b5a4f3e2d1c0b9a8f7e6d5c4b3a2f1e0d";
        world.github.holds_tag("v0.31.0", other);
        let said = refused_after(&world, &verified, &["tag"]);
        assert_eq!(
            said,
            format!(
                "github.com/boomdrivedev/flume holds the tag v0.31.0 at {other}, and the index \
                 of version 0.31.0 names the revision {REVISION}"
            )
        );
        assert!(world.github.release_of("v0.31.0").is_none());
    }
}

mod applies {
    //! Requirements: Apply makes a release the tag lacks as a draft;
    //! Apply attaches each asset the release does not hold; Apply
    //! publishes the release last, marked latest; GitHub's words are the
    //! answer.

    use super::*;

    fn upload(tag: &str, assets: &[&str]) -> Asked {
        Asked::Upload {
            tag: tag.into(),
            assets: assets.iter().map(|a| a.to_string()).collect(),
        }
    }

    fn publish(tag: &str, version: &str) -> Asked {
        Asked::Publish {
            tag: tag.into(),
            notes: notes(version),
        }
    }

    /// The release of `tag` holding each named asset of `version`'s
    /// bytes, published or a draft.
    fn holding(tag: &str, version: &str, draft: bool) -> Held {
        Held::of(tag, &notes(version), draft)
            .with(LINUX, &binary(LINUX, version))
            .with(DARWIN, &binary(DARWIN, version))
    }

    #[test]
    fn a_first_apply_makes_a_draft_uploads_its_assets_and_publishes_it_latest() {
        let world = World::new();
        let verified = world.version("0.31.0");
        assert_eq!(world.target.apply(&github(), &verified), Ok(()));
        assert_eq!(
            world.github.asked(),
            [
                Asked::Tag("v0.31.0".into()),
                Asked::Release("v0.31.0".into()),
                Asked::Create {
                    tag: "v0.31.0".into(),
                    notes: notes("0.31.0"),
                },
                upload("v0.31.0", &[LINUX, DARWIN]),
                publish("v0.31.0", "0.31.0"),
            ]
        );
        let release = world.github.release_of("v0.31.0").unwrap();
        assert!(!release.draft && !release.prerelease, "{release:?}");
        assert_eq!(release.title, "v0.31.0");
        assert_eq!(release.notes, notes("0.31.0"));
        assert_eq!(release, holding("v0.31.0", "0.31.0", false).updated(CLOCK));
        assert_eq!(world.github.latest_tag().as_deref(), Some("v0.31.0"));
    }

    #[test]
    fn a_draft_an_earlier_apply_left_is_finished_with_no_second_create() {
        let world = World::new();
        let verified = world.version("0.31.0");
        world.github.holds_release(
            "v0.31.0",
            Held::of("v0.31.0", &notes("0.31.0"), true).with(LINUX, &binary(LINUX, "0.31.0")),
        );
        assert_eq!(world.target.apply(&github(), &verified), Ok(()));
        assert_eq!(world.github.ran(), ["tag", "release", "upload", "publish"]);
        assert_eq!(world.github.asked()[2], upload("v0.31.0", &[DARWIN]));
        let release = world.github.release_of("v0.31.0").unwrap();
        assert_eq!(release, holding("v0.31.0", "0.31.0", false).updated(CLOCK));
    }

    #[test]
    fn every_asset_held_uploads_nothing() {
        let world = World::new();
        let verified = world.version("0.31.0");
        world
            .github
            .holds_release("v0.31.0", holding("v0.31.0", "0.31.0", false));
        assert_eq!(world.target.apply(&github(), &verified), Ok(()));
        assert_eq!(world.github.ran(), ["tag", "release", "publish"]);
    }

    #[test]
    fn only_the_assets_not_held_are_uploaded_and_none_is_removed() {
        let world = World::new();
        let verified = world.version("0.31.0");
        let mut held = Held::of("v0.31.0", &notes("0.31.0"), false)
            .with(LINUX, b"other bytes")
            .with(DARWIN, &binary(DARWIN, "0.31.0"))
            .with("checksums.txt", b"made by hand");
        held.assets.get_mut(DARWIN).unwrap().state = "starter".into();
        world.github.holds_release("v0.31.0", held);
        assert_eq!(world.target.apply(&github(), &verified), Ok(()));
        assert_eq!(world.github.ran(), ["tag", "release", "upload", "publish"]);
        assert_eq!(world.github.asked()[2], upload("v0.31.0", &[LINUX, DARWIN]));
        let release = world.github.release_of("v0.31.0").unwrap();
        assert_eq!(
            release,
            holding("v0.31.0", "0.31.0", false)
                .with("checksums.txt", b"made by hand")
                .updated(CLOCK)
        );
    }

    #[test]
    fn an_asset_github_answers_with_no_digest_is_uploaded_again() {
        let world = World::new();
        let verified = world.version("0.31.0");
        let mut held = holding("v0.31.0", "0.31.0", false);
        held.assets.get_mut(LINUX).unwrap().digest = None;
        world.github.holds_release("v0.31.0", held);
        assert_eq!(world.target.apply(&github(), &verified), Ok(()));
        assert_eq!(world.github.asked()[2], upload("v0.31.0", &[LINUX]));
    }

    #[test]
    fn a_release_of_notes_alone_uploads_nothing() {
        let world = World::new();
        let verified = world.version("0.31.0");
        let notes_alone = environment("github", table(&[]));
        assert_eq!(world.target.apply(&notes_alone, &verified), Ok(()));
        assert_eq!(world.github.ran(), ["tag", "release", "create", "publish"]);
        let release = world.github.release_of("v0.31.0").unwrap();
        assert!(release.assets.is_empty(), "{release:?}");
    }

    #[test]
    fn a_move_back_marks_the_earlier_release_latest_again_with_its_notes() {
        let world = World::new();
        let earlier = world.version("0.30.0");
        let later = world.version("0.31.0");
        world.target.apply(&github(), &earlier).unwrap();
        world.target.apply(&github(), &later).unwrap();
        assert_eq!(world.target.current(&github()), Ok(later.index().clone()));
        let mut edited = world.github.release_of("v0.30.0").unwrap();
        edited.notes = "edited by hand".into();
        world.github.holds_release("v0.30.0", edited);
        let before = world.github.asked().len();
        world.target.apply(&github(), &earlier).unwrap();
        assert_eq!(
            world.github.asked()[before..],
            [
                Asked::Tag("v0.30.0".into()),
                Asked::Release("v0.30.0".into()),
                publish("v0.30.0", "0.30.0"),
            ]
        );
        assert_eq!(world.github.latest_tag().as_deref(), Some("v0.30.0"));
        assert_eq!(
            world.github.release_of("v0.30.0").unwrap().notes,
            notes("0.30.0")
        );
        assert_eq!(world.target.current(&github()), Ok(earlier.index().clone()));
    }

    #[test]
    fn a_failure_at_each_step_is_the_answer_in_ghs_words_and_nothing_runs_after() {
        let words = "HTTP 403: Resource not accessible by personal access token (https://api.github.com/repos/boomdrivedev/flume/releases)";
        for (step, ran) in [
            ("release", &["tag", "release"][..]),
            ("create", &["tag", "release", "create"][..]),
            ("upload", &["tag", "release", "create", "upload"][..]),
            (
                "publish",
                &["tag", "release", "create", "upload", "publish"][..],
            ),
        ] {
            let world = World::new();
            let verified = world.version("0.31.0");
            world.github.fails(step, words);
            let (_, said) = unreadable(world.target.apply(&github(), &verified));
            assert_eq!(said, words, "{step}");
            assert_eq!(world.github.ran(), ran, "{step}");
            assert_ne!(
                world.github.latest_tag().as_deref(),
                Some("v0.31.0"),
                "{step}"
            );
        }
    }
}

mod group {
    //! Requirement: The profile passes the target group.

    use std::cell::Cell;

    use timber_conformance::seams::target::{self, TargetRig};
    use timber_conformance::Rig;

    use super::*;

    /// The token GitHub refuses every operation of, and its words.
    const REFUSED: &str = "ghp_revoked";
    const WORDS: &str = "HTTP 401: Bad credentials (https://api.github.com/repos/boomdrivedev/flume/releases/latest)";

    /// The target group's rig over the profile and the scripted GitHub:
    /// the target holds a marker, the release GitHub marks latest; each
    /// environment is one of its own over the one repository, the
    /// unreadable one naming a credential whose token GitHub refuses, and
    /// the foreign settings add a `token` (119, 45; design D11).
    struct GithubRig {
        world: World,
        environments: Cell<u32>,
    }

    impl GithubRig {
        fn new() -> GithubRig {
            let dir = TempDir::new().unwrap();
            let token = dir.path().join("flume-releases");
            fs::write(&token, format!("{TOKEN}\n")).unwrap();
            let revoked = dir.path().join("revoked");
            fs::write(&revoked, REFUSED).unwrap();
            let credentials = credentials(&[(CREDENTIAL, &token), ("revoked", &revoked)]);
            let world = World::in_dir(dir, Ok(repository()), &credentials);
            world.github.refuses(REFUSED, WORDS);
            GithubRig {
                world,
                environments: Cell::new(0),
            }
        }

        fn next(&self, settings: Value) -> Environment {
            self.environments.set(self.environments.get() + 1);
            environment(
                &format!("environment-{}", self.environments.get()),
                settings,
            )
        }
    }

    impl Rig for GithubRig {
        fn profile(&self) -> String {
            PROFILE.into()
        }
    }

    impl TargetRig for GithubRig {
        fn target(&self) -> &dyn Target {
            &self.world.target
        }

        /// A version published, its tag held by GitHub at its revision,
        /// and read at `digest` as a store holds an index at its own, the
        /// store resolving its version's name to `digest`: the suite
        /// stands in the core's place, which makes a `Verified` after its
        /// verify (42).
        fn verified(&self, digest: &Digest) -> Verified {
            let published = self.world.version("1.4.0");
            let store = &self.world.store;
            store
                .aliases
                .borrow_mut()
                .insert(digest.clone(), published.index().clone());
            let name = version_name(&ComponentName::new(COMPONENT), published.tag());
            store.name(&name, digest, Some(published.index())).unwrap();
            Verified::new(digest.clone(), published.version().clone(), published.tag())
        }

        /// The target is pushed to, handed the head at desired, so `head`
        /// is left unread.
        fn environment(&self, _head: Option<&Digest>) -> Environment {
            self.next(table(&[LINUX, DARWIN]))
        }

        fn unreadable_environment(&self) -> (Environment, String) {
            let settings = json!({ "credential": "revoked", "assets": [LINUX] });
            (self.next(settings), WORDS.into())
        }

        fn foreign_settings(&self) -> (Environment, String) {
            let mut environment = self.next(table(&[LINUX]));
            environment.settings["token"] = json!(TOKEN);
            (environment, "token".into())
        }
    }

    #[test]
    fn the_target_group_passes_over_the_scripted_github_holding_a_marker() {
        let report = timber_conformance::run(target::GROUP, &target::scenarios(), &|| {
            Box::new(GithubRig::new()) as Box<dyn TargetRig>
        });
        report.assert();
        let inapplicable: Vec<&str> = report.not_applicable.iter().map(|n| n.scenario).collect();
        assert_eq!(
            inapplicable,
            ["applied where no marker is held"],
            "{report}"
        );
    }
}
