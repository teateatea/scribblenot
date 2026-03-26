## Task

#58 - Resolve collision between TASKS.md sub-entry format (#N-2) and pathfinder sub-task nomenclature

## Context

TASKS.md uses indented sub-entries like `#34-2` and `#72-2` as supplementary clarification/context notes nested under a parent task. PM-1 step 3 of the pathfinder-premission skill reads TASKS.md to build the user-facing multi-select task list. Because the current instruction does not filter by indentation or ID format, these sub-entries appear alongside top-level tasks and could be selected. Finding C of the sub-task 1 audit confirmed this is a real latent risk: in the empty-ARGUMENTS path, sub-entries appear in the multi-select list; in the explicit-ARGUMENTS path, passing `#34-2` causes it to be treated as a full task. The fix is a single prose addition to PM-1 step 3 instructing the skill to exclude any entry whose task ID token contains a hyphen-digit suffix (`#\d+-\d+`).

## Approach

Add one sentence to PM-1 step 3 in `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` that explicitly instructs the skill to skip any TASKS.md line whose task ID matches the pattern `#<digits>-<digits>` (i.e., a sub-entry) when building the multi-select list or resolving explicit ARGUMENTS. No code is involved; this is a prose-only change to a skill instruction document.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` - PM-1 step 3 (line 23) is the only location to edit

## Reuse

No code utilities involved. The filtering rule reuses the format distinction already documented in the sub-task 1 audit: top-level tasks are `- [ ] **#N**` at column 0; sub-entries are `  - [ ] **#N-2**` with 2 leading spaces. The new instruction can reference either the indentation criterion or the `#\d+-\d+` ID pattern - using the ID pattern is more robust since it does not depend on whitespace counting by the LLM.

## Steps

1. Open `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` and locate PM-1 step 3 (line 23):

   Current text:
   ```
   3. If ARGUMENTS names specific task numbers (e.g. `#34 #71`), extract those tasks from TASKS.md. If ARGUMENTS is empty, list all incomplete tasks and use AskUserQuestion (multi-select) to ask which tasks to include in the mission.
   ```

   Replace with:
   ```
   3. If ARGUMENTS names specific task numbers (e.g. `#34 #71`), extract those tasks from TASKS.md. If ARGUMENTS is empty, list all incomplete tasks and use AskUserQuestion (multi-select) to ask which tasks to include in the mission. In both cases, skip any entry whose task ID matches the pattern `#<digits>-<digits>` (e.g. `#34-2`, `#72-2`): these are sub-entries used for clarification context, not standalone tasks, and must never appear as selectable items.
   ```

## Verification

### Manual tests

- Run `/pathfinder-premission` with no arguments in the scribblenot project. Confirm the multi-select list does not include `#34-2` or `#72-2`.
- Run `/pathfinder-premission #34-2` explicitly. Confirm the skill either skips the sub-entry entirely (proceeds as if no task was specified) or returns an error indicating `#34-2` is not a valid task ID. It must not treat it as a full task.

### Automated tests

No automated test harness exists for skill SKILL.md prose. The change can be manually validated by the two steps above. A future integration test could feed a mock TASKS.md containing both top-level and indented sub-entries to a premission invocation and assert the returned candidate list contains only top-level IDs.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | skip any entry whose task ID matches the pattern`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | #34-2`, `#72-2`): these are sub-entries`

## Changelog

### Review - 2026-03-26
- #1: Replaced em-dash with colon in proposed replacement text (Rule #19 - no em-dashes)
- #2: Updated doc check string to match corrected replacement text (no em-dash)
