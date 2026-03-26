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

- [ ] **#34** Support staged multi-premission briefings and --auto chain execution
  [D:65 C:55] Two linked features: (1) namespace all premission artifacts (MISSION-PERMISSIONS.json, PROJECT-FOUNDATION.md, MISSION-LOG, etc.) with mission numbers so multiple premissions can be staged concurrently without collisions -- requires analysis of exactly which files conflict before implementation; (2) add a --auto mode to pathfinder-mission-team that, after completing a mission, auto-discovers the next staged premission briefing and continues until all queued missions are exhausted.
  Joseph: Assuming /compacting doesn't actually interfere with /pathfinder-mission-team (we'll find out 2-3 missions after #32 is completed), it's actually possible we could have an incredibly long series of tasks assigned without issue. With this in mind, I'd like to be able to do two things: I'd like to be able to stage multiple /pathfinder-premission's without causing conflicts. This probably requires mission-numbered PERMISSIONS.json (is that true?), PROJECT-FOUNDATION, etc. Please confirm where the conflicts would be before this task gets to /pathfinder-mission-team. And second, I'd like something like "/pathfinder-mission-team --auto", where the mission team can pick up one of the premission "briefings" (collection of tasks, permissions, etc), complete it, and then move onto the NEXT mission automatically, repeating until all missions have been addressed.
  Context: not specified

  - [ ] **#34-2** Staged multi-premission briefings and --auto chain (pre-mission clarified)
    [D:65 C:60] Namespace premission artifacts by mission number for concurrent staging without conflicts; --auto chains missions by lowest number first; planning phase must enumerate conflicting files before implementation; #17 (pathfinder/ dir) must complete first.
    Joseph: Pre-mission clarifications captured: (1) planning phase should enumerate exactly which files conflict before implementation; (2) --auto discovers next mission by lowest mission number; (3) #17 (pathfinder/ directory) must be done first -- #34 namespacing assumes files already live in pathfinder/.
    Context: Removed from M6 during premission - requires /discuss-idea first before planning or implementation; too involved for autonomous mission without deeper design discussion.



- [ ] **#57** Fix M6 Start-Time recorded ~4 hours ahead of actual local time
  [D:20 C:45] MISSION-LOG-6 shows Start-Time T19:06 but the user reports it is ~15:12 and the mission just started; the timestamp is ~4 hours ahead of actual. Likely a timezone offset being applied incorrectly (double-counted or wrong sign) in the pathfinder Start-Time recording step, introduced after task #36 switched timestamps to Toronto local time.
  Joseph-Raw: Pretty sure M6 Start-Time is wrong. It says T19:06, but it's 3:12PM right now. It only started a few minutes ago, not... 4 hours in the future?? I'm guessing all times will be off for this mission, but I'm not interupting it for just this.
  Context: not specified

