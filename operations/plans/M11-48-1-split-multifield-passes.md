## Task

#48 - Generalize multi_field note rendering to support arbitrary sections beyond the appointment header

## Context

`render_note` in `src/note.rs` currently uses a single `find_map` over all section pairs to find the first `multi_field` section with a `SectionState::Header` state, render it as the appointment header, and stop. Any subsequent `multi_field` sections are silently dropped. Task #48 requires that ALL `multi_field` sections render; the appointment header (identified by `cfg.id == "header"`) must render first and byte-for-byte identically using the existing header functions, and every remaining `multi_field` section must be rendered generically afterward using the already-written `format_header_generic_preview` / `format_header_generic_export` functions. Five TDD tests already exist in `src/note.rs` (lines 1036-1303) and must pass after this change.

## Approach

Replace the single `find_map` block and its `if let Some(h)` push (lines 92-134) with two sequential passes:

1. **Pass 1 - appointment header**: iterate over `(cfg, state)` pairs; if `cfg.id == "header"` AND `state` is `SectionState::Header`, render using the existing `format_header_preview` / `format_header_export` / `format_header_generic_preview` / `format_header_generic_export` functions (identical logic to the current `find_map` body), push result into `parts`, then break.
2. **Pass 2 - generic multi_field sections**: iterate again over all `(cfg, state)` pairs; skip any where `cfg.id == "header"`; for each remaining `multi_field` section where state is `SectionState::Header`, render using `format_header_generic_preview` (Preview) or `format_header_generic_export` (Export) and push non-empty results into `parts`.

The rest of `render_note` (intake, subjective, separator, TX/plan, etc.) is untouched.

## Critical Files

- `src/note.rs` lines 92-134 (the `find_map` block and the `if let Some(h)` push)
- `src/note.rs` lines 1036-1319 (the five TDD tests that must pass)
- `src/note.rs` lines 394-534 (`format_header_preview`, `format_header_export`, `format_header_generic_preview`, `format_header_generic_export` - reused, not modified)

## Reuse

- `format_header_preview(hs, sticky_values)` - line 394, existing appointment header preview renderer
- `format_header_export(hs, sticky_values)` - line 426, existing appointment header export renderer
- `format_header_generic_preview(hs, sticky_values)` - line 477, generic preview renderer (also used for Pass 1 when `has_repeatable`)
- `format_header_generic_export(hs, sticky_values)` - line 507, generic export renderer (also used for Pass 1 when `has_repeatable`)
- `resolve_multifield_value` - line 3 import, used inside the existing preview `has_any` check (keep as-is for Pass 1)

## Steps

1. In `src/note.rs`, replace lines 92-134 (the comment, `find_map` block, and `if let Some(h)` push) with the two-pass implementation below.

```diff
-    // Header: always first - find the multi_field section
-    let header_text = sections.iter().zip(states.iter()).find_map(|(cfg, state)| {
-        if cfg.section_type == "multi_field" {
-            if let SectionState::Header(hs) = state {
-                let has_repeatable = hs.field_configs.iter().any(|c| c.repeat_limit.is_some());
-                match &mode {
-                    NoteRenderMode::Preview => {
-                        let has_any = hs.field_configs.iter().enumerate().any(|(i, fcfg)| {
-                            let confirmed = hs.repeated_values.get(i)
-                                .and_then(|v| v.first())
-                                .map(|s| s.as_str())
-                                .unwrap_or("");
-                            !resolve_multifield_value(confirmed, fcfg, sticky_values).is_empty_variant()
-                        });
-                        if has_any {
-                            if has_repeatable {
-                                Some(format_header_generic_preview(hs, sticky_values))
-                            } else {
-                                Some(format_header_preview(hs, sticky_values))
-                            }
-                        } else {
-                            None
-                        }
-                    }
-                    NoteRenderMode::Export => {
-                        if has_repeatable {
-                            format_header_generic_export(hs, sticky_values)
-                        } else {
-                            format_header_export(hs, sticky_values)
-                        }
-                    }
-                }
-            } else {
-                None
-            }
-        } else {
-            None
-        }
-    });
-
-    if let Some(h) = header_text {
-        parts.push(h);
-    }
+    // Pass 1: render the appointment header section (cfg.id == "header") first.
+    for (cfg, state) in sections.iter().zip(states.iter()) {
+        if cfg.section_type == "multi_field" && cfg.id == "header" {
+            if let SectionState::Header(hs) = state {
+                let has_repeatable = hs.field_configs.iter().any(|c| c.repeat_limit.is_some());
+                let rendered = match &mode {
+                    NoteRenderMode::Preview => {
+                        let has_any = hs.field_configs.iter().enumerate().any(|(i, fcfg)| {
+                            let confirmed = hs.repeated_values.get(i)
+                                .and_then(|v| v.first())
+                                .map(|s| s.as_str())
+                                .unwrap_or("");
+                            !resolve_multifield_value(confirmed, fcfg, sticky_values).is_empty_variant()
+                        });
+                        if has_any {
+                            if has_repeatable {
+                                Some(format_header_generic_preview(hs, sticky_values))
+                            } else {
+                                Some(format_header_preview(hs, sticky_values))
+                            }
+                        } else {
+                            None
+                        }
+                    }
+                    NoteRenderMode::Export => {
+                        if has_repeatable {
+                            format_header_generic_export(hs, sticky_values)
+                        } else {
+                            format_header_export(hs, sticky_values)
+                        }
+                    }
+                };
+                if let Some(h) = rendered {
+                    parts.push(h);
+                }
+            }
+            break;
+        }
+    }
+
+    // Pass 2: render all remaining multi_field sections generically (skip the appointment header).
+    for (cfg, state) in sections.iter().zip(states.iter()) {
+        if cfg.section_type == "multi_field" && cfg.id != "header" {
+            if let SectionState::Header(hs) = state {
+                match &mode {
+                    NoteRenderMode::Preview => {
+                        let rendered = format_header_generic_preview(hs, sticky_values);
+                        if !rendered.trim().is_empty() {
+                            parts.push(rendered);
+                        }
+                    }
+                    NoteRenderMode::Export => {
+                        if let Some(rendered) = format_header_generic_export(hs, sticky_values) {
+                            parts.push(rendered);
+                        }
+                    }
+                }
+            }
+        }
+    }
```

