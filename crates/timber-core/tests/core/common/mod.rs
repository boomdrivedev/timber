//! Repositories in a temporary directory, made, committed to, tagged,
//! branched, pushed and read in the test's own process with libgit2, by
//! a fixed author, for the core's tests over real history (design D8).
//! No test of the core runs the git command but the one contract test
//! holding the fake shared history to it.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use git2::{IndexAddOption, Oid, Repository, RepositoryInitOptions, Signature};

use tempfile::TempDir;
use timber_core::{History, Landed};
use timber_fakes::{FakeLanding, FakeSharedHistory};
use timber_seams::{Branch, Revision};

/// A directory holding the repositories of one test.
pub struct Fixture {
    root: TempDir,
}

/// A repository's own git-cliff template, as its `cliff.toml` holds it:
/// a heading naming the tag, the last release tag where there is one,
/// then each group that has commits, one line per commit with its
/// subject, `(breaking)` where it is one, its seven-character revision
/// and its author.
pub const CLIFF_TOML: &str = r#"[changelog]
body = '''
## {{ version }}
{%- if previous.version %}

Since {{ previous.version }}.
{%- endif %}
{%- for group in commits | commit_groups(groups=commit_parsers_groups) %}

### {{ group.group }}
{% for commit in group.commits %}
- {{ commit.message | split(pat="\n") | first | trim }}{% if commit.breaking %} (breaking){% endif %} ({{ commit.id | truncate(length=7, end="") }}) by {{ commit.author.name }}
{%- endfor %}
{%- endfor %}
'''

[git]
filter_unconventional = false
commit_parsers = [
  { message = "^feat", group = "Features" },
  { message = "^fix", group = "Fixes" },
  { message = ".*", group = "Other" },
]
"#;

/// One repository of a fixture: a working tree, or a bare one standing
/// for the shared history.
#[derive(Clone, Debug)]
pub struct Repo {
    pub path: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        Fixture {
            root: TempDir::new().unwrap(),
        }
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }

    /// A new repository with a working tree, on `main`.
    pub fn repo(&self, name: &str) -> Repo {
        let path = self.path(name);
        Repository::init_opts(&path, RepositoryInitOptions::new().initial_head("main")).unwrap();
        Repo { path }
    }

    /// A new bare repository, the history every publisher shares.
    pub fn bare(&self, name: &str) -> Repo {
        let path = self.path(name);
        Repository::init_opts(
            &path,
            RepositoryInitOptions::new().bare(true).initial_head("main"),
        )
        .unwrap();
        Repo { path }
    }

    /// A clone of `from` named `name`, its `main` following `origin`.
    pub fn clone_of(&self, from: &Repo, name: &str) -> Repo {
        let path = self.path(name);
        let cloned = git2::build::RepoBuilder::new()
            .clone(from.path.to_str().unwrap(), &path)
            .unwrap();
        // A clone of an empty repository is on no branch and sets no
        // upstream; git's, of one whose HEAD names `main`, is on `main`.
        cloned.set_head("refs/heads/main").unwrap();
        let mut config = cloned.config().unwrap();
        config.set_str("branch.main.remote", "origin").unwrap();
        config
            .set_str("branch.main.merge", "refs/heads/main")
            .unwrap();
        Repo { path }
    }
}

/// The fixed author, committing now.
fn author() -> Signature<'static> {
    Signature::now("A. Uthor", "author@example.com").unwrap()
}

/// `message` as `git commit -m` and `git tag -m` keep it: trailing
/// whitespace and surrounding blank lines stripped, one newline at its
/// end.
fn cleaned(message: &str) -> String {
    git2::message_prettify(message, None).unwrap()
}

impl Repo {
    fn open(&self) -> Repository {
        Repository::open(&self.path).unwrap()
    }

