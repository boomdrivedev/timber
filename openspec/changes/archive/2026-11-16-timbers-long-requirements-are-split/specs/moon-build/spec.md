## MODIFIED Requirements

### Requirement: Ensure is one run of the declared tasks at the revision

Ensure SHALL run moon once over the tasks of every output asked, in a
checkout holding exactly the revision asked and nothing of any other
checkout but moon's outputs directory, which it shares with the
checkout the build is bound over, as every checkout on a host shares it,
continuing past a task that fails, and replaying what moon has cached
(74, C.3). timber SHALL never run an output's command itself (15,
core.10).

#### Scenario: Three outputs in one asking

- **WHEN** ensure is asked for three outputs whose tasks pass, pass and fail, each leaving its declared files
- **THEN** moon is run once over the three tasks, and ensure answers the outputs present

#### Scenario: Outputs the build could not make

- **WHEN** ensure is asked for three outputs of which two tasks leave none of their declared files
- **THEN** ensure is refused naming exactly those two outputs, after the third task has run

#### Scenario: The same revision ensured again

- **WHEN** ensure is asked again for outputs already made at the revision
- **THEN** moon answers each task from its cache, and report answers the same build identity as before

#### Scenario: An archive placed in the bound checkout is replayed

- **WHEN** the archive of an output's task hash, made on another host, is placed in the outputs directory of the checkout the build is bound over, and ensure is asked for that output at the revision in a repository where the task has never run
- **THEN** moon answers the task from its cache without running its command, and the output's declared files hold the bytes the other host's build wrote

## ADDED Requirements

### Requirement: Ensure answers the outputs present, or refused naming those it could not make

Ensure SHALL answer the outputs present, or refused naming every output
whose task declared files the run left absent or whose task did not
run (74, C.3).

#### Scenario: An output whose task did not run

- **WHEN** ensure is asked for two outputs, and moon's run leaves the task of one of them not run
- **THEN** ensure is refused naming that output, and not the other
