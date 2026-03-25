# TASKS

## scribblenot

_Tasks for active development. Feature backlog lives in TODOS.md._

- [ ] **#44** Add /add-tasks as a forwarding alias to /add-task without duplicating the skill
  [D:15 C:60] Create a minimal /add-tasks skill entry that immediately delegates to /add-task so both trigger words work; the alias contains no logic of its own, avoiding a maintenance burden when /add-task changes.
  Joseph: The /add-task skill should also trigger on /add-tasks. It's too easy for me to add that s when I'm thining about adding several, and it might as well work correctly. Don't just copy the /add-task skill though, I don't want to have to maintain identical skills.
  Context: not specified

- [ ] **#21** Add persistent group-jump hotkeys in map column (e.g. Q=Intake, W=Subjective, F=Treatment)
  [D:62 C:55] When focus is on the map column, a fixed set of keys should always jump to the first section of each group regardless of current cursor position (e.g. F while on Post-Tx jumps to Tx Mods). These group-reserved keys must be excluded from the section-hint pool so no section hint is ever assigned a character that conflicts with a group jump key.
  Joseph: I'd like the hints for group headings to always be available when in the map column, regardless of what section the cursor is on. So Q INTAKE, W SUBJECTIVE, F TREATMENT, etc, the Q W F etc will always let me jump directly to that group. If I'm not currently in that group, put the cursor at the first section (if I'm over Post-Tx, and I hit F for TREATMENT group, it'll jump me to Tx Mods. Make sure the distribution correctly accounts for this, the section hints won't be able to ever use those group hint characters.
  Context: not specified

- [ ] **#2** Add Shift+Enter super-confirm keybinding to auto-complete remaining fields
  [D:70 C:55] Implement a Shift+Enter keybinding that, when pressed in any field or wizard modal, automatically confirms all remaining parts using already-confirmed values first, then sticky/default values -- skipping user interaction for fields that already have a valid answer.
  Joseph: Add Shift+Enter, for a "super confirm". Add an option for it in keybindings please. Super-confirm can be used on a field to automatically enter whatever is in the text box: Any entries that already got confirmed (green), then Sticky values and default values (grey). This should work in any field, but the example for Date would be a) Select Day: 24 to update the day, then Shift+Enter to auto-confirm the already correct Month and Year, or even b) if the Day is already a correct sticky, a Shift+Enter from the wizard directly skips all the modals and puts the sticky 2026-03-24.
  Context: not specified

