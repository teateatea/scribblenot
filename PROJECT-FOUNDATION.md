# Project Foundation

## Goals
This mission improves the Pathfinder autonomous-coding system's reliability, safety, and signal quality when applied to real projects. It addresses seven defects discovered during the first pathfinder run on scribblenot, covering mission log integrity, permission modeling, TDD feasibility detection, subagent cost control, scope validation, pre-mission clarification quality, and task provenance labeling.

## Requirements
- The mission log hook must only record genuine permission denial events, not every tool call
- Premission must enumerate and obtain approval for every command the mission loop may invoke, including `cargo test`
- The Decomposer must detect TUI and event-loop sub-tasks where compile-time failing tests cannot be written and set `test_runner` to `none` for those sub-tasks
- The Decomposer must cap sub-task output at 5 sub-tasks (or apply a coarseness heuristic) to prevent subagent count explosion
- Mission initialization (MT-1) must cross-check each task against `MISSION-PERMISSIONS.json` and skip with a log entry any task not covered by the premission scope
- Premission must prompt targeted clarifying questions for any task where the D score exceeds the C score before going dark
- The add-task skill must label the task submitter as `Joseph:` or `Claude:` instead of the generic `Original:` label

## Explicit Non-Goals
- No changes to scribblenot application code (TUI, Rust source files) as part of this mission
- No redesign of the Pathfinder mission loop architecture beyond the seven listed fixes
- No new Pathfinder features beyond what is required to address the identified defects
- No changes to the D/C scoring system itself, only how scores are acted on during premission

## Constraints
No DISCUSSION-*.md files exist; constraints are derived from task descriptions:
- All fixes must apply to the Pathfinder tooling layer, not the scribblenot application under test
- Changes to hook behavior must not suppress legitimate permission denial records
- The sub-task cap must not prevent correct decomposition of genuinely multi-step tasks; grouping is acceptable where steps are tightly coupled
- Scope validation at MT-1 must produce a log entry on skip so mission runs remain auditable
- Clarifying-question logic must fast-path low-D tasks to avoid unnecessary friction on trivial items
- Task provenance labels must reflect actual source at time of task creation, not be assigned retroactively in bulk

## Test Criteria
The mission is complete when: the mission log contains only genuine permission denial entries with no spurious noise; `MISSION-PERMISSIONS.json` covers every command a mission loop run requires; the Decomposer correctly marks infeasible-TDD sub-tasks and stays within the 5 sub-task cap; MT-1 skips and logs out-of-scope tasks rather than running them; premission prompts the user on all D > C tasks before going dark; and every new task entry carries a `Joseph:` or `Claude:` attribution label.
