//! B.9 Binding the contracts: the one binding the core is handed, and
//! reaches every seam through (92, rule core.2; design D8), with the
//! roots its store is bound for (C.1).

use std::collections::BTreeMap;

use crate::{
    Answer, Build, Ecosystem, Keeper, Landing, Not, Policy, Projection, RepositoryName, Seam,
    Signer, Source, Store, Target, TargetProfile,
};

/// 92: exactly one store written, one build asked, one landing read, one
/// policy and one keeper appended to; the signers bound for the host; and
/// any number of projections, by ecosystem, and targets, by the profile
/// each implements, which an environment's target names (C.10). How the
/// host's bindings file fills it (118) is a profile's.
pub struct Seams {
    pub store: Box<dyn Store>,
    pub build: Box<dyn Build>,
    pub landing: Box<dyn Landing>,
    pub policy: Box<dyn Policy>,
    pub keeper: Box<dyn Keeper>,
    pub signers: Vec<Box<dyn Signer>>,
    pub projections: BTreeMap<Ecosystem, Box<dyn Projection>>,
    pub targets: BTreeMap<TargetProfile, Box<dyn Target>>,
    /// 92, C.1: the root the one store written is bound for, and the store
    /// and keeper of a root the repository left, which are that store's
    /// and that keeper's over another repository of it (design D2).
    pub roots: Box<dyn Roots>,
}

impl Seams {
    /// The projection bound for `ecosystem`, or `None` when none is.
    pub fn projection(&self, ecosystem: &Ecosystem) -> Option<&dyn Projection> {
        self.projections.get(ecosystem).map(|p| p.as_ref())
    }

    /// The target bound for `profile`, or `None` when none is.
    pub fn target(&self, profile: &TargetProfile) -> Option<&dyn Target> {
        self.targets.get(profile).map(|t| t.as_ref())
    }
}

/// C.1: the roots of the binding's one store: the repository it is bound
/// for, whose root holds the repository's own spaces, and, asked for a
/// root the repository left, the store and keeper bound there (design
/// D2).
pub trait Roots {
    /// C.1: the repository the store is bound for, whose root holds its
    /// spaces; none where the store places no name by repository.
    fn bound(&self) -> Answer<Option<RepositoryName>>;
    /// C.1: the store and the ledger bound for `from`, a root the
    /// repository left for the bound one, reading past the mark naming
    /// the bound one.
    fn left(&self, from: &RepositoryName) -> Answer<RootLeft>;
}

/// C.1: a root the repository left, bound: its store, which reads past the
/// mark naming the root bound, the keeper over it, and the store's source
/// as a copy's answers name it, as `copy::End` takes it (design D2). It is
/// no build's `Left`, which names what a checkout's run left (139).
pub struct RootLeft {
    pub store: Box<dyn Store>,
    pub keeper: Box<dyn Keeper>,
    pub source: Source,
}

/// The roots of a binding whose store places no name by repository, as a
/// layout does, and of every binding in a test that moves no repository:
/// no root bound, and every root left absent naming the root asked.
pub struct NoRoots;

impl Roots for NoRoots {
    fn bound(&self) -> Answer<Option<RepositoryName>> {
        Ok(None)
    }

    fn left(&self, from: &RepositoryName) -> Answer<RootLeft> {
        Err(Not::absent(
            &Source::new(Seam::Host, "no roots"),
            format!("root left {from}"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_roots_answers_no_root_and_no_root_left() {
        let roots: Box<dyn Roots> = Box::new(NoRoots);
        assert_eq!(roots.bound(), Ok(None));
        let app = RepositoryName {
            owner: "acme".into(),
            name: "app".into(),
        };
        match roots.left(&app) {
            Err(Not::Absent(a)) => {
                assert_eq!(a.source.seam, Seam::Host);
                assert_eq!(a.what, "root left acme/app");
            }
            Err(not) => panic!("no roots answered {not:?}"),
            Ok(_) => panic!("no roots bound a root left"),
        }
    }
}