- [ ] **#23** Auto-generate multi-character hint permutations from base hint characters for overflow assignment
  [D:55 C:58] When the base hint pool (e.g. [q,w,f,p]) is smaller than the number of hints needed, generate 2-char (and if needed, 3-char+) permutations using n^r logic and append them to the hint list in keybindings.yml. Permutations should be priority-ordered by adjacency in the base list -- pairs of adjacent characters (qq, qw, wq, ww, wf, fw...) appear before distant pairs (qp, pq) -- so the most ergonomic combos are assigned first.
  Joseph: I'd like to be able to have any number of hint characters in the keybindings.yml. Currently, if I had hints: [q,w,f,p], anytime we need more than 4 hint characters, there's just no hint at all. Instead, let's prepare a list of hint permutations (that should be added to keybindings below hints), up to the number of max number of hints that we'll need. (The formula # permutations = n^r = 3^2 = 9 should help, n is the number of items/hint characters(3), r is the items to select (2), I doubt r will ever be more than 2 or 3 but I'd like the process to be resilient enough to handle any number). This should result in having a new list of hints, something like qq, qw, qf,qp, wq, ww, wf, etc. Ideally, use characters adjacent to each other in the hints list first, so qq, qw,wq,ww,wf,fw,ff,fp,pf,pp should have higher priority (appearing at the start of the permutations list), and qp should be near the end.
  Context: not specified

- [ ] **#22** Implement multi-character hint sequences with progressive prefix filtering
  [D:65 C:60] Add a hint-input buffer/state machine so that typing the first character of a multi-char hint (e.g. "z" for "zz"/"zx") highlights matching hints' typed prefix in white and grays out non-matching hints, then waits for the next keypress to resolve; a keypress with no remaining match resets all hints to normal active state. Single-character hints unaffected.
  Joseph: I'd like to be able to use multi-character hints. For example, I have hints: zz, zx in keybindings. It's currently impossible to type zz as a single key of course. Ideally, when I hit the first letter of the hint, the first character from *all* hints that match should turn white, and any hints that don't match should gray out. So typing z would highlight the leading z on both zz and zx, and deactivates all other hints. Then the next key that I type checks the next letter (the non-white second character now), so typing either z or x will let me select which of the two I'm aiming for. If I type a letter that doesn't correspond with the remaining hints, reset all hints to regular active (magenta). So typing zm briefly focuses on the zz and zx hints, then resets because nothing was found.
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

- [x] **#15** Suppress diff-view windows during pathfinder-mission-team execution *(implemented)*
  [D:30 C:50] File edits made by mission-team subagents trigger diff view windows that pop over the user's active window during autonomous operation, creating a risk of accidental input if the user is typing elsewhere. Diff windows should be disabled or suppressed for the duration of a mission-team run.
  Joseph: The /pathfinder-mission-team opens diffs when it creates or edits files ( I think). These open in a new window but because the permission gets handled (correctly) by the mission team, no user input is required. However, this is not intended behaviour, and introduces the possiblity of accidental input. Sometimes I'm typng somewhere else, and the window pops open for and it'd be pretty easy for me to cause problems if the timing was unlucky. When I'm working closely with Claude, I do like those diff windows, but they should not be used in pathfinder-mission-team.
  Context: not specified

- [x] **#16** Preserve Prefect-1 review report when Prefect-2 begins *(implemented)*
  [D:20 C:55] During pathfinder-mission-team, the plan file written by Prefect 1 is deleted before Prefect 2 starts, erasing the audit trail of where review changes originated. The Prefect-1 report should be retained so post-mission review can trace issues to specific review stages.
  Joseph: I'm not sure if it's from plan-review-team, or from pathfinder-mission-team, but during a /pathfinder-mission-team, the report that Prefect 1 writes gets removed before P2 begins. This is unnecessary, and actually introduces some ambiguity about where a change came from. Please do not remove that report, I'd like to be able to see where things happened, so I know what to look for if something goes wrong.
  Context: not specified
  - [x] **#16-2** Eliminate mechanical Prefect Report cleanup step between Pass 1 and Pass 2
    [D:25 C:55] After Pass 1 returns PREFECT FIXED, the Commander reads and edits out the Prefect Report section before spawning Pass 2 -- pure overhead. Options: (a) Pass 1 outputs findings as return text only; (b) Pass 2 ignores/overwrites any existing Prefect Report section; (c) Pass 1 writes findings to a temp location instead of the plan file.
    Claude: Eliminate manual Prefect Report removal step between Pass 1 and Pass 2: after Prefect Pass 1 returns PREFECT FIXED, the Commander must read the plan file, find the Prefect Report section boundary, edit it out, then spawn Pass 2. This is pure mechanical overhead. Options: (a) instruct Prefect Pass 1 to output its findings only as return text, not write them to the plan file; (b) instruct Prefect Pass 2 to ignore and overwrite any existing Prefect Report section without requiring Commander intervention; or (c) have Prefect Pass 1 write findings to a separate temp location rather than injecting into the plan.
    Context: Observed during mission 4 (tdd-warn-tracking). Every PREFECT FIXED result required a Commander read-edit-spawn cycle before Pass 2 could run, adding overhead to every plan that needed Prefect fixes.

- [x] **#17** Organize pathfinder mission artifacts into a dedicated [project]/pathfinder/ directory
  [D:50 C:52] PROJECT-FOUNDATION.md, MISSION-PERMISSIONS.json, and MISSION-LOG files are currently scattered in the project root; they should live under [project]/pathfinder/ to keep mission-specific artifacts separate. PROJECT_LOG.md and project tests remain in .claude/ since they serve broader, non-mission purposes -- user's read on this split is correct.
  Joseph: The files related to pathfinder-premission or mission-team should probably go into a [project]/pathfinder folder. That includes PROJECT-FOUNDATION, MISSION-PERMISSIONS, MISSION-LOGS. I *think* PROJECT_LOG AND PROJECT_TESTS can remain in project/.claude, because those could be used outside of pathfinder missions, but please advise.
  Context: not specified

- [x] **#18** Audit and eliminate blank-line churn in mission-team review loop
  [D:25 C:42] Diffs during pathfinder-mission-team show whitespace-only changes (e.g. two blank lines collapsed to one), suggesting one phase writes extra blank lines that a subsequent phase removes. Need to identify which phase introduces the unnecessary lines and stop adding them at the source.
  Joseph: In /pathfinder-mission-team, I notice some of the diffs involve only tidying up whitespace, like changing two blank lines into one. Please check the review loop, are we writing in blank lines, then removing them? If so, let's just stop adding the unnecessary lines to begin with.
  Context: not specified
  - [x] **#18-2** Audit Planner prompt template for blank-line generation patterns
    [D:20 C:50] The Planner prompt or its examples may be the source of extra blank lines that reviewers then flag and remove as whitespace-only diffs. Targeting the Planner specifically to eliminate the root cause rather than just the symptom.
    Claude: Blank line churn in plan review loop: planners are adding extra blank lines that reviewers or prefects then flag and remove, producing whitespace-only diffs with no semantic value. Audit the Planner prompt template to remove any instructions or examples that generate double blank lines between sections.
    Context: Observed during mission 4 (tdd-warn-tracking) and confirmed by Joseph in new task added post-mission. Review loop is spending subagent cycles on blank line normalization.

- [x] **#20** Fix mission-team timestamps to use actual time, not hardcoded midnight *(implemented)*
  [D:10 C:72] MISSION-LOG entries show timestamps like `2026-03-24T00:00:00Z` -- the date is correct but time is always midnight, indicating the skill constructs timestamps from the date alone without calling `date` to get the real time. Fix: run `date` and format the result as a proper ISO 8601 timestamp.
  Joseph: I'm pretty sure /pathfinder-mission-team doesn't run bash(date), I'm seeing multiple "Timestamp: 2026-03-24T00:00:00Z" entries. Please correctly add the time, not just the date!
  Context: not specified

- [x] **#24** Rename mission log to SUCCESSFUL-*.md when all tasks complete
  [D:10 C:75] On full mission completion, pathfinder-mission-team should rename the active MISSION-LOG file with a SUCCESSFUL- prefix so completed missions are immediately distinguishable from in-progress, failed, or abandoned ones without opening the file.
  Joseph: When /pathfinder-mission-team is able to complete all tasks, they should rename their mission log SUCCESSFUL-*.md when they're done.
  Context: not specified

- [x] **#25** Add "Context at finish:" field to Mission Complete section in MISSION-LOG template
  [D:10 C:72] Append a "Context at finish:" line to the ## Mission Complete section so Joseph can optionally record main-instance context usage after each mission. Enables correlation of context % vs mission difficulty over time to determine whether context is actually a limiting factor.
  Joseph: For /pathfinder-mission-team, at the end of the ## Mission Complete section, add a new line "Context at finish:" for Joseph to optionally record. Considering 2 missions have now completed with context usage around 65-70% (despite mission difficulties of 125 and 175), context might not actually matter as much as I thought, but I'd like to record the data to confirm it.
  Context: not specified

- [x] **#29** Add Task Observations and Mission Post-Mortem sections to MISSION-LOG at wrap-up *(implemented)*
  [D:25 C:68] Insert two sections just before ## Mission Complete: "Task Observations" (clear gaps between intent and implementation with suggested next steps -- omit if nothing obvious) and "Mission Post-Mortem" (process inefficiencies noted during this mission, written with enough detail to be submitted directly as /add-task entries). Both written by the mission team during wrap-up.
  Joseph: The /pathfinder-mission team should add two sections just before ## Mission Complete: 1) Task Observations: Please note any obvious next steps related to the completed tasks in this mission, and explain why you think they'd be an improvement. This section can be empty, only record clear gaps in intent vs implementation. 2) Mission Post-Mortem: Please note any inefficiencies that you'd note in the /pathfinder-mission-team process from this mission in enough detail that it could be successfully used as an /add-task.
  Context: not specified
  - [x] **#29-2** Use subagents for wrap-up sections to avoid maxing main instance context
    [D:15 C:60] The Task Observations and Mission Post-Mortem sections should be written via subagents rather than inline on the main instance; running this step in the main context immediately after mission completion previously forced a /compact.
    Joseph: additional instructions for 29: This should continue to use subagents in sequence. Calling this immediately after Mission 3 completed maxxed out the main instance's context forcing a /compact, which should be avoided.
    Context: not specified

- [ ] **#19** Remove the C:60 cap from add-task initial scoring; allow full 0-99 range
  [D:10 C:58] The add-task skill clamps Clarity Confidence at 60, deferring higher scores to later review passes. The cap should be removed so the initial score can reflect the full 0-99 range accurately. Review passes then serve their intended purpose -- correcting toward accuracy -- rather than mechanically bumping capped values upward.
  Joseph: I believe it's the add-task skill that is only allowed to score a new task up to C: 60, not greater. Originally, I thought we'd wait for a later review to bump it higher. Let's not do that, the initial score should be allowed to be the full range 0-99. This let's the initial scoring not be artificially clamped down. We can still do later reviews, but instead of those reviews pushing tasks higher, they'll now be pushing the clarity towards more *accurate*, which is the intended goal anyways.
  Context: not specified

- [x] **#26** Provide prior-attempt context to Decomposer on task re-queues
  [D:30 C:55] When a task is re-queued after failed project tests, the Decomposer prompt should include the prior attempt's sub-tasks and failed test criteria so it generates targeted gap-filling sub-tasks instead of re-discovering the full task scope from scratch.
  Claude: Decomposer context blindness on re-runs: when a task is re-queued after a failed project-test run, the Decomposer has no context about what was already attempted and what gaps remain. For task #5, the first-pass Decomposer generated sub-tasks to verify existing content (already implemented), wasting ~6 subagent spawns before the re-queue cycle generated the correct targeted fix. The Decomposer prompt (or Commander pre-prompt) should include prior attempt context -- what sub-tasks were run, what project-test criteria failed -- so it generates gap-targeted sub-tasks instead of re-discovering the whole task from scratch.
  Context: Observed during mission 4 (tdd-warn-tracking). Task #5 required two full decompose/plan/implement cycles because attempt 1 verified already-present content and missed rationale field + named step criteria.


- [x] **#28** Add mv fallback when git mv fails for gitignored plan files in COMPLETED- rename step *(implemented)*
  [D:20 C:60] Plan files in .claude/plans/ are gitignored in project repos, so git mv fails with "not under version control" for most renames. MT-3d should catch this error and fall back to regular mv, then git add the COMPLETED- file if the destination directory is tracked.
  Claude: git mv fallback for gitignored plan files: plan files live in .claude/plans/ which is gitignored in the scribblenot repo, so git mv fails silently for most of them during the COMPLETED- rename step. The MT-3d rename logic should try git mv first, catch the 'not under version control' error, and fall back to regular mv + git add for the new COMPLETED- file.
  Context: Observed during mission 4 finale. 9 of 12 plan renames required manual mv fallback because .claude/ is gitignored. The git mv partial-failure caused a commit that only captured 3 of 12 renames.

- [x] **#30** Prefix pathfinder plan filenames with mission number (e.g. M5-20-1-slug.md)
  [D:20 C:58] Pathfinder-mission-team should prepend the current mission number to each plan filename it creates, so plans from different missions are immediately distinguishable -- especially useful if a mission is interrupted and plans from multiple missions coexist in .claude/plans/.
  Joseph: Pathfinder plans need to include the mission number too. For example, M5-20-1-three-word-name.md. If a mission is ever interupted for any reason, it would help to be able to distinguish which mission a plan came from.
  Context: not specified

- [x] **#32** Add PreCompact hook to log compact events with timestamp during pathfinder missions
  [D:30 C:55] Install a PreCompact hook that fires just before Claude's automatic /compact, appending a timestamped entry to the active MISSION-LOG so post-mission review can identify exactly when compaction occurred and whether it had any negative effect on mission continuity.
  Joseph: For /pathfinder-mission-team, create a PreCompact hook. Claude agents can't actually see their context usage, so they don't know how close they are to an automatic /compact. IN THEORY, the pathfinder mission team relies fairly heavily on .md to track the progress, so it should be resilient against /compact information dilution. But just before a compact happens, we might as well immediately log it so we know where it happened in the process, what exact time, etc. Then Claude can do a review later to see if there was any negative effect.
  Context: not specified

- [x] **#31** Move completed task entries from TASKS.md to CLOSED-TASKS.md with completion timestamp *(implemented)*
  [D:25 C:58] When pathfinder-mission-team successfully completes a task, it should remove that task's entry from TASKS.md and append it to CLOSED-TASKS.md (creating the file if absent) with the completion date and time, keeping TASKS.md focused only on actionable work.
  Joseph: When /pathfinder-mission-team completes a task, let's actually move it to CLOSED-TASKS.md, just appended at the end of the file (and add the date and time please). No need to have TASKS.md cluttered up with tasks that no longer require action.
  Context: not specified

- [x] **#33** Add second clarification threshold to premission (D > 50, C < 70)
  [D:20 C:72] Extend /pathfinder-premission's clarification question trigger with a second condition: any task scoring D > 50 with C < 70 should prompt the user for more detail before going dark, ensuring medium-to-high difficulty tasks have a sufficient explanation on record.
  Joseph: In /pathfinder-premission, add a second clarification questions threshold: D > 50 with C < 70. Likely, we'll tweak these numbers, but for any reasonably complex task, I'd like to set a fairly high requirement for having a detailed explanation.
  Context: not specified

- [ ] **#34** Support staged multi-premission briefings and --auto chain execution
  [D:65 C:55] Two linked features: (1) namespace all premission artifacts (MISSION-PERMISSIONS.json, PROJECT-FOUNDATION.md, MISSION-LOG, etc.) with mission numbers so multiple premissions can be staged concurrently without collisions -- requires analysis of exactly which files conflict before implementation; (2) add a --auto mode to pathfinder-mission-team that, after completing a mission, auto-discovers the next staged premission briefing and continues until all queued missions are exhausted.
  Joseph: Assuming /compacting doesn't actually interfere with /pathfinder-mission-team (we'll find out 2-3 missions after #32 is completed), it's actually possible we could have an incredibly long series of tasks assigned without issue. With this in mind, I'd like to be able to do two things: I'd like to be able to stage multiple /pathfinder-premission's without causing conflicts. This probably requires mission-numbered PERMISSIONS.json (is that true?), PROJECT-FOUNDATION, etc. Please confirm where the conflicts would be before this task gets to /pathfinder-mission-team. And second, I'd like something like "/pathfinder-mission-team --auto", where the mission team can pick up one of the premission "briefings" (collection of tasks, permissions, etc), complete it, and then move onto the NEXT mission automatically, repeating until all missions have been addressed.
  Context: not specified

  - [ ] **#34-2** Staged multi-premission briefings and --auto chain (pre-mission clarified)
    [D:65 C:60] Namespace premission artifacts by mission number for concurrent staging without conflicts; --auto chains missions by lowest number first; planning phase must enumerate conflicting files before implementation; #17 (pathfinder/ dir) must complete first.
    Joseph: Pre-mission clarifications captured: (1) planning phase should enumerate exactly which files conflict before implementation; (2) --auto discovers next mission by lowest mission number; (3) #17 (pathfinder/ directory) must be done first -- #34 namespacing assumes files already live in pathfinder/.
    Context: Removed from M6 during premission - requires /discuss-idea first before planning or implementation; too involved for autonomous mission without deeper design discussion.

- [x] **#35** Enforce full Prefect approval loop; no corner-cutting on nits *(implemented)*
  [D:25 C:68] Investigate whether pathfinder-mission-team skips or short-circuits the Prefect review cycle when only minor issues remain, and if so, enforce that implementation never begins until Prefect gives unqualified approval -- the team must always run another reviewer pass rather than waiving remaining issues, consistent with the "slowly and perfectly" mission goal.
  Joseph: In /pathfinder-mission-team, it looks like sometimes, plans are being implemented without the full Prefect approval, would you look into it? Is this just the team cutting corners when all that remains is minor or nits? The goal for this team is more "slowly and perfectly, regardless of effort", so if it is, I'd rather you go back to another round of reviewers instead.
  Context: not specified
  - [x] **#35-2** Log Prefect-skip findings in Task Observations if root cause is unexpected
    [D:10 C:52] If the investigation in #35 reveals the Prefect approval is being skipped for a reason other than corner-cutting on nits, record the findings and the unexpected behavior in the Task Observations section of the MISSION-LOG so it can be reviewed and addressed.
    Joseph: as an addition to 35, please include the results of your findings in the Task Observations at the end of the mission if there's something else going on here.
    Context: not specified

- [x] **#36** Use local Toronto time instead of UTC in pathfinder timestamps
  [D:10 C:78] Pathfinder MISSION-LOG timestamps are correctly formatted as ISO 8601 but use the UTC/Zulu offset (Z suffix); since a single machine runs this skill, switch to local Toronto time (America/Toronto, ET) for all timestamps so they match the user's clock without requiring mental UTC conversion.
  Joseph: The pathfinder skills appear to use ISO 8601 (correctly), but the time is set to Zulu time (T07:30:29Z). Given that there's only one computer running this skill on one project at a time, let's just use local time (Toronto) for easier user comprehension.
  Context: not specified

- [x] **#37** Fix priority direction and replace linear decay with X² cumulative reduction *(implemented)*
  [D:40 C:62] Confirm whether tasks start at the wrong priority floor (0 instead of 99), then overhaul the decay algorithm: reduce priority by X² on each consecutive failed attempt (1, 4, 9, 16, 25...) where X resets to 1 after a successful intervening task completes; dependent tasks must receive the same reduction in lockstep; minimum priority is 0.
  Joseph: In /pathfinder-mission-team, I suspect the priority works backwards, please confirm. I think that tasks get abandoned at 0, meaning all tasks should probably start at 99. And actually, the priority deprecation should probably be a cumulative score: Priority is reduced by X^2, where X is the number of consecutive attempts count. So after the first attempt, reduce priority by 1, then 4, 9, 16, 25 etc. But if another task is successfully completed in between (because the first task got bumped below it), that X is reset back to one. Make sure that tasks with dependencies all receive the reduction to keep them in sync. Minimum priority score is 0.
  Context: not specified

- [x] **#38** Mirror casualty entries to numbered MISSION-LOG permission denials section
  [D:20 C:70] Casualty (permission denial) events are written to MISSION-LOG-active.md but not copied to the permanent numbered MISSION-LOG-#-*.md file under its Permission Denials heading; both files should receive the entry so the archived mission record is complete.
  Joseph: In pathfinder-mission-team, I think casualties get reported to MISSION-active, but not into the MISSION-LOG-#-*.md under the permission denials heading, but they should be there too!
  Context: not specified

- [ ] **#39** Parallelize PM-5 batch question generation with subagents when task count > 4
  [D:35 C:72] In pathfinder-premission step PM-5, when there are more than 4 tasks, all question batches should be prepared simultaneously by parallel subagents upfront, then presented to the user sequentially in groups of 4 -- eliminating the idle wait between batches while the main instance processes the next group.
  Joseph: The /pathfinder-premission skill should probably use subagents to prepare the batches in PM-5, when there's more than 4 tasks. It seems like I answer a batch, then I wait for the main instance to start thinking about the next 4. I'd rather have you subagent all the tasks simultaneously first, but only present them to me in batches of 4. Ignore if you're already using subagents here.
  Context: not specified

- [ ] **#40** Require detailed MISSION-LOG justification whenever mission-team skips a premission-approved task
  [D:25 C:58] pathfinder-mission-team must never silently drop a task from the user's starting command; any skip requires a written justification entry in MISSION-LOG, and the team's default stance should be that premission-approved tasks are mandatory -- skipping is a last resort that demands explicit reasoning on record.
  Joseph: When /pathfinder-mission-team skips any task listed in the user's starting command while building its initial task, it MUST justify this in the MISSION-LOG, in detail. As a general rule, the /pathfinder-mission-team should NEVER be skipping tasks, especially if they've gone through /pathfinder-premission! Their whole deal is to always get the job done.
  Context: not specified

- [ ] **#41** Confirm and fix mission-team task execution order to respect premission priority ranking
  [D:25 C:45] pathfinder-mission-team may be processing tasks in an arbitrary order rather than following the priority sequence established during premission; investigate whether the priority list from MISSION-PERMISSIONS.json is read and honoured at MT-1 initialization, then fix or clarify.
  Joseph: I think /pathfinder-mission-task might not be respecting the task priority order set out by the /pathfinder-premission. Confirm and fix, or clarify.
  Context: not specified

- [ ] **#42** Rename PROJECT-FOUNDATION to MISSION-#-BRIEF and add task priority order to it in both pathfinder skills
  [D:35 C:55] PROJECT-FOUNDATION.md should be renamed to MISSION-#-BRIEF.md (mission-numbered) in both pathfinder-premission and pathfinder-mission-team, and premission should write the approved task priority order into this file so mission-team can reference it during execution.
  Joseph: /pathfinder-premission should probably include its task priority order in the PROJECT-FOUNDATION, which we should now be naming the MISSION-#-BRIEF (in both pathfinder skills).
  Context: not specified

- [ ] **#43** Drop UTC offset from pathfinder timestamps, output bare local datetime
  [D:10 C:60] Change pathfinder skill timestamp format from `2026-03-25T15:30:00-0400` to `2026-03-25T15:30:00` by stripping the offset suffix -- single user, single machine, offset is noise.
  Joseph: On pathfinder skills, the current date-time format is 2026-03-25T15:30:00-0400. Let's omit the UTC offset (the -0400), I'm a single user working on a single machine and don't expect timezones to be important ever. (Possibly for that ONE hour per year where daylight savings skips backwards, but that's an acceptable risk for reducing noise for the rest of the year.)
  Context: not specified

