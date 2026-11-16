## MODIFIED Requirements

### Requirement: The branch's landing group passes, and S13 against it

The conformance suite's landing group SHALL pass over the branch
landing twice, with no remote bound and with one, its scenario of
verdicts as the landing recorded them named not applicable in both,
since the branch records none, and its scenario of a landing read as
last fetched not applicable with no remote, since the repository is
then the landing itself (93, C.6). S13 SHALL play against it as "S13
plays over the branch landing and the moon build" says.

#### Scenario: The landing group over a branch

- **WHEN** the suite's landing group runs over a rig binding the branch landing with no remote
- **THEN** the report names the profile `branch`, names verdicts as the landing recorded them and a landing read as last fetched not applicable with their reasons, and every other scenario passed

#### Scenario: The landing group over a branch on a remote

- **WHEN** the suite's landing group runs over a rig binding the branch landing to a remote
- **THEN** the report names the profile `branch`, names verdicts as the landing recorded them not applicable with its reason, and every other scenario, a landing read as last fetched among them, passed

#### Scenario: S13 over a plain branch and moon

- **WHEN** a scratch repository's branch, which no landing gate runs, holds a revision whose output moon makes and passes, and the revision is published and then verified under each of two policies
- **THEN** the publish offers the version with no verdict carried; verify under the policy requiring a verdict reports no verdict counted, and under the policy requiring provenance alone reports the provenance counted

## ADDED Requirements

### Requirement: S13 plays over the branch landing and the moon build

S13 SHALL play over the branch landing and the moon build: a revision
on a branch sluice does not run is published with no verdict carried;
verify under a policy that requires a verdict counts none, and under
one that requires provenance alone counts the provenance (S13, 15,
111).

#### Scenario: S13 published with no verdict

- **WHEN** S13 plays over a rig binding the branch landing and the moon build, and publishes a revision of a branch no landing gate runs
- **THEN** the version is offered with no verdict carried; verify under a policy requiring a verdict counts none, and under one requiring provenance alone counts the provenance
