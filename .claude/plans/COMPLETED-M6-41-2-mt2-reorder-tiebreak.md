## Task

#41 - Fix MT-2 Dependency Scout prompt and reorder step (sub-task 2: MT-2 reorder tie-break)

## Context

MT-2 currently reorders TASK_QUEUE after the Dependency Scout returns the DAG using the rule: "highest priority first, hardest (highest D score) first within the same priority tier." This tie-break by D score discards the premission order established in the BRIEF's `## Task Priority Order` section (or the token-list insertion order when no BRIEF is present). Because PRIORITY_MAP is initialized to 99 for all tasks (no [P:N] annotations), every task lands in the same priority tier and D-score ordering takes over entirely, overriding intentional premission ordering. The fix changes the tie-break from D score to TASK_LIST insertion order (the order tasks were extracted in MT-1 step 2, either from the BRIEF or from the token list).

## Approach

Change the single reorder instruction line in MT-2 so that within a priority tier, tasks are ordered by their position in TASK_LIST (lower index = higher priority) rather than by D score. No changes to the Dependency Scout prompt are needed; the DAG output is correct. No changes to PRIORITY_MAP initialization or MT-3a are in scope for this sub-task.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - line 133 (the reorder instruction)

## Reuse

TASK_LIST is already built and ordered in MT-1 step 2 (lines 21-31). The reorder step need only reference TASK_LIST position as the tie-break source; no new data structure is required.

## Steps

1. Edit line 133 of `SKILL.md` to change the reorder tie-break from D score to TASK_LIST insertion order:

```diff
- Reorder the TASK_QUEUE based on the DAG: highest priority first, hardest (highest D score) first within the same priority tier, respecting dependency ordering.
+ Reorder the TASK_QUEUE based on the DAG: highest priority first, then by position in TASK_LIST (earlier = higher priority) within the same priority tier, respecting dependency ordering.
```

## Verification

### Manual tests

- Invoke `/pathfinder-mission-team MISSION-6-BRIEF` (or any BRIEF with a `## Task Priority Order` section listing tasks with no `[P:N]` annotation). Confirm the mission log's `## Task Status` table lists tasks in BRIEF order rather than sorted by D score.
- Invoke `/pathfinder-mission-team #34 #71 #72` (token list). Confirm the task execution order in the log matches token-list order (#34 first, then #71, then #72) when priorities are equal, modulo dependency constraints.

### Automated tests

No automated test harness exists for SKILL.md prose. A realistic option: write a shell script that parses the SKILL.md line 133 and asserts it contains "position in TASK_LIST" and does not contain "highest D score first within the same priority tier".

## Changelog

### Review - 2026-03-25
- #1: Added `diff` language tag to the Steps code block for consistent syntax highlighting with other M6 plans.

## Progress

- Step 1: Edited SKILL.md line 133 to replace D-score tie-break with TASK_LIST position tie-break.

## Implementation
Complete - 2026-03-25