- [ ] **#45** Log reviewer and prefect pass counts per task in MISSION-LOG for planning effort correlation
  [D:30 C:55] After each task and sub-task completes, record how many Reviewer and Prefect passes were consumed in the MISSION-LOG entry; this data enables correlation of D/C scores against actual planning effort and could eventually drive a "pause and closer look" alert when pass counts exceed score-based expectations.
  Joseph: In a pathfinder-mission-team, as each task/sub-task gets recorded to the Log, please also record how many Reviews & Prefects were used. I'd like to whether D and C scores could be used to estimate the effort involved in planning a given task (which could also help flag a "pause & closer look" if more reviews / prefects are being used than expected).
  Context: not specified

- [ ] **#46** Enforce sub-task log entry before marking any task complete in mission-team
  [D:25 C:55] The mission-team skill must require a minimal sub-task log entry (Status, Implementation, Timestamp) before a task can be marked Complete in the Task Status table; the drift checker or a Prefect-style check should flag missing entries as a blocking issue rather than letting them pass silently.
  Claude: Enforce sub-task log entry for every completed task — mission-team must write at least a minimal log entry (Status, Implementation, Timestamp) before marking a task Complete; missing entries should be flagged by the drift checker or Prefect rather than silently accepted
  Context: Reviewing MISSION-LOG-5 post-mission; six tasks were marked Complete with no sub-task log entries, making post-mission auditing unreliable
  - [ ] **#46-2** M5 post-mortem corroboration: six tasks marked Complete with no sub-task log entries
    [D:55 C:55] Six tasks were marked Complete with no corresponding sub-task log entries, making post-mission auditing unreliable. Update MT-3c/MT-3d to require a minimal log entry (Status, Implementation summary, Timestamp) before completion, with Prefect rejecting completions that lack log entries.
    Claude: C) **Process issue**: Six tasks (#16, #15, #18, #26, #28, #31) were marked Complete in the task table with no corresponding sub-task log entries. Without log entries there is no verifiable record of what was changed, which edge cases were addressed, or whether the implementation satisfies test criteria defined in PROJECT-FOUNDATION.md. This is a recurring pattern that makes post-mission auditing unreliable. **Suggested fix**: Update the MT-3c or MT-3d step in the mission skill to enforce that a sub-task log entry is written (at minimum: Status, Implementation summary, Timestamp) before a task is marked Complete. The Prefect pass should reject a task completion that has no corresponding log entry.
    Context: Mission Post-Mortem entry C from SUCCESSFUL-MISSION-LOG-5-pathfinder-skill-overhaul.md -- directly observed during M5

- [ ] **#47** Use letters (A, B, C...) for post-mortem entries in mission log
  [D:15 C:80] Mission Post-Mortem section entries in pathfinder mission logs should be labeled A), B), C)... rather than numbered, eliminating visual ambiguity with task numbers (#N format) when reviewing or submitting post-mortem items as add-task entries.
  Joseph-Raw: In pathfinder missions, post-mortem entries should be lettered instead of numbered (A) Process issue:, B) Process issue:, etc.) to avoid conflict with task numbers
  Context: not specified

