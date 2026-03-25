# Plan: M6-46-2-2 - Make MT-3c Step 5 Mandatory

## Task

#46-2 sub-task 2: Verify MT-3c step 5 (log entry write) is mandatory and cannot be skipped

## Context

MT-3c step 5 is the sub-task log entry write. It runs after step 4 (Verify TDD tests pass). Two bypass paths exist where step 5 may not execute:

1. When step 4 fails after 3 TDD retries, the skill says "treat as a blocker (see MT-3e)". MT-3e's "implementation FAILED" branch says "Log to MISSION-LOG Sub-task Log with status Blocked" but does not explicitly invoke step 5. The step 5 log entry may be skipped because control jumps out of the MT-3c sequence.

2. When a permission denial occurs mid-sub-step (MT-3e permission path), the skill routes directly to "Continue with next task (MT-3a)" without mentioning step 5. Step 5 is never reached.

The `test_runner = "none"` path does NOT bypass step 5: it only skips steps 1 and 4; steps 2, 3, and 5 run normally.

## Approach

Add a clarifying sentence to step 5 making it explicit that the log entry is mandatory and must be written before transferring control to MT-3d or MT-3e. Also update both MT-3e branches to include a "write sub-task log entry (step 5 format, status=Blocked)" instruction before routing to MT-3a.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - MT-3c step 5: line 299
  - MT-3e permission denial branch: lines 371-385
  - MT-3e implementation FAILED branch: lines 387-393

## Reuse

No existing utilities. All changes are prose edits to SKILL.md.

## Steps

1. **Add mandatory-run sentence to MT-3c step 5.**

   Find the opening of step 5 in MT-3c:

   ```
   **5. Run `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S%z"` and store the result as `SUBTASK_TIME`. Append to MISSION-LOG** (series write - never concurrent). For `Reviewers: <N>`: use REVIEWER_COUNT. For `Prefects: <N>`: use PREFECT_COUNT.
   ```

   Replace with:

   ```
   **5. Run `TZ=America/Toronto date +"%Y-%m-%dT%H:%M:%S%z"` and store the result as `SUBTASK_TIME`. Append to MISSION-LOG** (series write - never concurrent). For `Reviewers: <N>`: use REVIEWER_COUNT. For `Prefects: <N>`: use PREFECT_COUNT. **This step is mandatory and must run regardless of `test_runner` value or implementation outcome. Write the sub-task log entry before proceeding to MT-3d or MT-3e.**
   ```

2. **Update the MT-3e permission-denial branch to write a step-5 log entry.**

   Current text (lines 382-385):
   ```
   - Increment `CONSECUTIVE_FAILURE_MAP[task]` by 1; call the new value X. Reduce `PRIORITY_MAP[task]` by X² (minimum 0). For each dependent task in TASK_QUEUE, apply the same reduction (minimum 0).
   - Re-queue task behind all higher-priority tasks.
   - Do NOT rename plan files; renaming only occurs on successful task completion.
   - Continue with next task (MT-3a).
   ```

   Replace with:
   ```
   - Before re-queuing, write a sub-task log entry using the MT-3c step 5 format with `Status: Blocked`, `Implementation: Permission denial - tool call blocked`, and `TDD: (no tests)` if tests were not yet written, or `TDD: <value from the Test Writer's output>` if tests were written before the denial.
   - Increment `CONSECUTIVE_FAILURE_MAP[task]` by 1; call the new value X. Reduce `PRIORITY_MAP[task]` by X² (minimum 0). For each dependent task in TASK_QUEUE, apply the same reduction (minimum 0).
   - Re-queue task behind all higher-priority tasks.
   - Do NOT rename plan files; renaming only occurs on successful task completion.
   - Continue with next task (MT-3a).
   ```

3. **Update the MT-3e implementation-FAILED branch to write a step-5 log entry.**

   Current text (lines 387-393):
   ```
   If an implementation FAILED result or unrecoverable error occurs:
   - Log to MISSION-LOG Sub-task Log with status Blocked.
   - Increment `CONSECUTIVE_FAILURE_MAP[task]` by 1; call the new value X. Reduce `PRIORITY_MAP[task]` by X² (minimum 0). For each dependent task in TASK_QUEUE, apply the same reduction (minimum 0).
   - Re-queue task.
   - Do NOT rename plan files; renaming only occurs on successful task completion.
   - Continue with next task.
   ```

   Replace with:
   ```
   If an implementation FAILED result or unrecoverable error occurs:
   - Write a sub-task log entry using the MT-3c step 5 format with `Status: Blocked`, `Implementation: <FAILED reason>`, and `TDD: <value from the Test Writer's output>` (if tests were written) or `TDD: (no tests)` (if no tests were written). This is mandatory before re-queuing.
   - Increment `CONSECUTIVE_FAILURE_MAP[task]` by 1; call the new value X. Reduce `PRIORITY_MAP[task]` by X² (minimum 0). For each dependent task in TASK_QUEUE, apply the same reduction (minimum 0).
   - Re-queue task.
   - Do NOT rename plan files; renaming only occurs on successful task completion.
   - Continue with next task.
   ```

## Verification

### Manual tests

- Read the updated SKILL.md and confirm step 5 contains the mandatory-run sentence.
- Read both MT-3e branches and confirm each starts with a log-entry write instruction before the re-queue step.
- Confirm no double blank lines were introduced in the edits.

### Automated tests

No automated tests apply to SKILL.md prose edits. Verification is manual inspection only.

## Status

Ready for review

## Prefect-2 Report

### Issues Found

1. **[minor] Line 60 - TDD field guidance incomplete for permission-denial branch (when tests were already written)**
   - `M6-46-2-2-step5-mandatory.md:60`
   - The proposed Step 2 replacement text specifies `TDD: (no tests)` only "if tests were not yet written". It leaves no guidance for the case where MT-3c step 3 already ran (tests were written) but the permission denial occurred during step 4 (implementation) or later. In that scenario, the agent has no instruction on what TDD value to log.
   - Suggested fix: extend the sentence to cover both cases, e.g.: `and \`TDD: (no tests)\` if tests were not yet written (or \`TDD: <TESTS WRITTEN file:line>\` if tests were written before the denial).`

