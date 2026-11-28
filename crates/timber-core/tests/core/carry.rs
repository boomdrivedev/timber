//! A moved repository's history carried between two roots over the
//! fakes, and a move read and carried at a publish's start (C.1, 22, 103,
//! 105; design D3 to D5, D7, D8).

use crate::world::World;
use serde_json::json;
use timber_core::carry::{self, ChannelCarried, Marked, Root, Rooted, VersionCarried};
use timber_core::{Declared, Landed, ROOT_REF};
use timber_fakes::{FakeKeeper, FakeStore, KeeperOp, RootsOp, StoreOp};
use timber_seams::{
    channel_name, moved_name, Answer, Artifact, ArtifactMeta, Audience, ChannelId, ChannelName,
    Claim, ClaimKind, ComponentName, DeclaredEnvironment, Digest, Ecosystem, EcosystemName,
    Envelope, EnvironmentName, Held, IndexMeta, Keeper, Manifest, Moved, MovedTo, Name, Not,
    Promotion, RepositoryName, Required, Revision, Satisfied, Seam, Signature, SignerId, Source,
    Store, Version, Why, IN_TOTO,
};

fn root(owner: &str, name: &str) -> RepositoryName {
    RepositoryName {
        owner: owner.into(),
        name: name.into(),
    }
}

fn app() -> RepositoryName {
    root("acme", "app")
}

fn site() -> RepositoryName {
    root("acme", "site")
}

fn beta() -> ChannelId {
    ChannelId::new(None, ChannelName::new("beta"))
}

fn stable() -> ChannelId {
    ChannelId::new(None, ChannelName::new("stable"))
}

fn sealed(claim: &Claim) -> Envelope {
    Envelope::signed(
        IN_TOTO,
        claim.payload(),
        Signature {
            signer: SignerId::new("gate"),
            keyid: String::new(),
            sig: vec![7],
        },
    )
}

/// The version `v<version>` published into `store`: one artifact with a
/// provenance, the index with an acceptance, named; its index.
fn published(store: &FakeStore, version: &str) -> Digest {
    published_as(store, &name(version), version)
}

/// The version `version` published into `store` and named `named`.
fn published_as(store: &FakeStore, named: &Name, version: &str) -> Digest {
    let meta = ArtifactMeta::new(
        Ecosystem::new("file"),
        EcosystemName::new("site"),
        Version::new(version),
    )
    .unwrap();
    let bytes = store.put_bytes(version.as_bytes()).unwrap().digest;
    let artifact = store
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![bytes],
        })
        .unwrap()
        .digest;
    let claim = |subject: &Digest, kind| {
        sealed(&Claim {
            subject: subject.clone(),
            kind,
            revision: Revision::new("4f2a9c1"),
            build: None,
            body: json!({ "of": version }),
        })
    };
    store
        .attach(&artifact, &claim(&artifact, ClaimKind::Provenance))
        .unwrap();
    let index = store
        .put_artifact(&Manifest::Index(
            IndexMeta::new(
                ComponentName::new("timber"),
                Version::new(version),
                vec![Artifact {
                    digest: artifact,
                    meta,
                }],
            )
            .unwrap(),
        ))
        .unwrap()
        .digest;
    store
        .attach(&index, &claim(&index, ClaimKind::Acceptance))
        .unwrap();
    store.name(named, &index, None).unwrap();
    index
}

fn name(version: &str) -> Name {
    Name::new(format!("v{version}"))
}

/// `channel` promoted to `version` at `index` in `keeper` after its head,
/// the promotion attached beside the index in `store` and the channel's
/// name moved there; the promotion's envelope.
fn promoted(
    store: &FakeStore,
    keeper: &FakeKeeper,
    channel: &ChannelId,
    version: &str,
    index: &Digest,
) -> Envelope {
    let before = keeper.chain(channel).ok().map(|c| c.claims);
    let predecessor = before.as_ref().and_then(|c| c.last()).map(Envelope::digest);
    let from = before
        .as_ref()
        .and_then(|c| c.last())
        .and_then(Claim::from_envelope)
        .and_then(|c| Promotion::of(&c))
        .map(|p| p.to);
    let promotion = Promotion {
        channel: channel.name.clone(),
        component: ComponentName::new("timber"),
        to: Moved {
            version: Version::new(version),
            index: index.clone(),
        },
        from: from.clone(),
        predecessor: predecessor.clone(),
        asked_by: SignerId::new("gate"),
        policy: Satisfied {
            revision: Revision::new("4f2a9c1"),
            claims: vec![],
        },
        at: 1_793_000_000,
    };
    let envelope = sealed(&promotion.claim(Revision::new("4f2a9c1")));
    keeper
        .append(channel, &envelope, predecessor.as_ref())
        .unwrap();
    store.attach(index, &envelope).unwrap();
    store
        .name(
            &channel_name(channel),
            index,
            from.as_ref().map(|m| &m.index),
        )
        .unwrap();
    envelope
}

