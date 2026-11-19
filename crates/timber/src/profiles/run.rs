//! The one way the profiles run a tool, git or moon, answering what it
//! printed or unreadable with the tool's own words (58, 59, rule core.19;
//! design D2).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use timber_seams::{Answer, Not, Source};

/// The variables that point git at another repository, work tree or
/// index than the one a profile names, as a hook or a gate running
/// timber may have set them: every profile's git, and moon, runs with
/// them removed.
pub(crate) const GIT_REPOSITORY_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_PREFIX",
];

/// A program a profile runs, and the source its answers name: the seam
/// and the profile that ran it.
#[derive(Clone, Debug)]
pub(crate) struct Tool {
    program: PathBuf,
    source: Source,
}

/// Variables set for one run alone, each a name and its value, set after
/// those the run removes, so one set under a removed prefix is seen.
pub(crate) type EnvSet<'a> = &'a [(&'a str, &'a OsStr)];

/// What a tool's run printed, and the exit it ended with, `None` when a
/// signal ended it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Ran {
    pub code: Option<i32>,
    /// The exit as the operating system words it.
    pub status: String,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl Tool {
    /// `program`, a path or a name found on `PATH`, answering as `source`.
    pub fn new(program: impl Into<PathBuf>, source: Source) -> Tool {
        Tool {
            program: program.into(),
            source,
        }
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    /// The tool run with `args`, in `dir` where one is given, stdin from
    /// `/dev/null`, every variable of timber's own environment that
    /// `env_drop` names removed, a name ending `*` naming every variable
    /// it begins, and then each of `env_set` set for this run alone. A
    /// run that exits with one of `allowed` answers what it
    /// printed; one that cannot start is unreadable carrying the
    /// operating system's words, and one that exits otherwise is
    /// unreadable carrying its stderr, trimmed, or its exit where it
    /// printed nothing.
    pub fn output<S: AsRef<OsStr>>(
        &self,
        dir: Option<&Path>,
        args: &[S],
        env_drop: &[&str],
        env_set: EnvSet<'_>,
        allowed: &[i32],
    ) -> Answer<Ran> {
        let ran = self.ran(dir, args, env_drop, env_set)?;
        if ran.code.is_some_and(|code| allowed.contains(&code)) {
            return Ok(ran);
        }
        Err(self.failed(args, &ran))
    }

    /// The tool run as [`output`](Tool::output) runs it, whatever it
    /// exits with: unreadable only when it cannot start.
    pub fn ran<S: AsRef<OsStr>>(
        &self,
        dir: Option<&Path>,
        args: &[S],
        env_drop: &[&str],
        env_set: EnvSet<'_>,
    ) -> Answer<Ran> {
        self.spawn(dir, args, env_drop, env_set, None)
    }

    /// The tool run as [`ran`](Tool::ran) runs it, with `input` on its
    /// stdin in place of `/dev/null`.
    pub fn fed<S: AsRef<OsStr>>(
        &self,
        dir: Option<&Path>,
        args: &[S],
        env_drop: &[&str],
        env_set: EnvSet<'_>,
        input: &[u8],
    ) -> Answer<Ran> {
        self.spawn(dir, args, env_drop, env_set, Some(input))
    }

    fn spawn<S: AsRef<OsStr>>(
        &self,
        dir: Option<&Path>,
        args: &[S],
        env_drop: &[&str],
        env_set: EnvSet<'_>,
        input: Option<&[u8]>,
    ) -> Answer<Ran> {
        let mut command = Command::new(&self.program);
        command
            .args(args)
            .stdin(if input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(dir) = dir {
            command.current_dir(dir);
        }
        for (name, _) in std::env::vars_os() {
            let name_text = name.to_string_lossy();
            let dropped = env_drop.iter().any(|drop| match drop.strip_suffix('*') {
                Some(prefix) => name_text.starts_with(prefix),
                None => name_text == *drop,
            });
            if dropped {
                command.env_remove(&name);
            }
        }
        for (name, value) in env_set {
            command.env(name, value);
        }
        let mut child = command
            .spawn()
            .map_err(|e| self.unreadable(args, e.to_string()))?;
        let stdin = child.stdin.take();
        let out = std::thread::scope(|scope| {
            if let (Some(mut stdin), Some(input)) = (stdin, input) {
                // A tool that exits before reading it all says why on
                // its stderr, which is the answer.
                scope.spawn(move || std::io::Write::write_all(&mut stdin, input));
            }
            child.wait_with_output()
        })
        .map_err(|e| self.unreadable(args, e.to_string()))?;
        Ok(Ran {
            code: out.status.code(),
            status: out.status.to_string(),
            stdout: out.stdout,
            stderr: out.stderr,
        })
    }

    /// Unreadable naming the command `ran` came from, carrying its
    /// stderr, trimmed, or its exit where it printed nothing.
    pub fn failed<S: AsRef<OsStr>>(&self, args: &[S], ran: &Ran) -> Not {
        let words = String::from_utf8_lossy(&ran.stderr).trim().to_string();
        self.unreadable(
            args,
            if words.is_empty() {
                ran.status.clone()
            } else {
                words
            },
        )
    }

    /// Unreadable naming the command run and carrying `said`.
    pub fn unreadable<S: AsRef<OsStr>>(&self, args: &[S], said: String) -> Not {
        let mut what = self.program.display().to_string();
        for arg in args {
            what.push(' ');
            what.push_str(&arg.as_ref().to_string_lossy());
        }
        Not::unreadable(&self.source, what, Some(said))
    }
}

/// What `f` answers with each of `set` set in timber's own environment,
/// as a host's may set them; one such test at a time, since the
/// variables are the process's, and each as it was after.
#[cfg(test)]
pub(crate) fn under_env<T>(set: &[(&str, &str)], f: impl FnOnce() -> T) -> T {
    static ENVIRONMENT: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _environment = ENVIRONMENT.lock().unwrap_or_else(|e| e.into_inner());
    let before: Vec<_> = set
        .iter()
        .map(|(name, _)| (*name, std::env::var_os(name)))
        .collect();
    for (name, value) in set {
        std::env::set_var(name, value);
    }
    let answer = f();
    for (name, value) in before {
        match value {
            Some(value) => std::env::set_var(name, value),
            None => std::env::remove_var(name),
        }
    }
    answer
}

#[cfg(test)]
mod tests {
    use timber_seams::Seam;

    use super::*;

    fn tool(program: &str) -> Tool {
        Tool::new(program, Source::new(Seam::Build, "a tool"))
    }

    fn said(answer: Answer<Ran>) -> String {
        match answer {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, Source::new(Seam::Build, "a tool"));
                u.said.expect("the tool's words")
            }
            other => panic!("answered {other:?}, not unreadable"),
        }
    }

    #[test]
    fn a_program_that_does_not_exist_carries_the_operating_systems_words() {
        let dir = tempfile::TempDir::new().unwrap();
        let gone = dir.path().join("moon");
        let words = said(tool(gone.to_str().unwrap()).output(None, &["--version"], &[], &[], &[0]));
        assert_eq!(words, "No such file or directory (os error 2)");
    }

    #[test]
    fn a_command_that_fails_carries_its_own_words_trimmed_and_unchanged() {
        let dir = tempfile::TempDir::new().unwrap();
        let missing = dir.path().join("repository");
        let args = [
            OsStr::new("-C"),
            missing.as_os_str(),
            OsStr::new("rev-parse"),
            OsStr::new("--git-common-dir"),
        ];
        let theirs = Command::new("git").args(args).output().unwrap();
        assert!(!theirs.status.success());
        let words = said(tool("git").output(None, &args, &[], &[], &[0]));
        assert_eq!(words, String::from_utf8_lossy(&theirs.stderr).trim());
        assert!(words.starts_with("fatal: cannot change to"), "{words}");
    }

    #[test]
    fn an_exit_the_caller_allows_answers_its_output() {
        let script = "echo printed; echo warned >&2; exit 3";
        let ran = tool("sh")
            .output(None, &["-c", script], &[], &[], &[0, 3])
            .unwrap();
        assert_eq!(ran.code, Some(3));
        assert_eq!(ran.stdout, b"printed\n");
        assert_eq!(ran.stderr, b"warned\n");
        assert_eq!(
            said(tool("sh").output(None, &["-c", script], &[], &[], &[0])),
            "warned"
        );
        assert_eq!(
            said(tool("sh").output(None, &["-c", "exit 4"], &[], &[], &[0])),
            "exit status: 4"
        );
    }

    #[test]
    fn what_it_is_fed_is_its_stdin() {
        let ran = tool("sh")
            .fed(None, &["-c", "tr a-z A-Z"], &[], &[], b"fed\n")
            .unwrap();
        assert_eq!(ran.stdout, b"FED\n");
        let empty = tool("sh").ran(None, &["-c", "cat"], &[], &[]).unwrap();
        assert_eq!(empty.stdout, b"");
    }

    #[test]
    fn the_variables_dropped_are_not_in_its_environment() {
        let ran = tool("sh")
            .output(
                Some(Path::new("/")),
                &["-c", "pwd; env"],
                &["LOGNAME", "HOM*"],
                &[],
                &[0],
            )
            .unwrap();
        let printed = String::from_utf8(ran.stdout).unwrap();
        assert!(printed.starts_with("/\n"), "{printed}");
        for dropped in ["HOME=", "LOGNAME="] {
            assert!(
                !printed.lines().any(|l| l.starts_with(dropped)),
                "{printed}"
            );
        }
    }

    #[test]
    fn a_run_is_handed_variables_of_its_own() {
        let print = ["-c", "printf %s \"${TIMBER_RUN_OWN-unset}\""];
        let own = [("TIMBER_RUN_OWN", OsStr::new("its own"))];
        let ran = tool("sh").output(None, &print, &[], &own, &[0]).unwrap();
        assert_eq!(ran.stdout, b"its own");
        let ran = tool("sh").ran(None, &print, &[], &own).unwrap();
        assert_eq!(ran.stdout, b"its own");
        let ran = tool("sh").fed(None, &print, &[], &own, b"").unwrap();
        assert_eq!(ran.stdout, b"its own");
        let none = tool("sh").output(None, &print, &[], &[], &[0]).unwrap();
        assert_eq!(none.stdout, b"unset");
    }

    #[test]
    fn a_variable_set_outlives_a_dropped_prefix() {
        let hosts = [
            ("GH_HOST", "the host's own"),
            ("GH_TOKEN", "the host's token"),
        ];
        let ran = under_env(&hosts, || {
            tool("sh").output(
                None,
                &["-c", "env"],
                &["GH_*"],
                &[("GH_TOKEN", OsStr::new("the run's own"))],
                &[0],
            )
        })
        .unwrap();
        let printed = String::from_utf8(ran.stdout).unwrap();
        assert!(
            printed.lines().any(|l| l == "GH_TOKEN=the run's own"),
            "{printed}"
        );
        assert!(
            !printed.lines().any(|l| l.starts_with("GH_HOST=")),
            "{printed}"
        );
        assert!(!printed.contains("the host's token"), "{printed}");
    }
}
