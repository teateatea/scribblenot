## Task
#33 - Add second clarification threshold to premission (D > 50, C < 70)

## Context
PM-1 step 5 of `/pathfinder-premission` currently routes a task to clarification only when `delta = D - C > 0`. This misses medium-to-high difficulty tasks that are reasonably well-understood on paper (delta <= 0) but still complex enough to benefit from a detailed explanation before going dark. Adding a second trigger -- `D > 50 AND C < 70` -- ensures tasks with meaningful complexity and sub-optimal clarity are surfaced even when C happens to meet or exceed D.

## Approach
Edit the bullet definitions for **fast-path** and **clarification candidates** in PM-1 step 5 of the SKILL.md file to add the second OR condition. The logic change is purely textual (a doc-only edit to a Markdown skill file); no code, scripts, or JSON need to change.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` lines 32-34 (PM-1 step 5 bullet definitions)
- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` line 49 (PM-1.5 question format string)

## Reuse
None -- this is a targeted text edit to a single Markdown file.

## Steps
1. Edit `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md`, replacing the PM-1 step 5 bullet definitions:

```diff
-   - Tasks where `delta <= 0` are **fast-path**: D score does not exceed C score, meaning the task is well-understood.
-   - Tasks where `delta > 0` are **clarification candidates**: difficulty exceeds clarity, raising the risk of mid-mission ambiguity.
-   - Tasks with missing D or C scores are treated as **fast-path** (skip threshold check for that task).
+   - Tasks where `delta <= 0` AND NOT (`D > 50` AND `C < 70`) are **fast-path**: D score does not exceed C score and the task is not in the medium-to-high difficulty / low-clarity zone.
+   - Tasks where `delta > 0` OR (`D > 50` AND `C < 70`) are **clarification candidates**: difficulty exceeds clarity, or the task is complex enough that a detailed explanation is required before going dark.
+   - Tasks with missing D or C scores are treated as **fast-path** (skip threshold check for that task).
```

2. Edit `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` line 49, removing the hardcoded `+` prefix from the delta format string to prevent `delta=+-5` output for negative-delta candidates routed by the new second condition:

```diff
-> "Task #<N> [D:<d> C:<c>, delta=+<delta>] - <title>
+> "Task #<N> [D:<d> C:<c>, delta=<delta>] - <title>
```

## Verification

### Manual tests
- Open `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` and confirm PM-1 step 5 now contains the phrase `D > 50` and `C < 70` in the clarification candidates bullet.
- Confirm line 49 now reads `delta=<delta>` (no hardcoded `+` prefix).
- Mentally trace three example tasks:
  - D:30 C:25 (delta=+5): still a clarification candidate via the first condition.
  - D:60 C:65 (delta=-5): now a clarification candidate via the second condition (D>50, C<70); format string renders `delta=-5` correctly.
  - D:60 C:75 (delta=-15): fast-path (D>50 but C>=70, and delta<=0).
  - D:40 C:50 (delta=-10): fast-path (delta<=0, D not >50).

### Automated tests
No test harness exists for SKILL.md files. A doc-check verifies the key strings are present post-edit.

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | delta > 0`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | D > 50`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | C < 70`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | missing | Tasks where \`delta > 0\` are **clarification candidates**: difficulty exceeds clarity`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | delta=<delta>`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | missing | delta=+<delta>`

## Changelog

### Review – 2026-03-25
- #1 (nit): Narrowed Critical Files line range from `lines 31-36` to `lines 32-34 (PM-1 step 5 bullet definitions)`

### Review – 2026-03-25
- #1 (minor): Added Step 2 to fix PM-1.5 format string `delta=+<delta>` -> `delta=<delta>` to prevent malformed `delta=+-5` output for negative-delta candidates; added line 49 to Critical Files; added manual test check and two doc checks for the format string fix.
- #2 (nit): Split malformed doc check line 42 into two separate pipe-check lines (`delta > 0` and `D > 50`); removed Prefect Report section.

## Progress
- Step 1: Updated PM-1 step 5 bullet definitions to add D>50/C<70 second clarification trigger
- Step 2: Removed hardcoded `+` prefix from delta format string in PM-1.5 question format