- [ ] **#48** Suppress diff windows globally when the auto-approve permission hook is active
  [D:25 C:45] The permission auto-approve hook approves file changes before the user can interact with a diff, leaving empty/stale diff windows behind. Locate where diff-view behavior is configured (settings.json, CLAUDE.md, or IDE settings) and disable diffs for the duration the hook is active.
  Joseph-Raw: I think we should not use diffs while we have that hook running lol, they'll nearly always misfire if you're going to change it anyways. /add-task ? That might touch on CLAUDE.md, I'm not sure where your instructions about using diffs live.
  Context: not specified

- [ ] **#49** Fix MT-1 false-positives from generic rationale entries causing tasks to be flagged as skipped
  [D:45 C:52] The MT-1 coverage check requires each task ID to appear explicitly in an approved_actions rationale, but premission generic entries cover multiple tasks without citing them by ID, causing false skip-flags. The fix is either to enforce per-task ID citations in the premission skill or to loosen MT-1 to accept wildcard file coverage as sufficient for all tasks touching that file.
  Claude: A) **Process issue**: The MT-1 validation rule requires each task ID to appear as an explicit `#N` token in at least one `approved_actions.rationale` field, but the premission used generic entries ("core mission skill - updated by most tasks this mission") that covered many tasks without citing them by ID. This caused 9 of 17 tasks to be flagged as skipped at mission start, requiring a user intervention to re-add them before work could proceed. **Suggested fix**: Update the premission skill (pathfinder-premission/SKILL.md) to either (a) require per-task rationale citations when a task touches a shared artifact, or (b) update the MT-1 validation rule to treat a wildcard write/edit entry covering a shared file (e.g., SKILL.md) as sufficient coverage for all tasks that modify that file. This would eliminate false-skip cycles without changing the security intent of the coverage check.
  Context: Mission Post-Mortem entry A from SUCCESSFUL-MISSION-LOG-5-pathfinder-skill-overhaul.md -- directly observed during M5


