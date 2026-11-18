//! The fake roots (B.9, C.1): the root the binding's store is bound for,
//! none until a test binds one, and per root the repository left a fake
//! store and a fake keeper the test binds, handed out as clones so the
//! test's handles see every asking (design D2, D8).

use std::collections::BTreeMap;

use serde_json::json;
use timber_seams::{Answer, Not, RepositoryName, RootLeft, Roots, Seam, Source};

use crate::{Fake, FakeKeeper, FakeStore, PROFILE};

/// The fake roots' operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootsOp {
    Bound,
    Left,
}

/// The root bound, and each root left's store and keeper by the root
/// lowercased, as two roots differing only in case are one (C.1).
#[derive(Clone, Default)]
pub struct RootsState {
    bound: Option<RepositoryName>,
    left: BTreeMap<RepositoryName, (FakeStore, FakeKeeper)>,
}

/// Roots in memory, answering as the host's bindings (92).
pub type FakeRoots = Fake<RootsState, RootsOp>;

impl Default for FakeRoots {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeRoots {
    /// No root bound and no root left bound.
    pub fn new() -> Self {
        Fake::with(Seam::Host, RootsState::default())
    }

    /// `root` answered as the root the store is bound for.
    pub fn bind(&self, root: &RepositoryName) {
        self.state().bound = Some(root.clone());
    }

    /// `from`, a root the repository left, bound with `store` and
    /// `keeper`, whose clones the core is handed.
    pub fn leave(&self, from: &RepositoryName, store: &FakeStore, keeper: &FakeKeeper) {
        self.state()
            .left
            .insert(from.lowercased(), (store.clone(), keeper.clone()));
    }

    /// The source a root left's store is named by, as a copy's answers
    /// name it.
    pub fn source_of(from: &RepositoryName) -> Source {
        Source::new(Seam::Store, format!("{PROFILE} {}", from.lowercased()))
    }
}

impl Roots for FakeRoots {
    fn bound(&self) -> Answer<Option<RepositoryName>> {
        self.ask(RootsOp::Bound, json!({}), "the root bound", |s, _| {
            Ok(s.bound.clone())
        })
    }

    fn left(&self, from: &RepositoryName) -> Answer<RootLeft> {
        let mut found = None;
        self.ask(
            RootsOp::Left,
            json!({ "from": from }),
            format!("root left {from}"),
            |s, source| match s.left.get(&from.lowercased()) {
                Some(pair) => {
                    found = Some(pair.clone());
                    Ok(from.clone())
                }
                None => Err(Not::absent(source, format!("root left {from}"))),
            },
        )?;
        let (store, keeper) = found.expect("a root left answered is one bound");
        Ok(RootLeft {
            store: Box::new(store),
            keeper: Box::new(keeper),
            source: FakeRoots::source_of(from),
        })
    }
}

#[cfg(test)]
mod tests {
    use timber_seams::{Name, Store};

    use super::*;

    fn root(owner: &str, name: &str) -> RepositoryName {
        RepositoryName {
            owner: owner.into(),
            name: name.into(),
        }
    }

    #[test]
    fn fake_roots_answer_the_root_a_test_binds() {
        let roots = FakeRoots::new();
        assert_eq!(roots.bound(), Ok(None));
        roots.bind(&root("acme", "site"));
        let bound: Box<dyn Roots> = Box::new(roots.clone());
        assert_eq!(bound.bound(), Ok(Some(root("acme", "site"))));
        let ops: Vec<RootsOp> = roots.calls().iter().map(|c| c.op).collect();
        assert_eq!(ops, [RootsOp::Bound, RootsOp::Bound]);
    }

    #[test]
    fn a_root_left_is_read_through_the_store_the_test_holds() {
        let roots = FakeRoots::new();
        roots.bind(&root("acme", "site"));
        let (store, keeper) = (FakeStore::new(), FakeKeeper::new());
        let index = store.put_bytes(b"v1.2.0's index").unwrap().digest;
        store.name(&Name::new("v1.2.0"), &index, None).unwrap();
        roots.leave(&root("acme", "app"), &store, &keeper);

        let bound: Box<dyn Roots> = Box::new(roots.clone());
        let left = bound.left(&root("Acme", "App")).unwrap();
        assert_eq!(left.store.resolve(&Name::new("v1.2.0")), Ok(index));
        assert_eq!(left.source, FakeRoots::source_of(&root("acme", "app")));
        assert_eq!(left.source.seam, Seam::Store);
        assert_eq!(store.calls().len(), 3, "the test's handle saw the asking");
        let ops: Vec<RootsOp> = roots.calls().iter().map(|c| c.op).collect();
        assert_eq!(ops, [RootsOp::Left]);
        assert_eq!(roots.calls()[0].given["from"]["name"], "App");
    }

    #[test]
    fn a_root_left_never_bound_is_absent() {
        let roots = FakeRoots::new();
        roots.leave(&root("acme", "app"), &FakeStore::new(), &FakeKeeper::new());
        match roots.left(&root("acme", "web")) {
            Err(Not::Absent(a)) => {
                assert_eq!(a.source, Source::new(Seam::Host, PROFILE));
                assert_eq!(a.what, "root left acme/web");
            }
            Err(not) => panic!("a root never bound answered {not:?}"),
            Ok(_) => panic!("a root never bound was bound"),
        }
        assert!(roots.calls()[0].answer.is_err());
    }
}
