//! The shared history at a path on the host, reached in the test's own
//! process with libgit2, answering as the git command answers for it
//! (design D7): the refs it lists, an annotated tag's commit again under
//! `^{}`; a ref pushed without force, refused where the shared history
//! holds a tag of that name elsewhere or the ref cannot move forward to
//! it; and a ref's objects fetched, with no ref written. It starts no
//! process.

use std::io::Write;
use std::path::Path;

use git2::{ErrorCode, ObjectType, Oid, Repository};
use timber_seams::{Answer, Not, PushedRef, SharedHistory, Source};

/// The shared history every remote names, reached in process.
#[derive(Clone, Copy, Debug, Default)]
pub struct FakeSharedHistory;

/// Unreadable naming `remote` and `what`, with libgit2's words.
fn unreadable(remote: &str, what: &str, e: &git2::Error) -> Not {
    Not::unreadable(
        &Source::history(remote),
        what,
        Some(e.message().to_string()),
    )
}

/// The repository at `repository`, and the shared history its remote
/// `remote` names: the address `remote.<remote>.url` gives, or `remote`
/// itself where the repository names no such remote.
fn opened(repository: &Path, remote: &str, what: &str) -> Answer<(Repository, Repository)> {
    let ours = Repository::open(repository).map_err(|e| unreadable(remote, what, &e))?;
    let url = match ours.find_remote(remote) {
        Ok(named) => named.url().unwrap_or(remote).to_string(),
        Err(_) => remote.to_string(),
    };
    let theirs = Repository::open(&url).map_err(|e| unreadable(remote, what, &e))?;
    Ok((ours, theirs))
}

/// The object `oid` and every object it reaches, copied from `from` into
/// `to`: a tag itself, and the commit it names with its history.
fn copied(from: &Repository, to: &Repository, oid: Oid) -> Result<(), git2::Error> {
    let mut pack = from.packbuilder()?;
    let mut object = from.find_object(oid, None)?;
    while object.kind() == Some(ObjectType::Tag) {
        pack.insert_object(object.id(), None)?;
        object = object
            .into_tag()
            .map_err(|_| git2::Error::from_str("a tag that is not a tag"))?
            .target()?;
    }
    let mut walk = from.revwalk()?;
    walk.push(object.peel_to_commit()?.id())?;
    pack.insert_walk(&mut walk)?;
    let mut bytes = git2::Buf::new();
    pack.write_buf(&mut bytes)?;
    // git makes the pack folder where a copy of a repository lacks it;
    // libgit2's writer does not.
    std::fs::create_dir_all(to.path().join("objects").join("pack"))
        .map_err(|e| git2::Error::from_str(&e.to_string()))?;
    let odb = to.odb()?;
    let mut writer = odb.packwriter()?;
    writer
        .write_all(&bytes)
        .map_err(|e| git2::Error::from_str(&e.to_string()))?;
    writer.commit()?;
    Ok(())
}

impl SharedHistory for FakeSharedHistory {
    fn list(
        &self,
        repository: &Path,
        remote: &str,
        pattern: &str,
        what: &str,
    ) -> Answer<Vec<(String, String)>> {
        let (_, theirs) = opened(repository, remote, what)?;
        let named = |name: &str| match pattern.strip_suffix('*') {
            Some(prefix) => name.starts_with(prefix),
            None => name == pattern,
        };
        let mut listed = Vec::new();
        for reference in theirs
            .references()
            .map_err(|e| unreadable(remote, what, &e))?
        {
            let reference = reference.map_err(|e| unreadable(remote, what, &e))?;
            let (Ok(name), Some(target)) = (reference.name(), reference.target()) else {
                continue;
            };
            if !named(name) {
                continue;
            }
            listed.push((name.to_string(), target.to_string()));
            let mut object = theirs
                .find_object(target, None)
                .map_err(|e| unreadable(remote, what, &e))?;
            if object.kind() == Some(ObjectType::Tag) {
                while let Ok(tag) = object.clone().into_tag() {
                    object = tag.target().map_err(|e| unreadable(remote, what, &e))?;
                }
                listed.push((format!("{name}^{{}}"), object.id().to_string()));
            }
        }
        listed.sort();
        Ok(listed)
    }

    fn push(
        &self,
        repository: &Path,
        remote: &str,
        from: &str,
        to: &str,
        what: &str,
    ) -> Answer<PushedRef> {
        let (ours, theirs) = opened(repository, remote, what)?;
        let oid = ours
            .revparse_single(from)
            .map_err(|e| unreadable(remote, what, &e))?
            .id();
        let refused = |why: &str| {
            Ok(PushedRef::Rejected(format!(
                "!\t{from}:{to}\t[rejected] ({why})\n"
            )))
        };
        match theirs.refname_to_id(to) {
            Ok(at) if at == oid => return Ok(PushedRef::Held),
            Ok(_) if to.starts_with("refs/tags/") => return refused("already exists"),
            Ok(at) if ours.find_commit(at).is_err() => return refused("fetch first"),
            Ok(at) => {
                let forward = ours
                    .graph_descendant_of(oid, at)
                    .map_err(|e| unreadable(remote, what, &e))?;
                if !forward {
                    return refused("non-fast-forward");
                }
            }
            Err(e) if e.code() == ErrorCode::NotFound => {}
            Err(e) => return Err(unreadable(remote, what, &e)),
        }
        copied(&ours, &theirs, oid).map_err(|e| unreadable(remote, what, &e))?;
        theirs
            .reference(to, oid, true, "push")
            .map_err(|e| unreadable(remote, what, &e))?;
        Ok(PushedRef::Held)
    }

    fn fetch(&self, repository: &Path, remote: &str, name: &str, what: &str) -> Answer<()> {
        let (ours, theirs) = opened(repository, remote, what)?;
        let oid = theirs
            .refname_to_id(name)
            .map_err(|e| unreadable(remote, what, &e))?;
        copied(&theirs, &ours, oid).map_err(|e| unreadable(remote, what, &e))
    }
}
