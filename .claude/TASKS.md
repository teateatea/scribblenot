# TASKS

## scribblenot

_Tasks for active development. Feature backlog lives in TODOS.md._

- [ ] **#45** Refactor data format to flat, type-tagged YML blocks with ID-based cross-references *(implemented - sub-task 1)*
  [D:80 C:75]
  Claude: Full refactor of the data loading pipeline. Currently sections.yml is a single deeply-nested file. The new format splits concerns across optional separate files (boxes.yml, groups.yml, sections.yml, fields.yml, plus freeform options files) all living in data/. Each block is flat and carries a `type:` field. Parents reference children by flat ID list (e.g. `fields: [field_date]`). Option lists are referenced by name (e.g. `options: minutes_list`). Loader scans data/ and merges all yml files found. ID uniqueness is scoped per type — same ID is valid across different types. Missing IDs and duplicate ID+type combos are loud load-time errors. Circular references must be actively detected and errored. The id/label/output option shape is preserved. Runtime behavior must be identical — this is a data authoring and loading change only. Full spec in operations/plans/DISCUSSION-flat-yml-split.md.
  Context: /discuss-idea session on reconfiguring sections.yml data format

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
