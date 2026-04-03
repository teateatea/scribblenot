**Task**: #51 Move hard-coded section metadata into sections.yml

**Context**: ST51.3 removed the three hard-coded functions. ST51.4 replaces all remaining
`cfg.id == "..."` render position checks in render_note() with `cfg.note_render_slot` lookups.
This also eliminates the `known_ids = ["tx_mods"]` shim from the catch-all block.
The note output is identical before and after -- this is a pure behavioral refactor.

**Approach**: Nine targeted replacements in render_note() plus the catch-all block.
Any test that constructs a SectionConfig with `id: "header"` but `note_render_slot: None`
will need `note_render_slot: Some("header".to_string())` added.

**Critical Files**:
- `src/note.rs` lines 83, 155, 183, 203, 220, 237, 246, 255, 270, 280-282 (cfg.id checks)

**Replacements** (exact old → new):

1. Line 83: `cfg.section_type == "multi_field" && cfg.id == "header"`
   → `cfg.section_type == "multi_field" && cfg.note_render_slot.as_deref() == Some("header")`

2. Line 155: `cfg.id == "subjective_section"`
   → `cfg.note_render_slot.as_deref() == Some("subjective_section")`

3. Line 183: `cfg.id == "tx_mods"`
   → `cfg.note_render_slot.as_deref() == Some("tx_mods")`

4. Line 203: `cfg.id == "tx_regions"`
   → `cfg.note_render_slot.as_deref() == Some("tx_regions")`

5. Line 220: `cfg.id == "objective_section"`
   → `cfg.note_render_slot.as_deref() == Some("objective_section")`

6. Line 237: `cfg.id == "post_treatment"`
   → `cfg.note_render_slot.as_deref() == Some("post_treatment")`

7. Line 246: `cfg.id == "remedial_section"`
   → `cfg.note_render_slot.as_deref() == Some("remedial_section")`

8. Line 255: `cfg.id == "tx_plan"`
   → `cfg.note_render_slot.as_deref() == Some("tx_plan")`

9. Line 270: `cfg.id == "infection_control_section"`
   → `cfg.note_render_slot.as_deref() == Some("infection_control_section")`

10. Catch-all block (lines 279-292): replace both the outer `cfg.id != "header"` check
    AND the inner `known_ids = ["tx_mods"]` shim:

```diff
-        if cfg.section_type == "multi_field" && cfg.id != "header" {
-            let known_ids = ["tx_mods"];
-            if !known_ids.contains(&cfg.id.as_str()) {
-                if let SectionState::Header(hs) = state {
-                    if let Some(rendered) = render_multifield_section(cfg, hs, sticky_values, mode.clone()) {
-                        if !rendered.trim().is_empty() {
-                            parts.push(format!("\n\n\n#### {}\n{}", cfg.name.to_uppercase(), rendered));
-                        }
-                    }
-                }
-            }
-        }
+        if cfg.section_type == "multi_field"
+            && cfg.note_render_slot.as_deref() != Some("header")
+            && cfg.note_render_slot.as_deref() != Some("tx_mods")
+        {
+            if let SectionState::Header(hs) = state {
+                if let Some(rendered) = render_multifield_section(cfg, hs, sticky_values, mode.clone()) {
+                    if !rendered.trim().is_empty() {
+                        parts.push(format!("\n\n\n#### {}\n{}", cfg.name.to_uppercase(), rendered));
+                    }
+                }
+            }
+        }
```

**Test fixes**: Any test that creates a SectionConfig with `id: "header"` but no
`note_render_slot` set must add `note_render_slot: Some("header".to_string())`.
The make_section_config helpers in note.rs tests use `id:` field for routing --
these need `note_render_slot` added to match the new dispatch logic.

**Steps**:
1. Apply replacements 1-9 (individual cfg.id checks).
2. Apply replacement 10 (catch-all block).
3. Run `cargo build` -- fix any test struct literal compilation errors by adding
   `note_render_slot: Some("<slot>".to_string())` to synthetic SectionConfig values.
4. Run `cargo test --manifest-path "C:/Users/solar/Documents/Claude Projects/scribblenot/Cargo.toml"` -- zero regressions.
5. Commit `src/note.rs` with message:
   `Implement task #51 sub-task 51.4: replace cfg.id render checks with note_render_slot`

**Verification**:

### Automated tests
- `cargo build` -- zero warnings
- `cargo test` (full suite) -- zero regressions

## Progress

Implemented 2026-04-03. Commit: 59e90c6.

All 9 individual cfg.id checks and the catch-all block replaced with note_render_slot lookups.
Test suite: 180 passing, 0 failed.

Synthetic SectionConfig sites fixed by updating 3 factory helpers:
- `make_section(id, section_type)` (line 702): set `note_render_slot: Some(id.to_string())`
  - Covers: "tx_plan" -> `Some("tx_plan")`, "adl" -> `Some("adl")`, etc.
- `make_multi_field_section(id)` (line 1059): set `note_render_slot: Some(id.to_string())`
  - Covers: "header" -> `Some("header")`, "test_section" -> `Some("test_section")`
- `make_multi_field_section_with_id(id)` (line 1454): set `note_render_slot: Some(id.to_string())`
  - Covers: "tx_mods" -> `Some("tx_mods")`, "my_custom_section" -> `Some("my_custom_section")`

The inline SectionConfig in `make_two_field_header` (id: "test_section") was left with
`note_render_slot: None` as it only exercises render_multifield_section directly, not render_note.
