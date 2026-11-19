//! The store seam (B.1) over an OCI registry, spoken to over the
//! distribution protocol (C.1, ADR 0003): bytes as blobs, an artifact or
//! an index as a manifest put by its own digest in the bytes the layout
//! writes, an attestation as a referrer of its subject, and a name as a
//! tag, at bytes through a cache manifest (design D2 to D8, D14).
//!
//! The host names a registry and a root, `<host>[:<port>]/<root>`, and
//! every name of the store is a tag of a repository under the root (C.1
//! as amended at `boomdrivedev/blueprints` f412008; record 0040). The
//! store is bound for one repository, the checkout's, whose own
//! repository is `<root>/<owner>/<name>`, the owner and name its
//! landing's remote gives, lowercased (114, 138). Three spaces hang under
//! the root: a declared component's versions and channels, tags of the
//! component's repository under the repository's own; each cache entry,
//! a tag of its build profile's cache repository, `cache/<build>` under
//! the repository's own; and each artifact's name, a tag of
//! `<root>/artifact/<ecosystem>/<name>`, one space for the whole store
//! and under no repository's own. The repository's own holds the
//! undeclared component's versions and channels, and what a publish
//! writes before any name reaches it. A store bound for no repository
//! answers every write, and every name of the repository's own spaces,
//! as its binding answered the repository, and reads the artifact space
//! as any store does.
//!
//! A root the repository left holds the mark `moved` in its own, at
//! bytes naming the root its history moved to (C.1 as amended at
//! `boomdrivedev/blueprints` e8700c8; record 0042). The store reads the
//! mark once in its life, at its first operation reaching the
//! repository's own, its components' or its cache repositories, and where
//! it names another root every such operation is refused naming the
//! repository the store is bound for and where it moved, with nothing
//! written; the artifact space is read as before. A store carrying a
//! history to a root reads past a mark naming that root alone. A push's
//! store reads no mark and writes its cache entries at a root left as at
//! any root: the mark guards history, and cache entries are none (135,
//! rule core.62; record 0042).
//!
//! The store never makes a repository. A loopback host is spoken to over
//! plain HTTP, as Docker speaks to one, and every other over HTTPS
//! (design D3). A digest or name the registry does not hold is absent; a
//! registry that refuses is refused in its own words; one that cannot be
//! reached, or answers what cannot be read, or fails with a 5xx, is
//! unreadable in the client's or the registry's words (58, 59, rule
//! core.19).

mod client;
mod tag;

#[cfg(test)]
pub(crate) mod stub;

use std::cell::RefCell;
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;
use timber_seams::{
    moved_name, Answer, BuildId, Digest, Ecosystem, EcosystemName, Envelope, Held, Manifest,
    MovedTo, Name, Not, Pushed, Put, RepositoryName, Seam, Source, Store, Version, Why,
    REPOSITORY_COMPONENT,
};
use ureq::http::{Method, StatusCode};

use super::oci::{
    self, Descriptor, Referrers, EMPTY_JSON, ENVELOPE, IMAGE_INDEX, IMAGE_MANIFEST, OCTETS,
};
use client::{encoded, Answered, Client};

/// What a cache entry's name begins with, and the space of repositories
/// under the repository's own its entries are held in, one per build
/// profile (52, C.1; design D9).
const CACHE_SPACE: &str = "cache";

/// What a manifest request accepts: timber's two manifest types.
const MANIFESTS: &str = "application/vnd.oci.image.manifest.v1+json, \
                         application/vnd.oci.image.index.v1+json";

/// How long name reads a tag again while it answers the place it was
/// moved from: a registry may answer a tag's former digest for seconds
/// after the put, ECR for over ten (C.1, ADR 0015).
const BOUND: Duration = Duration::from_secs(30);

/// The wait before a tag is read again the first time; each wait after
/// is twice the one before.
const FIRST_WAIT: Duration = Duration::from_millis(250);

/// A store held under one root of an OCI registry, bound for one
/// repository. Every answer names the store seam and the registry and
/// root.
pub struct Registry {
    repository: String,
    host: String,
    root: String,
    own: Answer<String>,
    origin: String,
    base: String,
    source: Source,
    client: Client,
    hook: Option<Box<dyn Fn(&str)>>,
    bound: Duration,
    build: Option<String>,
    push: bool,
    mounts: RefCell<Vec<(String, bool)>>,
    reached: RefCell<Vec<(Digest, String)>>,
    named_for: Option<RepositoryName>,
    carrying: Option<RepositoryName>,
    mark: RefCell<Option<Answer<()>>>,
}

/// The space of repositories under the root an artifact's names are held
/// in, one per ecosystem and name, under no repository's own (138, C.1;
/// design D8).
const ARTIFACT_SPACE: &str = "artifact";

/// Where a name is held: the repository path it is a tag of, the tag,
/// and the space the repository is in (design D8 to D10).
struct Place {
    repository: String,
    tag: String,
    space: Space,
}

/// The spaces of the store's names on a registry (C.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Space {
    Own,
    Artifact,
    Cache,
    Component,
}

/// The few fields of a manifest's JSON the store reads before its form.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Typed {
    #[serde(default)]
    media_type: Option<String>,
    #[serde(default)]
    artifact_type: Option<String>,
}

/// Whether `host`, with its port, is the loopback interface's.
fn loopback(host: &str) -> bool {
    let name = match host.strip_prefix('[') {
        Some(rest) => rest.split(']').next().map(|h| format!("[{h}]")),
        None => host.split(':').next().map(str::to_string),
    };
    matches!(
        name.as_deref(),
        Some("localhost") | Some("127.0.0.1") | Some("[::1]")
    )
}

impl Registry {
    /// The registry and the root `repository` names,
    /// `<host>[:<port>]/<root>`, bound for no repository, so every write
    /// and every name of the repository's own spaces is absent naming the
    /// repository it is bound for until [`Registry::for_repository`]
    /// gives one; refused naming the text when it names no host or no
    /// root (C.1; design D1).
    pub fn new(repository: &str) -> Answer<Registry> {
        let source = Source::new(Seam::Store, format!("OCI registry {repository}"));
        let refused = |why: &str| {
            Not::refused(
                &source,
                Why::Said,
                Some(format!(
                    "{repository:?} is not a registry host followed by a root's path: {why}"
                )),
            )
        };
        let Some((host, root)) = repository.split_once('/') else {
            return Err(refused("it holds no /"));
        };
        if host.is_empty() || root.is_empty() {
            return Err(refused("its host or its root is empty"));
        }
        if host.contains("://") || repository.contains(char::is_whitespace) {
            return Err(refused("it names a scheme or holds a space"));
        }
        if root.split('/').any(str::is_empty) {
            return Err(refused("its root holds an empty part"));
        }
        let scheme = if loopback(host) { "http" } else { "https" };
        let origin = format!("{scheme}://{host}");
        Ok(Registry {
            repository: repository.to_string(),
            host: host.to_string(),
            root: root.to_string(),
            own: Err(Not::absent(&source, "repository this store is bound for")),
            base: format!("{origin}/v2/{root}/"),
            origin,
            client: Client::new(host, &source),
            source,
            hook: None,
            bound: BOUND,
            build: None,
            push: false,
            mounts: RefCell::new(Vec::new()),
            reached: RefCell::new(Vec::new()),
            named_for: None,
            carrying: None,
            mark: RefCell::new(None),
        })
    }

    /// The registry bound for `repository`, the checkout's, as its
    /// binding answered it: its own repository `<root>/<owner>/<name>`,
    /// the owner and the name each written as one path component,
    /// lowercased; and where the answer is absent or unreadable, that
    /// answer kept, every write and every name of the repository's own
    /// spaces answering it. An owner written `artifact` is refused naming
    /// it and the artifact space, `<root>/artifact/`, which its own
    /// repository would share, so nothing is written (114, 138, C.1;
    /// design D2, D3).
    pub fn for_repository(mut self, repository: Answer<RepositoryName>) -> Registry {
        self.named_for = repository.as_ref().ok().map(RepositoryName::lowercased);
        self.own = repository.and_then(|named| {
            if tag::component(&named.owner) == ARTIFACT_SPACE {
                return Err(Not::refused(
                    &self.source,
                    Why::Said,
                    Some(format!(
                        "the owner {} of the repository {named} is written {ARTIFACT_SPACE}, \
                         and {}/{}/{ARTIFACT_SPACE}/ is the artifact space, which the \
                         repository's own would share",
                        named.owner, self.host, self.root
                    )),
                ));
            }
            self.under(
                &[named.owner.as_str(), named.name.as_str()],
                &format!("the repository {named}"),
            )
        });
        self
    }

    /// The store carrying a history to `to`: it reads and writes its
    /// repository's own spaces past a mark naming `to`, and is refused
    /// there as any store is where the mark names another root (C.1;
    /// design D4).
    pub fn carrying_to(mut self, to: RepositoryName) -> Registry {
        self.carrying = Some(to.lowercased());
        self
    }

    /// The registry bound for the build profile `build`, whose cache
    /// repository, `cache/<build>` under the repository's own, holds each
    /// cache entry; and, where `push`, putting bytes there too, since a
    /// push writes cache entries and nothing more, so a credential that
    /// writes `cache/` alone pushes them (52, 135, C.1; design D9).
    pub fn bound_for(mut self, build: &str, push: bool) -> Registry {
        self.build = Some(build.to_string());
        self.push = push;
        self
    }

    /// The store seam and this profile, naming the repository.
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// The registry and the root, as the host names them.
    pub fn repository(&self) -> &str {
        &self.repository
    }

    /// Where the API of the repositories under the root is:
    /// `<scheme>://<host>/v2/<root>/`.
    pub fn base(&self) -> &str {
        &self.base
    }

    /// `hook` run between a tag's put and its read back, with the tag,
    /// so a test can lose a race on purpose (design D7).
    #[doc(hidden)]
    pub fn between_put_and_read_back(&mut self, hook: impl Fn(&str) + 'static) {
        self.hook = Some(Box::new(hook));
    }

    /// Each mount this store asked, by the repository it was into, and
    /// whether the registry took it, so a run against a registry reports
    /// whether it takes a cross-repository mount (design D8, ADR 0023).
    #[doc(hidden)]
    pub fn mounts(&self) -> Vec<(String, bool)> {
        self.mounts.borrow().clone()
    }

    /// How long a tag may answer a place it was at after a move, before
    /// or after one read has shown the move: the profile's bound, 30 s,
    /// or the one a test set (C.1, 36, ADR 0015).
    pub fn bound(&self) -> Duration {
        self.bound
    }

    /// Whether a name moves only from the digest it is expected at by the
    /// registry's own condition on the put: never, since the distribution
    /// specification defines no conditional tag put and no registry this
    /// profile binds honours one. name's check before the put is a read a
    /// lagging replica defeats, so its refusal of a move from a place the
    /// name has left is best effort (C.1, ADR 0015).
    pub fn names_conditional(&self) -> bool {
        false
    }

    /// A tag read again for at most `bound` while it answers the place
    /// it was moved from, in place of the profile's, so a test sees the
    /// bound pass (ADR 0015).
    #[doc(hidden)]
    pub fn read_again_within(&mut self, bound: Duration) {
        self.bound = bound;
    }

    /// Where `path` of `repository`'s API is: `<scheme>://<host>/v2/<repository>/<path>`.
    fn url_in(&self, repository: &str, path: &str) -> String {
        format!("{}/v2/{repository}/{path}", self.origin)
    }

    /// `location`, as a registry answers it, as a URL.
    fn absolute(&self, location: &str) -> String {
        if location.starts_with("http://") || location.starts_with("https://") {
            location.to_string()
        } else if location.starts_with('/') {
            format!("{}{location}", self.origin)
        } else {
            format!("{}/{location}", self.origin)
        }
    }

    /// `method` of `url`, a request for `repository`, as the client
    /// sends it (design D8).
    fn send(
        &self,
        repository: &str,
        method: Method,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
    ) -> Answer<Answered> {
        self.client.send(repository, method, url, headers, body)
    }

    /// What a `HEAD` of `repository` the operation does not take is: what
    /// a `GET` of the same URL answers, which carries the registry's words
    /// where the `HEAD` carried none.
    fn other_head(
        &self,
        repository: &str,
        url: &str,
        accept: &[(&str, &str)],
        head: &Answered,
    ) -> Not {
        match self.send(repository, Method::GET, url, accept, None) {
            Ok(got) if !got.status.is_success() => self.other(url, &got),
            Ok(_) => self.other(url, head),
            Err(not) => not,
        }
    }

    /// What an answer the operation does not take is: refused in the
    /// registry's words, or unreadable naming `url` for a 5xx.
    fn other(&self, url: &str, answered: &Answered) -> Not {
        if answered.status.is_server_error() {
            Not::unreadable(&self.source, url, Some(answered.words()))
        } else {
            Not::refused(&self.source, Why::Said, Some(answered.words()))
        }
    }

    /// What a write to `repository` the registry did not take is: as
    /// [`Registry::other`] answers it, and, where the registry answers
    /// it 404, holding no such repository, as one that makes repositories
    /// only by a creation template does where none applies, refused
    /// naming the repository the write was for beside the registry's own
    /// words, so an operator reads which one to make; nothing is asked
    /// again (58, 59, C.1; design D13).
    fn other_put(&self, repository: &str, url: &str, answered: &Answered) -> Not {
        if answered.status != StatusCode::NOT_FOUND {
            return self.other(url, answered);
        }
        Not::refused(
            &self.source,
            Why::Said,
            Some(format!(
                "the registry holds no repository {}/{repository}, which the write was for: {}",
                self.host,
                answered.words()
            )),
        )
    }

    /// That `bytes`, answered for `digest`, are its bytes; refused naming
    /// both otherwise (2).
    fn same(&self, digest: &Digest, bytes: &[u8]) -> Answer<()> {
        let answered = Digest::of(bytes);
        if &answered != digest {
            return Err(Not::refused(
                &self.source,
                Why::OtherBytes {
                    expected: digest.clone(),
                    answered,
                },
                None,
            ));
        }
        Ok(())
    }

    /// That the digest a put's answer names, where it names one, is the
    /// digest of what was put; refused naming both otherwise (1).
    fn named(&self, digest: &Digest, answered: &Answered) -> Answer<()> {
        match answered
            .header("docker-content-digest")
            .and_then(|d| Digest::parse(d).ok())
        {
            Some(named) if &named != digest => Err(Not::refused(
                &self.source,
                Why::OtherBytes {
                    expected: digest.clone(),
                    answered: named,
                },
                None,
            )),
            _ => Ok(()),
        }
    }

