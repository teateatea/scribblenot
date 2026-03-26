## Task

#60 - Add ETA fields to MISSION-LOG template

## Context

The MISSION-LOG template in MT-1 step 5 of pathfinder-mission-team/SKILL.md has no fields for tracking estimated completion time. Sub-task 1 adds placeholder tokens for those fields to the template; sub-task 2 will wire up the runtime values when a mission starts.

## Approach

Insert two new lines directly above the `## Task Status` heading in the markdown template block (SKILL.md lines 69-94). Use placeholder tokens `<INITIAL_ETA>` and `<CURRENT_ETA>` so the template is syntactically complete and sub-task 2 can substitute real values at runtime.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` lines 78-80 (the gap between `Estimated-Duration` and `## Task Status`)

## Reuse

No existing utility to reuse; this is a pure template-text insertion.

## Steps

1. Read SKILL.md lines 69-94 to confirm current state before editing.

2. In the template block, insert two new lines between `Estimated-Duration` (line 78) and the blank line (line 79) before `## Task Status` (line 80). The diff for the template block:

```diff
- Estimated-Duration: ~<ESTIMATED_DURATION> min (T x 0.43)
-
- ## Task Status
+ Estimated-Duration: ~<ESTIMATED_DURATION> min (T x 0.43)
+ - Initial Estimated Completion Time: <INITIAL_ETA> (Started at <START_TIME>)
+ - Current Estimated Completion Time: <CURRENT_ETA> (Updated at <UPDATE_TIME>)
+
+ ## Task Status
```

   Use the Edit tool with `old_string` = the exact three lines (no leading spaces):
```
- Estimated-Duration: ~<ESTIMATED_DURATION> min (T x 0.43)

## Task Status
```
   and `new_string` adding the two ETA lines between them (no leading spaces):
```
- Estimated-Duration: ~<ESTIMATED_DURATION> min (T x 0.43)
- Initial Estimated Completion Time: <INITIAL_ETA> (Started at <START_TIME>)
- Current Estimated Completion Time: <CURRENT_ETA> (Updated at <UPDATE_TIME>)

## Task Status
```

3. Re-read SKILL.md lines 78-86 to confirm the two new lines are present and correctly positioned.

## Verification

### Manual tests

- Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` and visually confirm lines 78-82 of the template block now read: `Estimated-Duration`, `Initial Estimated Completion Time`, `Current Estimated Completion Time`, blank line, `## Task Status` - in that order with no extra blank lines.

### Automated tests

- No automated test suite exists for SKILL.md. A doc check (below) covers the change.

### Doc checks

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Initial Estimated Completion Time: <INITIAL_ETA>`
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Current Estimated Completion Time: <CURRENT_ETA>`

## Prefect-1 Report

Issues found and fixed:

**Blocking**
1. `M7-60-1-eta-template.md:38` - Step 2 instruction said "exact **four** lines" but the `old_string` block contains only 3 lines (Estimated-Duration, blank, ## Task Status). Fixed: changed "four" to "three".

**Minor**
2. `M7-60-1-eta-template.md:38-51` - `old_string`/`new_string` code blocks were indented 3 spaces as markdown list continuation, creating ambiguity about whether the leading spaces are part of the string. Fixed: unindented both code blocks and added "(no leading spaces)" clarifiers.
3. `M7-60-1-eta-template.md:33,48` - Current ETA line used `(Updated at <START_TIME>)` but `<START_TIME>` is the mission start time, not the update time. Using the same token for both lines makes "Updated at" meaningless. Fixed: changed to `<UPDATE_TIME>` on the Current ETA line in both the diff block and the `new_string` block.

**Nit**
4. `M7-60-1-eta-template.md:27` - Diff fence was a plain ` ``` ` block. Fixed: changed to ` ```diff ` for syntax highlighting.

## Prefect-2 Report

**Nit**
1. `M7-60-1-eta-template.md:25` - Step 2 prose says "the blank line before `## Task Status` (line 80)" but line 80 in the actual file is `## Task Status` itself; the blank line is at line 79. The parenthetical incorrectly labels line 80 as the blank line. The old_string/new_string blocks are unaffected and correct, so this has no impact on implementation. Suggested fix: change "(line 80)" to "(line 79)" or rephrase to "the blank line (line 79) before `## Task Status` (line 80)".

## Changelog

### Review - 2026-03-26
- #1: Fixed "exact four lines" -> "exact three lines" (old_string has 3 lines, not 4)
- #2: Unindented old_string/new_string code blocks and added "(no leading spaces)" note to prevent copy-paste errors
- #3: Changed `(Updated at <START_TIME>)` to `(Updated at <UPDATE_TIME>)` on Current ETA line
- #4: Changed plain ``` fence to ```diff for the diff block

### Review #2 - 2026-03-26
- #1 (nit): Clarified line reference in Step 2 prose: "the blank line (line 79) before `## Task Status` (line 80)" instead of the ambiguous "(line 80)"
