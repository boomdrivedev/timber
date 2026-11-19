//! Spec cache-keeping: prune over the fakes, reading the keeping time at
//! a landed revision and removing each cache entry pushed longer ago,
//! each entry answered on its own, nothing but cache entries removed and
//! nothing asked of the keeper, the signers, the projections or the
//! targets (145; design D7, D8).

mod common;
mod world;

use timber_core::{
    accept, answer_json, cache_name, prune, AcceptAsk, Fate, Judged, PruneAsk, Pruned, ToldBy,
};
use timber_fakes::{FakeSigner, FakeStore, Fakes, Fault, LandingOp, PolicyOp, StoreOp, Times};
use timber_seams::{
    Answer, AskedOf, Audience, Branch, BuildId, ChannelId, ChannelName, Check, ClaimKind,
    ComponentName, Digest, Ecosystem, EcosystemName, Envelope, Held, Keeper, Keeping, Manifest,
    Name, NamedSigner, Not, Outcome, Pushed, Put, Required, RequiredClaim, Revision, Seam, Seams,
    SignerId, Source, Store, TargetProfile, Version, Why,
};
use world::{build_of, version_of, World, HOST, LANDING};

/// The moment every prune here runs at.
const NOW: u64 = 1_800_000_000;

/// A day, in seconds.
const DAY: u64 = 86_400;

/// Fakes whose landing has landed `4f2a9c1` on `main`.
fn landed() -> (Fakes, PruneAsk) {
    let fakes = Fakes::new();
    let (main, revision) = (Branch::new("main"), Revision::new("4f2a9c1"));
    fakes.landing.land(&main, &revision);
    let ask = PruneAsk {
        branch: main,
        revision,
        now: NOW,
    };
    (fakes, ask)
}

/// The cache entry of `build` held in `store`, pushed at `at`, or with
/// no moment.
fn entry(store: &FakeStore, build: &str, at: Option<u64>) -> Digest {
    let digest = store
        .put_bytes(format!("the archive of {build}").as_bytes())
        .unwrap()
        .digest;
    store
        .name(&cache_name(&BuildId::new(build)), &digest, None)
        .unwrap();
    store.pushed(&BuildId::new(build), at);
    digest
}

fn pruned(seams: &Seams, ask: &PruneAsk) -> Pruned {
    prune(seams, ask).unwrap_or_else(|not| panic!("prune answered {not:?}"))
}

/// Each entry's fate, by its build identity.
fn fates(pruned: &Pruned) -> Vec<(&str, Answer<Fate>)> {
    pruned
        .entries
        .iter()
        .map(|(build, fate)| (build.as_str(), fate.clone()))
        .collect()
}

/// The build identities the store still lists.
fn listed(store: &FakeStore) -> Vec<String> {
    store
        .cache_entries()
        .unwrap()
        .into_iter()
        .map(|(build, _)| build.as_str().to_string())
        .collect()
}

fn store_ops(store: &FakeStore, since: usize) -> Vec<StoreOp> {
    store.calls()[since..].iter().map(|c| c.op).collect()
}

// Requirement: Prune reads the keeping time at a landed revision

#[test]
fn a_revision_not_landed_is_refused_before_anything_is_asked() {
    let (fakes, ask) = landed();
    entry(&fakes.store, "b7e1", None);
    let since = fakes.store.calls().len();
    let unlanded = PruneAsk {
        revision: Revision::new("77b0e3d"),
        ..ask
    };
    let Err(Not::Refused(r)) = prune(&fakes.seams(), &unlanded) else {
        panic!("a revision not landed was pruned")
    };
    assert_eq!(
        r.why,
        Why::NotLanded {
            branch: Branch::new("main"),
            revision: Revision::new("77b0e3d"),
        }
    );
    assert!(fakes.policy.calls().is_empty(), "the policy was asked");
    assert!(
        store_ops(&fakes.store, since).is_empty(),
        "the store was asked"
    );
    assert_eq!(listed(&fakes.store), ["b7e1"]);
}

