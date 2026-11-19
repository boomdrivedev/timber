## ADDED Requirements

### Requirement: Prune takes a landed revision and removes what has passed its time

`timber prune [revision] [--branch <name>] [--json]` SHALL ask the core
to prune at the revision, by default the local branch's head, with the
host's clock as the moment. Its text SHALL name the keeping time and
each entry's answer, `--json` SHALL print the answer as one document,
and its exit SHALL be the worst of the entries' answers (145, 57).

#### Scenario: The text of a prune

- **WHEN** prune removes one entry and keeps another
- **THEN** the text names the keeping time, the removed entry with its push moment, and the kept entry with the moment its time passes, and the exit is 0

#### Scenario: An entry that cannot be read

- **WHEN** one entry is answered unreadable and the rest are removed
- **THEN** the exit is unreadable's, and the JSON answer carries each entry's answer

### Requirement: Policy shows how long cache entries are kept

`timber policy` SHALL show the keeping time in force beneath what
acceptance and a layout require, and whether it is given or written,
and its JSON SHALL carry it as `cache` (145, C.10).

#### Scenario: The given time shown

- **WHEN** the policy file writes no `[cache]` table and `timber policy` runs
- **THEN** the text says cache entries are kept thirty days from their push, given
