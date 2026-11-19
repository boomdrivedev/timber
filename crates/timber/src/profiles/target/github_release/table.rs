//! The environment's `github_release` table, read at every operation of
//! the target: `credential`, the name of the host's credential its
//! `[credentials]` gives the path of; and `assets`, the component's
//! outputs whose file artifacts the release carries, each under its
//! output's name, possibly none. Both are required, and no other key is
//! read, so no token, path, host or repository sits in the table (45,
//! C.10, C.12; design D3). A key found wanting is named with the two the
//! table reads, and a value that may be a credential is in no words.

use std::collections::BTreeMap;
use std::fmt;

use serde_json::Value;
use timber_seams::{Answer, Environment, Not, OutputName, Source};

use super::PROFILE;

/// The key naming the host's credential.
pub(super) const CREDENTIAL: &str = "credential";

/// The key naming the outputs the release carries as its assets.
pub(super) const ASSETS: &str = "assets";

/// What an environment's table names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Table {
    pub credential: String,
    pub assets: Vec<OutputName>,
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

/// The words every refusal of the table ends with.
fn reads() -> String {
    format!("the {PROFILE} target reads {CREDENTIAL} and {ASSETS}")
}

impl Table {
    /// `environment`'s settings read as the table, or unreadable naming
    /// the environment, the first key found wanting and the two the table
    /// reads: one the target does not read, or a required one missing or
    /// of another form.
    pub fn read(environment: &Environment, source: &Source) -> Answer<Table> {
        let named = Named(environment);
        let wanting = |key: &str, said: String| {
            Not::unreadable(
                source,
                format!("setting {key} of {named}"),
                Some(format!("{said}; {}", reads())),
            )
        };
        let Some(settings) = environment.settings.as_object() else {
            return Err(Not::unreadable(
                source,
                format!("the {PROFILE} table of {named}"),
                Some(format!("the settings are no table, and {}", reads())),
            ));
        };
        let known = [CREDENTIAL, ASSETS];
        if let Some(key) = settings.keys().find(|k| !known.contains(&k.as_str())) {
            return Err(wanting(
                key,
                format!(
                    "the table names {key}, which the {PROFILE} target does not read: the token \
                     is the host's, named by {CREDENTIAL} in the host's [credentials], and the \
                     release is the checkout's repository's on github.com"
                ),
            ));
        }
        let what = "name of a credential the host's [credentials] gives";
        let credential = match settings.get(CREDENTIAL) {
            None => {
                return Err(wanting(
                    CREDENTIAL,
                    format!("the table names no {CREDENTIAL}, which is required: a {what}"),
                ))
            }
            Some(Value::String(text)) => match credential_problem(text) {
                None => text.clone(),
                Some(problem) => {
                    return Err(wanting(
                        CREDENTIAL,
                        format!("{CREDENTIAL} is no {what}: {problem}"),
                    ))
                }
            },
            Some(_) => {
                return Err(wanting(
                    CREDENTIAL,
                    format!("{CREDENTIAL} is no {what}: it is not a string"),
                ))
            }
        };
        let what = "list of the component's outputs the release carries as its assets";
        let assets = match settings.get(ASSETS) {
            None => {
                return Err(wanting(
                    ASSETS,
                    format!(
                        "the table names no {ASSETS}, which is required: a {what}, `[]` for a \
                         release of notes alone"
                    ),
                ))
            }
            Some(value) => assets(value)
                .map_err(|problem| wanting(ASSETS, format!("{ASSETS} is no {what}: {problem}")))?,
        };
        Ok(Table { credential, assets })
    }
}

/// What keeps `credential` from being a credential's name: nothing, or
/// whitespace.
fn credential_problem(credential: &str) -> Option<String> {
    if credential.is_empty() {
        return Some("it is empty".into());
    }
    credential
        .chars()
        .any(char::is_whitespace)
        .then(|| "it holds whitespace".into())
}

