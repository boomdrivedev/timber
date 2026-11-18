//! C.1: the mark a root the repository left holds, naming the root its
//! history moved to: one name of the repository's own spaces, at bytes
//! saying where (statement C.1 as amended at `boomdrivedev/blueprints`
//! e8700c8; record 0040 as amended on 2026-11-18; design D4).

use serde::{Deserialize, Serialize};

use crate::{canonical, Name, RepositoryName};

/// What the mark's name is: no version's name, which is a prefix and a
/// version; no channel's, under `channel/`; no cache entry's, under
/// `cache/`; and no artifact's, under `artifact/` (rule core.48).
const MOVED: &str = "moved";

/// C.1: the name of the mark in a root the repository left, `moved`, the
/// one function that makes it. No version's name can be it, since a
/// version's is a component's prefix then a semantic version, and none of
/// a channel's, a cache entry's or an artifact's, each under its own
/// space, `channel/`, `cache/` or `artifact/` (rule core.48).
pub fn moved_name() -> Name {
    Name::new(MOVED)
}

/// C.1: what the mark says: the root the repository's history moved to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovedTo {
    pub to: RepositoryName,
}

/// The mark's bytes as written: the root moved to, `<owner>/<name>`.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Mark {
    moved_to: String,
}

impl MovedTo {
    /// The mark's bytes, the canonical JSON `{"moved_to":"<owner>/<name>"}`
    /// of the root moved to, lowercased, as a store spells a root.
    pub fn to_bytes(&self) -> Vec<u8> {
        canonical(&Mark {
            moved_to: self.to.lowercased().to_string(),
        })
    }

    /// The mark `bytes` hold, or `None` where they are no mark: not that
    /// JSON, or a root that is not an owner and a name.
    pub fn of_bytes(bytes: &[u8]) -> Option<MovedTo> {
        let mark: Mark = serde_json::from_slice(bytes).ok()?;
        let (owner, name) = mark.moved_to.split_once('/')?;
        if owner.is_empty() || name.is_empty() || name.contains('/') {
            return None;
        }
        Some(MovedTo {
            to: RepositoryName {
                owner: owner.into(),
                name: name.into(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_marks_bytes_name_the_root_lowercased_and_read_back() {
        let mark = MovedTo {
            to: RepositoryName {
                owner: "Acme".into(),
                name: "Site".into(),
            },
        };
        let bytes = mark.to_bytes();
        assert_eq!(bytes, br#"{"moved_to":"acme/site"}"#);
        assert_eq!(
            MovedTo::of_bytes(&bytes),
            Some(MovedTo {
                to: RepositoryName {
                    owner: "acme".into(),
                    name: "site".into(),
                },
            })
        );
        assert_eq!(moved_name(), Name::new("moved"));
    }

    #[test]
    fn bytes_that_are_no_mark_read_as_none() {
        for bytes in [
            &b""[..],
            b"moved",
            b"{}",
            br#"{"moved_to":"acme"}"#,
            br#"{"moved_to":"/site"}"#,
            br#"{"moved_to":"acme/"}"#,
            br#"{"moved_to":"acme/site/web"}"#,
            br#"{"moved_to":7}"#,
            br#"{"moved_to":"acme/site","by":"x"}"#,
        ] {
            assert_eq!(
                MovedTo::of_bytes(bytes),
                None,
                "{}",
                String::from_utf8_lossy(bytes)
            );
        }
    }
}
