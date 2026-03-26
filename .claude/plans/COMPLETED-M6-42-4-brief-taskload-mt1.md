## Task
#42 - Rename PROJECT-FOUNDATION to MISSION-#-BRIEF in both skills

## Context
MT-1 step 2 currently only parses ARGUMENTS as a space-separated list of `#N` tokens (e.g. `#43 #46 #51`). The fourth criterion for task #42 requires that running `/pathfinder-mission-team MISSION-6-BRIEF` (or `MISSION-6-BRIEF.md`) loads the task list from that file's `## Task Priority Order` section, so the user does not need to re-enter task numbers manually after premission produces the BRIEF file.

The premission skill already writes a `## Task Priority Order` section in `MISSION-<N>-BRIEF.md` with entries of the form `- #N - <title>`, in confirmed priority order. MT-1 does not yet detect this filename pattern and falls back to treating the whole argument as a single invalid task token.

## Approach
Add a BRIEF-filename detection branch at the top of MT-1 step 2, before the existing `#N` token parse. If ARGUMENTS matches the BRIEF filename pattern, read the file, extract the task IDs from `## Task Priority Order`, and use them (in file order) as TASK_LIST. If ARGUMENTS does not match, proceed with the existing parse unchanged.

## Critical Files
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - MT-1 step 2 (line 21-22)

## Reuse
- The existing TASK_LIST variable and subsequent steps (2a validation, T computation, etc.) remain unchanged; this branch only populates TASK_LIST differently.

## Steps

1. Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` and locate MT-1 step 2 (currently the paragraph starting with "Parse ARGUMENTS into a TASK_LIST").

2. Replace the current step 2 paragraph with the revised version shown below. The detection branch is inserted before the existing parse logic; everything after TASK_LIST is built remains the same.

```diff
-2. Parse ARGUMENTS into a TASK_LIST (e.g. `#34 #71 #72` or `#34`). Read `<PROJECT_ROOT>/.claude/TASKS.md` to get task descriptions and priority/difficulty scores.
-   Compute T = sum of D scores for all tasks in TASK_LIST (parsed from the `[D:N ...]` annotation in TASKS.md). If a task has no D score annotation, treat its D as 0.
+2. Build TASK_LIST from ARGUMENTS using the following logic:
+
+   **2-A. BRIEF filename detection.**
+   Strip any leading/trailing whitespace from ARGUMENTS. If the result contains no `#` character and matches the pattern `MISSION-<digits>-BRIEF` or `MISSION-<digits>-BRIEF.md` (i.e. a single token with no spaces, no `#`), treat ARGUMENTS as a BRIEF filename:
+   - Normalize to `<filename>.md` if the `.md` extension is absent.
+   - Glob `<PROJECT_ROOT>/pathfinder/<filename>.md`. If the file does not exist, halt with: "BRIEF file not found: <PROJECT_ROOT>/pathfinder/<filename>.md — verify the filename and retry."
+   - Read the file. Locate the `## Task Priority Order` section. Extract every line matching `- #<N>` (one task ID per line, in file order). These task IDs become TASK_LIST, in the order they appear in the file.
+   - If the section is absent or yields no task IDs, halt with: "No task IDs found in ## Task Priority Order section of <filename>.md."
+
+   **2-B. Token list parse (existing behavior).**
+   If ARGUMENTS is not a BRIEF filename (i.e. it contains `#` or does not match the BRIEF pattern), parse it as a space-separated list of `#N` tokens (e.g. `#34 #71 #72` or `#34`).
+
+   After TASK_LIST is built (by either 2-A or 2-B), read `<PROJECT_ROOT>/.claude/TASKS.md` to get task descriptions and priority/difficulty scores.
+   Compute T = sum of D scores for all tasks in TASK_LIST (parsed from the `[D:N ...]` annotation in TASKS.md). If a task has no D score annotation, treat its D as 0.
```

3. Verify no other step references the old step-2 wording in a way that would conflict (e.g. inline comments about "parsing ARGUMENTS"). A quick read of steps 2a onward confirms they consume TASK_LIST, not raw ARGUMENTS, so no further edits are needed.

4. Read the modified SKILL.md from the top of MT-1 through step 3 to confirm the new wording integrates cleanly (no duplicate blank lines, no broken numbering).

## Verification

### Manual tests
- Run `/pathfinder-mission-team MISSION-6-BRIEF` in the scribblenot project (after premission has produced `pathfinder/MISSION-6-BRIEF.md` with a `## Task Priority Order` section). Confirm that the mission log's Tasks line lists the task IDs from the file in priority order, not a single malformed token.
- Run `/pathfinder-mission-team MISSION-6-BRIEF.md` (with explicit `.md`) and confirm identical behavior.
- Run `/pathfinder-mission-team #43 #46` and confirm the existing token-list path still works normally.
- Run `/pathfinder-mission-team MISSION-6-BRIEF` when no BRIEF file exists and confirm the skill halts with the "file not found" message rather than silently producing an empty task list.

### Automated tests
- Unit-style check: construct a minimal BRIEF markdown string with a `## Task Priority Order` section containing `- #43 - Drop UTC offset` and `- #46 - Sub-task log enforcement`, then apply the extraction regex to confirm the output is `["#43", "#46"]` in order.
- Regression check: confirm that a string like `#43 #46` still produces `["#43", "#46"]` via the token-list path (no BRIEF detection triggered).

### Doc checks
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | BRIEF filename detection`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Task Priority Order`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | 2-A.`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | 2-B.`

## Changelog

### Review - 2026-03-25
- #1 (blocking): TASKS.md read and T computation were inside the 2-B conditional block only; when the BRIEF path (2-A) was taken, T would be undefined at step 2b. Fixed by prefixing the TASKS.md read sentence with "After TASK_LIST is built (by either 2-A or 2-B)," making it unconditional.

## Progress
- Step 1: Located MT-1 step 2 in pathfinder-mission-team/SKILL.md (lines 21-22)
- Step 2: Replaced old step-2 paragraph with 2-A BRIEF detection branch + 2-B token-list parse + unconditional TASKS.md read
- Step 3: Confirmed no other steps reference old "Parse ARGUMENTS" wording; 2a/2b consume TASK_LIST not raw ARGUMENTS
- Step 4: Verified MT-1 lines 18-61 integrate cleanly: no duplicate blank lines, numbering intact (1, 2, 2a, 2b, 3, 4, 5)
