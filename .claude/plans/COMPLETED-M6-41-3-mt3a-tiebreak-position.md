## Task

#41 - Fix MT-3a task selection tie-break to use TASK_LIST position (sub-task 3)

## Context

MT-3a currently breaks priority ties by picking "the one with the highest difficulty score." This means that when all tasks share the same priority (the common case - PRIORITY_MAP is initialized to 99 for all tasks unless explicitly annotated), D score determines execution order rather than the premission-established order. Sub-task 41.2 already fixed the same class of bug in MT-2's reorder step by switching to TASK_LIST position. MT-3a needs the same fix for consistency: ties should resolve by position in the original TASK_LIST (earlier = higher priority), not by D score.

## Approach

Change the single tie-break phrase in MT-3a's pick-next-task sentence from "highest difficulty score" to "position in TASK_LIST (earlier = higher priority)". No other changes are needed - the blocked-task fallback clause is unrelated and stays as-is.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - line 145 (MT-3a pick-next-task sentence)

## Reuse

TASK_LIST is already built and ordered in MT-1 step 2 (lines 21-31). No new data structure is needed; MT-3a need only reference TASK_LIST position as the tie-break, the same way MT-2 now does (line 133).

## Steps

1. Edit line 145 of `SKILL.md` to replace the D-score tie-break with TASK_LIST position:

```diff
- Select the highest-priority unblocked task (all its dependencies are complete). On a tie, pick the one with the highest difficulty score. If all remaining tasks are blocked by incomplete dependencies, pick the blocked task whose dependencies are furthest along (most sub-tasks complete).
+ Select the highest-priority unblocked task (all its dependencies are complete). On a tie, pick the one with the earliest position in TASK_LIST (lower index = higher priority). If all remaining tasks are blocked by incomplete dependencies, pick the blocked task whose dependencies are furthest along (most sub-tasks complete).
```

## Verification

### Manual tests

- Invoke `/pathfinder-mission-team` with a BRIEF whose `## Task Priority Order` contains tasks with no `[P:N]` annotation and varying D scores. Confirm the mission log processes tasks in BRIEF order, not descending D-score order.
- Invoke `/pathfinder-mission-team #10 #20 #30` where #30 has a higher D score than #10. Confirm task #10 is attempted before #30 (token-list order wins over D score).

### Automated tests

No automated test harness exists for SKILL.md prose. A realistic option: write a shell script that reads SKILL.md line 145 and asserts it contains "earliest position in TASK_LIST" and does not contain "highest difficulty score".
