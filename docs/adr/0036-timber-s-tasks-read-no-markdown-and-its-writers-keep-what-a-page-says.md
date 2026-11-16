---
number: 36
title: timber's tasks read no Markdown, and its writers keep what a page says
status: proposed
date: 2026-11-15
decision-makers:
- Chuck Swanberg
- the code stage of the chore timbers-tasks-read-no-markdown
links:
- target: 30
  kind: amends
---

# timber's tasks read no Markdown, and its writers keep what a page says

Status: proposed. The decision is the user's, `boomdrivedev/blueprints` principle 16 at 3a5c19d, written 2026-11-11 from their words that day and amended 2026-11-15 on their answer. The chore's brief names what goes. The choices below that neither names are the code stage's own, made on the rules written and cheap to change.

Deciders: the user. Their words, on sluice's `root:installs`: "I don't want fucking checks on instructions. That has got to fucking stop. That is my biggest pet peeve. We do not fucking write code tests against Markdown files." Their answer of 2026-11-15: "OpenSpec's own validation stays; the rule is about checks of what pages say". The brief of the chore `timbers-tasks-read-no-markdown`: "timber's tasks read no Markdown file of its own: root:adrs goes, and root:toolchain no longer greps the OpenSpec skills for the pinned openspec's version." The chore's code stage, on the rest.

Sources: `boomdrivedev/blueprints` `design/principles.md` 16 at 3a5c19d; sluice's core.21 as its 4295ca8 amends it; flume's and boomdrive's chores of the same brief, 2026-11-15; ADR 0030; rules core.25, core.26.

## Context and problem statement

Principle 16: no check, test or gate task in any kit reads a Markdown file, a doc, a README, a skill, a site page, an instruction, a spec or a record; a rule about what a page must say is kept by its writers and enforced by no task. A tool's own validation of the files it works on is no check of what a page says. timber's gate read two:

1. `root:adrs`, `check-adrs.sh`, read every record's name and frontmatter and failed two records sharing a number, or a record whose frontmatter number was not its name's.
2. `root:toolchain`, `check-toolchain.sh`, read each OpenSpec skill under `.claude/skills/` and failed one whose `generatedBy` was not the release `devenv.nix` pins, as ADR 0030 decided.

Nothing else the gate runs reads a Markdown file of the tree: treefmt excludes `*.md` and `*.mdx` by name with their reason; `site:build` is Astro's build of the site's pages, the tool's own reading of its files; `conformance:coverage` holds the statement's clauses in `statement.rs` and reads no page; and the tests that write a `README.md` or another Markdown file write it into a scratch repository or a package as data.

## Considered options

For what `root:toolchain` keeps:

1. It keeps `openspec --version` against the pin, a tool the gate runs (core.26), and reads no skill.
2. It reads the skills by another means, such as writing them afresh with the pinned openspec into a scratch directory and comparing.

For a record's number:

1. Its writer keeps it: `adrs new` takes the next number on the tree, and a branch that lands after another took the same number is renumbered by its writer.
2. The check moves to a commit hook, outside the gate.

## Decision outcome

`root:toolchain` is option 1. Option 2 reads the skills just the same, through a comparison rather than a grep; a skill's release is kept by whoever bumps the pin, who writes the skills again in the same commit (ADR 0030's consequence, which stands).

A record's number is option 1. A hook that fails a commit for a page's frontmatter is a check reading Markdown wherever it runs.

ADR 0030 is amended: `root:toolchain` holds openspec to the pin and reads no skill. Its servers, its approval, its place's files and its tracking of the skills and commands stand.

### Consequences

- `root:adrs` and `check-adrs.sh` are gone. Nothing fails two records sharing a number.
- `root:toolchain` says `openspec is <pin>` and fails only while the shell's openspec is another release.
- Rule core.25 rules out a task or a test the gate runs that reads a Markdown file of the tree for what it says, citing principle 16 and this record.
- No spec names either check, so no OpenSpec change carries this.