/// `value` read as a list of output names, possibly empty, each named
/// once and each a name GitHub keeps as it is given; what keeps it from
/// being one, by the items' places, otherwise.
fn assets(value: &Value) -> Result<Vec<OutputName>, String> {
    let Value::Array(items) = value else {
        return Err("it is not a list".into());
    };
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    let mut names = Vec::new();
    for (place, item) in (1..).zip(items) {
        let name = match item {
            Value::String(name) if !name.is_empty() => name.as_str(),
            _ => return Err(format!("its item {place} is no output's name")),
        };
        if let Some(problem) = renamed(name) {
            return Err(format!(
                "its item {place}, {name:?}, {problem}, which GitHub would rename: an asset's \
                 name is letters, digits, `.`, `-` and `_`, neither beginning nor ending with `.`"
            ));
        }
        if let Some(first) = seen.insert(name, place) {
            return Err(format!(
                "its items {first} and {place} name one output, {name:?}"
            ));
        }
        names.push(OutputName::new(name));
    }
    Ok(names)
}

/// What GitHub would rename in `name`, an asset's: a character other
/// than a letter, a digit, `.`, `-` or `_`, or a `.` it begins or ends
/// with.
fn renamed(name: &str) -> Option<String> {
    if let Some(c) = name
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')))
    {
        return Some(format!("holds {c:?}"));
    }
    if name.starts_with('.') {
        return Some("begins with `.`".into());
    }
    if name.ends_with('.') {
        return Some("ends with `.`".into());
    }
    None
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use timber_seams::{ChannelId, ChannelName, ComponentName, EnvironmentName, Seam};

    use super::*;

    fn source() -> Source {
        Source::new(Seam::Target, PROFILE)
    }

    fn github(settings: Value) -> Environment {
        Environment {
            name: EnvironmentName::new("github"),
            channel: ChannelId::new(
                Some(ComponentName::new("flume")),
                ChannelName::new("stable"),
            ),
            waits: false,
            settings,
        }
    }

    fn table() -> Value {
        json!({
            "credential": "flume-releases",
            "assets": ["flume-x86_64-linux.tar.gz"],
        })
    }

    /// The table with `key` set to `value`.
    fn with(key: &str, value: Value) -> Value {
        let mut settings = table();
        settings[key] = value;
        settings
    }

    /// The table without `key`.
    fn without(key: &str) -> Value {
        let mut settings = table();
        settings.as_object_mut().unwrap().remove(key);
        settings
    }

    /// What `settings` is unreadable naming, and its words, once they
    /// name the two keys the table reads.
    fn wanting(settings: Value) -> (String, String) {
        match Table::read(&github(settings.clone()), &source()) {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, source());
                let said = u.said.expect("the target's words");
                assert!(
                    said.ends_with("the github_release target reads credential and assets"),
                    "{said}"
                );
                (u.what, said)
            }
            other => panic!("{settings} answered {other:?}"),
        }
    }

    fn setting(key: &str) -> String {
        format!("setting {key} of environment github of flume")
    }

    // Requirement: The table names the credential and the assets

    #[test]
    fn the_two_keys_are_read() {
        let settings = with(
            "assets",
            json!([
                "flume-aarch64-darwin.tar.gz",
                "flume-aarch64-linux.tar.gz",
                "flume_x86-64.linux.tar.gz"
            ]),
        );
        assert_eq!(
            Table::read(&github(settings), &source()),
            Ok(Table {
                credential: "flume-releases".into(),
                assets: vec![
                    OutputName::new("flume-aarch64-darwin.tar.gz"),
                    OutputName::new("flume-aarch64-linux.tar.gz"),
                    OutputName::new("flume_x86-64.linux.tar.gz"),
                ],
            })
        );
    }

    #[test]
    fn an_empty_assets_is_a_release_of_notes_alone() {
        let read = Table::read(&github(with("assets", json!([]))), &source());
        assert_eq!(read.map(|t| t.assets), Ok(vec![]));
    }

    #[test]
    fn each_key_missing_is_unreadable_naming_it_as_required() {
        for key in ["credential", "assets"] {
            let (what, said) = wanting(without(key));
            assert_eq!(what, setting(key));
            assert!(
                said.starts_with(&format!("the table names no {key}, which is required")),
                "{said}"
            );
        }
    }

    #[test]
    fn a_credential_that_is_no_name_is_unreadable_naming_credential_and_never_its_value() {
        for (credential, problem) in [
            (json!(""), "it is empty"),
            (json!("flume releases"), "it holds whitespace"),
            (json!("ghp_abc\n"), "it holds whitespace"),
            (json!({ "path": "/run/secrets/gh" }), "it is not a string"),
            (json!(["flume-releases"]), "it is not a string"),
        ] {
            let (what, said) = wanting(with("credential", credential.clone()));
            assert_eq!(what, setting("credential"));
            assert!(said.contains(problem), "{credential}: {said}");
            assert!(!said.contains("/run/secrets"), "{said}");
            assert!(!said.contains("ghp_abc"), "{said}");
        }
    }

    #[test]
    fn an_asset_with_a_space_is_unreadable_naming_assets_and_the_asset() {
        let (what, said) = wanting(with("assets", json!(["flume linux"])));
        assert_eq!(what, setting("assets"));
        assert!(
            said.contains("its item 1, \"flume linux\", holds ' ', which GitHub would rename"),
            "{said}"
        );
    }

    #[test]
    fn an_asset_beginning_or_ending_with_a_dot_is_unreadable_naming_it() {
        for (asset, problem) in [
            (".flume.tar.gz", "begins with `.`"),
            ("flume.tar.gz.", "ends with `.`"),
        ] {
            let (what, said) = wanting(with("assets", json!(["flume.tar.gz", asset])));
            assert_eq!(what, setting("assets"));
            assert!(
                said.contains(&format!(
                    "its item 2, {asset:?}, {problem}, which GitHub would rename"
                )),
                "{said}"
            );
        }
    }

    #[test]
    fn an_asset_github_would_rename_is_unreadable_naming_its_character() {
        for (asset, c) in [
            ("flume/linux", "'/'"),
            ("flüme.tar.gz", "'ü'"),
            ("flume+linux", "'+'"),
        ] {
            let (_, said) = wanting(with("assets", json!([asset])));
            assert!(said.contains(&format!("holds {c}")), "{asset}: {said}");
        }
    }

    #[test]
    fn a_repeated_asset_is_unreadable_naming_assets() {
        let (what, said) = wanting(with("assets", json!(["flume.tar.gz", "flume.tar.gz"])));
        assert_eq!(what, setting("assets"));
        assert!(
            said.contains("its items 1 and 2 name one output, \"flume.tar.gz\""),
            "{said}"
        );
    }

    #[test]
    fn assets_of_another_form_are_unreadable_naming_assets() {
        for (assets, problem) in [
            (json!("flume.tar.gz"), "it is not a list"),
            (json!({ "flume": "flume.tar.gz" }), "it is not a list"),
            (
                json!(["flume.tar.gz", ""]),
                "its item 2 is no output's name",
            ),
            (json!(["flume.tar.gz", 3]), "its item 2 is no output's name"),
            (json!(null), "it is not a list"),
        ] {
            let (what, said) = wanting(with("assets", assets.clone()));
            assert_eq!(what, setting("assets"));
            assert!(said.contains(problem), "{assets}: {said}");
        }
    }

    // Requirement: The table reads no other key

    #[test]
    fn a_token_in_the_table_is_unreadable_naming_it_and_the_two_and_never_its_value() {
        let (what, said) = wanting(with("token", json!("ghp_abc")));
        assert_eq!(what, setting("token"));
        assert!(
            said.starts_with(
                "the table names token, which the github_release target does not read"
            ),
            "{said}"
        );
        assert!(!said.contains("ghp_abc"), "{said}");
        assert!(!what.contains("ghp_abc"), "{what}");
        for key in ["repository", "host", "path"] {
            let (what, _) = wanting(with(key, json!("boomdrivedev/flume")));
            assert_eq!(what, setting(key));
        }
    }

    #[test]
    fn settings_that_are_no_table_are_unreadable_naming_the_environment() {
        for settings in [json!("ghp_abc"), json!(["flume-releases"]), json!(null)] {
            let (what, said) = wanting(settings.clone());
            assert_eq!(
                what,
                "the github_release table of environment github of flume"
            );
            assert!(said.starts_with("the settings are no table"), "{said}");
            assert!(!said.contains("ghp_abc"), "{said}");
        }
    }
}
