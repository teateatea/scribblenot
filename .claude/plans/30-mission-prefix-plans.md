## Task

#30 - Prefix pathfinder plan filenames with mission number (e.g. M5-20-1-slug.md)

## Context

When multiple missions run over time (or a mission is interrupted), plan files from different missions coexist in `.claude/plans/`. Currently plans are named `<task-N>-<slug>.md` with no mission identifier, making it impossible to tell at a glance which mission produced which plan. The fix is to prepend `M<MISSION_NUMBER>-` to every plan filename the mission team creates.

MISSION_NUMBER is already computed in MT-1 step 4: it is the `N` in `MISSION-LOG-<N>-<MISSION_SLUG>.md`. It must be passed into the MT-3c Planner prompt so each plan filename carries the mission number.

The COMPLETED- rename step in MT-3d must also be updated so that `COMPLETED-M5-30-1-slug.md` (not `COMPLETED-30-1-slug.md`) is the resulting name when a file already prefixed with `M<N>-` is renamed.

## Approach

1. In MT-1, after writing the log file and recording `MISSION_LOG_PATH`, explicitly define `MISSION_NUMBER = N` (the integer derived in step 4) as a named variable for use throughout the loop.
2. In the MT-3c Planner prompt, change the plan filename instruction from `N-<slug>.md` (where N is the task number) to `M<MISSION_NUMBER>-N-<slug>.md`. Pass `MISSION_NUMBER` into the prompt as a substituted value.
3. In MT-3d, the COMPLETED- rename logic already checks `if F does not already start with COMPLETED-` -- no extra prefix logic is needed because the rename simply prepends `COMPLETED-` to whatever `F` already is, preserving the `M<N>-` prefix in the result (e.g. `COMPLETED-M5-30-1-slug.md`). No change needed in the rename logic itself.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - MT-1 step 4 (line 36): where N is derived from the glob; add `MISSION_NUMBER = N` assignment
  - MT-3c Planner prompt (lines 196-199): filename instruction uses `N-<slug>.md`

## Reuse

- The existing `N` variable computed in MT-1 step 4 via `Glob ... MISSION-LOG-*-*.md` is the MISSION_NUMBER -- no new computation required.

## Steps

1. In MT-1 step 4 of `SKILL.md`, after the sentence ending "...so first log is `MISSION-LOG-1-<slug>.md`)", add a line that explicitly records the mission number:

```diff
-4. Glob `<PROJECT_ROOT>/pathfinder/MISSION-LOG-*-*.md` to find existing logs. Use N+1 where N is the highest number found (default N=0, so first log is `MISSION-LOG-1-<slug>.md`).
+4. Glob `<PROJECT_ROOT>/pathfinder/MISSION-LOG-*-*.md` to find existing logs. Use N+1 where N is the highest number found (default N=0, so first log is `MISSION-LOG-1-<slug>.md`). Record `MISSION_NUMBER = N+1` (the integer used in the log filename).
```

2. In MT-3c, update the Planner prompt's filename instruction (step 4 of the Planner's steps):

```diff
-> 4. Write the plan to `<PROJECT_ROOT>/.claude/plans/<three-word-slug>.md` (verify slug does not already exist using Glob). Prefix with task number: e.g. `N-<slug>.md`.
+> 4. Write the plan to `<PROJECT_ROOT>/.claude/plans/<three-word-slug>.md` (verify slug does not already exist using Glob). Prefix with mission number and task number: e.g. `M<MISSION_NUMBER>-N-<slug>.md`.
```

   Replace `<MISSION_NUMBER>` in the injected prompt with the actual value of `MISSION_NUMBER` at spawn time (the same substitution pattern already used for `<PROJECT_ROOT>`, `<N>`, `<SUB_ID>`, etc.).

3. No change is needed in MT-3d. The rename logic prepends `COMPLETED-` to `F` verbatim. A plan named `M6-30-1-slug.md` becomes `COMPLETED-M6-30-1-slug.md`, which is the desired format.

## Verification

### Manual tests

- Run `/pathfinder-mission-team` on any task.
- After the Planner subagent completes, check `.claude/plans/` and confirm the new plan file is named `M<N>-<task>-<slug>.md` where `N` matches the number in the active `MISSION-LOG-<N>-*.md` filename.
- After the task completes, confirm the plan is renamed to `COMPLETED-M<N>-<task>-<slug>.md`.

### Automated tests

- No automated tests exist for SKILL.md content; verification is manual (above).
- Optional: a shell script that globs `.claude/plans/` after a test mission run and asserts all non-COMPLETED plan files match the pattern `M[0-9]+-[0-9]+-*.md`.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | MISSION_NUMBER = N+1`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | M<MISSION_NUMBER>-N-<slug>.md`

## Changelog

### Review - 2026-03-25
- #1 (nit): Replaced `<task-N>` with bare `N` in the Planner filename example (`M<MISSION_NUMBER>-N-<slug>.md`) to match the existing SKILL.md convention where the task number token is unbracketed (e.g. `N-<slug>.md`). Updated Approach section and Doc checks to match.

## Progress
- Step 1: Added `MISSION_NUMBER = N+1` assignment to MT-1 step 4 in pathfinder-mission-team SKILL.md
- Step 2: Updated MT-3c Planner prompt step 4 filename instruction to use `M<MISSION_NUMBER>-N-<slug>.md`
