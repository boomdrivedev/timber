//! The repository's history, which the core reads itself through git
//! (section 9): the files of a landed revision's tree, the commits that
//! touch a component's paths, a commit's message and the commit as the
//! version rule's tool reads it, and the tags that carry its prefix, read
//! from the local history and the shared one (21 to 23, 65, 123, 134;
//! design D3, D6); and the record of the roots the repository published
//! under, `refs/timber/root` beside the tags, read and written as the
//! tags are claimed (22, 62, C.1).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use git2::{ErrorCode, Object, ObjectType, Oid, Repository};
use git_cliff_core::commit::Commit as CliffCommit;
use glob::Pattern;
use regex::Regex;
use semver::Version;
use timber_seams::{
    Answer, Branch, Not, PushedRef, RepositoryName, Revision, SharedHistory, Source, Why,
};

use crate::{GitCommand, Landed};

/// What the history's source names when the history read is the
/// repository's own, not a remote's.
pub const LOCAL: &str = "local";

/// A repository's history, opened at one path, and the shared history
/// its remotes are reached as.
pub struct History {
    path: PathBuf,
    git: Repository,
    cliff: git_cliff_core::repo::Repository,
    shared: Box<dyn SharedHistory>,
}

/// A commit: its revision and its message as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    pub revision: Revision,
    pub message: String,
}

/// A tag carrying a component's prefix: its name, the semantic version
/// after the prefix, and the revision it names, peeled to the commit.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag {
    pub name: String,
    pub version: Version,
    pub revision: Revision,
}

/// 123: whether a component's `paths` hold a path of the repository,
/// relative to its root, matched as git-cliff's own path filter matches
/// a commit's changed files: a pattern ending `/` holding everything
/// under it and a leading `./` dropped (ADR 0010). Unreadable naming the
/// pattern with the parser's words where one is no glob.
pub fn paths_hold(paths: &[String]) -> Answer<impl Fn(&str) -> bool> {
    let patterns = paths
        .iter()
        .map(|path| {
            let path = match path.strip_suffix('/') {
                Some(dir) => format!("{dir}/**"),
                None => path.clone(),
            };
            let path = path.strip_prefix("./").unwrap_or(&path);
            Pattern::new(path).map_err(|e| {
                Not::unreadable(&local(), format!("the paths {path}"), Some(e.to_string()))
            })
        })
        .collect::<Answer<Vec<Pattern>>>()?;
    Ok(move |path: &str| patterns.iter().any(|p| p.matches_path(Path::new(path))))
}

fn local() -> Source {
    Source::history(LOCAL)
}

fn words(error: &git2::Error) -> Option<String> {
    Some(error.message().to_string())
}

impl History {
    /// The repository at `path`, a working tree or a bare repository,
    /// with no search of its parents, its remotes reached with the git
    /// command; unreadable naming the path with git's words when it is
    /// none.
    pub fn open(path: impl AsRef<Path>) -> Answer<History> {
        History::reaching(path, Box::new(GitCommand))
    }

    /// The repository at `path`, as [`History::open`] opens it, its
    /// remotes reached as `shared`.
    pub fn reaching(path: impl AsRef<Path>, shared: Box<dyn SharedHistory>) -> Answer<History> {
        let path = path.as_ref().to_path_buf();
        let what = format!("the repository at {}", path.display());
        let git = Repository::open(&path)
            .map_err(|e| Not::unreadable(&local(), what.clone(), words(&e)))?;
        let cliff = git_cliff_core::repo::Repository::init(path.clone())
            .map_err(|e| Not::unreadable(&local(), what, Some(e.to_string())))?;
        Ok(History {
            path,
            git,
            cliff,
            shared,
        })
    }

