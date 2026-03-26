## Task
#61 - Add remaining count to Difficulty field in MISSION-LOG mission section

## Context
The Difficulty field in MISSION-LOG files currently shows `COMPLETED_D/T` (e.g. "Difficulty: 3/10"). The user wants the remaining difficulty appended in parentheses so it reads "Difficulty: 3/10 (7 remaining)", making progress easier to scan at a glance without mental subtraction.

Two places in `pathfinder-mission-team/SKILL.md` write the Difficulty line: the MT-1 template (initial value) and the MT-3 task-completion step (incremental update).

## Approach
Edit both Difficulty-writing locations in SKILL.md so they include the `(N remaining)` suffix. The MT-1 template initializes at `0/<T>` with all tasks remaining, so it should read `0/<T> (<T> remaining)`. The MT-3 update rewrites the line after each completion, so it should compute `remaining = T - COMPLETED_D` and write `<COMPLETED_D>/<T> (<remaining> remaining)`.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - Line 79: MT-1 template Difficulty initial value
  - Line 405: MT-3 task-completion Difficulty rewrite instruction

## Reuse
The MT-3 step already reads the existing `Difficulty:` line to extract T before rewriting (line 405 note). That same extraction provides T for computing `remaining = T - COMPLETED_D`.

## Steps

1. Update the MT-1 MISSION-LOG template (line 79) to include the remaining count:

```
- Difficulty: 0/<T>
+ Difficulty: 0/<T> (<T> remaining)
```

2. Update the MT-3 task-completion Difficulty rewrite instruction (line 405) to append the remaining count:

```
- In the `## Mission` section of MISSION_LOG_PATH, rewrite the `Difficulty:` line to: `- Difficulty: <COMPLETED_D>/<T>` (read the existing `Difficulty:` line to extract T before rewriting)
+ In the `## Mission` section of MISSION_LOG_PATH, rewrite the `Difficulty:` line to: `- Difficulty: <COMPLETED_D>/<T> (<T - COMPLETED_D> remaining)` (read the existing `Difficulty:` line to extract T before rewriting; compute remaining as T minus COMPLETED_D)
```

## Verification

### Manual tests
- After the next pathfinder mission starts, open the new MISSION-LOG and confirm the initial Difficulty line reads `0/<T> (<T> remaining)` where T matches the mission's total difficulty.
- After the first task in that mission completes, confirm the Difficulty line updates to `<D>/<T> (<T-D> remaining)` with the correct arithmetic.

### Automated tests
- No automated tests exist for SKILL.md prose. A doc check confirms the two strings are present after the edit.

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Difficulty: 0/<T> (<T> remaining)`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | <COMPLETED_D>/<T> (<T - COMPLETED_D> remaining)`

## Changelog

### Review - 2026-03-26
- #1: Removed spurious `\n` from doc-check "missing" pattern (line 48) — literal backslash-n is not a newline in doc-check patterns and would never match

### Prefect-1 – 2026-03-26
- #1: Removed broken "missing" doc check — after the Step 1 edit, the new line `- Difficulty: 0/<T> (<T> remaining)` still contains the substring `- Difficulty: 0/<T>`, so the "missing" check would always fail. The two "contains" checks are sufficient to verify correctness.

## Prefect-1 Report

**Issue found (blocking):** The `missing` doc check on (former) line 48 was broken. After applying Step 1, the file would contain `- Difficulty: 0/<T> (<T> remaining)`, which is a superset of the pattern `- Difficulty: 0/<T>`. The "missing" assertion would therefore always fail. The check was removed; the two "contains" checks already fully verify the intended outcome.

**Fix applied:** Deleted the line:
```
`...SKILL.md | missing | - Difficulty: 0/<T>`
```

**All other checks passed:** Line numbers in Critical Files match actual source; both diff blocks apply cleanly to lines 79 and 405 of SKILL.md.
