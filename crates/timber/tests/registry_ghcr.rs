//! Spec registry-store, its run against GitHub's registry: the
//! conformance suite's store group and the cache manifest's scenarios
//! under the root the host names in `env.TIMBER_TOOLS_GHCR` of its
//! `devenv.local.nix`, bound for the repository `timber-checks/store`,
//! reached with the host's Docker credential for `ghcr.io`, every name
//! they pass prefixed by the run so no run's names meet an earlier one's,
//! through the live rig the run against ECR shares (design D4, D5; the
//! change each-repositorys-spaces-sit-under-its-own-root's design D7). A
//! third test makes an artifact's name in a package the push makes,
//! reports the mounts the registry took, and reads the index there with
//! no credential, which must never be answered the index. A fourth makes
//! a declared component's version, a cache entry and an artifact's name,
//! and finds each in the package C.1 places it in, the version's and the
//! entry's under `<root>/timber-checks/store/` and the name's under
//! `<root>/artifact/`, and the index's claims read from the name's
//! package by a store bound for another repository. A fifth carries a
//! moved repository's history from `timber-checks/carried-from` to
//! `timber-checks/carried-to` under the root, every version and channel
//! under the run's prefix: the version and its claims and the channel
//! found under the new root, and the old root's package answering where
//! it went, a later run finding the mark naming the same root and
//! carrying again (C.1; the change
//! a-moved-repository-keeps-its-history-in-the-store's design D8). It
//! refuses to run,
//! naming the setting, on a host that names no root of `ghcr.io`, rather
//! than pass without running (93, C.1, rule core.38).

#![cfg(feature = "tools")]

mod registry;

use registry::{cache_entries, live, run_prefix, store_group_against, store_under, Prefixed, LIVE};
use timber_seams::{Digest, Envelope, Held, Manifest, Name, Signature, SignerId, Store, IN_TOTO};

/// The setting a host names its root of GitHub's registry by.
const SETTING: &str = "TIMBER_TOOLS_GHCR";

/// GitHub's registry's host.
const GHCR: &str = "ghcr.io";

/// What a read of a manifest accepts: timber's two manifest types.
const MANIFESTS: &str = "application/vnd.oci.image.index.v1+json, \
                         application/vnd.oci.image.manifest.v1+json";

/// The root the host names, `ghcr.io/<owner>/<path>`; a panic naming the
/// setting where it names none, or one of another host.
fn repository() -> String {
    let named = std::env::var(SETTING).unwrap_or_default();
    let named = named.trim();
    let mut parts = named.split('/');
    let ghcr = parts.next() == Some(GHCR);
    let path: Vec<&str> = parts.collect();
    if !ghcr || path.len() < 2 || path.iter().any(|p| p.is_empty()) {
        panic!(
            "the GHCR run names no root of GitHub's registry ({SETTING} is {named:?}): set \
             env.{SETTING} in devenv.local.nix to the root the GHCR check pushes under, \
             {GHCR}/<owner>/<path>, with a credential for {GHCR} in the host's Docker \
             configuration, a token with write:packages as `docker login {GHCR}` keeps it"
        );
    }
    named.to_string()
}

#[test]
fn ghcr_keeps_the_stores_contract() {
    store_group_against(&repository());
}

#[test]
fn ghcr_names_a_cache_entry_through_a_cache_manifest() {
    let registry = live(&repository()).bound_for("moon", false);
    let within = registry.bound();
    cache_entries(
        &Prefixed {
            registry,
            prefix: format!("{}-cache-", run_prefix()),
        },
        within,
    );
}