    /// The commit the landed revision names; unreadable when this
    /// repository does not hold it.
    fn commit(&self, revision: &Revision) -> Answer<git2::Commit<'_>> {
        self.git
            .find_commit_by_prefix(revision.as_str())
            .map_err(|e| Not::unreadable(&local(), format!("the revision {revision}"), words(&e)))
    }

    /// The landed revision by its full git id, as tags name revisions.
    pub fn revision(&self, landed: &Landed) -> Answer<Revision> {
        Ok(Revision::new(
            self.commit(landed.revision())?.id().to_string(),
        ))
    }

    /// The file at `path` in the landed revision's tree, never the working
    /// tree's (65); absent when the tree holds nothing there.
    pub fn blob(&self, landed: &Landed, path: &str) -> Answer<Vec<u8>> {
        self.file(landed.revision(), path)
    }

    /// The file at `path` in `revision`'s tree, never the working tree's
    /// (65); absent when the tree holds nothing there.
    pub(crate) fn file(&self, revision: &Revision, path: &str) -> Answer<Vec<u8>> {
        let what = format!("{path} at {revision}");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), what.clone(), words(&e));
        let tree = self.commit(revision)?.tree().map_err(unreadable)?;
        let entry = match tree.get_path(Path::new(path)) {
            Ok(entry) => entry,
            Err(e) if e.code() == ErrorCode::NotFound => {
                return Err(Not::absent(&local(), what));
            }
            Err(e) => return Err(unreadable(e)),
        };
        let object = entry.to_object(&self.git).map_err(unreadable)?;
        match object.into_blob() {
            Ok(blob) => Ok(blob.content().to_vec()),
            Err(_) => Err(Not::unreadable(
                &local(),
                what.clone(),
                Some(format!("{path} is a tree, not a file")),
            )),
        }
    }

    /// The commits the landed revision reaches, after `since` where it is
    /// given, that change a file matching one of `paths`, matched by
    /// git-cliff's own path filter (123, ADR 0010).
    pub fn commits(
        &self,
        landed: &Landed,
        since: Option<&Revision>,
        paths: &[String],
    ) -> Answer<Vec<Commit>> {
        let revision = self.commit(landed.revision())?.id();
        let range = match since {
            Some(since) => format!("{}..{revision}", self.commit(since)?.id()),
            None => revision.to_string(),
        };
        let what = format!("the commits of {range}");
        let patterns = paths
            .iter()
            .map(|p| Pattern::new(p))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Not::unreadable(&local(), what.clone(), Some(e.to_string())))?;
        let commits = self
            .cliff
            .commits(Some(&range), Some(patterns), None, false)
            .map_err(|e| Not::unreadable(&local(), what, Some(e.to_string())))?;
        Ok(commits
            .iter()
            .map(|c| Commit {
                revision: Revision::new(c.id().to_string()),
                message: String::from_utf8_lossy(c.message_bytes()).into_owned(),
            })
            .collect())
    }

    /// 134: the message of the commit `revision` names, as written, read
    /// from the local history alone and never a remote; absent naming the
    /// revision when the history holds no such commit (58; design D7).
    pub fn message(&self, revision: &Revision) -> Answer<String> {
        let commit = self.held(revision)?;
        Ok(String::from_utf8_lossy(commit.message_bytes()).into_owned())
    }

    /// 134: the commit `revision` names as the version rule's tool reads
    /// one, its message, author, committer and whether it merges, from the
    /// local history alone; absent naming the revision when the history
    /// holds no such commit (58).
    pub(crate) fn cliff_commit(&self, revision: &Revision) -> Answer<CliffCommit<'static>> {
        Ok(CliffCommit::from(&self.held(revision)?))
    }

    /// The commit `revision` names in full, from the local history alone;
    /// absent naming it when the history holds no such commit.
    fn held(&self, revision: &Revision) -> Answer<git2::Commit<'_>> {
        let what = format!("the commit {revision}");
        let absent = || Not::absent(&local(), what.clone());
        let oid = Oid::from_str(revision.as_str()).map_err(|_| absent())?;
        match self.git.find_commit(oid) {
            Ok(commit) => Ok(commit),
            Err(e) if e.code() == ErrorCode::NotFound => Err(absent()),
            Err(e) => Err(Not::unreadable(&local(), what.clone(), words(&e))),
        }
    }

    /// Every tag carrying `prefix` then a semantic version, in the local
    /// history and the shared one, that names the landed revision or a
    /// revision it reaches (22, 123). A tag only the shared history holds
    /// is read with `git ls-remote`, which writes nothing; a shared
    /// history that cannot be reached makes the read unreadable, never
    /// one of the local history alone.
    pub fn tags(&self, landed: &Landed, prefix: &str) -> Answer<Vec<Tag>> {
        let pattern = Regex::new(&format!("^{}(.+)$", regex::escape(prefix))).map_err(|e| {
            Not::unreadable(
                &local(),
                format!("the prefix {prefix}"),
                Some(e.to_string()),
            )
        })?;
        let version = |name: &str| {
            pattern
                .captures(name)
                .and_then(|c| Version::parse(&c[1]).ok())
        };
        let what = format!("the tags carrying {prefix}");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), what.clone(), words(&e));

        let mut found = BTreeSet::new();
        let names = self.git.tag_names(None).map_err(unreadable)?;
        for name in &names {
            // A name that is not UTF-8 carries no prefix the policy declares.
            let Some(name) = name.map_err(unreadable)? else {
                continue;
            };
            let Some(v) = version(name) else { continue };
            let object = self
                .git
                .revparse_single(&format!("refs/tags/{name}"))
                .map_err(unreadable)?;
            if let Some(oid) = peeled(object).map_err(unreadable)? {
                found.insert((name.to_string(), v, oid));
            }
        }
        if let Some(remote) = self.remote(landed.branch())? {
            for (name, oid) in self.remote_tags(&remote, &format!("refs/tags/{prefix}*"))? {
                if let Some(v) = version(&name) {
                    found.insert((name, v, oid));
                }
            }
        }

        let landed = self.commit(landed.revision())?.id();
        let mut tags = Vec::new();
        for (name, version, oid) in found {
            if self.reaches(landed, oid).map_err(unreadable)? {
                tags.push(Tag {
                    name,
                    version,
                    revision: Revision::new(oid.to_string()),
                });
            }
        }
        Ok(tags)
    }

    /// Whether `from` is `to` or a descendant of it; a commit this
    /// repository does not hold is one no revision it holds reaches.
    fn reaches(&self, from: Oid, to: Oid) -> Result<bool, git2::Error> {
        if from == to {
            return Ok(true);
        }
        match self.git.find_commit(to) {
            Ok(_) => self.git.graph_descendant_of(from, to),
            Err(e) if e.code() == ErrorCode::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// 22: the remote that holds the history every publisher of `branch`
    /// shares, as `branch.<name>.remote` names it; none when it names no
    /// remote, and the local history is then the whole of it.
    pub fn remote(&self, branch: &Branch) -> Answer<Option<String>> {
        let key = format!("branch.{branch}.remote");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), key.clone(), words(&e));
        match self.git.config().map_err(unreadable)?.get_string(&key) {
            Ok(remote) if remote == "." => Ok(None),
            Ok(remote) => Ok(Some(remote)),
            Err(e) if e.code() == ErrorCode::NotFound => Ok(None),
            Err(e) => Err(unreadable(e)),
        }
    }

    /// 114: the address `remote.<remote>.url` names in the repository's
    /// git configuration, as written, no `insteadOf` applied; none where
    /// it names no such remote.
    pub fn remote_url(&self, remote: &str) -> Answer<Option<String>> {
        let key = format!("remote.{remote}.url");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), key.clone(), words(&e));
        match self.git.config().map_err(unreadable)?.get_string(&key) {
            Ok(url) => Ok(Some(url)),
            Err(e) if e.code() == ErrorCode::NotFound => Ok(None),
            Err(e) => Err(unreadable(e)),
        }
    }

    /// The tags `remote` holds matching `pattern`, each peeled to the
    /// revision it names.
    fn remote_tags(&self, remote: &str, pattern: &str) -> Answer<Vec<(String, Oid)>> {
        let what = format!("the tags of {remote}");
        let listed = self.shared.list(&self.path, remote, pattern, &what)?;
        let mut tags: Vec<(String, Oid)> = Vec::new();
        for (name, oid) in &listed {
            let Some(name) = name.strip_prefix("refs/tags/") else {
                continue;
            };
            let oid = Oid::from_str(oid)
                .map_err(|e| Not::unreadable(&Source::history(remote), what.clone(), words(&e)))?;
            match name.strip_suffix("^{}") {
                Some(name) => match tags.iter_mut().find(|(n, _)| n == name) {
                    Some(tag) => tag.1 = oid,
                    None => tags.push((name.to_string(), oid)),
                },
                None if !tags.iter().any(|(n, _)| n == name) => tags.push((name.to_string(), oid)),
                None => {}
            }
        }
        Ok(tags)
    }

    /// The revision the local tag `name` names, peeled, or none when the
    /// local history holds no such tag.
    pub(crate) fn tag_at(&self, name: &str) -> Answer<Option<Revision>> {
        let what = format!("the tag {name}");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), what.clone(), words(&e));
        match self.git.revparse_single(&format!("refs/tags/{name}")) {
            Ok(object) => Ok(peeled(object)
                .map_err(unreadable)?
                .map(|oid| Revision::new(oid.to_string()))),
            Err(e) if e.code() == ErrorCode::NotFound => Ok(None),
            Err(e) => Err(unreadable(e)),
        }
    }

    /// A lightweight tag `name` at the landed revision in the local
    /// history, never forced over one already there (rule core.6).
    pub(crate) fn make_tag(&self, name: &str, landed: &Landed) -> Answer<()> {
        let what = format!("the tag {name}");
        let commit = self.commit(landed.revision())?;
        self.git
            .tag_lightweight(name, commit.as_object(), false)
            .map(|_| ())
            .map_err(|e| Not::unreadable(&local(), what, words(&e)))
    }

    /// Removes the local tag `name`, made by a claim the shared history
    /// refused.
    pub(crate) fn drop_tag(&self, name: &str) -> Answer<()> {
        self.git
            .tag_delete(name)
            .map_err(|e| Not::unreadable(&local(), format!("the tag {name}"), words(&e)))
    }

    /// Pushes the local tag `name` to `remote` without force, as the
    /// shared history is reached (rule core.21).
    pub(crate) fn push_tag(&self, remote: &str, name: &str) -> Answer<PushedRef> {
        let what = format!("{remote}, pushing {name}");
        let tag = format!("refs/tags/{name}");
        self.shared.push(&self.path, remote, &tag, &tag, &what)
    }

    /// The revision the tag `name` names in `remote`'s history, peeled,
    /// or none when it holds no such tag.
    pub(crate) fn remote_tag(&self, remote: &str, name: &str) -> Answer<Option<Revision>> {
        let refname = format!("refs/tags/{name}");
        Ok(self
            .remote_tags(remote, &refname)?
            .into_iter()
            .find(|(n, _)| n == name)
            .map(|(_, oid)| Revision::new(oid.to_string())))
    }
}

