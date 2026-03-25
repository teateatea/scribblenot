# TASKS

## scribblenot

_Tasks for active development. Feature backlog lives in TODOS.md._

- [ ] **#21** Add persistent group-jump hotkeys in map column (e.g. Q=Intake, W=Subjective, F=Treatment)
  [D:62 C:55] When focus is on the map column, a fixed set of keys should always jump to the first section of each group regardless of current cursor position (e.g. F while on Post-Tx jumps to Tx Mods). These group-reserved keys must be excluded from the section-hint pool so no section hint is ever assigned a character that conflicts with a group jump key.
  Joseph: I'd like the hints for group headings to always be available when in the map column, regardless of what section the cursor is on. So Q INTAKE, W SUBJECTIVE, F TREATMENT, etc, the Q W F etc will always let me jump directly to that group. If I'm not currently in that group, put the cursor at the first section (if I'm over Post-Tx, and I hit F for TREATMENT group, it'll jump me to Tx Mods. Make sure the distribution correctly accounts for this, the section hints won't be able to ever use those group hint characters.
  Context: not specified

- [ ] **#2** Add Shift+Enter super-confirm keybinding to auto-complete remaining fields
  [D:70 C:55] Implement a Shift+Enter keybinding that, when pressed in any field or wizard modal, automatically confirms all remaining parts using already-confirmed values first, then sticky/default values -- skipping user interaction for fields that already have a valid answer.
  Joseph: Add Shift+Enter, for a "super confirm". Add an option for it in keybindings please. Super-confirm can be used on a field to automatically enter whatever is in the text box: Any entries that already got confirmed (green), then Sticky values and default values (grey). This should work in any field, but the example for Date would be a) Select Day: 24 to update the day, then Shift+Enter to auto-confirm the already correct Month and Year, or even b) if the Day is already a correct sticky, a Shift+Enter from the wizard directly skips all the modals and puts the sticky 2026-03-24.
  Context: not specified

---

## Code Quality

- [ ] **#1** Remove dead code warning for unused `current_value` on `HeaderState`
  [D:10 C:55] Delete or use the `pub fn current_value()` method in `src/sections/header.rs` that triggers a dead_code warning on every `cargo build`/`cargo run`.
  Joseph: about that dead code clean up, I don't like that it pops up when I cargo run.
  Context: not specified

---

## Pathfinder Improvements

- [x] **#3** Fix hook logging all tool calls as PERMISSION DENIED in mission log
  [D:30 C:60] The MISSION-LOG-active.md hook appends every tool call as a permission denial entry regardless of outcome, polluting mission logs with noise. Needs hook filter on actual denial exit codes or watch-pattern rename.
  Joseph: Hook collision: MISSION-LOG-active.md hook appends every tool call as a PERMISSION DENIED entry regardless of outcome - pollutes mission logs with noise. Fix: rename active log away from hook watch pattern, or filter hook on actual denial exit codes.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization

- [x] **#4** Premission should enumerate all commands the mission loop may run
  [D:20 C:60] MISSION-PERMISSIONS.json only includes `cargo build` and `cargo clippy` but the mission TDD phase also needs `cargo test`. Premission must require explicit approval for every command the loop may invoke.
  Joseph: MISSION-PERMISSIONS.json goes stale after premission: only `cargo build` and `cargo clippy` were approved for dead-code-cleanup mission, but the mission loop needs `cargo test` too. Fix: premission should enumerate all commands the mission loop may run (including cargo test) and require explicit approval per command.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization

- [x] **#5** Add TDD-feasibility check to Decomposer for event-loop and TUI code
  [D:40 C:60] The mission loop assumes failing unit tests can always be written before implementation, but crossterm key-event handling has no test harness. The Decomposer should detect sub-tasks where compile-time failing tests are infeasible and set test_runner to none.
  Joseph: TDD feasibility gap for TUI/event-loop apps: pathfinder assumes failing unit tests can always be written before implementation, but crossterm key-event handling is tightly coupled to mutable TUI state and has no test harness. Fix: add a TDD-feasibility check step to the Decomposer that sets test_runner to none for sub-tasks where compile-time tests are impossible to write.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization

- [x] **#6** Cap or coarsen Decomposer sub-task count to reduce subagent overhead
  [D:25 C:60] Decomposer can produce 7+ sub-tasks per task, each triggering up to 6 subagent spawns, totalling 40+ agents for one feature. Add a 5 sub-task cap or coarseness heuristic to group related incremental steps.
  Joseph: Decomposer sub-task count bloat: for tightly coupled incremental features the decomposer can produce 7+ sub-tasks, each triggering 6 subagent spawns. Fix: add a cap (e.g. 5 sub-tasks max) or a coarseness heuristic to the Decomposer prompt so related implementation steps are grouped.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization

- [x] **#7** Mission MT-1 must validate task list against premission scope before starting
  [D:35 C:60] Tasks added after premission was run will have no approved permissions or test criteria. Mission initialization must cross-check each task against MISSION-PERMISSIONS.json and skip (with log entry) any task not explicitly covered.
  Joseph: Mission must validate premission scope before starting: if a task was added after premission was run it will not have approved permissions, tool allowlists, or test criteria. Fix: at MT-1 initialization, cross-check each task in TASK_LIST against MISSION-PERMISSIONS.json approved_actions and TASKS.md entries; skip any task not explicitly covered in premission and log the skip.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization

- [x] **#8** Premission should prompt clarifying questions when a task's D score exceeds its C score
  [D:25 C:55] During premission setup, any task where difficulty exceeds clarity confidence should trigger targeted user questions before going dark; low-D tasks can be fast-pathed automatically while high-D/low-C tasks warrant deeper discussion to avoid mid-mission surprises.
  Joseph: In pathfinder PREMISSION, any task with D greater than C should be discussed to clarify. Likely the minor fixes can be passed over quickly, anything that's Difficult but not Clear should prompt additional questioning while the user is available.
  Context: First pathfinder mission run on scribblenot, flagged during graceful mission exit

- [x] **#9** Replace `Original:` label in add-task entries with `Joseph:` or `Claude:` to indicate who submitted the task
  [D:10 C:58] The add-task skill's entry format uses a generic `Original:` label, but tasks can be submitted by the user (Joseph) or autonomously by Claude. Replacing the label with the actual source prevents ambiguity when reviewing task history.
  Joseph: The add-task skill can be prompted by either Joseph OR Claude. Change "Original" to either "Joseph:" or "Claude:" as appropriate to prevent ambiguity about the original source.
  Context: First pathfinder mission run on scribblenot, flagged during graceful mission exit

- [x] **#10** ~~Commander should execute simple skill/config edits directly without spawning subagents~~ [ABANDONED]
  [D:35 C:55] Adds a routing heuristic to pathfinder-mission-team where the Commander skips the Planner/Reviewer/Implementer subagent loop for low-complexity tasks (skill/config file edits, no compilation or deep exploration needed), executing changes directly in the main conversation to preserve context budget, avoid permission walls, and prevent Rule #31 casualties; reserves the full subagent loop for high-D, destructive, or exploration-heavy tasks.
  Claude: When pathfinder runs tasks that only edit skill/config files (no code compilation or complex exploration needed), the Commander should do the work directly in the main conversation rather than spawning Planner/Reviewer/Implementer subagents. Subagents cost context, hit permission walls, and can't create .md files (Rule #31). Direct execution from the main conversation is faster, cheaper, and avoids cascading casualties. The plan-review loop should be reserved for tasks that genuinely benefit from multi-agent review (complex code changes, destructive operations, high D-score tasks).
  Context: not specified

- [x] **#11** Mission team should rename completed task plans to COMPLETED-*.md on success
  [D:30 C:55] After pathfinder-mission-team successfully completes a task, it should rename the corresponding plan file with a COMPLETED- prefix so that /lets-start and future agents can identify and skip stale plans without cross-referencing TASKS.md.
  Joseph: I think /pathfinder-mission-team leaves plans unmarked when complete. Pretty sure after completing each task successfully, it should rename that plan COMPLETED-*.md, so that later agents know to ignore them.
  Context: not specified

- [x] **#12** Track cumulative mission difficulty in MISSION-LOG header, updated after each task
  [D:35 C:52] Add a Difficulty field under the ## MISSION heading in MISSION-LOG that shows completed vs total difficulty (e.g. "Difficulty: X/T"). The pathfinder-mission-team updates this after each task completes, enabling post-failure analysis of a suspected ~200-point difficulty ceiling.
  Joseph: MISSION-LOG should note Difficulty (meaning the total difficulty of all the tasks in this mission), probably under the ## MISSION heading. /pathfinder-mission-team should update that number after each task ("Difficulty: X/T", where X is the total difficulty of tasks completed, and T is the total difficulty of all tasks in this mission), so that if the mission fails, we'll learn about a difficulty ceiling. I suspect we can't go about 200, based on mission 3 (D: 115, main instance context got to 65%)
  Context: not specified

- [x] **#13** Warn user in premission when total mission difficulty exceeds context-load thresholds (140/200)
  [D:35 C:52] The premission step should sum D scores across all tasks and emit a warning if the total exceeds 140, indicating the main instance may run out of context. If the total exceeds 200, require a second explicit confirmation since that load level is untested.
  Joseph: /pathfinder-premission should note when a mission's total difficulty exceeds 140, and warn the user that the main instance's context may be overtaxed by the tasks included. If the total difficulty exceeds 200, confirm a second time to make absolutely certain; it's unknown whether this difficulty is possible.
  Context: not specified

