# serving-process Specification

## Purpose
The host's one serving process, `timber serve`: at the core's one fixed
address, answering each repository a checkout named under its remote's
owner and name with its status page and JSON, every repository the host
knows together at the root, and each served layout under its
repository's path; started by the first asker that finds none, under one
lock (114, 115, 132, core.36).

## Requirements

### Requirement: One process answers a host at one fixed address

`timber serve` SHALL answer HTTP at `127.0.0.1` port `7420`, the
core's name and port, which no setting, flag or variable moves, on
every host (114, core.36). It SHALL answer `GET /serving.json` with a
JSON document naming itself as timber's serving process, its revision
and its process id, asking no repository, so an asker tells it from
anything else on the port.

#### Scenario: The address on every host

- **WHEN** `timber serve` runs on a host
- **THEN** it answers `GET http://127.0.0.1:7420/serving.json` with a document naming timber, its revision and its process id, and no flag or setting is read that would move the address

### Requirement: The repository is in the path, by its remote's owner and name

The process SHALL answer a repository under `/<owner>/<name>/`, its owner
and name the last two parts of the path of the address its landing's
remote has in the repository's git configuration, without a trailing
`.git`, never a directory or a name an operator chose (49, 114). A
remote whose address does not end in an owner and a name SHALL make the
repository unnamed, unreadable naming the remote and its address.

#### Scenario: An HTTPS and an SSH remote

- **WHEN** the landing's remote is `https://example.com/agentplot/timber.git`, and in another checkout `git@example.com:agentplot/timber`
- **THEN** both are named `agentplot/timber`

#### Scenario: A remote that names no owner

- **WHEN** the landing's remote is `/srv/timber.git`
- **THEN** the repository is unreadable naming the remote and that address, and no entry is written

### Requirement: The process answers the repositories a checkout named

The host SHALL keep one file of the repositories named to its serving
process, at `$XDG_STATE_HOME/timber/repositories`, or
`~/.local/state/timber/repositories` where that variable is unset or
empty, readable with any file reader, one entry per repository naming
its owner and name and the path of the checkout that last named it
(62, 132).

#### Scenario: A repository none named

- **WHEN** `GET /agentplot/timber/status.json` is asked and the file names no `agentplot/timber`
- **THEN** the answer is absent, status 404, naming the repository and saying no checkout on this host has named it

#### Scenario: The last checkout to name it

- **WHEN** two checkouts of one repository name it in turn
- **THEN** the file holds one entry for the repository, naming the second checkout

### Requirement: A repository's page and its JSON are its status answer

The process SHALL answer `GET /<owner>/<name>/status.json` with the
status answer the core gives in the named checkout, as `timber
status --json` prints it there, and `GET /<owner>/<name>/status` with a
page rendering that answer whole, as "A repository's page shows its
channels and its record" and "A repository's page shows its
environments" say (57, 132). A path the process does not answer SHALL
be 404, and a method other than `GET` 405.

#### Scenario: The page and the JSON agree

- **WHEN** the page and the JSON of one named repository are asked
- **THEN** the JSON is the status answer in that checkout, and the page names every channel, version and promotion the JSON names, and every environment with its state

#### Scenario: A promotion after the page was read

- **WHEN** a channel is promoted after the page was asked, and the page is asked again
- **THEN** the second page shows the new version, with no restart and no timer between

#### Scenario: The changelog beside the channel

- **WHEN** a channel was promoted to two versions and the repository's page is asked
- **THEN** beneath each promotion the page shows who asked and the notes of the version it moved to, escaped, the same text the JSON carries; and a version whose notes are unreadable shows the unreadable answer with the source's words in their place

#### Scenario: The notes of what this host published

- **WHEN** this host published a version and the repository's page is asked
- **THEN** the page shows that version's notes beneath its row in the record, the same text the JSON carries

#### Scenario: The notes between beside an environment that is behind

- **WHEN** an environment runs 0.2.0 and its channel is at 0.4.0, and the repository's page is asked
- **THEN** beneath the environment the page shows 0.3.0 and 0.4.0 with their notes, escaped, the same text the JSON carries, and the page holds no script

### Requirement: The root answers every repository it knows together

The process SHALL answer `GET /status.json` with every repository the
host's file names, ordered by owner and name, each as the core's row of
its status answer in its checkout, computed anew on the request, and
`GET /` with a page rendering those rows: each repository linked to its
own page, every channel's head, and the environments' four counts or
the answer given in their place (132). The root SHALL write nothing but
the host's copy each status writes, and ask no signer or target (60).