/// C.1, 22, 62: the ref of the record of claimed versions that names
/// each root the repository published under, beside the version tags in
/// the shared history (design D1).
pub const ROOT_REF: &str = "refs/timber/root";

/// The one file of each commit of [`ROOT_REF`]'s chain, naming its root.
const ROOT_FILE: &str = "root";

/// The record's roots as one read found them: the commit at the chain's
/// tip, none where nothing holds the ref, and each root from the first.
struct RootRecord {
    tip: Option<Oid>,
    roots: Vec<RepositoryName>,
}

impl History {
    /// C.1, 22, 62: the roots the record of claimed versions names, from
    /// the first, the last being where the repository's history is
    /// (design D1). Where the landed branch names a remote, its
    /// [`ROOT_REF`] fetched into the repository's copy, which only moves
    /// forward: a remote holding less than the copy is unreadable naming
    /// the commits it lost, and one that cannot be reached is unreadable
    /// with git's words, never the copy alone (rule core.47). Where it
    /// names none, the repository's own ref; the empty list where neither
    /// holds it.
    pub fn roots(&self, landed: &Landed) -> Answer<Vec<RepositoryName>> {
        Ok(self.root_record(landed)?.roots)
    }

    /// C.1: `root`, lowercased, recorded after the record as read: a
    /// commit naming it on the chain's tip, pushed to the remote without
    /// force and the copy moved to it after, by the git command, so the
    /// host's own credential helper answers (rule core.21). A push refused
    /// is read again: done where the record's last root is now `root`,
    /// refused with git's words otherwise, and never retried (rule
    /// core.6). With no remote, the own ref moved from the commit it was
    /// read at.
    pub fn record_root(&self, landed: &Landed, root: &RepositoryName) -> Answer<()> {
        let root = root.lowercased();
        let read = self.root_record(landed)?;
        let commit = self.root_commit(landed, &root, read.tip)?;
        let Some(remote) = self.remote(landed.branch())? else {
            return match self.move_root(commit, read.tip) {
                Ok(()) => Ok(()),
                Err(e) => self.recorded_after_all(landed, &root, local(), e.message().into()),
            };
        };
        match self.push_root(&remote, commit)? {
            PushedRef::Held => {
                // The remote holds it; the copy follows, and a copy moved
                // meanwhile is brought forward by the next read.
                let _ = self.move_root(commit, read.tip);
                Ok(())
            }
            PushedRef::Rejected(words) => {
                self.recorded_after_all(landed, &root, Source::history(&remote), words)
            }
        }
    }

