## MODIFIED Requirements

### Requirement: Each artifact is judged against each required claim in order

For each artifact the index names, in order, the gate SHALL take, in
order, those of the acceptance policy's required claims that the policy
asks of artifacts made the artifact's way (134), each satisfied only by a
claim on the artifact or the index naming the version's revision, whose
maker the signer seam tells is a signer the required claim names, within
that signer's reach (9, 13, 80, 116, core.7).

#### Scenario: Every required claim found

- **WHEN** acceptance requires provenance from `host` and a verdict of `test` from `landing`, and each artifact carries provenance from `host` holding a passing `test` verdict from `landing` for its build
- **THEN** the version is accepted

#### Scenario: A claim from a signer the policy does not name

- **WHEN** acceptance requires a verdict of `test` from `landing`, and the verdict each artifact carries was signed by a signer the policy does not name
- **THEN** the version is refused naming the required verdict and the first artifact, and the unnamed signer's claim stays attached

#### Scenario: A claim beyond its signer's reach

- **WHEN** a signer the policy lets claim only provenance made a verdict of `test`
- **THEN** that verdict does not count, and the version is refused naming the required verdict

#### Scenario: A claim about another revision

- **WHEN** an artifact carries a provenance from the named signer that names a revision other than the version's
- **THEN** it does not count, and the version is refused naming the required provenance

#### Scenario: A claim asked of built artifacts alone

- **WHEN** acceptance requires provenance from `host` of every artifact and a verdict of `test` from `landing` of built artifacts alone, and the version's index names two outputs, each with `host`'s provenance holding a passing `test` verdict for its build, and its notes, with `host`'s provenance naming no build and carrying no verdict
- **THEN** the version is accepted, and the acceptance names the notes' provenance among the claims that satisfied the policy and no verdict of the notes

### Requirement: The gate told by a publish judges the index the publish named

When a publish's answer tells the gate of a version, the telling SHALL
carry the index digest that answer named (95). The gate SHALL resolve
the version's name as it does for any version (core.4) and, where the
name resolves to another digest than the one it was told, SHALL answer
unreadable naming the version's name, the digest it resolves to and the
digest the publish named, before any claim is read or anything is
written.

#### Scenario: Told the digest the name resolves to

- **WHEN** a publish's answer names index `D` for 0.2.0, and 0.2.0's name resolves to `D`
- **THEN** the gate judges `D`, and its answer names `D`

#### Scenario: Told a digest the name no longer resolves to

- **WHEN** a publish's answer names index `D` for 0.2.0, and 0.2.0's name resolves to `E`
- **THEN** the gate answers unreadable naming 0.2.0's name, `E` and `D`; no claim is read, nothing is attached, and no channel moves

## ADDED Requirements

### Requirement: A verdict counts only carried in the artifact's provenance

A verdict SHALL count toward a required claim only carried in the
artifact's provenance, for the check and build required (9, 13).

#### Scenario: A verdict for another build

- **WHEN** acceptance requires a verdict of `test` from `landing`, and an artifact's provenance from `host` holds a passing `test` verdict from `landing` for a build other than the artifact's
- **THEN** that verdict does not count, and the version is refused naming the required verdict and the artifact

### Requirement: A version no publish told of is judged by what its name resolves to

A version the gate is asked about by anyone but a publish telling it
SHALL carry no told digest, and SHALL be judged by what its name
resolves to (95, core.4).

#### Scenario: Asked by a person

- **WHEN** a person asks the gate to judge 0.2.0, whose name resolves to `E`
- **THEN** the gate judges `E`, and its answer names `E`
