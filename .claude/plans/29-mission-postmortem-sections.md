## Task

#29 — Add Task Observations and Mission Post-Mortem sections to MISSION-LOG at wrap-up

## Context

After each mission the main instance currently jumps straight from sub-task completion to the ## Mission Complete block. Two observations prompted this task:

1. There is no structured place to capture intent-vs-implementation gaps per task (useful for issuing follow-up work).
2. There is no structured place to capture process inefficiencies observed during the mission (useful for submitting as new /add-task entries).

Both sections must be written by subagents (not inline on the main instance) to avoid exhausting main-context after a long mission, per #29-2.

## Approach

Insert two sequential subagent spawns in MT-4, between step 4 (append Mission Complete markdown) and step 5 (restore diff-view windows). Each subagent reads the mission log plus relevant project files, writes one section directly to MISSION_LOG_PATH, and returns a one-line status string. The sections are appended after ## Mission Complete.

The two sections are:
- **## Task Observations** — intent-vs-implementation gaps with suggested next steps. The subagent omits the section entirely if it finds nothing substantive to say.
- **## Mission Post-Mortem** — process inefficiencies, each formatted as a ready-to-submit /add-task entry.

## Critical Files

- `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` — lines 357-384 (MT-4 section); insertion point is after the closing fence of the step 4 code block (line 375) and before step 5 (line 377)

## Reuse

- Existing MT-4 subagent spawn pattern (same shape as Drift Checker, Test Checker, etc.)
- MISSION_LOG_PATH variable already in scope at MT-4

## Steps

1. Open `C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md` for editing.

2. Insert the two new subagent spawns between step 4 and step 5 of MT-4. The diff below shows the insertion (context lines are the closing fence of the step 4 block and the existing step 5 line):

```diff
-5. Restore diff-view windows. Read `<PROJECT_ROOT>/.claude/settings.local.json`, remove the `autoAcceptEdits` key if present, and write it back.
+4a. Spawn a "Task Observations Writer" Sonnet subagent:
+
+> You are the Task Observations Writer. Your job is to review what was planned vs what was implemented for each task in this mission, and append a ## Task Observations section to the mission log if any clear intent-vs-implementation gaps exist.
+>
+> Read:
+> - `<MISSION_LOG_PATH>` (full mission log, including Sub-task Log entries and any Abandonment Records)
+> - `<PROJECT_ROOT>/.claude/TASKS.md` (original task descriptions and acceptance criteria)
+>
+> For each completed task, compare the original task description and intent against the sub-task log entries. Identify gaps where: (a) the implementation deviated from the stated intent in a meaningful way, (b) edge cases or acceptance criteria were left unaddressed, or (c) a follow-up action is clearly implied.
+>
+> If you find at least one substantive gap, append the following section to `<MISSION_LOG_PATH>`:
+>
+> ```markdown
+> ## Task Observations
+>
+> ### #N <Task Name>
+> - **Gap**: <one-sentence description of the intent-vs-implementation gap>
+> - **Suggested next step**: <one-sentence action, phrased as a concrete task>
+> ```
+>
+> Include one subsection per task that has a substantive gap. If a task has no gaps, omit it entirely. If no tasks have any substantive gaps, do NOT append the section at all.
+>
+> Do NOT use AskUserQuestion. Return exactly one of:
+> - `TASK OBSERVATIONS WRITTEN: <N> gap(s) noted`
+> - `TASK OBSERVATIONS SKIPPED: no substantive gaps found`
+
+4b. Spawn a "Mission Post-Mortem Writer" Sonnet subagent:
+
+> You are the Mission Post-Mortem Writer. Your job is to identify process inefficiencies from this mission and write them up as ready-to-submit /add-task entries.
+>
+> Read:
+> - `<MISSION_LOG_PATH>` (full mission log — Sub-task Log, Permission Denials, Abandonment Records, and any Prefect Issues notes)
+>
+> Identify process inefficiencies: repeated failures, permission denials, subagent confusion, plan-review loops that took more passes than expected, abandonment cycles, or any pattern that a future task could address by changing the mission skill or project config.
+>
+> Append the following section to `<MISSION_LOG_PATH>` (always append this section, even if the list has only one entry):
+>
+> ```markdown
+> ## Mission Post-Mortem
+>
+> Process inefficiencies observed during this mission. Each entry is formatted as a ready-to-submit /add-task entry.
+>
+> - **[Slug]**: <one-sentence description of the inefficiency and its impact>
+>   Suggested task: "<imperative-phrased task title>" — <one to two sentences of context explaining what change would fix it and why it matters>
+> ```
+>
+> Use a short bracketed slug (2-4 words, kebab-case) that names the pattern, e.g. [repeated-plan-failures], [permission-denial-loop], [abandonment-thrash].
+>
+> If no inefficiencies are identifiable (clean mission, zero denials, zero abandonments, no retry loops), append the section with a single entry: `- **[clean-mission]**: No process inefficiencies identified.`
+>
+> Do NOT use AskUserQuestion. Return exactly: `POST-MORTEM WRITTEN: <N> inefficiency/inefficiencies noted`
+
+5. Restore diff-view windows. Read `<PROJECT_ROOT>/.claude/settings.local.json`, remove the `autoAcceptEdits` key if present, and write it back.
```

## Verification

### Manual tests

1. Run `/pathfinder-mission-team` on a test task or a real task that has sub-task log entries.
2. After the mission completes, open the mission log and verify:
   - If gaps exist: `## Task Observations` appears after `## Mission Complete` with at least one `### #N` subsection.
   - If no gaps: `## Task Observations` is absent from the log entirely.
   - `## Mission Post-Mortem` always appears after `## Mission Complete` (or after `## Task Observations` if that section was written).
   - Each post-mortem entry has a bracketed slug, a description line, and a "Suggested task:" line.
3. Verify the Output line still fires: `Mission complete. See <MISSION_LOG_PATH> for full history.`

### Automated tests

No automated test runner exists for the SKILL.md file itself. A realistic option: write a shell integration test that runs a minimal mock mission (single no-op task) and asserts that the mission log contains the `## Mission Post-Mortem` header.

### Doc checks

`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Task Observations Writer`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | Mission Post-Mortem Writer`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | TASK OBSERVATIONS WRITTEN`
`C:/Users/solar/.claude/skills/pathfinder-mission-team/SKILL.md | contains | POST-MORTEM WRITTEN`

## Progress

- Step 1: Opened SKILL.md for editing (read lines 357-384 of MT-4 section)
- Step 2: Inserted 4a Task Observations Writer and 4b Mission Post-Mortem Writer subagent spawns between step 4 code block closing fence and step 5 in MT-4

## Changelog

### Review - 2026-03-25
- #1: Corrected Critical Files line range from 357-379 to 357-384 and added precise insertion point (after line 375, before line 377)

### Prefect Pass 1 - 2026-03-25
- #1: Fixed fallback entry in Post-Mortem Writer prompt to use slug format (`- **[clean-mission]**: No process inefficiencies identified.`) consistent with the established bullet format, preventing malformed output in the produced SKILL.md

## Implementation
Complete – 2026-03-25