    /// A write of `root` refused, for `words`, read again: done where the
    /// record's last root is now `root`, refused with the words otherwise.
    fn recorded_after_all(
        &self,
        landed: &Landed,
        root: &RepositoryName,
        source: Source,
        words: String,
    ) -> Answer<()> {
        let again = self.root_record(landed)?;
        if again.roots.last().map(RepositoryName::lowercased).as_ref() == Some(root) {
            return Ok(());
        }
        Err(Not::refused(&source, Why::Said, Some(words)))
    }

    /// The record as read now: the remote's where the landed branch names
    /// one, fetched into the copy, the repository's own ref otherwise.
    fn root_record(&self, landed: &Landed) -> Answer<RootRecord> {
        let copy = self.root_copy()?;
        let Some(remote) = self.remote(landed.branch())? else {
            return Ok(RootRecord {
                tip: copy,
                roots: self.walk_roots(copy, &local())?,
            });
        };
        let at = Source::history(&remote);
        let what = format!("the record of roots on {remote}");
        let listed = self
            .shared
            .list(&self.path, &remote, ROOT_REF, &what)?
            .into_iter()
            .find(|(name, _)| name == ROOT_REF)
            .map(|(_, oid)| Oid::from_str(&oid))
            .transpose()
            .map_err(|e| Not::unreadable(&at, what.clone(), words(&e)))?;
        let Some(theirs) = listed else {
            return match copy {
                None => Ok(RootRecord {
                    tip: None,
                    roots: Vec::new(),
                }),
                Some(copy) => Err(self.lost(&remote, copy, None)),
            };
        };
        self.shared.fetch(&self.path, &remote, ROOT_REF, &what)?;
        let unreadable = |e: git2::Error| Not::unreadable(&local(), ROOT_REF, words(&e));
        match copy {
            Some(copy) if copy == theirs => {}
            Some(copy) if self.reaches(theirs, copy).map_err(unreadable)? => {
                self.move_root(theirs, Some(copy)).map_err(unreadable)?
            }
            Some(copy) => return Err(self.lost(&remote, copy, Some(theirs))),
            None => self.move_root(theirs, None).map_err(unreadable)?,
        }
        Ok(RootRecord {
            tip: Some(theirs),
            roots: self.walk_roots(Some(theirs), &at)?,
        })
    }

