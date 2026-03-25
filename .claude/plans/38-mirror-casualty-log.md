## Task

#38 - Mirror casualty entries to numbered MISSION-LOG permission denials section

## Context

When a permission denial (hook exit non-zero) occurs during a mission, MT-3e currently instructs the Commander to "Log the denial to MISSION-LOG Permission Denials with full tool + input context." In practice, the denial event reaches MISSION-LOG-active.md (written by the hook), but the numbered MISSION-LOG file (MISSION_LOG_PATH, e.g. `MISSION-LOG-5-pathfinder-skill-overhaul.md`) does not receive a matching entry under its `## Permission Denials` section. This means the archived mission record is incomplete: a reader of the numbered log cannot see what was denied without also consulting MISSION-LOG-active.md.

## Approach

Update the MT-3e permission denial branch in `pathfinder-mission-team/SKILL.md` to explicitly append a `### Casualty N` entry to `MISSION_LOG_PATH` under `## Permission Denials`, mirroring the denial information already written to MISSION-LOG-active.md. The Commander maintains a per-mission casualty counter (CASUALTY_COUNT, initialized to 0 at MT-1) and increments it on each denial. The entry format extends the existing convention seen in `MISSION-LOG-5-pathfinder-skill-overhaul.md` (tool name, cause) by replacing the file-specific `- File:` field with the more general `- Input:` field and adding a `- Task:` field for traceability.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` - lines 327-334 (MT-3e blocker handling section; denial branch content lines 329-334)

## Reuse

- Existing numbered-log write pattern used throughout MT-3c step 5 (append sub-task entries to MISSION_LOG_PATH)
- Existing `### Casualty N` format established in `MISSION-LOG-5-pathfinder-skill-overhaul.md` Permission Denials section (lines 80-85). Note: the proposed entry extends the existing convention by replacing `- File:` with the more general `- Input:` field (tools are not always file-targeted) and adding a `- Task:` field for traceability.

## Steps

1. In `pathfinder-mission-team/SKILL.md`, locate the MT-1 initialization block (around line 111, where PLAN_FILES and PRIORITY_MAP are introduced). Add `CASUALTY_COUNT` to the list of tracked state variables, initialized to 0:

```diff
-Maintain a TASK_QUEUE (ordered list), a PRIORITY_MAP (task -> current priority score, initialized from TASKS.md), a D_MAP (task -> D score, initialized from TASKS.md alongside T in MT-1), a COMPLETED_D counter (running sum of D scores for all successfully completed tasks, initialized to 0), a PLAN_FILES map (task -> list of plan filenames produced during MT-3c for that task, initialized to `[]` per task in MT-3a), and a PRIOR_ATTEMPT_MAP (task -> list of prior-attempt records, initialized to `[]` per task; each record is added on the MT-3d failure branch and contains the sub-tasks that ran and the project-test criteria that failed).
+Maintain a TASK_QUEUE (ordered list), a PRIORITY_MAP (task -> current priority score, initialized from TASKS.md), a D_MAP (task -> D score, initialized from TASKS.md alongside T in MT-1), a COMPLETED_D counter (running sum of D scores for all successfully completed tasks, initialized to 0), a CASUALTY_COUNT (integer count of permission denials logged to MISSION_LOG_PATH, initialized to 0), a PLAN_FILES map (task -> list of plan filenames produced during MT-3c for that task, initialized to `[]` per task in MT-3a), and a PRIOR_ATTEMPT_MAP (task -> list of prior-attempt records, initialized to `[]` per task; each record is added on the MT-3d failure branch and contains the sub-tasks that ran and the project-test criteria that failed).
```

2. In `pathfinder-mission-team/SKILL.md`, update the MT-3e permission denial branch (lines 327-334) to append a casualty entry to `MISSION_LOG_PATH` after logging to MISSION-LOG and before re-queuing:

```diff
 If a permission denial (hook exit non-zero) occurs during any sub-step:
 - Log the denial to MISSION-LOG Permission Denials with full tool + input context.
+- Increment CASUALTY_COUNT by 1.
+- Append the following entry to MISSION_LOG_PATH under the `## Permission Denials` section:
+  ```
+  ### Casualty <CASUALTY_COUNT> — <ISO timestamp>
+  - Tool: <denied tool name>
+  - Input: <full tool input context>
+  - Task: #<N> sub-task <SUB_ID>
+  - Cause: Permission hook exited non-zero; tool call blocked.
+  ```
 - Reduce `PRIORITY_MAP[task]` by 1.
 - Re-queue task behind all higher-priority tasks.
 - Do NOT rename plan files; renaming only occurs on successful task completion.
 - Continue with next task (MT-3a).
```

## Verification

### Manual tests

1. Run `/pathfinder-mission-team` with a task that is known to trigger a permission denial (or temporarily configure the hook to deny a specific tool call).
2. After the denial fires, open the numbered MISSION-LOG file (`MISSION-LOG-N-*.md`).
3. Confirm that `## Permission Denials` contains a `### Casualty 1` entry with the correct tool name, input, task reference, and timestamp.
4. Run a second denial within the same mission and confirm `### Casualty 2` is appended (not overwriting Casualty 1).

### Automated tests

No test runner is available for SKILL.md edits (markdown-only change). A shell script could grep the numbered MISSION-LOG after a synthetic mission run for `### Casualty` entries to confirm the section is populated, but this requires a live mission execution environment.

## Changelog

### Review - 2026-03-25
- #1 (nit): Clarified Critical Files line reference to note that the `#### MT-3e` header is at line 327 but the denial branch content starts at line 329.

### Prefect Pass 1 - 2026-03-25
- #2 (minor): Updated Approach section claim from "matches" to "extends" re: Casualty entry format vs MISSION-LOG-5 convention.
- #3 (minor): Specified exact filename `MISSION-LOG-5-pathfinder-skill-overhaul.md` in Reuse section (previously ambiguous; SUCCESSFUL-MISSION-LOG-5 has no Casualty entries); added note explaining the format extension (Input vs File field, new Task field).

## Progress
- Step 1: Added CASUALTY_COUNT state variable (initialized to 0) to the MT-3 state variables list in pathfinder-mission-team/SKILL.md.
- Step 2: Updated MT-3e denial branch to increment CASUALTY_COUNT and append a Casualty entry to MISSION_LOG_PATH under ## Permission Denials after logging to MISSION-LOG.
