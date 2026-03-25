# Plan: 12-difficulty-field

## Task
#12 sub-task 1 of 2 - Track cumulative mission difficulty in MISSION-LOG header: initialize Difficulty field at mission start

## Context
The MISSION-LOG `## Mission` block currently records Slug, Date, and Tasks but has no difficulty tracking. Sub-task 1 (this plan) adds the initial `Difficulty: 0/T` line at mission start (MT-1), where T is the sum of D scores for all tasks in TASK_LIST. Sub-task 2 (separate plan) will update the X value after each task completes. This enables post-failure analysis of a suspected context-load ceiling.

## Approach
Edit the MT-1 section of `pathfinder-mission-team/SKILL.md`:
1. Add `- Difficulty: 0/T` to the `## Mission` block template (after the Tasks line).
2. Add a computation step before the file write: parse D scores from TASKS.md for each task in TASK_LIST and sum them to produce T.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` — lines 36-77 (MT-1 template and write step)

## Reuse
- The skill already reads `TASKS.md` at step 2 to get task descriptions and priority/difficulty scores. The D-score parsing for T uses the same read, no new file access needed.

## Steps

**Step 1.** After MT-1 step 2 (where TASKS.md is already read), add a computation note for T.

Unified diff for the instruction block between steps 2 and 3:

```diff
 2. Parse ARGUMENTS into a TASK_LIST (e.g. `#34 #71 #72` or `#34`). Read `<PROJECT_ROOT>/.claude/TASKS.md` to get task descriptions and priority/difficulty scores.
+   Compute T = sum of D scores for all tasks in TASK_LIST (parsed from the `[D:N ...]` annotation in TASKS.md). If a task has no D score annotation, treat its D as 0.

 2a. **Validate task list against premission scope.**
```

**Step 2.** Update the `## Mission` block in the log template to include the Difficulty line after Tasks.

Unified diff for the template block (lines 38-47 of SKILL.md):

```diff
 ## Mission
 - Slug: <MISSION_SLUG>
 - Date: <ISO date>
 - Tasks: <comma-separated list with initial priorities>
+- Difficulty: 0/<T>
```

No other sections of the template change.

## Verification

### Manual tests
1. Run `/pathfinder-mission-team #1` (or any valid task) against the scribblenot project.
2. Open the generated `MISSION-LOG-*.md` file.
3. Confirm the `## Mission` section contains a `- Difficulty: 0/T` line where T equals the sum of D scores for the supplied tasks (e.g. task #1 has D:10, so the line should read `Difficulty: 0/10`).
4. Confirm the line appears after the `- Tasks:` line and before `## Task Status`.

### Automated tests
- Doc check on the SKILL.md file after edit.

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | - Difficulty: 0/<T>`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Compute T = sum of D scores`

## Changelog

### Review - 2026-03-25
- #1: Clarified Task header and Context to declare this as sub-task 1 of 2, noting sub-task 2 (X-value update after each task) is a separate plan - resolves scope ambiguity vs task description
- #2: Corrected Critical Files line range from 36-67 to 36-77 to include the `Record MISSION_LOG_PATH` line at 76

### Prefect Pass 1 - 2026-03-25
- #1 (minor): Changed `0/T` to `0/<T>` in Step 2 diff block and doc-check string to match angle-bracket placeholder convention used throughout the template

## Progress
- Step 1: Added T computation note after MT-1 step 2 in pathfinder-mission-team/SKILL.md
- Step 2: Added `- Difficulty: 0/<T>` line to the ## Mission block template after the Tasks line