/// What the run records in record 0039 (design D5): an artifact's name
/// made at an index, so the push makes the name's repository under the
/// root, with whether GitHub's registry took each
/// cross-repository mount; and a read of the index in that repository
/// with no credential, which is never answered the index, since a
/// repository made public on push would publish a private store's
/// artifacts. Run with `-- --nocapture` to read both.
#[test]
fn ghcr_makes_an_artifacts_repository_on_push_and_keeps_it_private() {
    let repository = repository();
    let prefix = run_prefix();
    let registry = live(&repository);
    let meta = timber_seams::ArtifactMeta::new(
        timber_seams::Ecosystem::new("file"),
        timber_seams::EcosystemName::new(format!("{prefix}-site")),
        timber_seams::Version::new("1.0.0"),
    )
    .unwrap();
    let layer = registry
        .put_bytes(format!("{prefix} the site").as_bytes())
        .unwrap()
        .digest;
    let artifact = registry
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    let index = timber_seams::IndexMeta::new(
        timber_seams::ComponentName::new(timber_seams::REPOSITORY_COMPONENT),
        meta.version().clone(),
        vec![timber_seams::Artifact {
            digest: artifact,
            meta: meta.clone(),
        }],
    )
    .unwrap();
    let index = registry
        .put_artifact(&Manifest::Index(index))
        .unwrap()
        .digest;
    let name = timber_seams::artifact_name(meta.ecosystem(), meta.name(), meta.version());
    registry
        .name(&name, &index, None)
        .expect("the artifact name's repository, which the push makes");
    assert_eq!(registry.resolve(&name).as_ref(), Ok(&index));
    let mounts = registry.mounts();
    let taken = mounts.iter().filter(|(_, taken)| *taken).count();
    println!(
        "ghcr: {taken} of {} cross-repository mounts were taken, into {:?}",
        mounts.len(),
        mounts
            .iter()
            .map(|(r, _)| r)
            .collect::<std::collections::BTreeSet<_>>()
    );

    let (_, path) = repository.split_once('/').unwrap();
    let made = format!(
        "{path}/artifact/{}/{}",
        meta.ecosystem().as_str(),
        meta.name().as_str()
    );
    let (served, said) = anonymous(&made, &index);
    assert!(
        !served,
        "{GHCR}/{made} answered its index to a read with no credential: GitHub's registry made \
         it public on push, which would publish a private store's artifacts: {said}"
    );
    println!("ghcr: a read of {GHCR}/{made} with no credential answered: {said}");
}

/// An attestation holding `words`, as a signer would make it.
fn claim(words: &str) -> Envelope {
    Envelope::signed(
        IN_TOTO,
        words.as_bytes().to_vec(),
        Signature {
            signer: SignerId::new("the GHCR check"),
            keyid: String::new(),
            sig: vec![7],
        },
    )
}

/// C.1 on GitHub's registry (design D7 of the change
/// each-repositorys-spaces-sit-under-its-own-root): a declared component's
/// version and a cache entry made for the repository [`LIVE`] are held in
/// packages under `<root>/timber-checks/store/`, the component's and the
/// moon build profile's cache's, and found by no store bound for another
/// repository; an artifact's name at the version's index is held in
/// `<root>/artifact/`, where a store bound for `timber-checks/reader`
/// resolves it and reads the index's and the artifact's claims. Each
/// package a write went into is printed. Run with `-- --nocapture` to
/// read them.
#[test]
fn ghcr_keeps_each_repositorys_packages_under_its_own_root() {
    let root = repository();
    let prefix = run_prefix();
    let store = live(&root).bound_for("moon", false);
    let component = format!("{prefix}-lib");
    let meta = timber_seams::ArtifactMeta::new(
        timber_seams::Ecosystem::new("file"),
        timber_seams::EcosystemName::new(format!("{prefix}-lib")),
        timber_seams::Version::new("1.0.0"),
    )
    .unwrap();
    let layer = store
        .put_bytes(format!("{prefix} the lib").as_bytes())
        .unwrap()
        .digest;
    let artifact = store
        .put_artifact(&Manifest::Artifact {
            meta: meta.clone(),
            layers: vec![layer],
        })
        .unwrap()
        .digest;
    store
        .attach(&artifact, &claim("it was built here"))
        .unwrap();
    let index = timber_seams::IndexMeta::new(
        timber_seams::ComponentName::new(&component),
        meta.version().clone(),
        vec![timber_seams::Artifact {
            digest: artifact.clone(),
            meta: meta.clone(),
        }],
    )
    .unwrap();
    let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
    store.attach(&index, &claim("its notes")).unwrap();
    let version = Name::new(format!("{component}:{component}-v1.0.0"));
    store.name(&version, &index, None).unwrap();
    let bytes = store
        .put_bytes(format!("{prefix} a cache entry").as_bytes())
        .unwrap()
        .digest;
    let entry = Name::new(format!("cache/{prefix}-b7e1"));
    store.name(&entry, &bytes, None).unwrap();
    let name = timber_seams::artifact_name(meta.ecosystem(), meta.name(), meta.version());
    store.name(&name, &index, None).unwrap();
    for (made, at) in [(&version, &index), (&entry, &bytes), (&name, &index)] {
        assert_eq!(store.resolve(made).as_ref(), Ok(at), "{made}");
    }

    let (_, path) = root.split_once('/').unwrap();
    let own = format!("{path}/{}/{}", LIVE.0, LIVE.1);
    let places = [
        format!("{own}/{component}"),
        format!("{own}/cache/moon"),
        format!("{path}/artifact/file/{prefix}-lib"),
    ];
    let written: std::collections::BTreeSet<String> =
        store.mounts().into_iter().map(|(r, _)| r).collect();
    println!("ghcr: the places run wrote into {written:?}");
    for place in &places {
        assert!(
            written.contains(place),
            "nothing was written into {GHCR}/{place}: {written:?}"
        );
    }
    assert!(
        written.iter().all(|r| places.contains(r)),
        "a package outside C.1's places was written: {written:?}"
    );

    let reader = store_under(&root, LIVE.0, "reader").bound_for("moon", false);
    // A package that does not exist may be answered absent or refused,
    // as GitHub's registry chooses; either way it is not found.
    for unseen in [&version, &entry] {
        let answered = reader.resolve(unseen);
        println!("ghcr: a store of timber-checks/reader resolving {unseen} answered {answered:?}");
        assert!(
            answered.is_err(),
            "a store of timber-checks/reader found {unseen}, which is under {own}"
        );
    }
    assert_eq!(reader.resolve(&name).as_ref(), Ok(&index));
    assert!(matches!(
        reader.read(&index),
        Ok(Held::Manifest(Manifest::Index(_)))
    ));
    for subject in [&index, &artifact] {
        let claims = reader.claims_of(subject).unwrap();
        assert_eq!(claims.len(), 1, "{subject}: {claims:?}");
        assert_eq!(Ok(claims), store.claims_of(subject), "{subject}");
    }
}

