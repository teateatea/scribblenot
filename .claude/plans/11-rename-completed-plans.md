# Plan: Rename Completed Task Plans to COMPLETED-*.md in MT-3d Success Path

## Task
#11 - Mission team should rename completed task plans to COMPLETED-*.md on success

## Context
After `pathfinder-mission-team` successfully completes a task, the plan files created during MT-3c remain with their original slug names (e.g. `11-rename-completed-plans.md`). Future agents and `/lets-start` cannot distinguish stale completed plans from active ones without cross-referencing TASKS.md. Renaming each plan to `COMPLETED-<slug>.md` on the success path makes the distinction immediate and unambiguous.

## Approach
Three small edits to `pathfinder-mission-team/SKILL.md`:

1. **MT-3 state declaration** - Add `PLAN_FILES` to the MT-3 state tracking sentence alongside `TASK_QUEUE`, `PRIORITY_MAP`, `D_MAP`, and `COMPLETED_D`.

2. **MT-3c accumulation** - After the Planner subagent returns the plan filename, have the Commander record it. Add one sentence instructing the Commander to append the returned filename to `PLAN_FILES[task]`.

3. **MT-3d step 3 rename** - In the success path (step 3), after rewriting the `Difficulty:` line and before removing the task from TASK_QUEUE, add a step that runs `git mv` for each filename in `PLAN_FILES[task]`, renaming `<slug>` to `COMPLETED-<slug>` in place.

The Commander already receives the Planner's return value (it is the only output), so no new data flows are required - only explicit tracking and a rename action.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - line 109 (MT-3 state declaration), lines 113-115 (MT-3a), lines 172-210 (MT-3c Plan-review loop), lines 279-289 (MT-3d step 3 success path)

## Reuse
- `git -C "<PROJECT_ROOT>" mv` - same pattern used by Implementer subagents for staging; avoids raw filesystem rename so the rename is tracked in git history

## Steps

### Step 1: Add PLAN_FILES to MT-3 state declaration

Locate the MT-3 state sentence (currently: "Maintain a TASK_QUEUE (ordered list), a PRIORITY_MAP ..., a D_MAP ..., and a COMPLETED_D counter ..."). Add `PLAN_FILES` as a new state variable.

```diff
--- a/pathfinder-mission-team/SKILL.md
+++ b/pathfinder-mission-team/SKILL.md
@@ MT-3: Mission loop @@
-Maintain a TASK_QUEUE (ordered list), a PRIORITY_MAP (task -> current priority score, initialized from TASKS.md), a D_MAP (task -> D score, initialized from TASKS.md alongside T in MT-1), and a COMPLETED_D counter (running sum of D scores for all successfully completed tasks, initialized to 0).
+Maintain a TASK_QUEUE (ordered list), a PRIORITY_MAP (task -> current priority score, initialized from TASKS.md), a D_MAP (task -> D score, initialized from TASKS.md alongside T in MT-1), a COMPLETED_D counter (running sum of D scores for all successfully completed tasks, initialized to 0), and a PLAN_FILES map (task -> list of plan filenames produced during MT-3c for that task, initialized to `[]` per task in MT-3a).
```

### Step 2: Initialize PLAN_FILES per task in MT-3a

At the end of MT-3a ("Select the highest-priority unblocked task..."), add one sentence initializing PLAN_FILES for the selected task:

```diff
--- a/pathfinder-mission-team/SKILL.md
+++ b/pathfinder-mission-team/SKILL.md
@@ MT-3a: Pick next task @@
 Select the highest-priority unblocked task (all its dependencies are complete). On a tie, pick the one with the highest difficulty score. If all remaining tasks are blocked by incomplete dependencies, pick the blocked task whose dependencies are furthest along (most sub-tasks complete).
+
+Initialize `PLAN_FILES[task] = []` for the selected task before entering MT-3b.
```

### Step 3: Add PLAN_FILES accumulation in MT-3c Plan-review loop

Locate the Planner subagent block in MT-3c (currently ending with "Return ONLY the plan filename."). Immediately after the closing `>` line of the Planner prompt block, add one sentence instructing the Commander to accumulate the returned filename.

```diff
--- a/pathfinder-mission-team/SKILL.md
+++ b/pathfinder-mission-team/SKILL.md
@@ MT-3c Plan-review loop @@
 > Do NOT use AskUserQuestion.

+After the Planner returns the filename, append it to `PLAN_FILES[task]`. Example: `PLAN_FILES["#11"] = ["11-rename-completed-plans.md", "11-other-subtask.md"]`.
+
 Then run sequential reviewers (up to 3). For each reviewer pass (N=1,2,3), spawn a Sonnet subagent:
```

### Step 4: Add rename step in MT-3d step 3 success path

Locate MT-3d step 3. After the Difficulty rewrite line and before the "Remove task from TASK_QUEUE" line, insert the rename step.

```diff
--- a/pathfinder-mission-team/SKILL.md
+++ b/pathfinder-mission-team/SKILL.md
@@ MT-3d step 3 @@
 3. If passing and no drift:
    - Mark task complete in MISSION-LOG Task Status table.
    - Add `D_MAP[task]` to COMPLETED_D.
    - In the `## Mission` section of MISSION_LOG_PATH, rewrite the `Difficulty:` line to: `- Difficulty: <COMPLETED_D>/<T>` (read the existing `Difficulty:` line to extract T before rewriting)
+   - For each filename `F` in `PLAN_FILES[task]`: if `F` does not already start with `COMPLETED-`, run `git -C "<PROJECT_ROOT>" mv ".claude/plans/<F>" ".claude/plans/COMPLETED-<F>"`. Stage and commit with message: `"Mark task #<N> plans complete: rename <count> plan file(s)"`. Skip any file that does not exist on disk (log a warning to MISSION-LOG).
    - Remove task from TASK_QUEUE.
    - Go to MT-3a.
```

## Verification

### Manual tests
1. Run `/pathfinder-mission-team` on a task that completes successfully. Confirm that all plan files created during that task's MT-3c sub-task loops are now named `COMPLETED-<slug>.md` in `.claude/plans/`.
2. Confirm no plan files are renamed on the failure path (MT-3d step 4).
3. Confirm a plan file that does not exist on disk (e.g. was already manually deleted) produces a warning in MISSION-LOG rather than halting the mission.
4. Run `git log --oneline` in the project root and confirm a commit exists with the message pattern `"Mark task #N plans complete: rename N plan file(s)"`.

### Automated tests
- No automated test runner is applicable to `.md` skill file edits. The manual steps above are the primary verification path.

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | PLAN_FILES map`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | PLAN_FILES[task]`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | COMPLETED-<F>`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Initialize \`PLAN_FILES[task] = []\``

## Progress
- Step 1: Added PLAN_FILES map to MT-3 state declaration sentence (line 109)
- Step 2: Added `Initialize PLAN_FILES[task] = []` sentence at end of MT-3a
- Step 3: Added PLAN_FILES accumulation sentence after Planner returns filename in MT-3c
- Step 4: Added rename step in MT-3d step 3 success path after Difficulty rewrite, before Remove task

## Changelog

### Review - 2026-03-25
- #1 (minor): Added Step 1 to declare `PLAN_FILES` in the MT-3 state tracking sentence (line 109); updated Approach from "Two" to "Three small edits"; added line 109 and MT-3a line range to Critical Files; added `PLAN_FILES map` doc check.
- #2 (nit): Fixed Step 3 diff to use `+` blank line correctly (insert blank then new sentence before existing blank), removing the extra blank that would have produced a double blank line in the output.
- #3 (nit): Renumbered steps to maintain sequential order after adding Step 1.
