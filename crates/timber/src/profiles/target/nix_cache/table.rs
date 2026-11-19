//! The environment's `nix_cache` table, read at every operation of the
//! target: the cache, named once, by `attic`, the cache as attic's own
//! command names it, or by `cachix`, the cache's name on Cachix, with
//! `credential`, the name of the host's credential holding its token;
//! and `closures`, the outputs whose artifacts name closures; and no
//! other key, so no credential sits in the repository's file (45, 118,
//! rule core.21; the Nix cache unit's design D7, the Cachix change's
//! design D3).

use std::collections::BTreeSet;
use std::fmt;

use serde_json::{Map, Value};
use timber_seams::{Answer, Environment, Not, OutputName, Source};

use super::PROFILE;

/// The key naming an attic cache.
const ATTIC: &str = "attic";

/// The key naming a Cachix cache.
const CACHIX: &str = "cachix";

/// The key naming the host's credential a Cachix cache's token is in.
const CREDENTIAL: &str = "credential";

/// The key naming the closure outputs.
const CLOSURES: &str = "closures";

/// Every key the table reads.
const KEYS: [&str; 4] = [ATTIC, CACHIX, CREDENTIAL, CLOSURES];

/// The form attic's own command names a cache in.
pub(super) const CACHE_FORM: &str = "<server>:<cache>";

/// The cache a table names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Cache {
    /// An attic cache, `<server>:<cache>`, attic reading the server and
    /// the token from its own configuration on the host.
    Attic(String),
    /// A Cachix cache by its name on Cachix, its token in the host's
    /// credential `credential` names.
    Cachix { name: String, credential: String },
}

impl fmt::Display for Cache {
    /// The cache as an answer names it: `fleet:swancloud`, or `boomdrive
    /// on Cachix`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cache::Attic(attic) => f.write_str(attic),
            Cache::Cachix { name, .. } => write!(f, "{name} on Cachix"),
        }
    }
}

/// What an environment's table names: the cache, and one or more closure
/// outputs, each once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Table {
    pub cache: Cache,
    pub closures: Vec<OutputName>,
}

/// An environment as an answer names it: by its name and its component.
pub(super) struct Named<'a>(pub &'a Environment);

impl fmt::Display for Named<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let environment = self.0;
        write!(
            f,
            "environment {} of {}",
            environment.name,
            environment.channel.component_name()
        )
    }
}

