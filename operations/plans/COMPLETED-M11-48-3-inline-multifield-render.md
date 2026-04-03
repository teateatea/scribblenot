## Task

#48 - Generalize multi_field note rendering to support arbitrary sections beyond the appointment header

## Context

Sub-task 1 introduced a two-pass strategy where Pass 2 collects all non-header `multi_field` sections and prepends their output before the INTAKE block. This puts tx_mods-like content at the wrong position - before the INTAKE separator and before `## TREATMENT / PLAN`. The correct behavior is for each non-header `multi_field` section to render inline at the position already defined for its `cfg.id` in the `render_note` loop. The `tx_mods` section, for example, should appear inside the `// tx_mods` block under `## TREATMENT / PLAN`. Four TDD tests in `src/note.rs` (lines 1491-1644) already exist and currently fail because Pass 2 dumps content at the wrong position.

## Approach

Remove Pass 2 entirely (lines 132-151 in `render_note`). Then, at each named section block inside `render_note` that already handles a specific `cfg.id`, add a branch: if the section's `section_type` is `"multi_field"`, call `render_multifield_section` (already implemented as of sub-task 2) instead of `render_section_content`. For `tx_mods` specifically this means replacing the `render_section_content` call with `render_multifield_section`. For unknown/unrecognized `multi_field` section ids, add a catch-all at the end of the render loop (after INFECTION CONTROL, before the final separator) so their content is not silently dropped. This preserves position semantics for known ids and provides a safe fallback for future ids.

## Critical Files

- `src/note.rs` lines 132-151 (Pass 2 block to remove)
- `src/note.rs` lines 209-217 (tx_mods render block - call `render_multifield_section` here instead of `render_section_content`)
- `src/note.rs` line 556 (`render_multifield_section` public function, already exists - reuse as-is)
- `src/note.rs` lines 1491-1644 (four ST48-3 TDD tests that must pass after the fix)

## Reuse

- `render_multifield_section(cfg, hs, sticky_values, mode)` at line 556 - already public, call for any `SectionState::Header` branch inside named section blocks
- `format_header_generic_preview` and `format_header_generic_export` - used internally by `render_multifield_section`; no direct calls needed

## Steps

1. Delete Pass 2 from `render_note` (lines 132-151). This block prepends non-header multi_field output before INTAKE and is entirely replaced by inline handling:

```diff
-    // Pass 2: render all remaining multi_field sections generically (skip the appointment header).
-    for (cfg, state) in sections.iter().zip(states.iter()) {
-        if cfg.section_type == "multi_field" && cfg.id != "header" {
-            if let SectionState::Header(hs) = state {
-                match &mode {
-                    NoteRenderMode::Preview => {
-                        let rendered = format_header_generic_preview(hs, sticky_values);
-                        if !rendered.trim().is_empty() {
-                            parts.push(rendered);
-                        }
-                    }
-                    NoteRenderMode::Export => {
-                        if let Some(rendered) = format_header_generic_export(hs, sticky_values) {
-                            parts.push(rendered);
-                        }
-                    }
-                }
-            }
-        }
-    }
-
```

2. In the `// tx_mods` block (currently lines 209-217), replace the `render_section_content` call with `render_multifield_section` for the multi_field case. `NoteRenderMode` does not derive `Clone`, so first add `#[derive(Clone)]` to it (it is a simple enum with no non-Clone fields), then call `render_multifield_section(cfg, hs, sticky_values, mode.clone())`. Full rewrite for step 2:

```diff
+#[derive(Clone)]
 pub enum NoteRenderMode {
     Preview,
     Export,
 }
```

```diff
     // tx_mods
     for (cfg, state) in sections.iter().zip(states.iter()) {
         if cfg.id == "tx_mods" {
-            let rendered = render_section_content(cfg, state, &today);
-            if !rendered.trim().is_empty() {
-                tx_parts.push(format!("\n\n\n#### TREATMENT MODIFICATIONS & PREFERENCES\n{}", rendered));
-            }
+            if cfg.section_type == "multi_field" {
+                if let SectionState::Header(hs) = state {
+                    if let Some(rendered) = render_multifield_section(cfg, hs, sticky_values, mode.clone()) {
+                        if !rendered.trim().is_empty() {
+                            tx_parts.push(format!("\n\n\n#### TREATMENT MODIFICATIONS & PREFERENCES\n{}", rendered));
+                        }
+                    }
+                }
+            } else {
+                let rendered = render_section_content(cfg, state, &today);
+                if !rendered.trim().is_empty() {
+                    tx_parts.push(format!("\n\n\n#### TREATMENT MODIFICATIONS & PREFERENCES\n{}", rendered));
+                }
+            }
         }
     }
```

3. Add a catch-all block immediately after the INFECTION CONTROL block (before `parts.push("\n\n\n_______________\n"`) to render any non-header multi_field section whose id was not matched by any named block. This prevents unknown future sections from silently producing no output (required by ST48-3-TEST-2):

```diff
+    // Catch-all: non-header multi_field sections with unrecognized ids
+    for (cfg, state) in sections.iter().zip(states.iter()) {
+        if cfg.section_type == "multi_field" && cfg.id != "header" {
+            let known_ids = ["tx_mods"];
+            if !known_ids.contains(&cfg.id.as_str()) {
+                if let SectionState::Header(hs) = state {
+                    if let Some(rendered) = render_multifield_section(cfg, hs, sticky_values, mode.clone()) {
+                        if !rendered.trim().is_empty() {
+                            parts.push(format!("\n\n\n#### {}\n{}", cfg.name.to_uppercase(), rendered));
+                        }
+                    }
+                }
+            }
+        }
+    }
+
     parts.push("\n\n\n_______________\n".to_string());
```

4. Run `cargo test` to confirm all four ST48-3 tests pass and no existing tests regress.

## Verification

### Manual tests

- Open the app and navigate to a note with tx_mods content. Confirm the treatment modifications appear under `## TREATMENT / PLAN`, not before the intake separator.
- Confirm the appointment header still renders correctly at the top of the note.
- Copy note to clipboard (export mode) and confirm treatment modification values appear after `## TREATMENT / PLAN`.

### Automated tests

- `cargo test` - the four ST48-3 tests at lines 1491-1644 in `src/note.rs` must all pass:
  - `non_header_multi_field_section_appears_after_treatment_heading`: sentinel appears after `## TREATMENT / PLAN`
  - `non_header_multi_field_section_with_unknown_id_produces_output`: generic id content is not silently dropped
  - `non_header_multi_field_section_not_before_intake_separator`: sentinel does not appear before the first `_______________`
  - `export_non_header_multi_field_section_appears_after_treatment_heading`: export mode sentinel appears after `## TREATMENT / PLAN`
- All 152+ existing tests must continue to pass (no regression)

## Progress

- Step 1: Deleted Pass 2 block (lines 132-151) that dumped non-header multi_field sections before INTAKE
- Step 2: Added #[derive(Clone)] to NoteRenderMode; replaced tx_mods render_section_content call with multi_field-aware branch using render_multifield_section
- Step 3: Added catch-all block after INFECTION CONTROL for unrecognized multi_field section ids
- Step 4: cargo test passes - 156/156 tests OK including all four ST48-3 tests

## Implementation
Complete - 2026-04-03

## Changelog

### Review - 2026-04-03
- #1: Step 2 - removed two superseded intermediate diff blocks (the first used a non-existent `NoteRenderMode::from(&mode)`, the second contained syntactically broken Rust); retained only the final correct implementation using `#[derive(Clone)]` and `mode.clone()`
