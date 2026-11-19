## MODIFIED Requirements

### Requirement: Apply hands the target the verified version

Only once every claim is judged satisfied SHALL the core make the
verified version, the head's index, the version its promotion names, and
the version's tag, its name in the shared history, the component's
prefix followed by the version, and hand it to the target's apply (22,
90, 123).

#### Scenario: The channel's head applied

- **WHEN** the chain of `beta` of `web` has its head at `1.2.0`, every claim its policy requires is held, and the environment runs another version
- **THEN** the target is asked to apply `1.2.0` at its index, after the claims are judged, and the answer names the environment, `beta` of `web`, `1.2.0`, its index and the target's report

#### Scenario: The version's tag handed over

- **WHEN** the component `web` declares the prefix `web-v`, and the target is asked to apply the head `1.2.0`
- **THEN** the verified version it is handed names the tag `web-v1.2.0`