/// Whether GitHub's registry answers the manifest `digest` of
/// `repository` to a read with no credential, and its words: the read
/// sent with no `authorization`, a challenge followed to its realm with
/// none, and the manifest asked again with whatever token the realm
/// answers. Any answer but the manifest's is the registry's, printed as
/// it gave it.
fn anonymous(repository: &str, digest: &Digest) -> (bool, String) {
    let config = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let url = format!("https://{GHCR}/v2/{repository}/manifests/{digest}");
    let read = |authorization: Option<&str>| {
        let mut request = agent.get(&url).header("accept", MANIFESTS);
        if let Some(authorization) = authorization {
            request = request.header("authorization", authorization);
        }
        let mut answered = request
            .call()
            .unwrap_or_else(|e| panic!("{url} could not be read: {e}"));
        let challenge = answered
            .headers()
            .get("www-authenticate")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let body = answered.body_mut().read_to_string().unwrap_or_default();
        (answered.status().as_u16(), challenge, body)
    };

    let (status, challenge, body) = read(None);
    if status == 200 {
        return (true, format!("200 with no token: {body}"));
    }
    let Some(challenge) = challenge.filter(|_| status == 401) else {
        return (false, format!("{status} with no token: {body}"));
    };
    let param = |key: &str| {
        let start = challenge.find(&format!("{key}=\""))? + key.len() + 2;
        let end = challenge[start..].find('"')? + start;
        Some(challenge[start..end].to_string())
    };
    let Some(realm) = param("realm") else {
        return (false, format!("401 challenged with no realm: {challenge}"));
    };
    let mut asked = agent.get(&realm);
    for key in ["service", "scope"] {
        if let Some(value) = param(key) {
            asked = asked.query(key, value);
        }
    }
    let mut token = asked
        .call()
        .unwrap_or_else(|e| panic!("{realm} could not be read: {e}"));
    let token_status = token.status().as_u16();
    let token_body = token.body_mut().read_to_string().unwrap_or_default();
    if token_status != 200 {
        return (
            false,
            format!(
                "401 challenged {challenge}; the realm answered {token_status} with no credential: \
                 {token_body}"
            ),
        );
    }
    let answered: serde_json::Value = serde_json::from_str(&token_body).unwrap_or_default();
    let Some(token) = answered["token"]
        .as_str()
        .or(answered["access_token"].as_str())
        .filter(|t| !t.is_empty())
    else {
        return (
            false,
            format!("401 challenged {challenge}; the realm answered no token: {token_body}"),
        );
    };
    let (status, _, body) = read(Some(&format!("Bearer {token}")));
    (
        status == 200,
        format!(
            "401 challenged {challenge}; the realm answered an anonymous token; the read with it \
             answered {status}: {body}"
        ),
    )
}

/// C.1 on GitHub's registry (the change
/// a-moved-repository-keeps-its-history-in-the-store's design D8): a
/// moved repository's history carried from `timber-checks/carried-from`
/// to `timber-checks/carried-to` under the root the host names, every
/// version and channel under the run's prefix, as the tools layer
/// carries it on zot and the reference distribution.
#[test]
fn ghcr_carries_a_moved_repositorys_history_to_its_new_root() {
    registry::carried_between(
        &repository(),
        ("timber-checks", "carried-from"),
        ("timber-checks", "carried-to"),
        &format!("{}-", run_prefix()),
    );
}
