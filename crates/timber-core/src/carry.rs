//! A moved repository's history carried from the root it left to the
//! root its store is bound for (C.1 as amended at `boomdrivedev/blueprints`
//! e8700c8; record 0040 as amended on 2026-11-18; design D4, D5): the root
//! left marked with where it went before anything else, each version the
//! record claims copied with every claim on its digests (103), and each
//! channel's name made at its chain's head so the chain goes on from it
//! (105). Each step writes only what the bound root lacks, so a carry
//! again writes nothing, and a carry that stopped partway is carried
//! again by the next (rule core.6). No cache entry is carried, and the
//! store and the ledger are reached through their seams alone (rules
//! core.1, core.2).

use serde::Serialize;
use timber_seams::{
    channel_name, moved_name, Answer, ChannelId, Digest, Envelope, Held, Keeper, MovedTo, Name,
    Not, RepositoryName, Source, Store, Why,
};

use crate::repair::{from_source, head_of};
use crate::{copy, End};

/// One root of a carry: the repository whose root it is, its store with
/// the source a copy's answers name it by, and the keeper over it.
#[derive(Clone, Copy)]
pub struct Root<'a> {
    pub repository: &'a RepositoryName,
    pub store: &'a dyn Store,
    pub source: &'a Source,
    pub keeper: &'a dyn Keeper,
}

impl<'a> Root<'a> {
    fn end(&self) -> End<'a> {
        End {
            store: self.store,
            source: self.source,
        }
    }
}

/// 57: what a carry answers: the root left and the root carried to, the
/// mark, each version by its name and each channel with its head.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Carried {
    pub from: RepositoryName,
    pub to: RepositoryName,
    pub mark: Marked,
    pub versions: Vec<CarriedVersion>,
    pub channels: Vec<CarriedChannel>,
}

/// Whether the carry wrote the root left's mark or found it there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Marked {
    Made,
    HeldAlready,
}

/// A version's name and what the carry found of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CarriedVersion {
    pub name: Name,
    pub carried: VersionCarried,
}

/// 103: a version copied into the bound root, held there already whole,
/// or not held at the root left and passed over.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionCarried {
    Copied,
    HeldAlready,
    NotHeld,
}

/// A channel the carry found a chain of: the claim at its head at the
/// root left, the index that claim is about, and what became of its name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CarriedChannel {
    pub channel: ChannelId,
    pub head: Digest,
    pub index: Digest,
    pub name: ChannelCarried,
}

/// 105: the channel's name made at its head's index in the bound root,
/// there already, or moved on past the head left by a promotion after an
/// earlier carry and left where it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelCarried {
    Made,
    HeldAlready,
    MovedOn,
}

/// C.1, 103, 105, design D5: the history at `from`, the root left, carried
/// to `to`, the root bound: `versions`, the names the record claims, and
/// `channels`, each the policy answers, in order. The mark is written into
/// the root left first, `moved` from nothing at bytes naming `to`; one
/// naming another root refuses the carry naming it. Each version is then
/// copied, one the root left does not hold passed over; and each
/// channel's chain read from the root left's keeper, as its source holds
/// it, its name made in the bound store at its head's index, and the
/// bound keeper's chain checked to hold that head. It reads no git and no
/// policy, and nothing of the cache.
pub fn history(from: Root, to: Root, versions: &[Name], channels: &[ChannelId]) -> Answer<Carried> {
    let mark = mark(from, to)?;

    let mut carried_versions = Vec::new();
    for name in versions {
        let carried = match from.store.resolve(name) {
            Err(Not::Absent(_)) => VersionCarried::NotHeld,
            Err(not) => return Err(not),
            Ok(_) => {
                let copied = copy(from.end(), to.end(), name)?;
                if copied.name_held_already && copied.written.iter().all(|w| w.held_already) {
                    VersionCarried::HeldAlready
                } else {
                    VersionCarried::Copied
                }
            }
        };
        carried_versions.push(CarriedVersion {
            name: name.clone(),
            carried,
        });
    }

    let mut carried_channels = Vec::new();
    for channel in channels {
        if let Some(carried) = carry_channel(from, to, channel)? {
            carried_channels.push(carried);
        }
    }

    Ok(Carried {
        from: from.repository.lowercased(),
        to: to.repository.lowercased(),
        mark,
        versions: carried_versions,
        channels: carried_channels,
    })
}

