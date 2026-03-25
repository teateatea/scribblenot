## Task

#37 - Fix priority direction and replace linear decay with X² cumulative reduction

## Context

The pathfinder mission loop has two related bugs. First, PRIORITY_MAP is initialized with values read from TASKS.md but the skill never specifies the starting value clearly - tasks should start at 99 (high priority) not 0. Second, the failure-decay algorithm reduces priority by 1 on every failure, which is too gentle and produces identical decay regardless of how many consecutive failures a task has accumulated. The correct model is X² cumulative reduction where X is the consecutive-failure count for that specific task: first failure costs 1, second costs 4 (cumulative: 5), third costs 9 (cumulative: 14), etc. X resets to 1 (via clearing CONSECUTIVE_FAILURE_MAP to 0 then incrementing on next failure) when ANY other task successfully completes in between, so the next failure of that task costs only 1² = 1 again. Dependent tasks must receive the same reduction in lockstep.

## Approach

Edit `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` with four targeted changes:

1. Add `CONSECUTIVE_FAILURE_MAP` to the MT-3 state variable list and clarify that PRIORITY_MAP initializes each task at 99 (or the P score from TASKS.md if available).
2. MT-3d failure branch: replace the `-1` reduction with an X² increment-then-reduce, propagate the same reduction to all dependent tasks (floor at 0), and append the prior-attempt record as before.
3. MT-3e blocker handling: apply the same X² reduction in both the permission-denial branch and the implementation-FAILED branch (replacing the two `-1` reductions).
4. MT-3d success branch: when a task completes successfully, reset ALL other tasks' entries in CONSECUTIVE_FAILURE_MAP to 0 (not just the completed task).

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - Line 112: MT-3 state variable list (PRIORITY_MAP initialization, add CONSECUTIVE_FAILURE_MAP)
  - Line 314: MT-3d failure branch (`Reduce PRIORITY_MAP[task] by 1`)
  - Lines 296-312: MT-3d success branch (reset other tasks' X counters)
  - Line 341: MT-3e permission-denial branch (`Reduce PRIORITY_MAP[task] by 1`)
  - Line 348: MT-3e implementation-FAILED branch (`Reduce PRIORITY_MAP[task] by 1`)

## Reuse

No new utilities. All changes are prose edits to the SKILL.md instruction text.

## Steps

1. **Edit MT-3 state variable list (line 112):** Add `CONSECUTIVE_FAILURE_MAP` and clarify PRIORITY_MAP starting values.

```diff
- a PRIORITY_MAP (task -> current priority score, initialized from TASKS.md), a D_MAP
+ a PRIORITY_MAP (task -> current priority score, initialized to 99 per task, or the P score from TASKS.md if a `[P:N ...]` annotation is present), a CONSECUTIVE_FAILURE_MAP (task -> integer X, initialized to 0 per task; tracks consecutive failures without an intervening success), a D_MAP
```

2. **Edit MT-3d success branch (after "Remove task from TASK_QUEUE"):** Reset all other tasks' X counters when a task completes.

After the line `- Remove task from TASK_QUEUE.`, add:

```diff
+  - Reset CONSECUTIVE_FAILURE_MAP for all remaining tasks in TASK_QUEUE to 0 (an intervening success resets the consecutive-failure streak for every other task).
```

3. **Edit MT-3d failure branch (line 314):** Replace linear `-1` with X² reduction and propagate to dependents.

```diff
-  - Reduce `PRIORITY_MAP[task]` by 1 (minimum 0).
-  - Re-queue task behind all higher-priority tasks.
+  - Increment `CONSECUTIVE_FAILURE_MAP[task]` by 1; call the new value X.
+  - Reduce `PRIORITY_MAP[task]` by X² (minimum 0).
+  - For each task in TASK_QUEUE that depends on this task (per the dependency DAG from MT-2), apply the same reduction: `PRIORITY_MAP[dependent]` -= X² (minimum 0).
+  - Re-queue task behind all higher-priority tasks.
```

4. **Edit MT-3e permission-denial branch (line 341):** Replace `-1` with X² reduction.

```diff
- - Reduce `PRIORITY_MAP[task]` by 1.
+ - Increment `CONSECUTIVE_FAILURE_MAP[task]` by 1; call the new value X. Reduce `PRIORITY_MAP[task]` by X² (minimum 0). For each dependent task in TASK_QUEUE, apply the same reduction (minimum 0).
```

5. **Edit MT-3e implementation-FAILED branch (line 348):** Replace `-1` with X² reduction.

```diff
- - Reduce `PRIORITY_MAP[task]` by 1.
+ - Increment `CONSECUTIVE_FAILURE_MAP[task]` by 1; call the new value X. Reduce `PRIORITY_MAP[task]` by X² (minimum 0). For each dependent task in TASK_QUEUE, apply the same reduction (minimum 0).
```

## Verification

### Manual tests

- Read the edited SKILL.md and verify:
  - PRIORITY_MAP description mentions 99 as the default starting value.
  - CONSECUTIVE_FAILURE_MAP is present in the MT-3 state variable list, initialized to 0 per task.
  - MT-3d failure branch increments X, reduces by X², and propagates to dependents.
  - MT-3d success branch resets all other tasks' CONSECUTIVE_FAILURE_MAP entries to 0.
  - MT-3e both branches use the X² formula.
- Trace through a two-task scenario mentally:
  - Task A fails (X=1, reduction=1, new priority=98), task B succeeds (reset A's X to 0), task A fails again (X=1, reduction=1, new priority=97). Confirm decay matches expectation.
  - Task A fails twice consecutively (X=1 then X=2, reductions 1 then 4, net loss=5, new priority=94). Confirm X² behavior.

### Automated tests

The SKILL.md is a prose document with no runnable test harness. Verification is manual only. If a future test framework for skill correctness is added, the priority-decay logic would be a candidate for a unit test covering the X² formula and the cross-task X-reset behavior.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | CONSECUTIVE_FAILURE_MAP`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | initialized to 99`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | X²`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | Reduce \`PRIORITY_MAP[task]\` by 1`

## Progress
- Step 1: Added CONSECUTIVE_FAILURE_MAP to MT-3 state variable list and clarified PRIORITY_MAP starts at 99
- Step 2: Added reset of CONSECUTIVE_FAILURE_MAP for all remaining tasks after a successful task completion
- Step 3: Replaced linear -1 decay in MT-3d failure branch with X² increment-then-reduce and dependent propagation
- Step 4: Replaced -1 in MT-3e permission-denial branch with X² formula and dependent propagation
- Step 5: Replaced -1 in MT-3e implementation-FAILED branch with X² formula and dependent propagation

## Implementation
Complete – 2026-03-25

## Changelog

### Review - 2026-03-25
- #1 (minor): Context section said "X resets to 0" which conflicts with PROJECT-FOUNDATION.md ("X resets to 1"). Clarified to "X resets to 1 (via clearing CONSECUTIVE_FAILURE_MAP to 0 then incrementing on next failure)" to match foundation language while preserving the correct algorithmic explanation.