#### Scenario: Two repositories together

- **WHEN** two checkouts of two repositories have each named theirs, and `/status.json` and `/` are asked
- **THEN** the JSON holds two rows, each equal to the core's row of that repository's status but for the moment asked, and the page names both repositories, every channel's head and each row's four counts

#### Scenario: One checkout gone

- **WHEN** one of two named repositories' checkouts has been removed
- **THEN** its row is unreadable naming the path, and the other row is shown

#### Scenario: Nothing named yet

- **WHEN** the host's file is absent and `/status.json` is asked
- **THEN** the answer is an empty list of repositories, status 200

### Requirement: The first asker that finds none starts it, and one lock keeps it one

`timber status --page` SHALL name the checkout's repository in the
host's file, find the serving process at the core's address by its
`/serving.json`, start `timber serve` detached from its terminal and
from the checkout where none answers, wait for it to answer, and print
the address of the repository's own page; a port that answers as
something else SHALL be unreadable naming the address (114). One lock
SHALL keep the process one, as "One lock keeps the host's process one"
says.

#### Scenario: Two askers on one host

- **WHEN** two askers find no process at once and each starts one
- **THEN** one process holds the lock and answers, the other exits saying the first answers, and both askers print the same address answered by one process id

#### Scenario: A second serve

- **WHEN** `timber serve` runs while another holds the lock
- **THEN** it exits 0 naming the address the first answers at, and binds nothing

#### Scenario: Idle

- **WHEN** nothing asks the process for the idle time and no checkout the host's file names holds a served layout
- **THEN** it exits and lets the lock go, and the next asker starts one again

#### Scenario: Idle while a layout is served

- **WHEN** nothing asks the process for the idle time, and the checkout the host's file names for `agentplot/consumer` holds a rendered npm layout
- **THEN** the process still answers, and a request for that layout after the idle time is answered by the same process id

#### Scenario: The served layout removed

- **WHEN** the process stays for a served layout and that layout's directory is then removed, or the checkout holding it is
- **THEN** the process exits at its next idle time and lets the lock go

#### Scenario: A layout of a projection that serves none

- **WHEN** nothing asks the process for the idle time and the only rendered layouts of named checkouts are of projections that serve none
- **THEN** it exits as when no layout is held

#### Scenario: Something else on the port

- **WHEN** the port answers `GET /serving.json` with something other than timber's document
- **THEN** `timber status --page` is unreadable naming the address, and starts nothing

### Requirement: The process serves each served layout under its repository's path

The host's one serving process SHALL answer a request under
`/<owner>/<name>/<ecosystem>/` by handing it, with the base address it
was asked at, to the projection the checkout that last named the
repository binds for that ecosystem, over that checkout's layout of it,
and SHALL answer the projection's response as it is, bytes included
(86, 114, 132). The process SHALL hold no layout of its own and start
no second process (core.36).

#### Scenario: A served layout asked through the process
- **WHEN** a checkout of `agentplot/consumer` has synced its npm layout and named its repository, and `GET /agentplot/consumer/npm/@family%2fwidget` is asked of the process
- **THEN** the answer is the npm projection's response from that checkout's layout, with `application/json`

#### Scenario: A tarball's bytes through the process
- **WHEN** a tarball the layout holds is asked through the process
- **THEN** the response's body is the tarball's bytes exactly, and its length is theirs

#### Scenario: Two checkouts on one host
- **WHEN** two checkouts of one repository on one host each sync, and each one's `npm install` asks the host's address
- **THEN** both are answered by one process, of one process id, and neither checkout started one of its own

#### Scenario: An ecosystem nothing serves
- **WHEN** `GET /agentplot/consumer/file/notes.txt` is asked, the checkout's file projection serving none
- **THEN** the answer is `404` naming the ecosystem, and nothing is rendered

### Requirement: The process reads the host's file on each request

The process SHALL read the host's file of repositories on each request,
SHALL answer a repository no entry names absent, saying no command run
in a checkout of it on this host has named it, and an entry whose
checkout is gone unreadable naming the path (58).

#### Scenario: A named checkout gone

- **WHEN** the file names `agentplot/timber` at a checkout that has since been removed, and `GET /agentplot/timber/status.json` is asked
- **THEN** the answer is unreadable naming the checkout's path

