//! The fake store (B.1): what it holds by digest, the attestations beside
//! each, its names, and when each cache entry's name was made or moved,
//! in memory. A cache entry is listed with that moment and removed by its
//! build identity, its bytes kept (145, 54; design D6).

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use timber_seams::{
    artifact_name, cache_name, canonical, Answer, BuildId, Digest, Ecosystem, EcosystemName,
    Envelope, Held, Manifest, Name, Not, Pushed, Put, Seam, Store, Version, Why, CACHE,
};

use crate::Fake;

/// The fake store's operations, as its record and its faults name them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreOp {
    PutBytes,
    PutArtifact,
    Attach,
    ClaimsOf,
    Name,
    Resolve,
    Read,
    Versions,
    CacheEntries,
    RemoveCacheEntry,
}

/// Everything the fake store holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StoreState {
    /// Bytes and manifests, by the digest they are held under.
    pub held: BTreeMap<Digest, Held>,
    /// The attestations attached beside each digest, in the order
    /// attached.
    pub attached: BTreeMap<Digest, Vec<Envelope>>,
    pub names: BTreeMap<Name, Digest>,
    /// 145: when each name under `cache/` was made or last moved, in
    /// seconds since the Unix epoch; a name with none here has no moment.
    pub pushed: BTreeMap<Name, u64>,
    /// The moment a test set the store's clock to, the system's where
    /// none.
    pub clock: Option<u64>,
}

impl StoreState {
    fn hold(&mut self, digest: Digest, what: Held) -> Put {
        let held_already = self.held.contains_key(&digest);
        self.held.entry(digest.clone()).or_insert(what);
        Put {
            digest,
            held_already,
        }
    }

    /// The store's clock: the moment a test set, or the system's.
    fn now(&self) -> u64 {
        self.clock.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |since| since.as_secs())
        })
    }
}

/// A store in memory. A manifest's digest is the digest of its canonical
/// JSON (design D4).
pub type FakeStore = Fake<StoreState, StoreOp>;

impl Default for FakeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeStore {
    pub fn new() -> Self {
        Fake::with(Seam::Store, StoreState::default())
    }

    /// 2: `bytes`, not its own, held under `digest`, so read answers them
    /// and the core can refuse naming both.
    pub fn corrupt(&self, digest: &Digest, bytes: &[u8]) {
        self.state()
            .held
            .insert(digest.clone(), Held::Bytes(bytes.to_vec()));
    }

    /// 54: `name` removed and what it named left held, standing in for a
    /// store's own retention, which removes a cache entry's name behind
    /// timber's back; timber's own removal is
    /// [`Store::remove_cache_entry`], which names a build identity and is
    /// recorded.
    pub fn forget(&self, name: &Name) {
        let mut state = self.state();
        state.names.remove(name);
        state.pushed.remove(name);
    }

    /// 145: the store's clock set to `seconds` since the Unix epoch, the
    /// moment every cache entry's name made or moved from now on is
    /// stamped with.
    pub fn clock(&self, seconds: u64) {
        self.state().clock = Some(seconds);
    }

    /// 145: the cache entry of `build` stamped with `at`, or with no
    /// moment, as an entry pushed before moments were written lists.
    pub fn pushed(&self, build: &BuildId, at: Option<u64>) {
        let name = cache_name(build);
        let mut state = self.state();
        match at {
            Some(at) => state.pushed.insert(name, at),
            None => state.pushed.remove(&name),
        };
    }

    /// Everything the store holds now.
    pub fn contents(&self) -> StoreState {
        self.state().clone()
    }
}

