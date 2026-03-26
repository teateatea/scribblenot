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

- [ ] **#40** Require detailed MISSION-LOG justification whenever mission-team skips a premission-approved task
  [D:25 C:58] pathfinder-mission-team must never silently drop a task from the user's starting command; any skip requires a written justification entry in MISSION-LOG, and the team's default stance should be that premission-approved tasks are mandatory -- skipping is a last resort that demands explicit reasoning on record.
  Joseph: When /pathfinder-mission-team skips any task listed in the user's starting command while building its initial task, it MUST justify this in the MISSION-LOG, in detail. As a general rule, the /pathfinder-mission-team should NEVER be skipping tasks, especially if they've gone through /pathfinder-premission! Their whole deal is to always get the job done.
  Context: not specified
- Completed: 2026-03-25T23:59:51

- [ ] **#48** Suppress diff windows globally when the auto-approve permission hook is active
  [D:25 C:45] The permission auto-approve hook approves file changes before the user can interact with a diff, leaving empty/stale diff windows behind. Locate where diff-view behavior is configured (settings.json, CLAUDE.md, or IDE settings) and disable diffs for the duration the hook is active.
  Joseph-Raw: I think we should not use diffs while we have that hook running lol, they'll nearly always misfire if you're going to change it anyways. /add-task ? That might touch on CLAUDE.md, I'm not sure where your instructions about using diffs live.
  Context: not specified
- Completed: 2026-03-26T00:18:14

- [ ] **#54** Add Min/C and Min/U stats to Mission Complete section alongside Min/D
  [D:20 C:68] Extend the ## Mission Complete section in MISSION-LOG to include two new computed fields placed near Min/D: `Min/C` (mission duration divided by total C score) and `Min/U` (mission duration divided by U, where U = (number_of_tasks x 100) - sum(C)), enabling future correlation of clarity and uncertainty against actual mission duration. Example: 3 tasks with C:10,20,30 gives U = 300-60 = 240.
  Joseph-Raw: In pathfinder missions, it seems that D scores may be useful in predicting mission duration estimates. I'd like to use the C scores, but I suspect we should also track U (100-C). The Uncertainty score is the amount of "Missing" C. Please add the min/U (does uncertainty have an effect on duration) and min/C (does certainty have an effect on duration) in the same Mission Complete section near the min/D result too.
  Context: U formula clarified during M6 premission: U = (number_of_tasks x 100) - sum(C), not a simple 100-C per task.
- Completed: 2026-03-26T00:26:44

- [ ] **#47** Use letters (A, B, C...) for post-mortem entries in mission log
  [D:15 C:80] Mission Post-Mortem section entries in pathfinder mission logs should be labeled A), B), C)... rather than numbered, eliminating visual ambiguity with task numbers (#N format) when reviewing or submitting post-mortem items as add-task entries.
  Joseph-Raw: In pathfinder missions, post-mortem entries should be lettered instead of numbered (A) Process issue:, B) Process issue:, etc.) to avoid conflict with task numbers
  Context: not specified
- Completed: 2026-03-26T00:32:10

- [ ] **#19** Remove the C:60 cap from add-task initial scoring; allow full 0-99 range *(implemented)*
  [D:10 C:58] The add-task skill clamps Clarity Confidence at 60, deferring higher scores to later review passes. The cap should be removed so the initial score can reflect the full 0-99 range accurately. Review passes then serve their intended purpose -- correcting toward accuracy -- rather than mechanically bumping capped values upward.
  Joseph: I believe it's the add-task skill that is only allowed to score a new task up to C: 60, not greater. Originally, I thought we'd wait for a later review to bump it higher. Let's not do that, the initial score should be allowed to be the full range 0-99. This let's the initial scoring not be artificially clamped down. We can still do later reviews, but instead of those reviews pushing tasks higher, they'll now be pushing the clarity towards more *accurate*, which is the intended goal anyways.
  Context: not specified
- Completed: 2026-03-26T00:39:27

- [ ] **#42** Rename PROJECT-FOUNDATION to MISSION-#-BRIEF and add task priority order to it in both pathfinder skills
  [D:35 C:55] PROJECT-FOUNDATION.md should be renamed to MISSION-#-BRIEF.md (mission-numbered) in both pathfinder-premission and pathfinder-mission-team, and premission should write the approved task priority order into this file so mission-team can reference it during execution.
  Joseph: /pathfinder-premission should probably include its task priority order in the PROJECT-FOUNDATION, which we should now be naming the MISSION-#-BRIEF (in both pathfinder skills).
  Context: not specified
- Completed: 2026-03-26T00:52:53

- [ ] **#41** Confirm and fix mission-team task execution order to respect premission priority ranking *(sub-task 2 implemented)*
  [D:25 C:45] pathfinder-mission-team may be processing tasks in an arbitrary order rather than following the priority sequence established during premission; investigate whether the priority list from MISSION-PERMISSIONS.json is read and honoured at MT-1 initialization, then fix or clarify.
  Joseph: I think /pathfinder-mission-task might not be respecting the task priority order set out by the /pathfinder-premission. Confirm and fix, or clarify.
  Context: not specified
  - [ ] **#41-2** M6 live evidence: tasks #39, #49, #50 completed first while higher-listed tasks #19, #43, #47 remain queued
    [D:25 C:55] MISSION-LOG-6 at ~1h45m shows #39, #49, #50 at Attempts:1/Complete while tasks listed earlier in the premission order (#19, #43, #47 etc.) are still Queued. Confirms that execution order does not follow the premission priority list; provides concrete log data for diagnosing the root cause in the mission-team skill.
    Joseph-Raw: Following up on #41, /pathfinder-mission-task following unexpected task order. Here's the top of MISSION-LOG-6-skill-log-quality.md, at about 1h 45m into the mission: [log excerpt showing #39/49/50 completed while #19/43/47 still queued]
    Context: not specified
