//! A GitHub that answers as a test sets, in `gh`'s place, so the
//! profile's refusals, its order of steps, current, report and the target
//! group run in `check-fast` (design D11). It holds the repository's
//! tags, its releases with their assets and digests, and which release is
//! latest, in memory. Any operation fails with words a test gives, and
//! every operation handed a token a test names fails with that token's
//! words; every operation asked is recorded in order, with the token it
//! was handed.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use timber_seams::{Answer, Digest, Not, Revision, Source};

use super::gh::{Asset, At, Backend, Latest, Release, Upload};

/// When GitHub says each release it publishes was last updated.
pub(super) const CLOCK: &str = "2026-11-18T20:22:59Z";

/// An operation asked, with what it was asked of.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Asked {
    /// A read of this tag's commit.
    Tag(String),
    /// A read of this tag's release.
    Release(String),
    /// A draft of this tag made, holding these notes.
    Create { tag: String, notes: String },
    /// These assets, by name, uploaded to this tag's release.
    Upload { tag: String, assets: Vec<String> },
    /// This tag's release published, holding these notes.
    Publish { tag: String, notes: String },
    /// A read of the release marked latest.
    Latest,
}

impl Asked {
    /// The operation's name, as a test lists what ran.
    pub fn name(&self) -> &'static str {
        match self {
            Asked::Tag(_) => "tag",
            Asked::Release(_) => "release",
            Asked::Create { .. } => "create",
            Asked::Upload { .. } => "upload",
            Asked::Publish { .. } => "publish",
            Asked::Latest => "latest",
        }
    }
}

/// A release as the scripted GitHub holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Held {
    pub draft: bool,
    pub prerelease: bool,
    pub title: String,
    pub notes: String,
    pub assets: BTreeMap<String, Asset>,
    pub updated_at: Option<String>,
}

impl Held {
    /// A release of `tag` holding `notes` and no asset, published or a
    /// draft.
    pub fn of(tag: &str, notes: &str, draft: bool) -> Held {
        Held {
            draft,
            prerelease: false,
            title: tag.into(),
            notes: notes.into(),
            assets: BTreeMap::new(),
            updated_at: (!draft).then(|| "2026-11-01T00:00:00Z".into()),
        }
    }

    /// The release as GitHub says it was last updated at `at`.
    pub fn updated(mut self, at: &str) -> Held {
        self.updated_at = Some(at.into());
        self
    }

    /// The release holding `bytes` uploaded as the asset `name`.
    pub fn with(mut self, name: &str, bytes: &[u8]) -> Held {
        self.assets.insert(
            name.into(),
            Asset {
                name: name.into(),
                state: "uploaded".into(),
                digest: Some(Digest::of(bytes)),
            },
        );
        self
    }
}

#[derive(Default)]
struct State {
    tags: BTreeMap<String, Revision>,
    releases: BTreeMap<String, Held>,
    latest: Option<String>,
    fails: BTreeMap<&'static str, String>,
    refused: BTreeMap<String, String>,
    asked: Vec<(Asked, String)>,
}

/// The scripted GitHub; a clone shares its state.
#[derive(Clone)]
pub(super) struct Scripted {
    state: Rc<RefCell<State>>,
    source: Source,
}

impl Scripted {
    pub fn new(source: &Source) -> Scripted {
        Scripted {
            state: Rc::default(),
            source: source.clone(),
        }
    }

    /// GitHub holds `tag` at `revision`, as a publish pushed it.
    pub fn holds_tag(&self, tag: &str, revision: &str) {
        let mut state = self.state.borrow_mut();
        state.tags.insert(tag.into(), Revision::new(revision));
    }

    /// GitHub holds `release` of `tag`, as a hand or an earlier apply left
    /// it.
    pub fn holds_release(&self, tag: &str, release: Held) {
        self.state.borrow_mut().releases.insert(tag.into(), release);
    }

    /// GitHub marks `tag`'s release latest.
    pub fn marks_latest(&self, tag: &str) {
        self.state.borrow_mut().latest = Some(tag.into());
    }

    /// `tag`'s release as GitHub holds it, where it holds one.
    pub fn release_of(&self, tag: &str) -> Option<Held> {
        self.state.borrow().releases.get(tag).cloned()
    }

    /// The tag of the release GitHub marks latest, where it marks one.
    pub fn latest_tag(&self) -> Option<String> {
        self.state.borrow().latest.clone()
    }

    /// Every asking of `operation` fails, `gh` saying `words`.
    pub fn fails(&self, operation: &'static str, words: &str) {
        let mut state = self.state.borrow_mut();
        state.fails.insert(operation, words.into());
    }

    /// Every operation handed `token` fails, `gh` saying `words`.
    pub fn refuses(&self, token: &str, words: &str) {
        let mut state = self.state.borrow_mut();
        state.refused.insert(token.into(), words.into());
    }

