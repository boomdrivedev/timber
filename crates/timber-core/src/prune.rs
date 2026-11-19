//! `timber prune`: a repository's cache entries removed once the keeping
//! time its policy gives at a landed revision has passed since their push
//! (145; design D7, D8). The landing is asked first, and a revision its
//! branch has not landed is refused before anything else is asked (14);
//! then the keeping time, read from the policy at that revision, and
//! nothing is removed while it is not known; then the store's cache
//! entries, each answered on its own: an entry the store could not read
//! carried as answered and never removed, an entry with no push moment
//! removed, since it was pushed before moments were written, an entry
//! whose time has passed removed at the digest it was listed at, and any
//! other kept, naming when its time passes. One removal never stops the
//! next, and a removal answered absent is gone already, which another
//! prune or the store's own retention did (54).
//!
//! Prune asks the landing, the policy and the store's two cache
//! operations alone: no keeper, signer, projection or target, and no
//! other operation of the store. It removes cache entries and nothing
//! else, the store seam having no removal that names another name, so
//! anything accepted or released stays (54), and it reads and writes no
//! git ref, so the record of claimed versions (22) and the ledger (34)
//! are out of its reach. It signs nothing and keeps nothing (61). What
//! runs it is the operator's: timber starts nothing (rule core.52).

use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use timber_seams::{Answer, Branch, BuildId, Keeping, Not, Revision, Seams};

use crate::{answer_json, Landed};

/// The seconds of a day of the keeping time.
const DAY: u64 = 86_400;

/// What a prune is asked: the landed revision whose policy gives the
/// keeping time, the branch that landed it, and the moment it runs, in
/// seconds since the Unix epoch, which the command reads from the host's
/// clock.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PruneAsk {
    pub branch: Branch,
    pub revision: Revision,
    pub now: u64,
}

/// 145: what became of one cache entry the store listed, each moment in
/// seconds since the Unix epoch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "fate", rename_all = "snake_case")]
pub enum Fate {
    /// Removed: pushed at `pushed`, or, where none, before push moments
    /// were written.
    Removed { pushed: Option<u64> },
    /// Listed, and gone by the time it was removed (54).
    GoneAlready { pushed: Option<u64> },
    /// Kept: pushed at `pushed`, its time passing at `until`.
    Kept { pushed: u64, until: u64 },
}

/// 57: what a prune answers: the branch, the landed revision and the
/// keeping time read there, and each entry the store listed, in its
/// order, with its own answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pruned {
    pub branch: Branch,
    pub revision: Revision,
    pub keeping: Keeping,
    pub entries: Vec<(BuildId, Answer<Fate>)>,
}

/// Each entry's answer as a program reads it, the build identity beside
/// it.
fn entries_json(entries: &[(BuildId, Answer<Fate>)]) -> Vec<Value> {
    entries
        .iter()
        .map(|(build, answer)| {
            let mut value = answer_json(answer);
            value["build"] = json!(build);
            value
        })
        .collect()
}

impl Serialize for Pruned {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        json!({
            "branch": self.branch,
            "revision": self.revision,
            "keeping": self.keeping,
            "entries": entries_json(&self.entries),
        })
        .serialize(serializer)
    }
}

/// 145: each cache entry the store holds removed once `keeping`'s days
/// have passed since its push by `ask.now`, at or after the moment its
/// time passes, as the module says; refused naming the branch and the
/// revision where the branch has not landed it, and the policy's or the
/// store's own answer where the keeping time or the listing cannot be
/// had, nothing removed.
pub fn prune(seams: &Seams, ask: &PruneAsk) -> Answer<Pruned> {
    let landed = Landed::check(seams.landing.as_ref(), &ask.branch, &ask.revision)?;
    let keeping = seams.policy.cache_kept(landed.revision())?;
    let kept_for = u64::from(keeping.days) * DAY;
    let store = seams.store.as_ref();
    let entries = store
        .cache_entries()?
        .into_iter()
        .map(|(build, listed)| {
            let fate = listed.and_then(|pushed| {
                if let Some(at) = pushed.at {
                    let until = at.saturating_add(kept_for);
                    if until > ask.now {
                        return Ok(Fate::Kept { pushed: at, until });
                    }
                }
                match store.remove_cache_entry(&build, &pushed.digest) {
                    Ok(()) => Ok(Fate::Removed { pushed: pushed.at }),
                    Err(Not::Absent(_)) => Ok(Fate::GoneAlready { pushed: pushed.at }),
                    Err(not) => Err(not),
                }
            });
            (build, fate)
        })
        .collect();
    Ok(Pruned {
        branch: ask.branch.clone(),
        revision: landed.revision().clone(),
        keeping,
        entries,
    })
}