    /// The size of the blob held under `digest`; none when it is not.
    fn blob_size(&self, repository: &str, digest: &Digest) -> Answer<Option<u64>> {
        let url = self.url_in(repository, &format!("blobs/{digest}"));
        let answered = self.send(repository, Method::HEAD, &url, &[], None)?;
        match answered.status {
            StatusCode::OK => self.length(&url, &answered).map(Some),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(self.other_head(repository, &url, &[], &answered)),
        }
    }

    /// The media type and size of the manifest `reference`, a digest or a
    /// tag, names; none when it names none.
    fn manifest_head(
        &self,
        repository: &str,
        reference: &str,
    ) -> Answer<Option<(String, u64, Option<Digest>)>> {
        let url = self.url_in(repository, &format!("manifests/{reference}"));
        let accept = [("accept", MANIFESTS)];
        let answered = self.send(repository, Method::HEAD, &url, &accept, None)?;
        match answered.status {
            StatusCode::OK => {
                let size = self.length(&url, &answered)?;
                let media_type = answered
                    .header("content-type")
                    .map(|t| t.split(';').next().unwrap_or(t).trim().to_string())
                    .unwrap_or_else(|| IMAGE_MANIFEST.into());
                let digest = answered
                    .header("docker-content-digest")
                    .and_then(|d| Digest::parse(d).ok());
                Ok(Some((media_type, size, digest)))
            }
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(self.other_head(repository, &url, &accept, &answered)),
        }
    }

    fn length(&self, url: &str, answered: &Answered) -> Answer<u64> {
        answered
            .header("content-length")
            .and_then(|l| l.parse().ok())
            .ok_or_else(|| {
                Not::unreadable(
                    &self.source,
                    url,
                    Some("the registry answered no Content-Length".into()),
                )
            })
    }

    /// `bytes` uploaded as the blob `digest` in one request after the
    /// upload is opened (design D5).
    fn upload(&self, repository: &str, digest: &Digest, bytes: &[u8]) -> Answer<()> {
        let url = self.url_in(repository, "blobs/uploads/");
        let opened = self.send(repository, Method::POST, &url, &[], Some(&[]))?;
        if opened.status != StatusCode::ACCEPTED {
            return Err(self.other_put(repository, &url, &opened));
        }
        let location = opened.header("location").ok_or_else(|| {
            Not::unreadable(
                &self.source,
                &url,
                Some("the registry opened an upload and answered no Location".into()),
            )
        })?;
        self.upload_to(repository, location, digest, bytes)
    }

    /// `bytes` put as the blob `digest` of `repository` through the
    /// upload the registry opened at `location`, in one request (design
    /// D5).
    fn upload_to(
        &self,
        repository: &str,
        location: &str,
        digest: &Digest,
        bytes: &[u8],
    ) -> Answer<()> {
        let location = self.absolute(location);
        let sep = if location.contains('?') { '&' } else { '?' };
        let url = format!("{location}{sep}digest={}", encoded(digest.as_str()));
        let put = self.send(
            repository,
            Method::PUT,
            &url,
            &[("content-type", OCTETS)],
            Some(bytes),
        )?;
        if !put.status.is_success() {
            return Err(self.other_put(repository, &url, &put));
        }
        self.named(digest, &put)
    }

    /// `bytes` of `media_type` put as the manifest `reference`, a digest
    /// or a tag, answering what the registry answered.
    fn put_manifest(
        &self,
        repository: &str,
        reference: &str,
        media_type: &str,
        bytes: &[u8],
    ) -> Answer<Answered> {
        let url = self.url_in(repository, &format!("manifests/{reference}"));
        let put = self.send(
            repository,
            Method::PUT,
            &url,
            &[("content-type", media_type)],
            Some(bytes),
        )?;
        if !put.status.is_success() {
            return Err(self.other_put(repository, &url, &put));
        }
        self.named(&Digest::of(bytes), &put)?;
        Ok(put)
    }

    /// The bytes of the blob `digest`, once they are found to be its;
    /// none when it is not held. A redirect, as ECR's to its bucket, is
    /// followed, and the digest checked on what finally arrives (design
    /// D3). A `GET` answered 404 is asked after by a `HEAD`: where that
    /// answers the blob held, as GitHub's registry does in a package its
    /// push made that holds no manifest yet, the bytes are unreadable in
    /// the registry's words, never absent (59, rule core.19; record 0039).
    fn blob(&self, repository: &str, digest: &Digest) -> Answer<Option<Vec<u8>>> {
        let url = self.url_in(repository, &format!("blobs/{digest}"));
        let answered = self.send(repository, Method::GET, &url, &[], None)?;
        match answered.status {
            StatusCode::OK => {
                self.same(digest, &answered.body)?;
                Ok(Some(answered.body))
            }
            StatusCode::NOT_FOUND => match self.blob_size(repository, digest)? {
                None => Ok(None),
                Some(_) => Err(Not::unreadable(
                    &self.source,
                    &url,
                    Some(format!(
                        "the registry answers a HEAD of the blob that it holds it, and its GET \
                         404: {}",
                        answered.words()
                    )),
                )),
            },
            _ => Err(self.other(&url, &answered)),
        }
    }

    /// The bytes of the manifest `digest` of `repository` and the media
    /// type they name, once they are found to be its; none when it is not
    /// held. Each digest the manifest names is kept as reached in
    /// `repository` (design D4).
    fn manifest(&self, repository: &str, digest: &Digest) -> Answer<Option<(Vec<u8>, Typed)>> {
        let url = self.url_in(repository, &format!("manifests/{digest}"));
        let answered = self.send(
            repository,
            Method::GET,
            &url,
            &[("accept", MANIFESTS)],
            None,
        )?;
        match answered.status {
            StatusCode::OK => {
                self.same(digest, &answered.body)?;
                let mut typed: Typed = serde_json::from_slice(&answered.body)
                    .map_err(|e| Not::unreadable(&self.source, &url, Some(e.to_string())))?;
                if typed.media_type.is_none() {
                    typed.media_type = answered
                        .header("content-type")
                        .map(|t| t.split(';').next().unwrap_or(t).trim().to_string());
                }
                for (_, named) in oci::named(&answered.body).unwrap_or_default() {
                    self.keep(&named, repository);
                }
                Ok(Some((answered.body, typed)))
            }
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(self.other(&url, &answered)),
        }
    }

    /// The tag a name is held under; refused naming the name when no
    /// tag can hold it (design D8).
    fn tag(&self, name: &Name) -> Answer<String> {
        tag::of(name).map_err(|words| Not::refused(&self.source, Why::Said, Some(words)))
    }

    /// The digest `tag` names; none when it names nothing. At a cache
    /// manifest it is the digest of the bytes the manifest names (design
    /// D14); at any other manifest the one the registry names it by, or
    /// the digest of the manifest's bytes where it names none.
    fn tag_at(&self, repository: &str, tag: &str) -> Answer<Option<Digest>> {
        let url = self.url_in(repository, &format!("manifests/{tag}"));
        let answered = self.send(
            repository,
            Method::GET,
            &url,
            &[("accept", MANIFESTS)],
            None,
        )?;
        match answered.status {
            StatusCode::OK => {}
            StatusCode::NOT_FOUND => return Ok(None),
            _ => return Err(self.other(&url, &answered)),
        }
        let cached = oci::cached(&answered.body)
            .map_err(|words| Not::unreadable(&self.source, &url, Some(words)))?;
        Ok(Some(cached.unwrap_or_else(|| {
            answered
                .header("docker-content-digest")
                .and_then(|d| Digest::parse(d).ok())
                .unwrap_or_else(|| Digest::of(&answered.body))
        })))
    }

    /// The cache manifest of the bytes held under `digest`, `size` bytes
    /// long, held in the repository: the empty config put, and the
    /// manifest put by its digest unless it is held already (design
    /// D14).
    fn cache(&self, repository: &str, digest: &Digest, size: u64) -> Answer<Vec<u8>> {
        let bytes = oci::cache(digest, size);
        let at = Digest::of(&bytes);
        if self.manifest_head(repository, at.as_str())?.is_none() {
            self.put_bytes_in(repository, EMPTY_JSON)?;
            self.put_manifest(repository, at.as_str(), IMAGE_MANIFEST, &bytes)?;
        }
        Ok(bytes)
    }

    /// `bytes` held as a blob of `repository`: asked after first, and
    /// uploaded only when it is not held (5, 66).
    fn put_bytes_in(&self, repository: &str, bytes: &[u8]) -> Answer<Put> {
        let digest = Digest::of(bytes);
        if self.blob_size(repository, &digest)?.is_some() {
            return Ok(Put {
                digest,
                held_already: true,
            });
        }
        self.upload(repository, &digest, bytes)?;
        Ok(Put {
            digest,
            held_already: false,
        })
    }

    /// `digest` kept as reached in `repository`, after every repository
    /// it was reached in before, for the store's life alone: the next
    /// process keeps it again from the names it resolves, so it is no
    /// record (rule core.20; design D4).
    fn keep(&self, digest: &Digest, repository: &str) {
        let mut reached = self.reached.borrow_mut();
        if !reached.iter().any(|(d, r)| d == digest && r == repository) {
            reached.push((digest.clone(), repository.to_string()));
        }
    }

    /// The repositories `digest` was reached in, in the order reached.
    fn reached_in(&self, digest: &Digest) -> Vec<String> {
        self.reached
            .borrow()
            .iter()
            .filter(|(d, _)| d == digest)
            .map(|(_, r)| r.clone())
            .collect()
    }

    /// The first repository `ask` finds `digest` in, with what it found:
    /// each repository the digest was reached in first, in the order
    /// reached, then the repository's own, then the bound build
    /// profile's cache repository, each asked once; none where none
    /// holds it, and the binding's answer where the store is bound for no
    /// repository and no repository the digest was reached in holds it
    /// (72, 138, C.1; design D4).
    fn find<T>(
        &self,
        digest: &Digest,
        ask: impl Fn(&str) -> Answer<Option<T>>,
    ) -> Answer<Option<(String, T)>> {
        let reached = self.reached_in(digest);
        for repository in &reached {
            if let Some(found) = ask(repository)? {
                return Ok(Some((repository.clone(), found)));
            }
        }
        let mut rest = vec![self.own()?.to_string()];
        rest.extend(self.cache_repository()?);
        for repository in rest.into_iter().filter(|r| !reached.contains(r)) {
            if let Some(found) = ask(&repository)? {
                return Ok(Some((repository, found)));
            }
        }
        Ok(None)
    }

    /// The repository path `parts` make under the root, each written as
    /// one path component, for what `held` names as written; refused
    /// naming it, the path and its length where the path runs past 255
    /// characters (design D12).
    fn under(&self, parts: &[&str], held: &str) -> Answer<String> {
        tag::repository(&self.host, &self.root, parts, held)
            .map_err(|words| Not::refused(&self.source, Why::Said, Some(words)))
    }

    /// The repository's own, `<root>/<owner>/<name>`; what the binding
    /// answered for the repository where it gave none, so every write
    /// and every name of the repository's own spaces answers it and
    /// nothing is written (C.1; design D2, D3).
    fn own(&self) -> Answer<&str> {
        let own = self.own.as_deref().map_err(Clone::clone)?;
        // A push's store asks nothing of the repository's own, which its
        // credential may not read (135, rule core.62; record 0040), and
        // reads no mark: a root left refuses history, and a push's cache
        // entries are none (record 0042).
        if !self.push {
            self.unmarked(own)?;
        }
        Ok(own)
    }

    /// C.1, design D4: whether the repository's own holds no mark of a
    /// root left, or one naming the root this store carries to, read once
    /// in the store's life and answered the same after; refused naming
    /// the repository and where it moved where the mark names another
    /// root.
    fn unmarked(&self, own: &str) -> Answer<()> {
        if let Some(answer) = &*self.mark.borrow() {
            return answer.clone();
        }
        let answer = self.read_mark(own);
        *self.mark.borrow_mut() = Some(answer.clone());
        answer
    }

    /// The mark `moved` in `own` read: no tag there is no mark; bytes
    /// that name no root, or a tag or bytes that cannot be read, are
    /// unreadable naming the mark (58, 59).
    fn read_mark(&self, own: &str) -> Answer<()> {
        let tag = self.tag(&moved_name())?;
        let what = format!("the mark {tag} of {}/{own}", self.host);
        let unreadable = |not: Not| match not {
            Not::Unreadable(u) => Not::unreadable(&self.source, what.clone(), u.said),
            other => other,
        };
        let Some(at) = self.tag_at(own, &tag).map_err(unreadable)? else {
            return Ok(());
        };
        let bytes = self.blob(own, &at).map_err(unreadable)?;
        let Some(mark) = bytes.as_deref().and_then(MovedTo::of_bytes) else {
            return Err(Not::unreadable(
                &self.source,
                what,
                Some(format!(
                    "{tag} names {at}, which holds no mark naming a root"
                )),
            ));
        };
        let to = mark.to.lowercased();
        if self.carrying.as_ref() == Some(&to) {
            return Ok(());
        }
        let from = self
            .named_for
            .clone()
            .expect("a store with a repository's own is bound for that repository");
        Err(Not::refused(
            &self.source,
            Why::RootMoved { from, to },
            None,
        ))
    }

    /// The repository path `parts` make under the repository's own, as
    /// [`Registry::under`] makes one under the root.
    fn under_own(&self, parts: &[&str], held: &str) -> Answer<String> {
        tag::repository(&self.host, self.own()?, parts, held)
            .map_err(|words| Not::refused(&self.source, Why::Said, Some(words)))
    }

    /// Where `name` is held (design D3): the repository it is a tag of,
    /// and the tag, read as an artifact's name first, then a cache
    /// entry's, a channel's and a version's, so a `:` in a build identity
    /// or a version is never read as a component's. An artifact's name is
    /// the tag of its version in `artifact/<ecosystem>/<name>` under the
    /// root, under no repository's own; a cache entry's the tag of its
    /// build identity in the bound build profile's cache repository under
    /// the repository's own, refused naming the entry where the store is
    /// bound for none; a declared component's channel and version the
    /// tags of the channel's name and the version's tag in the
    /// component's repository under the repository's own; every other
    /// name, the undeclared component's versions and channels among them,
    /// a tag of the repository's own (125, 138, C.1). A name outside the
    /// artifact space answers the binding's answer where the store is
    /// bound for no repository.
    fn place(&self, name: &Name) -> Answer<Place> {
        if let Some((ecosystem, artifact, version)) = name.as_artifact() {
            return Ok(Place {
                repository: self.under(
                    &[ARTIFACT_SPACE, ecosystem.as_str(), artifact.as_str()],
                    &format!("the name {name}"),
                )?,
                tag: self.tag(&Name::new(version.as_str()))?,
                space: Space::Artifact,
            });
        }
        if let Some(identity) = name
            .as_str()
            .strip_prefix(CACHE_SPACE)
            .and_then(|rest| rest.strip_prefix('/'))
        {
            let repository = self.cache_repository()?.ok_or_else(|| {
                Not::refused(
                    &self.source,
                    Why::Said,
                    Some(format!(
                        "{name} is a cache entry's name, and this store is bound for no build \
                         profile, whose cache repository holds its entries"
                    )),
                )
            })?;
            return Ok(Place {
                repository,
                tag: self.tag(&Name::new(identity))?,
                space: Space::Cache,
            });
        }
        let component = match (name.as_channel(), name.as_version()) {
            (Some(channel), _) => channel
                .component
                .map(|component| (component, channel.name.to_string())),
            (None, Some((component, tag))) => Some((component, tag)),
            (None, None) => None,
        };
        if let Some((component, tag)) = component {
            return Ok(Place {
                repository: self.under_own(&[component.as_str()], &format!("the name {name}"))?,
                tag: self.tag(&Name::new(tag))?,
                space: Space::Component,
            });
        }
        Ok(Place {
            repository: self.own()?.to_string(),
            tag: self.tag(name)?,
            space: Space::Own,
        })
    }

