//! The shared history every publisher of a branch shares (22), as the
//! core's history reaches it through the remote `branch.<name>.remote`
//! names: the refs the remote lists, a ref pushed to it without force,
//! and a ref's objects fetched from it. It is no seam of Part B: section
//! 9 fixes git, and this is git's remote half. The core reaches it with
//! the git command, so the host's own credential helper answers (rule
//! core.21); the fakes reach a shared history at a path on the host in
//! their own process.

use std::path::Path;

use crate::Answer;

/// What a push of one ref to the shared history came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PushedRef {
    /// The shared history holds the ref where it was pushed from.
    Held,
    /// The shared history refused it, in its own words.
    Rejected(String),
}

/// The remote half of the repository's history: each operation is asked
/// of the repository at `repository` and the remote `remote` it names,
/// and an answer it cannot give is unreadable naming `what`, with the
/// remote's own words.
pub trait SharedHistory: Send + Sync {
    /// Each ref the remote holds that `pattern` names, a full ref name or
    /// one ending `*` for every name it begins, by its name and the id of
    /// the object it names, as `git ls-remote` lists them: an annotated
    /// tag's commit listed again under the tag's name and `^{}`.
    fn list(
        &self,
        repository: &Path,
        remote: &str,
        pattern: &str,
        what: &str,
    ) -> Answer<Vec<(String, String)>>;

    /// `from`, a ref or an object's id of the repository, pushed to the
    /// remote as the ref `to`, never forced: held where the remote holds
    /// it after, rejected in the remote's words where it refused it.
    fn push(
        &self,
        repository: &Path,
        remote: &str,
        from: &str,
        to: &str,
        what: &str,
    ) -> Answer<PushedRef>;

    /// The objects of the ref `name` the remote holds, fetched into the
    /// repository, with no ref, tag or `FETCH_HEAD` written.
    fn fetch(&self, repository: &Path, remote: &str, name: &str, what: &str) -> Answer<()>;
}
