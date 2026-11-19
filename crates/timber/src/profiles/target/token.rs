//! A target's token, read here for every target whose tool takes a
//! token only in a variable (record 0033's consequences, records 0043,
//! 0044, rule core.21; the Cachix change's design D6). The host's
//! bindings file's `[credentials]` names the path of the file holding
//! each credential; a target looks its credential's path up with
//! [`path`], opening no file, and reads the file at the moment of use
//! with [`read`] into a [`Token`], held for that operation alone and
//! handed to its tool's runs, written nowhere.
//!
//! Each answer names the credential as the asking target names it, by
//! its [`Credential`]: its name, the environment as the target names it,
//! and what the token is handed to. The targets share this module so
//! that none calls another (rule core.2), and it names no tool and no
//! service of its own.

use std::fmt;
use std::fs;
use std::path::Path;

use timber_seams::{Answer, Not, Source};

use crate::bindings::Credentials;

/// A token, held for one operation and written nowhere: its words in no
/// answer, no argument and no debug print.
#[derive(Clone, PartialEq, Eq)]
pub struct Token(String);

impl Token {
    pub fn new(token: impl Into<String>) -> Token {
        Token(token.into())
    }

    /// The token itself, handed to a run in its tool's variable alone.
    pub fn secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Token(…)")
    }
}

/// A credential a target asks for, as the target names it in its
/// answers.
pub struct Credential<'a> {
    /// The credential's name, as the environment's table gives it.
    pub name: &'a str,
    /// The environment that names it, as the target names it:
    /// `environment github of flume`.
    pub of: &'a dyn fmt::Display,
    /// What the token is handed to: `the github_release target hands gh`.
    pub hands: &'a str,
}

impl Credential<'_> {
    /// The credential as an answer names it, by its environment.
    fn what(&self) -> String {
        format!("credential {} of {}", self.name, self.of)
    }
}

/// The path of the file holding `credential`, as the host's
/// `credentials` name it, the file never opened; unreadable naming the
/// credential and the bindings file where it names none (118, rule
/// core.21).
pub fn path<'c>(
    credentials: &'c Credentials,
    credential: &Credential<'_>,
    source: &Source,
) -> Answer<&'c Path> {
    credentials.path(credential.name).ok_or_else(|| {
        Not::unreadable(
            source,
            credential.what(),
            Some(format!(
                "the bindings file {} names no {} in its [credentials], the path of the file \
                 holding the token {}",
                credentials.file.display(),
                credential.name,
                credential.hands
            )),
        )
    })
}

/// The token the file at `path` holds, read at the moment of use, its
/// trailing whitespace trimmed (64, rule core.21): unreadable naming the
/// credential and the path with the operating system's words where the
/// file cannot be read, and the credential where it holds nothing but
/// whitespace, so nothing runs.
pub fn read(path: &Path, credential: &Credential<'_>, source: &Source) -> Answer<Token> {
    let text = fs::read_to_string(path).map_err(|e| {
        Not::unreadable(
            source,
            format!("credential {} at {}", credential.name, path.display()),
            Some(e.to_string()),
        )
    })?;
    let token = text.trim_end();
    if token.is_empty() {
        return Err(Not::unreadable(
            source,
            credential.what(),
            Some(format!(
                "the file {} holds no token, only whitespace",
                path.display()
            )),
        ));
    }
    Ok(Token::new(token))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use tempfile::TempDir;
    use timber_seams::Seam;

    use super::*;

    const TOKEN: &str = "cachix-token-1f2e3d";
    const BINDINGS: &str = "/home/fleet/.config/timber/config.toml";

    fn source() -> Source {
        Source::new(Seam::Target, "nix_cache")
    }

    fn credential() -> Credential<'static> {
        Credential {
            name: "cachix-boomdrive",
            of: &"environment cachix of boomdrive",
            hands: "the nix_cache target hands cachix",
        }
    }

    fn credentials(named: &[(&str, &Path)]) -> Credentials {
        Credentials {
            file: PathBuf::from(BINDINGS),
            paths: named
                .iter()
                .map(|(name, path)| (name.to_string(), path.to_path_buf()))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    /// What an unreadable answer named and its words.
    fn unreadable<T: fmt::Debug>(answer: Answer<T>) -> (String, String) {
        match answer {
            Err(Not::Unreadable(u)) => {
                assert_eq!(u.source, source());
                (u.what, u.said.expect("the module's words"))
            }
            other => panic!("answered {other:?}"),
        }
    }

    #[test]
    fn a_path_is_looked_up_and_its_file_not_opened() {
        let dir = TempDir::new().unwrap();
        let gone = dir.path().join("no-such-file");
        let host = credentials(&[("cachix-boomdrive", &gone)]);
        assert_eq!(path(&host, &credential(), &source()), Ok(gone.as_path()));
    }

    #[test]
    fn a_credential_the_host_does_not_name_is_unreadable_naming_it_and_the_bindings_file() {
        let (what, said) = unreadable(path(&credentials(&[]), &credential(), &source()));
        assert_eq!(
            what,
            "credential cachix-boomdrive of environment cachix of boomdrive"
        );
        assert_eq!(
            said,
            format!(
                "the bindings file {BINDINGS} names no cachix-boomdrive in its [credentials], \
                 the path of the file holding the token the nix_cache target hands cachix"
            )
        );
    }

    #[test]
    fn a_file_that_cannot_be_read_is_unreadable_naming_the_credential_and_the_path() {
        let dir = TempDir::new().unwrap();
        let gone = dir.path().join("no-such-file");
        let (what, said) = unreadable(read(&gone, &credential(), &source()));
        assert_eq!(
            what,
            format!("credential cachix-boomdrive at {}", gone.display())
        );
        assert_eq!(said, "No such file or directory (os error 2)");
    }

    #[test]
    fn a_file_of_whitespace_alone_is_unreadable_naming_the_credential() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("cachix-boomdrive");
        for held in ["", "\n", " \t\n\n"] {
            fs::write(&file, held).unwrap();
            let (what, said) = unreadable(read(&file, &credential(), &source()));
            assert_eq!(
                what,
                "credential cachix-boomdrive of environment cachix of boomdrive"
            );
            assert_eq!(
                said,
                format!(
                    "the file {} holds no token, only whitespace",
                    file.display()
                )
            );
        }
    }

    #[test]
    fn a_trailing_newline_is_trimmed() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("cachix-boomdrive");
        for held in [
            format!("{TOKEN}\n"),
            format!("{TOKEN} \t\r\n\n"),
            TOKEN.into(),
        ] {
            fs::write(&file, held).unwrap();
            let token = read(&file, &credential(), &source()).unwrap();
            assert_eq!(token.secret(), TOKEN);
        }
    }

    #[test]
    fn the_token_is_in_no_answers_words_nor_its_debug_print() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("cachix-boomdrive");
        fs::write(&file, format!("{TOKEN}\n")).unwrap();
        let token = read(&file, &credential(), &source()).unwrap();
        assert_eq!(format!("{token:?}"), "Token(…)");
        assert!(!format!("{:?}", Ok::<_, Not>(token)).contains(TOKEN));

        let mut not_text = TOKEN.as_bytes().to_vec();
        not_text.extend([0xff, 0xfe, b'\n']);
        fs::write(&file, not_text).unwrap();
        let (what, said) = unreadable(read(&file, &credential(), &source()));
        assert!(!what.contains(TOKEN), "{what}");
        assert!(!said.contains(TOKEN), "{said}");

        let (what, said) = unreadable(read(dir.path(), &credential(), &source()));
        assert!(!format!("{what}{said}").contains(TOKEN), "{what}: {said}");
    }
}