    /// The bound build profile's cache repository, `cache/<build>` under
    /// the repository's own; none where the store is bound for none
    /// (design D3, D9).
    fn cache_repository(&self) -> Answer<Option<String>> {
        self.build
            .as_deref()
            .map(|build| {
                self.under_own(
                    &[CACHE_SPACE, build],
                    &format!("the cache of the build profile {build}"),
                )
            })
            .transpose()
    }

    /// The manifest `digest` held in `repository` with everything it
    /// names and timber's attestations on each, each asked after by a
    /// `HEAD` first and only what the repository lacks written: the
    /// manifest found as a read finds it, each manifest an index names
    /// held the same way, each blob mounted from the repository the
    /// manifest was found in, then the manifest put by its digest in its
    /// own bytes, since a registry takes a manifest only where what it
    /// names is held, and then its attestations carried from there (8,
    /// C.1, rule core.5; design D4, D5).
    fn hold(&self, repository: &str, digest: &Digest) -> Answer<()> {
        if self.manifest_head(repository, digest.as_str())?.is_some() {
            return Ok(());
        }
        let Some((from, (bytes, typed))) = self.find(digest, |r| self.manifest(r, digest))? else {
            return Err(Not::absent(&self.source, digest.to_string()));
        };
        let unreadable = |words: String| {
            Not::unreadable(
                &self.source,
                self.url_in(&from, &format!("manifests/{digest}")),
                Some(words),
            )
        };
        for (manifest, named) in oci::named(&bytes).map_err(unreadable)? {
            if manifest {
                self.hold(repository, &named)?;
            } else {
                self.mount(repository, &named, &from)?;
            }
        }
        let media_type = typed.media_type.unwrap_or_else(|| IMAGE_MANIFEST.into());
        self.put_manifest(repository, digest.as_str(), &media_type, &bytes)?;
        self.carry(&from, repository, digest)
    }

    /// Each of timber's attestations on `subject` that `from` holds, held
    /// in `repository` too: the referrer's config and layer mounted from
    /// `from`, the referrer put by its digest unless it is held, and
    /// listed in the subject's referrers tag where the put answers no
    /// `OCI-Subject`, or where it was held and the registry answers no
    /// referrers API. Another tool's referrers stay where they are (8,
    /// C.1, rule core.27; design D5).
    fn carry(&self, from: &str, repository: &str, subject: &Digest) -> Answer<()> {
        for referrer in self.referrers(from, subject)? {
            let at = referrer.digest.as_str();
            if self.manifest_head(repository, at)?.is_none() {
                let Some((bytes, _)) = self.manifest(from, &referrer.digest)? else {
                    return Err(Not::unreadable(
                        &self.source,
                        self.url_in(from, &format!("manifests/{at}")),
                        Some(format!(
                            "the referrers of {subject} name {at}, which the registry does not \
                             hold"
                        )),
                    ));
                };
                let named = oci::named(&bytes).map_err(|words| {
                    Not::unreadable(
                        &self.source,
                        self.url_in(from, &format!("manifests/{at}")),
                        Some(words),
                    )
                })?;
                for (_, blob) in named {
                    self.mount(repository, &blob, from)?;
                }
                let put = self.put_manifest(repository, at, IMAGE_MANIFEST, &bytes)?;
                if put.header("oci-subject").is_some() {
                    continue;
                }
            } else if self.referrers_api(repository, subject)?.is_some() {
                continue;
            }
            self.tag_referrer(repository, subject, &referrer)?;
        }
        Ok(())
    }

    /// The blob `digest` held in `repository`: nothing where it is held
    /// in `from` and `from` is `repository`, or a `HEAD` finds it there;
    /// mounted from `from` otherwise, and, where the registry declines the
    /// mount, its bytes read from `from` and uploaded under their digest
    /// through the upload the declined mount opened (rule core.5; design
    /// D8, D9).
    fn mount(&self, repository: &str, digest: &Digest, from: &str) -> Answer<()> {
        if repository == from || self.blob_size(repository, digest)?.is_some() {
            return Ok(());
        }
        let url = self.url_in(
            repository,
            &format!(
                "blobs/uploads/?mount={}&from={}",
                encoded(digest.as_str()),
                encoded(from)
            ),
        );
        let answered = self.send(repository, Method::POST, &url, &[], Some(&[]))?;
        if matches!(answered.status, StatusCode::CREATED | StatusCode::ACCEPTED) {
            self.mounts.borrow_mut().push((
                repository.to_string(),
                answered.status == StatusCode::CREATED,
            ));
        }
        match answered.status {
            StatusCode::CREATED => Ok(()),
            StatusCode::ACCEPTED => {
                let Some(bytes) = self.blob(from, digest)? else {
                    return Err(Not::absent(&self.source, digest.to_string()));
                };
                let location = answered.header("location").ok_or_else(|| {
                    Not::unreadable(
                        &self.source,
                        &url,
                        Some("the registry declined the mount and answered no Location".into()),
                    )
                })?;
                self.upload_to(repository, location, digest, &bytes)
            }
            _ => Err(self.other_put(repository, &url, &answered)),
        }
    }

    /// The manifest `place`'s tag is put at for `to`, with what it names
    /// held in `place`'s repository: `to`'s own where `to` is a manifest,
    /// held as [`Registry::hold`] holds it; and where `to` is bytes, the
    /// cache manifest of them, the bytes held as [`Registry::mount`]
    /// holds them (design D8, D14). `to` is asked of each repository it
    /// was reached in, the repository's own and then the cache
    /// repository, as a read is, and of the cache repository alone by a
    /// push's store, which asks nothing of the repository's own its
    /// credential may not read (135; design D4, D9). Absent where `to` is
    /// neither.
    fn named_at(&self, place: &Place, to: &Digest) -> Answer<(String, Vec<u8>)> {
        let asked = if self.push && place.space == Space::Cache {
            vec![place.repository.clone()]
        } else {
            let mut asked = self.reached_in(to);
            for repository in
                std::iter::once(self.own()?.to_string()).chain(self.cache_repository()?)
            {
                if !asked.contains(&repository) {
                    asked.push(repository);
                }
            }
            asked
        };
        for repository in &asked {
            let Some((bytes, typed)) = self.manifest(repository, to)? else {
                continue;
            };
            if let Ok(Some(layer)) = oci::cached(&bytes) {
                return Err(Not::refused(
                    &self.source,
                    Why::Said,
                    Some(format!(
                        "{to} is the cache manifest of {layer}, and a name names the bytes, \
                         {layer}"
                    )),
                ));
            }
            self.hold(&place.repository, to)?;
            let media_type = typed.media_type.unwrap_or_else(|| IMAGE_MANIFEST.into());
            return Ok((media_type, bytes));
        }
        for repository in &asked {
            if let Some(size) = self.blob_size(repository, to)? {
                self.mount(&place.repository, to, repository)?;
                return Ok((
                    IMAGE_MANIFEST.to_string(),
                    self.cache(&place.repository, to, size)?,
                ));
            }
        }
        Err(Not::absent(&self.source, to.to_string()))
    }

    /// The repositories timber's index `index`, read from `repository`,
    /// may be held in beside the repository's own: its component's
    /// repository, the repository's own for the undeclared component,
    /// and each of its artifacts' names' repositories, read from the
    /// index's own annotations (C.1; design D5). A name whose repository
    /// would run past 255 characters is held nowhere, and is left out.
    fn holders_of_index(&self, repository: &str, index: &Digest) -> Answer<Vec<String>> {
        let Some((bytes, typed)) = self.manifest(repository, index)? else {
            return Ok(Vec::new());
        };
        if !typed.artifact_type.as_deref().is_some_and(oci::is_timbers) {
            return Ok(Vec::new());
        }
        let media_type = typed.media_type.unwrap_or_default();
        let oci::Read::Index(meta) = oci::read(&media_type, &bytes).map_err(|words| {
            Not::unreadable(
                &self.source,
                self.url_in(repository, &format!("manifests/{index}")),
                Some(words),
            )
        })?
        else {
            return Ok(Vec::new());
        };
        let mut holders = Vec::new();
        if meta.component().as_str() == REPOSITORY_COMPONENT {
            holders.push(self.own()?.to_string());
        } else if let Ok(component) = self.under_own(&[meta.component().as_str()], "") {
            holders.push(component);
        }
        for artifact in meta.artifacts() {
            let (ecosystem, name) = (artifact.meta.ecosystem(), artifact.meta.name());
            if let Ok(named) = self.under(&[ARTIFACT_SPACE, ecosystem.as_str(), name.as_str()], "")
            {
                holders.push(named);
            }
        }
        Ok(holders)
    }

    /// The attestation `referrer` of `subject` held in `repository`,
    /// which holds the subject: its config and layer put, the referrer
    /// put by its digest, and listed in the subject's referrers tag where
    /// the put answers no `OCI-Subject`; where the referrer is held
    /// already, listed in the tag where the registry answers no referrers
    /// API. True where it was held already (8, 68; design D5, D6).
    fn attach_in(
        &self,
        repository: &str,
        subject: &Digest,
        referrer: &oci::Referrer,
    ) -> Answer<bool> {
        let descriptor = referrer.descriptor();
        if self
            .manifest_head(repository, descriptor.digest.as_str())?
            .is_some()
        {
            return Ok(match self.referrers_api(repository, subject)? {
                Some(_) => true,
                None => !self.tag_referrer(repository, subject, &descriptor)?,
            });
        }
        self.put_bytes_in(repository, &referrer.config)?;
        self.put_bytes_in(repository, &referrer.layer)?;
        let put = self.put_manifest(
            repository,
            descriptor.digest.as_str(),
            IMAGE_MANIFEST,
            &referrer.bytes,
        )?;
        if put.header("oci-subject").is_none() {
            self.tag_referrer(repository, subject, &descriptor)?;
        }
        Ok(false)
    }

    /// What `digest` holds in `repository`, read as [`Store::read`]
    /// answers it; none where the repository holds no such digest.
    fn read_in(&self, repository: &str, digest: &Digest) -> Answer<Option<Held>> {
        let Some((bytes, typed)) = self.manifest(repository, digest)? else {
            return Ok(self.blob(repository, digest)?.map(Held::Bytes));
        };
        if !typed.artifact_type.as_deref().is_some_and(oci::is_timbers) {
            return Ok(Some(Held::Bytes(bytes)));
        }
        let none = |digest: &Digest, words: String| {
            Not::unreadable(
                &self.source,
                self.url_in(repository, &format!("manifests/{digest}")),
                Some(words),
            )
        };
        let media_type = typed.media_type.unwrap_or_default();
        let manifest = match oci::read(&media_type, &bytes).map_err(|w| none(digest, w))? {
            oci::Read::Index(index) => Manifest::Index(index),
            oci::Read::Artifact { config, layers } => {
                let Some(meta) = self.blob(repository, &config)? else {
                    return Err(none(digest, format!("its config {config} is not held")));
                };
                Manifest::Artifact {
                    meta: oci::meta(&meta).map_err(|w| none(&config, w))?,
                    layers,
                }
            }
        };
        Ok(Some(Held::Manifest(manifest)))
    }

    fn read_back(&self, tag: &str) {
        if let Some(hook) = &self.hook {
            hook(tag);
        }
    }

    /// Where `tag`, put at `to` from `former`, is found once the registry
    /// shows the put: read back, and read again at growing intervals
    /// while it answers the former place, for at most the bound. A third
    /// digest is answered at once, and the former place once the bound
    /// has passed; the put is never repeated (C.1, 109, ADR 0015, rule
    /// core.6).
    fn shown(
        &self,
        repository: &str,
        tag: &str,
        former: Option<&Digest>,
        to: &Digest,
    ) -> Answer<Option<Digest>> {
        self.read_back(tag);
        let deadline = Instant::now() + self.bound;
        let mut wait = FIRST_WAIT;
        loop {
            let found = self.tag_at(repository, tag)?;
            let left = deadline.saturating_duration_since(Instant::now());
            if found.as_ref() == Some(to) || found.as_ref() != former || left.is_zero() {
                return Ok(found);
            }
            thread::sleep(wait.min(left));
            wait *= 2;
        }
    }

    /// timber's referrers of `subject` in `repository`, from the
    /// referrers API or, where the registry answers none, the referrers
    /// tag; another tool's referrers are never answered (design D5, D6).
    fn referrers(&self, repository: &str, subject: &Digest) -> Answer<Vec<Descriptor>> {
        Ok(match self.referrers_api(repository, subject)? {
            Some(listed) => listed,
            None => self
                .referrers_tag(repository, &tag::referrers(subject.hex()))?
                .map(|r| r.envelopes())
                .unwrap_or_default(),
        })
    }

    /// The referrers index the referrers tag `tag` of `repository` holds;
    /// none when the tag names nothing, and refused naming it when it
    /// names no image index (design D6).
    fn referrers_tag(&self, repository: &str, tag: &str) -> Answer<Option<Referrers>> {
        let url = self.url_in(repository, &format!("manifests/{tag}"));
        let answered = self.send(
            repository,
            Method::GET,
            &url,
            &[("accept", IMAGE_INDEX)],
            None,
        )?;
        match answered.status {
            StatusCode::OK => Referrers::parse(&answered.body).map(Some).map_err(|words| {
                Not::refused(
                    &self.source,
                    Why::Said,
                    Some(format!(
                        "the referrers tag {tag} holds no image index: {words}"
                    )),
                )
            }),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(self.other(&url, &answered)),
        }
    }