- [ ] **#51** Add Agent attribution field to sub-task log entries to verify delegation constraints
  [D:62 C:48] Task #29 requires subagent delegation for context-heavy writes but the sub-task log has no Agent field, making silent violations undetectable. Add an optional `Agent: subagent|main` field to sub-task log entries and a Prefect verification step for any task with a delegation constraint.
  Claude: D) **Process issue**: Task #29 specifies that Task Observations and Mission Post-Mortem must be written via subagents to avoid exhausting main-instance context, but the sub-task log has no entry for #29, making it impossible to confirm whether the SKILL.md instruction correctly delegates these writes or performs them inline. The functional output (sections written) may appear correct while the constraint (subagent delegation) is silently violated. **Suggested fix**: Add a verification step to the Prefect pass for task #29 (and any task with a delegation constraint) that checks not just whether the output exists, but whether the log entry records which agent wrote it (main instance vs. spawned subagent). The sub-task log format could include an optional `Agent: subagent | main` field.
  Context: Mission Post-Mortem entry D from SUCCESSFUL-MISSION-LOG-5-pathfinder-skill-overhaul.md -- directly observed during M5

- [ ] **#52** Require post-edit re-read validation in sub-task logs for any SKILL.md modification
  [D:62 C:52] When a mission sub-task edits a SKILL.md file, the sub-task log must record an explicit re-read and structural validation step confirming the edit is syntactically sound. The Prefect pass should treat absence of this confirmation as a blocking issue to prevent silent corruption of downstream sub-tasks.
  Claude: E) **Process issue**: Multiple tasks that edit pathfinder-mission-team/SKILL.md were bundled into a single mission with no integration tests between sub-tasks. Because SKILL.md is an executable skill read by the Claude runtime, a mid-mission edit that introduces a syntax or logic error in the skill could silently corrupt later sub-tasks that depend on the same skill. No sub-task log entry for any of these tasks records a validation step (e.g., re-reading the modified SKILL.md to confirm structural integrity). **Suggested fix**: For any sub-task that edits a SKILL.md file, require the sub-task log to include a validation step confirming the file was re-read after editing and the modified section is syntactically consistent with surrounding content. The Prefect pass should treat absence of this confirmation as a blocking issue.
  Context: Mission Post-Mortem entry E from SUCCESSFUL-MISSION-LOG-5-pathfinder-skill-overhaul.md -- directly observed during M5

