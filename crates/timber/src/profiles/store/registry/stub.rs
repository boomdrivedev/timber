//! A registry in memory on a loopback port, for the registry store's unit
//! tests (design D17): repositories of blobs, manifests and tags spoken
//! to over the distribution protocol as far as the store speaks it, every
//! request kept, a mount it may decline, repositories it may make on
//! push or hold only where made, a tag list it pages, a blob it may
//! serve only beside a manifest, and a manifest deleted by its digest
//! with every tag at it, or a delete it refuses. Where a test asks, it
//! plays GitHub's REST API for a namespace: the namespace's type, each
//! repository under it a package whose versions are its manifests, named
//! by their digests, read page by page and deleted by their ids with every
//! tag at them, each request kept with its headers (design D5).

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use timber_seams::Digest;

/// One request the stub was asked: its method and its path with query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Asked {
    pub method: String,
    pub path: String,
}

impl Asked {
    /// The repository the request is for, the path between `/v2/` and
    /// the operation, the operation read from the path's end, as a
    /// registry's routes match it: what follows the operation's word holds
    /// no `/` but an upload's, so a repository whose own path holds the
    /// word is read whole.
    pub(crate) fn repository(&self) -> Option<&str> {
        let rest = self.path.strip_prefix("/v2/")?;
        let bare = rest.split('?').next().unwrap_or(rest);
        ["/blobs/", "/manifests/", "/tags/", "/referrers/"]
            .iter()
            .filter_map(|op| bare.rfind(op))
            .max()
            .map(|at| &rest[..at])
    }
}

/// One repository the stub holds.
#[derive(Clone, Debug, Default)]
pub(crate) struct Held {
    pub blobs: BTreeMap<Digest, Vec<u8>>,
    pub manifests: BTreeMap<Digest, (String, Vec<u8>)>,
    pub tags: BTreeMap<String, Digest>,
}

#[derive(Default)]
struct State {
    repositories: BTreeMap<String, Held>,
    asked: Vec<Asked>,
    uploads: u64,
    /// The namespaces GitHub's API answers, each with its type.
    namespaces: BTreeMap<String, String>,
    /// Every request to GitHub's API, with its headers.
    api: Vec<ApiAsked>,
    /// A package version's id, by the digest it names: its place here.
    ids: Vec<Digest>,
    /// The manifests GitHub's API lists no version of.
    hidden: Vec<Digest>,
}

/// One request the stub was asked as GitHub's API: its method, its path
/// with query, and its headers, lowercased.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ApiAsked {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
}

impl ApiAsked {
    /// The header `name`'s value.
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
}

/// What the stub does beside holding what it is given.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Behaviour {
    /// It makes a repository at its first write, as zot does; otherwise
    /// a write to one it does not hold is `NAME_UNKNOWN`, as ECR answers
    /// where no creation template applies.
    pub makes_on_push: bool,
    /// It answers a mount 202, opening an upload, as a registry that
    /// declines cross-repository mounts does.
    pub declines_mounts: bool,
    /// The most tags one page of a tag list names.
    pub page: usize,
    /// It answers a tag list's every page after the first
    /// `NAME_UNKNOWN`, as a registry whose repository was deleted between
    /// pages does.
    pub loses_later_pages: bool,
    /// It answers a `GET` of a blob `BLOB_UNKNOWN` in a repository that
    /// holds no manifest, while a `HEAD` answers the blob held, as
    /// GitHub's registry does in a package its push made (record 0039).
    pub serves_blobs_beside_a_manifest_alone: bool,
    /// It answers a manifest's `DELETE` `UNSUPPORTED`, as the reference
    /// distribution does where deletes are not enabled.
    pub refuses_deletes: bool,
    /// It answers a package version's delete through GitHub's API 403,
    /// in [`GITHUB_FORBIDDEN`]'s words.
    pub github_refuses_deletes: bool,
}

impl Default for Behaviour {
    fn default() -> Self {
        Behaviour {
            makes_on_push: true,
            declines_mounts: false,
            page: 100,
            loses_later_pages: false,
            serves_blobs_beside_a_manifest_alone: false,
            refuses_deletes: false,
            github_refuses_deletes: false,
        }
    }
}

/// The stub, serving until the test process ends.
#[derive(Clone)]
pub(crate) struct Stub {
    pub port: u16,
    state: Arc<Mutex<State>>,
}

