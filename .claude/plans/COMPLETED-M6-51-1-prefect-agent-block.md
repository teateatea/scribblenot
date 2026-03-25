# Plan: M6-51-1 - Prefect Agent Block Assessment

## Task
Task #51 sub-task 1: Update MT-3c Reviewer/Prefect subagent prompts to explicitly instruct Prefect to treat a missing Agent field in any sub-task log entry as a blocking issue.

## Status
Draft

## Assessment

### What MT-3c Reviewer/Prefect prompts actually do
The MT-3c Reviewer and Prefect subagent prompts (pathfinder-mission-team/SKILL.md lines 219-270) review PLAN FILES. They read the plan file for the current sub-task, classify issues in the plan, apply fixes, and return approval/rejection signals for the plan. They do not read sub-task log entries at any point.

Adding an Agent-field check to the MT-3c Prefect prompts is inappropriate because:
1. The Prefect runs BEFORE a sub-task is implemented, during plan-review phase.
2. Sub-task log entries are written AFTER implementation completes (MT-3c step 5).
3. There are no sub-task log entries for the current sub-task to check at Prefect time.
4. Checking log entries from previously completed sub-tasks during Prefect review has no connection to plan quality and would produce confusing coupling between plan-review and log-auditing concerns.

### Where Agent-field blocking belongs
The MT-3d enforcement gate (SKILL.md lines 318-333) is the correct location. It already:
- Scans sub-task log entries for the completed task
- Checks for missing required fields including `Agent`
- Appends warning entries when fields are absent

However, it currently says "Do NOT block completion regardless of outcome." The PROJECT-TESTS criterion requires treating a missing Agent field as a **blocking** issue. The fix belongs in MT-3d, not MT-3c.

### Conclusion for sub-task 1
No change to MT-3c Reviewer/Prefect prompts is warranted. The prompts review plan files and have no access to or responsibility for sub-task log entries. Adding Agent-field checking there would be a category error.

Sub-task 2 (MT-3d) is where the blocking behavior must be implemented: change the per-entry field check in MT-3d from "Do NOT block completion" to halt or re-queue when a required field (including Agent) is missing.

## Changes

No file changes. This plan documents that sub-task 1 requires no code modification because the MT-3c Prefect prompts are plan-file reviewers, not log auditors.

## Changelog
- Initial draft: assessed MT-3c vs MT-3d scope; determined MT-3c Prefect prompts are inappropriate location for Agent-field log checking; documented that blocking behavior belongs in MT-3d (sub-task 2).