#### Scenario: A repository named while the process runs

- **WHEN** a checkout names its repository after the process started, and its status is then asked of the process
- **THEN** the process answers it, with no restart

### Requirement: A repository's page shows its channels and its record

A repository's page SHALL show each channel with its component, how it
moves, its version or that it is empty, and its chain with each
promotion's asker and the notes of the version it moved to beneath it;
and the record of what this host published, with each version's notes
beneath it; every refusal and unreadable answer shown with the source's
own words (57, 58, 59, 134).

#### Scenario: An unreadable chain on the page

- **WHEN** a repository's status answers one channel's chain unreadable with the ledger's words, and its page is asked
- **THEN** the page shows that channel unreadable with those words, and every other channel with its version and chain

### Requirement: A repository's page shows its environments

A repository's page SHALL show the environments' four counts and every
environment with its state, what it should run, what it runs and when
it converged, what it waits on, and beside one that is behind the notes
of every version between beneath it (98, 132, 134).

#### Scenario: Every environment on the page

- **WHEN** a repository's status answers one environment running what it should and one waiting on a person, and its page is asked
- **THEN** the page shows the four counts, and each environment with its state, what it should run, what it runs and when it converged, and what the waiting one waits on

### Requirement: Each request computes its answer anew and writes nothing but the host's copy

Each request for a repository's page or JSON SHALL compute the answer
anew, and SHALL write nothing but the host's copy of a chain a store
ledger holds and of each environment's reports, reach the store only to
fetch them, and ask no signer or target (60).

#### Scenario: A page asked reaches no signer or target

- **WHEN** a repository's page is asked
- **THEN** the process asks no signer or target anything, and writes nothing but the host's copy of the chains and reports it fetched

### Requirement: The root answers each repository on its own

An entry whose checkout is gone or whose status cannot be asked SHALL
be its own row's unreadable answer, naming the path, and every other row
SHALL be shown (58). A host whose file names no repository SHALL answer
an empty list, and a file that cannot be read SHALL be the root's
unreadable answer naming the file with the parser's words.

#### Scenario: A file of repositories that cannot be read

- **WHEN** the host's file of repositories holds what its parser cannot read, and `/status.json` is asked
- **THEN** the root's answer is unreadable naming the file, with the parser's words

### Requirement: One lock keeps the host's process one

The process SHALL hold the host's one lock, `timber/serve.lock` beside
the file of repositories, for as long as it runs: a `timber serve` that
finds the lock held SHALL exit 0 saying the host's process answers at
the address, and no asker SHALL start a second (114, as sluice 129).

#### Scenario: The lock held while the process runs

- **WHEN** the host's serving process answers
- **THEN** it holds `timber/serve.lock` beside the file of repositories, and lets it go only as it exits

### Requirement: The process ends when idle and no layout is served

The process SHALL end when nothing has asked it for ten minutes and no
checkout the host's file names holds a rendered layout whose projection
serves it; while one does, the process SHALL stay, since the
ecosystem's client cannot start it, and SHALL end at its next idle time
once the layout or its checkout is gone (114, 115). A host may run it
as a service of its own, and nothing SHALL tell a person to start one.

#### Scenario: Asked within the idle time

- **WHEN** the process is asked nine minutes after its last request, then nothing asks it for ten minutes, and no named checkout holds a served layout
- **THEN** it answers the request, and exits ten minutes after it

#### Scenario: No process found

- **WHEN** `timber status --page` finds no process answering
- **THEN** it starts one itself, and tells no person to start one

### Requirement: A version a served layout lacks is rendered in its checkout on the request

A version the layout lacks SHALL be rendered by the core on the
request, in the checkout that last named the repository (115).

#### Scenario: A miss through the process

- **WHEN** a checkout's npm layout lacks `@family/widget` 1.3.0, which the policy admits, and the process is asked for it
- **THEN** the core renders 1.3.0 into that checkout's layout on the request, and the process answers the npm projection's response for it

### Requirement: A request no projection can answer

A repository none named SHALL be answered absent as its status is; an
ecosystem the checkout binds no projection for, or one whose projection
serves none, SHALL be `404` naming the ecosystem; a checkout that cannot
be bound SHALL be its binding's answer (58).

#### Scenario: A layout of a repository none named

- **WHEN** `GET /agentplot/none/npm/x` is asked, and the host's file names no `agentplot/none`
- **THEN** the answer is absent as its status would be, and nothing is rendered