    /// Every operation asked, in order.
    pub fn asked(&self) -> Vec<Asked> {
        let state = self.state.borrow();
        state.asked.iter().map(|(asked, _)| asked.clone()).collect()
    }

    /// The names of every operation asked, in order.
    pub fn ran(&self) -> Vec<&'static str> {
        let state = self.state.borrow();
        state.asked.iter().map(|(asked, _)| asked.name()).collect()
    }

    /// The token each operation asked was handed, in order.
    pub fn tokens(&self) -> Vec<String> {
        let state = self.state.borrow();
        state.asked.iter().map(|(_, token)| token.clone()).collect()
    }

    /// `asked` recorded with the token handed, then the words it fails
    /// with, where it fails: for its token, or as the test set it to.
    fn ask(&self, asked: Asked, at: &At<'_>) -> Answer<()> {
        let name = asked.name();
        let mut state = self.state.borrow_mut();
        let token = at.token.secret().to_string();
        state.asked.push((asked, token.clone()));
        let what = format!("gh {name} in github.com/{}", at.repository);
        let words = state.refused.get(&token).or_else(|| state.fails.get(name));
        match words {
            Some(words) => Err(Not::unreadable(&self.source, what, Some(words.clone()))),
            None => Ok(()),
        }
    }

    /// Unreadable as `gh` would be where a release a write names is not
    /// there.
    fn not_found(&self, tag: &str) -> Not {
        Not::unreadable(
            &self.source,
            format!("gh release of {tag}"),
            Some("release not found".into()),
        )
    }
}

impl Backend for Scripted {
    fn tag(&self, at: &At<'_>, tag: &str) -> Answer<Option<Revision>> {
        self.ask(Asked::Tag(tag.into()), at)?;
        Ok(self.state.borrow().tags.get(tag).cloned())
    }

    fn release(&self, at: &At<'_>, tag: &str) -> Answer<Option<Release>> {
        self.ask(Asked::Release(tag.into()), at)?;
        Ok(self.release_of(tag).map(|held| Release {
            assets: held.assets.into_values().collect(),
        }))
    }

    fn create(&self, at: &At<'_>, tag: &str, notes: &[u8]) -> Answer<()> {
        let notes = String::from_utf8(notes.to_vec()).unwrap();
        let asked = Asked::Create {
            tag: tag.into(),
            notes: notes.clone(),
        };
        self.ask(asked, at)?;
        let mut state = self.state.borrow_mut();
        assert!(
            state.tags.contains_key(tag),
            "a draft was made of {tag}, which GitHub does not hold"
        );
        assert!(
            !state.releases.contains_key(tag),
            "a second release was made of {tag}"
        );
        state
            .releases
            .insert(tag.into(), Held::of(tag, &notes, true));
        Ok(())
    }

    fn upload(&self, at: &At<'_>, tag: &str, assets: &[Upload<'_>]) -> Answer<()> {
        let names = assets.iter().map(|a| a.output.to_string()).collect();
        let asked = Asked::Upload {
            tag: tag.into(),
            assets: names,
        };
        self.ask(asked, at)?;
        let mut state = self.state.borrow_mut();
        let Some(release) = state.releases.get_mut(tag) else {
            return Err(self.not_found(tag));
        };
        for asset in assets {
            let name = asset.output.to_string();
            let held = Asset {
                name: name.clone(),
                state: "uploaded".into(),
                digest: Some(Digest::of(asset.bytes)),
            };
            release.assets.insert(name, held);
        }
        Ok(())
    }

    fn publish(&self, at: &At<'_>, tag: &str, notes: &[u8]) -> Answer<()> {
        let notes = String::from_utf8(notes.to_vec()).unwrap();
        let asked = Asked::Publish {
            tag: tag.into(),
            notes: notes.clone(),
        };
        self.ask(asked, at)?;
        let mut state = self.state.borrow_mut();
        assert!(
            state.tags.contains_key(tag),
            "a release of {tag} was published, which GitHub does not hold"
        );
        let Some(release) = state.releases.get_mut(tag) else {
            return Err(self.not_found(tag));
        };
        release.draft = false;
        release.prerelease = false;
        release.title = tag.into();
        release.notes = notes;
        release.updated_at = Some(CLOCK.into());
        state.latest = Some(tag.into());
        Ok(())
    }

    fn latest(&self, at: &At<'_>) -> Answer<Option<Latest>> {
        self.ask(Asked::Latest, at)?;
        let state = self.state.borrow();
        Ok(state.latest.as_ref().map(|tag| Latest {
            tag: tag.clone(),
            updated_at: state.releases.get(tag).and_then(|r| r.updated_at.clone()),
        }))
    }
}