#[test]
fn a_keeping_time_not_read_asks_the_store_nothing() {
    let (fakes, ask) = landed();
    entry(&fakes.store, "b7e1", None);
    let since = fakes.store.calls().len();
    fakes.policy.fault(
        PolicyOp::CacheKept,
        Fault::Unreadable("[cache] names keep, and [cache] holds keep_days alone".into()),
        Times::Once,
    );
    let answered = prune(&fakes.seams(), &ask);
    assert!(
        matches!(&answered, Err(Not::Unreadable(u))
            if u.said.as_deref() == Some("[cache] names keep, and [cache] holds keep_days alone")),
        "{answered:?}"
    );
    assert!(
        store_ops(&fakes.store, since).is_empty(),
        "the store was asked"
    );
    assert_eq!(listed(&fakes.store), ["b7e1"]);
    assert_eq!(
        fakes.policy.calls()[0].given["revision"],
        ask.revision.as_str(),
        "the keeping time was read at another revision"
    );
}

// Requirement: An entry past its keeping time is removed

#[test]
fn an_entry_past_its_time_is_removed_and_one_within_kept() {
    let (fakes, ask) = landed();
    let (old, recent) = (NOW - 31 * DAY, NOW - 29 * DAY);
    entry(&fakes.store, "b7e1", Some(old));
    entry(&fakes.store, "c9d2", Some(recent));
    let pruned = pruned(&fakes.seams(), &ask);
    assert_eq!(
        pruned.keeping,
        Keeping {
            days: 30,
            given: true
        }
    );
    assert_eq!(
        fates(&pruned),
        [
            ("b7e1", Ok(Fate::Removed { pushed: Some(old) })),
            (
                "c9d2",
                Ok(Fate::Kept {
                    pushed: recent,
                    until: NOW + DAY
                })
            ),
        ]
    );
    assert_eq!(listed(&fakes.store), ["c9d2"]);
}

#[test]
fn an_entry_at_its_time_exactly_is_removed() {
    let (fakes, ask) = landed();
    let at = NOW - 30 * DAY;
    entry(&fakes.store, "b7e1", Some(at));
    entry(&fakes.store, "c9d2", Some(at + 1));
    let pruned = pruned(&fakes.seams(), &ask);
    assert_eq!(
        fates(&pruned),
        [
            ("b7e1", Ok(Fate::Removed { pushed: Some(at) })),
            (
                "c9d2",
                Ok(Fate::Kept {
                    pushed: at + 1,
                    until: NOW + 1
                })
            ),
        ]
    );
}

#[test]
fn a_written_keeping_time_is_used() {
    let (fakes, ask) = landed();
    fakes.policy.keeps_at(&ask.revision, 7);
    let at = NOW - 8 * DAY;
    entry(&fakes.store, "b7e1", Some(at));
    let pruned = pruned(&fakes.seams(), &ask);
    assert_eq!(
        pruned.keeping,
        Keeping {
            days: 7,
            given: false
        }
    );
    assert_eq!(
        fates(&pruned),
        [("b7e1", Ok(Fate::Removed { pushed: Some(at) }))]
    );
    let json = serde_json::to_value(&pruned).unwrap();
    assert_eq!(
        json["keeping"],
        serde_json::json!({ "days": 7, "given": false })
    );
    assert_eq!(json["entries"][0]["build"], "b7e1");
    assert_eq!(json["entries"][0]["answer"], "ok");
    assert_eq!(json["entries"][0]["value"]["fate"], "removed");
    assert_eq!(json["entries"][0]["value"]["pushed"], at);
}

#[test]
fn a_keeping_time_of_zero_removes_every_entry() {
    let (fakes, ask) = landed();
    fakes.policy.keeps(0);
    entry(&fakes.store, "a1", Some(NOW));
    entry(&fakes.store, "b2", Some(NOW - 1));
    entry(&fakes.store, "c3", None);
    let pruned = pruned(&fakes.seams(), &ask);
    assert!(
        pruned
            .entries
            .iter()
            .all(|(_, fate)| matches!(fate, Ok(Fate::Removed { .. }))),
        "{pruned:?}"
    );
    assert_eq!(pruned.entries.len(), 3);
    assert!(listed(&fakes.store).is_empty());
}

