//! The GitHub release target (C.12's GitHub release row), bound as
//! `github_release`: the GitHub release of a version's tag made or
//! updated in the checkout's repository on github.com, the version's
//! notes as its text and the outputs its table names as its assets, from
//! the host that holds a token that may write the repository's releases
//! (design D1 to D9).
//!
//! An environment names it in its `target` table, which names the host's
//! credential and the component's outputs the release carries, and
//! nothing else (45, C.10; design D3):
//!
//! ```toml
//! [environments.flume.github]
//! channel = "stable"
//! target.github_release = { credential = "flume-releases", assets = ["flume-x86_64-linux.tar.gz"] }
//! ```
//!
//! The token is the host's: the bindings file's `[credentials]` names the
//! path of the file holding it. `gh` takes a token only in a variable, so
//! the target reads the file when current, apply or report runs, holds
//! the token for that operation alone, and hands it to that operation's
//! runs of `gh` in `GH_TOKEN`, writing it nowhere (64, record 0033, rule
//! core.21; design D4). Every run is given nothing of the host's own `gh`
//! (see [`gh`]).
//!
//! Apply runs these steps, and stops at the first that fails, answering it
//! (90; design D6):
//!
//! 1. The table; the verified version's index, read through the profile's
//!    own store by digest, refused where it names another publishing
//!    repository than the checkout's, or none, or no revision (138).
//! 2. The version's notes and the file of each output `assets` names,
//!    read by digest through [`file_artifact`] (42, 134, rule core.4);
//!    the notes refused where they are not UTF-8.
//! 3. The token. Nothing has reached GitHub yet.
//! 4. The tag the version was claimed under, read on GitHub and peeled
//!    to its commit; the version refused where GitHub holds no such tag,
//!    or holds it at another revision than the index's (22, rule
//!    core.79).
//! 5. The tag's release, a draft or published.
//! 6. Where there is none, a draft made of the tag, titled by it, its
//!    text the notes.
//! 7. Each named asset the release holds in no uploaded asset of its
//!    name with its bytes' digest uploaded, in one run; none where every
//!    one is held, and none removed (C.12; design D7).
//! 8. The release published last, not a prerelease, marked latest,
//!    titled by its tag and its text the notes (design D8).
//!
//! No step makes, moves or deletes a tag (rule core.79).
//!
//! current and report read the release GitHub marks latest, and answer the
//! index the store resolves the environment's component's version of its
//! tag to, and report when GitHub says it was last updated; they answer
//! unread where GitHub marks none latest, or where its tag names no
//! version of the component, and never guess (89, 91, 119; design D9).
//! desired answers the head the core hands it (88).

mod gh;
#[cfg(test)]
mod scripted;
mod table;
#[cfg(test)]
mod tests;

use std::fs;
use std::path::PathBuf;

use timber_seams::{
    version_name, Answer, Digest, Environment, Head, IndexMeta, Not, RepositoryName, Revision,
    Running, Seam, Source, Store, Target, Verified, Why,
};

use super::file_artifact::{self, Words};
use crate::bindings::Credentials;
use gh::{At, Backend, Gh, Latest, Token, Upload, HOST};
use table::{Named, Table};

/// The profile name an environment's `target` table names this target by.
pub const PROFILE: &str = "github_release";

/// An asset's file, as the family module names one in an answer.
const ASSET: Words = Words {
    file: "asset file",
    carries: "asset",
};

/// A version's notes, as the family module names them in an answer.
const NOTES: Words = Words {
    file: "notes",
    carries: "notes",
};

/// The state GitHub answers an asset in once its bytes are all there.
const UPLOADED: &str = "uploaded";

/// The GitHub release target, reading through a store binding of its
/// own, opened and never made, and running `gh`.
pub struct GithubRelease {
    store: Box<dyn Store>,
    repository: Answer<RepositoryName>,
    credentials: Credentials,
    backend: Box<dyn Backend>,
    source: Source,
}

/// One named output's file, read by digest, as an asset of the release.
struct Read {
    output: timber_seams::OutputName,
    bytes: Vec<u8>,
    digest: Digest,
}

impl GithubRelease {
    /// The target reading through `store`, its releases made in
    /// `repository`, the checkout's, with the token of each environment
    /// the file of `credentials` names, and running `gh` as found on
    /// `PATH`.
    pub fn new(
        store: Box<dyn Store>,
        repository: Answer<RepositoryName>,
        credentials: &Credentials,
    ) -> GithubRelease {
        GithubRelease::with_program(store, repository, credentials, "gh")
    }