    /// The referrers of `subject` in `repository` of the envelope's type,
    /// as the referrers API answers them, every page of it; none where the
    /// registry answers no referrers API (design D5, D6).
    fn referrers_api(&self, repository: &str, subject: &Digest) -> Answer<Option<Vec<Descriptor>>> {
        let mut url = self.url_in(
            repository,
            &format!("referrers/{subject}?artifactType={}", encoded(ENVELOPE)),
        );
        let mut listed: Vec<Descriptor> = Vec::new();
        let mut first = true;
        loop {
            let answered = self.send(
                repository,
                Method::GET,
                &url,
                &[("accept", IMAGE_INDEX)],
                None,
            )?;
            match answered.status {
                StatusCode::OK => {}
                StatusCode::NOT_FOUND if first => return Ok(None),
                _ => return Err(self.other(&url, &answered)),
            }
            let page = Referrers::parse(&answered.body)
                .map_err(|words| Not::unreadable(&self.source, &url, Some(words)))?;
            for descriptor in page.envelopes() {
                if !listed.iter().any(|d| d.digest == descriptor.digest) {
                    listed.push(descriptor);
                }
            }
            match answered.header("link").and_then(next) {
                Some(link) => url = self.absolute(&link),
                None => return Ok(Some(listed)),
            }
            first = false;
        }
    }

    /// `referrer` listed in `subject`'s referrers tag of `repository`,
    /// once, every entry already there kept, and the tag read back: true
    /// when it was added now. An entry missing on the read back is
    /// another writer's push over it, answered unreadable naming the tag
    /// and the referrer, and nothing is pushed again (design D6, rule
    /// core.6).
    fn tag_referrer(
        &self,
        repository: &str,
        subject: &Digest,
        referrer: &Descriptor,
    ) -> Answer<bool> {
        let tag = tag::referrers(subject.hex());
        let mut referrers = self
            .referrers_tag(repository, &tag)?
            .unwrap_or_else(Referrers::empty);
        if !referrers.add(referrer) {
            return Ok(false);
        }
        self.put_manifest(repository, &tag, IMAGE_INDEX, &referrers.bytes())?;
        self.read_back(&tag);
        let back = self.referrers_tag(repository, &tag)?;
        if !back.is_some_and(|r| r.lists(&referrer.digest)) {
            return Err(Not::unreadable(
                &self.source,
                format!("the referrers tag {tag}"),
                Some(format!(
                    "the referrers tag {tag} no longer lists {} once it was pushed: another \
                     writer pushed over it",
                    referrer.digest
                )),
            ));
        }
        Ok(true)
    }

    /// The envelope the referrer `referrer` of `repository` holds, byte
    /// for byte (69).
    fn envelope(&self, repository: &str, referrer: &Descriptor) -> Answer<Envelope> {
        let at = |digest: &Digest, words: String| {
            Not::unreadable(
                &self.source,
                self.url_in(repository, &format!("manifests/{digest}")),
                Some(words),
            )
        };
        let Some((bytes, _)) = self.manifest(repository, &referrer.digest)? else {
            return Err(at(
                &referrer.digest,
                format!(
                    "the referrers list names {}, which the registry does not hold",
                    referrer.digest
                ),
            ));
        };
        let layer = oci::referred(&bytes).map_err(|words| at(&referrer.digest, words))?;
        let Some(bytes) = self.blob(repository, &layer.digest)? else {
            return Err(Not::unreadable(
                &self.source,
                self.url_in(repository, &format!("blobs/{}", layer.digest)),
                Some(format!(
                    "the referrer {} names {}, which the registry does not hold",
                    referrer.digest, layer.digest
                )),
            ));
        };
        serde_json::from_slice(&bytes).map_err(|e| {
            Not::unreadable(
                &self.source,
                self.url_in(repository, &format!("blobs/{}", layer.digest)),
                Some(e.to_string()),
            )
        })
    }
}

/// The URL a `Link` header names as the next page.
fn next(link: &str) -> Option<String> {
    link.split(',').find_map(|part| {
        let (target, params) = part.split_once(';')?;
        params
            .split(';')
            .any(|p| p.trim().replace(' ', "") == "rel=\"next\"")
            .then(|| {
                target
                    .trim()
                    .trim_start_matches('<')
                    .trim_end_matches('>')
                    .to_string()
            })
    })
}

/// Every operation below that reaches the repository's own, its
/// components' or its cache repositories reads the mark of a root left
/// first, once in the store's life, and is refused naming where the
/// repository moved where the mark names a root this store does not carry
/// to, with nothing written; an artifact's name, and the digests it
/// reaches, are read as before. A push's store reads no mark, and writes
/// its cache entries at a root left as at any root (C.1 as amended at
/// `boomdrivedev/blueprints` e8700c8; design D4; record 0042).
impl Store for Registry {
    /// The blob asked after first, and uploaded only when it is not held,
    /// in the repository's own, or, for a store bound for a push, its
    /// cache repository; the binding's answer where the store is bound
    /// for no repository, with nothing written (5, 66, 135, C.1; design
    /// D2, D3, D9).
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        let own = self.own()?;
        match self.cache_repository()? {
            Some(cache) if self.push => self.put_bytes_in(&cache, bytes),
            _ => self.put_bytes_in(own, bytes),
        }
    }

    /// The manifest the layout writes, in the repository's own, each
    /// digest it names asked after first, absent naming the first not
    /// held; the config put, and the manifest put by its digest alone
    /// unless it is held; the binding's answer where the store is bound
    /// for no repository (1, 5, 67, C.1).
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        let own = self.own()?;
        let size = |digest: &Digest| -> Answer<u64> {
            let held = match manifest {
                Manifest::Artifact { .. } => self.blob_size(own, digest)?,
                Manifest::Index(_) => self.manifest_head(own, digest.as_str())?.map(|(_, s, _)| s),
            };
            held.ok_or_else(|| Not::absent(&self.source, digest.to_string()))
        };
        let written = oci::write(manifest, &self.source, &size)?;
        let digest = Digest::of(&written.bytes);
        let held_already = self.manifest_head(own, digest.as_str())?.is_some();
        if !held_already {
            if let Some(config) = &written.config {
                self.put_bytes_in(own, config)?;
            }
            self.put_manifest(own, digest.as_str(), written.media_type, &written.bytes)?;
        }
        // C.1: a declared component's index is held in its own
        // repository under the repository's own too, from the publish
        // that puts it (design D3).
        if let Manifest::Index(index) = manifest {
            if index.component().as_str() != REPOSITORY_COMPONENT {
                let repository = self.under_own(
                    &[index.component().as_str()],
                    &format!("the index of the component {}", index.component()),
                )?;
                self.hold(&repository, &digest)?;
            }
        }
        Ok(Put {
            digest,
            held_already,
        })
    }

    /// The envelope pushed as a referrer of `subject` into each
    /// repository that holds the subject, by a `HEAD` of it: the
    /// repository's own, each repository the subject was reached in, and,
    /// for timber's index, its component's repository and each of its
    /// artifacts' names' repositories, read from the index's own
    /// annotations; and the one a read found it in. Each is written only
    /// where the attestation is missing, and listed in the subject's
    /// referrers tag where the registry answers the push with no
    /// `OCI-Subject`. Held already only where every holder held it, so an
    /// attach again after one that failed partway writes what the first
    /// left out. The subject is found as a read finds it: absent where it
    /// is no manifest held, and the binding's answer where the store is
    /// bound for no repository, with nothing written (8, 68, C.1, rule
    /// core.27; design D4 to D6).
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        let own = self.own()?.to_string();
        let found = self.find(subject, |r| self.manifest_head(r, subject.as_str()))?;
        let Some((found_in, (media_type, size, _))) = found else {
            return Err(Not::absent(&self.source, subject.to_string()));
        };
        let mut holders = vec![own];
        holders.extend(self.reached_in(subject));
        if media_type == IMAGE_INDEX {
            holders.extend(self.holders_of_index(&found_in, subject)?);
        }
        holders.push(found_in.clone());
        let referrer = oci::referrer(
            Descriptor {
                media_type,
                digest: subject.clone(),
                size,
                artifact_type: None,
                annotations: Default::default(),
            },
            attestation,
        );
        let mut held_already = true;
        let mut asked: Vec<&String> = Vec::new();
        for repository in &holders {
            if asked.contains(&repository) {
                continue;
            }
            asked.push(repository);
            if *repository != found_in
                && self.manifest_head(repository, subject.as_str())?.is_none()
            {
                continue;
            }
            held_already &= self.attach_in(repository, subject, &referrer)?;
        }
        Ok(Put {
            digest: attestation.digest(),
            held_already,
        })
    }

    /// Every envelope a referrer of the envelope's type holds for
    /// `digest` in the repository a read finds it in, from the referrers
    /// API or, where the registry answers none, the referrers tag;
    /// another tool's referrers are never answered, and bytes, which
    /// attach never makes a subject, have none (69, C.1; design D4 to
    /// D6).
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        let found = self.find(digest, |r| self.manifest_head(r, digest.as_str()))?;
        let Some((repository, _)) = found else {
            return match self.find(digest, |r| self.blob_size(r, digest))? {
                Some(_) => Ok(Vec::new()),
                None => Err(Not::absent(&self.source, digest.to_string())),
            };
        };
        self.referrers(&repository, digest)?
            .iter()
            .map(|r| self.envelope(&repository, r))
            .collect()
    }

    /// The name's tag read first and moved only from `expected`, by that
    /// read's best effort, the manifest `to` names put under it, or the
    /// cache manifest of the bytes it names, and the tag read back until
    /// the registry shows the put: refused naming where it is otherwise,
    /// before the put or after it, the place it was moved from once the
    /// bound has passed, and nothing is retried (70, 109, C.1, design D7,
    /// D14, ADR 0015, rule core.6). A make is a write: the binding's
    /// answer where the store is bound for no repository, an artifact's
    /// name among them, with nothing written (C.1; design D2).
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        self.own()?;
        let place = self.place(name)?;
        let (repository, tag) = (&place.repository, &place.tag);
        let at = self.tag_at(repository, tag)?;
        if at.as_ref() != expected {
            return Err(Not::refused(&self.source, Why::NameAt { at }, None));
        }
        let (media_type, bytes) = self.named_at(&place, to)?;
        self.put_manifest(repository, tag, &media_type, &bytes)?;
        let found = self.shown(repository, tag, at.as_ref(), to)?;
        if found.as_ref() == Some(to) {
            self.keep(to, repository);
            return Ok(());
        }
        let said =
            (found == at).then(|| {
                format!(
                "the registry still answered {tag} at {}, where it was before {to} was put under \
                 it, {:?} after the put, and it was not put again",
                found.as_ref().map_or_else(|| "nothing".into(), Digest::to_string),
                self.bound
            )
            });
        Err(Not::refused(&self.source, Why::NameAt { at: found }, said))
    }

    /// The digest the name's tag names, the bytes' at a cache manifest,
    /// kept as reached in the name's repository, so a read of it asks
    /// there first (71, C.1; design D4, D14).
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        let place = self.place(name)?;
        let digest = self
            .tag_at(&place.repository, &place.tag)?
            .ok_or_else(|| Not::absent(&self.source, format!("name {name}")))?;
        self.keep(&digest, &place.repository);
        Ok(digest)
    }

    /// The manifest held under `digest`, read in its form when it is
    /// timber's, an artifact's metadata from its config; any other
    /// manifest's bytes, or a blob's; asked of each repository the
    /// digest was reached in, then the repository's own and, where it
    /// holds no such digest, the bound build profile's cache repository,
    /// so bytes a push left there are read (72, 135, 138, rule core.20;
    /// design D4, D5, D9).
    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.find(digest, |r| self.read_in(r, digest))?
            .map(|(_, held)| held)
            .ok_or_else(|| Not::absent(&self.source, digest.to_string()))
    }

    /// 138: the versions of `name` of `ecosystem`, each tag of the
    /// name's repository read back but its index's referrers tag, which
    /// `tag::of` gives no name, every page of its tag list followed;
    /// the empty list where the registry holds no such repository or it
    /// holds no tag, and unreadable naming the repository and a tag that
    /// does not read back as one the store wrote, never leaving it out,
    /// or naming a page after the first that is not found, never a part
    /// of the list answered as the whole (rule core.19; design D11).
    fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>> {
        #[derive(Deserialize)]
        struct Tags {
            #[serde(default)]
            tags: Option<Vec<String>>,
        }
        let repository = self.under(
            &[ARTIFACT_SPACE, ecosystem.as_str(), name.as_str()],
            &format!("the {ecosystem} name {name}"),
        )?;
        let mut url = self.url_in(&repository, "tags/list");
        let mut versions = Vec::new();
        let mut first = true;
        loop {
            let answered = self.send(&repository, Method::GET, &url, &[], None)?;
            match answered.status {
                StatusCode::OK => {}
                StatusCode::NOT_FOUND if first => return Ok(versions),
                StatusCode::NOT_FOUND => {
                    return Err(Not::unreadable(
                        &self.source,
                        &url,
                        Some(format!(
                            "{}/{repository} answered a page of its tag list and then not the \
                             next, so its versions were not all read: {}",
                            self.host,
                            answered.words()
                        )),
                    ))
                }
                _ => return Err(self.other(&url, &answered)),
            }
            let page: Tags = serde_json::from_slice(&answered.body)
                .map_err(|e| Not::unreadable(&self.source, &url, Some(e.to_string())))?;
            for written in page.tags.unwrap_or_default() {
                if tag::is_referrers(&written) {
                    continue;
                }
                let Some(version) = tag::read(&written) else {
                    return Err(Not::unreadable(
                        &self.source,
                        format!("the tag {written} of {}/{repository}", self.host),
                        Some(format!(
                            "{}/{repository} holds the tag {written}, which names no version \
                             of {ecosystem} {name} as this store writes one",
                            self.host
                        )),
                    ));
                };
                versions.push(Version::new(version));
            }
            match answered.header("link").and_then(next) {
                Some(link) => url = self.absolute(&link),
                None => return Ok(versions),
            }
            first = false;
        }
    }

    fn cache_entries(&self) -> Answer<Vec<(BuildId, Answer<Pushed>)>> {
        Err(Not::absent(&self.source, "cache entries"))
    }

    fn remove_cache_entry(&self, _: &BuildId, _: &Digest) -> Answer<()> {
        Err(Not::absent(&self.source, "cache entries"))
    }
}

#[cfg(test)]
mod tests {
    use timber_seams::{
        artifact_name, Artifact, ArtifactMeta, ComponentName, IndexMeta, Signature, SignerId,
        IN_TOTO,
    };

    use super::stub::{Behaviour, Stub};
    use super::*;

    /// What a version a store holds is: its index, its artifacts, and
    /// the bytes and configs they name.
    struct Offered {
        index: Digest,
        artifacts: Vec<Digest>,
        blobs: Vec<Digest>,
    }