// Requirement: An entry with no push moment is removed

#[test]
fn an_entry_with_no_moment_is_removed() {
    let (fakes, ask) = landed();
    let bytes = entry(&fakes.store, "b7e1", None);
    let pruned = pruned(&fakes.seams(), &ask);
    assert_eq!(
        fates(&pruned),
        [("b7e1", Ok(Fate::Removed { pushed: None }))]
    );
    assert!(listed(&fakes.store).is_empty());
    assert_eq!(
        fakes.store.read(&bytes),
        Ok(Held::Bytes(b"the archive of b7e1".to_vec())),
        "the bytes went with the name"
    );
}

// Requirement: Each entry is answered on its own

/// A store that lists every entry of its fake and, besides, `unread`
/// answered unreadable in `said`; and, where `forgets` names an entry,
/// forgets it once it is listed, as a store's own retention would
/// between the listing and the removal.
struct Listing {
    inner: FakeStore,
    unread: Option<(&'static str, &'static str)>,
    forgets: Option<&'static str>,
}

impl Store for Listing {
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put> {
        self.inner.put_bytes(bytes)
    }
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put> {
        self.inner.put_artifact(manifest)
    }
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put> {
        self.inner.attach(subject, attestation)
    }
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>> {
        self.inner.claims_of(digest)
    }
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()> {
        self.inner.name(name, to, expected)
    }
    fn resolve(&self, name: &Name) -> Answer<Digest> {
        self.inner.resolve(name)
    }
    fn read(&self, digest: &Digest) -> Answer<Held> {
        self.inner.read(digest)
    }
    fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>> {
        self.inner.versions(ecosystem, name)
    }
    fn cache_entries(&self) -> Answer<Vec<(BuildId, Answer<Pushed>)>> {
        let mut listed = self.inner.cache_entries()?;
        if let Some((build, said)) = self.unread {
            listed.push((
                BuildId::new(build),
                Err(Not::unreadable(
                    &Source::new(Seam::Store, "fake"),
                    format!("the cache entry {build}"),
                    Some(said.into()),
                )),
            ));
        }
        if let Some(build) = self.forgets {
            self.inner.forget(&cache_name(&BuildId::new(build)));
        }
        Ok(listed)
    }
    fn remove_cache_entry(&self, build: &BuildId, at: &Digest) -> Answer<()> {
        self.inner.remove_cache_entry(build, at)
    }
}

fn listing(fakes: &Fakes, listing: Listing) -> Seams {
    let mut seams = fakes.seams();
    seams.store = Box::new(listing);
    seams
}

