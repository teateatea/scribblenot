## Task
#55 - Track premission duration and show estimate before committing to session

## Context
The pathfinder-premission skill has no timing instrumentation. Users running a large task list can spend 20-30 minutes in premission setup only to discover mid-flow that they lack the time to complete the session. Two additions address this: (1) actual elapsed time tracked via Bash `date` calls at PM-1 start and PM-6 end, so the Pre-Flight Summary shows real wall-clock premission overhead; (2) a formula-based estimate derived from D/C/U metrics displayed immediately after step 4.5's difficulty sum check, before the user commits to PM-1.5 through PM-6, giving them a chance to trim the task list if the projected premission time is too long.

## Approach
Insert a `PREMISSION_START` timestamp at the very beginning of PM-1 (before any file reads). Insert a `PREMISSION_END` timestamp in PM-6 immediately before the Pre-Flight Summary block. Compute estimated premission duration using a linear formula based on clarification candidates (U), task count (N), and total D score. Display the estimate as a note after step 4.5's difficulty note (line 31 of SKILL.md). Display both estimate and actual elapsed time in the PM-6 Pre-Flight Summary block.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md`
  - Line 20: PM-1 step 1 (insert PREMISSION_START capture here, before step 1)
  - Line 25: step 4.5 difficulty sum check begins
  - Line 31: existing estimated mission duration note (insert premission estimate note immediately after this line)
  - Line 228: PM-6 Pre-Flight Summary block begins (insert PREMISSION_END capture before line 228)
  - Lines 230-243: Pre-Flight Summary content (add two new fields inside this block)

## Reuse
- Existing `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` pattern from pathfinder-mission-team SKILL.md line 66 -- use the same command for PREMISSION_START and PREMISSION_END to ensure consistent timezone handling.
- Existing `round(T * 0.43)` wording pattern (mission-team SKILL.md line 62/78) -- mirror the phrasing style for the premission estimate note.

## Steps

1. **Capture PREMISSION_START at PM-1 entry (line 20 of pathfinder-premission/SKILL.md).**

   Insert a new step 0 before the existing PM-1 step 1:

   ```
   - Before step 1:
   + 0. Run `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` and store as `PREMISSION_START`.
   ```

   The new step reads:
   `0. Run \`TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"\` and store as \`PREMISSION_START\`.`

   Insert this as the first numbered item under `### PM-1: Gather context`, before the existing item `1. Determine PROJECT_ROOT`.

2. **Add premission estimate note after the step 4.5 difficulty note (after line 31).**

   After line 31 (the existing `**"Estimated duration: ~X min (total_D x 0.43)"**` note), insert:

   ```
   - After line 31 (existing difficulty note), add:
   +    Then output a second note (no question, no AskUserQuestion): **"Estimated premission setup: ~Y min"** where Y = `round(N * 1.5 + U * 2.5)`, N = count of tasks in the confirmed list, U = count of clarification candidates computed eagerly using the D/C threshold logic (count tasks where `D - C > 0` OR (`D > 50` AND `C < 70`)). Include the breakdown inline: "(N=<n> tasks x 1.5 + U=<u> candidates x 2.5)".
   ```

   Full inserted text after line 31:

   `   Also output this note immediately after the duration note (no AskUserQuestion): **"Estimated premission setup: ~Y min"** where Y = \`round(N * 1.5 + U * 2.5)\`, N = count of confirmed tasks, U = count of clarification candidates from step 5 (computed ahead using the same D/C threshold logic: count tasks where \`D - C > 0\` OR (\`D > 50\` AND \`C < 70\`)). Include the breakdown inline: "(N=<n> tasks x 1.5 + U=<u> candidates x 2.5)".`