- [x] **#14** Record Start-Time and End-Time in MISSION-LOG for duration tracking
  [D:35 C:45] The mission team should write timestamps when a mission begins and ends into the MISSION-LOG file. This data will eventually enable premission to estimate completion time based on task difficulty.
  Joseph: /pathfinder-mission-team should write Start-Time and End-Time into MISSION-LOG, when it starts and ends. We can start to gather data about difficulty vs duration, so that in the future, premission can give a completion time estimate.
  Context: not specified

- [ ] **#15** Suppress diff-view windows during pathfinder-mission-team execution
  [D:30 C:50] File edits made by mission-team subagents trigger diff view windows that pop over the user's active window during autonomous operation, creating a risk of accidental input if the user is typing elsewhere. Diff windows should be disabled or suppressed for the duration of a mission-team run.
  Joseph: The /pathfinder-mission-team opens diffs when it creates or edits files ( I think). These open in a new window but because the permission gets handled (correctly) by the mission team, no user input is required. However, this is not intended behaviour, and introduces the possiblity of accidental input. Sometimes I'm typng somewhere else, and the window pops open for and it'd be pretty easy for me to cause problems if the timing was unlucky. When I'm working closely with Claude, I do like those diff windows, but they should not be used in pathfinder-mission-team.
  Context: not specified

- [ ] **#16** Preserve Prefect-1 review report when Prefect-2 begins
  [D:20 C:55] During pathfinder-mission-team, the plan file written by Prefect 1 is deleted before Prefect 2 starts, erasing the audit trail of where review changes originated. The Prefect-1 report should be retained so post-mission review can trace issues to specific review stages.
  Joseph: I'm not sure if it's from plan-review-team, or from pathfinder-mission-team, but during a /pathfinder-mission-team, the report that Prefect 1 writes gets removed before P2 begins. This is unnecessary, and actually introduces some ambiguity about where a change came from. Please do not remove that report, I'd like to be able to see where things happened, so I know what to look for if something goes wrong.
  Context: not specified

- [ ] **#17** Organize pathfinder mission artifacts into a dedicated [project]/pathfinder/ directory
  [D:50 C:52] PROJECT-FOUNDATION.md, MISSION-PERMISSIONS.json, and MISSION-LOG files are currently scattered in the project root; they should live under [project]/pathfinder/ to keep mission-specific artifacts separate. PROJECT_LOG.md and project tests remain in .claude/ since they serve broader, non-mission purposes -- user's read on this split is correct.
  Joseph: The files related to pathfinder-premission or mission-team should probably go into a [project]/pathfinder folder. That includes PROJECT-FOUNDATION, MISSION-PERMISSIONS, MISSION-LOGS. I *think* PROJECT_LOG AND PROJECT_TESTS can remain in project/.claude, because those could be used outside of pathfinder missions, but please advise.
  Context: not specified

- [ ] **#18** Audit and eliminate blank-line churn in mission-team review loop
  [D:25 C:42] Diffs during pathfinder-mission-team show whitespace-only changes (e.g. two blank lines collapsed to one), suggesting one phase writes extra blank lines that a subsequent phase removes. Need to identify which phase introduces the unnecessary lines and stop adding them at the source.
  Joseph: In /pathfinder-mission-team, I notice some of the diffs involve only tidying up whitespace, like changing two blank lines into one. Please check the review loop, are we writing in blank lines, then removing them? If so, let's just stop adding the unnecessary lines to begin with.
  Context: not specified

- [ ] **#20** Fix mission-team timestamps to use actual time, not hardcoded midnight
  [D:10 C:72] MISSION-LOG entries show timestamps like `2026-03-24T00:00:00Z` -- the date is correct but time is always midnight, indicating the skill constructs timestamps from the date alone without calling `date` to get the real time. Fix: run `date` and format the result as a proper ISO 8601 timestamp.
  Joseph: I'm pretty sure /pathfinder-mission-team doesn't run bash(date), I'm seeing multiple "Timestamp: 2026-03-24T00:00:00Z" entries. Please correctly add the time, not just the date!
  Context: not specified

- [ ] **#19** Remove the C:60 cap from add-task initial scoring; allow full 0-99 range
  [D:10 C:58] The add-task skill clamps Clarity Confidence at 60, deferring higher scores to later review passes. The cap should be removed so the initial score can reflect the full 0-99 range accurately. Review passes then serve their intended purpose -- correcting toward accuracy -- rather than mechanically bumping capped values upward.
  Joseph: I believe it's the add-task skill that is only allowed to score a new task up to C: 60, not greater. Originally, I thought we'd wait for a later review to bump it higher. Let's not do that, the initial score should be allowed to be the full range 0-99. This let's the initial scoring not be artificially clamped down. We can still do later reviews, but instead of those reviews pushing tasks higher, they'll now be pushing the clarity towards more *accurate*, which is the intended goal anyways.
  Context: not specified

---
