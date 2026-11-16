# Tasks

Every command runs from the place's root, inside `devenv shell`. Commit
by pathspec, Conventional Commits scoped to the part of the tree,
citing clauses by number and rules by id. Submit nothing: Flume submits
the place once its last stage delivers.

## 1. The split

- [x] 1.1 Write a delta per spec splitting each requirement over 500 characters, as the design's Decisions 1 to 4 say: its MODIFIED block keeping its name and every scenario word for word, its text 500 characters or fewer, and the text cut away in ADDED requirements with a scenario each. Verify with `devenv shell -- openspec validate timbers-long-requirements-are-split --strict`.

## 2. The gate

- [x] 2.1 Add `check-changes.sh`, boomdrivedev/boomdrive's at d9414da6 whole, and `check-changes-cases.sh`, boomdrivedev/flume's at cdc3ddf3, each with a header citing record 0037 and core.77, and `moon.yml`'s `root:specs`, `root:changes` and `root:changes-cases`, each tagged `check-fast` (design Decision 5). Verify with `devenv shell -- moon run root:changes root:changes-cases`, `root:changes` archiving this change into specs that pass.
- [x] 2.2 Write record 0037, each open change validated as it would archive and timber's specs made to pass `--strict` first, proposed, and rule core.77 under "The gate" in `docs/architecture/core.md` with its details. Verify with `devenv shell -- treefmt --ci` and `devenv shell -- moon run root:format`.

## 3. The archive

- [x] 3.1 Archive the change once groups 1 and 2 are done. Verify with `devenv shell -- moon run root:specs root:changes`, `root:specs` passing all 49 specs and `root:changes` finding no open change.
