# TASKS

## scribblenot

_Tasks for active development. Feature backlog lives in TODOS.md._

- [ ] **#2** Add Shift+Enter super-confirm keybinding to auto-complete remaining fields
  [D:70 C:55] Implement a Shift+Enter keybinding that, when pressed in any field or wizard modal, automatically confirms all remaining parts using already-confirmed values first, then sticky/default values -- skipping user interaction for fields that already have a valid answer.
  Original: Add Shift+Enter, for a "super confirm". Add an option for it in keybindings please. Super-confirm can be used on a field to automatically enter whatever is in the text box: Any entries that already got confirmed (green), then Sticky values and default values (grey). This should work in any field, but the example for Date would be a) Select Day: 24 to update the day, then Shift+Enter to auto-confirm the already correct Month and Year, or even b) if the Day is already a correct sticky, a Shift+Enter from the wizard directly skips all the modals and puts the sticky 2026-03-24.
  Context: not specified

---

## Code Quality

- [ ] **#1** Remove dead code warning for unused `current_value` on `HeaderState`
  [D:10 C:55] Delete or use the `pub fn current_value()` method in `src/sections/header.rs` that triggers a dead_code warning on every `cargo build`/`cargo run`.
  Original: about that dead code clean up, I don't like that it pops up when I cargo run.
  Context: not specified

---

## Pathfinder Improvements

- [ ] **#3** Fix hook logging all tool calls as PERMISSION DENIED in mission log
  [D:30 C:60] The MISSION-LOG-active.md hook appends every tool call as a permission denial entry regardless of outcome, polluting mission logs with noise. Needs hook filter on actual denial exit codes or watch-pattern rename.
  Original: Hook collision: MISSION-LOG-active.md hook appends every tool call as a PERMISSION DENIED entry regardless of outcome - pollutes mission logs with noise. Fix: rename active log away from hook watch pattern, or filter hook on actual denial exit codes.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization

- [ ] **#4** Premission should enumerate all commands the mission loop may run
  [D:20 C:60] MISSION-PERMISSIONS.json only includes `cargo build` and `cargo clippy` but the mission TDD phase also needs `cargo test`. Premission must require explicit approval for every command the loop may invoke.
  Original: MISSION-PERMISSIONS.json goes stale after premission: only `cargo build` and `cargo clippy` were approved for dead-code-cleanup mission, but the mission loop needs `cargo test` too. Fix: premission should enumerate all commands the mission loop may run (including cargo test) and require explicit approval per command.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization

- [ ] **#5** Add TDD-feasibility check to Decomposer for event-loop and TUI code
  [D:40 C:60] The mission loop assumes failing unit tests can always be written before implementation, but crossterm key-event handling has no test harness. The Decomposer should detect sub-tasks where compile-time failing tests are infeasible and set test_runner to none.
  Original: TDD feasibility gap for TUI/event-loop apps: pathfinder assumes failing unit tests can always be written before implementation, but crossterm key-event handling is tightly coupled to mutable TUI state and has no test harness. Fix: add a TDD-feasibility check step to the Decomposer that sets test_runner to none for sub-tasks where compile-time tests are impossible to write.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization

- [ ] **#6** Cap or coarsen Decomposer sub-task count to reduce subagent overhead
  [D:25 C:60] Decomposer can produce 7+ sub-tasks per task, each triggering up to 6 subagent spawns, totalling 40+ agents for one feature. Add a 5 sub-task cap or coarseness heuristic to group related incremental steps.
  Original: Decomposer sub-task count bloat: for tightly coupled incremental features the decomposer can produce 7+ sub-tasks, each triggering 6 subagent spawns. Fix: add a cap (e.g. 5 sub-tasks max) or a coarseness heuristic to the Decomposer prompt so related implementation steps are grouped.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization

- [ ] **#7** Mission MT-1 must validate task list against premission scope before starting
  [D:35 C:60] Tasks added after premission was run will have no approved permissions or test criteria. Mission initialization must cross-check each task against MISSION-PERMISSIONS.json and skip (with log entry) any task not explicitly covered.
  Original: Mission must validate premission scope before starting: if a task was added after premission was run it will not have approved permissions, tool allowlists, or test criteria. Fix: at MT-1 initialization, cross-check each task in TASK_LIST against MISSION-PERMISSIONS.json approved_actions and TASKS.md entries; skip any task not explicitly covered in premission and log the skip.
  Context: First pathfinder mission run on scribblenot, flagged during mission initialization

- [ ] **#8** Premission should prompt clarifying questions when a task's D score exceeds its C score *(implemented)*
  [D:25 C:55] During premission setup, any task where difficulty exceeds clarity confidence should trigger targeted user questions before going dark; low-D tasks can be fast-pathed automatically while high-D/low-C tasks warrant deeper discussion to avoid mid-mission surprises.
  Original: In pathfinder PREMISSION, any task with D greater than C should be discussed to clarify. Likely the minor fixes can be passed over quickly, anything that's Difficult but not Clear should prompt additional questioning while the user is available.
  Context: First pathfinder mission run on scribblenot, flagged during graceful mission exit

- [ ] **#9** Replace `Original:` label in add-task entries with `Joseph:` or `Claude:` to indicate who submitted the task
  [D:10 C:58] The add-task skill's entry format uses a generic `Original:` label, but tasks can be submitted by the user (Joseph) or autonomously by Claude. Replacing the label with the actual source prevents ambiguity when reviewing task history.
  Joseph: The add-task skill can be prompted by either Joseph OR Claude. Change "Original" to either "Joseph:" or "Claude:" as appropriate to prevent ambiguity about the original source.
  Context: First pathfinder mission run on scribblenot, flagged during graceful mission exit

- [ ] **#10** Commander should execute simple skill/config edits directly without spawning subagents
  [D:35 C:55] Adds a routing heuristic to pathfinder-mission-team where the Commander skips the Planner/Reviewer/Implementer subagent loop for low-complexity tasks (skill/config file edits, no compilation or deep exploration needed), executing changes directly in the main conversation to preserve context budget, avoid permission walls, and prevent Rule #31 casualties; reserves the full subagent loop for high-D, destructive, or exploration-heavy tasks.
  Claude: When pathfinder runs tasks that only edit skill/config files (no code compilation or complex exploration needed), the Commander should do the work directly in the main conversation rather than spawning Planner/Reviewer/Implementer subagents. Subagents cost context, hit permission walls, and can't create .md files (Rule #31). Direct execution from the main conversation is faster, cheaper, and avoids cascading casualties. The plan-review loop should be reserved for tasks that genuinely benefit from multi-agent review (complex code changes, destructive operations, high D-score tasks).
  Context: not specified

---
