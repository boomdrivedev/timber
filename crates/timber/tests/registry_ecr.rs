//! Spec registry-store, its run against ECR: the conformance suite's
//! store group and the cache manifest's scenarios over the repository the
//! host names in `env.TIMBER_TOOLS_ECR` of its `devenv.local.nix`,
//! reached with the host's Docker credentials, ECR's helper among them,
//! every name they pass prefixed by the run so no run's names meet an
//! earlier one's (design D12, D14), through the live rig the run against
//! GitHub's registry shares, in `registry/mod.rs`. It refuses to run,
//! naming the setting, on a host that names no repository, rather than
//! pass without running.

#![cfg(feature = "tools")]

mod registry;

use registry::{cache_entries, run_prefix, store_group_against, Prefixed};
use timber::profiles::store::registry::Registry;
use timber_seams::{Manifest, Store};

/// The setting a host names its ECR repository by.
const SETTING: &str = "TIMBER_TOOLS_ECR";

/// The repository the host names; a panic naming the setting where it
/// names none.
fn repository() -> String {
    match std::env::var(SETTING) {
        Ok(repository) if !repository.trim().is_empty() => repository.trim().to_string(),
        _ => panic!(
            "the ECR run names no repository: set env.{SETTING} in devenv.local.nix to the ECR \
             repository the ECR check pushes to, with the AWS_PROFILE its credential helper \
             reads and the registry host mapped to ecr-login in the Docker configuration's \
             credHelpers"
        ),
    }
}

#[test]
fn ecr_keeps_the_stores_contract() {
    store_group_against(&repository());
}

/// What the run records in ADR 0023 (design D8, D13): an artifact's name
/// made at an index in the repository the templates make for it, with
/// whether ECR took each cross-repository mount, and a put into a
/// repository no template covers, a sibling of the host's, refused in
/// ECR's words naming it. Run with `-- --nocapture` to read both.
#[test]
fn ecr_mounts_into_an_artifacts_repository_and_refuses_one_nobody_made() {
    let repository = repository();
    let prefix = run_prefix();
    let registry = Registry::new(&repository).expect("the repository the host names");
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
        .expect("the artifact name's repository, which a template covers");
    assert_eq!(registry.resolve(&name), Ok(index));
    let mounts = registry.mounts();
    let taken = mounts.iter().filter(|(_, taken)| *taken).count();
    println!(
        "ecr: {taken} of {} cross-repository mounts were taken, into {:?}",
        mounts.len(),
        mounts
            .iter()
            .map(|(r, _)| r)
            .collect::<std::collections::BTreeSet<_>>()
    );

    let (host, path) = repository.split_once('/').unwrap();
    let uncovered = format!("{host}/{path}-{prefix}-uncovered");
    let nobodys = Registry::new(&uncovered).unwrap();
    match nobodys.put_bytes(b"bytes for a repository nobody made") {
        Err(timber_seams::Not::Refused(r)) => {
            let said = r.said.unwrap_or_default();
            println!("ecr: a repository no template covers answered: {said}");
            assert!(
                said.contains(&uncovered),
                "the refusal names no repository: {said}"
            );
        }
        other => panic!(
            "{uncovered} was written: a template covers it, and none may for this run: {other:?}"
        ),
    }
}

#[test]
fn ecr_names_a_cache_entry_through_a_cache_manifest() {
    let repository = repository();
    let registry = Registry::new(&repository)
        .expect("the repository the host names")
        .bound_for("moon", false);
    let within = registry.bound();
    cache_entries(
        &Prefixed {
            registry,
            prefix: format!("{}-cache-", run_prefix()),
        },
        within,
    );
}
