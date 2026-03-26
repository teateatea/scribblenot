## Task

#58 - Resolve collision between TASKS.md sub-entry format (#N-2) and pathfinder sub-task nomenclature

## Context

TASKS.md uses `#N-2` and `#N-3` suffixes for supplementary clarification/context entries nested under a parent task. The pathfinder skill suite (pathfinder-mission-team and pathfinder-premission) reads TASKS.md at multiple points to extract task IDs, descriptions, and D/C scores. If any parsing code treats `#N-2` as a task ID rather than a formatting artifact, it could misinterpret the entry as a decomposed sub-task from a prior run, causing incorrect task-list population, re-queue behavior, or difficulty score miscalculation.

This sub-task is a documentation/research deliverable. Its output informs sub-tasks 2 (fix parsing) and 3 (fix premission handling). No file edits are made here.

## Approach

Audit TASKS.md to enumerate every `#N-2` / `#N-3` entry, confirm their indentation level and format, then trace the exact code paths in the two skills that read TASKS.md to determine whether a real collision exists today or only a latent risk.

## Critical Files

- `C:/Users/solar/Documents/Claude Projects/scribblenot/.claude/TASKS.md` - source of truth for entry formats
- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - MT-1 step 2 and MT-2 read TASKS.md
- `C:/Users/solar/.claude/skills/pathfinder-premission/SKILL.md` - PM-1 step 2-3 reads TASKS.md

## Reuse

No implementation code involved. This step produces findings in the Steps section below that sub-tasks 2 and 3 will act on.

## Steps

### Finding A: All #N-2 / #N-3 entries in TASKS.md and their format

TASKS.md (as of the audit) contains exactly two suffixed entries:

**Entry #34-2** (lines 50-53):
- Indentation: 2 spaces before `- [ ]` (one level in from its parent `- [ ] **#34**` at column 0)
- Full prefix: `  - [ ] **#34-2**`
- Label: "Staged multi-premission briefings and --auto chain (pre-mission clarified)"
- D score annotation: `[D:65 C:60]`
- Parent task: `#34` (appears 5 lines above, top-level)

**Entry #72-2** (lines 109-112):
- Indentation: 2 spaces before `- [ ]` (one level in from its parent `- [ ] **#72**` at column 0)
- Full prefix: `  - [ ] **#72-2**`
- Label: "Reviewer opens diffs intermittently -- behavior not consistent across all tasks"
- D score annotation: `[D:25 C:28]`
- Parent task: `#72` (appears 4 lines above, top-level)

**Format distinction - top-level vs sub-entry:**
- Top-level task: `- [ ] **#N**` at column 0 (no leading spaces)
- Sub-entry: `  - [ ] **#N-2**` with 2 leading spaces (indented one level)

There are no `#N-3` entries currently. The two sub-entries are the only instances of the `#N-<digit>` pattern in TASKS.md.

### Finding B: Code paths in pathfinder-mission-team that read TASKS.md

**MT-1 step 2-B (token list parse):** When ARGUMENTS contains `#` tokens, the Commander parses them as a space-separated list of `#N` tokens. TASKS.md is then read to get descriptions and `[D:N C:N]` scores for each ID in TASK_LIST. If a user passes `#34-2` explicitly as an argument token, the Commander would attempt to look it up as a task and would find it (since it exists in TASKS.md). This is a real -- though user-triggered -- collision path.

**MT-1 step 2-A (BRIEF filename parse):** Extracts lines matching `- #<N>` from the BRIEF file's `## Task Priority Order` section. This regex pattern (`- #<N>`) expects a plain integer after `#`, so `#34-2` would only match if the BRIEF file explicitly listed it. A correctly authored BRIEF would only list parent task IDs. Low risk if BRIEF authoring is correct.

**MT-1 step 2 (D score computation):** After TASK_LIST is built, TASKS.md is scanned for `[D:N ...]` annotations for each task in TASK_LIST. If `#34-2` were in TASK_LIST, its `[D:65 C:60]` annotation would be found and counted toward T (total difficulty). This would inflate the estimated duration.

**MT-2 Dependency Scout:** The subagent prompt reads TASKS.md and the BRIEF. It is asked to build a dependency DAG for tasks in TASK_LIST. If `#34-2` appeared in TASK_LIST, the Scout would see it as a task and might incorrectly infer or assign dependencies.

