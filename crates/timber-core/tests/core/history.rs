//! The repository's history as the core reads it (design D3, D6).

use std::collections::BTreeSet;
use std::fs;

use crate::common::{landed, Fixture, Repo};
use semver::Version;
use timber_core::{paths_hold, GitCommand, History, ROOT_REF};
use timber_fakes::FakeSharedHistory;
use timber_seams::{Answer, Branch, Not, RepositoryName, Revision, Seam, SharedHistory, Why};

#[test]
fn a_path_that_is_not_a_repository_is_unreadable_with_gits_words() {
    let fixture = Fixture::new();
    let nowhere = fixture.path("nowhere");
    fs::create_dir_all(&nowhere).unwrap();
    let Err(Not::Unreadable(u)) = History::open(&nowhere) else {
        panic!("a directory with no repository opened")
    };
    assert_eq!(u.source.seam, Seam::History);
    assert_eq!(u.source.profile, "git, local");
    assert!(u.what.contains(nowhere.to_str().unwrap()), "{}", u.what);
    let said = u.said.unwrap();
    assert!(said.contains("could not find repository"), "{said}");
}

#[test]
fn a_file_is_read_from_the_revision_while_the_working_tree_holds_another() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let rev = repo.commit("chore: start", &[("notes.txt", "at the revision")]);
    fs::write(repo.path.join("notes.txt"), "in the working tree").unwrap();

    let history = repo.history();
    let at = landed(&rev);
    assert_eq!(history.blob(&at, "notes.txt").unwrap(), b"at the revision");
    assert!(matches!(
        history.blob(&at, "missing.txt"),
        Err(Not::Absent(a)) if a.what.contains("missing.txt")
    ));
}

#[test]
fn a_commit_outside_the_paths_is_not_listed() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let first = repo.touch("feat: lib", &["crates/lib"]);
    let site = repo.touch("feat: site", &["site"]);
    let lib = repo.touch("fix: lib", &["crates/lib"]);

    let history = repo.history();
    let at = landed(&lib);
    let revisions = |since: Option<&Revision>, paths: &[&str]| {
        let paths: Vec<String> = paths.iter().map(|p| p.to_string()).collect();
        history
            .commits(&at, since, &paths)
            .unwrap()
            .into_iter()
            .map(|c| (c.revision, c.message))
            .collect::<BTreeSet<_>>()
    };
    let commit = |rev: &Revision, message: &str| (rev.clone(), format!("{message}\n"));
    assert_eq!(
        revisions(None, &["crates/lib/**"]),
        BTreeSet::from([commit(&first, "feat: lib"), commit(&lib, "fix: lib")])
    );
    assert_eq!(
        revisions(Some(&first), &["crates/lib/**"]),
        BTreeSet::from([commit(&lib, "fix: lib")])
    );
    assert_eq!(
        revisions(Some(&first), &["**"]),
        BTreeSet::from([commit(&site, "feat: site"), commit(&lib, "fix: lib")])
    );
}

#[test]
fn a_commits_message_is_read_as_written_from_the_local_history() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let rev = repo.commit("feat: a call\n\nWith a body.", &[("lib.txt", "a call")]);
    let history = repo.history();
    assert_eq!(
        history.message(&rev).unwrap(),
        "feat: a call\n\nWith a body.\n"
    );
    let listed = history
        .commits(&landed(&rev), None, &["**".into()])
        .unwrap();
    assert_eq!(history.message(&rev).unwrap(), listed[0].message);
}

#[test]
fn a_commit_the_history_does_not_hold_is_absent_naming_it() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit("chore: start", &[("a.txt", "a")]);
    let history = repo.history();
    let gone = Revision::new("4f2a9c1e0b7d3a5f6c8e9d0a1b2c3d4e5f6a7b8c");
    let Err(Not::Absent(a)) = history.message(&gone) else {
        panic!("a commit the history does not hold was read")
    };
    assert_eq!(a.source.seam, Seam::History);
    assert!(a.what.contains(gone.as_str()), "{}", a.what);
}

