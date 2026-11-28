//! `timber publish <revision>`: a landed revision offered as a version
//! of each component it publishes (14 to 22). It takes no version: the
//! core derives it (21, rule core.13). The landing's cache entries are
//! rendered into the working tree of the checkout it runs in, and a
//! repository with none is absent before the core is asked (135).
//!
//! After the core's publish has answered, the command tells the host's
//! gate of each version it offered where the bindings file names this
//! host's publish as what tells it, or names no `[gate]` (118), in the
//! same run, and prints the gate's answer beside the version's; where
//! the file names a job, it says so and asks the gate nothing (95, 108,
//! 130; design D2, D5). The core's publish reaches nothing of the gate:
//! this step, the host's binding of what tells it, calls one and then
//! the other. It reads the moment of the gate's act from the clock and
//! hands it to the core, which reads none.
//!
//! Before the components, the text says the root the publish recorded,
//! or the history it carried from the root the repository left, as the
//! core's answer carries it (C.1; design D6).

use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Args;
use serde_json::{json, Value};
use timber_core::carry::{ChannelCarried, Rooted, VersionCarried};
use timber_core::{
    accept, answer_json, publish, AcceptAsk, Ask, Judged, Kept, Moved, Publish, Published, ToldBy,
    WhyWanting, LOCAL,
};
use timber_seams::{Answer, Branch, ComponentName, Not, RequiredClaim, Source};

use crate::bindings::Teller;
use crate::{render, unsigned, Binding};

#[derive(Debug, Args)]
pub struct PublishArgs {
    /// The revision to publish, any name git reads for a commit.
    pub revision: String,
    /// The branch that landed it.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// The one component to publish; by default every one the landing
    /// touched.
    #[arg(long)]
    pub component: Option<String>,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

/// The core's publish answer and, for each component in its order, what
/// tells the gate of the version it offered, none where it offered none.
struct Offered {
    publish: Publish,
    gate: Vec<Option<Gate>>,
}

/// What became of telling the gate of one version a publish offered
/// (design D5).
#[derive(Clone, Debug, PartialEq)]
enum Gate {
    /// This host's publish told its own gate, and this is its answer.
    Told(Answer<Judged>),
    /// A job outside timber tells the gate, in the file's words; the
    /// gate was not asked.
    Job(String),
}

impl Gate {
    /// 57: the version's `gate` member, in the core's one form.
    fn json(&self) -> Value {
        match self {
            Gate::Told(answer) => {
                let mut value = answer_json(answer);
                value["told_by"] = json!("publish");
                value
            }
            Gate::Job(words) => json!({ "told_by": "job", "job": words }),
        }
    }

    /// The exit code the gate's part adds: a refusal or a channel held
    /// counts as refused, and a channel's own answer by its kind; a job
    /// adds none, the publish having done what it was asked.
    fn code(&self) -> i32 {
        match self {
            Gate::Told(Err(not)) => render::code(not),
            Gate::Told(Ok(Judged::Refused { .. })) => render::REFUSED,
            Gate::Told(Ok(Judged::Accepted { channels, .. })) => {
                render::worst(channels.iter().map(|(_, moved)| match moved {
                    Ok(Moved::Held { .. }) => render::REFUSED,
                    other => render::exit(other),
                }))
            }
            Gate::Job(_) => render::OK,
        }
    }
}

pub fn run(args: &PublishArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = if args.json {
        json(out, &answer)
    } else {
        text(out, &answer)
    };
    match &answer {
        Ok(offered) => render::worst(
            offered
                .publish
                .components
                .iter()
                .map(|(_, a)| render::exit(a))
                .chain(offered.gate.iter().flatten().map(Gate::code)),
        ),
        Err(not) => render::code(not),
    }
}

fn ask(args: &PublishArgs, binding: &Answer<Binding>, at: &Path) -> Answer<Offered> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let signer = binding.signer.clone().ok_or_else(unsigned)?;
    let history = binding.history(at)?;
    let tree = history.workdir().ok_or_else(|| {
        Not::absent(
            &Source::history(LOCAL),
            format!(
                "working tree at {}, which the landing's cache entries are rendered into",
                at.display()
            ),
        )
    })?;
    let ask = Ask {
        branch: Branch::new(&args.branch),
        revision: history.resolve(&args.revision)?,
        component: args.component.as_ref().map(ComponentName::new),
        signer,
        store: binding.store.clone(),
        tree: tree.display().to_string(),
    };
    let publish = publish(&binding.seams, &history, &Kept::at(&history), &ask)?;
    // Only once the publish has answered: the gate, told of each version
    // it offered, where this host's publish is what tells it (130).
    let gate = publish
        .components
        .iter()
        .map(|(component, answer)| {
            let Ok(Published::Version { version, index, .. }) = answer else {
                return None;
            };
            Some(match &binding.gate {
                Teller::Publish => Gate::Told(accept(
                    &binding.seams,
                    &history,
                    &AcceptAsk {
                        branch: ask.branch.clone(),
                        component: Some(component.clone()),
                        version: version.clone(),
                        told: ToldBy::Publish {
                            index: index.clone(),
                        },
                        gate: ask.signer.clone(),
                        when: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map_or(0, |d| d.as_secs()),
                    },
                )),
                Teller::Job(words) => Gate::Job(words.clone()),
            })
        })
        .collect();
    Ok(Offered { publish, gate })
}