3. **Capture PREMISSION_END at PM-6 entry, before the Pre-Flight Summary (before line 228).**

   Insert before the `Present a pre-flight summary:` line:

   ```
   - Before line 228:
   + Run `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S"` and store as `PREMISSION_END`. Compute `PREMISSION_ELAPSED = difference in whole minutes between PREMISSION_START and PREMISSION_END` (parse both timestamps, subtract, round down to nearest whole minute; display as "<M> min" or "<M> min <S> sec" if seconds are non-zero).
   ```

4. **Add estimate and elapsed time fields to the Pre-Flight Summary block (lines 230-243).**

   Inside the Pre-Flight Summary fenced block, add two new lines after the `Tests:` line:

   ```diff
    Tests: <count> criteria across <task count> tasks
   +Premission Estimate: ~Y min (N=<n> x 1.5 + U=<u> x 2.5)
   +Premission Actual: <PREMISSION_ELAPSED> (started <PREMISSION_START>, ended <PREMISSION_END>)
    
    Files created:
   ```

   The `Y`, `n`, and `u` values are the same computed in step 2. `PREMISSION_ELAPSED`, `PREMISSION_START`, and `PREMISSION_END` are the values from steps 1 and 3.

## Verification

### Manual tests
- Run `/pathfinder-premission` with a small task list (2-3 tasks, no clarification candidates).
  - Confirm a `PREMISSION_START` Bash call is made at the very start of PM-1.
  - After step 4.5 difficulty check, confirm the "Estimated premission setup: ~Y min" note appears with the N/U breakdown.
  - For a list of 2 tasks with U=0: Y should equal `round(2 * 1.5 + 0 * 2.5)` = 3 min.
  - At PM-6 Pre-Flight Summary, confirm `Premission Estimate` and `Premission Actual` lines are present.
  - Confirm `Premission Actual` shows a plausible elapsed time (a few minutes for a short test run).
- Run `/pathfinder-premission` with tasks that trigger clarification candidates (e.g. a task with D=60, C=50).
  - Confirm U > 0 is reflected correctly in the estimate note: e.g. N=1, U=1 gives Y = round(1.5 + 2.5) = 4 min.

### Automated tests
- No automated test harness exists for skill `.md` files. A shell script could parse the SKILL.md and assert that:
  - The string `PREMISSION_START` appears before PM-2.
  - The string `PREMISSION_END` appears in PM-6 before the Pre-Flight Summary heading.
  - The string `Premission Estimate` appears inside the Pre-Flight Summary fenced block.
  - The string `Premission Actual` appears inside the Pre-Flight Summary fenced block.
- These four grep assertions cover the structural requirements without requiring a live skill run.

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | PREMISSION_START`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | PREMISSION_END`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | Premission Estimate`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | Premission Actual`

## Prefect-1 Report

All issues found and fixed in this pass:

**Nit**
- `M7-55-1-premission-duration.md:36` - Step 1 referred to the insertion point as "first bullet under `### PM-1: Gather context`". PM-1 uses a numbered list, not a bulleted list, so "bullet" was inaccurate. Changed to "first numbered item".

No blocking or minor issues found.

## Prefect-2 Report

**Minor**
- `M7-55-1-premission-duration.md:64-68` - Step 4's diff block omits the blank line (SKILL.md line 238) that currently exists between `Tests:` (line 237) and `Files created:` (line 239) in the Pre-Flight Summary fenced block. Without that trailing context line in the diff, the insertion point is ambiguous: an implementer could place the two new fields either before or after the blank line separator. The diff should show the blank line as trailing context (an unchanged ` ` line) so the insertion point is unambiguous.

No blocking issues found.

## Changelog

### Review - 2026-03-26
- #1: Step 2 diff block description of U contradicted the authoritative "Full inserted text" block (said "use 0" vs "compute ahead using threshold logic"). Reconciled the diff block to match the authoritative instruction (eager computation using D/C threshold logic).

### Prefect-1 - 2026-03-26
- Nit fix at Step 1: "first bullet" changed to "first numbered item" to match PM-1's actual numbered-list format (plan:36).

### Prefect-2 - 2026-03-26
- Minor fix at Step 4 diff (plan:64-68): added the blank context line (SKILL.md line 238) between the two new `+` fields and `Files created:` so the insertion point is unambiguous.
