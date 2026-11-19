## MODIFIED Requirements

### Requirement: The file names the credentials a target's tool reads

The file SHALL name each credential a target needs in `[credentials]`
(118): each key a credential's name, as an environment's target table
gives it, and each value the path of a file holding only that
credential, absolute or beginning with `~/`, expanded as the file's
other paths are. The file SHALL never hold a credential, and the binding
SHALL open no file a path names: a target reaches it at use (45, 64,
rule core.21; record 0043). A file with no `[credentials]` SHALL name
none.

#### Scenario: A credential named by its path

- **WHEN** the file names `[credentials] dash0-dashboards = "~/.local/share/secrets/dash0-dashboards"`, and `timber apply` runs
- **THEN** the targets are handed `dash0-dashboards` with that path expanded against the home directory, and the binding does not open the file at that path

#### Scenario: No credentials table

- **WHEN** the file names no `[credentials]`, and `timber publish` runs
- **THEN** it is bound as before, and the targets are handed no credential
