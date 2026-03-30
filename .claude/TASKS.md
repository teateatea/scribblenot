# TASKS

## scribblenot

_Tasks for active development. Feature backlog lives in TODOS.md._

- [x] **#44** Add /add-tasks as a forwarding alias to /add-task without duplicating the skill
  [D:15 C:60] Create a minimal /add-tasks skill entry that immediately delegates to /add-task so both trigger words work; the alias contains no logic of its own, avoiding a maintenance burden when /add-task changes.
  Joseph: The /add-task skill should also trigger on /add-tasks. It's too easy for me to add that s when I'm thining about adding several, and it might as well work correctly. Don't just copy the /add-task skill though, I don't want to have to maintain identical skills.
  Context: not specified

- [ ] **#21** Add persistent group-jump hotkeys in map column (e.g. Q=Intake, W=Subjective, F=Treatment)
  [D:62 C:55] When focus is on the map column, a fixed set of keys should always jump to the first section of each group regardless of current cursor position (e.g. F while on Post-Tx jumps to Tx Mods). These group-reserved keys must be excluded from the section-hint pool so no section hint is ever assigned a character that conflicts with a group jump key.
  Joseph: I'd like the hints for group headings to always be available when in the map column, regardless of what section the cursor is on. So Q INTAKE, W SUBJECTIVE, F TREATMENT, etc, the Q W F etc will always let me jump directly to that group. If I'm not currently in that group, put the cursor at the first section (if I'm over Post-Tx, and I hit F for TREATMENT group, it'll jump me to Tx Mods. Make sure the distribution correctly accounts for this, the section hints won't be able to ever use those group hint characters.
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

- [x] **#1** Remove dead code warning for unused `current_value` on `HeaderState`
  [D:10 C:55] Delete or use the `pub fn current_value()` method in `src/sections/header.rs` that triggers a dead_code warning on every `cargo build`/`cargo run`.
  Joseph: about that dead code clean up, I don't like that it pops up when I cargo run.
  Context: not specified