/// The answer as one JSON document: the publish's, in the core's one
/// form, each offered version's entry with its `gate` member (57).
fn json(out: &mut dyn Write, answer: &Answer<Offered>) -> io::Result<()> {
    let value = match answer {
        Ok(offered) => {
            let mut value = answer_json::<&Publish>(&Ok(&offered.publish));
            for (i, gate) in offered.gate.iter().enumerate() {
                if let Some(gate) = gate {
                    value["value"]["components"][i]["gate"] = gate.json();
                }
            }
            value
        }
        Err(not) => answer_json::<()>(&Err(not.clone())),
    };
    serde_json::to_writer_pretty(&mut *out, &value)?;
    writeln!(out)
}

/// A required claim as a person names it: its kind and its check.
fn wanting(wanting: &RequiredClaim, why: &WhyWanting) -> String {
    let claim = match &wanting.check {
        Some(check) => format!("{} of {check}", wanting.kind),
        None => wanting.kind.to_string(),
    };
    let why = match why {
        WhyWanting::Missing => "no claim attached counts for it".to_string(),
        WhyWanting::Expired { signer } => format!("{signer}'s identity has expired"),
        WhyWanting::OfferedByGate { signer } => format!(
            "its provenance is {signer}'s, the gate's own identity, which the policy does not let accept too"
        ),
        WhyWanting::Unmatched { signer } => format!(
            "the rebuild by {signer} is the offerer's or did not match"
        ),
    };
    format!("{claim}: {why}")
}

/// The gate's part of one offered version, below the version's lines.
fn gate(out: &mut dyn Write, version: &str, gate: &Gate) -> io::Result<()> {
    match gate {
        Gate::Told(Err(not)) => {
            writeln!(out, "  gate:")?;
            render::not(out, "    ", not)
        }
        Gate::Told(Ok(Judged::Refused {
            artifact,
            wanting: claim,
            why,
            ..
        })) => writeln!(
            out,
            "  gate: refused, {version} stays offered: {artifact} lacks {}",
            wanting(claim, why)
        ),
        Gate::Told(Ok(Judged::Accepted {
            acceptance,
            already,
            one_host,
            channels,
            ..
        })) => {
            write!(out, "  gate: accepted, acceptance {acceptance}")?;
            if *one_host {
                write!(out, ", one host offered and accepted")?;
            }
            if *already {
                write!(out, ", accepted already")?;
            }
            writeln!(out)?;
            for (channel, moved) in channels {
                match moved {
                    Ok(Moved::Promoted {
                        from,
                        to,
                        promotion,
                        ..
                    }) => writeln!(
                        out,
                        "    {channel} promoted from {} to {to}: claim {promotion}",
                        from.as_ref()
                            .map_or_else(|| "nothing".to_string(), ToString::to_string)
                    )?,
                    Ok(Moved::There { at }) => writeln!(out, "    {channel} at {at} already")?,
                    Ok(Moved::Held {
                        artifact,
                        wanting: claim,
                        why,
                    }) => writeln!(
                        out,
                        "    {channel} held: {artifact} lacks {}",
                        wanting(claim, why)
                    )?,
                    Err(not) => {
                        writeln!(out, "    {channel}:")?;
                        render::not(out, "      ", not)?;
                    }
                }
            }
            Ok(())
        }
        Gate::Job(words) => writeln!(
            out,
            "  gate: {version} is offered, and the gate is told by \"{words}\""
        ),
    }
}

/// The root a publish recorded, or the history it carried, before its
/// components (C.1; design D6): a line per version the root left did not
/// hold and per channel whose name the carry made.
fn root(out: &mut dyn Write, rooted: &Rooted) -> io::Result<()> {
    let carried = match rooted {
        Rooted::Recorded { root } => return writeln!(out, "recorded the root {root}"),
        Rooted::Carried(carried) => carried,
    };
    let versions = carried
        .versions
        .iter()
        .filter(|v| v.carried != VersionCarried::NotHeld)
        .count();
    writeln!(
        out,
        "carried {}'s history to {}: {versions} versions, {} channels",
        carried.from,
        carried.to,
        carried.channels.len()
    )?;
    for version in &carried.versions {
        if version.carried == VersionCarried::NotHeld {
            writeln!(
                out,
                "  {} passed over: {} does not hold it",
                version.name, carried.from
            )?;
        }
    }
    for channel in &carried.channels {
        if channel.name == ChannelCarried::Made {
            writeln!(
                out,
                "  {} made at {}, its head {}",
                channel.channel, channel.index, channel.head
            )?;
        }
    }
    Ok(())
}

