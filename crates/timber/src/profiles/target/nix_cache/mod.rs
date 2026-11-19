//! The Nix cache target (C.12's Nix binary cache row), bound as
//! `nix_cache`: a version's closures pushed to a Nix binary cache, the
//! fleet's with attic or one on Cachix with cachix, from the host that
//! holds them, so a closure built on one host is realised on another from
//! the cache alone.
//!
//! An environment names it in its `target` table, which names the cache
//! once and the outputs whose artifacts name closures, and nothing else
//! (C.10; design D7; the Cachix change's design D3): an attic cache as
//! attic's own command names it, `<server>:<cache>`, or a Cachix cache by
//! its name on Cachix with `credential`, the name of the host's
//! credential holding its token:
//!
//! ```toml
//! [environments.swancloud-srv.cache]
//! channel = "beta"
//! target.nix_cache = { attic = "fleet:swancloud", closures = ["swancloud-srv"] }
//!
//! [environments.boomdrive.cachix-x86_64-linux]
//! channel = "beta"
//! target.nix_cache = { cachix = "boomdrive", credential = "cachix-boomdrive", closures = ["boomdrive-x86_64-linux"] }
//! ```
//!
//! Any other key, both caches or neither, a `credential` beside `attic`,
//! a `cachix` with none, an empty `closures`, or a cache or credential of
//! another form is unreadable naming the environment and the key, at
//! every operation, so no token is written where timber reads (45, rule
//! core.21).
//!
//! A closure is one artifact naming its roots: a machine's build task
//! leaves a file of its closure's root store paths, one per line, which
//! its component declares as an output and the file projection packs.
//! Its form is read by [`nix_closure`], which the Nix
//! profiles share (design D6). The store holds that file and no store
//! path of the closure; the paths are the cache's (section 10).
//!
//! Apply reads the verified version's index and each closure's file
//! through the store by digest (42, rule core.4), then checks every root
//! valid in this host's Nix store with `nix-store --check-validity`,
//! refusing the first it does not hold, naming it and the host: the host
//! that applies is the host that built, and nothing is built, substituted
//! or fetched (15, I3; design D8). Then the cache is pushed each root
//! with its closure, skipping what it holds already, and the pushing
//! tool's words on a failure are the answer, unreadable, verbatim (59).
//!
//! - **attic**: `attic push <server>:<cache> <roots…>`. attic reads its
//!   servers and the environment's token from its own configuration on
//!   that host, `$XDG_CONFIG_HOME/attic/config.toml`, at the push: timber
//!   passes it no token, sets none of its variables and reads nothing of
//!   its configuration (45).
//! - **Cachix**: `cachix --config <folder>/cachix.dhall push <name>
//!   <roots…>`. The token is the host's: the bindings file's
//!   `[credentials]` names the path of the file holding it, by the name
//!   the table's `credential` gives. `cachix` reads no token file, so
//!   apply looks the credential's path up once the table is read, before
//!   the store is asked anything, reads the file through the family's
//!   [`token`](super::token) once every root is held, and hands the token
//!   to that one run of `cachix` alone, in `CACHIX_AUTH_TOKEN`, writing it
//!   nowhere (64, record 0044, rule core.21; the Cachix change's design
//!   D5, D7). The run has every `CACHIX_*` variable of timber's own
//!   environment removed, and its `--config` names a file never made, in
//!   an empty folder the operation makes under the system's temporary
//!   directory and removes when apply answers, so no token, signing key
//!   or configuration of the host's own cachix,
//!   `~/.config/cachix/cachix.dhall` among them, is read.
//!
//! A Nix binary cache holds store paths by their hash and keeps no name a
//! marker could be held under, so current and report are unread, saying
//! so and naming the cache, never the digest last pushed and never
//! converged, and read no credential (119, 89, 98; design D9). desired
//! answers the head the core hands it (88), and apply runs whenever it is
//! asked.

mod table;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

use timber_seams::{
    Answer, Digest, Environment, Head, Not, Running, Seam, Source, Store, Target, Verified, Why,
};

use super::nix_closure::{self, StorePath};
use super::token::{self, Credential, Token};
use crate::bindings::Credentials;
use crate::profiles::run::Tool;
use table::{Cache, Named, Table};

/// The profile name an environment's `target` table names this target by.
pub const PROFILE: &str = "nix_cache";

/// The variables removed from cachix's run: every one of the host's own
/// cachix settings, its token and its signing key among them.
const CACHIX_ENV: &[&str] = &["CACHIX_*"];

/// The variable cachix's run is handed the token in.
const CACHIX_AUTH_TOKEN: &str = "CACHIX_AUTH_TOKEN";

