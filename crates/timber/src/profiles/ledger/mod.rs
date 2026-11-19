//! The ledger seam's profiles (B.8), the git ref (C.13) and the store
//! (C.14), and the binding of the one the host's `[ledger]` names.

pub mod git_ref;
pub mod store;

use std::path::{Path, PathBuf};

use serde::Deserialize;
use timber_seams::{Answer, Keeper, RepositoryName, Seam, Source, Store};

use crate::bindings::Table;
use crate::profiles::run::{Tool, GIT_REPOSITORY_ENV};
use git_ref::GitRef;
use store::InStore;

/// What `[ledger] git_ref` is given: the remote holding the refs, where
/// one does (C.13).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GitRefOptions {
    remote: Option<String>,
}

/// What `[ledger] store` is given: nothing, the store being `[store]`'s.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreOptions {}

/// The ledger `[ledger]` names (92, C.13, C.14) for the repository at
/// `repository`: `git_ref`, with its optional remote, the repository
/// holding the refs where it names none; or `store`, the chain held in
/// the store `[store]` names, written or given, through the binding of
/// its own `open` makes, which never makes it but where a publish or a
/// push makes the layout at its first operation, and kept under the
/// repository's common git directory (61, 62). Unreadable naming any
/// other key, or an option its profile does not read.
pub fn bind(
    table: &Table,
    repository: &Path,
    open: &dyn Fn() -> Answer<Box<dyn Store>>,
) -> Answer<Box<dyn Keeper>> {
    match table.key() {
        "git_ref" => {
            let options: GitRefOptions = table.options()?;
            Ok(Box::new(GitRef::new(repository, options.remote)))
        }
        "store" => {
            let StoreOptions {} = table.options()?;
            let kept = common_dir(repository)?.join("timber").join("ledger");
            Ok(Box::new(InStore::new(open()?, kept)))
        }
        _ => Err(table.unknown(&["git_ref", "store"])),
    }
}

/// C.1: the ledger over `from`, a root the repository left, as the host
/// binds its own (design D2): `[ledger]`'s, `table`, or the given where
/// the file names none. The git ref ledger, written or given, is bound as
/// [`bind`] and [`given`] bind it, its refs the repository's, which move
/// with it; the store ledger over `open`'s store, the root left's, its
/// chains kept under `timber/left/<owner>/<name>` in the repository's
/// common git directory, apart from the bound root's (61, 62).
pub fn left(
    table: Option<&Table>,
    repository: &Path,
    remote: Option<String>,
    open: &dyn Fn() -> Answer<Box<dyn Store>>,
    from: &RepositoryName,
) -> Answer<Box<dyn Keeper>> {
    let Some(table) = table else {
        return Ok(given(repository, remote));
    };
    match table.key() {
        "store" => {
            let StoreOptions {} = table.options()?;
            let from = from.lowercased();
            let kept = common_dir(repository)?
                .join("timber")
                .join("left")
                .join(&from.owner)
                .join(&from.name);
            Ok(Box::new(InStore::new(open()?, kept)))
        }
        _ => bind(table, repository, open),
    }
}

/// The ledger a host binds where its file names no `[ledger]` (118,
/// C.13): the git ref over the repository at `repository`, kept on the
/// landing's remote, `remote`, fetched from it before each read and
/// pushed to it on each append as the branch is, so every checkout of
/// the repository reads one chain; and in the repository's own refs,
/// where it keeps its branch, with nothing pushed, where the landing has
/// none.
pub fn given(repository: &Path, remote: Option<String>) -> Box<dyn Keeper> {
    Box::new(GitRef::new(repository, remote))
}

