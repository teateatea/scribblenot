# Closed Tasks

- [ ] **#50** Enforce hook-update-before-file-move ordering in Decomposer; require shim-removal log confirmation
  [D:65 C:52] When a mission sub-task moves a file referenced by a hook or self-referential script, the Decomposer/Planner must order the hook-update sub-task before the file-move sub-task to avoid permission failures. Any sub-task that introduces a temporary shim or compatibility artifact must also produce a log entry confirming its removal.
  Claude: B) **Process issue**: The dependency order within task #17 (consolidate pathfinder artifacts) was incorrect: the manifest file (MISSION-PERMISSIONS.json) was moved to pathfinder/ before the hook scripts that depend on it were updated to reference the new path. This caused an immediate permission denial (Casualty 1) that required a compatibility shim workaround and a manual user approval. The shim was written with a note to remove it after sub-task 17.3, but no sub-task log entry for 17.2 or 17.3 exists, leaving it unverified whether the shim was actually removed. **Suggested fix**: Add a rule to the Decomposer or Planner prompt in the mission skill that when a sub-task moves a file that is read by a hook or other self-referential script, the hook update sub-task must be ordered BEFORE the file move sub-task. Additionally, require that any sub-task that creates a temporary shim or compatibility artifact must produce a log entry confirming its removal.
  Context: Mission Post-Mortem entry B from SUCCESSFUL-MISSION-LOG-5-pathfinder-skill-overhaul.md -- directly observed during M5
- Completed: 2026-03-25T19:25:48

- [ ] **#49** Fix MT-1 false-positives from generic rationale entries causing tasks to be flagged as skipped *(implemented)*
  [D:45 C:52] The MT-1 coverage check requires each task ID to appear explicitly in an approved_actions rationale, but premission generic entries cover multiple tasks without citing them by ID, causing false skip-flags. The fix is either to enforce per-task ID citations in the premission skill or to loosen MT-1 to accept wildcard file coverage as sufficient for all tasks touching that file.
  Claude: A) **Process issue**: The MT-1 validation rule requires each task ID to appear as an explicit `#N` token in at least one `approved_actions.rationale` field, but the premission used generic entries ("core mission skill - updated by most tasks this mission") that covered many tasks without citing them by ID. This caused 9 of 17 tasks to be flagged as skipped at mission start, requiring a user intervention to re-add them before work could proceed. **Suggested fix**: Update the premission skill (pathfinder-premission/SKILL.md) to either (a) require per-task rationale citations when a task touches a shared artifact, or (b) update the MT-1 validation rule to treat a wildcard write/edit entry covering a shared file (e.g., SKILL.md) as sufficient coverage for all tasks that modify that file. This would eliminate false-skip cycles without changing the security intent of the coverage check.
  Context: Mission Post-Mortem entry A from SUCCESSFUL-MISSION-LOG-5-pathfinder-skill-overhaul.md -- directly observed during M5
- Completed: 2026-03-25T19:38:17

- [ ] **#39** Parallelize PM-5 batch question generation with subagents when task count > 4
  [D:35 C:72] In pathfinder-premission step PM-5, when there are more than 4 tasks, all question batches should be prepared simultaneously by parallel subagents upfront, then presented to the user sequentially in groups of 4 -- eliminating the idle wait between batches while the main instance processes the next group.
  Joseph: The /pathfinder-premission skill should probably use subagents to prepare the batches in PM-5, when there's more than 4 tasks. It seems like I answer a batch, then I wait for the main instance to start thinking about the next 4. I'd rather have you subagent all the tasks simultaneously first, but only present them to me in batches of 4. Ignore if you're already using subagents here.
  Context: not specified
- Completed: 2026-03-25T20:33:02

- [ ] **#45** Log reviewer and prefect pass counts per task in MISSION-LOG for planning effort correlation
  [D:30 C:55] After each task and sub-task completes, record how many Reviewer and Prefect passes were consumed in the MISSION-LOG entry; this data enables correlation of D/C scores against actual planning effort and could eventually drive a "pause and closer look" alert when pass counts exceed score-based expectations.
  Joseph: In a pathfinder-mission-team, as each task/sub-task gets recorded to the Log, please also record how many Reviews & Prefects were used. I'd like to whether D and C scores could be used to estimate the effort involved in planning a given task (which could also help flag a "pause & closer look" if more reviews / prefects are being used than expected).
  Context: not specified
- Completed: 2026-03-25T20:54:10

- [ ] **#46** Enforce sub-task log entry before marking any task complete in mission-team
  [D:25 C:55] The mission-team skill must require a minimal sub-task log entry (Status, Implementation, Timestamp) before a task can be marked Complete in the Task Status table; the drift checker or a Prefect-style check should flag missing entries as a blocking issue rather than letting them pass silently.
  Claude: Enforce sub-task log entry for every completed task — mission-team must write at least a minimal log entry (Status, Implementation, Timestamp) before marking a task Complete; missing entries should be flagged by the drift checker or Prefect rather than silently accepted
  Context: Reviewing MISSION-LOG-5 post-mission; six tasks were marked Complete with no sub-task log entries, making post-mission auditing unreliable