- [ ] **#53** Add Min/D stat to mission log and duration estimates to premission and mission-team
  [D:25 C:58] Append a computed `Min/D:` field after `Duration:` in ## Mission Complete. In /pathfinder-premission (after the 140/200 D check) and /pathfinder-mission-team (near start), display an estimated duration using total D * 2.3 min as the current baseline rate, enabling future calibration as more mission data accumulates.
  Joseph-Raw: In a pathfinder mission log, in the ## Mission Complete section after Duration:, please add minutes per difficulty. I'd like to be able to eventually start estimating the future mission durations based on difficulty. Also, add a mission duration estimate to both /pathfinder-premission (after the 140/200 D check) and /pathfinder-mission-team (near the very start). Current data suggests 2.3min/D, so use that for now.
  Context: not specified
  - [ ] **#53-2** Correct duration estimate formula to D×0.43 min, validated against M5 log data
    [D:10 C:55] The original formula uses 2.3 min/D but the correct rate inverts to D×0.43; the implementation should verify this against M5 actual duration and difficulty before the value is baked in.
    Joseph-Raw: I got the math backwards for that min/D vs D/min in #53. Please make sure the math makes sense for estimates, test against the M-5 log data. I'm pretty sure it should be D * 0.43, not 2.3.
    Context: not specified