impl Table {
    /// `environment`'s settings read as the table, or unreadable naming
    /// the environment and the first key found wanting: one the target
    /// does not read; both caches, naming `cachix`; a `credential` beside
    /// `attic`; an `attic` that is not `<server>:<cache>`; a `cachix`
    /// that is no cache's name, or one with no `credential`; a
    /// `credential` that is no credential's name; or a `closures` that
    /// names no output, or one twice. A table naming no cache is
    /// unreadable naming the table.
    pub fn read(environment: &Environment, source: &Source) -> Answer<Table> {
        let named = Named(environment);
        let table = || format!("the {PROFILE} table of {named}");
        let wanting = |key: &str, said: String| {
            Not::unreadable(source, format!("setting {key} of {named}"), Some(said))
        };
        let Some(settings) = environment.settings.as_object() else {
            return Err(Not::unreadable(
                source,
                table(),
                Some(format!(
                    "it is {}, and the {PROFILE} target reads a table naming its cache, by \
                     {ATTIC} or by {CACHIX} with {CREDENTIAL}, and {CLOSURES}",
                    environment.settings
                )),
            ));
        };
        if let Some(key) = settings.keys().find(|k| !KEYS.contains(&k.as_str())) {
            return Err(wanting(
                key,
                format!(
                    "the {PROFILE} target reads {ATTIC}, or {CACHIX} with {CREDENTIAL}, and \
                     {CLOSURES}, and no setting {key}; attic reads the cache's token from its \
                     own configuration on the host that pushes, and a Cachix cache's token is \
                     in the host's credential {CREDENTIAL} names"
                ),
            ));
        }
        let cache = match (settings.get(ATTIC), settings.get(CACHIX)) {
            (Some(_), Some(_)) => {
                return Err(wanting(
                    CACHIX,
                    format!(
                        "the table names {ATTIC} too, and names one cache, by {ATTIC} or by \
                         {CACHIX}"
                    ),
                ))
            }
            (None, None) => {
                return Err(Not::unreadable(
                    source,
                    table(),
                    Some(format!(
                        "it names no cache, by {ATTIC}, the cache as {CACHE_FORM}, or by \
                         {CACHIX}, the cache's name on Cachix, with {CREDENTIAL}"
                    )),
                ))
            }
            (Some(attic), None) => Cache::Attic(attic_of(attic, settings, &wanting)?),
            (None, Some(cachix)) => cachix_of(cachix, settings, &wanting)?,
        };
        let closures = match settings.get(CLOSURES) {
            None => {
                return Err(wanting(
                    CLOSURES,
                    format!("the table names no {CLOSURES}, the outputs that name closures"),
                ))
            }
            Some(Value::Array(closures)) => closures,
            Some(other) => {
                return Err(wanting(
                    CLOSURES,
                    format!("{CLOSURES} is {other}, and is a list of output names"),
                ))
            }
        };
        if closures.is_empty() {
            return Err(wanting(
                CLOSURES,
                format!("{CLOSURES} names no output, and names one or more"),
            ));
        }
        let mut seen = BTreeSet::new();
        let mut outputs = Vec::new();
        for closure in closures {
            let name = match closure {
                Value::String(name) if !name.is_empty() => name,
                other => {
                    return Err(wanting(
                        CLOSURES,
                        format!("{CLOSURES} holds {other}, which is no output's name"),
                    ))
                }
            };
            if !seen.insert(name) {
                return Err(wanting(CLOSURES, format!("{CLOSURES} names {name} twice")));
            }
            outputs.push(OutputName::new(name.as_str()));
        }
        Ok(Table {
            cache,
            closures: outputs,
        })
    }
}

/// An attic cache, `attic`, as `<server>:<cache>`, with no `credential`
/// beside it, since attic reads its token from its own configuration.
fn attic_of(
    attic: &Value,
    settings: &Map<String, Value>,
    wanting: &dyn Fn(&str, String) -> Not,
) -> Answer<String> {
    if settings.contains_key(CREDENTIAL) {
        return Err(wanting(
            CREDENTIAL,
            format!(
                "the table names {ATTIC}, and attic reads its token from its own configuration \
                 on the host that pushes, so an attic cache takes no {CREDENTIAL}"
            ),
        ));
    }
    let attic = match attic {
        Value::String(attic) => attic,
        other => {
            return Err(wanting(
                ATTIC,
                format!("{ATTIC} is {other}, and names the cache as {CACHE_FORM}"),
            ))
        }
    };
    if let Some(problem) = attic_problem(attic) {
        return Err(wanting(
            ATTIC,
            format!(
                "{ATTIC} is {attic:?}, and attic's own command names a cache {CACHE_FORM}: \
                 {problem}"
            ),
        ));
    }
    Ok(attic.clone())
}

/// A Cachix cache, `cachix`, by its name, with the `credential` its token
/// is in.
fn cachix_of(
    cachix: &Value,
    settings: &Map<String, Value>,
    wanting: &dyn Fn(&str, String) -> Not,
) -> Answer<Cache> {
    const NAME_FORM: &str = "the cache's name on Cachix, as `cachix push` takes it and never its \
                             address: one or more lowercase letters, digits and `-`, beginning \
                             with no `-`";
    let name = match cachix {
        Value::String(name) => name,
        other => {
            return Err(wanting(
                CACHIX,
                format!("{CACHIX} is {other}, and is {NAME_FORM}"),
            ))
        }
    };
    if let Some(problem) = cachix_problem(name) {
        return Err(wanting(
            CACHIX,
            format!("{CACHIX} is {name:?}, and is {NAME_FORM}: {problem}"),
        ));
    }
    const CREDENTIAL_FORM: &str = "the name of the host's credential holding the cache's token, \
                                   as its bindings file's [credentials] names it: a name with no \
                                   whitespace";
    let credential = match settings.get(CREDENTIAL) {
        None => {
            return Err(wanting(
                CREDENTIAL,
                format!(
                    "the table names {CACHIX} and no {CREDENTIAL}, {CREDENTIAL_FORM}, and a \
                     Cachix cache is pushed to with its own token alone"
                ),
            ))
        }
        Some(Value::String(credential)) => credential,
        Some(other) => {
            return Err(wanting(
                CREDENTIAL,
                format!("{CREDENTIAL} is {other}, and is {CREDENTIAL_FORM}"),
            ))
        }
    };
    let problem = if credential.is_empty() {
        Some("it is empty")
    } else if credential.chars().any(char::is_whitespace) {
        Some("it holds whitespace")
    } else {
        None
    };
    if let Some(problem) = problem {
        return Err(wanting(
            CREDENTIAL,
            format!("{CREDENTIAL} is {credential:?}, and is {CREDENTIAL_FORM}: {problem}"),
        ));
    }
    Ok(Cache::Cachix {
        name: name.clone(),
        credential: credential.clone(),
    })
}