    fn meta(ecosystem: &str, name: &str) -> ArtifactMeta {
        ArtifactMeta::new(
            Ecosystem::new(ecosystem),
            EcosystemName::new(name),
            Version::new("1.3.0"),
        )
        .unwrap()
    }

    /// A version of `component` with an artifact of each `(ecosystem,
    /// name)`, put into `store` with an attestation on its index.
    fn published(store: &Registry, component: &str, names: &[(&str, &str)]) -> Offered {
        offered(store, component, names, false)
    }

    /// A version as a publish puts it: each artifact put with an
    /// attestation on it, then the index with one on it.
    fn attested(store: &Registry, component: &str, names: &[(&str, &str)]) -> Offered {
        offered(store, component, names, true)
    }

    /// A version of `component` with an artifact of each `(ecosystem,
    /// name)`, put into `store` with an attestation on its index, and,
    /// where `on_artifacts`, one on each artifact before the index is
    /// put.
    fn offered(
        store: &Registry,
        component: &str,
        names: &[(&str, &str)],
        on_artifacts: bool,
    ) -> Offered {
        let mut artifacts = Vec::new();
        let mut blobs = Vec::new();
        for (ecosystem, name) in names {
            let layer = store
                .put_bytes(format!("{ecosystem} {name} {component}").as_bytes())
                .unwrap()
                .digest;
            let manifest = Manifest::Artifact {
                meta: meta(ecosystem, name),
                layers: vec![layer.clone()],
            };
            let digest = store.put_artifact(&manifest).unwrap().digest;
            if on_artifacts {
                store
                    .attach(&digest, &claim(&format!("a claim on {ecosystem} {name}")))
                    .unwrap();
            }
            blobs.push(Digest::of(&timber_seams::canonical(&meta(ecosystem, name))));
            blobs.push(layer);
            artifacts.push(Artifact {
                digest,
                meta: meta(ecosystem, name),
            });
        }
        let index = IndexMeta::new(
            ComponentName::new(component),
            Version::new("1.3.0"),
            artifacts.clone(),
        )
        .unwrap();
        let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
        let envelope = Envelope::signed(
            IN_TOTO,
            b"a claim".to_vec(),
            Signature {
                signer: SignerId::new("host"),
                keyid: String::new(),
                sig: vec![1],
            },
        );
        store.attach(&index, &envelope).unwrap();
        Offered {
            index,
            artifacts: artifacts.into_iter().map(|a| a.digest).collect(),
            blobs,
        }
    }

    /// A store under the root `acme/store` of `stub`, bound for the
    /// repository `owner/name`.
    fn store_for(stub: &Stub, owner: &str, name: &str) -> Registry {
        Registry::new(&stub.repository("acme/store"))
            .unwrap()
            .for_repository(Ok(RepositoryName {
                owner: owner.into(),
                name: name.into(),
            }))
    }

    /// A store under the root `acme/store` of `stub`, bound for the
    /// repository `acme/site`, whose own is `acme/store/acme/site`.
    fn store(stub: &Stub) -> Registry {
        store_for(stub, "acme", "site")
    }

    fn named(ecosystem: &str, name: &str, version: &str) -> Name {
        artifact_name(
            &Ecosystem::new(ecosystem),
            &EcosystemName::new(name),
            &Version::new(version),
        )
    }

    fn repository(owner: &str, name: &str) -> RepositoryName {
        RepositoryName {
            owner: owner.into(),
            name: name.into(),
        }
    }

    /// The root `from` of `stub` marked moved to `to`, as a carry writes
    /// the mark: the bytes put and `moved` named at them from nothing.
    fn marked(stub: &Stub, from: (&str, &str), to: (&str, &str)) -> Digest {
        let to = repository(to.0, to.1);
        let carrying = store_for(stub, from.0, from.1).carrying_to(to.clone());
        let bytes = carrying
            .put_bytes(&MovedTo { to }.to_bytes())
            .unwrap()
            .digest;
        carrying.name(&moved_name(), &bytes, None).unwrap();
        bytes
    }

    /// What `acme/app` published before it moved: `web`'s version named
    /// by its artifact, its version's name and `beta` at its index, and
    /// a cache entry.
    struct Before {
        index: Digest,
        artifact: Name,
        version: Name,
        channel: Name,
        entry: Name,
    }

    fn published_before(stub: &Stub) -> Before {
        let app = store_for(stub, "acme", "app").bound_for("moon", false);
        let version = published(&app, "web", &[("eco", "site")]);
        let before = Before {
            index: version.index.clone(),
            artifact: named("eco", "site", "1.3.0"),
            version: Name::new("web:web-v1.3.0"),
            channel: Name::new("channel/web/beta"),
            entry: Name::new("cache/b7e1"),
        };
        for name in [&before.artifact, &before.version, &before.channel] {
            app.name(name, &version.index, None).unwrap();
        }
        let bytes = app.put_bytes(b"a cache entry").unwrap().digest;
        app.name(&before.entry, &bytes, None).unwrap();
        before
    }

    fn root_moved(answer: Answer<()>, from: (&str, &str), to: (&str, &str)) {
        match answer {
            Err(Not::Refused(r)) => assert_eq!(
                r.why,
                Why::RootMoved {
                    from: repository(from.0, from.1),
                    to: repository(to.0, to.1),
                }
            ),
            other => panic!("a root marked moved answered {other:?}"),
        }
    }

    /// Every operation of the repository's own spaces `store` is asked.
    fn own_operations(store: &Registry, before: &Before) -> Vec<Answer<()>> {
        vec![
            store.resolve(&before.channel).map(drop),
            store.resolve(&before.version).map(drop),
            store.resolve(&before.entry).map(drop),
            store.claims_of(&before.index).map(drop),
            store.read(&before.index).map(drop),
            store.put_bytes(b"after the move").map(drop),
            store
                .put_artifact(&Manifest::Artifact {
                    meta: meta("eco", "site"),
                    layers: vec![],
                })
                .map(drop),
            store
                .attach(&before.index, &claim("after the move"))
                .map(drop),
            store.name(&Name::new("channel/web/stable"), &before.index, None),
        ]
    }