- [ ] **#54** Add Min/C and Min/U stats to Mission Complete section alongside Min/D
  [D:20 C:68] Extend the ## Mission Complete section in MISSION-LOG to include two new computed fields placed near Min/D: `Min/C` (mission duration divided by total C score) and `Min/U` (mission duration divided by U, where U = (number_of_tasks x 100) - sum(C)), enabling future correlation of clarity and uncertainty against actual mission duration. Example: 3 tasks with C:10,20,30 gives U = 300-60 = 240.
  Joseph-Raw: In pathfinder missions, it seems that D scores may be useful in predicting mission duration estimates. I'd like to use the C scores, but I suspect we should also track U (100-C). The Uncertainty score is the amount of "Missing" C. Please add the min/U (does uncertainty have an effect on duration) and min/C (does certainty have an effect on duration) in the same Mission Complete section near the min/D result too.
  Context: U formula clarified during M6 premission: U = (number_of_tasks x 100) - sum(C), not a simple 100-C per task.

- [ ] **#55** Track premission duration and show estimate before committing to session
  [D:40 C:55] Add start/end timestamps to pathfinder-premission itself and compute a pre-session duration estimate (using D/C/U metrics) displayed before the user commits, so they can trim the task list when the estimated premission time exceeds available time.
  Joseph-Raw: I'd like to start tracking how long the pathfinder premission takes too. I assume this should go in the MISSION BRIEF, or recommend a better place. It should include the calculations we're tracking for missions too, premission start process / end process, estimates for duration vs D, C, U. For especially large premissions, I'd like to not be starting one that'll take 30 minutes to complete properly if I only have 10 minutes lol. I'd rather chop the list down, get the premission done, then send the pathfinder team on those priority items before I leave.
  Context: not specified

