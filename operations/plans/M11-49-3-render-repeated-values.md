## Task

#49 - Add repeat_limit: N to multi_field fields (sub-task 3: note renderer emits all repeated values)

## Context

`HeaderState.repeated_values` is now `Vec<Vec<String>>` (one inner Vec per field slot). Sub-task 2 migrated all call sites in `note.rs` to use `.last()` on each slot - which was the minimum change to keep existing tests compiling. Sub-task 3 finishes the rendering work: non-repeated appointment fields should read `.first()` (the only confirmed value in those slots), while fields with `repeat_limit` set should emit one output line per confirmed value in confirmation order (all entries in the slot, not just the last).

Currently `format_header_preview` and `format_header_export` are hardcoded to the four appointment fields (date, start_time, appointment_duration, appointment_type), none of which have `repeat_limit`. The `render_note` function routes all `multi_field` sections through these two functions. To support repeatable fields, a generic fallback renderer must iterate `field_configs` and for each field emit all confirmed values in the slot as separate lines.

## Approach

Add a generic multi-field section renderer (`format_header_generic_preview` / `format_header_generic_export`) that walks `field_configs` in order. For each field, it reads all entries in `repeated_values[i]` (confirmed values in order). Each confirmed value is rendered via `resolve_multifield_value` and emitted as a separate line. Non-repeatable fields (no `repeat_limit`) use only the first entry in the slot.

Update `format_header_preview` and `format_header_export` to use `.first()` instead of `.last()` for the appointment field lookups - making the intent explicit. The appointment section-specific format (date/time/duration/type layout) is preserved unchanged.

Route `render_note` through the generic renderer for any `multi_field` section whose `field_configs` contains at least one field with `repeat_limit`. The appointment header section has no `repeat_limit` fields, so it continues through the existing hardcoded formatters.

## Critical Files

- `src/note.rs` - `format_header_preview` (lines 383-411), `format_header_export` (lines 413-461), `render_note` multi_field dispatch (lines 93-119)
- `src/sections/multi_field.rs` - `resolve_multifield_value` (line 101) - reused, no changes
- `src/sections/header.rs` - `HeaderState.repeated_values`, `HeaderState.field_configs` - read only, no changes

## Reuse

- `resolve_multifield_value(confirmed, cfg, sticky_values)` from `src/sections/multi_field.rs` - already imported in `note.rs` (line 3); call with each entry in the slot as `confirmed`
- `format_header_date` and `format_header_time` from `src/note.rs` - reused by the appointment-specific formatters (unchanged)
- `ResolvedMultiFieldValue::preview_str()` and `ResolvedMultiFieldValue::export_value()` - existing methods on the resolved value type

## Steps

1. **In `format_header_preview` and `format_header_export`, change `.last()` to `.first()`** for the appointment field slot lookups. This makes the intent explicit: non-repeatable fields have exactly one confirmed value and `.first()` names that clearly.

In `format_header_preview` (lines 393-396):
```diff
-                let confirmed = hs.repeated_values.get(i)
-                    .and_then(|v| v.last())
-                    .map(|s| s.as_str())
-                    .unwrap_or("");
+                let confirmed = hs.repeated_values.get(i)
+                    .and_then(|v| v.first())
+                    .map(|s| s.as_str())
+                    .unwrap_or("");
```

In `format_header_export` (lines 425-428):
```diff
-                let confirmed = hs.repeated_values.get(i)
-                    .and_then(|v| v.last())
-                    .map(|s| s.as_str())
-                    .unwrap_or("");
+                let confirmed = hs.repeated_values.get(i)
+                    .and_then(|v| v.first())
+                    .map(|s| s.as_str())
+                    .unwrap_or("");
```

In `render_note` `has_any` check (line 99-100):
```diff
                     let confirmed = hs.repeated_values.get(i)
-                        .and_then(|v| v.last())
+                        .and_then(|v| v.first())
                         .map(|s| s.as_str())
                         .unwrap_or("");
```

2. **Add `format_header_generic_preview`** below `format_header_preview`. This function iterates all fields in order. For each field:
   - If `cfg.repeat_limit` is `Some(_)`: iterate all entries in `repeated_values[i]`, resolve each with `resolve_multifield_value`, and collect the non-empty `preview_str()` values.
   - If `cfg.repeat_limit` is `None`: use only the first entry (same as the appointment formatters).
   - Emit each resolved value as a line prefixed with the field name and a colon, e.g. `"Modification: <value>"`.
   - Fields with no resolved values show `"<field_name>: --"`.
   - Lines are joined with `\n`.

