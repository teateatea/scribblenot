**Task**: #51 Move hard-coded section metadata into sections.yml

**Context**: ST51.1-51.2 added fields to SectionConfig and populated sections.yml. ST51.3
replaces the three hard-coded functions in note.rs with cfg field lookups:
- `heading_anchor()` — section ID arms removed; usage in section_start_line() replaced with
  cfg.heading_search_text lookup. Group ID arms remain (groups don't have SectionConfig).
- `is_intake_section()` — replaced with cfg.is_intake
- `intake_heading()` — replaced with cfg.heading_label (with fallback)

The old `tx_mods_heading_anchor_maps_to_treatment_modifications` test was replaced in ST51.3
pre-work with `tx_mods_section_start_line_finds_treatment_modifications_heading` (behavioral
test that doesn't call heading_anchor() directly).

**Approach**: Three targeted edits in note.rs. Remove section ID arms from heading_anchor()
(keep group arms intact). Replace usages of is_intake_section() and intake_heading() in
render_note() with direct cfg field access. Remove the now-unused is_intake_section() and
intake_heading() functions. heading_anchor() stays but reduced to group IDs only.

**Critical Files**:
- `src/note.rs` lines 16-41 (heading_anchor -- remove section ID arms)
- `src/note.rs` lines 54 (heading_anchor(section_id) usage -- replace with cfg lookup)
- `src/note.rs` lines 133-154 (intake loop -- replace is_intake_section and intake_heading)
- `src/note.rs` lines 307-324 (is_intake_section and intake_heading -- delete both functions)

**Steps**:

1. **Replace heading_anchor usage at line 54** in `section_start_line()`:

```diff
-    let anchor = heading_anchor(section_id);
+    let anchor = sections
+        .iter()
+        .find(|s| s.id == section_id)
+        .and_then(|s| s.heading_search_text.as_deref())
+        .unwrap_or("");
```

2. **Shrink heading_anchor()** to group IDs only (remove all section ID arms):

```diff
 fn heading_anchor(id: &str) -> &'static str {
     match id {
-        // Section anchors
-        "adl"                      => "ACTIVITIES OF DAILY LIVING",
-        "exercise"                 => "EXERCISE HABITS",
-        "sleep_diet"               => "SLEEP & DIET",
-        "social"                   => "SOCIAL & STRESS",
-        "history"                  => "HISTORY & PREVIOUS DIAGNOSES",
-        "specialists"              => "SPECIALISTS & TREATMENT",
-        "subjective_section"       => "## SUBJECTIVE",
-        "tx_mods"                  => "TREATMENT MODIFICATIONS",
-        "tx_regions"               => "TREATMENT / PLAN",
-        "objective_section"        => "## OBJECTIVE / OBSERVATIONS",
-        "post_treatment"           => "## POST-TREATMENT",
-        "remedial_section"         => "REMEDIAL EXERCISES",
-        "tx_plan"                  => "TREATMENT PLAN",
-        "infection_control_section" => "INFECTION CONTROL",
         // Group anchors
         "subjective"               => "## SUBJECTIVE",
         "treatment"                => "## TREATMENT / PLAN",
         "objective"                => "## OBJECTIVE / OBSERVATIONS",
         "post_tx"                  => "## POST-TREATMENT",
-        // intake, header, and anything else: no anchor in the rendered note
         _                          => "",
     }
 }
```

3. **Replace is_intake_section() and intake_heading() in render_note()** (lines ~133-154):

```diff
-        .filter(|(cfg, _)| cfg.section_type != "multi_field" && is_intake_section(cfg))
+        .filter(|(cfg, _)| cfg.section_type != "multi_field" && cfg.is_intake)
```

```diff
-            let heading = intake_heading(cfg);
+            let heading = cfg
+                .heading_label
+                .as_deref()
+                .map(|s| s.to_string())
+                .unwrap_or_else(|| format!("#### {}", cfg.name.to_uppercase()));
```

4. **Delete is_intake_section() function** (lines 307-312).

5. **Delete intake_heading() function** (lines 314-324).

6. Run `cargo build --manifest-path "C:/Users/solar/Documents/Claude Projects/scribblenot/Cargo.toml"` -- must compile with zero warnings.

7. Run `cargo test --manifest-path "C:/Users/solar/Documents/Claude Projects/scribblenot/Cargo.toml"` -- must pass (expect ~180 passing).

8. Commit `src/note.rs` with message:
   `Implement task #51 sub-task 51.3: replace hardcoded section functions with cfg field lookups`

**Verification**:

### Automated tests
- `cargo test tx_mods_section_start_line_finds_treatment_modifications_heading` -- must pass
- `cargo test` (full suite) -- zero regressions
- `cargo build` -- zero warnings (no dead_code warnings for removed functions)

## Progress

Implemented 2026-04-03. All 6 edits applied to `src/note.rs`:
- Edit 1: `heading_anchor(section_id)` replaced with `cfg.heading_search_text` iterator lookup
- Edit 2: `heading_anchor()` reduced to 4 group-ID arms only (section arms removed)
- Edit 3: `is_intake_section(cfg)` replaced with `cfg.is_intake`
- Edit 4: `intake_heading(cfg)` replaced with `cfg.heading_label` field lookup + fallback
- Edit 5: `is_intake_section()` function deleted
- Edit 6: `intake_heading()` function deleted

One test fix required: `non_empty_tx_plan_returns_own_heading_line` used `make_section("tx_plan", ...)` without `heading_search_text`, which the old `heading_anchor()` covered. Updated test to set `heading_search_text: Some("TREATMENT PLAN".to_string())` on the synthetic config.

Build: zero warnings. Tests: 180 passed, 0 failed.
Commit: dcfe3c9