- [ ] **#56** Log command usage per mission; add Default Permissions baseline pulled into each premission
  [D:45 C:55] Mission-team records which approved commands were actually invoked vs. unused during a run; a persistent DEFAULT-PERMISSIONS file is read by premission as a starting baseline so per-mission manifests extend rather than replace accumulated history, preventing nuanced per-permission rationale from being silently overwritten.
  Joseph-Raw: Can the pathfinder mission note which commands did and didn't get used? I'd like to set up a "Default Permissions" file, that the pathfinder's can automatically pull from for each individual mission reliably (so that we don't accidentally overwrite a permission with a nuanced history).
  Context: not specified
  - [ ] **#56-2** Track per-mission command hit counts in DEFAULT-PERMISSIONS; add post-mortem recommendation section
    [D:30 C:58] Extend DEFAULT-PERMISSIONS to record how many missions used each command (binary used/unused per mission, not individual call counts), surfacing high-frequency entries as essential inclusions; add a dedicated post-mortem section for recommending commands be promoted to DEFAULT-PERMISSIONS with written justification.
    Joseph-Raw: It'd be nice if the DEFAULT-PERMISSIONS could also track the number of missions that ended up using that command (more than 0 times is sufficient, no need to count individual uses of each command). Over time, we'll see which commands are VERY IMPORTANT TO INCLUDE. And the MISSION post-mortem should include a new section about recommending a command be added to the default-permissions, and justify what issues it could prevent.
    Context: not specified

- [ ] **#57** Fix M6 Start-Time recorded ~4 hours ahead of actual local time
  [D:20 C:45] MISSION-LOG-6 shows Start-Time T19:06 but the user reports it is ~15:12 and the mission just started; the timestamp is ~4 hours ahead of actual. Likely a timezone offset being applied incorrectly (double-counted or wrong sign) in the pathfinder Start-Time recording step, introduced after task #36 switched timestamps to Toronto local time.
  Joseph-Raw: Pretty sure M6 Start-Time is wrong. It says T19:06, but it's 3:12PM right now. It only started a few minutes ago, not... 4 hours in the future?? I'm guessing all times will be off for this mission, but I'm not interupting it for just this.
  Context: not specified

- [ ] **#58** Resolve collision between TASKS.md sub-entry format (#N-2) and pathfinder sub-task nomenclature
  [D:35 C:40] TASKS.md uses #N-2 / #N-3 suffixes for supplementary context entries under a parent task, but pathfinder-mission-team uses its own sub-task numbering internally. When the mission team reads TASKS.md and encounters an entry like #53-2, it likely misinterprets it as a prior-run decomposed sub-task rather than a clarification/context record for #53, causing incorrect task-list parsing or re-queue behavior.
  Joseph-Raw: I'm pretty sure pathfinder-mission-team doesn't handle entries in TASKS like #53-2 very well. I suspect it conflicts with their subtask nomenclature, but in TASKS it's supposed to be additional information and context on #53
  Context: not specified

---