- [ ] **#59** Mirror PreCompact hook entries to the numbered MISSION-LOG file, not just MISSION-LOG-active
  [D:15 C:60] The PreCompact hook (added in #32) logs compact events to MISSION-LOG-active.md but not to the permanent numbered MISSION-LOG-N-*.md file. Compact events should appear in the human-readable mission log so post-mission review shows exactly when compaction occurred without needing to cross-reference a separate file.
  Joseph-Raw: On M6, at the 2 hour mark, I checked the logs and the active instance. I believe the precompact hook is firing, and logging into MISSION-LOG-active, but I'd like an entry in the human-readable MISSION-LOG-6* as well!
  Context: not specified

- [ ] **#58** Resolve collision between TASKS.md sub-entry format (#N-2) and pathfinder sub-task nomenclature
  [D:35 C:40] TASKS.md uses #N-2 / #N-3 suffixes for supplementary context entries under a parent task, but pathfinder-mission-team uses its own sub-task numbering internally. When the mission team reads TASKS.md and encounters an entry like #53-2, it likely misinterprets it as a prior-run decomposed sub-task rather than a clarification/context record for #53, causing incorrect task-list parsing or re-queue behavior.
  Joseph-Raw: I'm pretty sure pathfinder-mission-team doesn't handle entries in TASKS like #53-2 very well. I suspect it conflicts with their subtask nomenclature, but in TASKS it's supposed to be additional information and context on #53
  Context: not specified

- [ ] **#60** Add Initial and Current Estimated Completion Time fields to MISSION-LOG Task Status
  [D:30 C:55] Under ## Task Status in MISSION-LOG, add two wall-clock ETA fields: "Initial Estimated Completion Time: HH:mm (Started at HH:mm)" computed once at mission start from total D * 0.43 min/D rate, and "Current Estimated Completion Time: HH:mm (Updated at HH:mm)" recomputed from remaining D each time a new task begins (not sub-tasks).
  Joseph-Raw: In the MISSION-LOG-#, under ## Task Status, let's add some information so I can check in mid-mission:
  - "Initial Estimated Completion Time: HH:mm (Started at HH:mm)" (based on the total difficulty; I'd like a time not a duration)
  - "Current Estimated Completion Time: HH:mm (Updated at HH:mm)" (based on the current remaining difficulty)
  - These are calculated by whatever our min/D rate is, right now I think it's D * 0.43? But we only need to update these numbers every time we start a new task, not for sub-tasks.
  Context: not specified

- [ ] **#61** Add remaining count to Difficulty field in MISSION-LOG mission section
  [D:15 C:52] Update the Difficulty display format in MISSION-LOG files to append remaining difficulty in parentheses, e.g. "Difficulty: 3/10 (7 remaining)". Small formatting change to an existing log field updated by pathfinder-mission-team.
  Joseph-Raw: In the MISSION-LOG-#, under ## Mission, change "Difficulty: {total completed}/{mission total}" to "Difficulty: {total completed}/{mission total} ({total remaining} remaining)"
  Context: not specified

- [ ] **#62** Omit (P:99) priority annotation from Tasks list in MISSION-LOG ## Mission section
  [D:15 C:42] Tasks listed in the ## Mission Tasks field of MISSION-LOG should drop the "(P:N)" priority suffix so the list reads as plain task IDs (e.g. #19, #43, #47) without redundant annotation noise.
  Joseph-Raw: In the MISSION-LOG-#, under ## Mission, Tasks: can omit (P:99) on each task. #19, #43, #47, etc is fine.
  Context: not specified

- [ ] **#64** Add multi-file pattern search to Implementer prompt for repeated-pattern changes
  [D:25 C:75] Add a mandatory step to the Implementer prompt (MT-3c) requiring a grep across the full project including hooks/ for the exact pattern being changed before marking implementation complete, preventing single-file fixes that leave sibling files broken.
  Claude: "Add multi-file pattern search step to Implementer prompt for tasks that modify repeated patterns" -- Before marking implementation complete, the Implementer should grep the full project (including hooks/) for the exact pattern being changed and update all matching locations, preventing single-file fixes that leave sibling files broken.
  Context: Mission 6 post-mortem (pathfinder/SUCCESSFUL-MISSION-LOG-6-skill-log-quality.md) - Task #43 attempt 1 failed because the subagent updated SKILL.md but missed the identical pattern in pre-compact-mission-log.sh.

- [ ] **#70** Audit why diff windows still open during M7 despite two completed suppress-diff tasks
  [D:35 C:62] Tasks #15 and #48 were both marked complete and claimed to suppress diff windows during pathfinder missions, but Mission 7 still opens diffs. This task asks for an audit of what was actually implemented and why it is not working, with findings reported in the M7 mission log.
  Joseph-Raw: We have a handful of completed tasks in our CLOSED_TASKS.md, suggesting that diffs should not be opening anymore during pathfinder missions. M7 has just started, and it definitely still opens diffs. Please audit and report findings in mission log.
  Context: not specified

- [ ] **#71** Add cumulative review-round counter to Reviewer and Prefect entries in sub-task log
  [D:45 C:58] Each sub-task log entry should track which review round a Reviewer or Prefect approval came from (e.g., 'Prefect-1 [R3]'), making it clear whether approval was first-pass or after multiple retries. The task also requests investigation into whether Prefect reports are still being removed, and if so, stopping that behavior.
  Joseph-Raw: In pathfinder missions, I think Reviewers and Prefects should have a cumulative count number, per sub-task. There's a big difference between Prefect-1 approving, and Prefect-1 approving on the third round of reviews. This also means we can stop removing Prefect reports, which I believe is still happening. Please review in detail?
  Context: not specified

- [ ] **#72** Investigate planner/reviewer subagent as source of errant diff windows
  [D:30 C:35] Follow-on hypothesis to #70 -- user suspects the Planner or Reviewer subagent spawned during pathfinder missions is the process opening diff windows; confirm the hypothesis and implement a targeted fix in the offending subagent.
  Joseph-Raw: It seems to be the planner or reviewer opening up the errant diffs?
  Context: not specified
  - [ ] **#72-2** Reviewer opens diffs intermittently -- behavior not consistent across all tasks
    [D:25 C:28] Refines #72 -- the Reviewer subagent is the likely culprit for opening diffs, but the behavior is task-conditional, suggesting a specific code path or trigger in the Reviewer logic rather than a blanket misconfiguration.
    Joseph-Raw: Yeah, the reviewer opening diffs when it's not supposed to be, but it seems like it might not be for all tasks? Very strange.
    Context: not specified
