## ADDED Requirements

### Requirement: A name made at bytes says when it was made

The layout SHALL stamp the index entry of each name it makes or moves
at bytes with `org.opencontainers.image.created`, the moment it was
written, beside its ref name (145, C.2).

#### Scenario: A cache entry's index entry

- **WHEN** a name `cache/b7e1` is made at bytes
- **THEN** its entry in `index.json` carries the ref name and the moment it was made

### Requirement: The layout lists its cache entries and removes a name alone

The layout SHALL list each index entry whose ref name begins `cache/`,
with the digest it names and its moment, none where it has none; and
SHALL remove one such entry under the index's lock, only where it names
the digest given, leaving the bytes on disk (145, 54, 100).

#### Scenario: An entry removed and its bytes kept

- **WHEN** a cache entry is removed at the digest it names
- **THEN** its index entry is gone, the blob is still on disk, and an artifact holding those bytes still reads

#### Scenario: An entry from before

- **WHEN** a layout holds a `cache/` entry with no moment
- **THEN** the listing answers it with none