    /// The commit the repository's copy of [`ROOT_REF`] names, none where
    /// it holds no such ref.
    fn root_copy(&self) -> Answer<Option<Oid>> {
        match self.git.refname_to_id(ROOT_REF) {
            Ok(oid) => Ok(Some(oid)),
            Err(e) if e.code() == ErrorCode::NotFound => Ok(None),
            Err(e) => Err(Not::unreadable(&local(), ROOT_REF, words(&e))),
        }
    }

    /// The copy moved to `to` from `from`, or made where `from` is none,
    /// never over a ref moved meanwhile (rule core.47).
    fn move_root(&self, to: Oid, from: Option<Oid>) -> Result<(), git2::Error> {
        let message = "timber: the record of roots";
        match from {
            Some(from) => self
                .git
                .reference_matching(ROOT_REF, to, true, from, message)
                .map(|_| ()),
            None => self.git.reference(ROOT_REF, to, false, message).map(|_| ()),
        }
    }

    /// 122, rule core.47: `remote` no longer holds what the copy read
    /// from it, the commits from `copy` that `theirs` does not reach.
    fn lost(&self, remote: &str, copy: Oid, theirs: Option<Oid>) -> Not {
        let mut lost = Vec::new();
        let walked = self.git.revwalk().and_then(|mut walk| {
            walk.push(copy)?;
            if let Some(theirs) = theirs {
                walk.hide(theirs)?;
            }
            walk.collect::<Result<Vec<Oid>, _>>()
        });
        match walked {
            Ok(oids) => lost.extend(oids.into_iter().rev().map(|o| o.to_string())),
            Err(_) => lost.push(copy.to_string()),
        }
        Not::unreadable(
            &Source::history(remote),
            format!(
                "the record of roots on {remote}, which no longer holds {}",
                lost.join(", ")
            ),
            Some(format!(
                "{remote}'s {ROOT_REF} does not descend from what this repository read"
            )),
        )
    }