/// Every build identity a removal was asked of.
fn removals(store: &FakeStore) -> Vec<String> {
    store
        .calls()
        .iter()
        .filter(|c| c.op == StoreOp::RemoveCacheEntry)
        .map(|c| c.given["build"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn an_entry_listed_unreadable_is_carried_and_not_removed() {
    const WORDS: &str = "the tag x1 names sha256:00, which is no cache manifest";
    let (fakes, ask) = landed();
    entry(&fakes.store, "b7e1", None);
    entry(&fakes.store, "c9d2", Some(NOW - 40 * DAY));
    let seams = listing(
        &fakes,
        Listing {
            inner: fakes.store.clone(),
            unread: Some(("x1", WORDS)),
            forgets: None,
        },
    );
    let pruned = pruned(&seams, &ask);
    let answers = fates(&pruned);
    assert!(
        matches!(answers[0].1, Ok(Fate::Removed { .. })),
        "{answers:?}"
    );
    assert!(
        matches!(answers[1].1, Ok(Fate::Removed { .. })),
        "{answers:?}"
    );
    assert_eq!(answers[2].0, "x1");
    assert!(
        matches!(&answers[2].1, Err(Not::Unreadable(u)) if u.said.as_deref() == Some(WORDS)),
        "{answers:?}"
    );
    assert_eq!(removals(&fakes.store), ["b7e1", "c9d2"]);
}

#[test]
fn a_removal_refused_stops_no_other() {
    let (fakes, ask) = landed();
    entry(&fakes.store, "b7e1", None);
    entry(&fakes.store, "c9d2", None);
    fakes.store.fault(
        StoreOp::RemoveCacheEntry,
        Fault::Refused("denied: delete is not allowed".into()),
        Times::Once,
    );
    let pruned = pruned(&fakes.seams(), &ask);
    assert_eq!(
        fates(&pruned),
        [
            (
                "b7e1",
                Err(Not::refused(
                    &Source::new(Seam::Store, "fake"),
                    Why::Said,
                    Some("denied: delete is not allowed".into())
                ))
            ),
            ("c9d2", Ok(Fate::Removed { pushed: None })),
        ]
    );
    assert_eq!(listed(&fakes.store), ["b7e1"]);
}

#[test]
fn an_entry_gone_before_its_removal_is_gone_already() {
    let (fakes, ask) = landed();
    entry(&fakes.store, "b7e1", None);
    entry(&fakes.store, "c9d2", Some(NOW - 40 * DAY));
    let seams = listing(
        &fakes,
        Listing {
            inner: fakes.store.clone(),
            unread: None,
            forgets: Some("b7e1"),
        },
    );
    let pruned = pruned(&seams, &ask);
    assert_eq!(
        fates(&pruned),
        [
            ("b7e1", Ok(Fate::GoneAlready { pushed: None })),
            (
                "c9d2",
                Ok(Fate::Removed {
                    pushed: Some(NOW - 40 * DAY)
                })
            ),
        ]
    );
    assert!(listed(&fakes.store).is_empty());
}

// Requirement: Prune removes cache entries and nothing else

#[test]
fn prune_asks_nothing_of_the_keeper_the_signers_the_projections_or_the_targets() {
    let (mut fakes, ask) = landed();
    let projection = fakes.projection(&Ecosystem::new("eco-a"));
    let target = fakes.target(&TargetProfile::new("pushed"));
    entry(&fakes.store, "b7e1", None);
    entry(&fakes.store, "c9d2", Some(NOW));
    let since = fakes.store.calls().len();
    pruned(&fakes.seams(), &ask);
    assert!(fakes.keeper.calls().is_empty(), "the keeper was asked");
    assert!(fakes.signer.calls().is_empty(), "a signer was asked");
    assert!(projection.calls().is_empty(), "a projection was asked");
    assert!(target.calls().is_empty(), "a target was asked");
    assert!(fakes.build.calls().is_empty(), "the build was asked");
    assert!(fakes.roots.calls().is_empty(), "the roots were asked");
    assert_eq!(
        fakes
            .landing
            .calls()
            .iter()
            .map(|c| c.op)
            .collect::<Vec<_>>(),
        [LandingOp::Landed]
    );
    assert_eq!(
        fakes
            .policy
            .calls()
            .iter()
            .map(|c| c.op)
            .collect::<Vec<_>>(),
        [PolicyOp::CacheKept]
    );
    assert_eq!(
        store_ops(&fakes.store, since),
        [StoreOp::CacheEntries, StoreOp::RemoveCacheEntry]
    );
    let json = answer_json(&prune(&fakes.seams(), &ask));
    assert_eq!(json["answer"], "ok", "{json}");
}

/// The gate's identity, held by a signer of its own beside the host's.
const GATE: &str = "gate";

fn named(id: &str, kinds: &[ClaimKind], checks: &[&str]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: checks.iter().map(|c| Check::new(*c)).collect(),
    }
}

/// `lib`'s `beta`.
fn beta() -> ChannelId {
    ChannelId::new(Some(ComponentName::new("lib")), ChannelName::new("beta"))
}

/// A world whose `lib` was published at its head from the entries its
/// check jobs pushed, accepted by the gate under the host's provenance,
/// and promoted to `beta`, which follows each publish (95).
fn released() -> (World, Digest) {
    let world = World::new();
    for o in ["lib-pack", "lib-docs"] {
        world.verdict("test", &build_of(o, &world.head), Outcome::Pass, LANDING);
    }
    world.fakes.policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Provenance,
                check: None,
                environment: None,
                signers: vec![named(HOST, &[ClaimKind::Provenance], &[])],
                asked_of: AskedOf::Every,
            }],
        },
    );
    world.fakes.policy.require(
        Audience::Channel(beta()),
        Required::Channel {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                environment: None,
                signers: vec![named(GATE, &[ClaimKind::Acceptance], &[])],
                asked_of: AskedOf::Every,
            }],
            bound_to: None,
            promoted_by: vec![named(GATE, &[ClaimKind::Acceptance], &[])],
            approvers: vec![],
        },
    );
    world.fakes.policy.follows(&beta(), false);
    world.pushed(&["lib-pack", "lib-docs"]);
    let published = world.publish_one(&world.ask_for("lib")).unwrap();
    let (version, index) = version_of(&published);
    let (version, index) = (version.clone(), index.clone());
    let mut seams = world.fakes.seams();
    let gate = FakeSigner::new();
    gate.hold(&SignerId::new(GATE));
    seams.signers.push(Box::new(gate));
    let judged = accept(
        &seams,
        &world.history(),
        &AcceptAsk {
            branch: world.main.clone(),
            component: Some(ComponentName::new("lib")),
            version,
            told: ToldBy::Publish {
                index: index.clone(),
            },
            gate: SignerId::new(GATE),
            when: NOW,
        },
    );
    assert!(
        matches!(judged, Ok(Judged::Accepted { .. })),
        "the version was not accepted: {judged:?}"
    );
    (world, index)
}