/// The registry's words for a repository it does not hold.
pub(crate) const NAME_UNKNOWN: &str =
    r#"{"errors":[{"code":"NAME_UNKNOWN","message":"repository name not known to registry"}]}"#;

/// The reference distribution's words for a delete where deletes are
/// not enabled.
pub(crate) const UNSUPPORTED: &str =
    r#"{"errors":[{"code":"UNSUPPORTED","message":"The operation is unsupported."}]}"#;

/// GitHub's API's words for a delete the token may not make.
pub(crate) const GITHUB_FORBIDDEN: &str = r#"{"message":"Must have admin rights to Repository.","documentation_url":"https://docs.github.com/rest/packages/packages#delete-package-version-for-an-organization","status":"403"}"#;

/// GitHub's registry's words for a `GET` of a blob in a package that
/// holds no manifest, verbatim as it answered on 2026-11-17 (record 0039).
pub(crate) const GHCR_BLOB_UNKNOWN: &str =
    "{\"errors\":[{\"code\":\"BLOB_UNKNOWN\",\"message\":\"blob unknown to registry\"}]}\n";

impl Stub {
    /// A stub behaving as `behaviour`, holding the repositories `made`.
    pub(crate) fn start(behaviour: Behaviour, made: &[&str]) -> Stub {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let port = listener.local_addr().unwrap().port();
        let mut state = State::default();
        for repository in made {
            state
                .repositories
                .insert(repository.to_string(), Held::default());
        }
        let state = Arc::new(Mutex::new(state));
        let served = state.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                answer(stream, &served, behaviour, port);
            }
        });
        Stub { port, state }
    }

    /// The repository text a registry store binds for `path` here.
    pub(crate) fn repository(&self, path: &str) -> String {
        format!("127.0.0.1:{}/{path}", self.port)
    }

    /// Every request asked so far, in order.
    pub(crate) fn asked(&self) -> Vec<Asked> {
        self.state.lock().unwrap().asked.clone()
    }

    /// What `repository` holds; none where the stub holds no such
    /// repository.
    pub(crate) fn held(&self, repository: &str) -> Option<Held> {
        self.state
            .lock()
            .unwrap()
            .repositories
            .get(repository)
            .cloned()
    }

    /// `tag` of `repository` put at `digest`, as another tool would.
    pub(crate) fn tag(&self, repository: &str, tag: &str, digest: &Digest) {
        self.state
            .lock()
            .unwrap()
            .repositories
            .entry(repository.to_string())
            .or_default()
            .tags
            .insert(tag.to_string(), digest.clone());
    }

    /// `bytes` held as a manifest of `repository`, untagged, as another
    /// tool or an earlier timber would have put it; answers its digest.
    pub(crate) fn manifest(&self, repository: &str, bytes: &[u8]) -> Digest {
        let digest = Digest::of(bytes);
        self.state
            .lock()
            .unwrap()
            .repositories
            .entry(repository.to_string())
            .or_default()
            .manifests
            .insert(
                digest.clone(),
                (
                    "application/vnd.oci.image.manifest.v1+json".into(),
                    bytes.to_vec(),
                ),
            );
        digest
    }

    /// GitHub's API played for `namespace`, answered of type `kind`,
    /// `Organization` or `User`.
    pub(crate) fn plays_github(&self, namespace: &str, kind: &str) {
        self.state
            .lock()
            .unwrap()
            .namespaces
            .insert(namespace.to_string(), kind.to_string());
    }

    /// No version of `manifest` listed by GitHub's API, as one deleted
    /// through GitHub's settings would leave its tag behind a lagging
    /// registry.
    pub(crate) fn github_hides(&self, manifest: &Digest) {
        self.state.lock().unwrap().hidden.push(manifest.clone());
    }

    /// Every request asked of GitHub's API so far, in order.
    pub(crate) fn api_asked(&self) -> Vec<ApiAsked> {
        self.state.lock().unwrap().api.clone()
    }

    /// The origin GitHub's API is played at.
    pub(crate) fn origin(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Every repository the stub holds.
    pub(crate) fn repositories(&self) -> Vec<String> {
        self.state
            .lock()
            .unwrap()
            .repositories
            .keys()
            .cloned()
            .collect()
    }
}

/// A request read whole: its method, path, headers and body.
fn read(stream: &TcpStream) -> Option<(String, String, Vec<(String, String)>, Vec<u8>)> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut parts = line.split_whitespace();
    let (method, path) = (parts.next()?.to_string(), parts.next()?.to_string());
    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).ok()?;
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':')?;
        headers.push((name.trim().to_lowercase(), value.trim().to_string()));
    }
    let length = headers
        .iter()
        .find(|(n, _)| n == "content-length")
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;
    Some((method, path, headers, body))
}