    /// 121, design D1: each root the chain at `tip` names, from the first;
    /// unreadable naming the first commit with two parents, or naming no
    /// root, never answered in part.
    fn walk_roots(&self, tip: Option<Oid>, at: &Source) -> Answer<Vec<RepositoryName>> {
        let mut roots = Vec::new();
        let mut next = tip;
        while let Some(oid) = next {
            let what = format!("the record of roots at {oid}");
            let unreadable = |e: git2::Error| Not::unreadable(at, what.clone(), words(&e));
            let commit = self.git.find_commit(oid).map_err(unreadable)?;
            if commit.parent_count() > 1 {
                return Err(Not::unreadable(
                    at,
                    what,
                    Some(format!("{oid} has more than one parent")),
                ));
            }
            let named = commit
                .tree()
                .map_err(unreadable)?
                .get_name(ROOT_FILE)
                .and_then(|entry| entry.to_object(&self.git).ok())
                .and_then(|object| object.into_blob().ok())
                .and_then(|blob| root_named(blob.content()));
            let Some(root) = named else {
                return Err(Not::unreadable(
                    at,
                    what,
                    Some(format!("{oid} holds no {ROOT_FILE} naming a root")),
                ));
            };
            roots.push(root);
            next = commit.parent_ids().next();
        }
        roots.reverse();
        Ok(roots)
    }

    /// Design D1: the commit naming `root` on `parent`, its tree the one
    /// file [`ROOT_FILE`], written by timber at the landed revision's
    /// time, so no index or working tree is touched and the clock is not
    /// read.
    fn root_commit(
        &self,
        landed: &Landed,
        root: &RepositoryName,
        parent: Option<Oid>,
    ) -> Answer<Oid> {
        let what = format!("the record of roots, naming {root}");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), what.clone(), words(&e));
        let text = format!("{root}\n");
        let blob = self.git.blob(text.as_bytes()).map_err(unreadable)?;
        let mut tree = self.git.treebuilder(None).map_err(unreadable)?;
        tree.insert(ROOT_FILE, blob, 0o100644).map_err(unreadable)?;
        let tree = self
            .git
            .find_tree(tree.write().map_err(unreadable)?)
            .map_err(unreadable)?;
        let when = self.commit(landed.revision())?.committer().when();
        let timber =
            git2::Signature::new("timber", "timber@localhost", &when).map_err(unreadable)?;
        let parent = parent
            .map(|p| self.git.find_commit(p))
            .transpose()
            .map_err(unreadable)?;
        let parents: Vec<&git2::Commit<'_>> = parent.iter().collect();
        self.git
            .commit(None, &timber, &timber, &text, &tree, &parents)
            .map_err(unreadable)
    }

    /// The record's next commit pushed to `remote` without force, as the
    /// shared history is reached (rule core.21).
    fn push_root(&self, remote: &str, commit: Oid) -> Answer<PushedRef> {
        let what = format!("{remote}, pushing {ROOT_REF}");
        self.shared
            .push(&self.path, remote, &commit.to_string(), ROOT_REF, &what)
    }
}

