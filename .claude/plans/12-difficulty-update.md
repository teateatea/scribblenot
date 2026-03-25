## Task
#12 - Track cumulative mission difficulty in MISSION-LOG header, updated after each task

## Context
MISSION-LOG already has a `Difficulty: 0/<T>` line written at MT-1 initialization, but `pathfinder-mission-team` never updates it as tasks complete. This plan adds the update step to MT-3d so that after each successful task completion the `X` value is incremented by the completed task's D score, giving post-failure visibility into a suspected ~200-point context ceiling.

## Approach
Edit `pathfinder-mission-team/SKILL.md`: in MT-3d step 3 (the "if passing and no drift" branch), after marking the task complete in the Task Status table, add a step to rewrite the `Difficulty: X/<T>` line in the `## Mission` section with the updated cumulative D sum.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` — lines 274-286 (MT-3d section)

## Reuse
No external utilities. The Mission Commander already tracks `PRIORITY_MAP` and the per-task D scores parsed from TASKS.md; the running sum can be maintained as a `COMPLETED_D` counter in the same loop state.

## Steps

**1. Add COMPLETED_D tracking to the mission loop state**

At the top of MT-3 (line ~108), add `COMPLETED_D` to the maintained state description:

```diff
-Maintain a TASK_QUEUE (ordered list) and a PRIORITY_MAP (task -> current priority score, initialized from TASKS.md).
+Maintain a TASK_QUEUE (ordered list), a PRIORITY_MAP (task -> current priority score, initialized from TASKS.md), and a COMPLETED_D counter (running sum of D scores for all successfully completed tasks, initialized to 0).
```

**2. Add difficulty update step to MT-3d step 3**

Current MT-3d step 3 (lines 279-281):
```
3. If passing and no drift:
   - Mark task complete in MISSION-LOG Task Status table.
   - Remove task from TASK_QUEUE.
   - Go to MT-3a.
```

Replace with:
```diff
-3. If passing and no drift:
-   - Mark task complete in MISSION-LOG Task Status table.
-   - Remove task from TASK_QUEUE.
-   - Go to MT-3a.
+3. If passing and no drift:
+   - Mark task complete in MISSION-LOG Task Status table.
+   - Add the task's D score to COMPLETED_D.
+   - In the `## Mission` section of MISSION_LOG_PATH, rewrite the `Difficulty:` line to: `- Difficulty: <COMPLETED_D>/<T>` (read the existing `Difficulty:` line to extract T before rewriting)
+   - Remove task from TASK_QUEUE.
+   - Go to MT-3a.
```

## Verification

### Manual tests
1. Run `/pathfinder-mission-team` with two tasks that have known D scores (e.g. `#11 #14`, D=30 and D=35).
2. After the first task completes, open the MISSION-LOG and confirm the `Difficulty:` line has updated from `0/65` to `30/65` (or whichever task completed first).
3. After the second task completes, confirm it reads `65/65`.
4. If a task is re-queued (MT-3d step 4 path), confirm the `Difficulty:` line does NOT change.

### Automated tests
No automated test runner exists for skill `.md` files. A doc check covers the structural change:

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Add the task's D score to COMPLETED_D`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | COMPLETED_D counter`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Difficulty: <COMPLETED_D>/<T>`

## Prefect Report

### Pass 2 - 2026-03-25

**minor** `12-difficulty-update.md:23-24` (Step 1 diff) / `12-difficulty-update.md:45` (Step 2 diff) - D_MAP not established in state tracking, leaving D score lookup undefined in MT-3d.

Step 1 adds `COMPLETED_D counter` to maintained state at MT-3 (SKILL.md line 108), but does not add a `D_MAP (task -> D score)`. The individual D scores are only computed transiently in MT-1 step 2 to derive T; no named per-task structure persists them into the loop. When Step 2's new MT-3d instruction says "Add the task's D score to COMPLETED_D", the implementer has no specified source from which to retrieve that score. The `PRIORITY_MAP` analogy in the Reuse section implies a parallel map exists, but the current SKILL.md text has no such structure.

Fix: extend Step 1's `+` line to also initialize a `D_MAP`:

```
12-difficulty-update.md:24
-+Maintain a TASK_QUEUE (ordered list), a PRIORITY_MAP (task -> current priority score, initialized from TASKS.md), and a COMPLETED_D counter (running sum of D scores for all successfully completed tasks, initialized to 0).
++Maintain a TASK_QUEUE (ordered list), a PRIORITY_MAP (task -> current priority score, initialized from TASKS.md), a D_MAP (task -> D score, initialized from TASKS.md alongside T in MT-1), and a COMPLETED_D counter (running sum of D scores for all successfully completed tasks, initialized to 0).
```

And update Step 2's new bullet accordingly:

```
12-difficulty-update.md:45
-   - Add the task's D score to COMPLETED_D.
+   - Add `D_MAP[task]` to COMPLETED_D.
```

## Changelog

### Review - 2026-03-25
- #1 (nit): Updated manual test example from out-of-scope tasks `#1 #11` (D=10, D=30) to in-mission tasks `#11 #14` (D=30, D=35) to avoid referencing a task outside this mission's scope per PROJECT-FOUNDATION.md non-goals.

### Prefect Pass 1 - 2026-03-25
- #1 (minor): Added parenthetical to Step 2 diff `+` line clarifying that T must be read from the existing `Difficulty:` line before rewriting (`12-difficulty-update.md:46`).

## Progress
- Step 1: Added D_MAP and COMPLETED_D to MT-3 state tracking line in pathfinder-mission-team/SKILL.md
- Step 2: Added D_MAP[task] to COMPLETED_D and Difficulty line rewrite to MT-3d step 3 in pathfinder-mission-team/SKILL.md