- Completed: 2026-03-26T01:46:08

- [ ] **#43** Drop UTC offset from pathfinder timestamps, output bare local datetime
  [D:10 C:60] Change pathfinder skill timestamp format from `2026-03-25T15:30:00-0400` to `2026-03-25T15:30:00` by stripping the offset suffix -- single user, single machine, offset is noise.
  Joseph: On pathfinder skills, the current date-time format is 2026-03-25T15:30:00-0400. Let's omit the UTC offset (the -0400), I'm a single user working on a single machine and don't expect timezones to be important ever. (Possibly for that ONE hour per year where daylight savings skips backwards, but that's an acceptable risk for reducing noise for the rest of the year.)
  Context: not specified
- Completed: 2026-03-26T01:53:51

- [ ] **#3** Fix hook logging all tool calls as PERMISSION DENIED in mission log
  [D:30 C:60] The MISSION-LOG-active.md hook appends every tool call as a permission denial entry regardless of outcome, polluting mission logs with noise. Needs hook filter on actual denial exit codes or watch-pattern rename.
  Joseph: Hook collision: MISSION-LOG-active.md hook appends every tool call as a PERMISSION DENIED entry regardless of outcome - pollutes mission logs with noise. Fix: rename active log away from hook watch pattern, or filter hook on actual denial exit codes.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization
- Completed: 2026-03-25

- [ ] **#4** Premission should enumerate all commands the mission loop may run
  [D:20 C:60] MISSION-PERMISSIONS.json only includes `cargo build` and `cargo clippy` but the mission TDD phase also needs `cargo test`. Premission must require explicit approval for every command the loop may invoke.
  Joseph: MISSION-PERMISSIONS.json goes stale after premission: only `cargo build` and `cargo clippy` were approved for dead-code-cleanup mission, but the mission loop needs `cargo test` too. Fix: premission should enumerate all commands the mission loop may run (including cargo test) and require explicit approval per command.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization
- Completed: 2026-03-25

- [ ] **#5** Add TDD-feasibility check to Decomposer for event-loop and TUI code
  [D:40 C:60] The mission loop assumes failing unit tests can always be written before implementation, but crossterm key-event handling has no test harness. The Decomposer should detect sub-tasks where compile-time failing tests are infeasible and set test_runner to none.
  Joseph: TDD feasibility gap for TUI/event-loop apps: pathfinder assumes failing unit tests can always be written before implementation, but crossterm key-event handling is tightly coupled to mutable TUI state and has no test harness. Fix: add a TDD-feasibility check step to the Decomposer that sets test_runner to none for sub-tasks where compile-time tests are impossible to write.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization
- Completed: 2026-03-25

- [ ] **#6** Cap or coarsen Decomposer sub-task count to reduce subagent overhead
  [D:25 C:60] Decomposer can produce 7+ sub-tasks per task, each triggering up to 6 subagent spawns, totalling 40+ agents for one feature. Add a 5 sub-task cap or coarseness heuristic to group related incremental steps.
  Joseph: Decomposer sub-task count bloat: for tightly coupled incremental features the decomposer can produce 7+ sub-tasks, each triggering 6 subagent spawns. Fix: add a cap (e.g. 5 sub-tasks max) or a coarseness heuristic to the Decomposer prompt so related implementation steps are grouped.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization
- Completed: 2026-03-25

- [ ] **#7** Mission MT-1 must validate task list against premission scope before starting
  [D:35 C:60] Tasks added after premission was run will have no approved permissions or test criteria. Mission initialization must cross-check each task against MISSION-PERMISSIONS.json and skip (with log entry) any task not explicitly covered.
  Joseph: Mission must validate premission scope before starting: if a task was added after premission was run it will not have approved permissions, tool allowlists, or test criteria. Fix: at MT-1 initialization, cross-check each task in TASK_LIST against MISSION-PERMISSIONS.json approved_actions and TASKS.md entries; skip any task not explicitly covered in premission and log the skip.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization
- Completed: 2026-03-25

- [ ] **#8** Premission should prompt clarifying questions when a task's D score exceeds its C score
  [D:25 C:55] During premission setup, any task where difficulty exceeds clarity confidence should trigger targeted user questions before going dark; low-D tasks can be fast-pathed automatically while high-D/low-C tasks warrant deeper discussion to avoid mid-mission surprises.
  Joseph: In pathfinder PREMISSION, any task with D greater than C should be discussed to clarify. Likely the minor fixes can be passed over quickly, anything that's Difficult but not Clear should prompt additional questioning while the user is available.
  Context: First pathfinder mission run on scribblenot, flagged during graceful mission exit
- Completed: 2026-03-25