/// The root a commit's [`ROOT_FILE`] names: `<owner>/<name>` and a
/// newline, neither empty; `None` for anything else.
fn root_named(bytes: &[u8]) -> Option<RepositoryName> {
    let text = std::str::from_utf8(bytes).ok()?.strip_suffix('\n')?;
    let (owner, name) = text.split_once('/')?;
    if owner.is_empty() || name.is_empty() || name.contains('/') {
        return None;
    }
    Some(RepositoryName {
        owner: owner.into(),
        name: name.into(),
    })
}

/// The commit a tag's object names, through any annotated tags; none
/// when it names a tree or a file.
fn peeled(mut object: Object<'_>) -> Result<Option<Oid>, git2::Error> {
    loop {
        match object.kind() {
            Some(ObjectType::Commit) => return Ok(Some(object.id())),
            Some(ObjectType::Tag) => {
                object = object
                    .into_tag()
                    .map_err(|_| git2::Error::from_str("a tag that is not a tag"))?
                    .target()?
            }
            _ => return Ok(None),
        }
    }
}

impl History {
    /// The commit the local branch `branch` names, by its full id; absent
    /// when the repository holds no such branch.
    pub fn head(&self, branch: &Branch) -> Answer<Revision> {
        let what = format!("branch {branch}");
        match self
            .git
            .find_branch(branch.as_str(), git2::BranchType::Local)
        {
            Ok(found) => found
                .get()
                .peel_to_commit()
                .map(|c| Revision::new(c.id().to_string()))
                .map_err(|e| Not::unreadable(&local(), what, words(&e))),
            Err(e) if e.code() == ErrorCode::NotFound => Err(Not::absent(&local(), what)),
            Err(e) => Err(Not::unreadable(&local(), what, words(&e))),
        }
    }

    /// The newest revision of the local branch `branch` that the
    /// checkout's head descends from, their common ancestor, by its full
    /// id (design D4); absent naming the branch when the repository holds
    /// no such branch or the two share no history.
    pub fn base(&self, branch: &Branch) -> Answer<Revision> {
        let tip = self.head(branch)?;
        let what = format!("a revision of branch {branch} that HEAD descends from");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), what.clone(), words(&e));
        let head = self
            .git
            .head()
            .and_then(|h| h.peel_to_commit())
            .map_err(unreadable)?;
        let tip = Oid::from_str(tip.as_str()).map_err(unreadable)?;
        match self.git.merge_base(head.id(), tip) {
            Ok(base) => Ok(Revision::new(base.to_string())),
            Err(e) if e.code() == ErrorCode::NotFound => Err(Not::absent(&local(), what)),
            Err(e) => Err(unreadable(e)),
        }
    }

    /// The commit `text` names, a full or short id, a branch, a tag or
    /// any revision git reads, by its full id, so every act records the
    /// revision it acted on (rule core.4); absent with git's words when it
    /// names no commit.
    pub fn resolve(&self, text: &str) -> Answer<Revision> {
        let absent =
            |e: git2::Error| Not::absent(&local(), format!("commit named {text}: {}", e.message()));
        let object = match self.git.revparse_single(text) {
            Ok(object) => object,
            Err(e)
                if matches!(
                    e.code(),
                    ErrorCode::NotFound | ErrorCode::Ambiguous | ErrorCode::InvalidSpec
                ) =>
            {
                return Err(absent(e));
            }
            Err(e) => {
                return Err(Not::unreadable(
                    &local(),
                    format!("the revision {text}"),
                    words(&e),
                ))
            }
        };
        object
            .peel_to_commit()
            .map(|c| Revision::new(c.id().to_string()))
            .map_err(absent)
    }

    /// The root of the working tree this history was opened at, absolute;
    /// none for a bare repository.
    pub fn workdir(&self) -> Option<PathBuf> {
        let dir = self.git.workdir()?;
        Some(dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf()))
    }

    /// The repository's common git directory, which every worktree of it
    /// on the host shares and no push carries (62).
    pub fn common_dir(&self) -> PathBuf {
        let dir = self.git.commondir();
        dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf())
    }

    /// The git directory of the worktree this history was opened at, its
    /// own and no other worktree's, which no push carries (50, 62): the
    /// common directory for the main worktree, and the one git keeps for
    /// a linked worktree under it.
    pub fn git_dir(&self) -> PathBuf {
        let dir = self.git.path();
        dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf())
    }
}
