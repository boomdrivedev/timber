//! The shared history reached with the git command (22, 62; rule
//! core.21): the one way the core asks a remote for its refs, pushes a
//! version's tag or the record of roots to it, and fetches the record
//! from it, so a remote asks the host's own credential helper and never a
//! terminal.

use std::path::Path;
use std::process::{Command, Output};

use timber_seams::{Answer, Not, PushedRef, SharedHistory, Source};

/// The shared history as the git command at the repository reaches it.
#[derive(Clone, Copy, Debug, Default)]
pub struct GitCommand;

/// What git said to stderr.
fn said(output: &Output) -> Option<String> {
    Some(String::from_utf8_lossy(&output.stderr).into_owned())
}

impl GitCommand {
    /// The git command at `repository` with `args`, with no prompt;
    /// unreadable naming `remote` and `what` where it cannot start.
    fn run(&self, repository: &Path, remote: &str, what: &str, args: &[&str]) -> Answer<Output> {
        Command::new("git")
            .arg("-C")
            .arg(repository)
            .args(args)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .map_err(|e| Not::unreadable(&Source::history(remote), what, Some(e.to_string())))
    }
}

impl SharedHistory for GitCommand {
    /// `git ls-remote`, which writes nothing, limited to the remote's tags
    /// where `pattern` names tags.
    fn list(
        &self,
        repository: &Path,
        remote: &str,
        pattern: &str,
        what: &str,
    ) -> Answer<Vec<(String, String)>> {
        let mut args = vec!["ls-remote"];
        if pattern.starts_with("refs/tags/") {
            args.push("--tags");
        }
        args.extend([remote, pattern]);
        let out = self.run(repository, remote, what, &args)?;
        if !out.status.success() {
            return Err(Not::unreadable(&Source::history(remote), what, said(&out)));
        }
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .map(|(oid, name)| (name.to_string(), oid.to_string()))
            .collect())
    }

    /// `git push --porcelain` of `from:to`, read from the line git prints
    /// for `to`: made, up to date, or for a ref that is no tag moved
    /// forward, held; refused, rejected with what git printed. git moves
    /// no tag without force, so a tag is held only made or up to date.
    fn push(
        &self,
        repository: &Path,
        remote: &str,
        from: &str,
        to: &str,
        what: &str,
    ) -> Answer<PushedRef> {
        let refspec = format!("{from}:{to}");
        let out = self.run(
            repository,
            remote,
            what,
            &["push", "--porcelain", remote, &refspec],
        )?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        // Each ref's line is its flag, the refspec and git's summary.
        let flag = stdout.lines().find_map(|line| {
            let mut fields = line.split('\t');
            let flag = fields.next()?;
            fields.next()?.ends_with(&format!(":{to}")).then_some(flag)
        });
        let moved_forward = !to.starts_with("refs/tags/");
        match flag {
            Some("*" | "=") if out.status.success() => Ok(PushedRef::Held),
            Some(" ") if moved_forward && out.status.success() => Ok(PushedRef::Held),
            Some("!") => Ok(PushedRef::Rejected(format!(
                "{stdout}{}",
                String::from_utf8_lossy(&out.stderr)
            ))),
            _ => Err(Not::unreadable(&Source::history(remote), what, said(&out))),
        }
    }

    /// `git fetch` of `name` alone, with no tags and no `FETCH_HEAD`.
    fn fetch(&self, repository: &Path, remote: &str, name: &str, what: &str) -> Answer<()> {
        let out = self.run(
            repository,
            remote,
            what,
            &[
                "fetch",
                "--quiet",
                "--no-tags",
                "--no-write-fetch-head",
                remote,
                name,
            ],
        )?;
        if !out.status.success() {
            return Err(Not::unreadable(&Source::history(remote), what, said(&out)));
        }
        Ok(())
    }
}