- [ ] **#9** Replace `Original:` label in add-task entries with `Joseph:` or `Claude:` to indicate who submitted the task
  [D:10 C:58] The add-task skill's entry format uses a generic `Original:` label, but tasks can be submitted by the user (Joseph) or autonomously by Claude. Replacing the label with the actual source prevents ambiguity when reviewing task history.
  Joseph: The add-task skill can be prompted by either Joseph OR Claude. Change "Original" to either "Joseph:" or "Claude:" as appropriate to prevent ambiguity about the original source.
  Context: First pathfinder mission run on scribblenot, flagged during graceful mission exit
- Completed: 2026-03-25

- [ ] **#10** ~~Commander should execute simple skill/config edits directly without spawning subagents~~ [ABANDONED]
  [D:35 C:55] Adds a routing heuristic to pathfinder-mission-team where the Commander skips the Planner/Reviewer/Implementer subagent loop for low-complexity tasks (skill/config file edits, no compilation or deep exploration needed), executing changes directly in the main conversation to preserve context budget, avoid permission walls, and prevent Rule #31 casualties; reserves the full subagent loop for high-D, destructive, or exploration-heavy tasks.
  Claude: When pathfinder runs tasks that only edit skill/config files (no code compilation or complex exploration needed), the Commander should do the work directly in the main conversation rather than spawning Planner/Reviewer/Implementer subagents. Subagents cost context, hit permission walls, and can't create .md files (Rule #31). Direct execution from the main conversation is faster, cheaper, and avoids cascading casualties. The plan-review loop should be reserved for tasks that genuinely benefit from multi-agent review (complex code changes, destructive operations, high D-score tasks).
  Context: not specified
- Completed: 2026-03-25

- [ ] **#11** Mission team should rename completed task plans to COMPLETED-*.md on success
  [D:30 C:55] After pathfinder-mission-team successfully completes a task, it should rename the corresponding plan file with a COMPLETED- prefix so that /lets-start and future agents can identify and skip stale plans without cross-referencing TASKS.md.
  Joseph: I think /pathfinder-mission-team leaves plans unmarked when complete. Pretty sure after completing each task successfully, it should rename that plan COMPLETED-*.md, so that later agents know to ignore them.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#12** Track cumulative mission difficulty in MISSION-LOG header, updated after each task
  [D:35 C:52] Add a Difficulty field under the ## MISSION heading in MISSION-LOG that shows completed vs total difficulty (e.g. "Difficulty: X/T"). The pathfinder-mission-team updates this after each task completes, enabling post-failure analysis of a suspected ~200-point difficulty ceiling.
  Joseph: MISSION-LOG should note Difficulty (meaning the total difficulty of all the tasks in this mission), probably under the ## MISSION heading. /pathfinder-mission-team should update that number after each task ("Difficulty: X/T", where X is the total difficulty of tasks completed, and T is the total difficulty of all tasks in this mission), so that if the mission fails, we'll learn about a difficulty ceiling. I suspect we can't go about 200, based on mission 3 (D: 115, main instance context got to 65%)
  Context: not specified
- Completed: 2026-03-25

- [ ] **#13** Warn user in premission when total mission difficulty exceeds context-load thresholds (140/200)
  [D:35 C:52] The premission step should sum D scores across all tasks and emit a warning if the total exceeds 140, indicating the main instance may run out of context. If the total exceeds 200, require a second explicit confirmation since that load level is untested.
  Joseph: /pathfinder-premission should note when a mission's total difficulty exceeds 140, and warn the user that the main instance's context may be overtaxed by the tasks included. If the total difficulty exceeds 200, confirm a second time to make absolutely certain; it's unknown whether this difficulty is possible.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#14** Record Start-Time and End-Time in MISSION-LOG for duration tracking
  [D:35 C:45] The mission team should write timestamps when a mission begins and ends into the MISSION-LOG file. This data will eventually enable premission to estimate completion time based on task difficulty.
  Joseph: /pathfinder-mission-team should write Start-Time and End-Time into MISSION-LOG, when it starts and ends. We can start to gather data about difficulty vs duration, so that in the future, premission can give a completion time estimate.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#15** Suppress diff-view windows during pathfinder-mission-team execution
  [D:30 C:50] File edits made by mission-team subagents trigger diff view windows that pop over the user's active window during autonomous operation, creating a risk of accidental input if the user is typing elsewhere. Diff windows should be disabled or suppressed for the duration of a mission-team run.
  Joseph: The /pathfinder-mission-team opens diffs when it creates or edits files ( I think). These open in a new window but because the permission gets handled (correctly) by the mission team, no user input is required. However, this is not intended behaviour, and introduces the possiblity of accidental input. Sometimes I'm typng somewhere else, and the window pops open for and it'd be pretty easy for me to cause problems if the timing was unlucky. When I'm working closely with Claude, I do like those diff windows, but they should not be used in pathfinder-mission-team.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#16** Preserve Prefect-1 review report when Prefect-2 begins
  [D:20 C:55] During pathfinder-mission-team, the plan file written by Prefect 1 is deleted before Prefect 2 starts, erasing the audit trail of where review changes originated. The Prefect-1 report should be retained so post-mission review can trace issues to specific review stages.
  Joseph: I'm not sure if it's from plan-review-team, or from pathfinder-mission-team, but during a /pathfinder-mission-team, the report that Prefect 1 writes gets removed before P2 begins. This is unnecessary, and actually introduces some ambiguity about where a change came from. Please do not remove that report, I'd like to be able to see where things happened, so I know what to look for if something goes wrong.
  Context: not specified
  - [ ] **#16-2** Eliminate mechanical Prefect Report cleanup step between Pass 1 and Pass 2
    [D:25 C:55] After Pass 1 returns PREFECT FIXED, the Commander reads and edits out the Prefect Report section before spawning Pass 2 -- pure overhead. Options: (a) Pass 1 outputs findings as return text only; (b) Pass 2 ignores/overwrites any existing Prefect Report section; (c) Pass 1 writes findings to a temp location instead of the plan file.
    Claude: Eliminate manual Prefect Report removal step between Pass 1 and Pass 2: after Prefect Pass 1 returns PREFECT FIXED, the Commander must read the plan file, find the Prefect Report section boundary, edit it out, then spawn Pass 2. This is pure mechanical overhead. Options: (a) instruct Prefect Pass 1 to output its findings only as return text, not write them to the plan file; (b) instruct Prefect Pass 2 to ignore and overwrite any existing Prefect Report section without requiring Commander intervention; or (c) have Prefect Pass 1 write findings to a separate temp location rather than injecting into the plan.
    Context: Observed during mission 4 (tdd-warn-tracking). Every PREFECT FIXED result required a Commander read-edit-spawn cycle before Pass 2 could run, adding overhead to every plan that needed Prefect fixes.
