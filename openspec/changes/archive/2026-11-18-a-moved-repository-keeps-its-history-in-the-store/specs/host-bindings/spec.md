## ADDED Requirements

### Requirement: Every binding answers the root its store is bound for

Every binding SHALL answer the root its store is bound for: for a
registry, the repository the landing's remote names, as the store is
bound for it, its absent or unreadable answer among them; for a layout,
none, since a layout places no name by repository (C.1, C.2).

#### Scenario: A registry's root

- **WHEN** a host bound to a registry publishes from a checkout whose `origin` is `https://github.com/Acme/Site.git`
- **THEN** the binding answers the root `Acme/Site`

#### Scenario: A layout's root

- **WHEN** a host bound to a layout publishes
- **THEN** the binding answers no root

### Requirement: A root left is bound with the host's store and ledger

Asked for a root the repository left, a binding to a registry SHALL
bind a registry store for it under the same registry and root, reading
past a mark naming the bound repository, and the ledger the file names
over it: the git ref ledger as bound, and the store ledger over the
root left's store with its chains kept apart from the bound root's,
under `timber/left/<owner>/<name>` in the common git directory (61, 62).

#### Scenario: A root left with the store ledger

- **WHEN** a host whose file names `[ledger] store` binds the root left `acme/app`
- **THEN** its store reads `<root>/acme/app`, and a chain read through its ledger is kept under `timber/left/acme/app`, leaving the bound root's kept chains as they were

#### Scenario: A root left with the git ref ledger

- **WHEN** a host whose file names no `[ledger]` binds the root left `acme/app`
- **THEN** its ledger reads the repository's own ledger refs
