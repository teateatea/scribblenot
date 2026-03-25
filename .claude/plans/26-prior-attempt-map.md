# Plan: 26-prior-attempt-map.md

## Task
#26 - Provide prior-attempt context to Decomposer on task re-queues

## Context
When a task is re-queued after failing project tests (MT-3d failure branch), the Decomposer re-runs with no memory of what was already attempted. It re-discovers the full task scope from scratch, wastes subagent spawns verifying already-implemented sub-tasks, and may generate the same insufficient sub-task list again. Tracking which sub-tasks ran and which project-test criteria failed lets the Decomposer generate targeted gap-filling sub-tasks on the second pass.

## Approach
Add a `PRIOR_ATTEMPT_MAP` dictionary to the MT-3 state block alongside existing maps (PRIORITY_MAP, D_MAP, etc.). Initialize it empty at MT-3 start. On every MT-3d failure branch, populate an entry keyed by task ID recording the sub-tasks that just ran and the project-test criteria that failed. Pass the map entry into the MT-3b Decomposer prompt so retry passes receive the prior-attempt context automatically.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - Line 109: MT-3 state initialization block (add PRIOR_ATTEMPT_MAP here)
  - Line 117: MT-3a task initialization (add PRIOR_ATTEMPT_MAP[task] = [] here)
  - Lines 120-150: MT-3b Decomposer prompt (add prior-attempt context injection)
  - Lines 291-294: MT-3d failure branch (populate PRIOR_ATTEMPT_MAP here)

## Reuse
- Existing MT-3 state maps (PRIORITY_MAP, D_MAP, COMPLETED_D, PLAN_FILES) as structural model for PRIOR_ATTEMPT_MAP.
- Existing MISSION-LOG append pattern for recording failure context.
- The Decomposer's `sub_tasks` JSON array (already returned) as the source data for the sub-task list stored in PRIOR_ATTEMPT_MAP.

## Steps

1. **Add PRIOR_ATTEMPT_MAP to MT-3 state initialization (line 109)**

```diff
-Maintain a TASK_QUEUE (ordered list), a PRIORITY_MAP (task -> current priority score, initialized from TASKS.md), a D_MAP (task -> D score, initialized from TASKS.md alongside T in MT-1), a COMPLETED_D counter (running sum of D scores for all successfully completed tasks, initialized to 0), and a PLAN_FILES map (task -> list of plan filenames produced during MT-3c for that task, initialized to `[]` per task in MT-3a).
+Maintain a TASK_QUEUE (ordered list), a PRIORITY_MAP (task -> current priority score, initialized from TASKS.md), a D_MAP (task -> D score, initialized from TASKS.md alongside T in MT-1), a COMPLETED_D counter (running sum of D scores for all successfully completed tasks, initialized to 0), a PLAN_FILES map (task -> list of plan filenames produced during MT-3c for that task, initialized to `[]` per task in MT-3a), and a PRIOR_ATTEMPT_MAP (task -> list of prior-attempt records, initialized to `[]` per task; each record is added on the MT-3d failure branch and contains the sub-tasks that ran and the project-test criteria that failed).
```

2. **Add PRIOR_ATTEMPT_MAP initialization to MT-3a (line 117)**

MT-3a already initializes `PLAN_FILES[task] = []` at line 117. Add the parallel initialization for PRIOR_ATTEMPT_MAP on the same line:

```diff
-Initialize `PLAN_FILES[task] = []` for the selected task before entering MT-3b.
+Initialize `PLAN_FILES[task] = []` and `PRIOR_ATTEMPT_MAP[task] = []` for the selected task before entering MT-3b.
```

3. **Capture sub-task list after MT-3b Decomposer returns (end of MT-3b section, before MT-3c begins)**

After the Decomposer subagent returns and before entering MT-3c, store the returned sub-task list in a local variable `CURRENT_SUB_TASKS` so it is available on the failure branch.

The insertion point is immediately after the Decomposer subagent call block (after line 150 in SKILL.md, which ends with `> Do NOT use AskUserQuestion. Return only JSON.`), before the `#### MT-3c` heading.

```diff
 > Do NOT use AskUserQuestion. Return only JSON.
+
+After MT-3b returns the Decomposer JSON, store the `sub_tasks` array as `CURRENT_SUB_TASKS` for use in the MT-3d failure branch.
```

4. **Populate PRIOR_ATTEMPT_MAP on MT-3d failure branch (after lines 291-294)**

