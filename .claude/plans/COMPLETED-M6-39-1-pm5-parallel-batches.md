## Task

#39 - Parallelize PM-5 batch question generation with subagents when task count > 4

## Context

PM-5 asks users for acceptance criteria in batches of 4 via AskUserQuestion. When there are more than 4 tasks, the main instance sits idle between batches while it assembles the next group's question text. The user notices a lag between each batch. The fix is to prepare all question texts upfront in parallel subagents, then present them sequentially -- eliminating per-batch idle time.

## Approach

Add a conditional branch at the start of PM-5. When task count > 4, spawn one Sonnet subagent per batch of 4 tasks in parallel; each subagent returns only the formatted question strings for its batch. The main instance collects all subagent outputs, then loops through them in order calling AskUserQuestion once per batch. When task count <= 4, the existing single-batch behavior is unchanged (no subagents, single AskUserQuestion call).

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` (lines 185-211, PM-5 section)

## Reuse

- Existing PM-5 question format strings (lines 193-194 of SKILL.md) -- subagents must use the exact same format
- Existing AskUserQuestion batching logic (up to 4 tasks per call) -- unchanged

## Steps

1. Open `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` and locate the PM-5 section (line 185).

2. Replace only the opening sentence of the PM-5 section body (line 187: "For each task in the mission list, use AskUserQuestion...") with the following logic. All subsequent lines of PM-5 (format strings, batching rule, PROJECT-TESTS.md block) remain unchanged.

- **When task count <= 4 (unchanged path):** proceed exactly as before -- directly compose question text for each task and make a single AskUserQuestion call with all tasks batched.

- **When task count > 4 (new parallel path):**

  a. Divide the confirmed task list into batches of up to 4 (same batching as before).

  b. For each batch, spawn one "PM-5 Question Builder for batch <B>" Sonnet subagent in parallel. Use this prompt (substitute actual values):

  > You are the PM-5 Question Builder for batch <B>. Format the acceptance-criteria question text for each of the following tasks. Do NOT call AskUserQuestion. Return only the list of formatted question strings, one per task, separated by `---QUESTION---`.
  >
  > For each task, use the appropriate format:
  >
  > With pre-mission note:
  > "Task #<N> [D:<score> C:<score>] - <title>\n\n<full description>\n\nClarification already captured: <answer>\n\nWhat does 'done' look like? Select or describe 1-3 acceptance criteria."
  >
  > Without pre-mission note:
  > "Task #<N> [D:<score> C:<score>] - <title>\n\n<full description>\n\nWhat does 'done' look like? Select or describe 1-3 acceptance criteria."
  >
  > Tasks:
  > <for each task in this batch: task number, title, D score, C score, full description, and pre-mission note if present>
  >
  > Do NOT use AskUserQuestion. Return only the formatted question strings separated by `---QUESTION---`.

  c. Wait for all subagents to return before presenting any questions to the user.

  d. Loop through the collected batches in order. For each batch, call AskUserQuestion once with the pre-built question strings (up to 4 per call), exactly as the original PM-5 does.

3. Also update line 196 of the same file to clarify which actor performs the pre-mission note check:

```
-For each task, check whether a "Pre-mission note:" entry exists in the confirmed list. If it does, use the with-note format and substitute <answer> with the answer text after the "Pre-mission note: " prefix. If the task has no note, use the without-note format.
+For each task (main instance in the <= 4 path; subagent in the > 4 path), check whether a "Pre-mission note:" entry exists in the confirmed list. If it does, use the with-note format and substitute <answer> with the answer text after the "Pre-mission note: " prefix. If the task has no note, use the without-note format.
```

4. The rest of PM-5 (collecting criteria and writing PROJECT-TESTS.md) is unchanged.

The diff for the PM-5 section heading and opening paragraph:

```
-For each task in the mission list, use AskUserQuestion to ask the user for 1-3 high-level acceptance criteria. The question text must include:
+**When task count <= 4:** proceed directly to the AskUserQuestion calls below (no subagents).
+
+**When task count > 4:** First, divide the confirmed task list into batches of up to 4 tasks. Spawn one "PM-5 Question Builder for batch <B>" Sonnet subagent per batch in parallel. Each subagent receives the task details for its batch and returns the formatted question strings (one per task, separated by `---QUESTION---`) without calling AskUserQuestion. Wait for all subagents to return. Then proceed to the AskUserQuestion loop below using the pre-built strings.
+
+For each task (or for each pre-built question string when using the parallel path), the question text must include:
```

## Verification

### Manual tests

1. Run `/pathfinder-premission` with a mission containing 5+ tasks. Verify that after confirming the task list (PM-1), all PM-5 AskUserQuestion prompts for later batches appear without a noticeable delay between batches (subagents should have prepared them during the first batch's user-answering time).
2. Run `/pathfinder-premission` with a mission containing exactly 4 tasks. Verify behavior is identical to pre-change (single AskUserQuestion call, no subagents spawned).
3. Verify PROJECT-TESTS.md is written correctly in both cases with all tasks and their user-supplied criteria.

### Automated tests

No automated test harness exists for skill `.md` files. The change is doc-only (SKILL.md is instruction prose), so runtime behavior is validated manually above.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | When task count > 4`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | PM-5 Question Builder for batch`
`C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md | contains | When task count <= 4`

## Prefect-2 Report

### Minor

- **#1** `SKILL.md:196` / plan diff: After the diff is applied, line 196 of SKILL.md still reads "For each task, check whether a 'Pre-mission note:' entry exists in the confirmed list." This sentence is directed at "the main instance" in the original flow, but in the parallel path (> 4 tasks) the check must be performed inside each subagent -- not by the main instance. The subagent prompt (plan lines 41-47) does handle both formats correctly, so the runtime behavior is fine, but the surviving prose at line 196 remains ambiguous about which actor is responsible for the pre-mission-note check in the parallel path. Consider either (a) extending the diff to update line 196 to say "For each task (or for each subagent in the parallel path), check whether..." or (b) adding a note in the subagent prompt that explicitly calls out this responsibility so an implementer reading the applied SKILL.md does not misread line 196 as a main-instance responsibility in both paths.

## Prefect-1 Report

All issues found and fixed directly. No blocking or minor issues.

### Nit

- **#1** `M6-39-1-pm5-parallel-batches.md` Reuse section, line 19: Line range citation said "lines 193-196" for PM-5 format strings. Actual format strings are lines 193-194; lines 195-196 are the conditional-logic prose ("For each task, check whether..."). Fixed to "lines 193-194".

## Changelog

### Review - 2026-03-25
- #1: Clarified step 2 to say only the opening sentence (line 187) is replaced, not the full PM-5 section body; all format strings and subsequent lines remain unchanged.

### Prefect-1 – 2026-03-25
- #1 (nit): Corrected line range citation in Reuse section from "lines 193-196" to "lines 193-194"; lines 195-196 are the conditional-logic prose, not format strings.

### Review #3 – 2026-03-25
- #1 (minor): Added step 3 diff to update SKILL.md line 196 to clarify that the pre-mission-note check is performed by the main instance in the <= 4 path and by the subagent in the > 4 path, resolving the ambiguity flagged in the Prefect-2 Report.

## Progress
- Step 1: Located PM-5 section in pathfinder-premission/SKILL.md at line 185.
- Step 2: Replaced opening sentence of PM-5 with conditional parallel/sequential logic (task count <= 4 vs > 4 with subagent spawning).
- Step 3: Updated line 196 to clarify that pre-mission note check is performed by main instance (<=4 path) or subagent (>4 path).

## Implementation
Complete - 2026-03-25
