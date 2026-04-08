# Project Foundation

## Goals
Improve the quality, completeness, and structural integrity of pathfinder skill logging and mission artifacts. This mission addresses gaps in sub-task log enforcement, agent attribution, post-mortem entry formatting, timestamp noise, and file-validation discipline -- all identified from post-mission audits of M5. Secondary goals include tightening premission coverage checks, improving task ordering compliance, and extending the mission-complete statistics block with duration-estimation data.

## Requirements
- Timestamps in all pathfinder skills must omit the UTC offset suffix (e.g. `2026-03-25T15:30:00`, not `2026-03-25T15:30:00-0400`)
- Post-mortem entries in MISSION-LOG must be labeled with letters (A), B), C)...) rather than numbers to avoid visual ambiguity with task numbers
- Mission-team must require a minimal sub-task log entry (Status, Implementation summary, Timestamp) before marking any task Complete; missing entries must be flagged with a soft warning (Prefect records the gap and reason but allows completion) while diagnosing
- Every sub-task log entry must include an Agent field (`Agent: subagent | main`) so delegation constraints are verifiable; this field is mandatory for all sub-task log entries
- When a sub-task edits any critical shared file (SKILL.md files, hook scripts, MISSION-PERMISSIONS.json), the sub-task log must record an explicit post-edit re-read and structural validation step; the Prefect pass must treat absence of this confirmation as a blocking issue
- The Decomposer and Planner must enforce hook-update-before-file-move ordering when a sub-task moves a file referenced by a hook or self-referential script
- MT-1 must never silently drop a premission-approved task; any skip requires a detailed written justification entry in MISSION-LOG
- pathfinder-mission-team must process tasks in the priority order established during premission (read from MISSION-PERMISSIONS.json); arbitrary ordering is not permitted
- PROJECT-FOUNDATION.md must be renamed to MISSION-#-BRIEF.md (mission-numbered) in both pathfinder-premission and pathfinder-mission-team; premission must write the approved task priority order into this file
- PM-5 batch question generation must use parallel subagents when task count exceeds 4, presenting results to the user sequentially in batches of 4
- MT-1 coverage validation must be fixed so premission generic rationale entries covering shared files do not cause false skip-flags for tasks that modify those files
- MISSION-LOG ## Mission Complete section must include a computed `Min/D:` field (duration divided by total D) after `Duration:`, and a duration estimate using D x 0.43 min must be shown at premission (after the D-check) and near the start of mission-team
- MISSION-LOG ## Mission Complete section must also include `Min/C:` (duration / total C) and `Min/U:` (duration / total U, where U = (number_of_tasks x 100) - sum(C)) placed near Min/D
- Reviewer and Prefect pass counts per task and sub-task must be recorded in the MISSION-LOG entry for that task
- The add-task skill must allow Clarity Confidence scores in the full 0-99 range; the C:60 cap must be removed
- Diff-view windows must be suppressed globally when the auto-approve permission hook is active

## Explicit Non-Goals
- No changes to the scribblenot application (Rust TUI) source code or data files -- all work is confined to pathfinder skill files, hook scripts, and MISSION-LOG/TASKS.md templates
- Staged multi-premission briefings and --auto chain execution (task #34) are out of scope for this mission
- Retroactive correction of log entries or timestamps from past missions is not required
- Multi-character hint sequences, group-jump hotkeys, and other scribblenot feature work are out of scope

## Constraints
- Task #46-2 (sub-task log enforcement): implement as a soft warning while diagnosing -- Prefect flags the gap and records the reason but allows completion; upgrade to a hard block once the pattern is confirmed fixed
- Task #51 (Agent field): the field is mandatory for ALL sub-task log entries, not only tasks that carry an explicit delegation constraint
- Task #52 (re-read validation): applies to ALL critical shared files (SKILL.md files, hook scripts, MISSION-PERMISSIONS.json), not SKILL.md only
- Task #50 (shim removal): if a shim's removal log confirmation is absent, flag it in the post-mortem as a non-blocking observation rather than failing the sub-task outright
- Task #53-2: validate the D x 0.43 formula against actual M5 duration and difficulty data before baking it in; correct if the data shows a different rate
- Timestamp format change (#43) must strip the offset suffix only; the local-time base obtained via `date` in America/Toronto must remain unchanged from the #36 fix
- The MISSION-#-BRIEF rename (#42) must be applied in both pathfinder-premission and pathfinder-mission-team skill files consistently; the mission number must be embedded in the filename at runtime
- MT-1 false-positive fix (#49): the chosen approach (per-task ID citation enforcement in premission OR wildcard file coverage in MT-1) must be decided during planning; both options are acceptable but only one should be implemented to avoid conflicting logic

## Test Criteria
- All pathfinder skill timestamps produced during the mission use bare local datetime format (no offset suffix); no `-0400` or `Z` appears in any new MISSION-LOG entry
- Mission Post-Mortem entries in the completed log are labeled A), B), C)... with no numeric labels in that section
- No task is marked Complete in the Task Status table without a corresponding sub-task log entry containing Status, Implementation summary, Timestamp, and Agent fields; any gap triggers a soft warning (logged with reason, completion allowed)
- For every sub-task that edits a SKILL.md file, hook script, or MISSION-PERMISSIONS.json, the sub-task log includes an explicit re-read/validation confirmation; the Prefect pass rejects any such sub-task that lacks it
- When a file move sub-task precedes a hook update sub-task in the same task, the Decomposer/Planner re-orders them; no permission failure occurs due to a stale hook reference
- No premission-approved task is silently dropped at MT-1; any skip has a written MISSION-LOG justification
- Tasks are executed in the priority order listed in MISSION-PERMISSIONS.json; the execution sequence in the log matches the premission priority list
- The mission brief file is named MISSION-6-BRIEF.md and contains the approved task priority order; PROJECT-FOUNDATION.md does not appear as a new artifact
- With more than 4 tasks in PM-5, premission dispatches parallel subagents and presents questions in batches of 4 without idle wait between batches
- The MT-1 coverage check passes for all tasks that modify shared files when the premission uses generic rationale entries for those files; no valid task is flagged as skipped
- The ## Mission Complete section of the final log contains Min/D, Min/C, and Min/U computed values and a Reviewer/Prefect pass count per task
- The add-task skill accepts C scores above 60 without clamping; a new task submitted with C:85 records C:85
- No diff-view windows open while the auto-approve hook is active during the mission run