```diff
+/// Format a generic multi_field header section for live note preview.
+/// Non-repeatable fields show their first confirmed value (or "--").
+/// Repeatable fields (repeat_limit is Some) emit one line per confirmed value.
+fn format_header_generic_preview(
+    hs: &crate::sections::header::HeaderState,
+    sticky_values: &HashMap<String, String>,
+) -> String {
+    let mut lines: Vec<String> = Vec::new();
+    for (i, cfg) in hs.field_configs.iter().enumerate() {
+        let slot = hs.repeated_values.get(i).map(|v| v.as_slice()).unwrap_or(&[]);
+        if cfg.repeat_limit.is_some() {
+            // Emit one line per confirmed value in order
+            if slot.is_empty() {
+                lines.push(format!("{}: --", cfg.name));
+            } else {
+                for entry in slot {
+                    let resolved = resolve_multifield_value(entry.as_str(), cfg, sticky_values);
+                    lines.push(format!("{}: {}", cfg.name, resolved.preview_str()));
+                }
+            }
+        } else {
+            let confirmed = slot.first().map(|s| s.as_str()).unwrap_or("");
+            let resolved = resolve_multifield_value(confirmed, cfg, sticky_values);
+            lines.push(format!("{}: {}", cfg.name, resolved.preview_str()));
+        }
+    }
+    lines.join("\n")
+}
```

3. **Add `format_header_generic_export`** below `format_header_generic_preview`. Same logic but uses `export_value()` and omits fields with no resolved value entirely (no "--" placeholder).

```diff
+/// Format a generic multi_field header section for clipboard export.
+/// Non-repeatable fields emit their first confirmed value (if resolved).
+/// Repeatable fields (repeat_limit is Some) emit one line per confirmed value.
+/// Returns None when no fields resolve at all.
+fn format_header_generic_export(
+    hs: &crate::sections::header::HeaderState,
+    sticky_values: &HashMap<String, String>,
+) -> Option<String> {
+    let mut lines: Vec<String> = Vec::new();
+    for (i, cfg) in hs.field_configs.iter().enumerate() {
+        let slot = hs.repeated_values.get(i).map(|v| v.as_slice()).unwrap_or(&[]);
+        if cfg.repeat_limit.is_some() {
+            for entry in slot {
+                let resolved = resolve_multifield_value(entry.as_str(), cfg, sticky_values);
+                if let Some(val) = resolved.export_value() {
+                    lines.push(val.to_string());
+                }
+            }
+        } else {
+            let confirmed = slot.first().map(|s| s.as_str()).unwrap_or("");
+            let resolved = resolve_multifield_value(confirmed, cfg, sticky_values);
+            if let Some(val) = resolved.export_value() {
+                lines.push(val.to_string());
+            }
+        }
+    }
+    if lines.is_empty() {
+        None
+    } else {
+        Some(lines.join("\n"))
+    }
+}
```

4. **Update the `render_note` multi_field dispatch** (lines 93-119) to route non-appointment header sections through the generic renderers. A section is "appointment-style" when none of its fields have `repeat_limit`. Add a helper predicate and branch:

```diff
 if cfg.section_type == "multi_field" {
     if let SectionState::Header(hs) = state {
+        let has_repeatable = hs.field_configs.iter().any(|c| c.repeat_limit.is_some());
         match &mode {
             NoteRenderMode::Preview => {
                 let has_any = hs.field_configs.iter().enumerate().any(|(i, fcfg)| {
                     let confirmed = hs.repeated_values.get(i)
                         .and_then(|v| v.first())
                         .map(|s| s.as_str())
                         .unwrap_or("");
                     !resolve_multifield_value(confirmed, fcfg, sticky_values).is_empty_variant()
                 });
                 if has_any {
-                    Some(format_header_preview(hs, sticky_values))
+                    if has_repeatable {
+                        Some(format_header_generic_preview(hs, sticky_values))
+                    } else {
+                        Some(format_header_preview(hs, sticky_values))
+                    }
                 } else {
                     None
                 }
             }
-            NoteRenderMode::Export => format_header_export(hs, sticky_values),
+            NoteRenderMode::Export => {
+                if has_repeatable {
+                    format_header_generic_export(hs, sticky_values)
+                } else {
+                    format_header_export(hs, sticky_values)
+                }
+            }
         }
```