#[test]
fn an_annotated_and_a_lightweight_tag_both_peel_to_their_commit() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let one = repo.touch("feat: one", &["a"]);
    let two = repo.touch("feat: two", &["a"]);
    repo.annotated("v1.0.0", &one);
    repo.tag("v1.1.0", &two);
    repo.tag("v-not-a-version", &two);
    repo.tag("lib-v9.0.0", &two);

    let tags = repo.history().tags(&landed(&two), "v").unwrap();
    let named: Vec<_> = tags
        .iter()
        .map(|t| (t.name.as_str(), t.version.clone(), t.revision.clone()))
        .collect();
    assert_eq!(
        named,
        [
            ("v1.0.0", Version::new(1, 0, 0), one),
            ("v1.1.0", Version::new(1, 1, 0), two)
        ]
    );
}

#[test]
fn a_tag_on_a_revision_the_landed_one_does_not_reach_is_not_kept() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let base = repo.touch("feat: base", &["a"]);
    repo.tag("v1.0.0", &base);
    repo.git(&["checkout", "-q", "-b", "side"]);
    let side = repo.touch("feat: side", &["a"]);
    repo.tag("v2.0.0", &side);
    repo.git(&["checkout", "-q", "main"]);
    let main = repo.touch("fix: main", &["a"]);

    let tags = repo.history().tags(&landed(&main), "v").unwrap();
    assert_eq!(
        tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
        ["v1.0.0"]
    );
}

#[test]
fn a_tag_only_the_shared_history_holds_is_read() {
    let fixture = Fixture::new();
    let shared = fixture.bare("shared.git");
    let first = fixture.clone_of(&shared, "first");
    let rev = first.touch("feat: one", &["a"]);
    first.git(&["push", "-q", "origin", "main"]);
    let second = fixture.clone_of(&shared, "second");
    first.annotated("v1.0.0", &rev);
    first.git(&["push", "-q", "origin", "refs/tags/v1.0.0"]);
    first.git(&["tag", "-d", "v1.0.0"]);
    assert_eq!(second.tag_at("v1.0.0"), None);

    let history = second.history();
    assert_eq!(
        history.remote(&Branch::new("main")).unwrap().as_deref(),
        Some("origin")
    );
    let tags = history.tags(&landed(&rev), "v").unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "v1.0.0");
    assert_eq!(tags[0].revision, rev);
}

#[test]
fn a_shared_history_that_cannot_be_reached_makes_the_tags_unreadable() {
    let fixture = Fixture::new();
    let shared = fixture.bare("shared.git");
    let clone = fixture.clone_of(&shared, "clone");
    let rev = clone.touch("feat: one", &["a"]);
    clone.git(&["push", "-q", "origin", "main"]);
    fs::remove_dir_all(&shared.path).unwrap();

    let Err(Not::Unreadable(u)) = clone.history().tags(&landed(&rev), "v") else {
        panic!("tags read with the shared history gone")
    };
    assert_eq!(u.source.profile, "git, origin");
    let said = u.said.unwrap();
    assert!(
        said.contains("does not appear to be a git repository"),
        "{said}"
    );
}

#[test]
fn a_branch_with_no_remote_shares_no_history() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.touch("feat: one", &["a"]);
    assert_eq!(repo.history().remote(&Branch::new("main")).unwrap(), None);
}

#[test]
fn a_short_id_and_head_resolve_to_the_full_id() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    let first = repo.commit("chore: start", &[("a", "a")]);
    let second = repo.commit("feat: more", &[("b", "b")]);
    repo.tag("v0.1.0", &first);

    let history = repo.history();
    assert_eq!(history.resolve(&second.as_str()[..7]).unwrap(), second);
    assert_eq!(history.resolve("HEAD").unwrap(), second);
    assert_eq!(history.resolve("HEAD~1").unwrap(), first);
    assert_eq!(history.resolve("v0.1.0").unwrap(), first);
    assert_eq!(history.resolve(second.as_str()).unwrap(), second);
    assert_eq!(history.head(&Branch::new("main")).unwrap(), second);
}

#[test]
fn a_name_that_is_no_commit_is_absent_with_gits_words() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit("chore: start", &[("a", "a")]);

    let history = repo.history();
    let Err(Not::Absent(a)) = history.resolve("no-such-thing") else {
        panic!("a name that is no commit resolved")
    };
    assert_eq!(a.source.seam, Seam::History);
    let git = git2::Repository::open(&repo.path)
        .unwrap()
        .revparse_single("no-such-thing")
        .unwrap_err();
    assert_eq!(
        a.what,
        format!("commit named no-such-thing: {}", git.message())
    );
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
    assert!(matches!(history.resolve(&tree), Err(Not::Absent(_))));
    assert!(matches!(
        history.head(&Branch::new("elsewhere")),
        Err(Not::Absent(a)) if a.what == "branch elsewhere"
    ));
}