    /// A commit with `message` writing each file, and its revision.
    pub fn commit(&self, message: &str, files: &[(&str, &str)]) -> Revision {
        let repo = self.open();
        let mut index = repo.index().unwrap();
        for (file, contents) in files {
            let at = self.path.join(file);
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
        let sig = author();
        let made = repo
            .commit(Some("HEAD"), &sig, &sig, &cleaned(message), &tree, &parents)
            .unwrap();
        Revision::new(made.to_string())
    }

    /// A commit with `message` changing one file under each of `paths`.
    pub fn touch(&self, message: &str, paths: &[&str]) -> Revision {
        let files: Vec<(String, String)> = paths
            .iter()
            .map(|p| (format!("{p}/touched"), message.to_string()))
            .collect();
        let files: Vec<(&str, &str)> = files
            .iter()
            .map(|(f, c)| (f.as_str(), c.as_str()))
            .collect();
        self.commit(message, &files)
    }

    pub fn head(&self) -> Revision {
        let repo = self.open();
        let head = repo.head().unwrap().peel_to_commit().unwrap().id();
        Revision::new(head.to_string())
    }

    /// A lightweight tag `name` at `at`.
    pub fn tag(&self, name: &str, at: &Revision) {
        let repo = self.open();
        let target = repo
            .find_object(Oid::from_str(at.as_str()).unwrap(), None)
            .unwrap();
        repo.tag_lightweight(name, &target, false).unwrap();
    }

    /// An annotated tag `name` at `at`.
    pub fn annotated(&self, name: &str, at: &Revision) {
        let repo = self.open();
        let target = repo
            .find_object(Oid::from_str(at.as_str()).unwrap(), None)
            .unwrap();
        repo.tag(name, &target, &author(), &cleaned(name), false)
            .unwrap();
    }

    /// `refs` pushed to `origin`, each to the ref of its own name.
    pub fn push(&self, refs: &[&str]) {
        let repo = self.open();
        let specs: Vec<String> = refs.iter().map(|r| format!("{r}:{r}")).collect();
        repo.find_remote("origin")
            .unwrap()
            .push(&specs, None)
            .unwrap();
    }

    /// The revision the tag `name` names, peeled, or none.
    pub fn tag_at(&self, name: &str) -> Option<String> {
        let repo = self.open();
        let found = repo.revparse_single(&format!("refs/tags/{name}^{{commit}}"));
        found.ok().map(|commit| commit.id().to_string())
    }

    /// The day `at` was committed, in UTC, as a template's date filter
    /// prints its time.
    pub fn day(&self, at: &Revision) -> String {
        let repo = self.open();
        let commit = repo
            .find_commit(Oid::from_str(at.as_str()).unwrap())
            .unwrap();
        let seconds = u64::try_from(commit.committer().when().seconds()).unwrap();
        timber_seams::utc(seconds)[..10].to_string()
    }

    /// The repository's history, its shared history reached in the
    /// test's own process.
    pub fn history(&self) -> History {
        History::reaching(&self.path, std::sync::Arc::new(FakeSharedHistory)).unwrap()
    }

    /// The branch `name` checked out, its tree written over the working
    /// tree and the index, as `git checkout` does.
    pub fn checkout(&self, name: &str) {
        let repo = self.open();
        let branch = format!("refs/heads/{name}");
        let tree = repo.revparse_single(&branch).unwrap();
        repo.checkout_tree(&tree, Some(git2::build::CheckoutBuilder::new().force()))
            .unwrap();
        repo.set_head(&branch).unwrap();
    }

    /// A branch `name` made at `at`, or at HEAD where none is named, and
    /// checked out, as `git checkout -b` does.
    pub fn branch(&self, name: &str, at: Option<&Revision>) {
        let repo = self.open();
        let commit = match at {
            Some(at) => repo.find_commit(Oid::from_str(at.as_str()).unwrap()),
            None => repo.head().unwrap().peel_to_commit(),
        }
        .unwrap();
        repo.branch(name, &commit, false).unwrap();
        self.checkout(name);
    }

    /// HEAD on `name`, a branch with no commit yet, the index and the
    /// working tree as they stand, as `git checkout --orphan` leaves them.
    pub fn orphan(&self, name: &str) {
        self.open().set_head(&format!("refs/heads/{name}")).unwrap();
    }

    /// The tag `name` deleted here alone.
    pub fn delete_tag(&self, name: &str) {
        self.open().tag_delete(name).unwrap();
    }

    /// Every tag's name here matching `pattern`, git's glob, or every
    /// tag's where none is given, in order.
    pub fn tags(&self, pattern: Option<&str>) -> Vec<String> {
        let names = self.open().tag_names(pattern).unwrap();
        let mut names: Vec<String> = names
            .iter()
            .flatten()
            .flatten()
            .map(str::to_string)
            .collect();
        names.sort();
        names
    }

    /// Every ref here with the object it names, in order of name.
    pub fn refs(&self) -> Vec<(String, String)> {
        let repo = self.open();
        let mut all: Vec<(String, String)> = repo
            .references()
            .unwrap()
            .map(|r| {
                let r = r.unwrap();
                let at = r.resolve().unwrap().target().unwrap();
                (r.name().unwrap().to_string(), at.to_string())
            })
            .collect();
        all.sort();
        all
    }

    /// The object `spec` names, as `git rev-parse` names it.
    pub fn rev_parse(&self, spec: &str) -> String {
        self.open().revparse_single(spec).unwrap().id().to_string()
    }

    /// The text of the blob `spec` names, as `git cat-file blob` prints
    /// it, its trailing whitespace trimmed.
    pub fn blob(&self, spec: &str) -> String {
        let repo = self.open();
        let blob = repo.revparse_single(spec).unwrap().peel_to_blob().unwrap();
        String::from_utf8(blob.content().to_vec())
            .unwrap()
            .trim_end()
            .to_string()
    }

    /// The commit `spec` names as `git log -1
    /// --format='%an <%ae>|%cn <%ce>|%s|%P'` prints it.
    pub fn commit_line(&self, spec: &str) -> String {
        let repo = self.open();
        let commit = repo
            .revparse_single(spec)
            .unwrap()
            .peel_to_commit()
            .unwrap();
        let (author, committer) = (commit.author(), commit.committer());
        let parents: Vec<String> = commit.parent_ids().map(|p| p.to_string()).collect();
        format!(
            "{} <{}>|{} <{}>|{}|{}",
            author.name().unwrap(),
            author.email().unwrap(),
            committer.name().unwrap(),
            committer.email().unwrap(),
            commit.summary().unwrap().unwrap(),
            parents.join(" ")
        )
    }

    /// The ref `name` moved to `to`, as `git update-ref` moves it.
    pub fn update_ref(&self, name: &str, to: &str) {
        self.open()
            .reference(name, Oid::from_str(to).unwrap(), true, "moved")
            .unwrap();
    }

    /// The ref `name` deleted.
    pub fn delete_ref(&self, name: &str) {
        self.open().find_reference(name).unwrap().delete().unwrap();
    }

    /// The name of every remote here.
    pub fn remotes(&self) -> Vec<String> {
        let names = self.open().remotes().unwrap();
        names
            .iter()
            .flatten()
            .flatten()
            .map(str::to_string)
            .collect()
    }

    /// A remote `name` at `url`.
    pub fn remote_add(&self, name: &str, url: &str) {
        self.open().remote(name, url).unwrap();
    }

    /// The remote `name` moved to `url`.
    pub fn remote_set_url(&self, name: &str, url: &str) {
        self.open().remote_set_url(name, url).unwrap();
    }

    /// `key` set to `value` in the repository's own configuration.
    pub fn config(&self, key: &str, value: &str) {
        self.open().config().unwrap().set_str(key, value).unwrap();
    }

    /// `key` removed from the repository's own configuration.
    pub fn config_unset(&self, key: &str) {
        self.open().config().unwrap().remove(key).unwrap();
    }

    /// A worktree `name` at `path`, on a new branch of its name at HEAD,
    /// as `git worktree add -b <name>` makes it.
    pub fn worktree(&self, name: &str, path: &Path) {
        self.open().worktree(name, path, None).unwrap();
    }
}

/// `revision` as landed on `main`, by a fake landing that says so.
pub fn landed(revision: &Revision) -> Landed {
    let landing = FakeLanding::new();
    let main = Branch::new("main");
    landing.land(&main, revision);
    Landed::check(&landing, &main, revision).unwrap()
}