fn text(out: &mut dyn Write, answer: &Answer<Offered>) -> io::Result<()> {
    let offered = match answer {
        Ok(offered) => offered,
        Err(not) => return render::not(out, "", not),
    };
    let publish = &offered.publish;
    if let Some(rooted) = &publish.rooted {
        root(out, rooted)?;
    }
    if publish.components.is_empty() {
        writeln!(out, "no component was touched since its last release")?;
    }
    for ((component, answer), told) in publish.components.iter().zip(&offered.gate) {
        match answer {
            Ok(Published::Version {
                version,
                tag,
                revision,
                index,
                artifacts,
                notes,
                ..
            }) => {
                writeln!(
                    out,
                    "{component} {version} ({tag}) from {revision}: index {index}"
                )?;
                for artifact in artifacts {
                    writeln!(
                        out,
                        "  {} {} {}, named {}, from {} by build {}",
                        artifact.ecosystem,
                        artifact.name,
                        artifact.digest,
                        artifact.store_name,
                        artifact.output,
                        artifact.build
                    )?;
                    for claim in &artifact.claims {
                        writeln!(
                            out,
                            "    {} by {}, envelope {}",
                            claim.kind, claim.signer, claim.envelope
                        )?;
                    }
                }
                if let Some(notes) = notes {
                    writeln!(out, "  notes {}", notes.digest)?;
                    for claim in &notes.claims {
                        writeln!(
                            out,
                            "    {} by {}, envelope {}",
                            claim.kind, claim.signer, claim.envelope
                        )?;
                    }
                    for line in notes.text.lines() {
                        writeln!(out, "{}", format!("    {line}").trim_end())?;
                    }
                }
                if let Some(told) = told {
                    gate(out, version.as_str(), told)?;
                }
            }
            Ok(Published::Unchanged { last, .. }) => match last {
                Some(last) => writeln!(out, "{component} unchanged since {last}")?,
                None => writeln!(out, "{component} unchanged, and never released")?,
            },
            Err(not) => {
                writeln!(out, "{component}:")?;
                render::not(out, "  ", not)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use timber_core::carry::{Carried, CarriedChannel, CarriedVersion, Marked};
    use timber_seams::{ChannelId, ChannelName, Digest, Name, RepositoryName};

    use super::*;

    fn root(owner: &str, name: &str) -> RepositoryName {
        RepositoryName {
            owner: owner.into(),
            name: name.into(),
        }
    }

    fn printed(rooted: Rooted) -> String {
        let offered = Offered {
            publish: Publish {
                rooted: Some(rooted),
                components: vec![],
            },
            gate: vec![],
        };
        let mut out = Vec::new();
        text(&mut out, &Ok(offered)).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn the_text_says_the_root_recorded() {
        let text = printed(Rooted::Recorded {
            root: root("acme", "site"),
        });
        assert_eq!(
            text,
            "recorded the root acme/site\nno component was touched since its last release\n"
        );
    }

    #[test]
    fn the_text_says_the_history_carried() {
        let (head, index) = (Digest::of(b"the head"), Digest::of(b"v1.2.0's index"));
        let beta = ChannelId::new(None, ChannelName::new("beta"));
        let stable = ChannelId::new(None, ChannelName::new("stable"));
        let text = printed(Rooted::Carried(Carried {
            from: root("acme", "app"),
            to: root("acme", "site"),
            mark: Marked::Made,
            versions: vec![
                CarriedVersion {
                    name: Name::new("v1.0.0"),
                    carried: VersionCarried::NotHeld,
                },
                CarriedVersion {
                    name: Name::new("v1.1.0"),
                    carried: VersionCarried::HeldAlready,
                },
                CarriedVersion {
                    name: Name::new("v1.2.0"),
                    carried: VersionCarried::Copied,
                },
            ],
            channels: vec![
                CarriedChannel {
                    channel: beta,
                    head: head.clone(),
                    index: index.clone(),
                    name: ChannelCarried::Made,
                },
                CarriedChannel {
                    channel: stable,
                    head: head.clone(),
                    index: index.clone(),
                    name: ChannelCarried::HeldAlready,
                },
            ],
        }));
        assert_eq!(
            text,
            format!(
                "carried acme/app's history to acme/site: 2 versions, 2 channels\n\
                 \x20 v1.0.0 passed over: acme/app does not hold it\n\
                 \x20 beta made at {index}, its head {head}\n\
                 no component was touched since its last release\n"
            )
        );
    }
}