- Completed: 2026-03-25

- [ ] **#17** Organize pathfinder mission artifacts into a dedicated [project]/pathfinder/ directory
  [D:50 C:52] PROJECT-FOUNDATION.md, MISSION-PERMISSIONS.json, and MISSION-LOG files are currently scattered in the project root; they should live under [project]/pathfinder/ to keep mission-specific artifacts separate. PROJECT_LOG.md and project tests remain in .claude/ since they serve broader, non-mission purposes.
  Joseph: The files related to pathfinder-premission or mission-team should probably go into a [project]/pathfinder folder. That includes PROJECT-FOUNDATION, MISSION-PERMISSIONS, MISSION-LOGS. I *think* PROJECT_LOG AND PROJECT_TESTS can remain in project/.claude, because those could be used outside of pathfinder missions, but please advise.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#18** Audit and eliminate blank-line churn in mission-team review loop
  [D:25 C:42] Diffs during pathfinder-mission-team show whitespace-only changes (e.g. two blank lines collapsed to one), suggesting one phase writes extra blank lines that a subsequent phase removes. Need to identify which phase introduces the unnecessary lines and stop adding them at the source.
  Joseph: In /pathfinder-mission-team, I notice some of the diffs involve only tidying up whitespace, like changing two blank lines into one. Please check the review loop, are we writing in blank lines, then removing them? If so, let's just stop adding the unnecessary lines to begin with.
  Context: not specified
  - [ ] **#18-2** Audit Planner prompt template for blank-line generation patterns
    [D:20 C:50] The Planner prompt or its examples may be the source of extra blank lines that reviewers then flag and remove as whitespace-only diffs. Targeting the Planner specifically to eliminate the root cause rather than just the symptom.
    Claude: Blank line churn in plan review loop: planners are adding extra blank lines that reviewers or prefects then flag and remove, producing whitespace-only diffs with no semantic value. Audit the Planner prompt template to remove any instructions or examples that generate double blank lines between sections.
    Context: Observed during mission 4 (tdd-warn-tracking) and confirmed by Joseph in new task added post-mission. Review loop is spending subagent cycles on blank line normalization.
- Completed: 2026-03-25

- [ ] **#20** Fix mission-team timestamps to use actual time, not hardcoded midnight
  [D:10 C:72] MISSION-LOG entries show timestamps like `2026-03-24T00:00:00Z` -- the date is correct but time is always midnight, indicating the skill constructs timestamps from the date alone without calling `date` to get the real time. Fix: run `date` and format the result as a proper ISO 8601 timestamp.
  Joseph: I'm pretty sure /pathfinder-mission-team doesn't run bash(date), I'm seeing multiple "Timestamp: 2026-03-24T00:00:00Z" entries. Please correctly add the time, not just the date!
  Context: not specified
- Completed: 2026-03-25

