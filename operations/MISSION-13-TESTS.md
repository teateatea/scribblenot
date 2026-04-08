# Project Tests

## Task #70: Implement canonical 6-level YAML data hierarchy
- [ ] cargo test passes with all tests rewritten to hierarchy-format YAML and all new ST1/ST2 unit tests added
- [ ] load_hierarchy_dir against actual data/ directory returns Ok with groups in correct order (intake, subjective, treatment, objective, post_tx)
- [ ] SectionConfig from migrated objective_section has date_prefix == Some(true)
- [ ] tx_regions.yml parsed as HierarchyFile has back_lower_prone list with fascial_l4l5 item having default: Some(false)
- [ ] hierarchy_to_runtime produces block_select_data with key "tx_regions" containing at least one HierarchyList entry
- [ ] src/flat_file.rs is deleted and mod flat_file removed from src/main.rs
- [ ] No map_label: keys survive in any data/*.yml file (all renamed to nav_label:)
- [ ] All data YAML files (sections.yml, tx_regions.yml, boilerplate.yml, objective_findings.yml, remedial.yml, infection_control.yml) parse as HierarchyFile without error
- [ ] cargo build compiles cleanly with no warnings from new code
- [ ] Manual note output remains byte-for-byte identical to the pre-migration baseline
- [ ] Mission validation does not depend on the separate list-select add-entry persistence issue tracked in roadmap item #2