5. **Update and redirect the pre-written ST49-3 TDD tests** in `src/note.rs` (lines ~781-941). The TDD tests were written before the generic-renderer architecture was chosen. They call `format_header_export` and `format_header_preview` directly with a `modifications` field (`repeat_limit: Some(N)`), but those appointment-specific functions do not look up non-appointment fields — so those calls will produce wrong results (None / placeholder) rather than the expected repeated-value output.

   For each test that passes a `repeat_limit: Some(_)` field and calls `format_header_export` or `format_header_preview` directly, redirect the call to the generic equivalent:
   - `format_header_export` → `format_header_generic_export`
   - `format_header_preview` → `format_header_generic_preview`

   Tests affected (call site must change, assertions remain the same):
   - `export_emits_all_values_for_repeated_field` (calls `format_header_export` → redirect to `format_header_generic_export`)
   - `export_emits_single_value_for_repeated_field_with_one_entry` (same redirect)
   - `export_emits_repeated_values_in_confirmation_order` (same redirect)
   - `preview_emits_all_values_for_repeated_field` (calls `format_header_preview` → redirect to `format_header_generic_preview`)

   Tests that call the appointment-specific formatters with non-repeatable fields (`repeat_limit: None`) are correct as-is and do NOT need changing:
   - `export_uses_first_value_for_non_repeated_field`
   - `preview_uses_first_value_for_non_repeated_field`

   No new test functions need to be added: the redirected TDD tests already cover the four required scenarios (multi-value emit, single-value emit, order preservation, None-when-empty).

6. **Run `cargo test`** and fix any compilation errors or test failures.

## Verification

### Manual tests

- Launch the app, navigate to the appointment header, confirm all four fields. Verify the rendered note preview shows date/time/duration/type in the existing format (unchanged behavior).
- In sections.yml, set `repeat_limit: 2` on a header field in a second multi_field section (not the appointment header). Confirm two values for that field. Verify the note preview and clipboard export each emit two separate lines for that field, one per confirmed value.
- Confirm a non-repeating field in the same section and verify only one line appears (not duplicated).

### Automated tests

- `cargo test` - all existing note.rs tests must pass (including `export_full_header`, `export_omits_unresolved_fields`, `preview_shows_placeholder_for_unresolved`).
- The four redirected ST49-3 TDD tests updated in step 5 must all pass: `export_emits_all_values_for_repeated_field`, `export_emits_single_value_for_repeated_field_with_one_entry`, `export_emits_repeated_values_in_confirmation_order`, `preview_emits_all_values_for_repeated_field`.

## Changelog

### Review - 2026-04-03
- #1 (nit): Step 4 diff: changed `.last()` to `.first()` in the `has_any` check for consistency with Step 1's `.first()` migration in the appointment formatters.

### Review - 2026-04-03
- #2 (blocking): Step 5 replaced with redirect instructions for the pre-written ST49-3 TDD tests. Those tests call `format_header_export`/`format_header_preview` directly with `repeat_limit: Some(N)` fields; the appointment-specific formatters ignore non-appointment field IDs and would produce wrong output (None or placeholder). Step 5 now instructs the implementer to change the four affected test call sites to use `format_header_generic_export`/`format_header_generic_preview` instead, keeping all assertions unchanged. Verification section updated to name those four tests explicitly.

### Review - 2026-04-03
- #3 (blocking): Step 1 was missing a diff for the `has_any` check in `render_note` (note.rs line 100), which also uses `.last()` and must be changed to `.first()`. Step 4's diff already showed this line as a context line with `.first()` (R1's nit), but the change was never listed in Step 1, so the diff context in Step 4 would not apply cleanly against the unmodified file. Added an explicit third diff block to Step 1 covering note.rs lines 99-100.

## Implementation
Complete - 2026-04-03

## Progress
- Step 1: Changed .last() to .first() in format_header_preview, format_header_export, and render_note has_any check
- Step 2: Added format_header_generic_preview function below format_header_export
- Step 3: Added format_header_generic_export function below format_header_generic_preview
- Step 4: Updated render_note to route sections with repeat_limit fields through generic renderers
- Step 5: Redirected 4 TDD tests to call generic functions instead of appointment-specific ones
- Step 6: cargo test passed - all 142 tests pass including all 6 ST49-3 tests