    /// The target as [`GithubRelease::new`] makes it, running `gh` as
    /// `program`.
    pub fn with_program(
        store: Box<dyn Store>,
        repository: Answer<RepositoryName>,
        credentials: &Credentials,
        program: impl Into<PathBuf>,
    ) -> GithubRelease {
        GithubRelease::with_backend(
            store,
            repository,
            credentials,
            Box::new(Gh::new(program, &GithubRelease::source())),
        )
    }

    fn with_backend(
        store: Box<dyn Store>,
        repository: Answer<RepositoryName>,
        credentials: &Credentials,
        backend: Box<dyn Backend>,
    ) -> GithubRelease {
        GithubRelease {
            store,
            repository,
            credentials: credentials.clone(),
            backend,
            source: GithubRelease::source(),
        }
    }

    /// The target seam and this profile, as every answer names them.
    pub fn source() -> Source {
        Source::new(Seam::Target, PROFILE)
    }

    /// Refused, by this target, for `said`.
    fn refused(&self, said: String) -> Not {
        Not::refused(&self.source, Why::Said, Some(said))
    }

    /// The token the table's credential names, read from the file the
    /// host's `[credentials]` gives the path of at the moment of use, its
    /// trailing whitespace trimmed (64, rule core.21; design D4):
    /// unreadable naming the credential and the bindings file where it
    /// names none, the credential and the path with the operating
    /// system's words where the file cannot be read, and the credential
    /// where it holds nothing but whitespace, so nothing runs.
    fn token(&self, environment: &Environment, table: &Table) -> Answer<Token> {
        let name = &table.credential;
        let what = || format!("credential {name} of {}", Named(environment));
        let path = self.credentials.path(name).ok_or_else(|| {
            Not::unreadable(
                &self.source,
                what(),
                Some(format!(
                    "the bindings file {} names no {name} in its [credentials], the path of the \
                     file holding the token the {PROFILE} target hands gh",
                    self.credentials.file.display()
                )),
            )
        })?;
        let text = fs::read_to_string(path).map_err(|e| {
            Not::unreadable(
                &self.source,
                format!("credential {name} at {}", path.display()),
                Some(e.to_string()),
            )
        })?;
        let token = text.trim_end();
        if token.is_empty() {
            return Err(Not::unreadable(
                &self.source,
                what(),
                Some(format!(
                    "the file {} holds no token, only whitespace",
                    path.display()
                )),
            ));
        }
        Ok(Token::new(token))
    }

    /// The release GitHub marks latest in the checkout's repository, and
    /// the index the store resolves the environment's component's version
    /// of its tag to; unread naming why where GitHub marks none latest, or
    /// where its tag names no version of the component (89, 119; design
    /// D9).
    fn running(&self, environment: &Environment) -> Answer<(Digest, Latest)> {
        let table = Table::read(environment, &self.source)?;
        let repository = self.repository.clone()?;
        let token = self.token(environment, &table)?;
        let at = At {
            repository: &repository,
            token: &token,
        };
        let unread = |said: String| {
            Not::unreadable(
                &self.source,
                format!("what {} runs", Named(environment)),
                Some(said),
            )
        };
        let Some(latest) = self.backend.latest(&at)? else {
            return Err(unread(format!(
                "{HOST}/{repository} marks no release latest, so no version is released there"
            )));
        };
        let component = environment.channel.component_name();
        let name = version_name(&component, &latest.tag);
        match self.store.resolve(&name) {
            Ok(digest) => Ok((digest, latest)),
            Err(Not::Absent(_)) => Err(unread(format!(
                "the release {HOST}/{repository} marks latest is of the tag {}, which names no \
                 version of {component} in the store: a release made by hand, or another \
                 component's",
                latest.tag
            ))),
            Err(not) => Err(not),
        }
    }

    /// The index of `version` read by digest, and the revision it names;
    /// refused where it names another publishing repository than the
    /// checkout's, `repository`, or none, or no revision, since the
    /// release goes where the publish pushed the tag (138, rule core.60;
    /// design D5).
    fn index(
        &self,
        repository: &RepositoryName,
        version: &Verified,
    ) -> Answer<(IndexMeta, Revision)> {
        let index = file_artifact::index(self.store.as_ref(), version, &self.source)?;
        let (digest, number) = (version.index(), version.version());
        match index.repository() {
            Some(named) if named == repository => {}
            Some(named) => {
                return Err(self.refused(format!(
                    "the index {digest} of version {number} names the publishing repository \
                     {named}, and this checkout is of {repository}, whose releases the \
                     {PROFILE} target makes"
                )))
            }
            None => {
                return Err(self.refused(format!(
                    "the index {digest} of version {number} names no publishing repository, and \
                     the release is made in this checkout's, {repository}"
                )))
            }
        }
        let Some(revision) = index.revision().cloned() else {
            return Err(self.refused(format!(
                "the index {digest} of version {number} names no revision, so the tag {} \
                 cannot be checked against it on GitHub",
                version.tag()
            )));
        };
        Ok((index, revision))
    }