impl Store for FakeStore {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        let digest = Digest::of(bytes);
        self.ask(
            StoreOp::PutBytes,
            json!({ "bytes": bytes }),
            format!("the put of {digest}"),
            |s, _| Ok(s.hold(digest, Held::Bytes(bytes.to_vec()))),
        )
    }

    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        let digest = Digest::of(&canonical(manifest));
        self.ask(
            StoreOp::PutArtifact,
            json!({ "manifest": manifest }),
            format!("the put of {digest}"),
            |s, _| Ok(s.hold(digest, Held::Manifest(manifest.clone()))),
        )
    }

    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        let given = json!({ "subject": subject, "attestation": attestation });
        self.ask(
            StoreOp::Attach,
            given,
            format!("the claims of {subject}"),
            |s, source| {
                if !s.held.contains_key(subject) {
                    return Err(Not::absent(source, subject.to_string()));
                }
                let attached = s.attached.entry(subject.clone()).or_default();
                let held_already = attached.contains(attestation);
                if !held_already {
                    attached.push(attestation.clone());
                }
                Ok(Put {
                    digest: attestation.digest(),
                    held_already,
                })
            },
        )
    }

    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        self.ask(
            StoreOp::ClaimsOf,
            json!({ "digest": digest }),
            format!("the claims of {digest}"),
            |s, source| {
                if !s.held.contains_key(digest) {
                    return Err(Not::absent(source, digest.to_string()));
                }
                Ok(s.attached.get(digest).cloned().unwrap_or_default())
            },
        )
    }

    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        let given = json!({ "name": name, "to": to, "expected": expected });
        self.ask(
            StoreOp::Name,
            given,
            format!("the name {name}"),
            |s, source| {
                let at = s.names.get(name).cloned();
                if at.as_ref() != expected {
                    return Err(Not::refused(source, Why::NameAt { at }, None));
                }
                s.names.insert(name.clone(), to.clone());
                if name.as_str().starts_with(CACHE) {
                    let now = s.now();
                    s.pushed.insert(name.clone(), now);
                }
                Ok(())
            },
        )
    }

    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.ask(
            StoreOp::Resolve,
            json!({ "name": name }),
            format!("the name {name}"),
            |s, source| {
                s.names
                    .get(name)
                    .cloned()
                    .ok_or_else(|| Not::absent(source, format!("name {name}")))
            },
        )
    }

    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.ask(
            StoreOp::Read,
            json!({ "digest": digest }),
            digest.to_string(),
            |s, source| {
                s.held
                    .get(digest)
                    .cloned()
                    .ok_or_else(|| Not::absent(source, digest.to_string()))
            },
        )
    }

    /// 138: the version of every name it holds that reads back as an
    /// artifact's of `ecosystem` and `name`, in the order of its names.
    fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>> {
        let given = json!({ "ecosystem": ecosystem, "name": name });
        let of = artifact_name(ecosystem, name, &Version::new(""));
        self.ask(
            StoreOp::Versions,
            given,
            format!("the names {of}"),
            |s, _| {
                Ok(s.names
                    .keys()
                    .filter_map(Name::as_artifact)
                    .filter(|(e, n, _)| e == ecosystem && n == name)
                    .map(|(_, _, v)| v)
                    .collect())
            },
        )
    }

    /// 145: every name under `cache/`, in the order of its names, with
    /// the digest it is at and its moment, none where it has none.
    fn cache_entries(&self) -> Answer<Vec<(BuildId, Answer<Pushed>)>> {
        self.ask(
            StoreOp::CacheEntries,
            json!({}),
            "the cache entries",
            |s, _| {
                Ok(s.names
                    .iter()
                    .filter_map(|(name, digest)| {
                        let build = name.as_str().strip_prefix(CACHE)?;
                        let pushed = Pushed {
                            digest: digest.clone(),
                            at: s.pushed.get(name).copied(),
                        };
                        Some((BuildId::new(build), Ok(pushed)))
                    })
                    .collect())
            },
        )
    }

    /// 145, 54: the name of `build`'s entry removed where it is at `at`,
    /// its moment with it, and the bytes it named kept.
    fn remove_cache_entry(&self, build: &BuildId, at: &Digest) -> Answer<()> {
        let name = cache_name(build);
        let given = json!({ "build": build, "at": at });
        self.ask(
            StoreOp::RemoveCacheEntry,
            given,
            format!("the name {name}"),
            |s, source| match s.names.get(&name) {
                None => Err(Not::absent(source, format!("cache entry {build}"))),
                Some(held) if held != at => Err(Not::refused(
                    source,
                    Why::NameAt {
                        at: Some(held.clone()),
                    },
                    None,
                )),
                Some(_) => {
                    s.names.remove(&name);
                    s.pushed.remove(&name);
                    Ok(())
                }
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use timber_seams::{
        Artifact, ArtifactMeta, ComponentName, Ecosystem, EcosystemName, IndexMeta, Signature,
        SignerId, Source, Version, IN_TOTO,
    };

    use super::*;
    use crate::{Fault, Times};

    fn artifact(name: &str) -> Manifest {
        Manifest::Artifact {
            meta: ArtifactMeta::new(
                Ecosystem::new("eco"),
                EcosystemName::new(name),
                Version::new("1.0.0"),
            )
            .unwrap(),
            layers: vec![Digest::of(name.as_bytes())],
        }
    }

    fn envelope(payload: &[u8]) -> Envelope {
        Envelope::signed(
            IN_TOTO,
            payload.to_vec(),
            Signature {
                signer: SignerId::new("host"),
                keyid: String::new(),
                sig: vec![1],
            },
        )
    }

    fn source() -> Source {
        Source::new(Seam::Store, "fake")
    }

    #[test]
    fn a_names_versions_are_listed_from_the_names_held() {
        let store = FakeStore::new();
        let index = store.put_bytes(b"an index").unwrap().digest;
        for (name, version) in [
            ("site", "1.0.0"),
            ("site", "1.1.0+b.1"),
            ("site-cli", "1.0.0"),
            ("@acme/site", "2.0.0"),
        ] {
            let name = artifact_name(
                &Ecosystem::new("eco"),
                &EcosystemName::new(name),
                &Version::new(version),
            );
            store.name(&name, &index, None).unwrap();
        }
        store.name(&Name::new("site-v1.0.0"), &index, None).unwrap();
        let held = store.contents();
        let versions = |e: &str, n: &str| {
            store
                .versions(&Ecosystem::new(e), &EcosystemName::new(n))
                .unwrap()
        };
        assert_eq!(
            versions("eco", "site"),
            [Version::new("1.0.0"), Version::new("1.1.0+b.1")]
        );
        assert_eq!(versions("eco", "@acme/site"), [Version::new("2.0.0")]);
        assert_eq!(versions("eco", "web"), Vec::<Version>::new());
        assert_eq!(versions("other", "site"), Vec::<Version>::new());
        assert_eq!(store.contents(), held);
    }

    #[test]
    fn the_same_bytes_put_twice_are_held_once() {
        let store = FakeStore::new();
        let first = store.put_bytes(b"site").unwrap();
        let after_first = store.contents();
        let second = store.put_bytes(b"site").unwrap();
        assert_eq!(first.digest, Digest::of(b"site"));
        assert!(!first.held_already);
        assert_eq!(second.digest, first.digest);
        assert!(second.held_already);
        assert_eq!(store.contents(), after_first);
        assert_eq!(store.contents().held.len(), 1);
    }

    #[test]
    fn nothing_is_written_the_second_time() {
        let store = FakeStore::new();
        store.put_bytes(b"site").unwrap();
        store.put_artifact(&artifact("site")).unwrap();
        let after_first = store.contents();
        assert!(store.put_bytes(b"site").unwrap().held_already);
        assert!(store.put_artifact(&artifact("site")).unwrap().held_already);
        assert_eq!(store.contents(), after_first);
        let ops: Vec<StoreOp> = store.calls().iter().map(|c| c.op).collect();
        assert_eq!(
            ops,
            [
                StoreOp::PutBytes,
                StoreOp::PutArtifact,
                StoreOp::PutBytes,
                StoreOp::PutArtifact
            ]
        );
    }

    #[test]
    fn the_order_of_puts_is_recorded() {
        let store = FakeStore::new();
        let a = store.put_artifact(&artifact("a")).unwrap();
        let b = store.put_artifact(&artifact("b")).unwrap();
        let meta = |m: Manifest| match m {
            Manifest::Artifact { meta, .. } => meta,
            Manifest::Index(_) => unreachable!(),
        };
        let index = IndexMeta::new(
            ComponentName::new("web"),
            Version::new("1.0.0"),
            vec![
                Artifact {
                    digest: a.digest,
                    meta: meta(artifact("a")),
                },
                Artifact {
                    digest: b.digest,
                    meta: meta(artifact("b")),
                },
            ],
        )
        .unwrap();
        store.put_artifact(&Manifest::Index(index)).unwrap();
        let given: Vec<String> = store
            .calls()
            .iter()
            .map(|c| match &c.given["manifest"] {
                m if m.get("index").is_some() => "index".to_string(),
                m => m["artifact"]["meta"]["name"].as_str().unwrap().to_string(),
            })
            .collect();
        assert_eq!(given, ["a", "b", "index"]);
    }

    #[test]
    fn an_artifact_reads_back_as_its_manifest() {
        let store = FakeStore::new();
        let put = store.put_artifact(&artifact("site")).unwrap();
        assert_eq!(put.digest, Digest::of(&canonical(&artifact("site"))));
        assert_eq!(
            store.read(&put.digest),
            Ok(Held::Manifest(artifact("site")))
        );
        assert_eq!(
            store.read(&Digest::of(b"site")),
            Err(Not::absent(&source(), Digest::of(b"site").to_string()))
        );
    }

    #[test]
    fn a_digest_not_held_is_absent_never_refused_or_unreadable() {
        let store = FakeStore::new();
        let d = Digest::of(b"nowhere");
        assert_eq!(store.read(&d), Err(Not::absent(&source(), d.to_string())));
    }

    #[test]
    fn a_name_moves_only_from_where_it_is_expected() {
        let store = FakeStore::new();
        let name = Name::new("web-v1.0.0");
        let (a, b, c) = (Digest::of(b"a"), Digest::of(b"b"), Digest::of(b"c"));
        assert_eq!(
            store.resolve(&name),
            Err(Not::absent(&source(), "name web-v1.0.0"))
        );
        assert_eq!(
            store.name(&name, &a, None),
            Ok(()),
            "a name made from nothing"
        );
        assert_eq!(store.resolve(&name), Ok(a.clone()));
        assert_eq!(
            store.name(&name, &b, Some(&c)),
            Err(Not::refused(
                &source(),
                Why::NameAt {
                    at: Some(a.clone())
                },
                None
            ))
        );
        assert_eq!(
            store.name(&name, &b, None),
            Err(Not::refused(
                &source(),
                Why::NameAt {
                    at: Some(a.clone())
                },
                None
            ))
        );
        assert_eq!(store.resolve(&name), Ok(a.clone()), "the name stays");
        assert_eq!(
            store.name(&Name::new("other"), &b, Some(&a)),
            Err(Not::refused(&source(), Why::NameAt { at: None }, None))
        );
        assert_eq!(store.name(&name, &b, Some(&a)), Ok(()));
        assert_eq!(store.resolve(&name), Ok(b));
    }

    #[test]
    fn attestations_are_held_beside_their_digest() {
        let store = FakeStore::new();
        let d = store.put_bytes(b"site").unwrap().digest;
        let (e1, e2) = (envelope(b"one"), envelope(b"two"));
        assert_eq!(store.claims_of(&d), Ok(vec![]));
        assert_eq!(
            store.attach(&d, &e1),
            Ok(Put {
                digest: e1.digest(),
                held_already: false
            })
        );
        store.attach(&d, &e2).unwrap();
        assert!(store.attach(&d, &e1).unwrap().held_already);
        assert_eq!(store.claims_of(&d), Ok(vec![e1.clone(), e2]));

        let elsewhere = Digest::of(b"not held");
        assert_eq!(
            store.claims_of(&elsewhere),
            Err(Not::absent(&source(), elsewhere.to_string()))
        );
        assert_eq!(
            store.attach(&elsewhere, &e1),
            Err(Not::absent(&source(), elsewhere.to_string()))
        );
        assert!(!store.contents().attached.contains_key(&elsewhere));
    }

    #[test]
    fn a_store_that_cannot_be_reached_answers_unreadable_never_empty() {
        let store = FakeStore::new();
        let d = store.put_bytes(b"site").unwrap().digest;
        store.fault(
            StoreOp::ClaimsOf,
            Fault::Unreadable("connection refused".into()),
            Times::UntilHealed,
        );
        let answer = store.claims_of(&d);
        assert_eq!(
            answer,
            Err(Not::unreadable(
                &source(),
                format!("the claims of {d}"),
                Some("connection refused".into())
            ))
        );
        let Err(Not::Unreadable(u)) = answer else {
            unreachable!()
        };
        assert_eq!(
            u.said.as_deref().map(str::as_bytes),
            Some("connection refused".as_bytes())
        );
        store.heal(StoreOp::ClaimsOf);
        assert_eq!(store.claims_of(&d), Ok(vec![]));
    }

    #[test]
    fn a_put_refused_once_holds_nothing_and_the_next_holds_the_bytes() {
        let store = FakeStore::new();
        store.fault(
            StoreOp::PutBytes,
            Fault::Refused("quota exceeded".into()),
            Times::Once,
        );
        assert_eq!(
            store.put_bytes(b"site"),
            Err(Not::refused(
                &source(),
                Why::Said,
                Some("quota exceeded".into())
            ))
        );
        assert!(store.contents().held.is_empty());
        assert!(!store.put_bytes(b"site").unwrap().held_already);
        assert_eq!(store.contents().held.len(), 1);
    }

    #[test]
    fn other_bytes_under_a_digest_are_read_back() {
        let store = FakeStore::new();
        let d = store.put_bytes(b"site").unwrap().digest;
        store.corrupt(&d, b"other");
        assert_eq!(store.read(&d), Ok(Held::Bytes(b"other".to_vec())));
    }

    #[test]
    fn a_forgotten_name_resolves_absent_and_its_digest_still_reads() {
        let store = FakeStore::new();
        let name = Name::new("cache/b7e1");
        let d = store.put_bytes(b"archive").unwrap().digest;
        store.name(&name, &d, None).unwrap();
        store.forget(&name);
        assert_eq!(
            store.resolve(&name),
            Err(Not::absent(&source(), "name cache/b7e1"))
        );
        assert_eq!(store.read(&d), Ok(Held::Bytes(b"archive".to_vec())));
        assert_eq!(
            store.name(&name, &d, None),
            Ok(()),
            "made again from nothing"
        );
    }

    #[test]
    fn state_is_seen_through_a_copy() {
        let kept = FakeStore::new();
        let bound: Box<dyn Store> = Box::new(kept.clone());
        let d = bound.put_bytes(b"site").unwrap().digest;
        assert!(kept.contents().held.contains_key(&d));
        assert_eq!(kept.calls().len(), 1);
    }

    #[test]
    fn every_operation_can_be_faulted_and_is_recorded() {
        let store = FakeStore::new();
        let d = Digest::of(b"x");
        let name = Name::new("n");
        let ops = [
            StoreOp::PutBytes,
            StoreOp::PutArtifact,
            StoreOp::Attach,
            StoreOp::ClaimsOf,
            StoreOp::Name,
            StoreOp::Resolve,
            StoreOp::Read,
            StoreOp::Versions,
            StoreOp::CacheEntries,
            StoreOp::RemoveCacheEntry,
        ];
        for op in ops {
            store.fault(op, Fault::Refused("no".into()), Times::Once);
        }
        let answers = [
            store.put_bytes(b"x").map(drop),
            store.put_artifact(&artifact("x")).map(drop),
            store.attach(&d, &envelope(b"x")).map(drop),
            store.claims_of(&d).map(drop),
            store.name(&name, &d, None).map(drop),
            store.resolve(&name).map(drop),
            store.read(&d).map(drop),
            store
                .versions(&Ecosystem::new("eco"), &EcosystemName::new("x"))
                .map(drop),
            store.cache_entries().map(drop),
            store
                .remove_cache_entry(&BuildId::new("b7e1"), &d)
                .map(drop),
        ];
        for answer in answers {
            assert_eq!(
                answer,
                Err(Not::refused(&source(), Why::Said, Some("no".into())))
            );
        }
        assert_eq!(store.calls().iter().map(|c| c.op).collect::<Vec<_>>(), ops);
        assert_eq!(store.contents(), StoreState::default());
    }

    /// A store at a fixed clock holding `bytes` under the cache entry of
    /// `build`.
    fn with_entry(build: &str, bytes: &[u8], at: u64) -> (FakeStore, Digest) {
        let store = FakeStore::new();
        store.clock(at);
        let d = store.put_bytes(bytes).unwrap().digest;
        store
            .name(&cache_name(&BuildId::new(build)), &d, None)
            .unwrap();
        (store, d)
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    #[test]
    fn the_fake_store_lists_a_cache_entry_with_its_moment() {
        let before = now();
        let store = FakeStore::new();
        let d = store.put_bytes(b"archive").unwrap().digest;
        store
            .name(&cache_name(&BuildId::new("b7e1")), &d, None)
            .unwrap();
        let after = now();
        store.name(&Name::new("channel/stable"), &d, None).unwrap();
        let listed = store.cache_entries().unwrap();
        assert_eq!(listed.len(), 1, "{listed:?}");
        let (build, pushed) = &listed[0];
        assert_eq!(build, &BuildId::new("b7e1"));
        let pushed = pushed.clone().unwrap();
        assert_eq!(pushed.digest, d);
        let at = pushed.at.expect("a moment");
        assert!(before <= at && at <= after, "{before} {at} {after}");
    }

    #[test]
    fn the_fake_store_removes_an_entry_and_keeps_its_bytes() {
        let (store, d) = with_entry("b7e1", b"archive", 1_793_000_000);
        store
            .name(&cache_name(&BuildId::new("c9d2")), &d, None)
            .unwrap();
        assert_eq!(store.remove_cache_entry(&BuildId::new("b7e1"), &d), Ok(()));
        assert_eq!(
            store.resolve(&cache_name(&BuildId::new("b7e1"))),
            Err(Not::absent(&source(), "name cache/b7e1"))
        );
        assert_eq!(
            store.resolve(&cache_name(&BuildId::new("c9d2"))),
            Ok(d.clone())
        );
        assert_eq!(store.read(&d), Ok(Held::Bytes(b"archive".to_vec())));
        let listed: Vec<BuildId> = store
            .cache_entries()
            .unwrap()
            .into_iter()
            .map(|(build, _)| build)
            .collect();
        assert_eq!(listed, [BuildId::new("c9d2")]);
        assert!(!store
            .contents()
            .pushed
            .contains_key(&cache_name(&BuildId::new("b7e1"))));
    }

    #[test]
    fn the_fake_store_refuses_a_removal_from_where_the_name_is_not() {
        let (store, d) = with_entry("b7e1", b"archive", 1_793_000_000);
        let held = store.contents();
        assert_eq!(
            store.remove_cache_entry(&BuildId::new("b7e1"), &Digest::of(b"other")),
            Err(Not::refused(&source(), Why::NameAt { at: Some(d) }, None))
        );
        assert_eq!(store.contents(), held);
    }

    #[test]
    fn the_fake_store_answers_a_removal_of_no_entry_absent() {
        let (store, d) = with_entry("b7e1", b"archive", 1_793_000_000);
        store.name(&Name::new("c9d2"), &d, None).unwrap();
        let held = store.contents();
        assert_eq!(
            store.remove_cache_entry(&BuildId::new("c9d2"), &d),
            Err(Not::absent(&source(), "cache entry c9d2"))
        );
        assert_eq!(store.contents(), held, "a name outside cache/ is no entry");
    }

    #[test]
    fn a_moment_a_test_sets_is_listed() {
        let (store, d) = with_entry("b7e1", b"archive", 1_793_000_000);
        let listed = |store: &FakeStore| store.cache_entries().unwrap();
        assert_eq!(
            listed(&store),
            [(
                BuildId::new("b7e1"),
                Ok(Pushed {
                    digest: d.clone(),
                    at: Some(1_793_000_000)
                })
            )]
        );
        store.pushed(&BuildId::new("b7e1"), Some(1_700_000_000));
        assert_eq!(listed(&store)[0].1.clone().unwrap().at, Some(1_700_000_000));
        store.pushed(&BuildId::new("b7e1"), None);
        assert_eq!(
            listed(&store),
            [(
                BuildId::new("b7e1"),
                Ok(Pushed {
                    digest: d,
                    at: None
                })
            )]
        );
    }

    #[test]
    fn the_fake_store_records_its_cache_operations() {
        let (store, d) = with_entry("b7e1", b"archive", 1_793_000_000);
        store.cache_entries().unwrap();
        store.remove_cache_entry(&BuildId::new("b7e1"), &d).unwrap();
        let calls = store.calls();
        let ours: Vec<_> = calls
            .iter()
            .filter(|c| matches!(c.op, StoreOp::CacheEntries | StoreOp::RemoveCacheEntry))
            .collect();
        assert_eq!(ours.len(), 2, "{calls:?}");
        assert_eq!(ours[0].op, StoreOp::CacheEntries);
        assert_eq!(ours[1].op, StoreOp::RemoveCacheEntry);
        assert_eq!(ours[1].given["build"], "b7e1");
        assert_eq!(ours[1].given["at"], json!(d));
        assert_eq!(ours[1].answer, Ok(json!(null)));
    }
}
