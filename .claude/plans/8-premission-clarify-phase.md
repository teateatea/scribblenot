# Plan: 8-premission-clarify-phase

## Task
#8 - premission clarification phase (PM-1.5)

## Context

The current PM-1 step 5 in `pathfinder-premission/SKILL.md` handles clarification candidates by asking one targeted question per task, one at a time (sequentially). The task requires a PM-1.5 phase that:
- Asks 1-3 targeted questions per clarification-candidate task (not just one)
- Batches up to 4 tasks per AskUserQuestion call (instead of one task at a time)
- Captures answers as task-level clarification notes for downstream phases

Sub-task 1 already inserted a threshold-check mechanism in PM-1 step 5 that asks one question per task sequentially. Sub-task 2 replaces that single-question sequential approach with a multi-question batched approach, either by modifying the existing PM-1 step 5 or by extracting it into a new PM-1.5 phase immediately after PM-1.

## Approach

Modify the clarification logic currently in PM-1 step 5 of `pathfinder-premission/SKILL.md`. Replace the single-question, one-at-a-time approach with a PM-1.5 phase block that:
1. Groups clarification candidates into batches of up to 4
2. For each batch, calls AskUserQuestion once with 1-3 questions per task (derived from task description, D score, and C score)
3. Appends all answers as "Pre-mission note:" entries on each task in the confirmed list

This keeps all clarification logic contiguous and avoids a disconnected mid-phase section.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` - lines 25-37 (PM-1 step 5, the clarification block to be replaced)

## Reuse

- Existing AskUserQuestion pattern from PM-5 (lines 139-148) which already demonstrates batching up to 4 tasks per call - reuse the same batching pattern and question format
- Existing "Pre-mission note:" convention from PM-1 step 5 line 34 - preserve this downstream annotation pattern

## Steps

1. Open `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md`.

2. Replace the clarification block in PM-1 step 5 (lines 25-37) and add a new PM-1.5 section between PM-1 and PM-2. The replacement removes the inline single-question logic from PM-1 step 5 and moves it to PM-1.5.

   Replace (PM-1 step 5, lines 25-37):
   ```
   5. **D/C threshold check.** After the user confirms the task list, compute `delta = D - C` for each task.
      - Tasks where `delta <= 0` are **fast-path**: D score does not exceed C score, meaning the task is well-understood. No extra questions needed.
      - Tasks where `delta > 0` are **clarification candidates**: difficulty exceeds clarity, raising the risk of mid-mission ambiguity.
      - Tasks with missing D or C scores are treated as **fast-path** (skip threshold check for that task).

      If any clarification candidates exist, for each one use AskUserQuestion to ask one targeted open-ended question that would most reduce ambiguity for that task. Frame the question as:

      > "Task #<N> [D:<d> C:<c>, delta=+<delta>] - <title>: <one targeted question addressing the biggest unknown>"

      Process clarification candidates one at a time (do not batch them). Append each answer to that task's entry in the confirmed list as a "Pre-mission note:" so downstream phases (PM-2 through PM-6) can reference it.

      Fast-path tasks require no questions - proceed directly to PM-2.
   ```

   With:
   ```
   5. **D/C threshold check.** After the user confirms the task list, compute `delta = D - C` for each task.
      - Tasks where `delta <= 0` are **fast-path**: D score does not exceed C score, meaning the task is well-understood.
      - Tasks where `delta > 0` are **clarification candidates**: difficulty exceeds clarity, raising the risk of mid-mission ambiguity.
      - Tasks with missing D or C scores are treated as **fast-path** (skip threshold check for that task).

      If there are no clarification candidates, skip PM-1.5 and proceed directly to PM-2.

   ---

   ### PM-1.5: Clarify ambiguous tasks

   For each clarification candidate, derive 1-3 targeted questions that would most reduce ambiguity. Base each question on:
   - The task description (what is unclear or underspecified)
   - The D score (high D = complex implementation details to clarify)
   - The C score (low C = goals or scope need clarification)

   Group clarification candidates into batches of up to 4. For each batch, make one AskUserQuestion call. Format each question within the batch as:

   > "Task #<N> [D:<d> C:<c>, delta=+<delta>] - <title>
   > Q1: <first targeted question>
   > Q2: <second targeted question, if warranted>
   > Q3: <third targeted question, if warranted>"

   Only include Q2 and Q3 when genuinely needed (e.g. when delta is large or the task description has multiple unknowns). Do not pad with filler questions.

   After each AskUserQuestion response, append the answers to each task's entry in the confirmed list as "Pre-mission note: <answer>" so PM-2 through PM-6 can reference them.

   Process all batches before proceeding to PM-2.
   ```

3. Verify the edit: read `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` and confirm:
   - PM-1 step 5 ends with "skip PM-1.5 and proceed directly to PM-2" (fast-path exit)
   - PM-1.5 section exists between PM-1 and PM-2
   - PM-1.5 references batching up to 4 tasks per AskUserQuestion call
   - PM-1.5 references 1-3 questions per task
   - "Pre-mission note:" convention is preserved

## Verification

### Manual tests
- Run `/pathfinder-premission` on a TASKS.md that has at least 5 tasks with varying D/C scores:
  - Tasks with delta <= 0 should skip PM-1.5 questions entirely
  - Tasks with delta > 0 should appear in PM-1.5 with 1-3 questions each
  - If there are 5 clarification candidates, they should be batched as one call of 4 + one call of 1 (not 5 sequential single-task calls)
  - Answers should appear as "Pre-mission note:" in the confirmed task list used by PM-2 onward
- Run `/pathfinder-premission` on a TASKS.md where all tasks are fast-path (delta <= 0): PM-1.5 should be skipped entirely with no AskUserQuestion calls

### Automated tests
- None applicable: skill behavior is driven by LLM instruction following, not unit-testable code paths

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | PM-1.5: Clarify ambiguous tasks`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | batches of up to 4`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | 1-3 targeted questions`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | missing | Process clarification candidates one at a time (do not batch them)`

## Changelog

### Review - 2026-03-24
- #1 (minor): Doc check string "Batch up to 4" did not match proposed replacement text; updated to "batches of up to 4" which is the exact phrase used in the PM-1.5 block.

## Progress
- Step 1: Opened `pathfinder-premission/SKILL.md` and read it in full
- Step 2: Replaced PM-1 step 5 single-question sequential logic with fast-path exit + new PM-1.5 batched clarification phase
- Step 3: Verified all 4 doc checks pass (PM-1.5 present, batches of up to 4 present, 1-3 targeted questions present, old one-at-a-time text absent)
