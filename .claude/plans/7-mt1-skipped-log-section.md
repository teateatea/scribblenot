# Plan: MT-1 Skipped Tasks Log Section

## Task
#7 - Update MT-1 mission log write step to include a Skipped Tasks section

## Context

Sub-task 1 already added the premission filtering logic (step 2a) and the Permission Denials write-back (lines 65-70 of SKILL.md). Skipped tasks are currently appended into the existing `## Permission Denials` section with the line:

```
- Task <task>: SKIPPED - <reason> (removed from TASK_LIST at MT-1 initialization)
```

The task asks that skipped tasks be visible in the log **from the start** via a **dedicated `## Skipped Tasks` section**, separate from Permission Denials (which is reserved for hook-blocked tool calls during execution). This makes the two concerns distinct and easier to scan at a glance.

## Approach

Add a `## Skipped Tasks` section to the mission log template written in MT-1 step 5, and update the post-write instruction to append skipped-task entries there instead of (or in addition to) Permission Denials.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - Lines 38-63: mission log markdown template
  - Lines 65-70: post-write SKIPPED_TASKS append instruction

## Reuse

No new utilities needed. Edit in place using the Edit tool.

## Steps

1. Add a `## Skipped Tasks` section to the log template between `## Task Status` and `## Sub-task Log`, and update the post-write instruction to append entries into that section instead of Permission Denials.

```diff
 ## Task Status

 | Task | Priority | Status | Attempts |
 |------|----------|--------|----------|
 | #N   | <P>      | Queued | 0        |

+## Skipped Tasks
+
+(tasks removed by pre-mission check before execution began)
+
 ## Sub-task Log

 (filled per task)
```

2. Update the post-write instruction (lines 65-70) to target the new `## Skipped Tasks` section:

```diff
-   After writing the header, for each entry in SKIPPED_TASKS append to the Permission Denials
-   section:
-
-   ```
-   - Task <task>: SKIPPED - <reason> (removed from TASK_LIST at MT-1 initialization)
-   ```
+   After writing the header, for each entry in SKIPPED_TASKS append to the **Skipped Tasks**
+   section:
+
+   ```
+   - Task <task>: <reason>
+   ```
```

## Verification

### Manual tests

1. Invoke `/pathfinder-mission-team` with a task ID that is NOT listed in `MISSION-PERMISSIONS.json`.
2. Open the resulting `MISSION-LOG-*.md` file.
3. Confirm a `## Skipped Tasks` section exists near the top of the log (after Task Status, before Sub-task Log).
4. Confirm the skipped task appears in that section with its reason.
5. Confirm the `## Permission Denials` section remains empty (not populated with the skipped-task entry).

### Automated tests

None applicable - this is a documentation/template change inside a skill file. No test runner is configured for skill markdown files.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | ## Skipped Tasks`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | (tasks removed by pre-mission check before execution began)`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | append to the Permission Denials`

## Status
Draft

## Progress
- Step 1: Added ## Skipped Tasks section to the log template between ## Task Status and ## Sub-task Log
- Step 2: Updated post-write instruction to append SKIPPED_TASKS entries to Skipped Tasks section instead of Permission Denials

## Changelog

### Review - 2026-03-24
- #1: Fixed typo `premission` -> `pre-mission` in Step 1 diff template text (plan:43)
- #2: Fixed same typo in Doc checks expected string (plan:84) to stay consistent with the template fix
