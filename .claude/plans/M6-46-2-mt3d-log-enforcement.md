# Plan: MT-3d Log-Entry Enforcement Gate

## Task

#46 - Add Prefect-style enforcement gate in MT-3d that validates sub-task log fields

## Context

Sub-task log entries in MISSION-LOG are written in MT-3c step 5. Each entry is expected to have Status, Implementation, Timestamp, and Agent fields. Currently MT-3d marks a task complete without verifying these entries are complete. If any field was accidentally omitted (e.g. a subagent wrote a partial entry), the gap is silently buried in the log. A soft enforcement gate before the "mark complete" step catches this at the earliest recoverable moment and records the gap for post-mortem review, without blocking completion.

## Approach

Insert a new numbered step (step 3.0) inside the MT-3d success branch of `pathfinder-mission-team/SKILL.md`, immediately before the existing "Mark task complete in MISSION-LOG Task Status table" bullet. The step reads the current MISSION_LOG_PATH, finds all sub-task entries for the current task (by scanning for `### Sub-task <N>.` headers), checks each entry for the four required fields (Status, Implementation, Timestamp, Agent), and if any are missing, appends a soft warning to the MISSION-LOG Sub-task Log. Completion is never blocked.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - lines 313-335 (MT-3d success branch)

## Reuse

- The MISSION_LOG_PATH variable is already available in MT-3d scope.
- The task number N is already available (used in the same block for renaming plan files and archiving).
- No new helpers needed; the check is a prose-level procedural instruction for the Mission Commander.

## Steps

1. Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` and locate MT-3d step 3 (the "If passing and no drift:" success branch). The current first bullet under that branch is "Mark task complete in MISSION-LOG Task Status table."

2. Insert a new numbered sub-step **before** "Mark task complete in MISSION-LOG Task Status table." using the diff below:

```diff
 3. If passing and no drift:
+   - **Log-entry enforcement gate.** Read `MISSION_LOG_PATH`. Scan the `## Sub-task Log` section for all entries whose header matches `### Sub-task <N>.<any-id>:` (where N is the current task number). For each such entry, check whether all four required fields are present as `- <Field>:` lines: `Status`, `Implementation`, `Timestamp`, `Agent`. For each entry where one or more fields are absent, append the following block to the `## Sub-task Log` section of `MISSION_LOG_PATH` (after the last existing sub-task entry for this task):
+     ```
+     ### Sub-task <N>.<SUB_ID> enforcement warning
+     - Warning: log entry incomplete — missing field(s): <comma-separated list of missing field names>
+     - Task: #<N>
+     - Timestamp: <current TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S">
+     ```
+     If all entries are complete, skip silently. Do NOT block completion regardless of outcome.
    - Mark task complete in MISSION-LOG Task Status table.
```

## Verification

### Manual tests

1. Run a mission in which a sub-task log entry is deliberately written with one field omitted (e.g. omit `Agent:`). After MT-3d completes for that task, open the MISSION-LOG and confirm a `### Sub-task N.X enforcement warning` block appears in the Sub-task Log listing the missing field.
2. Run a clean mission where all sub-task entries are fully populated. Confirm no enforcement warning blocks appear in the MISSION-LOG.
3. Confirm that in both cases, the task is still marked Complete in the Task Status table (enforcement never blocks completion).

### Automated tests

No automated test runner applies to SKILL.md prose. Coverage can be validated by a grep check:

- After a mission run with a partial log entry, run: `grep "enforcement warning" <MISSION_LOG_PATH>` - should return at least one match.
- After a clean mission run: `grep "enforcement warning" <MISSION_LOG_PATH>` - should return no matches.

## Changelog

### Review - 2026-03-25
- #1: Fixed enforcement warning timestamp format from `%Y-%m-%dT%H:%M:%S%z` to `%Y-%m-%dT%H:%M:%S` to comply with MISSION-6-BRIEF.md requirement that all pathfinder skill timestamps omit the UTC offset suffix.

## Progress
- Step 1: Located MT-3d step 3 success branch in pathfinder-mission-team/SKILL.md
- Step 2: Inserted log-entry enforcement gate bullet before "Mark task complete in MISSION-LOG Task Status table"

## Implementation
Complete - 2026-03-25
