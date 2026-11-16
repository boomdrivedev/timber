# Design: timber's long requirements are split

## Context

93 requirements of 25 specs run past OpenSpec's 500 characters, and
the family's check (boomdrivedev/boomdrive record 0050), which timber
takes unchanged on the ruling of boomdrive-design of 2026-11-16, fails a
change whose archive writes such a spec. This change splits them so
that every spec passes `openspec validate --strict`, changing nothing
any requirement asks for.

OpenSpec 1.14.3, the release `devenv.nix` pins, read in a scratch tree
on 2026-11-16:

- A MODIFIED block that leaves out a scenario its requirement holds is
  an error of `openspec validate --changes` and is refused by
  `openspec archive` ("archive refuses to drop them").
- One name both REMOVED and ADDED in a delta is an error ("Requirement
  present in both ADDED and REMOVED").
- An ADDED requirement is appended after the spec's last.
- `--strict` fails a requirement with no scenario.

## Decisions

1. **Each long requirement is MODIFIED, keeping its name, its place and
   every scenario word for word.** Its text is cut to 500 characters or
   fewer, at a sentence or a clause, in its own words.
   - Not taken: each long requirement REMOVED and ADDED again in parts
     under new names, its scenarios dealt among them. Every requirement
     would be renamed and moved to the spec's end, and an open change on
     another branch that MODIFIES one by its name would be refused at its
     archive.
2. **The text cut away moves into ADDED requirements, each one
   behaviour in the original's words.** A pronoun whose antecedent stays
   behind is spelled out, and a sentence the original wrote without
   `SHALL` takes it, since `--strict` reads a requirement by it; neither
   changes what is asked. A clause citation stays with the sentence it
   was written beside. Where the cut MODIFIED text no longer names a
   behaviour the original did, it points to the ADDED requirement by
   name.
3. **Each ADDED requirement has a scenario of its own that instances its
   text and asks nothing more.** The scenarios that exercise the moved
   text stay with the MODIFIED requirement, since OpenSpec refuses to
   move them (Context). A new scenario takes its values from the
   original's scenarios where they serve.
   - Not taken: a copy of an original scenario under the ADDED
     requirement. Two copies of one scenario drift apart at the next
     change to either.
4. **Each spec's long requirements are cut in one delta of this one
   change**, so the archive writes each spec once, and `root:changes`
   proves before the archive that every spec it writes passes
   `--strict`.
5. **The gate check is Boomdrive's record 0050 unchanged, and Flume's
   cases with it**, as record 0037 decides.

## Risks

- A new scenario could ask more than its text. Each was written from
  its requirement's text alone and read against the original requirement
  and its scenarios; a word count of each spec's original texts against
  the split texts finds no word of an obligation dropped.
- An open change on another branch that MODIFIES one of these
  requirements with its old text puts the long text back at its archive.
  `root:changes` fails that branch, naming the change and the spec, so
  its writer cuts it as this change does.

## Task notes

- The deltas are under `specs/<capability>/spec.md`, one per spec.
- The archive is `openspec archive timbers-long-requirements-are-split
  --yes`, after `root:changes` passes over the open change.
