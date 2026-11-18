## ADDED Requirements

### Requirement: The fake roots bind a store and a keeper per root

The fake roots SHALL answer the root a test binds, or none, and, asked
for a root left, a fake store and a fake keeper of that root's own,
which a test holds handles to, absent for a root the test bound none
for; and SHALL record every asking (92).

#### Scenario: A root left asked

- **WHEN** a test binds `acme/site` and a root left `acme/app` with a store holding `v1.2.0`, and the core asks for `acme/app`
- **THEN** the core reads `v1.2.0` through that store, and the test's handle sees the asking