#[test]
fn two_worktrees_of_one_repository_answer_one_common_directory() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit("chore: start", &[("a", "a")]);
    let other = fixture.path("other");
    repo.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "other",
        other.to_str().unwrap(),
    ]);

    let here = repo.history().common_dir();
    let there = History::open(&other).unwrap().common_dir();
    assert_eq!(here, there);
    assert_eq!(here, repo.path.join(".git").canonicalize().unwrap());

    let elsewhere = fixture.repo("elsewhere");
    elsewhere.commit("chore: start", &[("a", "a")]);
    assert_ne!(elsewhere.history().common_dir(), here);
}

#[test]
fn two_worktrees_of_one_repository_answer_two_git_directories() {
    let fixture = Fixture::new();
    let repo = fixture.repo("repo");
    repo.commit("chore: start", &[("a", "a")]);
    let other = fixture.path("other");
    repo.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "other",
        other.to_str().unwrap(),
    ]);

    let (here, there) = (repo.history(), History::open(&other).unwrap());
    assert_ne!(here.git_dir(), there.git_dir());
    assert_eq!(here.common_dir(), there.common_dir());
    assert_eq!(
        here.git_dir(),
        repo.path.join(".git").canonicalize().unwrap()
    );
    let theirs = repo.git(&[
        "-C",
        other.to_str().unwrap(),
        "rev-parse",
        "--absolute-git-dir",
    ]);
    assert_eq!(
        there.git_dir(),
        std::path::Path::new(&theirs).canonicalize().unwrap()
    );
    assert!(
        there.git_dir().starts_with(here.common_dir()),
        "{} is not under the common directory",
        there.git_dir().display()
    );
}

#[test]
fn a_components_paths_hold_a_path_as_git_cliffs_filter_matches_it() {
    let holds = paths_hold(&["crates/fam/**".into(), "./site/**".into(), "docs/".into()]).unwrap();
    for path in [
        "crates/fam/Cargo.toml",
        "crates/fam/src/lib.rs",
        "site/index.html",
        "docs/guide/start.md",
    ] {
        assert!(holds(path), "{path}");
    }
    for path in [
        "crates/family/Cargo.toml",
        "crates/base/Cargo.toml",
        "Cargo.toml",
    ] {
        assert!(!holds(path), "{path}");
    }
    let Err(Not::Unreadable(u)) = paths_hold(&["crates/[".into()]) else {
        panic!("a path that is no glob was read")
    };
    assert_eq!(u.source.seam, Seam::History);
    assert!(u.what.contains("crates/["), "{u:?}");
}

fn root(owner: &str, name: &str) -> RepositoryName {
    RepositoryName {
        owner: owner.into(),
        name: name.into(),
    }
}

/// A clone of a new shared history, its `main` pushed at one commit, and
/// that commit landed.
fn shared_clone(fixture: &Fixture) -> (Repo, Repo, Revision) {
    let shared = fixture.bare("shared.git");
    let clone = fixture.clone_of(&shared, "clone");
    let rev = clone.touch("feat: one", &["a"]);
    clone.git(&["push", "-q", "origin", "main"]);
    (shared, clone, rev)
}

/// The root `repo`'s ref of the record names at its tip, as git reads it.
fn root_at_tip(repo: &Repo) -> String {
    repo.git(&["cat-file", "blob", &format!("{ROOT_REF}:root")])
}

/// A commit of the record naming `named` on `parent`, written into `repo`
/// by the git command as another host would, and pushed to `refs/b/root`
/// of its remote; its id.
fn written_elsewhere(repo: &Repo, named: &str, parent: Option<&str>) -> String {
    let blob = git_fed(
        repo,
        &["hash-object", "-w", "--stdin"],
        &format!("{named}\n"),
    );
    let tree = git_fed(repo, &["mktree"], &format!("100644 blob {blob}\troot\n"));
    let mut args = vec!["commit-tree", tree.as_str(), "-m", named];
    if let Some(parent) = parent {
        args.extend(["-p", parent]);
    }
    let commit = repo.git(&args);
    repo.git(&["push", "-q", "origin", &format!("{commit}:refs/b/root")]);
    commit
}

