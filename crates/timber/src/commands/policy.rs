//! `timber policy`: the policy in force at the governed branch's landed
//! head, the one table the givens and the repository's file make, as the
//! policy seam answers it: what acceptance and a layout require, and
//! every channel, the givens beside the repository's own, with how each
//! moves, whether a following one takes prereleases, the claims it
//! requires, its promoters and who may ask for it; and beneath them each
//! given held off, with how it would move and the line that switches it
//! on, from the same channels answer (82, 131, 32, C.10; design D11,
//! D13). Beneath what acceptance and a layout require it says how long
//! cache entries are kept from their push, and whether that is the
//! givens' time or the file's, or the answer that was not one (145). A
//! claim asked of built artifacts alone is shown so (134). It is
//! bound as status is, to the landing and the policy, binds no
//! store and no signer, and writes nothing (60, rule core.32).

use std::io::{self, Write};
use std::path::Path;

use clap::Args;
use timber_core::{in_force, ChannelInForce, InForce, InForceAsk};
use timber_seams::{
    Answer, AskedOf, Branch, HeldChannel, Keeping, Moves, NamedSigner, Not, Required, RequiredClaim,
};

use crate::{render, Binding};

#[derive(Debug, Args)]
pub struct PolicyArgs {
    /// The branch whose landed head holds the policy.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

/// The policy in force asked over `binding` in the repository at `at`:
/// exit 0 whenever an answer is made, each part carrying its own, and the
/// answer's own code where the branch's head cannot be read or has not
/// landed.
pub fn run(args: &PolicyArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = match (&answer, args.json) {
        (_, true) => render::json(out, &answer),
        (Ok(in_force), false) => text(out, in_force),
        (Err(not), false) => render::not(out, "", not),
    };
    render::exit(&answer)
}

fn ask(args: &PolicyArgs, binding: &Answer<Binding>, at: &Path) -> Answer<InForce> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let history = binding.history(at)?;
    in_force(
        &binding.seams,
        &history,
        &InForceAsk {
            branch: Branch::new(&args.branch),
        },
    )
}

fn text(out: &mut dyn Write, in_force: &InForce) -> io::Result<()> {
    writeln!(out, "{} at {}", in_force.branch, in_force.revision)?;
    writeln!(out)?;
    writeln!(out, "acceptance")?;
    required(out, &in_force.acceptance)?;
    writeln!(out, "layout")?;
    required(out, &in_force.layout)?;
    cache(out, &in_force.cache)?;
    writeln!(out)?;
    writeln!(out, "channels")?;
    match &in_force.channels {
        Ok(channels) => {
            for channel in &channels.in_force {
                self::channel(out, channel)?;
            }
            if channels.held.is_empty() {
                return Ok(());
            }
            writeln!(out)?;
            writeln!(out, "held off")?;
            for held in &channels.held {
                self::held(out, held)?;
            }
            Ok(())
        }
        Err(not) => render::not(out, "  ", not),
    }
}

/// 145: how long cache entries are kept from their push, and whether the
/// givens or the file says so; or the answer that was not one (58).
fn cache(out: &mut dyn Write, answer: &Answer<Keeping>) -> io::Result<()> {
    match answer {
        Ok(Keeping { days, given }) => writeln!(
            out,
            "cache entries are kept {days} {} from their push ({})",
            if *days == 1 { "day" } else { "days" },
            if *given { "given" } else { "written" }
        ),
        Err(not) => {
            writeln!(out, "cache entries")?;
            render::not(out, "  ", not)
        }
    }
}

/// How a channel moves, or would once switched on, in the words the
/// channels are shown with.
fn moves(moves: &Moves) -> String {
    match moves {
        Moves::EachPublish { prereleases: false } => "each publish".to_string(),
        Moves::EachPublish { prereleases: true } => "each publish, prereleases too".to_string(),
        Moves::Promotion => "promotion".to_string(),
        Moves::Bound { to, .. } => format!("bound to {to}"),
    }
}

/// One given held off, spelled, how it would move, and the line that
/// switches it on, as the policy words it (C.10).
fn held(out: &mut dyn Write, held: &HeldChannel) -> io::Result<()> {
    writeln!(
        out,
        "  {}, {}, switch on with {}",
        held.channel.channel,
        moves(&held.channel.moves),
        held.switch
    )
}

