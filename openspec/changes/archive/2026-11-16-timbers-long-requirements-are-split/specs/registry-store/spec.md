## MODIFIED Requirements

### Requirement: One declared repository holds the store

The registry store SHALL hold every blob, manifest, attestation and
name of the store in the one repository the host names, addressed by
digest, on every registry, save the three spaces "Three spaces under
the named repository" names (C.1, ADR 0003). It SHALL never create a
repository of its own accord; a registry that creates repositories on
push may create the named one, a component's, an artifact name's and a
cache repository on the first put.

#### Scenario: Everything in one repository

- **WHEN** two artifacts of two ecosystem names, an index of the undeclared component naming both, an attestation on each and a name that is neither an artifact's, a cache entry's nor a declared component's version or channel are put into a registry store
- **THEN** the registry's one named repository holds every one of them, and no other repository is written

#### Scenario: A repository that is not there

- **WHEN** the named repository does not exist on a registry that does not create repositories on push, and bytes are put
- **THEN** the put is refused carrying the registry's own words verbatim and naming the repository, and nothing is written elsewhere

## ADDED Requirements

### Requirement: Three spaces under the named repository

Three spaces SHALL be held under the named repository: a declared
component's versions and channels, tags of its own repository holding
its indexes and what they name; each artifact's name, a tag of its own
repository holding what its index names; and each cache entry, a tag of
its build profile's cache repository holding its bytes (C.1, ADR 0003).

#### Scenario: A component's version, an artifact's name and a cache entry

- **WHEN** a declared component's version, an artifact's name and a cache entry are put into a registry store
- **THEN** the version's tag is held in the component's own repository with its index and what it names, the artifact's name in a repository of its own with what its index names, and the cache entry's tag in its build profile's cache repository with its bytes, each under the named repository
