# Project Tests

<!-- Per-task acceptance test checklists. Sections matching ## Task # are
     automatically migrated to the next three MISSION-*-TESTS.md files at
     mission completion and then removed from this file. Add new entries here
     during implementation so they get forwarded to upcoming missions. -->

## Task #67 - Data Hotspot Split Slice 0 Scaffold

### Acceptance checklist
- [ ] New helper files exist with explicit names: `src/data_model.rs`, `src/data_hints.rs`, `src/data_source.rs`, `src/data_load.rs`, `src/data_runtime.rs`, `src/data_validate.rs`
- [ ] `src/main.rs` declares the new helper modules cleanly
- [ ] `src/data.rs` remains the public facade for callers using `crate::data::*`
- [ ] No new `src/data/mod.rs` facade is introduced
- [ ] `cargo check --quiet` passes

### Nice-to-have verification
- [ ] `cargo test --quiet` passes
- [ ] Diff shows no behavior change outside module scaffolding
