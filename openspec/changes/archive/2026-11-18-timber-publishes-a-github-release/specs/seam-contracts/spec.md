## MODIFIED Requirements

### Requirement: A target's contract

A target SHALL answer desired with the digest an environment's channel
names, or absent when the channel is empty (88), given the head the core
read from the channel's chain, which a target that is pushed to answers
and a target that pulls answers by its own reading (44, rule core.50);
current with the digest the environment runs, or unreadable with the
reason (89); and apply, given a verified version, its index, version and
tag, with the environment converged or what stopped it (22, 90).

#### Scenario: An environment that cannot be read

- **WHEN** current is asked of an environment whose state cannot be read
- **THEN** the answer is unreadable with the reason, never a digest

#### Scenario: A push target answers the head it is handed

- **WHEN** desired is asked of a target that is pushed to, with a head that answers a digest
- **THEN** it answers that digest, and with a head that answers the channel empty it answers absent

#### Scenario: A target that holds no marker

- **WHEN** a version is applied to an environment whose target can hold no marker, and current and report are asked
- **THEN** each answers unreadable with the reason, and neither names the digest applied

#### Scenario: The verified version names its tag

- **WHEN** the fake target is asked to apply a verified version made with the index `A`, the version `1.2.0` and the tag `v1.2.0`
- **THEN** it records `A`, `1.2.0` and `v1.2.0`, and the verified version round-trips through JSON with all three