/// Two roots: `acme/app`, left, holding `v1.1.0` and `v1.2.0`, `beta` at
/// `v1.2.0` and `stable` at `v1.1.0`, and `acme/site`, bound, holding
/// nothing; one keeper over both, as the git ref ledger's refs move with
/// the repository.
struct Moving {
    left: FakeStore,
    bound: FakeStore,
    keeper: FakeKeeper,
    indexes: (Digest, Digest),
    heads: (Envelope, Envelope),
    sources: (Source, Source),
}

impl Moving {
    fn new() -> Self {
        let (left, keeper) = (FakeStore::new(), FakeKeeper::new());
        let one = published(&left, "1.1.0");
        let two = published(&left, "1.2.0");
        let beta_head = promoted(&left, &keeper, &beta(), "1.2.0", &two);
        let stable_head = promoted(&left, &keeper, &stable(), "1.1.0", &one);
        Moving {
            left,
            bound: FakeStore::new(),
            keeper,
            indexes: (one, two),
            heads: (beta_head, stable_head),
            sources: (
                Source::new(Seam::Store, "fake acme/app"),
                Source::new(Seam::Store, "fake acme/site"),
            ),
        }
    }

    fn carry_with(&self, bound_keeper: &dyn Keeper, versions: &[&str]) -> Answer<carry::Carried> {
        let (app, site) = (app(), site());
        let names: Vec<Name> = versions.iter().map(|v| name(v)).collect();
        carry::history(
            Root {
                repository: &app,
                store: &self.left,
                source: &self.sources.0,
                keeper: &self.keeper,
            },
            Root {
                repository: &site,
                store: &self.bound,
                source: &self.sources.1,
                keeper: bound_keeper,
            },
            &names,
            &[beta(), stable()],
        )
    }

    fn carry(&self) -> Answer<carry::Carried> {
        self.carry_with(&self.keeper, &["1.1.0", "1.2.0"])
    }
}

fn mark_digest() -> Digest {
    Digest::of(&MovedTo { to: site() }.to_bytes())
}

