## MODIFIED Requirements

### Requirement: Opening a row shows that repository whole

Opening a row SHALL show the repository's own answer as the process
gives it at its path: the branch and the landed revision the policy was
read at, and how the landing was read; each channel with its version or
that it is empty, and the chain of promotions that put it there, newest
first; its environments, as "An opened row shows every environment"
says; and what this host published. A key SHALL return to every
repository.

#### Scenario: A row opened and closed

- **WHEN** the person opens the row of `o/r`
- **THEN** the pane shows `o/r`'s branch, landed revision, each channel with its version and its chain newest first (the version moved from, who asked, when, and the claim), each environment and what this host published, and the back key returns to every repository

#### Scenario: A repository whose checkout is gone

- **WHEN** the person opens the row of a repository the process answers absent at its own path
- **THEN** the pane shows the absence with the process's words, and the back key returns to every repository

#### Scenario: An environment behind

- **WHEN** the person opens a repository whose answer holds `srv` behind, running 0.2.0 where its channel is at 0.4.0, with the notes of 0.3.0 and 0.4.0
- **THEN** the pane shows `srv` behind, what it runs and since when, what it should run, and both versions' notes beneath it, as the answer gives them

#### Scenario: An environment that cannot be read

- **WHEN** the person opens a repository whose answer holds `desk` unread, saying no counted report names it
- **THEN** the pane shows `desk` as cannot be read with those words, and neither as running what it should nor as behind

## ADDED Requirements

### Requirement: An opened row shows every environment

An opened row SHALL show the environments' four counts and every
environment with its state, what it should run, what it runs and since
when or why that cannot be read, what it waits on, and beside one that
is behind the notes of every version between, as the repository's
answer gives them.

#### Scenario: The environments of an opened row

- **WHEN** the person opens the row of `o/r`, whose answer holds `srv` running what it should and `prod` waiting on a person
- **THEN** the pane shows the four counts, `srv` with its state, what it should run and what it runs and since when, and `prod` with its state and what it waits on