#[test]
fn a_released_version_reads_back_whole_after_its_entries_are_removed() {
    let (world, index) = released();
    let store = &world.fakes.store;
    let names_outside: Vec<(Name, Digest)> = store
        .contents()
        .names
        .into_iter()
        .filter(|(name, _)| !name.as_str().starts_with("cache/"))
        .collect();
    assert!(
        names_outside
            .iter()
            .any(|(name, at)| name.as_str() == "channel/lib/beta" && *at == index),
        "beta was not promoted: {names_outside:?}"
    );
    let claims: Vec<(Digest, Vec<Envelope>)> = store
        .contents()
        .attached
        .keys()
        .map(|digest| (digest.clone(), store.claims_of(digest).unwrap()))
        .collect();
    let chain = world.fakes.keeper.chain(&beta()).unwrap().claims;
    for build in ["lib-pack", "lib-docs"] {
        store.pushed(&build_of(build, &world.head), Some(NOW - 31 * DAY));
    }
    let entries_before = listed(store);
    assert_eq!(entries_before.len(), 2, "{entries_before:?}");
    let counts = (
        world.fakes.keeper.calls().len(),
        world.fakes.signer.calls().len(),
        world.fakes.store.calls().len(),
    );

    let ask = PruneAsk {
        branch: world.main.clone(),
        revision: world.head.clone(),
        now: NOW,
    };
    let pruned = pruned(&world.fakes.seams(), &ask);
    assert_eq!(
        world.fakes.keeper.calls().len(),
        counts.0,
        "the keeper was asked"
    );
    assert_eq!(
        world.fakes.signer.calls().len(),
        counts.1,
        "a signer was asked"
    );
    assert_eq!(
        store_ops(store, counts.2),
        [
            StoreOp::CacheEntries,
            StoreOp::RemoveCacheEntry,
            StoreOp::RemoveCacheEntry
        ],
        "prune asked the store otherwise than to list and remove cache entries"
    );
    assert!(
        pruned
            .entries
            .iter()
            .all(|(_, fate)| matches!(fate, Ok(Fate::Removed { .. }))),
        "{pruned:?}"
    );
    assert!(listed(store).is_empty());
    for (name, at) in &names_outside {
        assert_eq!(store.resolve(name).as_ref(), Ok(at), "{name}");
    }
    for (digest, envelopes) in &claims {
        assert_eq!(store.claims_of(digest).as_ref(), Ok(envelopes), "{digest}");
    }
    assert_eq!(world.fakes.keeper.chain(&beta()).unwrap().claims, chain);
}