fn git_fed(repo: &Repo, args: &[&str], input: &str) -> String {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new("git")
        .arg("-C")
        .arg(&repo.path)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// A hook in `shared` that, at the first push it is handed, moves the
/// record's ref to `refs/b/root` before the push's own update, as another
/// host's write landing first would.
fn racing_hook(shared: &Repo) {
    let hook = shared.path.join("hooks").join("pre-receive");
    fs::write(
        &hook,
        "#!/bin/sh\n\
         cat >/dev/null\n\
         [ -e raced ] && exit 0\n\
         touch raced\n\
         env -u GIT_QUARANTINE_PATH git update-ref refs/timber/root refs/b/root\n",
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn the_root_record_is_empty_where_no_ref_is_held() {
    let fixture = Fixture::new();
    let (_, clone, rev) = shared_clone(&fixture);
    assert_eq!(clone.history().roots(&landed(&rev)).unwrap(), []);
    let alone = fixture.repo("alone");
    let rev = alone.touch("feat: one", &["a"]);
    assert_eq!(alone.history().roots(&landed(&rev)).unwrap(), []);
}

#[test]
fn the_root_record_holds_the_first_root_written_and_pushed() {
    let fixture = Fixture::new();
    let (shared, clone, rev) = shared_clone(&fixture);
    let history = clone.history();
    history
        .record_root(&landed(&rev), &root("acme", "app"))
        .unwrap();

    assert_eq!(history.roots(&landed(&rev)).unwrap(), [root("acme", "app")]);
    assert_eq!(root_at_tip(&shared), "acme/app");
    assert_eq!(
        shared.git(&["rev-parse", ROOT_REF]),
        clone.git(&["rev-parse", ROOT_REF]),
        "the copy is not where the remote is"
    );
    assert_eq!(
        shared.git(&["log", "-1", "--format=%an <%ae>|%cn <%ce>|%s|%P", ROOT_REF]),
        "timber <timber@localhost>|timber <timber@localhost>|acme/app|"
    );
    let other = fixture.clone_of(&shared, "other");
    assert_eq!(
        other.history().roots(&landed(&rev)).unwrap(),
        [root("acme", "app")]
    );
}

#[test]
fn the_root_record_appends_a_second_root_after_the_first() {
    let fixture = Fixture::new();
    let (shared, clone, rev) = shared_clone(&fixture);
    let history = clone.history();
    history
        .record_root(&landed(&rev), &root("acme", "app"))
        .unwrap();
    let first = shared.git(&["rev-parse", ROOT_REF]);
    history
        .record_root(&landed(&rev), &root("acme", "site"))
        .unwrap();

    assert_eq!(
        history.roots(&landed(&rev)).unwrap(),
        [root("acme", "app"), root("acme", "site")]
    );
    assert_eq!(shared.git(&["rev-parse", &format!("{ROOT_REF}^")]), first);
    assert_eq!(root_at_tip(&shared), "acme/site");
}

#[test]
fn the_root_record_written_lowercases_the_root() {
    let fixture = Fixture::new();
    let (shared, clone, rev) = shared_clone(&fixture);
    let history = clone.history();
    history
        .record_root(&landed(&rev), &root("Acme", "Site"))
        .unwrap();
    assert_eq!(root_at_tip(&shared), "acme/site");
    assert_eq!(
        history.roots(&landed(&rev)).unwrap(),
        [root("acme", "site")]
    );
}

#[test]
fn the_root_record_read_again_after_a_push_refused_is_done_where_it_names_the_root() {
    let fixture = Fixture::new();
    let (shared, clone, rev) = shared_clone(&fixture);
    let theirs = written_elsewhere(&clone, "acme/site", None);
    racing_hook(&shared);

    let history = clone.history();
    history
        .record_root(&landed(&rev), &root("acme", "site"))
        .unwrap();
    assert_eq!(shared.git(&["rev-parse", ROOT_REF]), theirs);
    assert_eq!(
        history.roots(&landed(&rev)).unwrap(),
        [root("acme", "site")]
    );
}

#[test]
fn the_root_record_refused_where_another_root_was_written_first_names_gits_words() {
    let fixture = Fixture::new();
    let (shared, clone, rev) = shared_clone(&fixture);
    let theirs = written_elsewhere(&clone, "acme/web", None);
    racing_hook(&shared);

    let history = clone.history();
    let Err(Not::Refused(r)) = history.record_root(&landed(&rev), &root("acme", "site")) else {
        panic!("a root written over another's was recorded")
    };
    assert_eq!(r.source.profile, "git, origin");
    assert_eq!(r.why, Why::Said);
    let said = r.said.unwrap();
    assert!(said.contains(ROOT_REF), "{said}");
    assert!(said.contains("rejected"), "{said}");
    assert_eq!(shared.git(&["rev-parse", ROOT_REF]), theirs);
    assert_eq!(history.roots(&landed(&rev)).unwrap(), [root("acme", "web")]);
}

#[test]
fn the_root_record_is_unreadable_where_the_remote_holds_less_than_the_copy() {
    let fixture = Fixture::new();
    let (shared, clone, rev) = shared_clone(&fixture);
    let history = clone.history();
    history
        .record_root(&landed(&rev), &root("acme", "app"))
        .unwrap();
    let first = shared.git(&["rev-parse", ROOT_REF]);
    history
        .record_root(&landed(&rev), &root("acme", "site"))
        .unwrap();
    let second = shared.git(&["rev-parse", ROOT_REF]);
    shared.git(&["update-ref", ROOT_REF, &first]);

    let Err(Not::Unreadable(u)) = history.roots(&landed(&rev)) else {
        panic!("a remote holding less than the copy was read")
    };
    assert_eq!(u.source.profile, "git, origin");
    assert!(u.what.contains(&second), "{}", u.what);
    assert!(!u.what.contains(&first), "{}", u.what);
    assert_eq!(
        clone.git(&["rev-parse", ROOT_REF]),
        second,
        "the copy moved back"
    );

    shared.git(&["update-ref", "-d", ROOT_REF]);
    let Err(Not::Unreadable(u)) = history.roots(&landed(&rev)) else {
        panic!("a remote holding none of the copy was read")
    };
    assert!(
        u.what.contains(&first) && u.what.contains(&second),
        "{}",
        u.what
    );
}

#[test]
fn the_root_record_is_unreadable_where_the_remote_cannot_be_reached() {
    let fixture = Fixture::new();
    let (shared, clone, rev) = shared_clone(&fixture);
    let history = clone.history();
    history
        .record_root(&landed(&rev), &root("acme", "app"))
        .unwrap();
    fs::remove_dir_all(&shared.path).unwrap();

    for answer in [
        history.roots(&landed(&rev)).map(|_| ()),
        history.record_root(&landed(&rev), &root("acme", "site")),
    ] {
        let Err(Not::Unreadable(u)) = answer else {
            panic!("the record was read with its remote gone: {answer:?}")
        };
        assert_eq!(u.source.profile, "git, origin");
        let said = u.said.unwrap();
        assert!(
            said.contains("does not appear to be a git repository"),
            "{said}"
        );
    }
    assert_eq!(root_at_tip(&clone), "acme/app");
}

#[test]
fn the_root_record_of_a_branch_with_no_remote_is_the_own_ref() {
    let fixture = Fixture::new();
    let repo = fixture.repo("alone");
    let rev = repo.touch("feat: one", &["a"]);
    let history = repo.history();
    history
        .record_root(&landed(&rev), &root("acme", "app"))
        .unwrap();
    history
        .record_root(&landed(&rev), &root("acme", "site"))
        .unwrap();
    assert_eq!(
        history.roots(&landed(&rev)).unwrap(),
        [root("acme", "app"), root("acme", "site")]
    );
    assert_eq!(root_at_tip(&repo), "acme/site");
    assert_eq!(repo.git(&["remote"]), "");
}

// The shared history: the git command and the fake answer alike

/// What `answer` came to, its revisions named by `names` and a refusal by
/// git's reason alone, so the git command's answers and the fake's
/// compare.
fn told<T: std::fmt::Debug>(answer: Answer<T>, names: &[(String, &str)]) -> String {
    let mut text = match answer {
        Ok(value) => format!("{value:?}"),
        Err(Not::Unreadable(_)) => "unreadable".into(),
        Err(other) => format!("{other:?}"),
    };
    if let Some(at) = text.find("[rejected] (") {
        let reason = &text[at..];
        let end = reason.find(')').unwrap();
        text = format!("rejected {}", &reason["[rejected] ".len()..=end]);
    }
    for (id, name) in names {
        text = text.replace(id.as_str(), name);
    }
    text
}

/// One shared history and two clones of it, reached as `shared`: tags
/// pushed, again, and refused; an annotated tag listed with its commit;
/// the record's ref pushed, moved forward, refused, fetched and refused
/// again; and what neither holds.
fn reached_as(shared: &dyn SharedHistory) -> Vec<String> {
    let fixture = Fixture::new();
    let bare = fixture.bare("shared.git");
    let first = fixture.clone_of(&bare, "first");
    let second = fixture.clone_of(&bare, "second");
    let one = first.commit("feat: one", &[("a", "1")]);
    first.tag("v1.0.0", &one);
    first.annotated("v1.1.0", &one);
    let two = second.commit("feat: two", &[("b", "2")]);
    second.tag("v1.0.0", &two);
    let three = first.commit("feat: three", &[("a", "3")]);
    let names = [
        (one.as_str().to_string(), "one"),
        (two.as_str().to_string(), "two"),
        (three.as_str().to_string(), "three"),
    ];
    let w = "the test's read";
    let (a, b) = (&first.path, &second.path);
    let tag = |name: &str| format!("refs/tags/{name}");
    let mut said = vec![
        told(
            shared.push(a, "origin", &tag("v1.0.0"), &tag("v1.0.0"), w),
            &names,
        ),
        told(
            shared.push(a, "origin", &tag("v1.0.0"), &tag("v1.0.0"), w),
            &names,
        ),
        told(
            shared.push(a, "origin", &tag("v1.1.0"), &tag("v1.1.0"), w),
            &names,
        ),
        told(
            shared.push(b, "origin", &tag("v1.0.0"), &tag("v1.0.0"), w),
            &names,
        ),
    ];
    let listed = shared.list(a, "origin", "refs/tags/v*", w).map(|refs| {
        refs.into_iter()
            .filter(|(name, _)| !name.starts_with("refs/tags/v1.1.0") || name.ends_with("^{}"))
            .collect::<Vec<_>>()
    });
    said.push(told(listed, &names));
    said.push(told(
        shared.push(a, "origin", one.as_str(), ROOT_REF, w),
        &names,
    ));
    said.push(told(
        shared.push(a, "origin", three.as_str(), ROOT_REF, w),
        &names,
    ));
    said.push(told(
        shared.push(b, "origin", two.as_str(), ROOT_REF, w),
        &names,
    ));
    said.push(told(shared.fetch(b, "origin", ROOT_REF, w), &names));
    said.push(told(
        shared.push(b, "origin", two.as_str(), ROOT_REF, w),
        &names,
    ));
    said.push(told(shared.list(b, "origin", ROOT_REF, w), &names));
    let fetched = git2::Repository::open(b).unwrap();
    said.push(format!(
        "fetched {}, no ref {}",
        fetched
            .find_commit(git2::Oid::from_str(three.as_str()).unwrap())
            .is_ok(),
        fetched.find_reference(ROOT_REF).is_err()
    ));
    said.push(told(
        shared.fetch(b, "origin", "refs/timber/none", w),
        &names,
    ));
    said.push(told(shared.list(b, "nowhere", ROOT_REF, w), &names));
    said
}

/// The one contract test of the shared history against real git (rule
/// logic-is-tested-as-units-and-few-tests-run-the-binary): the fake the
/// core's tests reach it through answers every operation as the git
/// command does.
#[test]
fn the_git_command_and_the_fake_reach_a_shared_history_alike() {
    let git = reached_as(&GitCommand);
    assert_eq!(
        git[..4],
        ["Held", "Held", "Held", "rejected (already exists)"]
    );
    assert_eq!(git[7], "rejected (fetch first)", "{git:?}");
    assert_eq!(git[9], "rejected (non-fast-forward)", "{git:?}");
    assert_eq!(reached_as(&FakeSharedHistory), git);
}
