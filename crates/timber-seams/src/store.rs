//! B.1 The store: bytes held by their digest, attestations beside them,
//! names moved only from where they are expected, the versions a name of
//! an ecosystem is held at, and the cache entries it holds, listed with
//! when each was pushed and removed one by one (66 to 73, 138, 145).

use serde::{Deserialize, Serialize};

use crate::{
    Answer, BuildId, Digest, Ecosystem, EcosystemName, Envelope, Held, Manifest, Name, Version,
};

/// What a put answers: the digest the store holds what it was given
/// under, and whether it held it already (5, 66).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Put {
    pub digest: Digest,
    pub held_already: bool,
}

/// 145: a cache entry the store holds: the digest its name is at, and
/// when it was pushed, in seconds since the Unix epoch; `None` where
/// the store holds no moment for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pushed {
    pub digest: Digest,
    pub at: Option<u64>,
}

/// 73: a store holds bytes by digest, never alters them, and answers the
/// same bytes for the same digest for as long as it holds them.
pub trait Store {
    /// 66: `bytes` held under their digest. The same bytes put twice are
    /// held once, and the second put answers that they were held already
    /// (5).
    fn put_bytes(&self, bytes: &[u8]) -> Answer<Put>;

    /// 67: an artifact's manifest, or an index's, held under the digest
    /// the store names it by thereafter (design D4).
    fn put_artifact(&self, manifest: &Manifest) -> Answer<Put>;

    /// 68: `attestation` held beside `subject`, readable by
    /// [`Store::claims_of`], answering the attestation's own digest;
    /// absent when the subject is not held, since an attestation is
    /// never kept where its digest is not (8).
    fn attach(&self, subject: &Digest, attestation: &Envelope) -> Answer<Put>;

    /// 69: every attestation attached to `digest`, as attached; absent
    /// when the digest is not held.
    fn claims_of(&self, digest: &Digest) -> Answer<Vec<Envelope>>;

    /// 70: `name` moved to `to` from `expected`, the digest it is
    /// expected to be at, or from nothing, which makes it; refused
    /// [`Why::NameAt`](crate::Why::NameAt) naming where it is otherwise,
    /// and the name does not move.
    fn name(&self, name: &Name, to: &Digest, expected: Option<&Digest>) -> Answer<()>;

    /// 71: the digest `name` names; absent when it names nothing.
    fn resolve(&self, name: &Name) -> Answer<Digest>;

    /// 72: what `digest` holds, its bytes or its manifest (design D4);
    /// absent when the digest is not held.
    fn read(&self, digest: &Digest) -> Answer<Held>;

    /// 138: the versions the store names `name` of `ecosystem` at, every
    /// [`artifact_name`](crate::artifact_name) of the two it holds, in no
    /// order the core relies on (23); the empty list where it holds none,
    /// and unreadable only where the listing cannot be read, never an
    /// empty list for a read that failed (rule core.19).
    fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>>;

    /// 145: every cache entry the store holds for the repository and the
    /// build profile it is bound for, each by its build identity, in no
    /// order the core relies on; an entry the store cannot read answered
    /// on its own; the empty list where it holds none, and unreadable only
    /// where the listing cannot be read (rule core.19).
    fn cache_entries(&self) -> Answer<Vec<(BuildId, Answer<Pushed>)>>;

    /// 145, 54: the cache entry of `build` removed, only where its name is
    /// at `at`; refused [`Why::NameAt`](crate::Why::NameAt) naming where
    /// it is otherwise, and nothing removed; absent where the store names
    /// no entry for it. No other name, and no bytes another name reaches,
    /// are removed (100).
    fn remove_cache_entry(&self, build: &BuildId, at: &Digest) -> Answer<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::round_trip;

    #[test]
    fn a_put_round_trips_through_json() {
        round_trip(Put {
            digest: Digest::of(b"bytes"),
            held_already: true,
        });
    }

    #[test]
    fn a_pushed_round_trips_through_json() {
        round_trip(Pushed {
            digest: Digest::of(b"bytes"),
            at: Some(1_793_000_000),
        });
        round_trip(Pushed {
            digest: Digest::of(b"bytes"),
            at: None,
        });
    }
}
