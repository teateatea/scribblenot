## Task

#70 - Implement canonical 6-level YAML data hierarchy

## Context

Sub-task 1 added the hierarchy structs (TypeTag, HierarchyItem, HierarchyList, HierarchyField, HierarchySection, HierarchyGroup, HierarchyTemplate, BoilerplateEntry, HierarchyFile) and all 10 struct tests pass. Sub-task 2 implements the directory-scanning loader that merges multiple YAML files into one HierarchyFile and validates the merged result. The loader is the gatekeeper for all later sub-tasks: the shim (ST3) and AppData::load wiring (ST4) both depend on it. Six TDD tests in the hierarchy_loader_tests module at src/data.rs:2462 define the exact contract and currently fail because load_hierarchy_dir does not exist.

## Approach

Add `pub fn load_hierarchy_dir(dir: &std::path::Path) -> Result<HierarchyFile, String>` in src/data.rs, directly after the existing load_data_dir function. The function scans all *.yml files in the directory (skipping keybindings.yml and config.yml), parses each as HierarchyFile via serde_yaml, merges the top-level Option<Vec<>> fields into a single HierarchyFile, then runs four validation passes in order: (1) template cardinality, (2) typed ID uniqueness with a HashSet<(TypeTag, String)>, (3) boilerplate ID uniqueness with a HashSet<String>, (4) cross-reference validation and DFS cycle detection using the same algorithm as the existing load_data_dir DFS at lines 756-782. No other files are modified in this sub-task.

## Critical Files

- `src/data.rs` - insert load_hierarchy_dir after load_data_dir (which ends at line 867); the six failing tests are in hierarchy_loader_tests module at lines 2462-2696

## Reuse

- Existing DFS pattern from load_data_dir lines 756-782: visited/in_stack HashSet<String> (keyed on id string instead of usize index), same pre/post-order logic
- Existing HashSet and HashMap imports already in scope at data.rs:3-4
- Existing fs and Path imports already in scope at data.rs:5-7
- serde_yaml::from_str already used by load_data_dir at line 714

## Steps

1. Insert the following function in src/data.rs immediately after the closing brace of load_data_dir (after line 867):

