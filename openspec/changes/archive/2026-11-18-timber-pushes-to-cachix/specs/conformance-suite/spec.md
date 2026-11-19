## ADDED Requirements

### Requirement: The Nix cache target on Cachix runs the target group

The target group SHALL run over the Nix cache target naming a Cachix
cache, in its own tests with a stand-in `cachix` and `nix-store`,
declaring that its target holds no marker, its foreign settings a
`token` key. It SHALL also run by hand against one cache, under
`check-cachix` (93, 119; rule core.38).

#### Scenario: The stand-in Cachix

- **WHEN** the Nix cache target's tests run
- **THEN** the target group passes over the Nix cache target naming a Cachix cache, and "converged and reported" is reported not applicable