- [ ] **#24** Rename mission log to SUCCESSFUL-*.md when all tasks complete
  [D:10 C:75] On full mission completion, pathfinder-mission-team should rename the active MISSION-LOG file with a SUCCESSFUL- prefix so completed missions are immediately distinguishable from in-progress, failed, or abandoned ones without opening the file.
  Joseph: When /pathfinder-mission-team is able to complete all tasks, they should rename their mission log SUCCESSFUL-*.md when they're done.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#25** Add "Context at finish:" field to Mission Complete section in MISSION-LOG template
  [D:10 C:72] Append a "Context at finish:" line to the ## Mission Complete section so Joseph can optionally record main-instance context usage after each mission. Enables correlation of context % vs mission difficulty over time to determine whether context is actually a limiting factor.
  Joseph: For /pathfinder-mission-team, at the end of the ## Mission Complete section, add a new line "Context at finish:" for Joseph to optionally record. Considering 2 missions have now completed with context usage around 65-70% (despite mission difficulties of 125 and 175), context might not actually matter as much as I thought, but I'd like to record the data to confirm it.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#26** Provide prior-attempt context to Decomposer on task re-queues
  [D:30 C:55] When a task is re-queued after failed project tests, the Decomposer prompt should include the prior attempt's sub-tasks and failed test criteria so it generates targeted gap-filling sub-tasks instead of re-discovering the full task scope from scratch.
  Claude: Decomposer context blindness on re-runs: when a task is re-queued after a failed project-test run, the Decomposer has no context about what was already attempted and what gaps remain. For task #5, the first-pass Decomposer generated sub-tasks to verify existing content (already implemented), wasting ~6 subagent spawns before the re-queue cycle generated the correct targeted fix. The Decomposer prompt (or Commander pre-prompt) should include prior attempt context -- what sub-tasks were run, what project-test criteria failed -- so it generates gap-targeted sub-tasks instead of re-discovering the whole task from scratch.
  Context: Observed during mission 4 (tdd-warn-tracking). Task #5 required two full decompose/plan/implement cycles because attempt 1 verified already-present content and missed rationale field + named step criteria.
- Completed: 2026-03-25

- [ ] **#28** Add mv fallback when git mv fails for gitignored plan files in COMPLETED- rename step
  [D:20 C:60] Plan files in .claude/plans/ are gitignored in project repos, so git mv fails with "not under version control" for most renames. MT-3d should catch this error and fall back to regular mv, then git add the COMPLETED- file if the destination directory is tracked.
  Claude: git mv fallback for gitignored plan files: plan files live in .claude/plans/ which is gitignored in the scribblenot repo, so git mv fails silently for most of them during the COMPLETED- rename step. The MT-3d rename logic should try git mv first, catch the 'not under version control' error, and fall back to regular mv + git add for the new COMPLETED- file.
  Context: Observed during mission 4 finale. 9 of 12 plan renames required manual mv fallback because .claude/ is gitignored. The git mv partial-failure caused a commit that only captured 3 of 12 renames.
- Completed: 2026-03-25

- [ ] **#29** Add Task Observations and Mission Post-Mortem sections to MISSION-LOG at wrap-up
  [D:25 C:68] Insert two sections just before ## Mission Complete: "Task Observations" (clear gaps between intent and implementation with suggested next steps -- omit if nothing obvious) and "Mission Post-Mortem" (process inefficiencies noted during this mission, written with enough detail to be submitted directly as /add-task entries). Both written by the mission team during wrap-up.
  Joseph: The /pathfinder-mission team should add two sections just before ## Mission Complete: 1) Task Observations: Please note any obvious next steps related to the completed tasks in this mission, and explain why you think they'd be an improvement. This section can be empty, only record clear gaps in intent vs implementation. 2) Mission Post-Mortem: Please note any inefficiencies that you'd note in the /pathfinder-mission-team process from this mission in enough detail that it could be successfully used as an /add-task.
  Context: not specified
  - [ ] **#29-2** Use subagents for wrap-up sections to avoid maxing main instance context
    [D:15 C:60] The Task Observations and Mission Post-Mortem sections should be written via subagents rather than inline on the main instance; running this step in the main context immediately after mission completion previously forced a /compact.
    Joseph: additional instructions for 29: This should continue to use subagents in sequence. Calling this immediately after Mission 3 completed maxxed out the main instance's context forcing a /compact, which should be avoided.
    Context: not specified
- Completed: 2026-03-25

- [ ] **#30** Prefix pathfinder plan filenames with mission number (e.g. M5-20-1-slug.md)
  [D:20 C:58] Pathfinder-mission-team should prepend the current mission number to each plan filename it creates, so plans from different missions are immediately distinguishable -- especially useful if a mission is interrupted and plans from multiple missions coexist in .claude/plans/.
  Joseph: Pathfinder plans need to include the mission number too. For example, M5-20-1-three-word-name.md. If a mission is ever interupted for any reason, it would help to be able to distinguish which mission a plan came from.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#31** Move completed task entries from TASKS.md to CLOSED-TASKS.md with completion timestamp
  [D:25 C:58] When pathfinder-mission-team successfully completes a task, it should remove that task's entry from TASKS.md and append it to CLOSED-TASKS.md (creating the file if absent) with the completion date and time, keeping TASKS.md focused only on actionable work.
  Joseph: When /pathfinder-mission-team completes a task, let's actually move it to CLOSED-TASKS.md, just appended at the end of the file (and add the date and time please). No need to have TASKS.md cluttered up with tasks that no longer require action.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#32** Add PreCompact hook to log compact events with timestamp during pathfinder missions
  [D:30 C:55] Install a PreCompact hook that fires just before Claude's automatic /compact, appending a timestamped entry to the active MISSION-LOG so post-mission review can identify exactly when compaction occurred and whether it had any negative effect on mission continuity.
  Joseph: For /pathfinder-mission-team, create a PreCompact hook. Claude agents can't actually see their context usage, so they don't know how close they are to an automatic /compact. IN THEORY, the pathfinder mission team relies fairly heavily on .md to track the progress, so it should be resilient against /compact information dilution. But just before a compact happens, we might as well immediately log it so we know where it happened in the process, what exact time, etc. Then Claude can do a review later to see if there was any negative effect.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#33** Add second clarification threshold to premission (D > 50, C < 70)
  [D:20 C:72] Extend /pathfinder-premission's clarification question trigger with a second condition: any task scoring D > 50 with C < 70 should prompt the user for more detail before going dark, ensuring medium-to-high difficulty tasks have a sufficient explanation on record.
  Joseph: In /pathfinder-premission, add a second clarification questions threshold: D > 50 with C < 70. Likely, we'll tweak these numbers, but for any reasonably complex task, I'd like to set a fairly high requirement for having a detailed explanation.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#35** Enforce full Prefect approval loop; no corner-cutting on nits
  [D:25 C:68] Investigate whether pathfinder-mission-team skips or short-circuits the Prefect review cycle when only minor issues remain, and if so, enforce that implementation never begins until Prefect gives unqualified approval -- the team must always run another reviewer pass rather than waiving remaining issues, consistent with the "slowly and perfectly" mission goal.
  Joseph: In /pathfinder-mission-team, it looks like sometimes, plans are being implemented without the full Prefect approval, would you look into it? Is this just the team cutting corners when all that remains is minor or nits? The goal for this team is more "slowly and perfectly, regardless of effort", so if it is, I'd rather you go back to another round of reviewers instead.
  Context: not specified
  - [ ] **#35-2** Log Prefect-skip findings in Task Observations if root cause is unexpected
    [D:10 C:52] If the investigation in #35 reveals the Prefect approval is being skipped for a reason other than corner-cutting on nits, record the findings and the unexpected behavior in the Task Observations section of the MISSION-LOG so it can be reviewed and addressed.
    Joseph: as an addition to 35, please include the results of your findings in the Task Observations at the end of the mission if there's something else going on here.
    Context: not specified