/// The claims an audience requires, by kind, its check and its signers,
/// in the order the verdict reads them; or its own answer (58).
fn required(out: &mut dyn Write, answer: &Answer<Required>) -> io::Result<()> {
    match answer {
        Ok(required) if required.claims().is_empty() => writeln!(out, "  requires no claim"),
        Ok(required) => {
            for required in required.claims() {
                writeln!(out, "  {}", claim(required))?;
            }
            Ok(())
        }
        Err(not) => render::not(out, "  ", not),
    }
}

/// One channel, spelled, and how it moves; then the claims its policy
/// requires, its promoters and who may ask for it, any asker where it
/// names none, or its policy's own answer (58).
fn channel(out: &mut dyn Write, channel: &ChannelInForce) -> io::Result<()> {
    writeln!(
        out,
        "  {}, {}",
        channel.channel.channel,
        moves(&channel.channel.moves)
    )?;
    match &channel.policy {
        Ok(policy) => {
            let claims: Vec<String> = policy.claims().iter().map(claim).collect();
            if claims.is_empty() {
                writeln!(out, "    requires no claim")?;
            } else {
                writeln!(out, "    requires {}", claims.join(", then "))?;
            }
            writeln!(out, "    promoted by {}", signers(policy.promoted_by()))?;
            let askers = match policy.approvers() {
                [] => "any asker".to_string(),
                approvers => signers(approvers),
            };
            writeln!(out, "    may be asked for by {askers}")
        }
        Err(not) => render::not(out, "    ", not),
    }
}

/// One claim required: its kind, its check where it is a verdict, the
/// signers that may make it, and that it is asked of built artifacts
/// alone where the policy says so, nothing where it is asked of every
/// artifact (134).
fn claim(claim: &RequiredClaim) -> String {
    let check = claim
        .check
        .as_ref()
        .map_or_else(String::new, |c| format!(" of {c}"));
    let asked = match claim.asked_of {
        AskedOf::Every => "",
        AskedOf::Built => ", of built artifacts alone",
    };
    format!(
        "{}{check} by {}{asked}",
        claim.kind.name(),
        signers(&claim.signers)
    )
}

/// Signers by their names, or no signer.
fn signers(signers: &[NamedSigner]) -> String {
    if signers.is_empty() {
        return "no signer".into();
    }
    signers
        .iter()
        .map(|s| s.id.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use timber_core::ChannelsInForce;
    use timber_seams::{Revision, Seam, Source};

    use super::*;

    fn shown(cache: Answer<Keeping>) -> String {
        let in_force = InForce {
            branch: Branch::new("main"),
            revision: Revision::new("4f2a9c1"),
            acceptance: Ok(Required::Acceptance { claims: vec![] }),
            layout: Err(Not::absent(
                &Source::new(Seam::Policy, "policy file"),
                "policy for layout in .config/timber.toml",
            )),
            cache,
            channels: Ok(ChannelsInForce {
                in_force: vec![],
                held: vec![],
            }),
        };
        let mut out = Vec::new();
        text(&mut out, &in_force).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn the_text_says_how_long_cache_entries_are_kept() {
        let given = shown(Ok(Keeping {
            days: 30,
            given: true,
        }));
        let (above, below) = given
            .split_once("cache entries are kept 30 days from their push (given)\n")
            .expect(&given);
        assert!(
            above.contains("acceptance") && above.contains("layout"),
            "{given}"
        );
        assert!(below.contains("channels"), "{given}");

        let written = shown(Ok(Keeping {
            days: 1,
            given: false,
        }));
        assert!(
            written.contains("cache entries are kept 1 day from their push (written)\n"),
            "{written}"
        );

        let not = Not::unreadable(
            &Source::new(Seam::Policy, "policy file"),
            ".config/timber.toml at 4f2a9c1",
            Some("[cache] names keep, and [cache] holds keep_days alone".into()),
        );
        let unread = shown(Err(not.clone()));
        let mut words = Vec::new();
        render::not(&mut words, "  ", &not).unwrap();
        let words = String::from_utf8(words).unwrap();
        assert!(
            unread.contains(&format!("cache entries\n{words}")),
            "{unread}"
        );
    }
}