2. Run `cargo test` to confirm all five ST48-1 tests pass and no existing tests regress.

## Verification

### Manual tests

- Open the app with a standard appointment config (sections.yml containing a `multi_field` section with `id: header`). Confirm the appointment header renders at the top of the note preview pane as before.
- If a second `multi_field` section exists in sections.yml (e.g. future tx_mods), confirm its output appears below the appointment header in the note preview.

### Automated tests

- `cargo test` - the five pre-written tests at lines 1036-1319 in `src/note.rs` cover all required behaviors:
  - `preview_renders_header_section_output` (TEST-1): header section renders in preview
  - `preview_renders_second_multi_field_section_output` (TEST-2): second section renders in preview
  - `preview_renders_both_multi_field_sections` (TEST-3): both sections render together in preview
  - `export_renders_second_multi_field_section_output` (TEST-4): second section renders in export
  - `header_output_unchanged_when_second_section_present` (TEST-5): header output is byte-for-byte identical before and after refactor

## Prefect-2 Report

### Nit
1. **Critical Files range includes 3 context lines not in the diff** (`M11-48-1-split-multifield-passes.md:20`) — Critical Files says "lines 89-134 (the `find_map` block and the `if let Some(h)` push)" but lines 89-91 (`let mut parts`, `let today`, and a blank line) are function-body boilerplate that is NOT removed by the diff. Step 1 correctly targets lines 92-134. The Critical Files range should read "92-134" to match.

## Changelog

### Review - 2026-04-03
- #1: Approach section line range "93-130" corrected to "92-134" to include the comment and `if let Some(h)` push that the diff actually removes

### Review - 2026-04-03 (R2)
- #1 (nit): Critical Files test end line corrected from 1303 to 1319; the fifth test block closes at line 1319, not 1303

### Prefect-1 – 2026-04-03 (R1)
- #1 (minor): Verification section line 153 still referenced the old "1036-1303" range that R2 fixed only in Critical Files; corrected to "1036-1319" to match actual source

### Review – 2026-04-03 (R4)
- #1 (nit): Critical Files start line corrected from 89 to 92; lines 89-91 are boilerplate not removed by the diff (flagged in Prefect-2 Report)
- #2 (nit): Critical Files end line for helper functions corrected from 533 to 534; `format_header_generic_export` closing brace is at line 534

## Implementation
Complete - 2026-04-03

## Progress
- Step 1: Replaced find_map block (lines 92-134) with two-pass implementation in render_note
- Step 2: Ran cargo test - all 147 tests pass (including 5 ST48-1 tests)

## Prefect-1 Report

### Minor
1. **Verification/Automated tests line range stale** (`M11-48-1-split-multifield-passes.md:153`) — The Verification section's `cargo test` bullet still said "1036-1303". The R2 changelog updated Critical Files but left Verification unchanged. Corrected to "1036-1319" to match the actual closing line of the fifth test block in `src/note.rs`.