- Completed: 2026-03-25

- [ ] **#36** Use local Toronto time instead of UTC in pathfinder timestamps
  [D:10 C:78] Pathfinder MISSION-LOG timestamps are correctly formatted as ISO 8601 but use the UTC/Zulu offset (Z suffix); since a single machine runs this skill, switch to local Toronto time (America/Toronto, ET) for all timestamps so they match the user's clock without requiring mental UTC conversion.
  Joseph: The pathfinder skills appear to use ISO 8601 (correctly), but the time is set to Zulu time (T07:30:29Z). Given that there's only one computer running this skill on one project at a time, let's just use local time (Toronto) for easier user comprehension.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#37** Fix priority direction and replace linear decay with X² cumulative reduction
  [D:40 C:62] Confirm whether tasks start at the wrong priority floor (0 instead of 99), then overhaul the decay algorithm: reduce priority by X² on each consecutive failed attempt (1, 4, 9, 16, 25...) where X resets to 1 after a successful intervening task completes; dependent tasks must receive the same reduction in lockstep; minimum priority is 0.
  Joseph: In /pathfinder-mission-team, I suspect the priority works backwards, please confirm. I think that tasks get abandoned at 0, meaning all tasks should probably start at 99. And actually, the priority deprecation should probably be a cumulative score: Priority is reduced by X^2, where X is the number of consecutive attempts count. So after the first attempt, reduce priority by 1, then 4, 9, 16, 25 etc. But if another task is successfully completed in between (because the first task got bumped below it), that X is reset back to one. Make sure that tasks with dependencies all receive the reduction to keep them in sync. Minimum priority score is 0.
  Context: not specified
- Completed: 2026-03-25

- [ ] **#38** Mirror casualty entries to numbered MISSION-LOG permission denials section
  [D:20 C:70] Casualty (permission denial) events are written to MISSION-LOG-active.md but not copied to the permanent numbered MISSION-LOG-#-*.md file under its Permission Denials heading; both files should receive the entry so the archived mission record is complete.
  Joseph: In pathfinder-mission-team, I think casualties get reported to MISSION-active, but not into the MISSION-LOG-#-*.md under the permission denials heading, but they should be there too!
  Context: not specified
- Completed: 2026-03-25

