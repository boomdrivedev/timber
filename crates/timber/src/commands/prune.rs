//! `timber prune [revision]`: each of the repository's cache entries
//! removed once the keeping time its policy gives at a landed revision
//! has passed since its push, thirty days unless `[cache] keep_days`
//! says otherwise, by default at the local branch's head, as of the
//! host's clock (145). It asks the landing, the policy and the store's
//! cache alone, signs nothing and keeps nothing, and removes nothing but
//! cache entries (54, 61). timber never starts it: what runs it is the
//! operator's, a scheduled workflow of the repository's own, a host's
//! own schedule, or a person (rule core.52, C.16).

use std::io::{self, Write};
use std::path::Path;

use clap::Args;
use timber_core::{prune, Fate, History, PruneAsk, Pruned};
use timber_seams::{Answer, Branch, BuildId, Not};

use crate::commands::status::now;
use crate::{render, Binding};

#[derive(Debug, Args)]
pub struct PruneArgs {
    /// The landed revision whose policy gives the keeping time, any name
    /// git reads for a commit; by default the local branch's head.
    pub revision: Option<String>,
    /// The branch that landed it.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

/// Prune asked over `binding` in the repository at `at`: exit 0 where
/// every entry was answered, the worst of the entries' answers
/// otherwise, and the answer's own code where none was made.
pub fn run(args: &PruneArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = if args.json {
        render::json(out, &answer)
    } else {
        text(out, &answer)
    };
    match &answer {
        Err(not) => render::code(not),
        Ok(pruned) => worst(&pruned.entries),
    }
}

fn ask(args: &PruneArgs, binding: &Answer<Binding>, at: &Path) -> Answer<Pruned> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let history = History::open(at)?;
    let branch = Branch::new(&args.branch);
    let revision = match &args.revision {
        Some(text) => history.resolve(text)?,
        None => history.head(&branch)?,
    };
    prune(
        &binding.seams,
        &PruneAsk {
            branch,
            revision,
            now: now(),
        },
    )
}

/// The worst of the entries' refused and unreadable answers, an entry
/// gone since it was listed counting as an answer (54).
fn worst(entries: &[(BuildId, Answer<Fate>)]) -> i32 {
    render::worst(entries.iter().map(|(_, entry)| match entry {
        Ok(_) | Err(Not::Absent(_)) => render::OK,
        Err(not) => render::code(not),
    }))
}

/// The branch, the revision and the keeping time, then a line per entry:
/// removed with its push moment, gone already, kept until its time
/// passes, or why not.
fn text(out: &mut dyn Write, answer: &Answer<Pruned>) -> io::Result<()> {
    let pruned = match answer {
        Ok(pruned) => pruned,
        Err(not) => return render::not(out, "", not),
    };
    writeln!(out, "{} at {}", pruned.branch, pruned.revision)?;
    let days = pruned.keeping.days;
    writeln!(
        out,
        "cache entries are kept {days} {} from their push ({})",
        if days == 1 { "day" } else { "days" },
        if pruned.keeping.given {
            "given"
        } else {
            "written"
        }
    )?;
    if pruned.entries.is_empty() {
        writeln!(out, "the store holds no cache entry")?;
    }
    for (build, entry) in &pruned.entries {
        match entry {
            Ok(Fate::Removed { pushed: Some(at) }) => {
                writeln!(out, "{build} removed, pushed {}", render::utc(*at))?
            }
            Ok(Fate::Removed { pushed: None }) => {
                writeln!(out, "{build} removed, pushed before push times were kept")?
            }
            Ok(Fate::GoneAlready { .. }) => writeln!(out, "{build} gone already")?,
            Ok(Fate::Kept { pushed, until }) => writeln!(
                out,
                "{build} kept until {}, pushed {}",
                render::utc(*until),
                render::utc(*pushed)
            )?,
            Err(not) => render::not(out, &format!("{build}: "), not)?,
        }
    }
    Ok(())
}