On the failure branch of MT-3d (step 4), after reducing priority and re-queuing, append a record to `PRIOR_ATTEMPT_MAP[task]`:

```diff
 4. If failing or drift detected:
    - Reduce `PRIORITY_MAP[task]` by 1 (minimum 0).
    - Re-queue task behind all higher-priority tasks.
    - Log failure context + prevention plan to MISSION-LOG Abandonment Records.
    - Do NOT rename plan files in `PLAN_FILES[task]`; renaming only happens on the success branch.
+   - Append a prior-attempt record to `PRIOR_ATTEMPT_MAP[task]`:
+     ```
+     {
+       "attempt": <length of PRIOR_ATTEMPT_MAP[task] before this append + 1>,
+       "sub_tasks": <CURRENT_SUB_TASKS>,
+       "failed_criteria": <list of project-test criteria that failed, as plain-text strings>
+     }
+     ```
    - Go to MT-3a.
```

5. **Inject prior-attempt context into the MT-3b Decomposer prompt**

Modify the Decomposer subagent prompt to include the prior-attempt block when `PRIOR_ATTEMPT_MAP[task]` is non-empty:

```diff
 > You are the Decomposer for task #<N>. Read:
 > - `<PROJECT_ROOT>/.claude/TASKS.md` (task description)
 > - `<PROJECT_ROOT>/pathfinder/PROJECT-FOUNDATION.md` (if it exists)
 > - Any `DISCUSSION-*.md` or `APPROVED-*.md` files in `<PROJECT_ROOT>/.claude/plans/` that mention task #<N>
 >
+> **Prior attempts (inject only if PRIOR_ATTEMPT_MAP[task] is non-empty):**
+> This task has been attempted <N> time(s) and re-queued due to project-test failures. Do NOT re-implement work that was already completed. Focus exclusively on the gaps identified by the failed criteria below.
+>
+> Prior attempt records:
+> <PRIOR_ATTEMPT_MAP[task] formatted as a numbered list; for each record show attempt number, sub-tasks run (IDs + descriptions), and failed criteria>
+>
 > Break task #<N> into an ordered list of sub-tasks. Group tightly coupled steps...
```

When `PRIOR_ATTEMPT_MAP[task]` is empty (first attempt), omit the prior-attempt block entirely.

## Verification

### Manual tests
- Run a pathfinder mission with a task that is expected to fail project tests on the first attempt, then re-queue. After re-queue, inspect the Decomposer subagent prompt in MISSION-LOG or output to confirm the prior-attempt block appears with the correct sub-tasks and failed criteria.
- Confirm that on the first attempt (no prior failures) the Decomposer prompt contains no prior-attempt block.
- Confirm that on a second re-queue (two prior failures) both prior-attempt records appear in the prompt.

### Automated tests
- No existing test suite covers skill SKILL.md text content; this change is a documentation/prompt change in a Markdown file, so automated test coverage is not applicable.
- If a test harness for prompt construction is added in future, it should assert: given `PRIOR_ATTEMPT_MAP["#5"] = [{attempt:1, sub_tasks:[...], failed_criteria:["..."]}]`, the Decomposer prompt string contains the phrase "Prior attempt records:" and the sub-task descriptions.

## Changelog
- Initial plan written.

## Progress
- Step 1: Added PRIOR_ATTEMPT_MAP to MT-3 state initialization description at line 109
- Step 2: Added PRIOR_ATTEMPT_MAP[task] = [] initialization to MT-3a alongside PLAN_FILES[task] = []
- Step 3: Added CURRENT_SUB_TASKS storage instruction after MT-3b Decomposer returns, before MT-3c heading
- Step 4: Added PRIOR_ATTEMPT_MAP[task] record append to MT-3d failure branch
- Step 5: Injected prior-attempt context block into MT-3b Decomposer prompt (conditional on non-empty map)

### Review - 2026-03-25
- #1 (blocking): Step 2 diff context line was wrong - it targeted line 192 (`After the Planner returns...`) which is inside MT-3c's plan-review loop, not at the end of MT-3b. Fixed to target line 150 (`> Do NOT use AskUserQuestion. Return only JSON.`) which is the true end of the MT-3b Decomposer block, and updated the surrounding prose to clarify placement.

### Prefect Pass 1 - 2026-03-25
- #1 (blocking): Added missing Step 2 diff for PRIOR_ATTEMPT_MAP[task] = [] initialization in MT-3a (SKILL.md:117); updated Critical Files section; renumbered old steps 2/3/4 to 3/4/5.
