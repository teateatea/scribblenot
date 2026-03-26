## Task

#41 Confirm and fix mission-team task execution order to respect premission priority ranking

## Context

Sub-task 4 was scoped to fix the 2-B (token-list) path to initialize PRIORITY_MAP using TASK_LIST position as descending priority values, so insertion order is honoured even when no BRIEF file exists.

Sub-tasks 41.2 and 41.3 already inserted position-based tie-break logic at two decision points:

- MT-2 reorder: "highest priority first, then by position in TASK_LIST (earlier = higher priority) within the same priority tier"
- MT-3a pick-next: "On a tie, pick the one with the earliest position in TASK_LIST (lower index = higher priority)"

When path 2-B is used and TASKS.md contains no `[P:N]` annotations, every task initializes to PRIORITY_MAP = 99. All tasks are equal priority, so both tie-breaks fire and tasks are processed in the exact order the user typed them in the token list -- which is the correct premission order. The fix is already in place.

Artificially encoding position as descending integers (99, 98, 97...) is explicitly unsafe: the X²-decay system reduces PRIORITY_MAP by X² on each failure. A task at position 3 initialized to 97 could drop to 0 after a single failure (X=10 -> 97-100 = 0), while an identically-failing task initialized to 99 survives longer. That asymmetry would corrupt the decay system without solving any real problem.

The edge case where tasks have different `[P:N]` annotations from TASKS.md is also handled correctly: those P values were user-assigned intentional rankings, and the tie-break resolves any remaining equal-priority pairs by TASK_LIST position. No change is needed there either.

## Approach

No code change. Document why sub-task 4 is a no-op so the mission log records the reasoning and the task can be closed.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` (read-only reference; no change)
  - Line 133: MT-2 tie-break (position in TASK_LIST)
  - Line 145: MT-3a tie-break (position in TASK_LIST)
  - Line 139: PRIORITY_MAP initialization definition

## Reuse

No implementation required; nothing to reuse.

## Steps

1. Verify that SKILL.md line 133 contains the MT-2 tie-break phrase "by position in TASK_LIST" and line 145 contains the MT-3a tie-break phrase "earliest position in TASK_LIST". Both should already be present from sub-tasks 41.2 and 41.3.

2. Mark task #41 complete in `.claude/TASKS.md` (change `[ ]` to `[x]`) with an annotation that sub-tasks 41.1-41.4 are done; sub-task 4 is a no-op because the tie-break changes from 41.2 and 41.3 are sufficient and no PRIORITY_MAP initialization change is needed or safe. Note: `#41-4` is an internal decomposition sub-task, not a standalone TASKS.md entry; the parent `#41` is the entry to close.

## Verification

### Manual tests

- Invoke `/pathfinder-mission-team #A #B #C` (three real task IDs, none with `[P:N]` annotations). Confirm in the resulting MISSION-LOG that the first task attempted is #A, not #B or #C. If #A fails and is re-queued, confirm #B is attempted next. This validates that 2-B ordering is honoured through the existing tie-break.

### Automated tests

No automated tests exist for skill execution order. A realistic option would be a shell script that parses MISSION-LOG after a dry-run and asserts the first "Status: Complete" entry matches the first token from the invocation arguments.

## Changelog

### Review - 2026-03-25
- #1: Step 2 referenced `#41-4` as a TASKS.md entry, but that sub-task is not a standalone TASKS.md entry; corrected to mark parent task `#41` complete and added clarifying note that `#41-4` is an internal decomposition sub-task.
