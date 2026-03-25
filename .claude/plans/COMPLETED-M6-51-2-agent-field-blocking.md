## Task
#51 - Upgrade MT-3d per-entry field check so missing Agent field is a hard block

## Context
MT-3d's per-entry field check currently treats all missing fields (Status, Implementation, Timestamp, Agent) as soft warnings: it logs a warning entry but never blocks task completion. The Agent field identifies who performed work and is considered mandatory for audit integrity. A log entry with no Agent field should halt task completion and re-queue the task, the same way a failed project test does. All other missing fields (Status, Implementation, Timestamp) should retain the existing soft-warning behavior.

## Approach
Split the per-entry field check in MT-3d step 3 into two passes:
1. A hard-block pass that checks only for the Agent field and triggers the MT-3d step 4 failure branch if any entry is missing it.
2. A soft-warning pass (existing behavior) for the remaining fields (Status, Implementation, Timestamp), which only runs when no Agent-field violation was found.

This keeps the change minimal: one targeted split of the existing check, no new data structures, no new functions.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` lines 326-333 (the per-entry field check block inside MT-3d step 3)

## Reuse
- The existing MT-3d step 4 failure branch (lines 352-367 of SKILL.md) is reused as-is for the hard-block path; no new failure logic is needed.

## Steps
1. Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`.
2. Locate the **Per-entry field check** block (currently lines 326-333). Replace it with the following text (exact replacement shown as a diff):

```
-     - **Per-entry field check (runs only when at least one entry was found):** For each matching entry, check whether all four required fields are present as `- <Field>:` lines: `Status`, `Implementation`, `Timestamp`, `Agent`. For each entry where one or more fields are absent, append the following block to the `## Sub-task Log` section of `MISSION_LOG_PATH` (after the last existing sub-task entry for this task):
-       ```
-       ### Sub-task <N>.<SUB_ID> enforcement warning
-       - Warning: log entry incomplete — missing field(s): <comma-separated list of missing field names>
-       - Task: #<N>
-       - Timestamp: <current TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S">
-       ```
-     If all entries are complete (and at least one entry exists), skip the per-entry warnings silently. Do NOT block completion regardless of outcome.
+     - **Per-entry Agent check (hard block — runs only when at least one entry was found):** For each matching entry, check whether the `Agent` field is present as a `- Agent:` line. For each entry where the `Agent` field is absent, append the following block to the `## Sub-task Log` section of `MISSION_LOG_PATH` (after the last existing sub-task entry for this task):
+       ```
+       ### Sub-task <N>.<SUB_ID> enforcement warning
+       - Warning: log entry incomplete — missing required field: Agent
+       - Task: #<N>
+       - Timestamp: <current TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S">
+       ```
+       After processing all entries, if any were missing the `Agent` field, treat this as a test failure: go to MT-3d step 4 (do NOT mark the task complete, do NOT rename plan files).
+     - **Per-entry soft-field check (runs only when Agent check passed):** For each matching entry, check whether the remaining three fields are present as `- <Field>:` lines: `Status`, `Implementation`, `Timestamp`. For each entry where one or more of these fields are absent, append the following block to the `## Sub-task Log` section of `MISSION_LOG_PATH` (after the last existing sub-task entry for this task):
+       ```
+       ### Sub-task <N>.<SUB_ID> enforcement warning
+       - Warning: log entry incomplete — missing field(s): <comma-separated list of missing field names>
+       - Task: #<N>
+       - Timestamp: <current TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S">
+       ```
+     If all entries are complete (and at least one entry exists), skip the per-entry warnings silently. Do NOT block completion based on soft-field warnings.
```

## Verification
### Manual tests
- Run a mission task through MT-3d where a sub-task log entry is intentionally missing the `Agent` field. Confirm the task is re-queued (step 4 branch) rather than marked complete.
- Run a mission task through MT-3d where a sub-task log entry is intentionally missing `Status` only. Confirm a soft warning is appended but the task is marked complete.
- Run a mission task through MT-3d where all four fields are present. Confirm no warning is appended and the task is marked complete normally.

### Automated tests
- No automated test harness exists for SKILL.md prose. The three manual scenarios above cover the critical branches. A future test script could parse MISSION-LOG entries and assert presence/absence of enforcement warning blocks.

## Changelog

### Review - 2026-03-25
- #1: Clarified Agent-check phrasing from "If any entry is missing... append" to "For each entry where absent, append... After processing all entries, if any were missing, go to step 4" — eliminates ambiguity about whether one or multiple per-entry warnings are logged before failing.

## Progress
- Step 1-2: Replaced per-entry field check block in SKILL.md lines 326-333 with two-pass check: hard-block Agent check and soft-warning check for Status/Implementation/Timestamp
