## Task

#45 - Update MT-3c plan-review loop to maintain REVIEWER_COUNT and PREFECT_COUNT accumulators per sub-task

## Context

MT-3c step 5 instructs the Mission Commander to log `Reviewers: <N>` and `Prefects: <N>` counts for each sub-task, but the skill never tells the Commander *when* to initialize or increment those counters. The Commander must infer the values after the fact, which leads to errors when early-exit paths, conditional passes, or retry loops are involved. Making the accumulator lifecycle explicit (init before the loop, increment at each spawn site) eliminates ambiguity and ensures the step-5 log values are always correct.

## Approach

Add one sentence initializing REVIEWER_COUNT = 0 and PREFECT_COUNT = 0 immediately after the Planner returns (before the reviewer loop starts in MT-3c step 2), then add an increment instruction immediately after each reviewer subagent spawn and each prefect subagent spawn. The existing step-5 prose that says "use the total count..." is then updated to reference the accumulators directly instead of re-describing the counting logic.

No new state variables are introduced at the MT-3 level (these are per-sub-task counters, reset at the start of each plan-review loop iteration). No other steps are affected.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - Line 215: "Then run sequential reviewers (up to 3)." - reviewer loop starts here
  - Line 236: "After reviewers (or early exit), run 2 Prefect passes." - prefect section starts here
  - Line 250: "If `PREFECT FIXED`: run **Prefect Pass 2**" - conditional second prefect
  - Line 262: retry reviewer loop and Prefect Pass 3
  - Line 291: step 5 log write with `Reviewers: <N>` and `Prefects: <N>` fields

## Reuse

No existing utilities. This is a pure prose/instruction edit to SKILL.md.

## Steps

1. In MT-3c step 2 ("Plan-review loop"), immediately after the sentence "After the Planner returns the filename, append it to `PLAN_FILES[task]`." and before "Then run sequential reviewers (up to 3).", insert:

   ```
   Initialize REVIEWER_COUNT = 0 and PREFECT_COUNT = 0 for this sub-task.
   ```

2. After the reviewer subagent prompt block (lines 217-233) and before the early-exit rule sentence, insert an increment instruction. The location is after the closing `>` of the reviewer prompt and before "Early-exit rule:":

   ```
   After each reviewer subagent completes (regardless of output), increment REVIEWER_COUNT by 1.
   ```

3. After the Prefect Pass 1 subagent prompt block and before "If `PREFECT FIXED`:", insert:

   ```
   After Prefect Pass 1 completes, increment PREFECT_COUNT by 1.
   ```

4. After the Prefect Pass 2 subagent prompt block and before "If `PREFECT ISSUES` after Pass 2:", insert:

   ```
   After Prefect Pass 2 completes, increment PREFECT_COUNT by 1.
   ```

5. In the retry reviewer loop sentence (currently: "run up to 2 additional reviewer passes (retry reviewer loop) using the same reviewer subagent prompt"), add after the existing description of the retry loop:

   ```
   Increment REVIEWER_COUNT by 1 after each retry reviewer pass completes.
   ```

6. After the Prefect Pass 3 instruction (currently ends with "proceed to implementation" or "log the remaining issues...then proceed"), insert:

   ```
   After Prefect Pass 3 completes, increment PREFECT_COUNT by 1.
   ```

7. In step 5, replace the parenthetical descriptions of how to compute Reviewers and Prefects:

   - Old text (starting at "For `Reviewers: <N>`"):
     ```
     For `Reviewers: <N>`: use the total count of reviewer subagent passes that ran (1-3 initial plus any retry passes; always >= 1 because the plan-review loop runs unconditionally regardless of test_runner). For `Prefects: <N>`: use 0 if skipped, otherwise 1, 2, or 3 (the number of prefect passes that ran):
     ```
   - New text:
     ```
     For `Reviewers: <N>`: use REVIEWER_COUNT. For `Prefects: <N>`: use PREFECT_COUNT.
     ```

## Verification

### Manual tests

- Read the updated SKILL.md and trace through a non-destructive sub-task scenario where Reviewer 1 returns `RECOMMENDS APPROVAL`: confirm REVIEWER_COUNT reaches 1, PREFECT_COUNT reaches 1 (Pass 1 only), and step 5 logs `Reviewers: 1` / `Prefects: 1`.
- Trace a destructive sub-task where all 3 reviewers run, Pass 1 returns `PREFECT FIXED`, Pass 2 returns `PREFECT ISSUES`, 2 retry reviewers run, Pass 3 returns `PREFECT APPROVAL`: confirm REVIEWER_COUNT = 5, PREFECT_COUNT = 3.

### Automated tests

- Doc checks verify the key strings are present after the edit.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Initialize REVIEWER_COUNT = 0 and PREFECT_COUNT = 0 for this sub-task.`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | increment REVIEWER_COUNT by 1`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | increment PREFECT_COUNT by 1`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | use REVIEWER_COUNT`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | use PREFECT_COUNT`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | 1-3 initial plus any retry passes`

## Changelog

### Review - 2026-03-25
- #1: Fixed Approach section - "two sentences" corrected to "one sentence" and "before the Planner spawn" corrected to "after the Planner returns (before the reviewer loop starts)" to match the Steps section

## Progress
- Step 1: Inserted "Initialize REVIEWER_COUNT = 0 and PREFECT_COUNT = 0 for this sub-task." after PLAN_FILES append line and before reviewer loop
- Step 2: Inserted "After each reviewer subagent completes (regardless of output), increment REVIEWER_COUNT by 1." before Early-exit rule
- Step 3: Inserted "After Prefect Pass 1 completes, increment PREFECT_COUNT by 1." before "If PREFECT FIXED"
- Step 4: Inserted "After Prefect Pass 2 completes, increment PREFECT_COUNT by 1." before "If PREFECT ISSUES after Pass 2"
- Step 5: Added "Increment REVIEWER_COUNT by 1 after each retry reviewer pass completes." to retry loop sentence
- Step 6: Appended "After Prefect Pass 3 completes, increment PREFECT_COUNT by 1." to Pass 3 sentence
- Step 7: Replaced verbose Reviewers/Prefects counting prose with "use REVIEWER_COUNT" / "use PREFECT_COUNT"
