## Task

#41 — Confirm and fix mission-team task execution order to respect premission priority ranking
#41-2 — M6 live evidence: tasks #39, #49, #50 completed first while higher-listed tasks #19, #43, #47 remain queued

Sub-task 1 of #41: investigation and documentation only; no file changes.

## Context

Mission 6 live evidence (MISSION-LOG-6) shows tasks #39, #49, #50 completing at ~1h45m while tasks listed earlier in the premission order (#19, #43, #47, etc.) remain Queued. This confirms execution order does not follow the premission priority list.

The root cause is a three-part chain:

1. PRIORITY_MAP initializes to 99 for every task (per MT-3 definition), because no M6 task in TASKS.md carries a `[P:N ...]` annotation. All tasks enter the queue at equal priority.

2. MT-2 (Dependency Scout) reorders the TASK_QUEUE by "highest priority first, hardest (highest D score) first within the same priority tier." With every task at priority 99, the tie-break is purely D score descending. This re-sorts the queue by difficulty, discarding whatever order TASK_LIST was built in.

3. MT-3a selects the highest-priority unblocked task; on a tie it picks the highest D score. This is consistent with MT-2 - both reinforce D-score ordering rather than premission order.

The BRIEF path (MT-1 step 2-A) reads task IDs from `## Task Priority Order` in the BRIEF file and preserves premission order as TASK_LIST. Task #42 (rename PROJECT-FOUNDATION.md to MISSION-#-BRIEF.md) is complete, but the BRIEF file (MISSION-6-BRIEF.md) does NOT contain a `## Task Priority Order` section - meaning the BRIEF path either fails to find task IDs or the section was not written by premission. Even if TASK_LIST preserves premission order, MT-2 immediately re-sorts it by D score, erasing that order.

The token-list path (MT-1 step 2-B) preserves user-typed order only until MT-2 re-sorts.

## Approach

This sub-task is investigation only. Document the confirmed root cause so sub-tasks 2-4 can apply targeted fixes:

- Sub-task 2: ensure premission writes `## Task Priority Order` into the BRIEF file with explicit `[P:N]` annotations per task reflecting premission rank.
- Sub-task 3: change PRIORITY_MAP initialization to use premission rank (read from BRIEF `## Task Priority Order`) rather than defaulting all tasks to 99.
- Sub-task 4: update MT-2 reorder and MT-3a tie-break to use PRIORITY_MAP values correctly, so premission rank governs order when D scores are equal or when D-score order conflicts with premission rank.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` — MT-1 step 2 (TASK_LIST build, PRIORITY_MAP init), MT-2 (reorder step), MT-3a (pick next task)
- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` — PM output: BRIEF file, `## Task Priority Order` section (absent in current M6 output)
- `C:/Users/solar/Documents/Claude Projects/scribblenot/pathfinder/MISSION-6-BRIEF.md` — missing `## Task Priority Order` section; confirms premission did not write priority order into BRIEF
- `C:/Users/solar/Documents/Claude Projects/scribblenot/pathfinder/MISSION-LOG-6-skill-log-quality.md` — live execution log; Step 1 reads this to capture concrete task execution order as evidence

## Reuse

No new utilities needed. Sub-tasks 2-4 will edit SKILL.md files in place using the Edit tool.

## Steps

1. Read MISSION-LOG-6 to capture the concrete execution sequence as evidence (task IDs and order of first Attempt:1 entries).
2. Read `pathfinder-premission/SKILL.md` to locate the step that writes the BRIEF file. Note: task #42 added `## Task Priority Order` to the Foundation Author prompt during M6, so the current SKILL.md template DOES include it. The investigation point is that M6's premission ran BEFORE task #42 completed, which is why MISSION-6-BRIEF.md lacks the section. Confirm this by checking the M6 log execution order (task #42 Sub-task 42.1 timestamp vs the mission start time).
3. Read `pathfinder-mission-team/SKILL.md` MT-1 step 2-A to confirm the BRIEF path depends on `## Task Priority Order` being present.
4. Append findings summary to MISSION-LOG-6 under a new `## Sub-task Log` entry for task #41 sub-task 1 (investigation complete, no code changes).

## Verification

### Manual tests

- Confirm MISSION-6-BRIEF.md has no `## Task Priority Order` section (visual check).
- Confirm no TASKS.md entry for M6 tasks carries a `[P:N ...]` annotation (visual check).

### Automated tests

No automated tests apply; this sub-task produces documentation only. Subsequent sub-tasks (2-4) will add tests to verify correct execution order in a future mission run.

## Prefect-1 Report

### Issues Found

**BLOCKING — Step 2 instructs an incorrect finding (plan:46)**

Step 2 says "confirm `## Task Priority Order` is absent from its template." This is factually wrong: task #42 (Sub-task 42.1, timestamp 2026-03-25T20:19:04) edited `pathfinder-premission/SKILL.md` during M6 and added `## Task Priority Order` to the Foundation Author prompt. An implementer executing Step 2 would find the section IS present, contradicting the step's expected result and producing misleading investigation notes. The step has been rewritten to accurately state that the current template includes the section, and that the evidence point is that M6's premission ran before task #42 completed (hence MISSION-6-BRIEF.md lacks it).

**Fix applied:** Step 2 rewritten at plan line 46 to correct the expected finding and direct the implementer to verify the timeline (task #42 Sub-task 42.1 timestamp vs mission start time) as the actual evidence.

---

## Changelog

### Review – 2026-03-25
- N1: Added MISSION-LOG-6 path to Critical Files section (Step 1 reads it but it was absent from the list)

### Review – 2026-03-25 (Prefect-1)
- B1: Step 2 rewrote expected finding — task #42 added `## Task Priority Order` to premission template during M6; step previously instructed implementer to confirm its absence, which would produce incorrect findings (plan:46)

## Progress

- Step 1: Read MISSION-LOG-6; captured execution sequence — #50, #49, #39, #42, #45, #46, #46-2, #51, #52, #53, #48, #54, #47, #19 order; #19/#43/#47 completed last despite being first in premission list
- Step 2: Read pathfinder-premission/SKILL.md; confirmed `## Task Priority Order` IS present in current template (added by sub-task 42.1 at 20:19:04); M6 premission ran at 19:06:43 (before 42.1), so MISSION-6-BRIEF.md lacks the section — timeline confirmed
- Step 3: Read pathfinder-mission-team/SKILL.md MT-1 step 2-A; confirmed BRIEF path halts with error if `## Task Priority Order` is absent or yields no IDs
- Step 4: Appended findings summary to MISSION-LOG-6 under Sub-task 41.1 entry

## Implementation
Complete – 2026-03-25
