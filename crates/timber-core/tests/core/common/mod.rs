//! Repositories in a temporary directory, made, committed to, tagged and
//! pushed in the test's own process with libgit2, by a fixed author, for
//! the core's tests over real history (design D8). [`git`] runs the git
//! command for the few states a test makes that the fixture does not.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// The git command at `dir`, by the fixed author, with no configuration
/// of the host's, so no host's signing or hooks reach a test.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "A. Uthor")
        .env("GIT_AUTHOR_EMAIL", "author@example.com")
        .env("GIT_COMMITTER_NAME", "A. Uthor")
        .env("GIT_COMMITTER_EMAIL", "author@example.com")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .trim_end()
        .to_string()
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
    pub fn git(&self, args: &[&str]) -> String {
        git(&self.path, args)
    }

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
        History::reaching(&self.path, Box::new(FakeSharedHistory)).unwrap()
    }
}

/// `revision` as landed on `main`, by a fake landing that says so.
pub fn landed(revision: &Revision) -> Landed {
    let landing = FakeLanding::new();
    let main = Branch::new("main");
    landing.land(&main, revision);
    Landed::check(&landing, &main, revision).unwrap()
}
