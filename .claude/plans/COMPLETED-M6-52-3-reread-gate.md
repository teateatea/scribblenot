## Task
#52 sub-task 3 - Add Re-read enforcement gate in MT-3d step 3

## Context
The MT-3d step 3 log-entry enforcement gate (added by tasks #46, #46-2, #51) currently checks for a missing `Agent` field (hard block) and missing soft fields (`Status`, `Implementation`, `Timestamp`). The `Re-read` field was added to the sub-task log entry format to capture whether a SKILL.md, hook script, or MISSION-PERMISSIONS.json was re-read after editing. There is currently no enforcement ensuring this field is present. A sub-task that edits critical files could skip the re-read and go undetected. This plan adds a third check in the gate: a hard block when `Re-read` is absent from any log entry.

## Approach
Add a new hard-block check between the existing Agent check and the soft-field check. For every matching sub-task log entry, verify that a `- Re-read:` line is present. If absent from any entry, append an enforcement warning block and go to MT-3d step 4 (failure). This mirrors the Agent check pattern exactly. The check does not attempt to parse the value of `Re-read` - it only checks presence. Valid values (`N/A`, `Confirmed: ...`, `Absent`) are all acceptable as long as the field exists; the field being entirely missing from the entry is the hard block condition. This keeps the gate simple and avoids heuristic detection of which files were edited.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`, lines 328-343 (the enforcement gate block inside MT-3d step 3)

## Reuse
- The Agent check pattern at line 328-335 is the direct model. Copy its structure (block appended to Sub-task Log, go to step 4 after processing all entries).

## Steps
1. Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`.
2. Locate the enforcement gate in MT-3d step 3. The gate currently has three checks in this order:
   - Zero-entry check (soft, no block)
   - Per-entry Agent check (hard block)
   - Per-entry soft-field check (no block)
3. After the Agent check block (after line 335, before the soft-field check), insert a new hard-block check for the `Re-read` field:

```diff
       After processing all entries, if any were missing the `Agent` field, treat this as a test failure: go to MT-3d step 4 (do NOT mark the task complete, do NOT rename plan files).
+     - **Per-entry Re-read check (hard block — runs only when Agent check passed):** For each matching entry, check whether the `Re-read` field is present as a `- Re-read:` line. For each entry where the `Re-read` field is absent, append the following block to the `## Sub-task Log` section of `MISSION_LOG_PATH` (after the last existing sub-task entry for this task):
+       ```
+       ### Sub-task <N>.<SUB_ID> enforcement warning
+       - Warning: log entry incomplete — missing required field: Re-read
+       - Task: #<N>
+       - Timestamp: <current TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S">
+       ```
+       After processing all entries, if any were missing the `Re-read` field, treat this as a test failure: go to MT-3d step 4 (do NOT mark the task complete, do NOT rename plan files).
     - **Per-entry soft-field check (runs only when Agent check passed):** ...
```

   Note: the soft-field check's condition label must be updated to read "runs only when Agent check and Re-read check passed":

```diff
-     - **Per-entry soft-field check (runs only when Agent check passed):** For each matching entry, check whether the remaining three fields are present as `- <Field>:` lines: `Status`, `Implementation`, `Timestamp`.
+     - **Per-entry soft-field check (runs only when Agent check and Re-read check passed):** For each matching entry, check whether the remaining three fields are present as `- <Field>:` lines: `Status`, `Implementation`, `Timestamp`.
```

4. Save the file.

## Verification
### Manual tests
- Manually inspect the updated SKILL.md to confirm: (a) the Re-read check block appears between the Agent check block and the soft-field check block; (b) the warning entry format matches the Agent check warning format; (c) "runs only when Agent check and Re-read check passed" appears in the soft-field check label; (d) no double blank lines were introduced.

### Automated tests
- Doc check: verify the updated SKILL.md contains the new field name string.

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | missing required field: Re-read`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Re-read check passed`

## Changelog

### Review - 2026-03-25
- #1: Fixed off-by-one indentation in both diff blocks - all `+`/`-`/context lines corrected from 4-space to 5-space indent (bullet level) and 6-space to 7-space indent (code fence/inner content) to match actual SKILL.md formatting