/// An answer: its status, headers and body.
struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Reply {
    fn new(status: u16) -> Reply {
        Reply {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    fn header(mut self, name: &str, value: impl Into<String>) -> Reply {
        self.headers.push((name.into(), value.into()));
        self
    }

    fn body(mut self, body: impl Into<Vec<u8>>) -> Reply {
        self.body = body.into();
        self
    }

    fn error(status: u16, code: &str, message: &str) -> Reply {
        let body = if code == "NAME_UNKNOWN" {
            NAME_UNKNOWN.to_string()
        } else {
            json!({ "errors": [{ "code": code, "message": message }] }).to_string()
        };
        Reply::new(status)
            .header("content-type", "application/json")
            .body(body)
    }
}

fn answer(mut stream: TcpStream, state: &Mutex<State>, behaviour: Behaviour, port: u16) {
    let Some((method, path, headers, body)) = read(&stream) else {
        return;
    };
    let reply = {
        let mut state = state.lock().unwrap();
        state.asked.push(Asked {
            method: method.clone(),
            path: path.clone(),
        });
        handle(&mut state, behaviour, port, &method, &path, &headers, body)
    };
    let head = method == "HEAD";
    let mut text = format!("HTTP/1.1 {} Stub\r\n", reply.status);
    for (name, value) in &reply.headers {
        text.push_str(&format!("{name}: {value}\r\n"));
    }
    if !reply.headers.iter().any(|(n, _)| n == "content-length") {
        text.push_str(&format!("content-length: {}\r\n", reply.body.len()));
    }
    text.push_str("connection: close\r\n\r\n");
    let mut bytes = text.into_bytes();
    if !head {
        bytes.extend_from_slice(&reply.body);
    }
    let _ = stream.write_all(&bytes);
}

/// The query's value of `key`, percent-decoded.
fn query(path: &str, key: &str) -> Option<String> {
    let (_, query) = path.split_once('?')?;
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k == key).then(|| decoded(v))
    })
}

fn decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&text[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap()
}

/// The digests a manifest's bytes name: an index's manifests, or an image
/// manifest's config and layers.
fn named(bytes: &[u8]) -> Vec<(bool, Digest)> {
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return Vec::new();
    };
    let digest = |v: &Value| v["digest"].as_str().and_then(|d| Digest::parse(d).ok());
    if let Some(manifests) = value["manifests"].as_array() {
        return manifests
            .iter()
            .filter_map(digest)
            .map(|d| (true, d))
            .collect();
    }
    let mut blobs: Vec<(bool, Digest)> = value["config"]
        .as_object()
        .and_then(|_| digest(&value["config"]))
        .map(|d| (false, d))
        .into_iter()
        .collect();
    if let Some(layers) = value["layers"].as_array() {
        blobs.extend(layers.iter().filter_map(digest).map(|d| (false, d)));
    }
    blobs
}

