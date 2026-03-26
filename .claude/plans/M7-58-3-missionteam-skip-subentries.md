## Task

#58 - Resolve collision between TASKS.md sub-entry format (#N-2) and pathfinder sub-task nomenclature

## Context

TASKS.md uses indented sub-entries like `#34-2` and `#72-2` as supplementary context records under a parent task. The pathfinder-mission-team skill reads TASKS.md at two points where a sub-entry could accidentally enter TASK_LIST: (1) MT-1 step 2-A extracts lines matching `- #<N>` from a BRIEF file's `## Task Priority Order` section -- if a BRIEF were incorrectly authored to include `#34-2`, it would be extracted as a task ID; (2) MT-2 Dependency Scout receives TASK_LIST and reads TASKS.md to build a DAG -- if a sub-entry ID were already in TASK_LIST, the Scout might treat it as a real task node. Adding explicit guards at both points ensures sub-entries can never enter or persist in TASK_LIST regardless of how a BRIEF is authored.

## Approach

Add a prose filter clause to the MT-1 2-A extraction instruction so that any line whose task ID token matches `#\d+-\d+` (a digit sequence, a hyphen, another digit sequence) is silently skipped during extraction. Add a one-sentence note to the MT-2 Dependency Scout prompt instructing it to ignore any task ID in TASK_LIST that contains a hyphen (e.g. `#34-2`) and to exclude such IDs from the DAG entirely. Both changes are prose additions -- no regex code, no new data structures.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - Line 27: MT-1 2-A extraction instruction (`Extract every line matching - #<N>`)
  - Lines 121-135: MT-2 Dependency Scout prompt block

## Reuse

No utilities to reuse. Both changes are editorial additions to existing prose instructions.

## Steps

1. **MT-1 2-A: add sub-entry filter.** In `SKILL.md` at line 27, the current instruction reads:

   > Extract every line matching `- #<N>` (one task ID per line, in file order).

   Change it to:

   > Extract every line matching `- #<N>` (one task ID per line, in file order). Skip any line whose task ID token matches the pattern `#\d+-\d+` (a hyphen followed by one or more digits after the task number, e.g. `#34-2`) -- these are TASKS.md sub-entries and must never be added to TASK_LIST.

2. **MT-2 Dependency Scout: add sub-entry exclusion note.** In `SKILL.md` at line 135, the current last line of the Scout prompt reads:

   > If no dependencies exist, return an empty `depends_on` array for each task. Do NOT use AskUserQuestion. Return only JSON.

   Change it to:

   > If no dependencies exist, return an empty `depends_on` array for each task. If any task ID in the provided list contains a hyphen followed by digits (e.g. `#34-2`), exclude it from the DAG entirely -- it is a TASKS.md sub-entry, not a real task. Do NOT use AskUserQuestion. Return only JSON.

## Verification

### Manual tests

- Open a correctly authored BRIEF file and confirm that `#34-2` or `#72-2` do not appear in its `## Task Priority Order` section (normal case -- filter is never triggered).
- Manually insert `- #34-2` into a test BRIEF's `## Task Priority Order` section, invoke `/pathfinder-mission-team MISSION-X-BRIEF`, and confirm that `#34-2` is absent from TASK_LIST in the mission log header and that the mission proceeds with only the remaining valid task IDs.
- Pass `#34-2` as part of TASK_LIST to a manual MT-2 run (by temporarily listing it in a BRIEF) and confirm the Dependency Scout's JSON output contains no entry for `#34-2`.

### Automated tests

Both changes are doc-only edits to a SKILL.md prose file. No executable code is modified, so no automated test runner applies. A doc-check (below) verifies the exact strings are present after the edit.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Skip any line whose task ID token matches the pattern`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | it is a TASKS.md sub-entry, not a real task`
