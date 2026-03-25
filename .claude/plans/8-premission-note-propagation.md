# Plan: 8-premission-note-propagation.md

## Task
#8 - Propagate Pre-Mission Clarification Notes into PM-4 and PM-5

## Context

PM-1.5 captures user answers to clarification questions and appends them as "Pre-mission note: <answer>" to each task's entry in the confirmed list. However, neither PM-4 (Foundation Author subagent) nor PM-5 (acceptance-criteria prompts) reference these notes. As a result, the clarification work done in PM-1.5 is silently discarded and does not inform the foundation constraints or the acceptance-criteria questions asked in PM-5.

## Approach

Make two targeted edits to the PM-4 and PM-5 sections of `pathfinder-premission/SKILL.md`:

1. **PM-4**: Inject the "Pre-mission notes" into the Foundation Author subagent prompt so the subagent can incorporate them into the Constraints and Requirements sections of PROJECT-FOUNDATION.md.
2. **PM-5**: Include any "Pre-mission note:" text for each task in the AskUserQuestion format string, so the user can see what was already clarified when specifying acceptance criteria.

No new files, no structural changes to the skill flow.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md`
  - PM-4 Foundation Author subagent prompt: lines 118-148
  - PM-5 AskUserQuestion format string: lines 155-176

## Reuse

- The confirmed task list (built incrementally through PM-1 and PM-1.5) already holds the "Pre-mission note:" strings. No new data structure needed; the planner/commander just needs to serialize them into the subagent prompts.

## Steps

1. **Edit PM-4 Foundation Author subagent prompt** to pass Pre-mission notes.

   In the Foundation Author subagent prompt (after the instruction to read TASKS.md and DISCUSSION-*.md files), add a block that passes any pre-mission clarification notes collected during PM-1.5, and update the Constraints section instruction to mention them explicitly.

   **Commander construction note**: Before spawning the Foundation Author subagent, the Commander must dynamically build the pre-mission notes block by iterating over the confirmed task list and collecting all entries that have a "Pre-mission note:" annotation. For each such task, emit a line `"Task #<N>: <answer>"` (using only the answer text after the "Pre-mission note: " prefix). Omit the entire block if no tasks have notes.

   ```diff
   -   > You are the Foundation Author. Read these files:
   -   > - `<PROJECT_ROOT>/.claude/TASKS.md`
   -   > - All `DISCUSSION-*.md` files in `<PROJECT_ROOT>/.claude/plans/` (use Glob to find them; skip if none exist)
   +   > You are the Foundation Author. Read these files:
   +   > - `<PROJECT_ROOT>/.claude/TASKS.md`
   +   > - All `DISCUSSION-*.md` files in `<PROJECT_ROOT>/.claude/plans/` (use Glob to find them; skip if none exist)
   +   >
   +   > Additionally, the following pre-mission clarification notes were captured from the user for ambiguous tasks. Treat each note as a direct constraint or requirement for its task:
   +   > <for each task that has a Pre-mission note, insert a line: "Task #<N>: <Pre-mission note text>">
   +   > (Omit this section if no tasks had clarification notes.)
   ```

   And update the Constraints section description:

   ```diff
   -   > ## Constraints
   -   > <Derived from DISCUSSION-*.md constraints and anti-goals; if no discussion files exist, derive from task descriptions>
   +   > ## Constraints
   +   > <Derived from DISCUSSION-*.md constraints and anti-goals, plus any pre-mission clarification notes above; if neither exists, derive from task descriptions>
   ```

2. **Edit PM-5 AskUserQuestion format string** to surface Pre-mission notes.

   In the format string used to ask acceptance criteria per task, add the Pre-mission note after the full task description so the user sees what was already clarified.

   **Commander construction note**: For each task, check whether a "Pre-mission note:" entry exists in the confirmed list. If it does, insert `"Clarification already captured: <answer>\n\n"` (using only the answer text after the "Pre-mission note: " prefix) immediately before the closing question. If the task has no note, omit that segment entirely.

   ```diff
   -   Format: "Task #<N> [D:<score> C:<score>] - <title>\n\n<full description>\n\nWhat does 'done' look like? Select or describe 1-3 acceptance criteria."
   +   Format (task with note):    "Task #<N> [D:<score> C:<score>] - <title>\n\n<full description>\n\nClarification already captured: <answer>\n\nWhat does 'done' look like? Select or describe 1-3 acceptance criteria."
   +   Format (task without note): "Task #<N> [D:<score> C:<score>] - <title>\n\n<full description>\n\nWhat does 'done' look like? Select or describe 1-3 acceptance criteria."
   ```

## Verification

### Manual tests

1. Run `/pathfinder-premission` on a project that has at least one task where `D - C > 0` (i.e., a clarification candidate).
2. Confirm that PM-1.5 fires and the user's answer is recorded as a "Pre-mission note".
3. After PM-4 runs, open `PROJECT-FOUNDATION.md` and verify the Constraints section mentions the substance of the pre-mission note.
4. When PM-5 asks acceptance criteria, confirm the AskUserQuestion text includes the "Clarification already captured:" line for the task that had a note.
5. For tasks with no clarification note, confirm no spurious "Clarification already captured:" text appears.

### Automated tests

No automated test harness exists for this skill. The two Doc checks below cover the structural text changes:

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | pre-mission clarification notes were captured`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | Clarification already captured:`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | plus any pre-mission clarification notes above`

## Changelog

### Review - 2026-03-24
- #1: Added Commander construction note to Step 1 (PM-4) clarifying runtime note-block assembly and "Pre-mission note: " prefix stripping
- #2: Replaced ambiguous inline conditional in Step 2 (PM-5) format string with explicit Commander construction note and two-variant diff (with-note / without-note)

## Progress
- Step 1: Added pre-mission clarification notes block to PM-4 Foundation Author prompt and updated Constraints section description
- Step 2: Updated PM-5 AskUserQuestion format string with two variants (with/without note) and Commander construction instructions
