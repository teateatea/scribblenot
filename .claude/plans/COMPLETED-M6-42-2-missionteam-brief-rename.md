## Task

#42 - Rename PROJECT-FOUNDATION to MISSION-#-BRIEF and add task priority order to it in both pathfinder skills

## Context

The pathfinder-mission-team skill references `PROJECT-FOUNDATION.md` in four prompts: the MT-2 Dependency Scout, the MT-3b Decomposer, the MT-3c Reviewer context line, and the MT-3d Drift Checker. Task #42 renames this file to `MISSION-<MISSION_NUMBER>-BRIEF.md` (mission-numbered) so each mission reads from a brief scoped to that mission's number, established in MT-1 step 4. All four references must be updated to use the `MISSION_NUMBER` variable already in scope.

## Approach

Perform four targeted string replacements in `pathfinder-mission-team/SKILL.md`. Replace each occurrence of the static path `pathfinder/PROJECT-FOUNDATION.md` with the dynamic path `pathfinder/MISSION-<MISSION_NUMBER>-BRIEF.md`, using the `MISSION_NUMBER` variable that is assigned in MT-1 step 4. All four locations (MT-2, MT-3b, MT-3c, MT-3d) are in scope for this sub-task.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md`
  - Line 95: MT-2 Dependency Scout prompt
  - Line 133: MT-3b Decomposer prompt
  - Line 230: MT-3c Reviewer context line
  - Line 305: MT-3d Drift Checker prompt

## Reuse

No existing utilities. Direct string replacements using the Edit tool.

## Steps

1. Read `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` in full to confirm current content at each target line.

2. Replace the MT-2 Dependency Scout reference (line 95):

```diff
- > You are the Dependency Scout. Read `<PROJECT_ROOT>/.claude/TASKS.md` and `<PROJECT_ROOT>/pathfinder/PROJECT-FOUNDATION.md` (if it exists).
+ > You are the Dependency Scout. Read `<PROJECT_ROOT>/.claude/TASKS.md` and `<PROJECT_ROOT>/pathfinder/MISSION-<MISSION_NUMBER>-BRIEF.md` (if it exists).
```

2.5. Replace the MT-3b Decomposer reference (line 133):

```diff
- > - `<PROJECT_ROOT>/pathfinder/PROJECT-FOUNDATION.md` (if it exists)
+ > - `<PROJECT_ROOT>/pathfinder/MISSION-<MISSION_NUMBER>-BRIEF.md` (if it exists)
```

3. Replace the MT-3c Reviewer context line (line 230):

```diff
- > - Foundation: <PROJECT_ROOT>/pathfinder/PROJECT-FOUNDATION.md (read if it exists)
+ > - Foundation: <PROJECT_ROOT>/pathfinder/MISSION-<MISSION_NUMBER>-BRIEF.md (read if it exists)
```

4. Replace the MT-3d Drift Checker reference (line 305):

```diff
- 2. Spawn a "Drift Checker" Sonnet subagent to compare results against `<PROJECT_ROOT>/pathfinder/PROJECT-FOUNDATION.md` and all `DISCUSSION-*.md` files.
+ 2. Spawn a "Drift Checker" Sonnet subagent to compare results against `<PROJECT_ROOT>/pathfinder/MISSION-<MISSION_NUMBER>-BRIEF.md` and all `DISCUSSION-*.md` files.
```

5. Verify no remaining occurrences of `PROJECT-FOUNDATION.md` exist anywhere in the file (all four target lines at 95, 133, 230, 305 should be absent from grep results).

6. Stage and commit: `git commit -m "Implement task #42 sub-task 2: rename PROJECT-FOUNDATION.md to MISSION-<MISSION_NUMBER>-BRIEF.md in MT-2, MT-3b, MT-3c, MT-3d"`

## Verification

### Manual tests

- Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` and confirm:
  - Line ~95 (MT-2 Dependency Scout prompt) now reads `MISSION-<MISSION_NUMBER>-BRIEF.md`
  - Line ~133 (MT-3b Decomposer prompt) now reads `MISSION-<MISSION_NUMBER>-BRIEF.md`
  - Line ~230 (MT-3c Reviewer context) now reads `MISSION-<MISSION_NUMBER>-BRIEF.md`
  - Line ~305 (MT-3d Drift Checker prompt) now reads `MISSION-<MISSION_NUMBER>-BRIEF.md`

### Automated tests

- Grep the file for `PROJECT-FOUNDATION.md` and confirm zero occurrences remain; all four target lines should be absent from results.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | MISSION-<MISSION_NUMBER>-BRIEF.md` (if it exists).
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | <PROJECT_ROOT>/pathfinder/MISSION-<MISSION_NUMBER>-BRIEF.md` (if it exists). (MT-3b Decomposer line)
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Foundation: <PROJECT_ROOT>/pathfinder/MISSION-<MISSION_NUMBER>-BRIEF.md (read if it exists)`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | compare results against \`<PROJECT_ROOT>/pathfinder/MISSION-<MISSION_NUMBER>-BRIEF.md\``

## Changelog

### Review - 2026-03-25
- #1: Fixed garbled doc check on line 69 - removed copy-paste debris `(if it exists).\n> \n> Tasks in this mission` and mismatched trailing backtick from the first doc check string.

### Review - 2026-03-25
- #2: Blocking - added MT-3b Decomposer (line 133) as a fourth replacement target; the plan omitted it despite there being no follow-up sub-task to handle it, which would leave the Decomposer silently reading nothing after the file is renamed. Updated Context, Approach, Critical Files, Steps 2.5/5/6, Verification (manual tests, automated tests, doc checks) accordingly.

## Progress
- Step 1: Read pathfinder-mission-team/SKILL.md in full; confirmed all four PROJECT-FOUNDATION.md occurrences at lines 95, 133, 230, 305.
- Step 2: Replaced MT-2 Dependency Scout reference (line 95) with MISSION-<MISSION_NUMBER>-BRIEF.md.
- Step 2.5: Replaced MT-3b Decomposer reference (line 133) with MISSION-<MISSION_NUMBER>-BRIEF.md.
- Step 3: Replaced MT-3c Reviewer context line (line 230) with MISSION-<MISSION_NUMBER>-BRIEF.md.
- Step 4: Replaced MT-3d Drift Checker reference (line 305) with MISSION-<MISSION_NUMBER>-BRIEF.md.
- Step 5: Grep confirmed zero remaining occurrences of PROJECT-FOUNDATION.md in the file.