- Completed: 2026-03-25T21:06:29

- [ ] **#46-2** M5 post-mortem corroboration: six tasks marked Complete with no sub-task log entries
  [D:55 C:55] Six tasks were marked Complete with no corresponding sub-task log entries, making post-mission auditing unreliable. Update MT-3c/MT-3d to require a minimal log entry (Status, Implementation summary, Timestamp) before completion, with Prefect rejecting completions that lack log entries.
  Claude: C) **Process issue**: Six tasks (#16, #15, #18, #26, #28, #31) were marked Complete in the task table with no corresponding sub-task log entries. Without log entries there is no verifiable record of what was changed, which edge cases were addressed, or whether the implementation satisfies test criteria defined in PROJECT-FOUNDATION.md. This is a recurring pattern that makes post-mission auditing unreliable. **Suggested fix**: Update the MT-3c or MT-3d step in the mission skill to enforce that a sub-task log entry is written (at minimum: Status, Implementation summary, Timestamp) before a task is marked Complete. The Prefect pass should reject a task completion that has no corresponding log entry.
  Context: Mission Post-Mortem entry C from SUCCESSFUL-MISSION-LOG-5-pathfinder-skill-overhaul.md -- directly observed during M5
- Completed: 2026-03-25T21:24:24

- [ ] **#51** Add Agent attribution field to sub-task log entries to verify delegation constraints
  [D:62 C:48] Task #29 requires subagent delegation for context-heavy writes but the sub-task log has no Agent field, making silent violations undetectable. Add an optional `Agent: subagent|main` field to sub-task log entries and a Prefect verification step for any task with a delegation constraint.
  Claude: D) **Process issue**: Task #29 specifies that Task Observations and Mission Post-Mortem must be written via subagents to avoid exhausting main-instance context, but the sub-task log has no entry for #29, making it impossible to confirm whether the SKILL.md instruction correctly delegates these writes or performs them inline. The functional output (sections written) may appear correct while the constraint (subagent delegation) is silently violated. **Suggested fix**: Add a verification step to the Prefect pass for task #29 (and any task with a delegation constraint) that checks not just whether the output exists, but whether the log entry records which agent wrote it (main instance vs. spawned subagent). The sub-task log format could include an optional `Agent: subagent | main` field.
  Context: Mission Post-Mortem entry D from SUCCESSFUL-MISSION-LOG-5-pathfinder-skill-overhaul.md -- directly observed during M5
- Completed: 2026-03-25T21:35:19

- [ ] **#52** Require post-edit re-read validation in sub-task logs for any SKILL.md modification
  [D:62 C:52] When a mission sub-task edits a SKILL.md file, the sub-task log must record an explicit re-read and structural validation step confirming the edit is syntactically sound. The Prefect pass should treat absence of this confirmation as a blocking issue to prevent silent corruption of downstream sub-tasks.
  Claude: E) **Process issue**: Multiple tasks that edit pathfinder-mission-team/SKILL.md were bundled into a single mission with no integration tests between sub-tasks. Because SKILL.md is an executable skill read by the Claude runtime, a mid-mission edit that introduces a syntax or logic error in the skill could silently corrupt later sub-tasks that depend on the same skill. No sub-task log entry for any of these tasks records a validation step (e.g., re-reading the modified SKILL.md to confirm structural integrity). **Suggested fix**: For any sub-task that edits a SKILL.md file, require the sub-task log to include a validation step confirming the file was re-read after editing and the modified section is syntactically consistent with surrounding content. The Prefect pass should treat absence of this confirmation as a blocking issue.
  Context: Mission Post-Mortem entry E from SUCCESSFUL-MISSION-LOG-5-pathfinder-skill-overhaul.md -- directly observed during M5
- Completed: 2026-03-25T21:54:49

- [ ] **#53** Add Min/D stat to mission log and duration estimates to premission and mission-team
  [D:25 C:58] Append a computed `Min/D:` field after `Duration:` in ## Mission Complete. In /pathfinder-premission (after the 140/200 D check) and /pathfinder-mission-team (near start), display an estimated duration using total D * 0.43 min as the baseline rate.
  Joseph-Raw: In a pathfinder mission log, in the ## Mission Complete section after Duration:, please add minutes per difficulty. I'd like to be able to eventually start estimating the future mission durations based on difficulty. Also, add a mission duration estimate to both /pathfinder-premission (after the 140/200 D check) and /pathfinder-mission-team (near the very start). Current data suggests 2.3min/D, so use that for now.
  Context: not specified
- Completed: 2026-03-25T23:50:39

