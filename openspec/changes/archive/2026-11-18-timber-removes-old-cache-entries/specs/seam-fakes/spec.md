## ADDED Requirements

### Requirement: The fake store lists and removes cache entries with when each was pushed

The fake store SHALL stamp each name it makes or moves under `cache/`
with its clock, the system's unless a test sets one, and SHALL let a
test set an entry's moment, or none. It SHALL list and remove cache
entries as the store's contract says, keep the bytes a removed name
named, and record both operations (145, 54).

#### Scenario: A removed entry's bytes

- **WHEN** a test removes a cache entry from the fake store
- **THEN** the name is gone, the bytes it named are still held, and the record names the removal

#### Scenario: A moment a test sets

- **WHEN** a test sets the fake store's clock and a cache entry is then made
- **THEN** the listing answers the entry pushed at that moment

### Requirement: The fake policy answers the keeping time a test sets

The fake policy SHALL answer thirty days, given, unless a test sets a
keeping time for every revision or for one, which it then answers as
written (145).

#### Scenario: A time set for one revision

- **WHEN** a test sets seven days at one revision and the core asks at another
- **THEN** the first is answered seven days written, and the other thirty days given