#[test]
fn a_carry_marks_the_root_left_before_it_copies_anything() {
    let moving = Moving::new();
    let asked = moving.left.calls().len();
    let carried = moving.carry().unwrap();
    assert_eq!(carried.mark, Marked::Made);
    let calls = moving.left.calls();
    let ops: Vec<StoreOp> = calls[asked..].iter().map(|c| c.op).collect();
    assert_eq!(
        &ops[..3],
        [StoreOp::PutBytes, StoreOp::Name, StoreOp::Resolve]
    );
    assert_eq!(calls[asked + 1].given["name"], "moved");
    assert_eq!(
        moving.left.contents().names.get(&moved_name()),
        Some(&mark_digest())
    );
    assert_eq!(
        moving.left.contents().held.get(&mark_digest()),
        Some(&Held::Bytes(br#"{"moved_to":"acme/site"}"#.to_vec()))
    );
    assert!(
        !moving.bound.contents().names.contains_key(&moved_name()),
        "the bound root was marked"
    );
}

#[test]
fn a_carry_copies_each_version_with_every_claim_on_it() {
    let moving = Moving::new();
    let carried = moving.carry().unwrap();
    assert_eq!(
        carried
            .versions
            .iter()
            .map(|v| (v.name.as_str(), v.carried))
            .collect::<Vec<_>>(),
        [
            ("v1.1.0", VersionCarried::Copied),
            ("v1.2.0", VersionCarried::Copied)
        ]
    );
    let (left, bound) = (moving.left.contents(), moving.bound.contents());
    for (version, index) in [("1.1.0", &moving.indexes.0), ("1.2.0", &moving.indexes.1)] {
        assert_eq!(bound.names.get(&name(version)), Some(index));
        assert_eq!(bound.attached.get(index), left.attached.get(index));
    }
    let claims = bound.attached.get(&moving.indexes.1).unwrap();
    assert!(claims.contains(&moving.heads.0), "the promotion was left");
    assert_eq!(claims.len(), 2);
    assert_eq!((carried.from, carried.to), (app(), site()));
}

#[test]
fn a_carry_passes_over_a_version_the_root_left_does_not_hold() {
    let moving = Moving::new();
    let carried = moving
        .carry_with(&moving.keeper, &["1.0.0", "1.1.0", "1.2.0"])
        .unwrap();
    assert_eq!(carried.versions[0].name, name("1.0.0"));
    assert_eq!(carried.versions[0].carried, VersionCarried::NotHeld);
    assert_eq!(carried.versions[1].carried, VersionCarried::Copied);
    assert!(!moving.bound.contents().names.contains_key(&name("1.0.0")));
}

#[test]
fn a_carry_makes_each_channels_name_at_its_heads_index() {
    let moving = Moving::new();
    let carried = moving.carry().unwrap();
    assert_eq!(carried.channels.len(), 2);
    for (carried, (channel, head, index)) in carried.channels.iter().zip([
        (beta(), &moving.heads.0, &moving.indexes.1),
        (stable(), &moving.heads.1, &moving.indexes.0),
    ]) {
        assert_eq!(carried.channel, channel);
        assert_eq!(carried.head, head.digest());
        assert_eq!(&carried.index, index);
        assert_eq!(carried.name, ChannelCarried::Made);
        assert_eq!(
            moving.bound.contents().names.get(&channel_name(&channel)),
            Some(index)
        );
    }
}

#[test]
fn a_carry_passes_over_a_channel_with_no_chain() {
    let moving = Moving::new();
    let (app, site) = (app(), site());
    let nightly = ChannelId::new(None, ChannelName::new("nightly"));
    let carried = carry::history(
        Root {
            repository: &app,
            store: &moving.left,
            source: &moving.sources.0,
            keeper: &moving.keeper,
        },
        Root {
            repository: &site,
            store: &moving.bound,
            source: &moving.sources.1,
            keeper: &moving.keeper,
        },
        &[name("1.2.0")],
        &[nightly.clone(), beta()],
    )
    .unwrap();
    assert_eq!(
        carried
            .channels
            .iter()
            .map(|c| c.channel.clone())
            .collect::<Vec<_>>(),
        [beta()]
    );
    assert!(!moving
        .bound
        .contents()
        .names
        .contains_key(&channel_name(&nightly)));
}

#[test]
fn a_carry_of_a_chain_as_last_read_is_unreadable() {
    let moving = Moving::new();
    moving.keeper.chain(&beta()).unwrap();
    moving.keeper.unreachable("ref unreadable");
    match moving.carry() {
        Err(Not::Unreadable(u)) => {
            assert_eq!(u.source.seam, Seam::Keeper);
            assert_eq!(u.what, "the chain of beta");
            assert_eq!(u.said.as_deref(), Some("ref unreadable"));
        }
        other => panic!("a chain as last read was carried: {other:?}"),
    }
    assert!(!moving
        .bound
        .contents()
        .names
        .contains_key(&channel_name(&beta())));
}

#[test]
fn a_carry_again_writes_nothing() {
    let moving = Moving::new();
    moving.carry().unwrap();
    let (left, bound) = (moving.left.contents(), moving.bound.contents());
    let again = moving.carry().unwrap();
    assert_eq!(again.mark, Marked::HeldAlready);
    assert!(again
        .versions
        .iter()
        .all(|v| v.carried == VersionCarried::HeldAlready));
    assert!(again
        .channels
        .iter()
        .all(|c| c.name == ChannelCarried::HeldAlready));
    assert_eq!(moving.left.contents(), left);
    assert_eq!(moving.bound.contents(), bound);
}

#[test]
fn a_carry_leaves_a_channel_moved_on_since_where_it_is() {
    let moving = Moving::new();
    moving.carry().unwrap();
    let three = published(&moving.bound, "1.3.0");
    let on = promoted(&moving.bound, &moving.keeper, &beta(), "1.3.0", &three);

    // The root left's chain, read through a keeper of its own, still has
    // its head where the carry found it.
    let left_keeper = FakeKeeper::new();
    for (channel, head) in [(beta(), &moving.heads.0), (stable(), &moving.heads.1)] {
        left_keeper.append(&channel, head, None).unwrap();
    }
    let (app, site) = (app(), site());
    let again = carry::history(
        Root {
            repository: &app,
            store: &moving.left,
            source: &moving.sources.0,
            keeper: &left_keeper,
        },
        Root {
            repository: &site,
            store: &moving.bound,
            source: &moving.sources.1,
            keeper: &moving.keeper,
        },
        &[name("1.1.0"), name("1.2.0")],
        &[beta(), stable()],
    )
    .unwrap();
    assert_eq!(again.channels[0].name, ChannelCarried::MovedOn);
    assert_eq!(again.channels[0].head, moving.heads.0.digest());
    assert_eq!(
        moving.bound.contents().names.get(&channel_name(&beta())),
        Some(&three)
    );
    assert_eq!(
        moving.keeper.chain(&beta()).unwrap().claims.last(),
        Some(&on)
    );
}

#[test]
fn a_carry_finding_two_heads_is_refused_as_a_fork() {
    let moving = Moving::new();
    let (bound_keeper, elsewhere) = (FakeKeeper::new(), published(&moving.bound, "1.3.0"));
    let theirs = promoted(&moving.bound, &bound_keeper, &beta(), "1.3.0", &elsewhere);
    match moving.carry_with(&bound_keeper, &["1.1.0", "1.2.0"]) {
        Err(Not::Refused(r)) => {
            assert_eq!(r.source, moving.sources.1);
            assert_eq!(
                r.why,
                Why::Fork {
                    predecessor: None,
                    claims: vec![moving.heads.0.digest(), theirs.digest()],
                }
            );
        }
        other => panic!("two heads were carried: {other:?}"),
    }
    assert_eq!(
        moving.bound.contents().names.get(&channel_name(&beta())),
        Some(&elsewhere)
    );
}

#[test]
fn a_carry_finding_a_mark_naming_another_root_is_refused_naming_it() {
    let moving = Moving::new();
    let web = moving
        .left
        .put_bytes(
            &MovedTo {
                to: root("acme", "web"),
            }
            .to_bytes(),
        )
        .unwrap()
        .digest;
    moving.left.name(&moved_name(), &web, None).unwrap();
    assert_eq!(
        moving.carry().map(|_| ()),
        Err(Not::refused(
            &moving.sources.0,
            Why::RootMoved {
                from: app(),
                to: root("acme", "web"),
            },
            None
        ))
    );
    assert!(
        moving.bound.calls().is_empty(),
        "the bound root was written"
    );
}

#[test]
fn a_carry_reads_and_writes_nothing_of_the_cache() {
    let moving = Moving::new();
    let entry = moving.left.put_bytes(b"a cache entry").unwrap().digest;
    let cache = Name::new("cache/b7e1");
    moving.left.name(&cache, &entry, None).unwrap();
    let asked = moving.left.calls().len();
    moving.carry().unwrap();
    for call in &moving.left.calls()[asked..] {
        let given = call.given.to_string();
        assert!(
            !given.contains("cache/") && !given.contains(entry.as_str()),
            "the cache was read: {given}"
        );
    }
    for call in moving.bound.calls() {
        assert!(!call.given.to_string().contains("cache/"));
    }
    assert!(!moving.bound.contents().names.contains_key(&cache));
    assert!(!moving.bound.contents().held.contains_key(&entry));
}

/// The world's landed head and what its policy file declares there.
fn landed(world: &World) -> (Landed, Declared) {
    let landed = Landed::check(&world.fakes.landing, &world.main, &world.head).unwrap();
    let declared = Declared::read(&world.history(), &landed).unwrap();
    (landed, declared)
}

fn rooted(world: &World) -> Answer<Option<Rooted>> {
    let (landed, declared) = landed(world);
    carry::root(&world.fakes.seams(), &world.history(), &landed, &declared)
}

fn record(world: &World) -> Vec<RepositoryName> {
    world.history().roots(&landed(world).0).unwrap()
}

fn recorded(world: &World, root: &RepositoryName) {
    world.history().record_root(&landed(world).0, root).unwrap();
}

fn lib(channel: &str) -> ChannelId {
    ChannelId::new(Some(ComponentName::new("lib")), ChannelName::new(channel))
}

fn web(channel: &str) -> ChannelId {
    ChannelId::new(Some(ComponentName::new("web")), ChannelName::new(channel))
}

fn in_force(world: &World, channel: &ChannelId) {
    world.fakes.policy.require(
        Audience::Channel(channel.clone()),
        Required::Channel {
            claims: vec![],
            bound_to: None,
            promoted_by: vec![],
            approvers: vec![],
        },
    );
}

/// The world moved from `acme/app` to `acme/site`: the record naming
/// `acme/app`, the roots bound to `acme/site`, and `acme/app` bound with
/// a store holding the world's two tagged versions, `lib/beta` promoted
/// to `lib`'s, and a keeper of its own; the root left's store and keeper.
/// The bound keeper answers `lib/beta`'s chain as the root left's does,
/// as the git ref ledger's refs, which move with the repository, would.
fn moved(world: &World) -> (FakeStore, FakeKeeper) {
    recorded(world, &app());
    world.fakes.roots.bind(&site());
    let (left, keeper) = (FakeStore::new(), FakeKeeper::new());
    let lib_index = published_as(&left, &Name::new("lib:lib-v0.1.0"), "0.1.0");
    published_as(&left, &Name::new("web:web-v0.1.0"), "0.1.0");
    let head = promoted(&left, &keeper, &lib("beta"), "0.1.0", &lib_index);
    world
        .fakes
        .keeper
        .append(&lib("beta"), &head, None)
        .unwrap();
    world.fakes.roots.leave(&app(), &left, &keeper);
    in_force(world, &lib("beta"));
    (left, keeper)
}

fn roots_ops(world: &World) -> Vec<RootsOp> {
    world.fakes.roots.calls().iter().map(|c| c.op).collect()
}

#[test]
fn no_root_bound_reads_nothing_and_writes_nothing() {
    let world = World::new();
    assert_eq!(rooted(&world), Ok(None));
    assert!(world.fakes.store.calls().is_empty(), "the store was asked");
    assert!(
        world.fakes.keeper.calls().is_empty(),
        "the ledger was asked"
    );
    assert_eq!(roots_ops(&world), [RootsOp::Bound]);
    assert_eq!(
        world
            .shared
            .refs()
            .iter()
            .filter(|(name, _)| name == ROOT_REF)
            .count(),
        0,
        "the record was written"
    );
}

#[test]
fn an_empty_record_records_the_bound_root() {
    let world = World::new();
    world.fakes.roots.bind(&root("Acme", "Site"));
    assert_eq!(rooted(&world), Ok(Some(Rooted::Recorded { root: site() })));
    assert_eq!(record(&world), [site()]);
    assert_eq!(roots_ops(&world), [RootsOp::Bound]);
    assert!(world
        .fakes
        .store
        .calls()
        .iter()
        .all(|c| c.op == StoreOp::Resolve));
}

#[test]
fn the_same_root_in_another_case_carries_nothing() {
    let world = World::new();
    recorded(&world, &site());
    world.fakes.roots.bind(&root("Acme", "Site"));
    assert_eq!(rooted(&world), Ok(None));
    assert_eq!(record(&world), [site()]);
    assert_eq!(roots_ops(&world), [RootsOp::Bound]);
}

#[test]
fn a_moved_repository_is_carried_and_its_root_recorded_last() {
    let world = World::new();
    let (left, _) = moved(&world);
    let answer = rooted(&world);
    let Ok(Some(Rooted::Carried(carried))) = answer else {
        panic!("the move was not carried: {answer:?}")
    };
    assert_eq!((carried.from.clone(), carried.to.clone()), (app(), site()));
    assert_eq!(carried.mark, Marked::Made);
    assert_eq!(
        carried
            .versions
            .iter()
            .map(|v| (v.name.as_str(), v.carried))
            .collect::<Vec<_>>(),
        [
            ("lib:lib-v0.1.0", VersionCarried::Copied),
            ("web:web-v0.1.0", VersionCarried::Copied)
        ]
    );
    assert_eq!(carried.channels.len(), 1);
    assert_eq!(carried.channels[0].channel, lib("beta"));
    assert_eq!(carried.channels[0].name, ChannelCarried::Made);
    assert_eq!(roots_ops(&world), [RootsOp::Bound, RootsOp::Left]);
    assert_eq!(world.fakes.roots.calls()[1].given["from"]["name"], "app");
    assert_eq!(record(&world), [app(), site()]);
    assert_eq!(
        left.contents().names.get(&moved_name()),
        Some(&mark_digest())
    );
    let bound = world.fakes.store.contents();
    assert!(bound.names.contains_key(&Name::new("lib:lib-v0.1.0")));
    assert_eq!(
        bound.names.get(&channel_name(&lib("beta"))),
        bound.names.get(&Name::new("lib:lib-v0.1.0"))
    );

    // Carried and recorded, the next publish's read carries nothing.
    assert_eq!(rooted(&world), Ok(None));
}

#[test]
fn a_carry_refused_leaves_the_record_at_the_root_left() {
    let world = World::new();
    let (left, _) = moved(&world);
    let web_mark = left
        .put_bytes(
            &MovedTo {
                to: root("acme", "web"),
            }
            .to_bytes(),
        )
        .unwrap()
        .digest;
    left.name(&moved_name(), &web_mark, None).unwrap();
    match rooted(&world) {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::RootMoved {
                from: app(),
                to: root("acme", "web"),
            }
        ),
        other => panic!("a carry from a root marked elsewhere answered {other:?}"),
    }
    assert_eq!(record(&world), [app()]);
}

#[test]
fn a_bound_root_holding_a_mark_is_refused_naming_where_it_went() {
    let world = World::new();
    recorded(&world, &app());
    world.fakes.roots.bind(&site());
    let store = &world.fakes.store;
    let web_mark = store
        .put_bytes(
            &MovedTo {
                to: root("acme", "web"),
            }
            .to_bytes(),
        )
        .unwrap()
        .digest;
    store.name(&moved_name(), &web_mark, None).unwrap();
    assert_eq!(
        rooted(&world),
        Err(Not::refused(
            &Source::new(Seam::Store, "bound"),
            Why::RootMoved {
                from: site(),
                to: root("acme", "web"),
            },
            None
        ))
    );
    assert_eq!(roots_ops(&world), [RootsOp::Bound]);
    assert_eq!(record(&world), [app()]);
}

#[test]
fn a_carry_lists_every_declared_components_versions_and_every_channel_once() {
    let world = World::new();
    world.repo.tag("lib-v0.9.0", &world.start);
    world.repo.tag("lib-v0.10.0", &world.head);
    let (_, keeper) = moved(&world);
    for channel in [lib("stable"), web("beta")] {
        in_force(&world, &channel);
    }
    for (name, channel) in [("first", lib("beta")), ("edge", web("edge"))] {
        world.fakes.policy.declares_at(
            &world.head,
            DeclaredEnvironment {
                name: EnvironmentName::new(name),
                channel,
                waits: false,
                approvers: vec![],
                target: None,
            },
        );
    }
    let asked = keeper.calls().len();
    let answer = rooted(&world);
    let Ok(Some(Rooted::Carried(carried))) = answer else {
        panic!("the move was not carried: {answer:?}")
    };
    assert_eq!(
        carried
            .versions
            .iter()
            .map(|v| v.name.as_str())
            .collect::<Vec<_>>(),
        [
            "lib:lib-v0.1.0",
            "lib:lib-v0.9.0",
            "lib:lib-v0.10.0",
            "web:web-v0.1.0"
        ]
    );
    let asked_of: Vec<String> = keeper.calls()[asked..]
        .iter()
        .filter(|c| c.op == KeeperOp::Chain)
        .map(|c| c.given["channel"].to_string())
        .collect();
    let mut once = asked_of.clone();
    once.dedup();
    assert_eq!(asked_of.len(), 4, "{asked_of:?}");
    assert_eq!(once, asked_of, "a channel was asked twice: {asked_of:?}");
}
