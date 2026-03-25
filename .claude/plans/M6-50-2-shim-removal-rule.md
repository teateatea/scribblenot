## Task

#50 - Enforce hook-update-before-file-move ordering in Decomposer; require shim-removal log confirmation

## Context

Sub-task 1 added the hook-reference ordering rule to the Decomposer prompt. Sub-task 2 adds the shim-tracking rule: any sub-task that introduces a temporary shim or compatibility artifact must include an explicit removal step, and its sub-task log entry must contain a confirmation that the shim was removed. A corresponding note instructs the Reviewer/Prefect to flag absent removal-log confirmations as a non-blocking observation in the post-mortem rather than failing the sub-task outright.

## Approach

Add a new named step ("Enforce shim-removal tracking") to the Decomposer prompt in MT-3b of `pathfinder-mission-team/SKILL.md`, immediately after the existing "Enforce hook-reference ordering" step. Also add a corresponding rule in the sub-task log format in MT-3c step 5, and a note in the Reviewer/Prefect prompt about how to treat absent confirmations.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - lines 139 (end of hook-reference ordering step), 283-291 (sub-task log format in MT-3c step 5), 210-225 (Reviewer prompt)

## Reuse

No utility functions - this is a doc-only change to the SKILL.md file.

## Steps

1. In `SKILL.md`, after the "Enforce hook-reference ordering" step (line 139, ending with "If no such pair exists, proceed without reordering."), insert a new shim-tracking step in the Decomposer prompt:

```diff
 > If no such pair exists, proceed without reordering.
 >
+> **Step: Enforce shim-removal tracking.** Before finalizing the `sub_tasks` array, scan every sub-task for any operation that introduces a temporary shim, compatibility stub, or transitional artifact (e.g. a forwarding wrapper, an alias file, a temporary symlink, or a compatibility re-export added solely to bridge old and new code during a migration). For each such shim, verify that a later sub-task in the array explicitly removes it. If no removal sub-task exists, add one as the final sub-task of the sequence (or merge it into the nearest subsequent cleanup sub-task if one already exists), staying within the 5 sub-task cap. If the cap would be exceeded, merge the removal step into the closest subsequent sub-task and note the merge in that sub-task's description. This is a mandatory completeness constraint; update the `sub_tasks` array to satisfy it before returning.
+>
 > **Step: Evaluate TDD feasibility.** For each sub-task, inspect whether runnable failing tests are feasible.
```

2. In `SKILL.md`, update the sub-task log format in MT-3c step 5 to add a `Shim-removal` field:

```diff
 ### Sub-task N.<SUB_ID>: <description>
 - Status: Pass / Fail / Blocked
 - TDD: <TESTS WRITTEN file:line> / (no tests) / FAILED: <reason>
 - Implementation: <summary>
+- Shim-removal: <"N/A" if no shim was introduced | "Confirmed: <what was removed>" if a shim was removed | "Absent" if a shim was introduced but no removal confirmation was logged>
 - Timestamp: <SUBTASK_TIME>
```

3. In `SKILL.md`, add a note to the Reviewer subagent prompt (the prompt block that begins "You are Reviewer #<N> for plan") instructing it to flag absent shim-removal confirmations as a non-blocking observation. Insert after the line "- This sub-task is destructive: <true/false>":

```diff
 > - This sub-task is destructive: <true/false>
+> - If any sub-task log entry for a previously completed sub-task in this task has `Shim-removal: Absent`, flag it as a non-blocking observation in the post-mortem (do NOT fail the sub-task or block plan approval). Add a note to the plan's Changelog: "Observation: shim-removal confirmation absent for sub-task <ID> — recommend post-mortem note."
```

## Verification

### Manual tests

- Read the updated `SKILL.md` and confirm:
  1. The "Enforce shim-removal tracking" step appears in the Decomposer prompt immediately after the "Enforce hook-reference ordering" step.
  2. The sub-task log format in MT-3c step 5 includes the `Shim-removal:` field with the three documented values.
  3. The Reviewer prompt block includes the non-blocking observation note referencing `Shim-removal: Absent`.

### Automated tests

No automated tests exist for SKILL.md prose content. A doc-check is used instead.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Enforce shim-removal tracking`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Shim-removal:`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Shim-removal: Absent`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | non-blocking observation`

## Progress
- Step 1: Inserted "Enforce shim-removal tracking" step in Decomposer prompt after hook-reference ordering step
- Step 2: Added `Shim-removal:` field to sub-task log format in MT-3c step 5
- Step 3: Added non-blocking observation note to Reviewer prompt after "This sub-task is destructive" line

## Changelog

### Review - 2026-03-25
- #1: Fixed Step 1 diff trailing context line - replaced ` > For each sub-task, identify:` with the actual adjacent line ` > **Step: Evaluate TDD feasibility.**` to avoid ambiguous insertion point for the implementer.
