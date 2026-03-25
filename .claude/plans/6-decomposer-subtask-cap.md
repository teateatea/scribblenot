# Plan: 6-decomposer-subtask-cap

## Task
#6 - Add 5 sub-task cap and coarseness heuristic to Decomposer prompt in MT-3b

## Context
The current Decomposer prompt in MT-3b instructs the subagent to break tasks into "the minimum meaningful unit that can be tested independently." This framing encourages over-decomposition, producing too many fine-grained sub-tasks. The goal is to replace it with a directive to group tightly coupled steps and cap decomposition at 5 sub-tasks, keeping the mission loop manageable and reducing unnecessary overhead.

## Approach
Make a targeted text replacement in the Decomposer prompt inside MT-3b of `pathfinder-mission-team/SKILL.md`. Replace the "minimum meaningful unit" sentence with a "group tightly coupled steps" directive plus an explicit 5 sub-task maximum.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - line 100: the sentence to replace

## Reuse
No existing utilities to reuse. Single Edit operation on the SKILL.md file.

## Steps

1. Edit `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`, replacing the decomposition instruction at line 100:

```diff
- > Break task #<N> into an ordered list of sub-tasks. Each sub-task must be the minimum meaningful unit that can be tested independently.
+ > Break task #<N> into an ordered list of sub-tasks. Group tightly coupled steps into a single sub-task rather than splitting them apart. Use the coarsest granularity that still allows each sub-task to be tested or verified independently. Cap the list at a maximum of 5 sub-tasks. If the task can be covered in fewer, prefer fewer.
```

## Verification

### Manual tests
- Read `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` lines 94-121 and confirm:
  - The old phrase "minimum meaningful unit" is gone.
  - The new text includes "Group tightly coupled steps", "coarsest granularity", and "maximum of 5 sub-tasks".
  - The JSON schema example block and surrounding instructions are unchanged.

### Automated tests
None applicable. This is a doc-only change to a skill prompt file.

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | missing | minimum meaningful unit`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Group tightly coupled steps`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | maximum of 5 sub-tasks`

## Changelog

### Review - 2026-03-24
- #1: Replaced em-dash in proposed replacement text (line 24) with a period to comply with Rule #19
- #2: Replaced em-dash in Automated tests section (line 36) with a period to comply with Rule #19
- #3: Replaced em-dash in Critical Files section (line 13) with a hyphen to comply with Rule #19

## Progress
- Step 1: Replaced "minimum meaningful unit" sentence with group/coarsest/5-cap directive in pathfinder-mission-team/SKILL.md line 100

## Implementation
Complete - 2026-03-24
