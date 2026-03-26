## Task

#54 - Add Min/C and Min/U stats to Mission Complete section alongside Min/D

## Context

The Mission Complete section of pathfinder mission logs currently records Min/D (mission duration divided by total difficulty). Task #54 extends this with two more metrics: Min/C (duration divided by total C score) and Min/U (duration divided by U, the aggregate uncertainty). These metrics enable future correlation analysis between mission duration and task clarity or uncertainty. C scores are already annotated in TASKS.md as `[D:N C:N]` but are not currently parsed or tracked by the SKILL.md mission loop.

## Approach

Add C score tracking to the MT-3 state initialization and the MT-3 task-complete branch, mirroring the existing D_MAP / COMPLETED_D pattern exactly. Then extend MT-4 step 3 to compute TOTAL_C, MIN_C, NUM_COMPLETED, TOTAL_U, and MIN_U after the existing MIN_D computation. Finally add Min/C and Min/U to the MT-4 step 4 Mission Complete template immediately after Min/D.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - Line 127: MT-3 state variable initialization (D_MAP, COMPLETED_D defined here)
  - Line 363: `Add D_MAP[task] to COMPLETED_D` (task-complete branch)
  - Lines 454-457: MT-4 step 3 duration and MIN_D computation block
  - Lines 460-471: MT-4 step 4 Mission Complete template

## Reuse

- Pattern: `D_MAP` / `COMPLETED_D` / `TOTAL_D` / `MIN_D` - replicate identically for C scores using `C_MAP` / `COMPLETED_C` / `TOTAL_C` / `MIN_C`.
- C score parsing: same annotation format `[D:N C:N]` in TASKS.md; parse `C:N` the same way D is parsed from `[D:N ...]`.

## Steps

1. **MT-3 state init (line 127): add C_MAP and COMPLETED_C alongside D_MAP and COMPLETED_D.**

```
- a D_MAP (task -> D score, initialized from TASKS.md alongside T in MT-1), a COMPLETED_D counter (running sum of D scores for all successfully completed tasks, initialized to 0),
+ a D_MAP (task -> D score, initialized from TASKS.md alongside T in MT-1), a COMPLETED_D counter (running sum of D scores for all successfully completed tasks, initialized to 0), a C_MAP (task -> C score, initialized from TASKS.md `[D:N C:N]` annotation; if absent treat as 0), a COMPLETED_C counter (running sum of C scores for all successfully completed tasks, initialized to 0), a NUM_COMPLETED counter (count of successfully completed tasks, initialized to 0),
```

2. **MT-3 task-complete branch (line 363): increment COMPLETED_C and NUM_COMPLETED alongside COMPLETED_D.**

```
-   - Add `D_MAP[task]` to COMPLETED_D.
+   - Add `D_MAP[task]` to COMPLETED_D.
+   - Add `C_MAP[task]` to COMPLETED_C.
+   - Increment NUM_COMPLETED by 1.
```

3. **MT-4 step 3 (lines 456-457): add TOTAL_C, MIN_C, TOTAL_U, and MIN_U computations after MIN_D.**

```
-   Set `TOTAL_D = COMPLETED_D` (the running sum of D scores for all successfully completed tasks).
-   Compute `MIN_D`: if `TOTAL_D > 0`, `MIN_D = round(DURATION_MINUTES / TOTAL_D, 2)`; otherwise `MIN_D = "N/A"`.
+   Set `TOTAL_D = COMPLETED_D` (the running sum of D scores for all successfully completed tasks).
+   Compute `MIN_D`: if `TOTAL_D > 0`, `MIN_D = round(DURATION_MINUTES / TOTAL_D, 2)`; otherwise `MIN_D = "N/A"`.
+   Set `TOTAL_C = COMPLETED_C` (the running sum of C scores for all successfully completed tasks).
+   Compute `MIN_C`: if `TOTAL_C > 0`, `MIN_C = round(DURATION_MINUTES / TOTAL_C, 2)`; otherwise `MIN_C = "N/A"`.
+   Compute `TOTAL_U = (NUM_COMPLETED * 100) - TOTAL_C`.
+   Compute `MIN_U`: if `TOTAL_U > 0`, `MIN_U = round(DURATION_MINUTES / TOTAL_U, 2)`; otherwise `MIN_U = "N/A"`.
```

4. **MT-4 step 4 Mission Complete template (line 469): add Min/C and Min/U after Min/D.**

```
-  - Min/D: <MIN_D>
+  - Min/D: <MIN_D>
+  - Min/C: <MIN_C>
+  - Min/U: <MIN_U>
```

## Verification

### Manual tests

- Run a short pathfinder mission (e.g. `/pathfinder-mission-team #54`) after applying the changes.
- After mission completes, open the resulting `SUCCESSFUL-MISSION-LOG-*.md` file.
- Confirm `## Mission Complete` contains all three lines: `Min/D:`, `Min/C:`, and `Min/U:`.
- Verify the numeric values are plausible: for a mission of duration D_min with tasks having known C scores, manually compute TOTAL_C = sum(C), TOTAL_U = (count * 100) - TOTAL_C, and check MIN_C = D_min / TOTAL_C and MIN_U = D_min / TOTAL_U match the log.
- Verify edge case: if all tasks have C:0, Min/C and Min/U should both show "N/A".

### Automated tests

- Doc check on the SKILL.md itself (post-edit assertions below).
- Unit-level: no automated test runner is needed for a `.md` skill file; correctness is verified by the doc checks.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | C_MAP (task -> C score`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | COMPLETED_C`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | NUM_COMPLETED`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Add \`C_MAP[task]\` to COMPLETED_C`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | TOTAL_C = COMPLETED_C`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | MIN_C`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | TOTAL_U = (NUM_COMPLETED`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | MIN_U`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Min/C: <MIN_C>`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Min/U: <MIN_U>`

## Progress

- Step 1: Added C_MAP, COMPLETED_C, and NUM_COMPLETED to MT-3 state initialization at line 127
- Step 2: Added `C_MAP[task]` to COMPLETED_C and NUM_COMPLETED increment in MT-3 task-complete branch
- Step 3: Added TOTAL_C, MIN_C, TOTAL_U, and MIN_U computations in MT-4 step 3 after MIN_D
- Step 4: Added Min/C and Min/U lines to MT-4 Mission Complete template after Min/D
