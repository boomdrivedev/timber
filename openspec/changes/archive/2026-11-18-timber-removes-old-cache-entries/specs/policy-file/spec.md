## ADDED Requirements

### Requirement: The keeping time is read from `[cache]`, given thirty days

The profile SHALL answer the keeping time from `keep_days` under the
file's `[cache]` table, a whole number of days from 0, as written; and
where the file writes none, or the revision holds no file, the
fragment's `[cache] keep_days = 30`, as given (145, C.10).

#### Scenario: A time written

- **WHEN** the file at the revision holds `[cache]` with `keep_days = 14`
- **THEN** the profile answers fourteen days, written

#### Scenario: No time written

- **WHEN** the file at the revision holds no `[cache]` table
- **THEN** the profile answers thirty days, given

#### Scenario: A time of none

- **WHEN** the file writes `keep_days = 0`
- **THEN** the profile answers zero days, written

### Requirement: A `[cache]` table outside its form is unreadable

A key under `[cache]` other than `keep_days`, and a `keep_days` that is
no whole number from 0, SHALL make the keeping time unreadable naming
the key and the file (58, C.10).

#### Scenario: Another key

- **WHEN** the file's `[cache]` writes `keep = 14`
- **THEN** the keeping time is unreadable naming `keep`

#### Scenario: A time of another form

- **WHEN** the file writes `keep_days = "14d"`, or `keep_days = -1`
- **THEN** the keeping time is unreadable naming `keep_days`
