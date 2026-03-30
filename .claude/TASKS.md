# TASKS

## scribblenot

_Tasks for active development. Feature backlog lives in TODOS.md._

- [x] **#44** Add /add-tasks as a forwarding alias to /add-task without duplicating the skill
  [D:15 C:60] Create a minimal /add-tasks skill entry that immediately delegates to /add-task so both trigger words work; the alias contains no logic of its own, avoiding a maintenance burden when /add-task changes.
  Joseph: The /add-task skill should also trigger on /add-tasks. It's too easy for me to add that s when I'm thining about adding several, and it might as well work correctly. Don't just copy the /add-task skill though, I don't want to have to maintain identical skills.
  Context: not specified

---

## Code Quality

- [x] **#1** Remove dead code warning for unused `current_value` on `HeaderState`
  [D:10 C:55] Delete or use the `pub fn current_value()` method in `src/sections/header.rs` that triggers a dead_code warning on every `cargo build`/`cargo run`.
  Joseph: about that dead code clean up, I don't like that it pops up when I cargo run.
  Context: not specified