/// What keeps `attic` from naming a cache as attic's own command names
/// one: no `:`, nothing before it or after it, a second one, or a
/// server beginning `-`, which attic would read as an option.
fn attic_problem(attic: &str) -> Option<&'static str> {
    let Some((server, cache)) = attic.split_once(':') else {
        return Some("it names no `:`");
    };
    if server.is_empty() {
        Some("it names no server before its `:`")
    } else if cache.is_empty() {
        Some("it names no cache after its `:`")
    } else if cache.contains(':') {
        Some("it names more than one `:`")
    } else if server.starts_with('-') {
        Some("its server begins `-`, which attic reads as an option")
    } else {
        None
    }
}

/// What keeps `name` from being a cache's name on Cachix: nothing, a
/// first `-`, which cachix would read as an option, or a character that
/// is no lowercase letter, digit or `-`, as an address's `:` and `.`.
fn cachix_problem(name: &str) -> Option<String> {
    if name.is_empty() {
        Some("it is empty".into())
    } else if name.starts_with('-') {
        Some("it begins `-`, which cachix reads as an option".into())
    } else {
        name.chars()
            .find(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-'))
            .map(|c| format!("it holds {c:?}, which is no lowercase letter, digit or `-`"))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use timber_seams::{ChannelId, ChannelName, ComponentName, EnvironmentName, Seam};

    use super::*;

    fn source() -> Source {
        Source::new(Seam::Target, PROFILE)
    }

    fn cache(settings: Value) -> Environment {
        Environment {
            name: EnvironmentName::new("cache"),
            channel: ChannelId::new(
                Some(ComponentName::new("swancloud-srv")),
                ChannelName::new("beta"),
            ),
            waits: false,
            settings,
        }
    }

    /// The key `settings` is unreadable for, from `what`, and its words.
    fn wanting(settings: Value) -> (String, String) {
        match Table::read(&cache(settings.clone()), &source()) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, source());
                (u.what, u.said.expect("the target's words"))
            }
            other => panic!("{settings} answered {other:?}"),
        }
    }

    fn setting(key: &str) -> String {
        format!("setting {key} of environment cache of swancloud-srv")
    }

    #[test]
    fn the_cache_and_its_closures_are_read() {
        let table = Table::read(
            &cache(json!({ "attic": "fleet:swancloud", "closures": ["swancloud-srv", "notes"] })),
            &source(),
        )
        .unwrap();
        assert_eq!(table.cache, Cache::Attic("fleet:swancloud".into()));
        assert_eq!(table.cache.to_string(), "fleet:swancloud");
        assert_eq!(
            table.closures,
            [OutputName::new("swancloud-srv"), OutputName::new("notes")]
        );
    }

    #[test]
    fn a_token_in_the_table_is_unreadable_naming_the_environment_and_the_key() {
        let (what, said) = wanting(json!({
            "attic": "fleet:swancloud",
            "closures": ["swancloud-srv"],
            "token": "eyJhbGciOi",
        }));
        assert_eq!(what, setting("token"));
        assert!(said.contains("no setting token"), "{said}");
        assert!(!said.contains("eyJhbGciOi"), "{said}");
    }

    #[test]
    fn a_cache_with_no_server_is_unreadable_saying_it_is_server_and_cache() {
        let (what, said) = wanting(json!({ "attic": "swancloud", "closures": ["swancloud-srv"] }));
        assert_eq!(what, setting("attic"));
        assert!(said.contains("<server>:<cache>"), "{said}");
        assert!(said.ends_with("it names no `:`"), "{said}");
    }

    #[test]
    fn a_cache_attic_cannot_name_is_unreadable_saying_why() {
        for (attic, problem) in [
            (":swancloud", "it names no server before its `:`"),
            ("fleet:", "it names no cache after its `:`"),
            ("fleet:swan:cloud", "it names more than one `:`"),
            (
                "-fleet:swancloud",
                "its server begins `-`, which attic reads as an option",
            ),
        ] {
            let (what, said) = wanting(json!({ "attic": attic, "closures": ["swancloud-srv"] }));
            assert_eq!(what, setting("attic"));
            assert!(said.ends_with(problem), "{attic}: {said}");
        }
        let (what, said) = wanting(json!({ "attic": 3, "closures": ["swancloud-srv"] }));
        assert_eq!(what, setting("attic"));
        assert!(said.starts_with("attic is 3"), "{said}");
    }

    #[test]
    fn an_empty_closures_is_unreadable_naming_it() {
        let (what, said) = wanting(json!({ "attic": "fleet:swancloud", "closures": [] }));
        assert_eq!(what, setting("closures"));
        assert!(said.contains("names no output"), "{said}");
    }

    #[test]
    fn closures_that_are_no_list_of_names_each_once_are_unreadable_naming_it() {
        for (closures, words) in [
            (json!("swancloud-srv"), "closures is \"swancloud-srv\""),
            (json!([""]), "closures holds \"\""),
            (json!([4]), "closures holds 4"),
            (
                json!(["swancloud-srv", "swancloud-srv"]),
                "closures names swancloud-srv twice",
            ),
        ] {
            let (what, said) = wanting(json!({ "attic": "fleet:swancloud", "closures": closures }));
            assert_eq!(what, setting("closures"));
            assert!(said.starts_with(words), "{said}");
        }
    }

    #[test]
    fn a_table_missing_a_key_is_unreadable_naming_it() {
        let (what, _) = wanting(json!({ "closures": ["swancloud-srv"] }));
        assert_eq!(what, table());
        let (what, _) = wanting(json!({ "attic": "fleet:swancloud" }));
        assert_eq!(what, setting("closures"));
        let (what, _) = wanting(json!({ "cachix": "boomdrive", "credential": "cachix-boomdrive" }));
        assert_eq!(what, setting("closures"));
    }

    fn table() -> String {
        "the nix_cache table of environment cache of swancloud-srv".into()
    }

    // Requirement: A Cachix cache is named by its name, with the host's
    // credential

    #[test]
    fn boomdrives_cache_is_read_as_the_cachix_cache_its_credential_and_its_closure() {
        let table = Table::read(
            &cache(json!({
                "cachix": "boomdrive",
                "credential": "cachix-boomdrive",
                "closures": ["boomdrive-x86_64-linux"],
            })),
            &source(),
        )
        .unwrap();
        assert_eq!(
            table.cache,
            Cache::Cachix {
                name: "boomdrive".into(),
                credential: "cachix-boomdrive".into(),
            }
        );
        assert_eq!(table.cache.to_string(), "boomdrive on Cachix");
        assert_eq!(table.closures, [OutputName::new("boomdrive-x86_64-linux")]);
    }

    #[test]
    fn a_key_the_target_does_not_read_beside_cachix_is_unreadable_naming_it() {
        let (what, said) = wanting(json!({
            "cachix": "boomdrive",
            "credential": "cachix-boomdrive",
            "closures": ["boomdrive-x86_64-linux"],
            "token": "eyJhbGciOi",
        }));
        assert_eq!(what, setting("token"));
        assert!(said.contains("no setting token"), "{said}");
        assert!(!said.contains("eyJhbGciOi"), "{said}");
    }

    #[test]
    fn both_caches_are_unreadable_naming_cachix_and_saying_the_table_names_one() {
        let (what, said) = wanting(json!({
            "attic": "fleet:swancloud",
            "cachix": "boomdrive",
            "credential": "cachix-boomdrive",
            "closures": ["swancloud-srv"],
        }));
        assert_eq!(what, setting("cachix"));
        assert!(said.contains("names one cache"), "{said}");
    }

    #[test]
    fn no_cache_is_unreadable_naming_the_table_and_saying_it_names_none() {
        let (what, said) = wanting(json!({ "closures": ["swancloud-srv"] }));
        assert_eq!(what, table());
        assert!(
            said.starts_with(
                "it names no cache, by attic, the cache as <server>:<cache>, or by cachix"
            ),
            "{said}"
        );
        let (what, _) =
            wanting(json!({ "credential": "cachix-boomdrive", "closures": ["swancloud-srv"] }));
        assert_eq!(what, table());
    }

    #[test]
    fn a_credential_beside_attic_is_unreadable_saying_attic_reads_its_own_configuration() {
        let (what, said) = wanting(json!({
            "attic": "fleet:swancloud",
            "credential": "attic-fleet",
            "closures": ["swancloud-srv"],
        }));
        assert_eq!(what, setting("credential"));
        assert!(
            said.contains("attic reads its token from its own configuration on the host"),
            "{said}"
        );
    }

    #[test]
    fn cachix_with_no_credential_is_unreadable_naming_credential() {
        let (what, said) = wanting(json!({ "cachix": "boomdrive", "closures": ["boomdrive"] }));
        assert_eq!(what, setting("credential"));
        assert!(
            said.starts_with("the table names cachix and no credential"),
            "{said}"
        );
    }

    #[test]
    fn a_cachix_that_is_no_caches_name_is_unreadable_saying_it_is_the_name_on_cachix() {
        for (cachix, problem) in [
            (
                json!("https://boomdrive.cachix.org"),
                "it holds ':', which is no lowercase letter, digit or `-`",
            ),
            (
                json!("-boomdrive"),
                "it begins `-`, which cachix reads as an option",
            ),
            (
                json!("Boomdrive"),
                "it holds 'B', which is no lowercase letter, digit or `-`",
            ),
            (json!(""), "it is empty"),
        ] {
            let (what, said) = wanting(json!({
                "cachix": cachix,
                "credential": "cachix-boomdrive",
                "closures": ["boomdrive"],
            }));
            assert_eq!(what, setting("cachix"));
            assert!(said.contains("the cache's name on Cachix"), "{said}");
            assert!(said.ends_with(problem), "{cachix}: {said}");
        }
        let (what, said) = wanting(json!({
            "cachix": ["boomdrive"],
            "credential": "cachix-boomdrive",
            "closures": ["boomdrive"],
        }));
        assert_eq!(what, setting("cachix"));
        assert!(said.starts_with("cachix is [\"boomdrive\"]"), "{said}");
    }

    #[test]
    fn a_credential_that_is_no_credentials_name_is_unreadable_saying_its_form() {
        for (credential, problem) in [
            (json!("cachix boomdrive"), "it holds whitespace"),
            (json!(""), "it is empty"),
        ] {
            let (what, said) = wanting(json!({
                "cachix": "boomdrive",
                "credential": credential,
                "closures": ["boomdrive"],
            }));
            assert_eq!(what, setting("credential"));
            assert!(said.contains("a name with no whitespace"), "{said}");
            assert!(said.ends_with(problem), "{credential}: {said}");
        }
        let (what, said) = wanting(json!({
            "cachix": "boomdrive",
            "credential": 7,
            "closures": ["boomdrive"],
        }));
        assert_eq!(what, setting("credential"));
        assert!(said.starts_with("credential is 7"), "{said}");
    }

    #[test]
    fn settings_that_are_no_table_are_unreadable_naming_the_environment() {
        let (what, said) = wanting(json!("fleet:swancloud"));
        assert_eq!(
            what,
            "the nix_cache table of environment cache of swancloud-srv"
        );
        assert!(said.starts_with("it is \"fleet:swancloud\""), "{said}");
    }
}