/// The Nix cache target, reading through a store binding of its own,
/// opened and never made, and running attic, cachix and nix-store.
pub struct NixCache {
    store: Box<dyn Store>,
    credentials: Credentials,
    attic: Tool,
    cachix: Tool,
    nix_store: Tool,
    source: Source,
}

impl NixCache {
    /// The target reading through `store`, with the token of each Cachix
    /// cache the file of `credentials`, the host's, names, running
    /// `attic`, `cachix` and `nix-store` as found on `PATH`.
    pub fn new(store: Box<dyn Store>, credentials: &Credentials) -> NixCache {
        NixCache::with_programs(store, credentials, "attic", "cachix", "nix-store")
    }

    /// The target as [`NixCache::new`] makes it, running attic as
    /// `attic`, cachix as `cachix` and nix-store as `nix_store`.
    pub fn with_programs(
        store: Box<dyn Store>,
        credentials: &Credentials,
        attic: impl Into<PathBuf>,
        cachix: impl Into<PathBuf>,
        nix_store: impl Into<PathBuf>,
    ) -> NixCache {
        NixCache {
            store,
            credentials: credentials.clone(),
            attic: Tool::new(attic, NixCache::source()),
            cachix: Tool::new(cachix, NixCache::source()),
            nix_store: Tool::new(nix_store, NixCache::source()),
            source: NixCache::source(),
        }
    }

    /// The target seam and this profile, as every answer names them.
    pub fn source() -> Source {
        Source::new(Seam::Target, PROFILE)
    }

    /// 119: what the environment runs, unread, since the cache keeps no
    /// marker of what was pushed to it (design D9).
    fn no_marker(&self, environment: &Environment, table: &Table) -> Not {
        Not::unreadable(
            &self.source,
            format!("what {} runs", Named(environment)),
            Some(format!(
                "the Nix binary cache {} holds store paths by their hash and keeps no marker of \
                 what was pushed to it",
                table.cache
            )),
        )
    }

    /// Refused, by this target, for `said`.
    fn refused(&self, said: String) -> Not {
        Not::refused(&self.source, Why::Said, Some(said))
    }

    /// Every root of `roots` valid in this host's Nix store, or refused
    /// naming the first it does not hold and the host; nothing is built,
    /// substituted or fetched (15, I3; design D8).
    fn held(&self, roots: &[StorePath]) -> Answer<()> {
        let mut args = vec!["--check-validity", "--print-invalid"];
        args.extend(roots.iter().map(StorePath::as_str));
        let ran = self.nix_store.output(None, &args, &[], &[], &[0])?;
        let printed = String::from_utf8_lossy(&ran.stdout);
        let invalid: BTreeSet<&str> = printed.lines().collect();
        match roots.iter().find(|root| invalid.contains(root.as_str())) {
            None => Ok(()),
            Some(root) => Err(self.refused(format!(
                "{root} is not valid in the Nix store of {}, and the {PROFILE} target pushes \
                 only what the host that applies holds, building and fetching nothing",
                host()
            ))),
        }
    }

    /// `attic push <server>:<cache> <roots…>`: attic reads the cache's
    /// server and token from its own configuration and pushes each root
    /// with its closure; timber passes it no token and sets none of its
    /// variables (45, rule core.21; design D7). attic's words on a
    /// failure are the answer, unreadable.
    fn push_attic(&self, attic: &str, roots: &[StorePath]) -> Answer<()> {
        let mut args = vec!["push", attic];
        args.extend(roots.iter().map(StorePath::as_str));
        self.attic.output(None, &args, &[], &[], &[0]).map(|_| ())
    }

    /// `cachix --config <folder>/cachix.dhall push <name> <roots…>`, in a
    /// folder of the operation's own under the system's temporary
    /// directory, empty and removed once the run ends, whatever it
    /// answered; the run has every `CACHIX_*` variable of timber's own
    /// environment removed and `token` in `CACHIX_AUTH_TOKEN`, and in no
    /// argument (45, 64, rule core.21; the Cachix change's design D5, D7).
    /// cachix pushes each root with its closure, skipping what the cache
    /// holds; its words on a failure are the answer, unreadable.
    fn push_cachix(&self, name: &str, token: &Token, roots: &[StorePath]) -> Answer<()> {
        let folder = tempfile::Builder::new()
            .prefix("timber-cachix-")
            .tempdir()
            .map_err(|e| {
                Not::unreadable(
                    &self.source,
                    format!(
                        "a folder for a run of cachix under {}",
                        std::env::temp_dir().display()
                    ),
                    Some(e.to_string()),
                )
            })?;
        let config = folder.path().join("cachix.dhall");
        let mut args: Vec<&OsStr> = vec![
            OsStr::new("--config"),
            config.as_os_str(),
            OsStr::new("push"),
            OsStr::new(name),
        ];
        args.extend(roots.iter().map(|root| OsStr::new(root.as_str())));
        let set: [(&str, &OsStr); 1] = [(CACHIX_AUTH_TOKEN, OsStr::new(token.secret()))];
        let pushed = self
            .cachix
            .output(None, &args, CACHIX_ENV, &set, &[0])
            .map(|_| ());
        drop(folder);
        pushed
    }