### No Other Issues Found

- Step 1 target text at SKILL.md:299 confirmed to match exactly.
- Step 2 target text at SKILL.md:382-385 confirmed to match exactly.
- Step 3 target text at SKILL.md:387-393 confirmed to match exactly.
- Prefect-1 em-dash fix (line 60) is present in the current plan file.
- Combined belt-and-suspenders approach (step 5 mandatory sentence + both MT-3e branches) is architecturally sound.
- No double blank lines in proposed replacement text.

## Prefect-1 Report

### Fixes Applied

1. **[minor] Line 60 - em-dash in proposed implementation text**
   - `M6-46-2-2-step5-mandatory.md:60`
   - The proposed replacement text for the MT-3e permission-denial branch contained an em-dash (`—`) in the string `Implementation: Permission denial — tool call blocked`. Rule #19 prohibits em-dashes.
   - Fixed: replaced `—` with `-`.

### No Other Issues Found

- Step 1 target text at SKILL.md:299 matches exactly.
- Step 2 target text at SKILL.md:382-385 matches exactly.
- Step 3 target text at SKILL.md:387-393 matches exactly.
- Both MT-3e bypass paths are correctly identified and addressed.
- No double blank lines in proposed replacement text.
- Approach is sound: belt-and-suspenders fix at both step 5 and the two MT-3e branches.

## Changelog

### Prefect-1 Review - 2026-03-25
- #1: Replaced em-dash with hyphen in Step 2 proposed replacement text (line 60)

### Review – 2026-03-25
- #1: Extended Step 2 TDD field guidance to cover the case where tests were already written before the permission denial (added `TDD: <value from the Test Writer's output>` clause)
- #2: Extended Step 3 TDD field guidance with explicit `TDD:` value instructions for both the tests-written and no-tests cases (parallel fix to #1 for the implementation-FAILED branch)

## Progress

- Step 1: Added mandatory-run sentence to MT-3c step 5 in SKILL.md
- Step 2: Updated MT-3e permission-denial branch to write sub-task log entry before re-queuing
- Step 3: Updated MT-3e implementation-FAILED branch to write sub-task log entry before re-queuing

## Implementation
Complete - 2026-03-25
