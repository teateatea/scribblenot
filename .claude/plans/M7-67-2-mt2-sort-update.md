## Task

#67 - Update MT-2 reorder step sort key

## Context

After sub-task 1 of task #67, PRIORITY_MAP uses 100-position scoring: the first task listed in the BRIEF gets score 99, the second gets 98, and so on. Higher score means higher premission priority. The MT-2 reorder sentence still references the old sort logic ("highest priority first, then by position in TASK_LIST (earlier = higher priority) within the same priority tier"), which is now inaccurate and ambiguous. The new sort must use PRIORITY_MAP score as the primary key (highest score first) with the D score as a tiebreaker only when scores are equal.

## Approach

Single-line edit to the MT-2 reorder sentence in the pathfinder-mission-team SKILL.md. Remove the "position in TASK_LIST" language and replace with PRIORITY_MAP score semantics.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - line 137

## Reuse

No utilities to reuse. This is a documentation-only string replacement.

## Steps

1. Edit line 137 of `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`:

```diff
- Reorder the TASK_QUEUE based on the DAG: highest priority first, then by position in TASK_LIST (earlier = higher priority) within the same priority tier, respecting dependency ordering.
+ Reorder the TASK_QUEUE based on the DAG: sort by PRIORITY_MAP score descending (highest score = highest priority, executed first); use D score as tiebreaker only when two tasks share the same PRIORITY_MAP score; respect dependency ordering.
```

## Verification

### Manual tests

- Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` and confirm line 137 contains "PRIORITY_MAP score descending" and does not contain "position in TASK_LIST".

### Automated tests

No automated tests applicable for a single-line doc edit.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | sort by PRIORITY_MAP score descending`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | use D score as tiebreaker only when two tasks share the same PRIORITY_MAP score`

Note: After applying this change, line 149 (MT-3a) will still contain "earliest position in TASK_LIST (lower index = higher priority)" as a tiebreaker. That reference is out of scope for this sub-task but should be updated in a follow-on sub-task.

## Changelog

### Review – 2026-03-26
- #1 (nit): Changed plain code fence to ```diff in the Steps section for proper diff syntax recognition.

### Prefect-1 – 2026-03-26
- Nit: Doc check block removed the `| missing | position in TASK_LIST` assertion (which would fail after this edit since line 149 still contains the phrase) and replaced it with an explanatory note scoping the residual reference to a follow-on sub-task.

## Prefect-1 Report

**Pass result:** Nit only - no blocking or minor issues.

**Diff check:** The `-` line in the Steps diff matches line 137 of `pathfinder-mission-team/SKILL.md` exactly. Applies cleanly.

**Nit found:** The original doc-check assertion ``missing | position in TASK_LIST`` would have failed post-edit because line 149 (MT-3a) still contains "earliest position in TASK_LIST". The phrase is legitimately out of scope for this sub-task. Fixed by replacing the failing assertion with a scoped note directing a follow-on sub-task to address line 149.

## Progress

- Step 1: Edited line 137 of pathfinder-mission-team/SKILL.md - replaced old sort key with PRIORITY_MAP score descending + D score tiebreaker language

## Implementation
Complete - 2026-03-26