    /// How apply pushes to `cache`: an attic cache, or a Cachix cache with
    /// the path of the file holding its token, looked up in the host's
    /// credentials by the name the table gives it, the file never opened
    /// (118).
    fn push_of<'a>(&'a self, environment: &Environment, cache: &'a Cache) -> Answer<Push<'a>> {
        match cache {
            Cache::Attic(attic) => Ok(Push::Attic(attic)),
            Cache::Cachix { name, credential } => {
                let named = Named(environment);
                let path = token::path(
                    &self.credentials,
                    &cachix_credential(credential, &named),
                    &self.source,
                )?;
                Ok(Push::Cachix {
                    name,
                    credential,
                    path,
                })
            }
        }
    }
}

/// How apply pushes, as the table names its cache.
enum Push<'a> {
    /// `attic push`, to the cache `<server>:<cache>`.
    Attic(&'a str),
    /// `cachix push`, to the cache `name`, its token in the file at
    /// `path` the credential `credential` names.
    Cachix {
        name: &'a str,
        credential: &'a str,
        path: &'a Path,
    },
}

/// The Cachix cache's credential `name`, of the environment `of`, as this
/// target names it in its answers.
fn cachix_credential<'a>(name: &'a str, of: &'a Named<'a>) -> Credential<'a> {
    Credential {
        name,
        of,
        hands: "the nix_cache target hands cachix",
    }
}

/// The host this target runs on, by the name the operating system
/// answers, or saying it cannot be read.
fn host() -> String {
    let mut name = [0u8; 256];
    // SAFETY: the pointer and the length are the buffer's own, and
    // gethostname writes no more than the length it is given.
    let made = unsafe { libc::gethostname(name.as_mut_ptr().cast(), name.len()) };
    if made != 0 {
        return format!(
            "this host, whose name cannot be read: {}",
            io::Error::last_os_error()
        );
    }
    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    format!("the host {}", String::from_utf8_lossy(&name[..end]))
}

impl Target for NixCache {
    /// 88: the head the core handed, as a target that is pushed to
    /// answers it, once the table is read.
    fn desired(&self, environment: &Environment, head: Head<'_>) -> Answer<Digest> {
        Table::read(environment, &self.source)?;
        head()
    }

    fn current(&self, environment: &Environment) -> Answer<Digest> {
        let table = Table::read(environment, &self.source)?;
        Err(self.no_marker(environment, &table))
    }

    /// 90: the roots of every closure the table names, read from the
    /// version's index through the store, checked held in this host's Nix
    /// store, and pushed with attic or cachix; a Cachix cache's credential
    /// is looked up before the store is asked and its token read once
    /// every root is held; nothing runs before every closure is read, and
    /// neither attic nor cachix runs before every root is held.
    fn apply(&self, environment: &Environment, version: &Verified) -> Answer<()> {
        let table = Table::read(environment, &self.source)?;
        let push = self.push_of(environment, &table.cache)?;
        let index = nix_closure::index(self.store.as_ref(), version, &self.source)?;
        let store_dir = nix_closure::store_dir();
        let mut roots: Vec<StorePath> = Vec::new();
        for output in &table.closures {
            let artifact = nix_closure::artifact_of(&index, output, &self.source)?;
            let bytes = nix_closure::bytes(self.store.as_ref(), artifact, &self.source)?;
            for root in nix_closure::roots(artifact, &bytes, &store_dir, &self.source)? {
                if !roots.contains(&root) {
                    roots.push(root);
                }
            }
        }
        self.held(&roots)?;
        match push {
            Push::Attic(attic) => self.push_attic(attic, &roots),
            Push::Cachix {
                name,
                credential,
                path,
            } => {
                let named = Named(environment);
                let token =
                    token::read(path, &cachix_credential(credential, &named), &self.source)?;
                self.push_cachix(name, &token, &roots)
            }
        }
    }

    fn report(&self, environment: &Environment) -> Answer<Running> {
        let table = Table::read(environment, &self.source)?;
        Err(self.no_marker(environment, &table))
    }
}