/// The common git directory of the repository at `repository`, which
/// every worktree of it on the host shares and no push carries (62).
fn common_dir(repository: &Path) -> Answer<PathBuf> {
    let git = Tool::new("git", Source::new(Seam::Keeper, "store"));
    let ran = git.output(
        None,
        &[
            "-C",
            &repository.display().to_string(),
            "rev-parse",
            "--path-format=absolute",
            "--git-common-dir",
        ],
        GIT_REPOSITORY_ENV,
        &[],
        &[0],
    )?;
    Ok(PathBuf::from(
        String::from_utf8_lossy(&ran.stdout).trim().to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use tempfile::TempDir;
    use timber_fakes::FakeStore;
    use timber_seams::{
        channel_name, ChannelId, ChannelName, ComponentName, Digest, Envelope, Moved, Promotion,
        Revision, Satisfied, Signature, SignerId, Version, IN_TOTO,
    };

    use super::*;

    /// A repository of one commit in a scratch directory, by the git
    /// command with no configuration of the host's.
    fn repository() -> TempDir {
        let dir = TempDir::new().unwrap();
        for args in [
            &["init", "-q", "-b", "main"][..],
            &["commit", "-q", "--allow-empty", "-m", "start"],
        ] {
            let out = Command::new("git")
                .arg("-C")
                .arg(dir.path())
                .args(args)
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_AUTHOR_NAME", "A. Uthor")
                .env("GIT_AUTHOR_EMAIL", "author@example.com")
                .env("GIT_COMMITTER_NAME", "A. Uthor")
                .env("GIT_COMMITTER_EMAIL", "author@example.com")
                .output()
                .unwrap();
            assert!(out.status.success(), "{out:?}");
        }
        dir
    }

    fn beta() -> ChannelId {
        ChannelId::new(None, ChannelName::new("beta"))
    }

    fn app() -> RepositoryName {
        RepositoryName {
            owner: "Acme".into(),
            name: "App".into(),
        }
    }

    /// `beta`'s first promotion, to the index `index`.
    fn promotion(index: &Digest) -> Envelope {
        let promotion = Promotion {
            channel: ChannelName::new("beta"),
            component: ComponentName::new(timber_seams::REPOSITORY_COMPONENT),
            to: Moved {
                version: Version::new("1.2.0"),
                index: index.clone(),
            },
            from: None,
            predecessor: None,
            asked_by: SignerId::new("gate"),
            policy: Satisfied {
                revision: Revision::new("4f2a9c1"),
                claims: vec![],
            },
            at: 1_793_000_000,
        };
        Envelope::signed(
            IN_TOTO,
            promotion.claim(Revision::new("4f2a9c1")).payload(),
            Signature {
                signer: SignerId::new("gate"),
                keyid: String::new(),
                sig: vec![3],
            },
        )
    }

    #[test]
    fn a_root_lefts_store_ledger_keeps_its_chains_apart() {
        let repo = repository();
        let file = repo.path().join("config.toml");
        let table = Table::given(
            &file,
            "ledger",
            "store",
            toml::Value::Table(Default::default()),
        );
        let store = FakeStore::new();
        let index = store.put_bytes(b"v1.2.0's index").unwrap().digest;
        let head = promotion(&index);
        store.attach(&index, &head).unwrap();
        store.name(&channel_name(&beta()), &index, None).unwrap();
        let open = || -> Answer<Box<dyn Store>> { Ok(Box::new(store.clone())) };

        let left = left(Some(&table), repo.path(), None, &open, &app()).unwrap();
        assert_eq!(left.chain(&beta()).unwrap().claims, vec![head]);

        let common = repo.path().join(".git").canonicalize().unwrap();
        let kept = common.join("timber/left/acme/app/beta.json");
        assert!(kept.is_file(), "{} was not kept", kept.display());
        assert!(
            !common.join("timber/ledger").exists(),
            "the bound root's kept chains were written"
        );
    }

    #[test]
    fn a_root_lefts_git_ref_ledger_reads_the_repositorys_refs() {
        let repo = repository();
        let index = Digest::of(b"v1.2.0's index");
        let head = promotion(&index);
        let bound = given(repo.path(), None);
        bound.append(&beta(), &head, None).unwrap();
        let open = || -> Answer<Box<dyn Store>> { panic!("a git ref ledger opened a store") };

        let left = left(None, repo.path(), None, &open, &app()).unwrap();
        assert_eq!(left.chain(&beta()).unwrap().claims, vec![head.clone()]);

        let file = repo.path().join("config.toml");
        let written = Table::given(
            &file,
            "ledger",
            "git_ref",
            toml::Value::Table(Default::default()),
        );
        let left = super::left(Some(&written), repo.path(), None, &open, &app()).unwrap();
        assert_eq!(left.chain(&beta()).unwrap().claims, vec![head]);
    }
}
