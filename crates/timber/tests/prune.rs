//! Spec command-line: `timber prune` as the built binary, over a scratch
//! repository whose landing is its own `main`, a host's file naming a
//! layout, and a policy file writing `keep_days` (145; design D7).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tempfile::TempDir;
use timber::profiles::store::layout::Layout;
use timber_seams::{utc, Digest, Name, Store};

/// A day, in seconds.
const DAY: u64 = 86_400;

fn git(dir: &Path, args: &[&str]) -> String {
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

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// A repository on `main`, its own branches its landing, whose policy
/// file keeps cache entries seven days; a layout beside it; and the
/// host's file naming both.
struct Host {
    root: TempDir,
    repo: PathBuf,
    layout: PathBuf,
}

impl Host {
    fn new() -> Host {
        let root = TempDir::new().unwrap();
        let repo = root.path().join("repo");
        fs::create_dir_all(repo.join(".config")).unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        fs::write(
            repo.join(".config/timber.toml"),
            "outputs = [\"site\"]\n\n[cache]\nkeep_days = 7\n",
        )
        .unwrap();
        git(&repo, &["add", "--", ".config/timber.toml"]);
        git(
            &repo,
            &["commit", "-q", "-m", "chore: keep cache entries a week"],
        );
        let layout = root.path().join("layout");
        Layout::create(&layout).unwrap();
        let file = root.path().join("config/timber/config.toml");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(
            &file,
            format!(
                "[store]\nlayout = {:?}\n\n[landing]\nbranch = {{}}\n",
                layout.display().to_string()
            ),
        )
        .unwrap();
        Host { root, repo, layout }
    }

    /// The cache entry of `build` written into the layout at `at`.
    fn entry(&self, build: &str, at: u64) -> Digest {
        let layout = Layout::open(&self.layout).unwrap().with_clock(move || at);
        let bytes = layout
            .put_bytes(format!("the archive of {build}").as_bytes())
            .unwrap()
            .digest;
        layout
            .name(&Name::new(format!("cache/{build}")), &bytes, None)
            .unwrap();
        bytes
    }

    /// The build identities the layout lists.
    fn listed(&self) -> Vec<String> {
        Layout::open(&self.layout)
            .unwrap()
            .cache_entries()
            .unwrap()
            .into_iter()
            .map(|(build, _)| build.as_str().to_string())
            .collect()
    }

    /// The built binary run in the repository, its home and its
    /// configuration and data directories in the scratch directory.
    fn timber(&self, args: &[&str]) -> Output {
        let home = self.root.path().join("home");
        fs::create_dir_all(&home).unwrap();
        Command::new(env!("CARGO_BIN_EXE_timber"))
            .args(args)
            .current_dir(&self.repo)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("HOME", &home)
            .env("XDG_CONFIG_HOME", self.root.path().join("config"))
            .env("XDG_DATA_HOME", self.root.path().join("data"))
            .env("XDG_STATE_HOME", self.root.path().join("state"))
            .output()
            .expect("the binary runs")
    }
}

fn json(out: &Output) -> Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "prune printed no JSON document ({e}): {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

#[test]
fn prune_removes_an_entry_past_its_time_and_keeps_one_within() {
    let host = Host::new();
    let (old, recent) = (now() - 8 * DAY, now() - DAY);
    host.entry("b7e1", old);
    host.entry("c9d2", recent);
    let out = host.timber(&["prune"]);
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(0), "{text}");
    let head = git(&host.repo, &["rev-parse", "HEAD"]);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], format!("main at {head}"));
    assert_eq!(
        lines[1],
        "cache entries are kept 7 days from their push (written)"
    );
    assert_eq!(lines[2], format!("b7e1 removed, pushed {}", utc(old)));
    assert_eq!(
        lines[3],
        format!(
            "c9d2 kept until {}, pushed {}",
            utc(recent + 7 * DAY),
            utc(recent)
        )
    );
    assert_eq!(host.listed(), ["c9d2"]);
    assert_eq!(git(&host.repo, &["status", "--porcelain"]), "");
    assert!(
        !host.root.path().join("data").exists(),
        "prune made the data directory"
    );
}

#[test]
fn prune_prints_its_answer_as_json() {
    let host = Host::new();
    let old = now() - 8 * DAY;
    host.entry("b7e1", old);
    let out = host.timber(&["prune", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let json = json(&out);
    assert_eq!(json["answer"], "ok", "{json}");
    let value = &json["value"];
    assert_eq!(value["branch"], "main");
    assert_eq!(value["revision"], git(&host.repo, &["rev-parse", "HEAD"]));
    assert_eq!(
        value["keeping"],
        serde_json::json!({ "days": 7, "given": false })
    );
    let entry = &value["entries"][0];
    assert_eq!(entry["build"], "b7e1");
    assert_eq!(entry["answer"], "ok");
    assert_eq!(entry["value"]["fate"], "removed");
    assert_eq!(entry["value"]["pushed"], old);
    assert!(host.listed().is_empty());
}

#[test]
fn prune_of_a_revision_not_landed_is_refused() {
    let host = Host::new();
    host.entry("b7e1", now() - 8 * DAY);
    git(&host.repo, &["checkout", "-q", "-b", "elsewhere"]);
    git(
        &host.repo,
        &["commit", "-q", "--allow-empty", "-m", "chore: not landed"],
    );
    let elsewhere = git(&host.repo, &["rev-parse", "HEAD"]);
    git(&host.repo, &["checkout", "-q", "main"]);
    let out = host.timber(&["prune", &elsewhere, "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let json = json(&out);
    assert_eq!(json["answer"], "refused", "{json}");
    assert_eq!(json["refused"]["why"]["not_landed"]["revision"], elsewhere);
    assert_eq!(host.listed(), ["b7e1"], "an entry was removed");
}

#[test]
fn prune_exits_unreadable_where_an_entry_is() {
    let host = Host::new();
    host.entry("b7e1", now() - 8 * DAY);
    host.entry("c9d2", now() - 8 * DAY);
    let index = host.layout.join("index.json");
    let mut written: Value = serde_json::from_slice(&fs::read(&index).unwrap()).unwrap();
    for entry in written["manifests"].as_array_mut().unwrap() {
        if entry["annotations"]["org.opencontainers.image.ref.name"] == "cache/c9d2" {
            entry["annotations"]["org.opencontainers.image.created"] = "yesterday".into();
        }
    }
    fs::write(&index, serde_json::to_vec(&written).unwrap()).unwrap();
    let out = host.timber(&["prune", "--json"]);
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let json = json(&out);
    let entries = json["value"]["entries"].as_array().unwrap();
    assert_eq!(entries[0]["build"], "b7e1");
    assert_eq!(entries[0]["answer"], "ok");
    assert_eq!(entries[1]["build"], "c9d2");
    assert_eq!(entries[1]["answer"], "unreadable", "{json}");
    assert!(
        entries[1]["unreadable"]["said"]
            .as_str()
            .is_some_and(|s| s.contains("yesterday")),
        "{json}"
    );
    assert_eq!(host.listed(), ["c9d2"]);
}