/// D4, step 1: the mark naming `to` put into the root left as bytes and
/// `moved` named at them from nothing; a mark at the same bytes is a carry
/// again, and one at other bytes refused naming the root it names, or
/// unreadable naming it where its bytes are no mark.
fn mark(from: Root, to: Root) -> Answer<Marked> {
    let bytes = MovedTo {
        to: to.repository.lowercased(),
    }
    .to_bytes();
    let put = from.store.put_bytes(&bytes)?;
    let at = match from.store.name(&moved_name(), &put.digest, None) {
        Ok(()) => return Ok(Marked::Made),
        Err(Not::Refused(r)) => match &r.why {
            Why::NameAt { at: Some(at) } if *at == put.digest => return Ok(Marked::HeldAlready),
            Why::NameAt { at: Some(at) } => at.clone(),
            _ => return Err(Not::Refused(r)),
        },
        Err(not) => return Err(not),
    };
    let other = match from.store.read(&at)? {
        Held::Bytes(bytes) => MovedTo::of_bytes(&bytes),
        Held::Manifest(_) => None,
    };
    match other {
        Some(other) => Err(Not::refused(
            from.source,
            Why::RootMoved {
                from: from.repository.lowercased(),
                to: other.to,
            },
            None,
        )),
        None => Err(Not::unreadable(
            from.source,
            format!("the mark {} at {at}", moved_name()),
            Some(format!("{at} holds no mark naming a root")),
        )),
    }
}

/// D5, step 3: `channel` carried: its chain read from the root left's
/// keeper, `None` where it holds none; its name made at the head's index
/// in the bound store from nothing; a name elsewhere left where it is
/// where the bound keeper's chain holds the head left, and refused as a
/// fork naming both heads otherwise (105); the bound keeper's chain then
/// read, whose head must be the head left or a claim after it.
fn carry_channel(from: Root, to: Root, channel: &ChannelId) -> Answer<Option<CarriedChannel>> {
    let left = match from_source(from.keeper, channel) {
        Ok(claims) => claims,
        Err(Not::Absent(_)) => return Ok(None),
        Err(not) => return Err(not),
    };
    let (head, index) = match head_of(channel, &left) {
        Ok(head) => head,
        Err(Not::Absent(_)) => return Ok(None),
        Err(not) => return Err(not),
    };
    let head_digest = head.digest();
    let name = channel_name(channel);
    let made = match to.store.name(&name, &index, None) {
        Ok(()) => ChannelCarried::Made,
        Err(Not::Refused(r)) => match &r.why {
            Why::NameAt { at: Some(at) } if *at == index => ChannelCarried::HeldAlready,
            Why::NameAt { at: Some(_) } => ChannelCarried::MovedOn,
            _ => return Err(Not::Refused(r)),
        },
        Err(not) => return Err(not),
    };

    let bound = match from_source(to.keeper, channel) {
        Ok(claims) => claims,
        Err(Not::Absent(_)) => Vec::new(),
        Err(not) => return Err(not),
    };
    if !bound.iter().any(|claim| claim.digest() == head_digest) {
        if made == ChannelCarried::MovedOn {
            return Err(fork(to.source, &left, &bound));
        }
        return Err(Not::unreadable(
            to.source,
            format!("the chain of {channel} at {}", to.repository.lowercased()),
            Some(format!(
                "its head is {}, which does not follow the head left at {}, {head_digest}",
                bound
                    .last()
                    .map_or_else(|| "nothing".to_string(), |c| c.digest().to_string()),
                from.repository.lowercased()
            )),
        ));
    }
    Ok(Some(CarriedChannel {
        channel: channel.clone(),
        head: head_digest,
        index,
        name: made,
    }))
}

/// 105: the chain at the root left and the bound one part: refused as a
/// fork naming both heads, after the last claim they share, none where
/// they share none.
fn fork(source: &Source, left: &[Envelope], bound: &[Envelope]) -> Not {
    let shared = left
        .iter()
        .zip(bound)
        .take_while(|(a, b)| a.digest() == b.digest())
        .count();
    let claims = left
        .last()
        .into_iter()
        .chain(bound.last())
        .map(Envelope::digest)
        .collect();
    Not::refused(
        source,
        Why::Fork {
            predecessor: shared.checked_sub(1).map(|i| left[i].digest()),
            claims,
        },
        None,
    )
}