- [ ] **#66** Replace tilde paths with absolute paths in all subagent prompts referencing ~/.claude *(implemented)*
  [D:20 C:85] Search all Implementer and Reviewer subagent prompts in pathfinder-mission-team/SKILL.md for ~/.claude references and substitute the literal path C:/Users/solar/.claude, eliminating the recurring permission-hook denial class from tilde expansion.
  Claude: "Replace tilde paths with absolute paths in all Implementer and Reviewer subagent prompts that reference ~/.claude" -- Substituting the literal home directory path (C:/Users/solar/.claude) for ~ in skill prompts eliminates a recurring permission-hook denial class with no functional change.
  Context: Mission 6 post-mortem (pathfinder/SUCCESSFUL-MISSION-LOG-6-skill-log-quality.md) - Casualties 1-3 show subagents searching ~/.claude/skills/** with tilde paths blocked by the permission hook.
- Completed: 2026-03-26T04:27:39

- [ ] **#65** Rewrite MT-3d plan-rename step to use individual mv + git add commands per file *(implemented)*
  [D:20 C:82] Replace the compound bash command in the MT-3d plan-rename block with individual mv and git add calls per file, eliminating the compound-command pattern that consistently triggers the permission hook denial.
  Claude: "Rewrite MT-3d plan-rename step to emit individual mv + git add commands instead of compound bash" -- Changing the rename block to issue one mv and one git add per file eliminates the compound-command pattern that the permission hook rejects, removing a recurring casualty source.
  Context: Mission 6 post-mortem (pathfinder/SUCCESSFUL-MISSION-LOG-6-skill-log-quality.md) - Casualties 4 and 5 show MT-3d plan-rename consistently triggering permission-hook denials with compound commands.
- Completed: 2026-03-26T04:45:12

- [ ] **#68** Upgrade MT-3d Status/Implementation/Timestamp check from soft warning to hard block
  [D:20 C:80] The MT-3d enforcement gate checks Status, Implementation, and Timestamp fields with a soft warning that does not block task completion; upgrade these to a hard block (same pattern as the Agent check added by task #51) so a missing core log field re-queues the task rather than letting it pass with a warning.
  Claude: Task Observation from Mission 6: Both #46 and #46-2 stated the log entry check should be a blocking issue, but the MT-3d enforcement gate for Status/Implementation/Timestamp fields was implemented as a soft warning; only the Agent field (added by #51) became a hard block. Upgrade the Status/Implementation/Timestamp soft-warning check to a hard block so missing core log fields re-queue the task.
  Context: Mission 6 Task Observations (pathfinder/SUCCESSFUL-MISSION-LOG-6-skill-log-quality.md) - Gap between stated intent (#46/#46-2 wanted hard blocks) and what was implemented (soft warning).
- Completed: 2026-03-26T04:57:06

- [ ] **#69** Truncate MISSION-LOG-active.md at mission end to prevent indefinite accumulation *(implemented)*
  [D:15 C:88] Add a truncation step to MT-4 that empties MISSION-LOG-active.md after relevant entries have been copied to the numbered log, so each mission starts with a clean file instead of inheriting all prior mission history.
  Claude: Truncate MISSION-LOG-active.md at mission end (MT-4) -- the active log is never cleared between missions, causing it to accumulate indefinitely (87KB after 6 missions). MT-4 should truncate or empty the file after copying relevant entries to the numbered log, so each mission starts with a clean slate.
  Context: pathfinder/MISSION-LOG-active.md found at 87KB, containing entries from all 6 missions with no rotation/clear step
- Completed: 2026-03-26T05:04:05

- [ ] **#67** Store premission rank in PRIORITY_MAP from BRIEF and use it as primary sort key in MT-2 and MT-3a *(implemented sub-task 2)*
  [D:40 C:72] Extend MT-1 step 2-A to extract each task's list position from ## Task Priority Order as its PRIORITY_MAP rank; update MT-2 reorder and MT-3a tie-break to sort by rank before D score, making execution order match the user-reviewed premission sequence.
  Claude: "Store premission rank in PRIORITY_MAP during MT-1 2-A branch and use it as the primary sort key in MT-2 and MT-3a" -- After sub-task 42.1 added Task Priority Order to the BRIEF, MT-1 2-A reads task IDs from that section but does not extract their position as a rank value; storing rank-as-priority and sorting by it before D score would make execution order match the user-reviewed premission sequence.
  Context: Mission 6 post-mortem (pathfinder/SUCCESSFUL-MISSION-LOG-6-skill-log-quality.md) - All M6 tasks ran at P:99 with MT-2/MT-3a falling back to D-score ordering, inverting the user-set premission sequence.
- Completed: 2026-03-26T05:31:20

- [ ] **#63** Cross-reference PROJECT-TESTS.md criteria into task descriptions at creation time
  [D:35 C:68] Modify the task-creation flow to check PROJECT-TESTS.md for criteria matching a new task and include them in the TASKS.md entry, so implementers see the full acceptance bar without consulting a separate file.
  Claude: "Cross-reference PROJECT-TESTS.md criteria into TASKS.md task descriptions at task-creation time" -- When a task is added, any matching PROJECT-TESTS.md criterion should be copied into or linked from the task description so implementers see the full acceptance bar without consulting a separate file.
  Context: Mission 6 post-mortem (pathfinder/SUCCESSFUL-MISSION-LOG-6-skill-log-quality.md) - Task #42 attempt 1 failed because a required test criterion existed in PROJECT-TESTS.md but was absent from the task description.
- Completed: 2026-03-26T06:02:22

- [ ] **#56** Log command usage per mission; add Default Permissions baseline pulled into each premission
  [D:45 C:55] Mission-team records which approved commands were actually invoked vs. unused during a run; a persistent DEFAULT-PERMISSIONS file is read by premission as a starting baseline so per-mission manifests extend rather than replace accumulated history, preventing nuanced per-permission rationale from being silently overwritten.
  Joseph-Raw: Can the pathfinder mission note which commands did and didn't get used? I'd like to set up a "Default Permissions" file, that the pathfinder's can automatically pull from for each individual mission reliably (so that we don't accidentally overwrite a permission with a nuanced history).
  Context: not specified
  - [ ] **#56-2** Track per-mission command hit counts in DEFAULT-PERMISSIONS; add post-mortem recommendation section
    [D:30 C:58] Extend DEFAULT-PERMISSIONS to record how many missions used each command (binary used/unused per mission, not individual call counts), surfacing high-frequency entries as essential inclusions; add a dedicated post-mortem section for recommending commands be promoted to DEFAULT-PERMISSIONS with written justification.
    Joseph-Raw: It'd be nice if the DEFAULT-PERMISSIONS could also track the number of missions that ended up using that command (more than 0 times is sufficient, no need to count individual uses of each command). Over time, we'll see which commands are VERY IMPORTANT TO INCLUDE. And the MISSION post-mortem should include a new section about recommending a command be added to the default-permissions, and justify what issues it could prevent.
    Context: not specified
- Completed: 2026-03-26T06:32:48

- [ ] **#55** Track premission duration and show estimate before committing to session
  [D:40 C:55] Add start/end timestamps to pathfinder-premission itself and compute a pre-session duration estimate (using D/C/U metrics) displayed before the user commits, so they can trim the task list when the estimated premission time exceeds available time.
  Joseph-Raw: I'd like to start tracking how long the pathfinder premission takes too. I assume this should go in the MISSION BRIEF, or recommend a better place. It should include the calculations we're tracking for missions too, premission start process / end process, estimates for duration vs D, C, U. For especially large premissions, I'd like to not be starting one that'll take 30 minutes to complete properly if I only have 10 minutes lol. I'd rather chop the list down, get the premission done, then send the pathfinder team on those priority items before I leave.
  Context: not specified
- Completed: 2026-03-26T06:43:56

- [ ] **#58** Resolve collision between TASKS.md sub-entry format (#N-2) and pathfinder sub-task nomenclature
  [D:35 C:40] TASKS.md uses #N-2 / #N-3 suffixes for supplementary context entries under a parent task, but pathfinder-mission-team uses its own sub-task numbering internally. When the mission team reads TASKS.md and encounters an entry like #53-2, it likely misinterprets it as a prior-run decomposed sub-task rather than a clarification/context record for #53, causing incorrect task-list parsing or re-queue behavior.
  Joseph-Raw: I'm pretty sure pathfinder-mission-team doesn't handle entries in TASKS like #53-2 very well. I suspect it conflicts with their subtask nomenclature, but in TASKS it's supposed to be additional information and context on #53
  Context: not specified
- Completed: 2026-03-26T06:59:59

- [ ] **#60** Add Initial and Current Estimated Completion Time fields to MISSION-LOG Task Status
  [D:30 C:55] Under ## Task Status in MISSION-LOG, add two wall-clock ETA fields: "Initial Estimated Completion Time: HH:mm (Started at HH:mm)" computed once at mission start from total D * 0.43 min/D rate, and "Current Estimated Completion Time: HH:mm (Updated at HH:mm)" recomputed from remaining D each time a new task begins (not sub-tasks).
  Joseph-Raw: In the MISSION-LOG-#, under ## Task Status, let's add some information so I can check in mid-mission.
  Context: not specified
- Completed: 2026-03-26T07:16:31


- [ ] **#57** Fix M6 Start-Time recorded ~4 hours ahead of actual local time
  [D:20 C:45] MISSION-LOG-6 shows Start-Time T19:06 but the user reports it is ~15:12 and the mission just started; the timestamp is ~4 hours ahead of actual. Likely a timezone offset being applied incorrectly (double-counted or wrong sign) in the pathfinder Start-Time recording step, introduced after task #36 switched timestamps to Toronto local time.
  Joseph-Raw: Pretty sure M6 Start-Time is wrong. It says T19:06, but it's 3:12PM right now. It only started a few minutes ago, not... 4 hours in the future?? I'm guessing all times will be off for this mission, but I'm not interupting it for just this.
  Context: not specified
- Completed: 2026-03-26T07:50:00

- [ ] **#59** Mirror PreCompact hook entries to the numbered MISSION-LOG file, not just MISSION-LOG-active
  [D:15 C:60] The PreCompact hook (added in #32) logs compact events to MISSION-LOG-active.md but not to the permanent numbered MISSION-LOG-N-*.md file. Compact events should appear in the human-readable mission log so post-mission review shows exactly when compaction occurred without needing to cross-reference a separate file.
  Joseph-Raw: On M6, at the 2 hour mark, I checked the logs and the active instance. I believe the precompact hook is firing, and logging into MISSION-LOG-active, but I'd like an entry in the human-readable MISSION-LOG-6* as well!
  Context: not specified
- Completed: 2026-03-26T11:52:00

- [ ] **#61** Add remaining count to Difficulty field in MISSION-LOG mission section
  [D:15 C:52] Update the Difficulty display format in MISSION-LOG files to append remaining difficulty in parentheses, e.g. "Difficulty: 3/10 (7 remaining)". Small formatting change to an existing log field updated by pathfinder-mission-team.
  Joseph-Raw: In the MISSION-LOG-#, under ## Mission, change "Difficulty: {total completed}/{mission total}" to "Difficulty: {total completed}/{mission total} ({total remaining} remaining)"
  Context: not specified
- Completed: 2026-03-26T12:10:00

- [ ] **#62** Omit (P:99) priority annotation from Tasks list in MISSION-LOG ## Mission section
  [D:15 C:42] Tasks listed in the ## Mission Tasks field of MISSION-LOG should drop the "(P:N)" priority suffix so the list reads as plain task IDs (e.g. #19, #43, #47) without redundant annotation noise.
  Joseph-Raw: In the MISSION-LOG-#, under ## Mission, Tasks: can omit (P:99) on each task. #19, #43, #47, etc is fine.
  Context: not specified
- Completed: 2026-03-26T12:20:00
