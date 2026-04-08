# Project Tests

## Task #70: Propose a plan to implement the canonical 6-level YAML data hierarchy
- [ ] A plan file exists in operations/plans/ that defines Rust structs for all six hierarchy levels (Template, Group, Section, Field, List, Item) with the correct mandatory/optional fields per DISCUSSION-yaml-data-hierarchy.md
- [ ] The plan describes a YAML loader that scans data/ for all .yml files and resolves cross-file ID references without a root config
- [ ] The plan includes a concrete migration strategy for existing data files (sections.yml, tx_regions.yml, tx_mods.yml, objective_findings.yml, remedial.yml, infection_control.yml)
- [ ] The plan specifies load-time validation for missing IDs, duplicate ID+type combinations, and circular references
- [ ] The plan explicitly excludes Boxes/UI layer, multi-template switching, date_prefix, and skip-level referencing as out of scope