fn handle(
    state: &mut State,
    behaviour: Behaviour,
    port: u16,
    method: &str,
    path: &str,
    headers: &[(String, String)],
    body: Vec<u8>,
) -> Reply {
    if path == "/v2/" || path == "/v2" {
        return Reply::new(200).body("{}");
    }
    if path.starts_with("/users/") || path.starts_with("/orgs/") {
        state.api.push(ApiAsked {
            method: method.into(),
            path: path.into(),
            headers: headers.to_vec(),
        });
        return github(state, behaviour, method, path);
    }
    let asked = Asked {
        method: method.into(),
        path: path.into(),
    };
    let Some(repository) = asked.repository().map(str::to_string) else {
        return Reply::error(404, "NOT_FOUND", "no such route");
    };
    let rest = &path["/v2/".len() + repository.len()..];
    let bare = rest.split('?').next().unwrap_or(rest);
    let write = matches!(method, "PUT" | "POST");
    if !state.repositories.contains_key(&repository) {
        if write && behaviour.makes_on_push {
            state
                .repositories
                .insert(repository.clone(), Held::default());
        } else {
            return Reply::error(404, "NAME_UNKNOWN", "");
        }
    }

    if let Some(digest) = bare
        .strip_prefix("/blobs/")
        .filter(|d| !d.starts_with("uploads"))
    {
        let held = &state.repositories[&repository];
        if method == "GET"
            && behaviour.serves_blobs_beside_a_manifest_alone
            && held.manifests.is_empty()
        {
            return Reply::new(404)
                .header("content-type", "application/json")
                .body(GHCR_BLOB_UNKNOWN);
        }
        return match Digest::parse(digest).ok().and_then(|d| held.blobs.get(&d)) {
            Some(bytes) => Reply::new(200)
                .header("content-length", bytes.len().to_string())
                .header("docker-content-digest", digest)
                .body(bytes.clone()),
            None => Reply::error(404, "BLOB_UNKNOWN", "blob unknown to registry"),
        };
    }
    if bare == "/blobs/uploads/" && method == "POST" {
        if let (Some(mount), Some(from)) = (query(path, "mount"), query(path, "from")) {
            let digest = Digest::parse(&mount).unwrap();
            let found = state
                .repositories
                .get(&from)
                .and_then(|r| r.blobs.get(&digest))
                .cloned();
            if let (Some(bytes), false) = (found, behaviour.declines_mounts) {
                state
                    .repositories
                    .get_mut(&repository)
                    .unwrap()
                    .blobs
                    .insert(digest.clone(), bytes);
                return Reply::new(201)
                    .header("location", format!("/v2/{repository}/blobs/{digest}"))
                    .header("docker-content-digest", digest.as_str());
            }
        }
        state.uploads += 1;
        return Reply::new(202).header(
            "location",
            format!(
                "http://127.0.0.1:{port}/v2/{repository}/blobs/uploads/{}",
                state.uploads
            ),
        );
    }
    if bare.starts_with("/blobs/uploads/") && method == "PUT" {
        let Some(digest) = query(path, "digest").and_then(|d| Digest::parse(&d).ok()) else {
            return Reply::error(400, "DIGEST_INVALID", "no digest");
        };
        if Digest::of(&body) != digest {
            return Reply::error(400, "DIGEST_INVALID", "the bytes are not the digest's");
        }
        state
            .repositories
            .get_mut(&repository)
            .unwrap()
            .blobs
            .insert(digest.clone(), body);
        return Reply::new(201).header("docker-content-digest", digest.as_str());
    }
    if let Some(reference) = bare.strip_prefix("/manifests/") {
        let held = state.repositories.get_mut(&repository).unwrap();
        if method == "DELETE" {
            if behaviour.refuses_deletes {
                return Reply::new(405)
                    .header("content-type", "application/json")
                    .body(UNSUPPORTED);
            }
            let Ok(digest) = Digest::parse(reference) else {
                return Reply::error(400, "UNSUPPORTED", "a manifest is deleted by its digest");
            };
            if held.manifests.remove(&digest).is_none() {
                return Reply::error(404, "MANIFEST_UNKNOWN", "manifest unknown");
            }
            held.tags.retain(|_, at| *at != digest);
            return Reply::new(202);
        }
        if method == "PUT" {
            let media_type = headers
                .iter()
                .find(|(n, _)| n == "content-type")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            for (manifest, digest) in named(&body) {
                let there = if manifest {
                    held.manifests.contains_key(&digest)
                } else {
                    held.blobs.contains_key(&digest)
                };
                if !there {
                    let code = if manifest {
                        "MANIFEST_UNKNOWN"
                    } else {
                        "MANIFEST_BLOB_UNKNOWN"
                    };
                    return Reply::error(400, code, &format!("{digest} is not in {repository}"));
                }
            }
            let digest = Digest::of(&body);
            held.manifests.insert(digest.clone(), (media_type, body));
            if Digest::parse(reference).is_err() {
                held.tags.insert(reference.to_string(), digest.clone());
            }
            return Reply::new(201).header("docker-content-digest", digest.as_str());
        }
        let digest = match Digest::parse(reference) {
            Ok(digest) => Some(digest),
            Err(_) => held.tags.get(reference).cloned(),
        };
        return match digest.and_then(|d| held.manifests.get(&d).map(|m| (d, m.clone()))) {
            Some((digest, (media_type, bytes))) => Reply::new(200)
                .header("content-type", media_type)
                .header("content-length", bytes.len().to_string())
                .header("docker-content-digest", digest.as_str())
                .body(bytes),
            None => Reply::error(404, "MANIFEST_UNKNOWN", "manifest unknown"),
        };
    }
    if bare == "/tags/list" {
        let held = &state.repositories[&repository];
        let last = query(path, "last");
        if last.is_some() && behaviour.loses_later_pages {
            return Reply::error(404, "NAME_UNKNOWN", "");
        }
        let tags: Vec<&String> = held
            .tags
            .keys()
            .filter(|t| last.as_ref().is_none_or(|l| t.as_str() > l.as_str()))
            .collect();
        let page: Vec<&String> = tags.iter().take(behaviour.page).copied().collect();
        let mut reply = Reply::new(200)
            .header("content-type", "application/json")
            .body(json!({ "name": repository, "tags": page }).to_string());
        if tags.len() > page.len() {
            let last = page.last().unwrap();
            reply = reply.header(
                "link",
                format!(
                    "</v2/{repository}/tags/list?n={}&last={last}>; rel=\"next\"",
                    behaviour.page
                ),
            );
        }
        return reply;
    }
    Reply::error(404, "NOT_FOUND", "no such route")
}