    #[test]
    fn a_root_marked_moved_refuses_every_operation_of_its_own_spaces_naming_where_it_went() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let before = published_before(&stub);
        marked(&stub, ("acme", "app"), ("Acme", "Site"));
        let app = store_for(&stub, "Acme", "App").bound_for("moon", false);
        for answer in own_operations(&app, &before) {
            root_moved(answer, ("acme", "app"), ("acme", "site"));
        }
    }

    #[test]
    fn a_root_marked_moved_still_reads_an_artifacts_name_and_its_claims() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let before = published_before(&stub);
        marked(&stub, ("acme", "app"), ("acme", "site"));
        let app = store_for(&stub, "acme", "app");
        let (read, asked) = asked_while(&stub, || {
            let index = app.resolve(&before.artifact).unwrap();
            assert_eq!(index, before.index);
            assert!(matches!(
                app.read(&index),
                Ok(Held::Manifest(Manifest::Index(_)))
            ));
            assert_eq!(app.claims_of(&index).unwrap().len(), 1);
            app.versions(&Ecosystem::new("eco"), &EcosystemName::new("site"))
        });
        assert_eq!(read, Ok(vec![Version::new("1.3.0")]));
        assert_eq!(asked, ["acme/store/artifact/eco/site"]);
    }

    #[test]
    fn a_root_marked_moved_writes_nothing() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let before = published_before(&stub);
        marked(&stub, ("acme", "app"), ("acme", "site"));
        let app = store_for(&stub, "acme", "app").bound_for("moon", false);
        let since = stub.asked().len();
        own_operations(&app, &before);
        assert_eq!(writes(&stub, since), Vec::<String>::new());
        assert!(stub.asked()[since..]
            .iter()
            .all(|a| a.method == "GET" || a.method == "HEAD"));
    }

    #[test]
    fn the_mark_is_read_once_in_a_stores_life() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let site = store(&stub);
        site.put_bytes(b"one").unwrap();
        site.put_bytes(b"two").unwrap();
        assert!(site.resolve(&Name::new("channel/beta")).is_err());
        let marks = stub
            .asked()
            .iter()
            .filter(|a| a.path.ends_with("/manifests/moved"))
            .map(|a| format!("{} {}", a.method, a.path))
            .collect::<Vec<_>>();
        assert_eq!(marks, ["GET /v2/acme/store/acme/site/manifests/moved"]);
    }

    #[test]
    fn a_store_carrying_to_the_root_a_mark_names_reads_and_writes_past_it() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let before = published_before(&stub);
        marked(&stub, ("acme", "app"), ("acme", "site"));
        let carrying = store_for(&stub, "acme", "app")
            .bound_for("moon", false)
            .carrying_to(repository("Acme", "Site"));
        assert_eq!(carrying.resolve(&before.channel), Ok(before.index.clone()));
        assert_eq!(carrying.resolve(&before.version), Ok(before.index.clone()));
        assert_eq!(carrying.claims_of(&before.index).unwrap().len(), 1);
        assert!(carrying.put_bytes(b"read past the mark").is_ok());
        assert!(carrying.resolve(&before.entry).is_ok());
    }

    #[test]
    fn a_store_carrying_elsewhere_than_the_mark_names_is_refused() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let before = published_before(&stub);
        marked(&stub, ("acme", "app"), ("acme", "site"));
        let carrying = store_for(&stub, "acme", "app").carrying_to(repository("acme", "web"));
        root_moved(
            carrying.resolve(&before.channel).map(drop),
            ("acme", "app"),
            ("acme", "site"),
        );
    }

    #[test]
    fn the_mark_written_is_a_tag_of_the_repositorys_own_through_a_cache_manifest() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let bytes = marked(&stub, ("acme", "app"), ("acme", "site"));
        assert_eq!(
            bytes,
            Digest::of(br#"{"moved_to":"acme/site"}"#),
            "the mark's bytes"
        );
        let own = stub.held("acme/store/acme/app").unwrap();
        let at = own.tags.get("moved").expect("no tag moved").clone();
        assert_eq!(oci::cached(&own.manifests[&at].1), Ok(Some(bytes.clone())));
        assert!(own.blobs.contains_key(&bytes));
        assert_eq!(stub.repositories(), ["acme/store/acme/app"]);
        let carrying = store_for(&stub, "acme", "app").carrying_to(repository("acme", "site"));
        assert_eq!(carrying.resolve(&moved_name()), Ok(bytes));
    }

    #[test]
    fn a_pushs_store_at_a_root_marked_moved_writes_its_cache_and_reads_no_mark() {
        let stub = Stub::start(Behaviour::default(), &[]);
        marked(&stub, ("acme", "app"), ("acme", "site"));
        let since = stub.asked().len();
        let push = store_for(&stub, "acme", "app").bound_for("moon", true);
        let bytes = push.put_bytes(b"a pre-move build's entry").unwrap().digest;
        let entry = Name::new("cache/b7e1");
        push.name(&entry, &bytes, None).unwrap();
        assert_eq!(push.resolve(&entry), Ok(bytes.clone()));
        let asked = &stub.asked()[since..];
        assert!(
            asked
                .iter()
                .all(|a| a.repository() == Some("acme/store/acme/app/cache/moon")),
            "{asked:#?}"
        );
        assert!(!asked.iter().any(|a| a.path.ends_with("/manifests/moved")));
        let cache = stub.held("acme/store/acme/app/cache/moon").unwrap();
        assert!(cache.blobs.contains_key(&bytes));
        assert!(cache.tags.contains_key("b7e1"));
    }

    #[test]
    fn a_mark_that_cannot_be_read_is_unreadable_naming_it() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let writer = store_for(&stub, "acme", "app").carrying_to(repository("acme", "site"));
        let bytes = writer.put_bytes(b"no mark at all").unwrap().digest;
        writer.name(&moved_name(), &bytes, None).unwrap();
        let app = store_for(&stub, "acme", "app");
        match app.resolve(&Name::new("channel/beta")) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, *app.source());
                assert!(u.what.contains("the mark moved"), "{}", u.what);
                assert!(
                    u.said
                        .as_deref()
                        .is_some_and(|s| s.contains("holds no mark naming a root")),
                    "{:?}",
                    u.said
                );
            }
            other => panic!("a mark that is no mark answered {other:?}"),
        }
    }

    #[test]
    fn an_artifacts_name_is_a_tag_of_its_own_repository_holding_its_index() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        let version = published(&store, "web", &[("eco", "site"), ("eco", "cli")]);
        let name = named("eco", "site", "1.3.0");
        store.name(&name, &version.index, None).unwrap();
        assert_eq!(store.resolve(&name), Ok(version.index.clone()));

        let held = stub.held("acme/store/artifact/eco/site").unwrap();
        assert_eq!(held.tags.get("1.3.0"), Some(&version.index));
        for digest in std::iter::once(&version.index).chain(&version.artifacts) {
            assert!(held.manifests.contains_key(digest), "{digest} not held");
        }
        for blob in &version.blobs {
            assert!(held.blobs.contains_key(blob), "{blob} not held");
        }
        assert_eq!(
            stub.repositories(),
            [
                "acme/store/acme/site",
                "acme/store/acme/site/web",
                "acme/store/artifact/eco/site"
            ]
        );
        let own = stub.held("acme/store/acme/site").unwrap();
        assert!(
            own.tags.keys().all(|t| t.starts_with("sha256-")),
            "{:?}",
            own.tags
        );
        assert!(
            held.tags.contains_key(&tag::referrers(version.index.hex())),
            "the attestation on the index was not held in the artifact's repository: {:?}",
            held.tags
        );
        // The index, its two artifacts, the attestation and its subject's
        // referrers tag.
        assert_eq!(held.manifests.len(), 5, "{:?}", held.manifests.keys());
        assert_eq!(store.claims_of(&version.index).unwrap().len(), 1);
    }

    /// The writes `stub` was asked since its request `since`, as method
    /// and path without the query.
    fn writes(stub: &Stub, since: usize) -> Vec<String> {
        stub.asked()[since..]
            .iter()
            .filter(|a| a.method == "PUT" || a.method == "POST")
            .map(|a| format!("{} {}", a.method, a.path.split('?').next().unwrap()))
            .collect()
    }

    #[test]
    fn an_entry_named_from_bytes_the_repositorys_own_holds_is_held_in_its_cache_repository() {
        for declines_mounts in [false, true] {
            let stub = Stub::start(
                Behaviour {
                    declines_mounts,
                    ..Behaviour::default()
                },
                &[],
            );
            let store = store(&stub).bound_for("moon", false);
            let bytes = store.put_bytes(b"a cache entry").unwrap().digest;
            let before = stub.asked().len();
            let entry = Name::new("cache/b7e1");
            store.name(&entry, &bytes, None).unwrap();
            assert_eq!(store.resolve(&entry), Ok(bytes.clone()));

            let cache = "/v2/acme/store/acme/site/cache/moon";
            let written = writes(&stub, before);
            assert!(
                written.iter().all(|w| w.contains(cache)),
                "{declines_mounts}: {written:#?}"
            );
            let tag = written
                .iter()
                .position(|w| w.ends_with("/manifests/b7e1"))
                .unwrap();
            let manifest = written
                .iter()
                .position(|w| w.contains("/manifests/sha256:"))
                .unwrap();
            let blob = written
                .iter()
                .position(|w| w.contains("/blobs/uploads/"))
                .unwrap();
            assert!(blob < manifest && manifest < tag, "{written:#?}");
            let held = stub.held("acme/store/acme/site/cache/moon").unwrap();
            assert_eq!(
                held.blobs.get(&bytes).map(|b| Digest::of(b)),
                Some(bytes.clone())
            );
            assert!(held.tags.contains_key("b7e1"));
        }
    }

    #[test]
    fn a_pushs_cache_entry_is_held_in_the_cache_repository_alone() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let push = store(&stub).bound_for("moon", true);
        let bytes = push.put_bytes(b"a cache entry").unwrap().digest;
        let entry = Name::new("cache/b7e1");
        push.name(&entry, &bytes, None).unwrap();
        assert_eq!(push.resolve(&entry), Ok(bytes.clone()));
        assert!(
            stub.asked()
                .iter()
                .all(|a| a.repository() == Some("acme/store/acme/site/cache/moon")),
            "the repository's own was asked: {:#?}",
            stub.asked()
        );
        assert_eq!(stub.repositories(), ["acme/store/acme/site/cache/moon"]);
        let held = stub.held("acme/store/acme/site/cache/moon").unwrap();
        assert!(held.blobs.contains_key(&bytes));
        let at = held.tags["b7e1"].clone();
        assert_eq!(oci::cached(&held.manifests[&at].1), Ok(Some(bytes.clone())));

        let sync = store(&stub).bound_for("moon", false);
        let resolved = sync.resolve(&entry).unwrap();
        assert_eq!(
            sync.read(&resolved),
            Ok(Held::Bytes(b"a cache entry".to_vec()))
        );
    }

    #[test]
    fn a_store_bound_for_no_build_profile_refuses_a_cache_entrys_name() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        let bytes = store.put_bytes(b"a cache entry").unwrap().digest;
        let before = stub.asked().len();
        match store.name(&Name::new("cache/b7e1"), &bytes, None) {
            Err(Not::Refused(r)) => {
                assert!(r.said.unwrap().contains("cache/b7e1"));
            }
            other => panic!("a cache entry with no build profile answered {other:?}"),
        }
        assert_eq!(stub.asked().len(), before);
    }

    #[test]
    fn a_declared_components_version_is_a_tag_of_its_own_repository() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        let version = published(&store, "lib", &[("eco", "lib")]);
        let name = Name::new("lib:lib-v1.2.0");
        store.name(&name, &version.index, None).unwrap();
        assert_eq!(store.resolve(&name), Ok(version.index.clone()));
        let held = stub.held("acme/store/acme/site/lib").unwrap();
        assert_eq!(held.tags.get("lib-v1.2.0"), Some(&version.index));
        let own = stub.held("acme/store/acme/site").unwrap();
        assert!(!own.tags.contains_key("lib-v1.2.0"));
        assert!(!own.tags.contains_key("lib_3alib-v1.2.0"));
    }

    #[test]
    fn two_components_channels_of_one_name_are_tags_of_their_own_repositories() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        let lib = published(&store, "lib", &[("eco", "lib")]);
        let app = published(&store, "app", &[("eco", "app")]);
        let (libs, apps) = (Name::new("channel/lib/beta"), Name::new("channel/app/beta"));
        store.name(&libs, &lib.index, None).unwrap();
        store.name(&apps, &app.index, None).unwrap();
        assert_eq!(store.resolve(&libs), Ok(lib.index.clone()));
        assert_eq!(store.resolve(&apps), Ok(app.index.clone()));
        assert_eq!(
            stub.held("acme/store/acme/site/lib")
                .unwrap()
                .tags
                .get("beta"),
            Some(&lib.index)
        );
        assert_eq!(
            stub.held("acme/store/acme/site/app")
                .unwrap()
                .tags
                .get("beta"),
            Some(&app.index)
        );
        let own = stub.held("acme/store/acme/site").unwrap();
        assert!(
            own.tags.keys().all(|t| t.starts_with("sha256-")),
            "{:?}",
            own.tags
        );
    }

    #[test]
    fn a_repository_declaring_no_component_keeps_its_version_and_channel_in_its_own() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        let version = published(&store, REPOSITORY_COMPONENT, &[("eco", "site")]);
        for name in ["v1.2.0", "channel/stable"] {
            store.name(&Name::new(name), &version.index, None).unwrap();
        }
        let own = stub.held("acme/store/acme/site").unwrap();
        assert_eq!(own.tags.get("v1.2.0"), Some(&version.index));
        assert_eq!(own.tags.get("channel_2fstable"), Some(&version.index));
        assert_eq!(stub.repositories(), ["acme/store/acme/site"]);
    }

    #[test]
    fn a_repositorys_components_and_cache_hang_under_its_lowercased_owner_and_name() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store_for(&stub, "Acme", "Site").bound_for("moon", false);
        let version = published(&store, "lib", &[("eco", "site")]);
        store
            .name(&Name::new("lib:lib-v1.2.0"), &version.index, None)
            .unwrap();
        let bytes = store.put_bytes(b"a cache entry").unwrap().digest;
        store.name(&Name::new("cache/b7e1"), &bytes, None).unwrap();
        store
            .name(&named("eco", "site", "1.3.0"), &version.index, None)
            .unwrap();
        let tag = |repository: &str, tag: &str| {
            stub.held(repository)
                .and_then(|held| held.tags.get(tag).cloned())
        };
        assert_eq!(
            tag("acme/store/acme/site/lib", "lib-v1.2.0"),
            Some(version.index.clone())
        );
        assert!(tag("acme/store/acme/site/cache/moon", "b7e1").is_some());
        assert_eq!(
            tag("acme/store/artifact/eco/site", "1.3.0"),
            Some(version.index.clone())
        );
        assert!(stub
            .held("acme/store/acme/site")
            .unwrap()
            .manifests
            .contains_key(&version.index));
        assert_eq!(
            stub.repositories(),
            [
                "acme/store/acme/site",
                "acme/store/acme/site/cache/moon",
                "acme/store/acme/site/lib",
                "acme/store/artifact/eco/site"
            ]
        );
    }

    #[test]
    fn two_repositories_components_of_one_name_are_held_apart() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let (site, app) = (
            store_for(&stub, "acme", "site"),
            store_for(&stub, "acme", "app"),
        );
        let ours = published(&site, "lib", &[("eco", "site-lib")]);
        let theirs = published(&app, "lib", &[("eco", "app-lib")]);
        assert_ne!(ours.index, theirs.index);
        let name = Name::new("lib:lib-v1.0.0");
        site.name(&name, &ours.index, None).unwrap();
        app.name(&name, &theirs.index, None).unwrap();
        assert_eq!(site.resolve(&name), Ok(ours.index.clone()));
        assert_eq!(app.resolve(&name), Ok(theirs.index.clone()));
        assert_eq!(
            stub.held("acme/store/acme/site/lib")
                .unwrap()
                .tags
                .get("lib-v1.0.0"),
            Some(&ours.index)
        );
        assert_eq!(
            stub.held("acme/store/acme/app/lib")
                .unwrap()
                .tags
                .get("lib-v1.0.0"),
            Some(&theirs.index)
        );
    }

    #[test]
    fn an_owner_named_artifact_is_refused_naming_it_and_nothing_written() {
        for owner in ["artifact", "Artifact"] {
            let stub = Stub::start(Behaviour::default(), &[]);
            let store = store_for(&stub, owner, "npm");
            for answer in [
                store.put_bytes(b"site").map(drop),
                store.name(&Name::new("v1.2.0"), &Digest::of(b"an index"), None),
                store.name(
                    &named("npm", "site", "1.3.0"),
                    &Digest::of(b"an index"),
                    None,
                ),
            ] {
                match answer {
                    Err(Not::Refused(r)) => {
                        assert_eq!(r.why, Why::Said);
                        let said = r.said.unwrap();
                        assert!(said.contains(&format!("the owner {owner} ")), "{said}");
                        assert!(
                            said.contains(&format!("{}/artifact/ ", stub.repository("acme/store"))),
                            "{said}"
                        );
                    }
                    other => panic!("the owner {owner} answered {other:?}"),
                }
            }
            assert_eq!(writes(&stub, 0), Vec::<String>::new());
            assert_eq!(stub.repositories(), Vec::<String>::new());
        }
    }

    #[test]
    fn a_store_bound_for_no_repository_refuses_its_writes_and_resolves_an_artifacts_name() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let site = store(&stub);
        let version = published(&site, "web", &[("eco", "site")]);
        let name = named("eco", "site", "1.3.0");
        site.name(&name, &version.index, None).unwrap();

        let unbound = Registry::new(&stub.repository("acme/store")).unwrap();
        let before = stub.asked().len();
        assert_eq!(unbound.resolve(&name), Ok(version.index.clone()));
        assert_eq!(
            unbound.versions(&Ecosystem::new("eco"), &EcosystemName::new("site")),
            Ok(vec![Version::new("1.3.0")])
        );
        assert!(
            stub.asked()[before..]
                .iter()
                .all(|a| a.repository() == Some("acme/store/artifact/eco/site")),
            "{:#?}",
            &stub.asked()[before..]
        );
        let envelope = Envelope::signed(
            IN_TOTO,
            b"another claim".to_vec(),
            Signature {
                signer: SignerId::new("host"),
                keyid: String::new(),
                sig: vec![2],
            },
        );
        let before = stub.asked().len();
        for answer in [
            unbound.put_bytes(b"bytes").map(drop),
            unbound
                .put_artifact(&Manifest::Artifact {
                    meta: meta("eco", "site"),
                    layers: vec![],
                })
                .map(drop),
            unbound.attach(&version.index, &envelope).map(drop),
            unbound.name(&Name::new("v1.2.0"), &version.index, None),
            unbound.name(&named("eco", "site", "1.4.0"), &version.index, None),
            unbound.resolve(&Name::new("v1.2.0")).map(drop),
        ] {
            match answer {
                Err(Not::Absent(a)) => {
                    assert_eq!(a.source, *unbound.source());
                    assert_eq!(a.what, "repository this store is bound for");
                }
                other => panic!("a store bound for no repository answered {other:?}"),
            }
        }
        assert_eq!(stub.asked().len(), before, "a store bound for none asked");
    }

    /// An attestation holding `words`.
    fn claim(words: &str) -> Envelope {
        Envelope::signed(
            IN_TOTO,
            words.as_bytes().to_vec(),
            Signature {
                signer: SignerId::new("host"),
                keyid: String::new(),
                sig: vec![1],
            },
        )
    }

    /// What `read` answers, and the repositories `stub` was asked of
    /// while it ran, in order, each once.
    fn asked_while<T>(stub: &Stub, read: impl FnOnce() -> T) -> (T, Vec<String>) {
        let before = stub.asked().len();
        let answer = read();
        let mut asked: Vec<String> = Vec::new();
        for a in &stub.asked()[before..] {
            let repository = a.repository().unwrap_or_default().to_string();
            if !asked.contains(&repository) {
                asked.push(repository);
            }
        }
        (answer, asked)
    }

    #[test]
    fn a_digest_is_read_where_its_name_was_resolved() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let site = store(&stub);
        let version = published(&site, "web", &[("eco", "site")]);
        site.attach(&version.artifacts[0], &claim("the artifact's claim"))
            .unwrap();
        let name = named("eco", "site", "1.3.0");
        site.name(&name, &version.index, None).unwrap();

        let app = store_for(&stub, "acme", "app");
        let (answers, asked) = asked_while(&stub, || {
            let resolved = app.resolve(&name);
            let read: Vec<Answer<Held>> =
                [&version.index, &version.artifacts[0], &version.blobs[1]]
                    .into_iter()
                    .map(|digest| app.read(digest))
                    .collect();
            let claims: Vec<Answer<Vec<Envelope>>> = [&version.index, &version.artifacts[0]]
                .into_iter()
                .map(|digest| app.claims_of(digest))
                .collect();
            (resolved, read, claims)
        });
        let (resolved, read, claims) = answers;
        assert_eq!(resolved, Ok(version.index.clone()));
        for (digest, held) in [&version.index, &version.artifacts[0], &version.blobs[1]]
            .into_iter()
            .zip(read)
        {
            assert_eq!(held, site.read(digest), "{digest}");
        }
        for (digest, claims) in [&version.index, &version.artifacts[0]]
            .into_iter()
            .zip(claims)
        {
            let claims = claims.unwrap();
            assert_eq!(claims.len(), 1, "{digest}");
            assert_eq!(claims, site.claims_of(digest).unwrap(), "{digest}");
        }
        assert_eq!(asked, ["acme/store/artifact/eco/site"]);
        assert!(stub.held("acme/store/acme/app").is_none());
    }

    /// The referrers tag `subject` has in `repository` of `stub`.
    fn referrers_in(stub: &Stub, repository: &str, subject: &Digest) -> Option<Referrers> {
        let held = stub.held(repository)?;
        let at = held.tags.get(&tag::referrers(subject.hex()))?;
        Some(Referrers::parse(&held.manifests[at].1).unwrap())
    }

    #[test]
    fn an_artifacts_name_holds_the_claims_on_its_index_and_artifacts() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let site = store(&stub);
        let version = attested(&site, "web", &[("eco", "site"), ("eco", "cli")]);
        let names = [
            (named("eco", "site", "1.3.0"), &version.artifacts[0], "site"),
            (named("eco", "cli", "1.3.0"), &version.artifacts[1], "cli"),
        ];
        for (name, _, _) in &names {
            site.name(name, &version.index, None).unwrap();
        }
        for (name, artifact, of) in names {
            let repository = format!("acme/store/artifact/eco/{of}");
            for subject in [&version.index, artifact] {
                let listed = referrers_in(&stub, &repository, subject)
                    .unwrap_or_else(|| panic!("{repository} lists nothing on {subject}"));
                assert_eq!(listed.envelopes().len(), 1, "{repository}: {subject}");
            }
            let reader = store_for(&stub, "acme", "app");
            let (claims, asked) = asked_while(&stub, || {
                reader.resolve(&name).unwrap();
                reader.read(&version.index).unwrap();
                [&version.index, artifact].map(|subject| reader.claims_of(subject))
            });
            assert_eq!(asked, [repository.as_str()]);
            for (subject, claims) in [&version.index, artifact].into_iter().zip(claims) {
                assert_eq!(claims, site.claims_of(subject), "{subject}");
                assert_eq!(claims.unwrap().len(), 1, "{subject}");
            }
        }
    }

    #[test]
    fn a_components_repository_holds_the_claims_on_the_artifacts_its_index_names() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let site = store(&stub);
        let version = attested(&site, "lib", &[("eco", "lib"), ("eco", "lib-docs")]);
        let lib = "acme/store/acme/site/lib";
        let held = stub.held(lib).unwrap();
        assert!(held.manifests.contains_key(&version.index));
        for artifact in &version.artifacts {
            assert!(held.manifests.contains_key(artifact), "{artifact}");
            let listed = referrers_in(&stub, lib, artifact).unwrap();
            assert_eq!(listed.envelopes().len(), 1, "{artifact}");
            let claims: Vec<Envelope> = site
                .referrers(lib, artifact)
                .unwrap()
                .iter()
                .map(|r| site.envelope(lib, r).unwrap())
                .collect();
            assert_eq!(Ok(claims), site.claims_of(artifact), "{artifact}");
        }
    }

    #[test]
    fn another_tools_referrer_is_not_carried_into_a_names_repository() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let site = store(&stub);
        let version = attested(&site, "web", &[("eco", "site")]);
        let (own, artifact) = ("acme/store/acme/site", &version.artifacts[0]);
        let signature = site.put_bytes_in(own, b"another tool's signature").unwrap();
        site.put_bytes_in(own, EMPTY_JSON).unwrap();
        let (_, size, _) = site.manifest_head(own, artifact.as_str()).unwrap().unwrap();
        let other = "application/vnd.example.signature.v1";
        let bytes = serde_json::to_vec(&serde_json::json!({
            "schemaVersion": 2,
            "mediaType": IMAGE_MANIFEST,
            "artifactType": other,
            "config": {
                "mediaType": oci::EMPTY,
                "digest": Digest::of(EMPTY_JSON),
                "size": EMPTY_JSON.len(),
            },
            "layers": [{
                "mediaType": OCTETS,
                "digest": signature.digest,
                "size": b"another tool's signature".len(),
            }],
            "subject": { "mediaType": IMAGE_MANIFEST, "digest": artifact, "size": size },
        }))
        .unwrap();
        let theirs = Digest::of(&bytes);
        site.put_manifest(own, theirs.as_str(), IMAGE_MANIFEST, &bytes)
            .unwrap();
        let descriptor = Descriptor {
            media_type: IMAGE_MANIFEST.into(),
            digest: theirs.clone(),
            size: bytes.len() as u64,
            artifact_type: Some(other.into()),
            annotations: Default::default(),
        };
        assert!(site.tag_referrer(own, artifact, &descriptor).unwrap());

        site.name(&named("eco", "site", "1.3.0"), &version.index, None)
            .unwrap();
        let repository = "acme/store/artifact/eco/site";
        let listed = referrers_in(&stub, repository, artifact).unwrap();
        assert_eq!(listed.envelopes().len(), 1);
        assert!(!listed.lists(&theirs), "another tool's referrer was listed");
        assert!(!stub
            .held(repository)
            .unwrap()
            .manifests
            .contains_key(&theirs));
        assert!(referrers_in(&stub, own, artifact).unwrap().lists(&theirs));
    }

    #[test]
    fn a_claim_attached_after_the_names_is_held_in_each_repository_holding_its_subject() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let site = store(&stub);
        let version = attested(&site, "lib", &[("eco", "lib"), ("eco", "lib-docs")]);
        site.name(&Name::new("lib:lib-v1.3.0"), &version.index, None)
            .unwrap();
        for of in ["lib", "lib-docs"] {
            site.name(&named("eco", of, "1.3.0"), &version.index, None)
                .unwrap();
        }
        let gate = store(&stub);
        let acceptance = claim("the gate accepted it");
        let put = gate.attach(&version.index, &acceptance).unwrap();
        assert!(!put.held_already);
        let holders = [
            "acme/store/acme/site",
            "acme/store/acme/site/lib",
            "acme/store/artifact/eco/lib",
            "acme/store/artifact/eco/lib-docs",
        ];
        for repository in holders {
            let claims: Vec<Envelope> = site
                .referrers(repository, &version.index)
                .unwrap()
                .iter()
                .map(|r| site.envelope(repository, r).unwrap())
                .collect();
            assert!(claims.contains(&acceptance), "{repository}: {claims:?}");
            assert_eq!(claims.len(), 2, "{repository}");
        }
        assert!(
            gate.attach(&version.index, &acceptance)
                .unwrap()
                .held_already
        );
    }

    #[test]
    fn an_attestation_held_in_part_is_written_where_it_is_missing() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let site = store(&stub);
        let version = published(&site, "web", &[("eco", "site")]);
        site.name(&named("eco", "site", "1.3.0"), &version.index, None)
            .unwrap();
        let (own, named_in) = ("acme/store/acme/site", "acme/store/artifact/eco/site");
        let acceptance = claim("the gate accepted it");
        let (media_type, size, _) = site
            .manifest_head(own, version.index.as_str())
            .unwrap()
            .unwrap();
        let referrer = oci::referrer(
            Descriptor {
                media_type,
                digest: version.index.clone(),
                size,
                artifact_type: None,
                annotations: Default::default(),
            },
            &acceptance,
        );
        assert!(!site.attach_in(own, &version.index, &referrer).unwrap());
        let listed = |repository: &str| {
            referrers_in(&stub, repository, &version.index)
                .unwrap()
                .lists(&referrer.descriptor().digest)
        };
        assert!(listed(own) && !listed(named_in));

        let before = stub.asked().len();
        let put = store(&stub).attach(&version.index, &acceptance).unwrap();
        assert!(!put.held_already, "held in part answered held already");
        assert!(listed(named_in) && listed("acme/store/acme/site/web"));
        assert_eq!(
            writes_of(&stub, before, own),
            0,
            "the repository's own, which held it, was written"
        );
        assert!(
            store(&stub)
                .attach(&version.index, &acceptance)
                .unwrap()
                .held_already
        );
    }

    #[test]
    fn a_digest_no_name_reached_is_read_from_the_repositorys_own_then_its_cache() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let push = store(&stub).bound_for("moon", true);
        let cached = push.put_bytes(b"bytes a push left").unwrap().digest;
        let sync = store(&stub).bound_for("moon", false);
        let owned = sync
            .put_bytes(b"bytes of the repository's own")
            .unwrap()
            .digest;
        let (own, cache) = ("acme/store/acme/site", "acme/store/acme/site/cache/moon");

        let (held, asked) = asked_while(&stub, || sync.read(&owned));
        assert_eq!(
            held,
            Ok(Held::Bytes(b"bytes of the repository's own".to_vec()))
        );
        assert_eq!(asked, [own]);

        let (held, asked) = asked_while(&stub, || sync.read(&cached));
        assert_eq!(held, Ok(Held::Bytes(b"bytes a push left".to_vec())));
        assert_eq!(asked, [own, cache]);

        let (held, asked) = asked_while(&stub, || sync.read(&Digest::of(b"never put")));
        assert!(matches!(held, Err(Not::Absent(_))), "{held:?}");
        assert_eq!(asked, [own, cache]);
    }

    #[test]
    fn a_channel_moved_from_where_it_is_not_is_refused_naming_where_it_is_in_its_repository() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        let [one, two, three] =
            ["one", "two", "three"].map(|n| published(&store, "lib", &[("eco", n)]));
        let beta = Name::new("channel/lib/beta");
        store.name(&beta, &one.index, None).unwrap();
        match store.name(&beta, &two.index, Some(&three.index)) {
            Err(Not::Refused(r)) => assert_eq!(
                r.why,
                Why::NameAt {
                    at: Some(one.index.clone())
                }
            ),
            other => panic!("a move from a third index answered {other:?}"),
        }
        assert_eq!(store.resolve(&beta), Ok(one.index));
    }

    #[test]
    fn a_declared_components_index_is_held_in_its_repository_untagged_from_the_put() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        let version = published(&store, "lib", &[("eco", "lib"), ("eco", "lib-docs")]);
        let held = stub.held("acme/store/acme/site/lib").unwrap();
        for digest in std::iter::once(&version.index).chain(&version.artifacts) {
            assert!(held.manifests.contains_key(digest), "{digest} not held");
        }
        for blob in &version.blobs {
            assert!(held.blobs.contains_key(blob), "{blob} not held");
        }
        // The index is untagged; the attestation attached to it after the
        // put is held beside it, listed in its referrers tag.
        assert_eq!(
            held.tags.keys().collect::<Vec<_>>(),
            [&tag::referrers(version.index.hex())],
            "{:?}",
            held.tags
        );
        assert_eq!(held.manifests.len(), 5, "{:?}", held.manifests.keys());
        let own = stub.held("acme/store/acme/site").unwrap();
        assert!(own.manifests.contains_key(&version.index));
        assert_eq!(store.claims_of(&version.index).unwrap().len(), 1);
    }

    #[test]
    fn an_index_its_components_repository_lacks_is_held_there_before_a_channels_tag() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        // An index the component's repository does not hold, as one put
        // before indexes were held there is.
        let version = published(&store, REPOSITORY_COMPONENT, &[("eco", "lib")]);
        assert_eq!(
            stub.held("acme/store/acme/site/lib")
                .map(|h| h.manifests.len()),
            None
        );
        let before = stub.asked().len();
        store
            .name(&Name::new("channel/lib/beta"), &version.index, None)
            .unwrap();
        let written = writes(&stub, before);
        let tag = written
            .iter()
            .position(|w| w.ends_with("/v2/acme/store/acme/site/lib/manifests/beta"))
            .unwrap();
        let index = written
            .iter()
            .position(|w| {
                w.ends_with(&format!(
                    "/v2/acme/store/acme/site/lib/manifests/{}",
                    version.index
                ))
            })
            .unwrap();
        assert!(index < tag, "{written:#?}");
        let held = stub.held("acme/store/acme/site/lib").unwrap();
        assert!(held.manifests.contains_key(&version.artifacts[0]));
    }

    /// A refusal's words, once it carries the registry's words for a
    /// repository it does not hold and names `repository` of `stub`.
    fn refused_naming(stub: &Stub, answer: Answer<()>, repository: &str) {
        match answer {
            Err(Not::Refused(r)) => {
                let said = r.said.unwrap();
                assert!(said.contains(super::stub::NAME_UNKNOWN), "{said}");
                assert!(said.contains(&stub.repository(repository)), "{said}");
            }
            other => panic!("a write into {repository} answered {other:?}"),
        }
    }

    /// The writes `stub` was asked of `repository` since its request
    /// `since`.
    fn writes_of(stub: &Stub, since: usize, repository: &str) -> usize {
        stub.asked()[since..]
            .iter()
            .filter(|a| {
                (a.method == "PUT" || a.method == "POST") && a.repository() == Some(repository)
            })
            .count()
    }

    #[test]
    fn a_repositorys_own_that_is_not_there_is_refused_naming_it() {
        let stub = Stub::start(
            Behaviour {
                makes_on_push: false,
                ..Behaviour::default()
            },
            &[],
        );
        let store = store(&stub);
        refused_naming(
            &stub,
            store.put_bytes(b"site").map(drop),
            "acme/store/acme/site",
        );
        assert_eq!(stub.repositories(), Vec::<String>::new());
    }

    #[test]
    fn an_artifact_names_repository_missing_is_refused_naming_it_and_no_tag_put() {
        let stub = Stub::start(
            Behaviour {
                makes_on_push: false,
                ..Behaviour::default()
            },
            &["acme/store/acme/site"],
        );
        let store = store(&stub);
        let version = published(&store, REPOSITORY_COMPONENT, &[("eco", "site")]);
        let before = stub.asked().len();
        let repository = "acme/store/artifact/eco/site";
        refused_naming(
            &stub,
            store.name(&named("eco", "site", "1.3.0"), &version.index, None),
            repository,
        );
        assert_eq!(writes_of(&stub, before, repository), 1, "asked again");
        assert!(!stub
            .asked()
            .iter()
            .any(|a| a.method == "PUT" && a.path.ends_with("/manifests/1.3.0")));
    }

    #[test]
    fn a_components_repository_missing_is_refused_naming_it_and_never_asked_again() {
        let stub = Stub::start(
            Behaviour {
                makes_on_push: false,
                ..Behaviour::default()
            },
            &["acme/store/acme/site"],
        );
        let store = store(&stub);
        let version = published(&store, REPOSITORY_COMPONENT, &[("eco", "lib")]);
        let before = stub.asked().len();
        refused_naming(
            &stub,
            store.name(&Name::new("lib:lib-v1.2.0"), &version.index, None),
            "acme/store/acme/site/lib",
        );
        assert_eq!(
            writes_of(&stub, before, "acme/store/acme/site/lib"),
            1,
            "asked again"
        );
        assert!(!stub
            .asked()
            .iter()
            .any(|a| a.method == "PUT" && a.path.ends_with("/manifests/lib-v1.2.0")));
    }

    #[test]
    fn a_cache_repository_missing_is_refused_naming_it_and_the_repositorys_own_is_not_written() {
        let stub = Stub::start(
            Behaviour {
                makes_on_push: false,
                ..Behaviour::default()
            },
            &["acme/store/acme/site"],
        );
        let push = store(&stub).bound_for("moon", true);
        refused_naming(
            &stub,
            push.put_bytes(b"a cache entry").map(drop),
            "acme/store/acme/site/cache/moon",
        );
        assert!(
            stub.asked()
                .iter()
                .all(|a| a.repository() == Some("acme/store/acme/site/cache/moon")),
            "{:#?}",
            stub.asked()
        );
        assert_eq!(
            writes_of(&stub, 0, "acme/store/acme/site/cache/moon"),
            1,
            "asked again"
        );
    }

    #[test]
    fn a_names_versions_are_its_repositorys_tags_every_page_read_back() {
        let stub = Stub::start(
            Behaviour {
                page: 2,
                ..Behaviour::default()
            },
            &[],
        );
        let store = store(&stub);
        let version = published(&store, REPOSITORY_COMPONENT, &[("npm", "@acme/site")]);
        for v in ["1.0.0", "1.1.0+b.1", "2.0.0-rc.1"] {
            store
                .name(&named("npm", "@acme/site", v), &version.index, None)
                .unwrap();
        }
        let mut listed = store
            .versions(&Ecosystem::new("npm"), &EcosystemName::new("@acme/site"))
            .unwrap();
        listed.sort();
        assert_eq!(
            listed,
            [
                Version::new("1.0.0"),
                Version::new("1.1.0+b.1"),
                Version::new("2.0.0-rc.1")
            ]
        );
        let pages = stub
            .asked()
            .iter()
            .filter(|a| a.path.contains("/tags/list"))
            .count();
        assert_eq!(pages, 2, "every page was not read");
        assert_eq!(
            store.versions(&Ecosystem::new("npm"), &EcosystemName::new("never-named")),
            Ok(vec![])
        );
    }

    #[test]
    fn a_names_versions_leave_out_its_referrers_tags() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        let version = published(&store, REPOSITORY_COMPONENT, &[("eco", "site")]);
        for v in ["1.3.0", "1.4.0"] {
            store
                .name(&named("eco", "site", v), &version.index, None)
                .unwrap();
        }
        let tags = stub.held("acme/store/artifact/eco/site").unwrap().tags;
        assert!(
            tags.contains_key(&tag::referrers(version.index.hex())),
            "{tags:?}"
        );
        assert_eq!(tags.len(), 3, "{tags:?}");
        let mut listed = store
            .versions(&Ecosystem::new("eco"), &EcosystemName::new("site"))
            .unwrap();
        listed.sort();
        assert_eq!(listed, [Version::new("1.3.0"), Version::new("1.4.0")]);
    }

    #[test]
    fn a_tag_list_page_after_the_first_not_found_is_unreadable_never_a_part_of_the_list() {
        let stub = Stub::start(
            Behaviour {
                page: 2,
                loses_later_pages: true,
                ..Behaviour::default()
            },
            &[],
        );
        let store = store(&stub);
        let version = published(&store, REPOSITORY_COMPONENT, &[("eco", "site")]);
        for v in ["1.0.0", "1.1.0", "1.2.0"] {
            store
                .name(&named("eco", "site", v), &version.index, None)
                .unwrap();
        }
        match store.versions(&Ecosystem::new("eco"), &EcosystemName::new("site")) {
            Err(Not::Unreadable(u)) => {
                assert!(u.what.contains("/tags/list?"), "{}", u.what);
                let said = u.said.unwrap();
                assert!(said.contains(super::stub::NAME_UNKNOWN), "{said}");
                assert!(
                    said.contains(&stub.repository("acme/store/artifact/eco/site")),
                    "{said}"
                );
            }
            other => panic!("a lost second page answered {other:?}"),
        }
        assert_eq!(
            store.versions(&Ecosystem::new("eco"), &EcosystemName::new("never-named")),
            Ok(vec![]),
            "a repository not found at its first page did not answer the empty list"
        );
    }

    #[test]
    fn a_name_whose_repository_holds_an_operations_word_is_held_and_refused_as_its_own() {
        for word in ["tags", "blobs", "manifests", "referrers", "uploads"] {
            let stub = Stub::start(
                Behaviour {
                    page: 1,
                    ..Behaviour::default()
                },
                &[],
            );
            let named_in = store(&stub);
            let version = published(&named_in, REPOSITORY_COMPONENT, &[("npm", word)]);
            let repository = format!("acme/store/artifact/npm/{word}");
            for v in ["1.0.0", "1.1.0"] {
                let name = named("npm", word, v);
                named_in.name(&name, &version.index, None).unwrap();
                assert_eq!(named_in.resolve(&name), Ok(version.index.clone()), "{word}");
            }
            let held = stub.held(&repository).unwrap();
            assert_eq!(held.tags.get("1.0.0"), Some(&version.index), "{word}");
            assert!(held.manifests.contains_key(&version.index), "{word}");
            let mut listed = named_in
                .versions(&Ecosystem::new("npm"), &EcosystemName::new(word))
                .unwrap();
            listed.sort();
            assert_eq!(
                listed,
                [Version::new("1.0.0"), Version::new("1.1.0")],
                "{word}"
            );

            let refusing = Stub::start(
                Behaviour {
                    makes_on_push: false,
                    ..Behaviour::default()
                },
                &["acme/store/acme/site"],
            );
            let refuser = store(&refusing);
            let version = published(&refuser, REPOSITORY_COMPONENT, &[("npm", word)]);
            refused_naming(
                &refusing,
                refuser.name(&named("npm", word, "1.0.0"), &version.index, None),
                &repository,
            );
        }
    }

    #[test]
    fn a_tag_that_reads_back_as_no_version_is_unreadable_naming_it() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        stub.tag("acme/store/artifact/eco/site", "_zz", &Digest::of(b"x"));
        match store.versions(&Ecosystem::new("eco"), &EcosystemName::new("site")) {
            Err(Not::Unreadable(u)) => {
                assert!(u.what.contains("_zz"), "{}", u.what);
                assert!(
                    u.what.contains("acme/store/artifact/eco/site"),
                    "{}",
                    u.what
                );
            }
            other => panic!("a tag no version writes answered {other:?}"),
        }
    }

    #[test]
    fn a_blob_held_and_not_served_is_unreadable_in_the_registrys_words_never_absent() {
        let stub = Stub::start(
            Behaviour {
                serves_blobs_beside_a_manifest_alone: true,
                ..Behaviour::default()
            },
            &[],
        );
        let store = store(&stub);
        let bytes = b"bytes in a repository that holds no manifest";
        let digest = store.put_bytes(bytes).unwrap().digest;
        assert!(store.put_bytes(bytes).unwrap().held_already);
        match store.read(&digest) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, *store.source());
                assert_eq!(u.what, format!("{}acme/site/blobs/{digest}", store.base()));
                let said = u.said.unwrap();
                assert!(
                    said.ends_with(&format!("404: {}", super::stub::GHCR_BLOB_UNKNOWN)),
                    "{said}"
                );
            }
            other => panic!("bytes the registry holds and will not serve answered {other:?}"),
        }
        assert!(matches!(
            store.read(&Digest::of(b"bytes never put")),
            Err(Not::Absent(_))
        ));

        let manifest = Manifest::Artifact {
            meta: meta("file", "site"),
            layers: vec![digest.clone()],
        };
        store.put_artifact(&manifest).unwrap();
        assert_eq!(store.read(&digest), Ok(Held::Bytes(bytes.to_vec())));
    }

    #[test]
    fn a_scoped_packages_name_is_escaped_into_one_path_component() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        let first = published(&store, "web", &[("npm", "@acme/site")]);
        let second = published(&store, "other", &[("npm", "acme-site")]);
        let (scoped, plain) = (
            named("npm", "@acme/site", "1.3.0"),
            named("npm", "acme-site", "1.3.0"),
        );
        store.name(&scoped, &first.index, None).unwrap();
        store.name(&plain, &second.index, None).unwrap();
        assert_eq!(store.resolve(&scoped), Ok(first.index.clone()));
        assert_eq!(store.resolve(&plain), Ok(second.index.clone()));
        let tags = |repository: &str| stub.held(repository).unwrap().tags;
        assert_eq!(
            tags("acme/store/artifact/npm/0_40acme_2fsite").get("1.3.0"),
            Some(&first.index)
        );
        assert_eq!(
            tags("acme/store/artifact/npm/acme-site").get("1.3.0"),
            Some(&second.index)
        );
    }

    #[test]
    fn a_declined_mount_uploads_the_bytes_before_the_manifests_and_the_tag() {
        let stub = Stub::start(
            Behaviour {
                declines_mounts: true,
                ..Behaviour::default()
            },
            &[],
        );
        let store = store(&stub);
        let version = published(&store, "web", &[("eco", "site")]);
        let before = stub.asked().len();
        let name = named("eco", "site", "1.3.0");
        store.name(&name, &version.index, None).unwrap();
        assert_eq!(store.resolve(&name), Ok(version.index.clone()));

        let repository = "acme/store/artifact/eco/site";
        let into: Vec<bool> = store
            .mounts()
            .into_iter()
            .filter(|(r, _)| r == repository)
            .map(|(_, taken)| taken)
            .collect();
        // The version's blobs, and the empty config and the envelope of
        // the attestation on its index.
        assert_eq!(
            into,
            vec![false; version.blobs.len() + 2],
            "a declined mount was taken"
        );
        let writes: Vec<String> = stub.asked()[before..]
            .iter()
            .filter(|a| {
                a.repository() == Some(repository) && a.method != "GET" && a.method != "HEAD"
            })
            .map(|a| format!("{} {}", a.method, a.path.split('?').next().unwrap()))
            .collect();
        let mounts = writes.iter().filter(|w| w.starts_with("POST")).count();
        assert_eq!(mounts, version.blobs.len() + 2, "{writes:#?}");
        let first_manifest = writes
            .iter()
            .position(|w| w.contains("/manifests/"))
            .unwrap();
        assert_eq!(first_manifest, 2 * version.blobs.len(), "{writes:#?}");
        assert!(
            writes[..first_manifest]
                .iter()
                .all(|w| w.contains("/blobs/uploads/")),
            "{writes:#?}"
        );
        let manifests: Vec<&String> = writes
            .iter()
            .filter(|w| w.contains("/manifests/"))
            .collect();
        let referrers = tag::referrers(version.index.hex());
        assert_eq!(manifests.len(), 5, "{writes:#?}");
        assert_eq!(
            [manifests[0], manifests[1], manifests[3], manifests[4]],
            [
                &format!("PUT /v2/{repository}/manifests/{}", version.artifacts[0]),
                &format!("PUT /v2/{repository}/manifests/{}", version.index),
                &format!("PUT /v2/{repository}/manifests/{referrers}"),
                &format!("PUT /v2/{repository}/manifests/1.3.0"),
            ]
        );
        assert!(
            manifests[2].starts_with(&format!("PUT /v2/{repository}/manifests/sha256:")),
            "{writes:#?}"
        );
        assert_eq!(writes.last(), Some(manifests[4]), "{writes:#?}");
        let held = stub.held(repository).unwrap();
        for blob in &version.blobs {
            assert_eq!(
                held.blobs.get(blob).map(|b| Digest::of(b)),
                Some(blob.clone()),
                "{blob} was not uploaded under its digest"
            );
        }
    }

    #[test]
    fn a_second_name_in_a_repository_holding_the_index_writes_the_tag_alone() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        let version = published(&store, "web", &[("eco", "site")]);
        store
            .name(&named("eco", "site", "1.3.0"), &version.index, None)
            .unwrap();
        let before = stub.asked().len();
        store
            .name(&named("eco", "site", "1.3.0-again"), &version.index, None)
            .unwrap();
        let writes: Vec<String> = stub.asked()[before..]
            .iter()
            .filter(|a| a.method == "PUT" || a.method == "POST")
            .map(|a| a.path.clone())
            .collect();
        assert_eq!(
            writes,
            ["/v2/acme/store/artifact/eco/site/manifests/1.3.0-again"]
        );
    }

    #[test]
    fn a_name_made_again_from_nothing_in_its_repository_is_refused_naming_where_it_is() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        let first = published(&store, "web", &[("eco", "site")]);
        let second = published(&store, "other", &[("eco", "site")]);
        let name = named("eco", "site", "1.3.0");
        store.name(&name, &first.index, None).unwrap();
        match store.name(&name, &second.index, None) {
            Err(Not::Refused(r)) => assert_eq!(
                r.why,
                Why::NameAt {
                    at: Some(first.index.clone())
                }
            ),
            other => panic!("the second make answered {other:?}"),
        }
        assert_eq!(store.resolve(&name), Ok(first.index));
    }

    #[test]
    fn a_name_whose_repository_runs_past_255_characters_is_refused_and_nothing_written() {
        let stub = Stub::start(Behaviour::default(), &[]);
        let store = store(&stub);
        let version = published(&store, "web", &[("eco", "site")]);
        let before = stub.asked().len();
        let written = format!("@acme/{}", "n".repeat(230));
        let long = named("npm", &written, "1.3.0");
        let path = format!(
            "{}/artifact/npm/0_40acme_2f{}",
            stub.repository("acme/store"),
            "n".repeat(230)
        );
        match store.name(&long, &version.index, None) {
            Err(Not::Refused(r)) => {
                let said = r.said.unwrap();
                assert!(
                    said.starts_with(&format!("the name artifact/npm/{written}@1.3.0 ")),
                    "{said}"
                );
                assert!(said.contains(&path), "{said}");
                assert!(
                    said.contains(&format!("{} characters", path.len())),
                    "{said}"
                );
            }
            other => panic!("a path past 255 characters answered {other:?}"),
        }
        assert_eq!(stub.asked().len(), before, "a request was sent");
    }

    fn refused(answer: Answer<Registry>) -> String {
        match answer {
            Err(Not::Refused(r)) => {
                assert_eq!(r.why, Why::Said);
                r.said.unwrap()
            }
            Err(other) => panic!("answered {other:?}, not refused"),
            Ok(r) => panic!("answered a registry at {}", r.base()),
        }
    }

    #[test]
    fn a_loopback_registry_is_spoken_to_over_http() {
        let registry = Registry::new("localhost:5000/timber").unwrap();
        assert_eq!(registry.base(), "http://localhost:5000/v2/timber/");
        assert_eq!(
            registry.source(),
            &Source::new(Seam::Store, "OCI registry localhost:5000/timber")
        );
        assert_eq!(registry.repository(), "localhost:5000/timber");
        for (text, base) in [
            ("127.0.0.1:41000/a/b", "http://127.0.0.1:41000/v2/a/b/"),
            ("[::1]:5000/timber", "http://[::1]:5000/v2/timber/"),
            ("localhost/timber", "http://localhost/v2/timber/"),
        ] {
            assert_eq!(Registry::new(text).unwrap().base(), base);
        }
    }

    #[test]
    fn any_other_registry_is_spoken_to_over_https() {
        let ecr = "123456789012.dkr.ecr.us-east-1.amazonaws.com/timber/store";
        assert_eq!(
            Registry::new(ecr).unwrap().base(),
            "https://123456789012.dkr.ecr.us-east-1.amazonaws.com/v2/timber/store/"
        );
        assert_eq!(
            Registry::new("localhost.example.com:5000/x")
                .unwrap()
                .base(),
            "https://localhost.example.com:5000/v2/x/"
        );
    }

    #[test]
    fn a_text_naming_no_host_or_no_path_is_refused_naming_it() {
        for text in [
            "timber",
            "localhost:5000/",
            "/timber",
            "",
            "a//b",
            "https://x/y",
        ] {
            let words = refused(Registry::new(text));
            assert!(words.contains(&format!("{text:?}")), "{text}: {words}");
        }
    }

    #[test]
    fn the_next_page_is_read_from_a_link_header() {
        assert_eq!(
            next(r#"</v2/timber/referrers/sha256:00?n=10&last=x>; rel="next""#).as_deref(),
            Some("/v2/timber/referrers/sha256:00?n=10&last=x")
        );
        assert_eq!(next(r#"</v2/x>; rel="prev""#), None);
    }
}
