# Plan: 8-premission-dc-threshold

## Task
#8 - Add D/C threshold check after PM-1 task confirmation step

## Context
PM-1 step 4 confirms a prioritized task list with the user, but all tasks proceed identically from that point forward regardless of complexity. Tasks where D (difficulty) is greater than C (clarity) are ambiguous - the implementer will likely encounter unknowns mid-execution that would require user interaction. Since the mission runs dark (no user interaction), these tasks need targeted clarifying questions before launch. Tasks where D <= C are already well-understood and need no extra questions.

## Approach
Insert a new threshold-check block at the end of PM-1 (after the user confirms the prioritized list) that:
1. Computes `delta = D - C` for each task.
2. Partitions tasks into fast-path (delta <= 0) and clarification-candidates (delta > 0).
3. If any clarification-candidates exist, presents them to the user and asks one targeted question per candidate before proceeding to PM-2.

No new files. One section added to the existing SKILL.md.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` - PM-1 section ends at line 23; new block inserts after that line.

## Reuse
- The existing `AskUserQuestion` tool (already in `allowed-tools`) handles per-task clarification prompts.
- D and C scores are already gathered in PM-1 step 4 (displayed in the confirmation table), so no new reads are needed.

## Steps

1. Open `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` for editing.

2. After the line:
   ```
   4. Confirm the prioritized list with the user: ordered highest priority first; within the same priority tier, hardest first by D score (if available in TASKS.md). Present the ordered list as a table with full details (task number, D score, C score, one-line summary, and full description) in priority order, and ask the user to confirm or reorder before continuing.
   ```
   Insert the following new step (keeping the blank line before `---` that separates PM-1 from PM-2):

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

   The unified diff for this change is:

   ```diff
   -4. Confirm the prioritized list with the user: ordered highest priority first; within the same priority tier, hardest first by D score (if available in TASKS.md). Present the ordered list as a table with full details (task number, D score, C score, one-line summary, and full description) in priority order, and ask the user to confirm or reorder before continuing.
   -
   ----
   +4. Confirm the prioritized list with the user: ordered highest priority first; within the same priority tier, hardest first by D score (if available in TASKS.md). Present the ordered list as a table with full details (task number, D score, C score, one-line summary, and full description) in priority order, and ask the user to confirm or reorder before continuing.
   +
   +5. **D/C threshold check.** After the user confirms the task list, compute `delta = D - C` for each task.
   +   - Tasks where `delta <= 0` are **fast-path**: D score does not exceed C score, meaning the task is well-understood. No extra questions needed.
   +   - Tasks where `delta > 0` are **clarification candidates**: difficulty exceeds clarity, raising the risk of mid-mission ambiguity.
   +   - Tasks with missing D or C scores are treated as **fast-path** (skip threshold check for that task).
   +
   +   If any clarification candidates exist, for each one use AskUserQuestion to ask one targeted open-ended question that would most reduce ambiguity for that task. Frame the question as:
   +
   +   > "Task #<N> [D:<d> C:<c>, delta=+<delta>] - <title>: <one targeted question addressing the biggest unknown>"
   +
   +   Process clarification candidates one at a time (do not batch them). Append each answer to that task's entry in the confirmed list as a "Pre-mission note:" so downstream phases (PM-2 through PM-6) can reference it.
   +
   +   Fast-path tasks require no questions - proceed directly to PM-2.
   +
   +---
   ```

## Verification

### Manual tests
- Run `/pathfinder-premission` on a project whose TASKS.md has at least one task with D > C and one with D <= C.
- Confirm that only the high-delta task receives a clarifying question.
- Confirm that the fast-path task is not questioned and that PM-2 proceeds immediately after.
- Confirm that the Pre-mission note is appended to the task's entry in the confirmed list so it is available in context for downstream phases (PM-2 through PM-6). Note: PM-5's explicit prompt template does not include pre-mission notes verbatim, but the note is present in the session context.

### Automated tests
- No automated test harness exists for skill `.md` files. A shell script could run `/pathfinder-premission` in dry-run mode (if supported) and assert the presence of the new step text in the rendered output.

### Doc checks
- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | D/C threshold check`
- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | clarification candidates`
- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | Pre-mission note:`
- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | fast-path`

## Changelog

### Review - 2026-03-24
- #1 (nit): Fixed typo in Doc checks - `.claire` corrected to `.claude` in the fast-path path reference (line 86)

### Review #2 - 2026-03-24
- #1 (minor): Added missing-scores fallback rule to step 5 instruction text and diff block - tasks with absent D or C scores are treated as fast-path
- #2 (minor): Corrected inaccurate PM-5 verification claim - pre-mission note is in session context but PM-5's explicit prompt template does not include it verbatim

## Progress
- Step 1: Opened pathfinder-premission/SKILL.md for editing (read complete)
- Step 2: Inserted new PM-1 step 5 D/C threshold check block after step 4, before the PM-2 separator

## Implementation
Complete - 2026-03-24