**MT-3b Decomposer:** Reads TASKS.md for the task description. If `#34-2` were selected, it would read the indented sub-entry as the task description -- which is a clarification note, not an implementation specification.

**MT-3 PRIOR_ATTEMPT_MAP:** The suffix pattern `#N-<digit>` is also used internally by the mission team's retry logic (e.g., sub-task IDs like "sub-task 2 of task #34" in log entries). However, this numbering lives in the MISSION-LOG and JSON structures, not in TASKS.md. The collision is a naming convention overlap, not a structural one in the data.

### Finding C: Code paths in pathfinder-premission that read TASKS.md

**PM-1 step 2:** Reads TASKS.md unconditionally to get the full task list.

**PM-1 step 3:** If ARGUMENTS names specific task numbers (e.g., `#34-2`), extracts those tasks from TASKS.md. A user accidentally passing `#34-2` to premission would cause it to treat the sub-entry as a full task candidate, display it in the confirmation table with its D/C scores, and proceed to the D/C threshold check. This is a real -- though user-triggered -- path.

If ARGUMENTS is empty, PM-1 lists all incomplete tasks via AskUserQuestion multi-select. The sub-entries (`#34-2`, `#72-2`) would appear in that multi-select list alongside top-level tasks. A user could inadvertently select them. This is a real latent risk even without explicit argument passing.

### Finding D: Is there a real collision today or only potential risk?

**Today (no explicit sub-entry selection):** If users only pass top-level task IDs to premission and mission-team, no collision occurs. The sub-entries are never selected into TASK_LIST, so the parsing code never encounters them as task IDs. The risk is latent, not active.

**Real collision paths (triggerable today):**
1. PM-1 step 3 (ARGUMENTS empty) -- sub-entries appear in the multi-select list and could be selected by the user.
2. PM-1 step 3 (ARGUMENTS with `#34-2`) -- sub-entry treated as a full task.
3. MT-1 step 2-B (ARGUMENTS with `#34-2`) -- sub-entry treated as a full task for the mission.

**Naming convention overlap:** Internally, pathfinder logs sub-tasks with IDs like `1`, `2`, `3` per task (not `#N-2` style), so there is no structural collision in log files. The overlap is purely in the TASKS.md token namespace.

**Conclusion:** The collision is not happening autonomously during normal missions. It is a latent parsing risk triggered when sub-entries leak into task selection -- most likely through the PM-1 empty-ARGUMENTS multi-select path. Sub-tasks 2 and 3 should address: (2) filtering sub-entries out of TASKS.md parsing so they are never presented as selectable tasks; (3) confirming premission correctly skips indented entries during list display.

## Verification

### Manual tests

- After sub-tasks 2 and 3 are implemented: run `/pathfinder-premission` with no arguments and confirm `#34-2` and `#72-2` do not appear in the multi-select task list.
- Pass `#34-2` explicitly as an argument to `/pathfinder-premission` and confirm an appropriate error or skip (not treatment as a full task).

### Automated tests

This sub-task produces no code changes, so no automated tests apply here. Sub-tasks 2 and 3 will introduce parsing changes that can be unit-tested by feeding a mock TASKS.md string containing indented sub-entries and asserting they are excluded from the returned task ID list.

### Doc checks

`C:/Users/solar/Documents/Claude Projects/scribblenot/.claude/plans/M7-58-1-nN2-collision-audit.md | contains | #34-2`
`C:/Users/solar/Documents/Claude Projects/scribblenot/.claude/plans/M7-58-1-nN2-collision-audit.md | contains | indented sub-entry`

## Prefect-1 Report

### Fixes applied

- **nit** `M7-58-1-nN2-collision-audit.md:53` - Label "MT-1 step 2 (token list parse)" corrected to "MT-1 step 2-B (token list parse)" to match the actual skill section name "2-B. Token list parse (existing behavior)" and be consistent with the adjacent "MT-1 step 2-A" bullet at line 55.

## Changelog

### Review - 2026-03-26
- #1 (nit): Corrected "#34 appears 8 lines above" to "5 lines above" to match actual TASKS.md line numbers (line 45 to line 50).

### Prefect-1 – 2026-03-26
- nit: Corrected step label "MT-1 step 2 (token list parse)" to "MT-1 step 2-B (token list parse)" in Finding B to match actual skill section naming.
