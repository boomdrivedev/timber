## ADDED Requirements

### Requirement: The GitHub release target runs the target group

The target group SHALL run over the GitHub release target, in its own
tests against a scripted GitHub, declaring that its target holds a
marker, the release GitHub marks latest. It SHALL also run by hand
against one repository, under `check-github-release` (93, 119, rule
core.38).

#### Scenario: The scripted GitHub

- **WHEN** the GitHub release target's tests run
- **THEN** the target group passes over the GitHub release target with the scripted GitHub