```rust
+ pub fn load_hierarchy_dir(dir: &std::path::Path) -> Result<HierarchyFile, String> {
+     // --- Phase 1: scan and parse ---
+     let entries = fs::read_dir(dir)
+         .map_err(|e| format!("failed to read directory {:?}: {}", dir, e))?;
+
+     let mut merged = HierarchyFile {
+         template: None,
+         groups: None,
+         sections: None,
+         fields: None,
+         lists: None,
+         items: None,
+         boilerplate: Vec::new(),
+     };
+     let mut template_count = 0usize;
+
+     for entry in entries {
+         let entry = entry.map_err(|e| format!("directory entry error: {}", e))?;
+         let file_path = entry.path();
+         let file_name = file_path
+             .file_name()
+             .and_then(|n| n.to_str())
+             .unwrap_or("");
+         if !file_name.ends_with(".yml") {
+             continue;
+         }
+         if file_name == "keybindings.yml" || file_name == "config.yml" {
+             continue;
+         }
+         let content = fs::read_to_string(&file_path)
+             .map_err(|e| format!("failed to read {:?}: {}", file_path, e))?;
+         let hf: HierarchyFile = serde_yaml::from_str(&content)
+             .map_err(|e| format!("parse error in {:?}: {}", file_path, e))?;
+
+         // Merge template (count occurrences for cardinality check)
+         if hf.template.is_some() {
+             template_count += 1;
+             if merged.template.is_none() {
+                 merged.template = hf.template;
+             }
+         }
+         // Merge Option<Vec<>> fields
+         if let Some(v) = hf.groups {
+             merged.groups.get_or_insert_with(Vec::new).extend(v);
+         }
+         if let Some(v) = hf.sections {
+             merged.sections.get_or_insert_with(Vec::new).extend(v);
+         }
+         if let Some(v) = hf.fields {
+             merged.fields.get_or_insert_with(Vec::new).extend(v);
+         }
+         if let Some(v) = hf.lists {
+             merged.lists.get_or_insert_with(Vec::new).extend(v);
+         }
+         if let Some(v) = hf.items {
+             merged.items.get_or_insert_with(Vec::new).extend(v);
+         }
+         merged.boilerplate.extend(hf.boilerplate);
+     }
+
+     // --- Phase 2: template cardinality ---
+     match template_count {
+         0 => return Err("no template defined: exactly 1 template is required across all hierarchy files".to_string()),
+         1 => {}
+         n => return Err(format!("multiple templates defined: found {}, expected exactly 1", n)),
+     }
+
+     // --- Phase 3: typed ID uniqueness ---
+     let mut seen: HashSet<(TypeTag, String)> = HashSet::new();
+     for g in merged.groups.as_deref().unwrap_or(&[]) {
+         let key = (TypeTag::Group, g.id.clone());
+         if !seen.insert(key) {
+             return Err(format!("duplicate group id: {}", g.id));
+         }
+     }
+     for s in merged.sections.as_deref().unwrap_or(&[]) {
+         let key = (TypeTag::Section, s.id.clone());
+         if !seen.insert(key) {
+             return Err(format!("duplicate section id: {}", s.id));
+         }
+     }
+     for f in merged.fields.as_deref().unwrap_or(&[]) {
+         let key = (TypeTag::Field, f.id.clone());
+         if !seen.insert(key) {
+             return Err(format!("duplicate field id: {}", f.id));
+         }
+     }
+     for l in merged.lists.as_deref().unwrap_or(&[]) {
+         let key = (TypeTag::List, l.id.clone());
+         if !seen.insert(key) {
+             return Err(format!("duplicate list id: {}", l.id));
+         }
+     }
+
+     // --- Phase 4: boilerplate ID uniqueness ---
+     let mut bp_seen: HashSet<String> = HashSet::new();
+     for bp in &merged.boilerplate {
+         if !bp_seen.insert(bp.id.clone()) {
+             return Err(format!("duplicate boilerplate id: {}", bp.id));
+         }
+     }
+
+     // --- Phase 5: cross-reference validation ---
+     // Build typed lookup sets for O(1) existence checks
+     let group_ids: HashSet<&str> = merged.groups.as_deref().unwrap_or(&[])
+         .iter().map(|g| g.id.as_str()).collect();
+     let section_ids: HashSet<&str> = merged.sections.as_deref().unwrap_or(&[])
+         .iter().map(|s| s.id.as_str()).collect();
+     let field_ids: HashSet<&str> = merged.fields.as_deref().unwrap_or(&[])
+         .iter().map(|f| f.id.as_str()).collect();
+     let list_ids: HashSet<&str> = merged.lists.as_deref().unwrap_or(&[])
+         .iter().map(|l| l.id.as_str()).collect();
+
+     // Template -> group refs
+     let template = merged.template.as_ref().unwrap(); // safe: cardinality already checked
+     for gref in &template.groups {
+         if !group_ids.contains(gref.as_str()) {
+             return Err(format!("unresolved template group ref: {}", gref));
+         }
+     }
+     // Group -> section refs
+     for g in merged.groups.as_deref().unwrap_or(&[]) {
+         for sref in &g.sections {
+             if !section_ids.contains(sref.as_str()) {
+                 return Err(format!("unresolved section ref '{}' in group '{}'", sref, g.id));
+             }
+         }
+     }
+     // Section -> field refs (fields: is Option<Vec<HierarchyField>>; fields are inline, not refs)
+     // Section -> list refs (lists: is Option<Vec<HierarchyList>>; lists are inline, not refs)
+     // Field -> list_id ref
+     for f in merged.fields.as_deref().unwrap_or(&[]) {
+         if let Some(ref lid) = f.list_id {
+             if !list_ids.contains(lid.as_str()) {
+                 return Err(format!("unresolved list_id ref '{}' in field '{}'", lid, f.id));
+             }
+         }
+     }
+
+     // --- Phase 6: DFS cycle detection over group->section->field->list refs ---
+     // Build adjacency: each node id -> Vec of child ids it references
+     // Only top-level cross-refs (group.sections, field.list_id) are ref edges.
+     // Inline fields/lists inside sections are not separate nodes in the ref graph.
+     fn dfs_hier(
+         node: &str,
+         adj: &HashMap<String, Vec<String>>,
+         visited: &mut HashSet<String>,
+         in_stack: &mut HashSet<String>,
+     ) -> Result<(), String> {
+         if in_stack.contains(node) {
+             return Err(format!("cycle detected at node id={}", node));
+         }
+         if visited.contains(node) {
+             return Ok(());
+         }
+         visited.insert(node.to_string());
+         in_stack.insert(node.to_string());
+         if let Some(children) = adj.get(node) {
+             for child in children {
+                 dfs_hier(child, adj, visited, in_stack)?;
+             }
+         }
+         in_stack.remove(node);
+         Ok(())
+     }
+
+     let mut adj: HashMap<String, Vec<String>> = HashMap::new();
+     for g in merged.groups.as_deref().unwrap_or(&[]) {
+         adj.entry(g.id.clone()).or_default().extend(g.sections.iter().cloned());
+     }
+     for f in merged.fields.as_deref().unwrap_or(&[]) {
+         if let Some(ref lid) = f.list_id {
+             adj.entry(f.id.clone()).or_default().push(lid.clone());
+         }
+     }
+
+     let mut visited: HashSet<String> = HashSet::new();
+     let mut in_stack: HashSet<String> = HashSet::new();
+     for g in merged.groups.as_deref().unwrap_or(&[]) {
+         dfs_hier(&g.id, &adj, &mut visited, &mut in_stack)?;
+     }
+     for f in merged.fields.as_deref().unwrap_or(&[]) {
+         dfs_hier(&f.id, &adj, &mut visited, &mut in_stack)?;
+     }
+
+     Ok(merged)
+ }
```

2. Run `cargo test hierarchy_loader_tests` and confirm all 6 tests pass (TEST-1 through TEST-6). Then run `cargo test` to confirm no regressions among the full 190-test suite.

## Verification

### Manual tests

- None required for this sub-task. All validation is automated.

### Automated tests

- `cargo test hierarchy_loader_tests` must print 6 tests all ok.
- `cargo test` must pass with zero failures (190+ tests).
- Specific coverage by test:
  - TEST-1: valid single-file with one template/group/section returns Ok; merged groups.len() == 1
  - TEST-2: zero templates returns Err containing "template"
  - TEST-3: two templates across two files returns Err containing "template"
  - TEST-4: duplicate group id across two files returns Err containing "grp1" or "duplicate"
  - TEST-5: group references nonexistent section id returns Err containing "sec_nonexistent" or "missing"/"not found"
  - TEST-6: duplicate boilerplate id across two files returns Err containing "bp1", "duplicate", or "boilerplate"

## Changelog

### Review - 2026-04-03
- #1 (nit): Replace `std::fs::read_dir` and `std::fs::read_to_string` with the already-imported `fs::` alias to match existing file style.

## Progress
- Step 1: Inserted load_hierarchy_dir function in src/data.rs after load_data_dir (after line 867), using fs:: alias per review nit #1
- Step 2: All 6 hierarchy_loader_tests pass; full suite 196/196 pass with zero regressions

## Implementation
Complete - 2026-04-03
