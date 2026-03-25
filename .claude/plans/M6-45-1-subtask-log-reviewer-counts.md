## Task

#45 - Update MT-3c step 5 sub-task log template to add Reviewers and Prefects fields

## Context

The sub-task log template in MT-3c step 5 of the pathfinder-mission-team skill does not record how many reviewers or prefect passes ran for each sub-task. Adding `Reviewers: N` and `Prefects: N` fields makes the log self-describing and allows post-mission analysis of review overhead per sub-task.

## Approach

Edit the markdown template block in MT-3c step 5 (lines 293-300 of the SKILL.md) to insert two new bullet fields after `- TDD:` and before `- Implementation:`. The Mission Commander tracks reviewer and prefect pass counts implicitly as it runs the plan-review loop and appends the log entry in step 5; the new fields capture those counts directly in the log.

The new fields are:

- `- Reviewers: <N>` - total reviewer passes that ran for this sub-task (the actual count of reviewer subagents spawned, including retry passes; always >= 1 because the plan-review loop runs unconditionally)
- `- Prefects: <N>` - total prefect passes that ran for this sub-task (0 if skipped, otherwise 1, 2, or 3)

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` lines 291-300 (MT-3c step 5 template block)

## Reuse

No existing utilities. This is a pure documentation-template edit to one markdown file.

## Steps

1. Edit `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`.

   **Edit 1a** - In the MT-3c step 5 template block (the fenced code block beginning at line 293), insert two lines after `- TDD:` and before `- Implementation:`:

   ```diff
   ### Sub-task N.<SUB_ID>: <description>
   - Status: Pass / Fail / Blocked
   - TDD: <TESTS WRITTEN file:line> / (no tests) / FAILED: <reason>
   + - Reviewers: <N>
   + - Prefects: <N>
   - Implementation: <summary>
   - Shim-removal: <"N/A" if no shim was introduced | "Confirmed: <what was removed>" if a shim was removed | "Absent" if a shim was introduced but no removal confirmation was logged>
   - Timestamp: <SUBTASK_TIME>
   ```

   **Edit 1b** - Extend the step 5 prose at line 291 to tell the Mission Commander how to populate the new fields. Replace:

   ```
   **5. Run `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S%z"` and store the result as `SUBTASK_TIME`. Append to MISSION-LOG** (series write - never concurrent):
   ```

   With:

   ```
   **5. Run `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S%z"` and store the result as `SUBTASK_TIME`. Append to MISSION-LOG** (series write - never concurrent). For `Reviewers: <N>`: use the total count of reviewer subagent passes that ran (1-3 initial plus any retry passes; always >= 1 because the plan-review loop runs unconditionally regardless of test_runner). For `Prefects: <N>`: use 0 if skipped, otherwise 1, 2, or 3 (the number of prefect passes that ran):
   ```

## Verification

### Manual tests

- Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` and locate the MT-3c step 5 fenced code block. Confirm it contains `- Reviewers: <N>` immediately after `- TDD:` and `- Prefects: <N>` immediately after `- Reviewers: <N>`, both before `- Implementation:`.
- Confirm the step 5 prose at line 291 contains filling instructions for both new fields (Reviewers and Prefects count logic).

### Automated tests

- Doc check (see below).

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | - Reviewers: <N>`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | - Prefects: <N>`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | For \`Reviewers: <N>\``

## Prefect-1 Report

1. [minor] `M6-45-1-subtask-log-reviewer-counts.md:11` (Approach) - Incorrect attribution: the original text said "the Implementer subagent will record the final counts when it appends the log entry." The Implementer does not append to MISSION-LOG; the Mission Commander does in MT-3c step 5. Fixed to accurately state the Mission Commander appends the log entry.

2. [minor] `M6-45-1-subtask-log-reviewer-counts.md:28-45` (Steps) - The original Step 1 only inserted template lines but gave no instruction to also update the step 5 prose to tell the Mission Commander how to populate `Reviewers: <N>` and `Prefects: <N>`. Without this, the template would have unfillable `<N>` placeholders. Added Edit 1b to also update the SKILL.md step 5 prose with the fill-in logic. Updated Verification to include a doc check for the new prose.

## Prefect-2 Report

1. [blocking] `M6-45-1-subtask-log-reviewer-counts.md:15` and `M6-45-1-subtask-log-reviewer-counts.md:52` (Approach and Edit 1b) - The plan states `Reviewers: 0` when `test_runner` is "none" because "the plan-review loop was skipped." This is incorrect. Per `pathfinder-mission-team/SKILL.md:174`, `test_runner = "none"` only skips the Red phase (step 1) and the Verify TDD phase (step 4); the Plan-review loop (step 2) is explicitly entered immediately after: "proceed directly to the Plan-review loop after resolving `<TEST_RUNNER>`." Reviewers always run (minimum count = 1). There is no documented condition in the skill under which the plan-review loop is skipped entirely, so the `Reviewers: 0` case does not exist in practice. Both the Approach description and the Edit 1b replacement prose must remove or replace the "0 if test_runner is none / plan-review loop was skipped" language.

## Changelog

### Review - 2026-03-25
- #1: Fixed Approach to correctly attribute MISSION-LOG appending to the Mission Commander, not the Implementer subagent
- #2: Split Step 1 into Edit 1a (template lines) and Edit 1b (step 5 prose fill-in guidance); added Verification manual test and doc check for Edit 1b

### Review #2 - 2026-03-25
- #1: Fixed Approach (line 15) - removed "0 if test_runner is none and plan-review loop was skipped" language for Reviewers field; plan-review loop runs unconditionally so count is always >= 1
- #2: Fixed Edit 1b replacement prose (line 52) - replaced "use 0 if the plan-review loop was skipped (test_runner is none)" with correct instruction that count is always >= 1 because the plan-review loop runs unconditionally regardless of test_runner

## Progress
- Step 1 (Edit 1a): Inserted `- Reviewers: <N>` and `- Prefects: <N>` after `- TDD:` and before `- Implementation:` in the MT-3c step 5 fenced code block
- Step 1 (Edit 1b): Updated MT-3c step 5 prose to include fill-in instructions for Reviewers and Prefects fields

## Implementation
Complete - 2026-03-25