    /// The version's notes, read by digest through the family module and
    /// refused where the index names none or they are not UTF-8 (134;
    /// design D10).
    fn notes(&self, index: &IndexMeta, version: &Verified) -> Answer<String> {
        let artifact = file_artifact::notes_of(index, NOTES, &self.source)?;
        let bytes = file_artifact::bytes(self.store.as_ref(), artifact, NOTES, &self.source)?;
        String::from_utf8(bytes).map_err(|e| {
            self.refused(format!(
                "the notes of version {} are not UTF-8, and a release's text is: {e}",
                version.version()
            ))
        })
    }

    /// The file of each output the table names, read by digest through
    /// the family module (42, rule core.4).
    fn assets(&self, index: &IndexMeta, table: &Table) -> Answer<Vec<Read>> {
        let mut read = Vec::new();
        for output in &table.assets {
            let artifact = file_artifact::artifact_of(index, output, ASSET, &self.source)?;
            let bytes = file_artifact::bytes(self.store.as_ref(), artifact, ASSET, &self.source)?;
            read.push(Read {
                output: output.clone(),
                digest: Digest::of(&bytes),
                bytes,
            });
        }
        Ok(read)
    }

    /// The version refused where GitHub holds its tag at no commit or at
    /// another than `revision`, the index's, naming the tag and each
    /// revision (22, rule core.79).
    fn tagged(&self, at: &At<'_>, version: &Verified, revision: &Revision) -> Answer<()> {
        let (tag, repository) = (version.tag(), at.repository);
        match self.backend.tag(at, tag)? {
            None => Err(self.refused(format!(
                "{HOST}/{repository} holds no tag {tag}, which version {} was claimed under, and \
                 a release is made only on the tag GitHub holds at the version's revision",
                version.version()
            ))),
            Some(held) if held != *revision => Err(self.refused(format!(
                "{HOST}/{repository} holds the tag {tag} at {held}, and the index of version {} \
                 names the revision {revision}",
                version.version()
            ))),
            Some(_) => Ok(()),
        }
    }
}

impl Target for GithubRelease {
    /// 88: the head the core handed, as a target that is pushed to answers
    /// it, once the table is read; no credential is read.
    fn desired(&self, environment: &Environment, head: Head<'_>) -> Answer<Digest> {
        Table::read(environment, &self.source)?;
        head()
    }

    /// 89: the index of the version the release GitHub marks latest is of.
    fn current(&self, environment: &Environment) -> Answer<Digest> {
        self.running(environment).map(|(index, _)| index)
    }

    /// 90: the version's release made or updated on its tag, a draft until
    /// its assets are attached, and published last, marked latest (design
    /// D6).
    fn apply(&self, environment: &Environment, version: &Verified) -> Answer<()> {
        let table = Table::read(environment, &self.source)?;
        let repository = self.repository.clone()?;
        let (index, revision) = self.index(&repository, version)?;
        let notes = self.notes(&index, version)?;
        let assets = self.assets(&index, &table)?;
        let token = self.token(environment, &table)?;
        let at = At {
            repository: &repository,
            token: &token,
        };
        self.tagged(&at, version, &revision)?;
        let tag = version.tag();
        let held = match self.backend.release(&at, tag)? {
            Some(release) => release.assets,
            None => {
                self.backend.create(&at, tag, notes.as_bytes())?;
                Vec::new()
            }
        };
        let lacking: Vec<Upload<'_>> = assets
            .iter()
            .filter(|asset| {
                !held.iter().any(|h| {
                    h.name == asset.output.as_str()
                        && h.state == UPLOADED
                        && h.digest.as_ref() == Some(&asset.digest)
                })
            })
            .map(|asset| Upload {
                output: &asset.output,
                bytes: &asset.bytes,
            })
            .collect();
        if !lacking.is_empty() {
            self.backend.upload(&at, tag, &lacking)?;
        }
        self.backend.publish(&at, tag, notes.as_bytes())
    }

    /// 91: the index of the version the release GitHub marks latest is of,
    /// and when GitHub says that release was last updated.
    fn report(&self, environment: &Environment) -> Answer<Running> {
        let (digest, latest) = self.running(environment)?;
        let converged_at = latest.updated_at.ok_or_else(|| {
            Not::unreadable(
                &self.source,
                format!("when {} converged", Named(environment)),
                Some(format!(
                    "GitHub answered the latest release, of the tag {}, with no updated_at, so \
                     when it was last written is not known",
                    latest.tag
                )),
            )
        })?;
        Ok(Running {
            digest,
            converged_at,
        })
    }
}