/// GitHub's error, as its API answers one.
fn github_error(status: u16, message: &str) -> Reply {
    Reply::new(status)
        .header("content-type", "application/json; charset=utf-8")
        .body(json!({ "message": message, "documentation_url": "https://docs.github.com/rest", "status": status.to_string() }).to_string())
}

/// GitHub's REST API as far as the store speaks it (design D5): a
/// namespace's type, a package's versions by page, and a version's
/// delete, the package being the repository under the namespace its name
/// names with each `%2F` a `/`.
fn github(state: &mut State, behaviour: Behaviour, method: &str, path: &str) -> Reply {
    let bare = path.split('?').next().unwrap_or(path);
    let parts: Vec<&str> = bare.trim_start_matches('/').split('/').collect();
    if let ["users", namespace] = parts.as_slice() {
        return match state.namespaces.get(*namespace) {
            Some(kind) => Reply::new(200)
                .header("content-type", "application/json; charset=utf-8")
                .body(json!({ "login": namespace, "id": 1, "type": kind }).to_string()),
            None => github_error(404, "Not Found"),
        };
    }
    let (owner, namespace, package, version) = match parts.as_slice() {
        [owner, namespace, "packages", "container", package, "versions"] => {
            (*owner, *namespace, *package, None)
        }
        [owner, namespace, "packages", "container", package, "versions", id] => {
            (*owner, *namespace, *package, Some(*id))
        }
        _ => return github_error(404, "Not Found"),
    };
    let kind = state.namespaces.get(namespace).map(String::as_str);
    let expected = match owner {
        "orgs" => "Organization",
        _ => "User",
    };
    if kind != Some(expected) {
        return github_error(404, "Not Found");
    }
    let repository = format!("{namespace}/{}", decoded(package));
    let Some(held) = state.repositories.get(&repository) else {
        return github_error(404, "Package not found.");
    };
    let digests: Vec<Digest> = held
        .manifests
        .keys()
        .filter(|d| !state.hidden.contains(d))
        .cloned()
        .collect();
    let mut ids = Vec::new();
    for digest in &digests {
        let id = match state.ids.iter().position(|d| d == digest) {
            Some(at) => at + 1,
            None => {
                state.ids.push(digest.clone());
                state.ids.len()
            }
        };
        ids.push((id, digest.clone()));
    }
    match (method, version) {
        ("GET", None) => {
            let per_page: usize = query(path, "per_page")
                .and_then(|p| p.parse().ok())
                .unwrap_or(30);
            let page: usize = query(path, "page")
                .and_then(|p| p.parse().ok())
                .unwrap_or(1);
            let size = per_page.min(behaviour.page);
            let listed: Vec<Value> = ids
                .iter()
                .skip((page - 1) * size)
                .take(size)
                .map(|(id, digest)| json!({ "id": id, "name": digest, "package_html_url": "https://github.com" }))
                .collect();
            Reply::new(200)
                .header("content-type", "application/json; charset=utf-8")
                .body(Value::Array(listed).to_string())
        }
        ("DELETE", Some(id)) => {
            if behaviour.github_refuses_deletes {
                return Reply::new(403)
                    .header("content-type", "application/json; charset=utf-8")
                    .body(GITHUB_FORBIDDEN);
            }
            let Some((_, digest)) = ids.iter().find(|(i, _)| i.to_string() == id) else {
                return github_error(404, "Not Found");
            };
            let held = state.repositories.get_mut(&repository).unwrap();
            held.manifests.remove(digest);
            held.tags.retain(|_, at| at != digest);
            Reply::new(204)
        }
        _ => github_error(404, "Not Found"),
    }
}
