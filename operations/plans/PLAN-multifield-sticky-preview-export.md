## Task

Generalize `multi_field` value resolution so live preview, note rendering, and super confirm all use the same rules:

- confirmed value first
- then sticky value
- then configured default
- then empty

Keep `--` as a preview-only placeholder. Do not export fake or malformed header content to the clipboard.

## Why This Exists

The current bug is visible in the appointment header, but the underlying problem is broader:

- `note.rs` currently hard-codes header rendering behavior
- live preview and exported note content are conflated
- composite field defaults are resolved differently in different places
- super confirm only understands simple defaults / flat sticky values

If this is fixed only for `Header`, the same inconsistency will return when new `multi_field` sections are added.

## Goals

1. Fix the current header bugs.
2. Make value resolution reusable for future `multi_field` sections.
3. Use real sticky/default data, not fabricated placeholder values.
4. Keep preview placeholders out of clipboard/export output.
5. Make super confirm reuse the same resolution logic as rendering.

## Non-Goals

- Do not change YAML data shape in this task.
- Do not add new app state for header preview.
- Do not redesign note layout for non-header sections in this task.
- Do not silently broaden sticky persistence behavior beyond what already exists.

## Current Facts In The Codebase

- `src/note.rs`
  - `render_note()` currently renders the header when `hs.completed` or any stored header value is non-empty.
  - `format_header()` is header-specific and currently couples value resolution with final string formatting.
  - `section_start_line()` also calls `render_note()`, so any signature change must update that internal caller too.
  - The older duration lookup bug may already be fixed in the current tree; verify before treating it as active work.
- `src/ui.rs`
  - `compute_field_preload()` already has useful logic for composite fields:
    - sticky part value
    - then part default
    - then preview
  - This is preview/input-oriented logic, not export-oriented logic.
- `src/modal.rs`
  - composite field navigation already uses sticky part values correctly.
- `src/app.rs`
  - header `super confirm` currently resolves only:
    - `self.config.sticky_values.get(&cfg.id)`
    - then `cfg.default`
  - that means simple fields work, but composite sticky/default behavior is not honored.
- `data/config.yml`
  - sticky values already persist composite parts like:
    - `date.year`
    - `date.month`
    - `date.day`

## Design Decision

Do not solve this by inventing composite defaults in `note.rs`.

That approach produces fake content when a field has no real resolved value, for example:

- `date` becoming `2025-01-01` just because the first option exists
- `start_time` becoming a plausible time even when nothing has been confirmed

Instead, use one shared resolver that understands:

- confirmed stored value
- sticky values
- configured defaults

Then format the resolved result differently for:

- preview
- export / clipboard

## Recommended Architecture

### 1. Extract generic `multi_field` resolution helpers

Do not make the new helpers header-specific unless the type system forces it.

Preferred direction:

- a generic resolver for one field
- reusable from:
  - note rendering
  - preview rendering
  - super confirm

Suggested helper responsibilities:

- Resolve simple fields:
  - confirmed value
  - sticky field value if applicable
  - field default
  - empty
- Resolve composite fields part-by-part:
  - sticky part value if `sticky: true`
  - part default
  - empty
- Reassemble a composite string from the field's format string.

Suggested output shape:

```rust
struct ResolvedMultiFieldValue {
    value: String,
    is_empty: bool,
    is_partial: bool,
}
```

This does not have to be the exact struct, but the caller needs to know:

- whether the field resolved to a usable value
- whether it is partial
- what final string it would use if rendered

If a lighter shape is more practical, this is also acceptable:

```rust
enum ResolvedMultiFieldValue {
    Empty,
    Partial(String),
    Complete(String),
}
```

### 2. Separate resolution from presentation

Keep two layers:

1. resolution
2. formatting

That separation is the key to meeting the requirement:

- preview may show `--`
- export must omit unresolved values cleanly

Do not let `--` become part of the actual note text copied to the clipboard.

### 3. Keep the header layout as a formatter, not a resolver

The appointment header still has a special output shape:

```text
{date} at {time} ({duration} min)
{appointment_type}
```

That is fine to keep as a header-specific formatting layer for now.

What should become generic is field resolution, not necessarily every section's final text layout.

## Concrete Implementation Plan

### Step 1. Verify the current tree before applying the old duration fix

Before changing anything, verify whether `src/note.rs` still looks up:

- `"duration"`

or whether it already uses:

- `"appointment_duration"`

If the current tree already uses `"appointment_duration"`, do not treat this as an implementation step. Just keep it in mind as historical context for why the bug report existed.
If the tree still uses `"duration"`, fix it as part of this task.

### Step 2. Make render context explicit: preview vs export

The plan must not leave this implicit. The implementation should not guess which formatting mode applies.

Recommended approach:

```rust
enum NoteRenderMode {
    Preview,
    Export,
}

pub fn render_note(
    sections: &[SectionConfig],
    states: &[SectionState],
    sticky_values: &HashMap<String, String>,
    mode: NoteRenderMode,
) -> String
```

Alternative acceptable shape:

- `render_note_preview(...)`
- `render_note_export(...)`

Either approach is fine. The key requirement is that the call sites are explicit.

Required call-site behavior:

- `src/ui.rs` note preview pane must use preview mode
- `src/main.rs` clipboard copy paths must use export mode
- `src/note.rs::section_start_line()` should use export mode unless there is a concrete reason it needs preview placeholders for scroll targeting

Rationale:

- preview mode may show `--`
- export mode must not leak `--` into the clipboard

### Step 3. Thread sticky values into note rendering

Whichever preview/export API shape is chosen, the renderer currently only receives:

- `sections`
- `states`

That is not enough to resolve true rolling defaults.

Change the signature so note rendering can access sticky values. Recommended:

```rust
pub fn render_note(
    sections: &[SectionConfig],
    states: &[SectionState],
    sticky_values: &HashMap<String, String>,
    mode: NoteRenderMode,
) -> String
```

Pass `&app.config.sticky_values` from all current callers:

- `src/main.rs`
- `src/ui.rs`
- `src/note.rs::section_start_line()`

Reason for passing only `sticky_values` rather than full `Config`:

- smaller dependency surface
- easier testing
- avoids coupling note rendering to unrelated config flags

### Step 4. Extract generic field resolution helpers

Add a small shared helper module if convenient, or keep the helpers local first if the write scope should stay small.

Recommended helper behavior for one field:

#### Simple field

Resolve in this order:

1. confirmed stored value
2. sticky field value
3. field default
4. empty

Notes:

- if simple-field sticky values are not currently persisted in practice, still support them cleanly if a key exists
- use the field id as the key if that matches current behavior

#### Composite field

Resolve each part in this order:

1. sticky part value if `part.sticky == true`
2. part default
3. empty

Then substitute into the composite format string.

Important:

- do not substitute UI preview labels into export values
- use option `output()` values, not labels
- if a part cannot resolve, treat that part as empty

Suggested helper decomposition:

```rust
fn resolve_simple_multifield_value(...)
fn resolve_composite_part_value(...)
fn resolve_composite_multifield_value(...)
fn resolve_multifield_value(...)
```

If a shared module is created, choose a generic name such as:

- `src/sections/multi_field.rs`
- `src/multifield.rs`

Avoid naming it `header_helpers.rs` or similar.

### Step 5. Add preview-vs-export formatting behavior

Create two formatting paths for the header, and tie them to the explicit render mode from Step 2:

- preview formatting
- export formatting

Recommended shape:

```rust
fn format_header_preview(...)
fn format_header_export(...) -> Option<String>
```

The export function can return `None` when nothing meaningful should be emitted.

This mapping must be explicit:

- `render_note(..., NoteRenderMode::Preview)` -> use `format_header_preview(...)`
- `render_note(..., NoteRenderMode::Export)` -> use `format_header_export(...)`

#### Preview rules

Preview should show placeholders so the user can see structure while entering values.

Per field:

- empty date -> `--`
- empty time -> `--`
- empty duration -> `--`
- empty appointment type -> `--`

This is UI-only behavior.

#### Export rules

Export should not include placeholders.

Rules:

- if all header fields are unresolved, omit the header entirely
- if some fields are unresolved, omit those pieces cleanly rather than emitting broken text
- never emit:
  - ` at  ( min)`
  - `--`
  - fake first-option-derived values

Recommended export formatting behavior:

- line 1:
  - include date if present
  - include `at {time}` only if time is present
  - include `({duration} min)` only if duration is present
  - join only the pieces that exist
- line 2:
  - include appointment type only if present

Examples:

- full:
  - `Thu Apr 2, 2026 at 1:00pm (60 min)\nTreatment focused massage`
- date + duration only:
  - `Thu Apr 2, 2026 (60 min)`
- appointment type only:
  - `Treatment focused massage`
- nothing resolved:
  - no header block at all

If the current note format assumes exactly two header lines, adjust carefully, but do not emit empty or misleading text.

### Step 6. Decide the visibility gate based on resolved content, not only `field_index`

The older minimal bug fix proposal was:

- `hs.completed || hs.field_index > 0`

The current tree already goes a step further:

- `hs.completed || hs.values.iter().any(|v| !v.is_empty())`

That is better than the original proposal, but still not the final rule if sticky/default-driven preview is introduced.

Recommendation:

render the header in preview if any header field resolves to a non-empty preview value.

That is more stable and matches the user's intent better.

Possible helper:

```rust
fn should_render_multifield_preview(...)
```

For the appointment header, this can mean:

- any confirmed value exists
- or any sticky/default-derived preview value exists

If implementation pressure is high, `hs.completed || hs.field_index > 0` is an acceptable first pass, but the resolved-content gate is the preferred version.

### Step 7. Include super confirm in the same refactor

This should be part of the plan, not a future suggestion.

Reason:

- once rendering uses `confirmed > sticky > default`, input confirmation should not use weaker rules
- otherwise the user will see one value in preview and get a different result from Shift+Enter

In `src/app.rs`, update the header `super confirm` branch to call the shared resolver for the active field.

Required behavior:

- resolve the active field using the same logic as note rendering
- if the field resolves to a concrete complete value:
  - `set_current_value(value)`
  - `advance()`
  - if done, `advance_section()`
- if the field does not resolve:
  - no-op
  - stay on the field

#### Composite field instruction for super confirm

Do not auto-confirm a partially resolved composite field.

Rule:

- only auto-confirm a composite field when every required part resolves to a non-empty value

That prevents silently committing values like:

- `2026-04-`
- `1:`
- an empty appointment type

This is the safest behavior and easiest to explain.

### Step 8. Preserve stable sticky key handling

Current persisted composite sticky keys use this shape:

- `date.year`
- `date.month`
- `date.day`

The resolver must support the existing format.

For now, keep compatibility with current keys.

Longer-term note:

If future `multi_field` sections may reuse field ids across sections, a section-qualified key space would be safer:

- `<section_id>.<field_id>`
- `<section_id>.<field_id>.<part_id>`

Do not migrate that in this task unless it is necessary. It is broader and should be discussed separately.

## Suggested File-Level Changes

### `src/note.rs`

- update `render_note()` signature to receive `sticky_values`
- make preview/export mode explicit in the API
- update `section_start_line()` to call the appropriate render mode explicitly
- fix `"duration"` -> `"appointment_duration"`
- split header formatting into preview/export-aware helpers
- use the generic resolver instead of direct `get_value()` calls alone
- replace the current completed-only header gate

### `src/app.rs`

- update header `super confirm` to use the shared resolver
- ensure composite fields only auto-confirm when fully resolved

### `src/ui.rs`

- update preview-pane `render_note()` call to pass `sticky_values` and preview mode
- optionally reuse the new resolver if it meaningfully reduces duplication
- do not break the current modal preview behavior unless intentionally consolidating it

### `src/main.rs`

- update clipboard/export `render_note()` call sites to pass `sticky_values` and export mode

### Optional new shared helper module

If useful, add one small shared module for generic `multi_field` resolution.

Recommended if Claude finds the logic starts duplicating between:

- `note.rs`
- `app.rs`
- `ui.rs`

## Practical Resolver Rules

These rules should be followed exactly unless implementation reality forces a small variation.

### Rule A: confirmed beats everything

If the user has already confirmed a field value into `HeaderState.values`, use that exact stored value.

### Rule B: sticky beats default

If there is no confirmed value, prefer sticky over configured default.

### Rule C: composite uses part outputs

When resolving a composite field from defaults, use the part option's `output()`, not its label.

### Rule D: preview placeholders are visual only

Use `--` in preview formatting only.

### Rule E: export omits unresolved content

Never export placeholders or malformed punctuation scaffolding.

### Rule F: super confirm must not commit partial composite values

Partial composite resolution is acceptable for preview, not for auto-confirm.

## Validation

### Automated

Add tests where practical for:

1. simple field resolution:
   - confirmed beats sticky and default
2. composite field resolution:
   - sticky part values assemble correctly
3. export formatting:
   - unresolved fields are omitted, not rendered as `--`
4. preview formatting:
   - unresolved fields show `--`
5. super confirm:
   - simple default field auto-confirms and advances
   - composite sticky/default field auto-confirms only when fully resolvable
   - partial composite field does not auto-confirm

If the existing test surface makes this awkward, prefer targeted unit tests around the resolver helpers rather than large app-level tests only.

### Manual

1. Start with empty or minimal sticky values.
   - Enter the header.
   - Confirm one field.
   - Verify the live preview appears.

2. Fresh unresolved fields in preview.
   - Ensure unresolved header fields show `--` in preview.
   - Ensure clipboard/export output does not contain `--`.

3. Sticky date carry-forward.
   - Complete one note with date `2026-04-02`.
   - Start the next note.
   - Verify preview/header resolution prefers sticky date values.

4. Appointment duration fix.
   - Verify duration resolves from `appointment_duration`, not the broken `duration` key.

5. Super confirm on simple field.
   - On `appointment_duration`, Shift+Enter should use sticky/default value and advance.

6. Super confirm on composite field.
   - On date or appointment type, Shift+Enter should only auto-confirm if the full value is resolvable from sticky/default data.

7. Clipboard/export behavior.
   - Copy a note with a partially entered header.
   - Confirm unresolved header fields are omitted cleanly rather than rendered as placeholders.

## Risks / Watchouts

- Do not accidentally replace preview behavior with export behavior or vice versa.
- Do not use `part.preview` for note export values.
- Be careful with time formatting:
  - resolved raw time may be `13:00` or `1:00`
  - `format_header_time()` still needs to normalize appropriately
- Avoid hard-coding this logic to `HeaderState` only if future `multi_field` reuse is a real goal.
- `render_note()` must not silently default to preview or export mode; the caller should choose explicitly.

## Recommendation Summary

Implement this as a small generalization, not a header-only patch:

- generic `multi_field` resolver
- preview/export separation
- sticky values threaded into note rendering
- super confirm moved onto the same resolver

That gives a clean fix for the current header bugs without reintroducing the same class of problem when more `multi_field` sections are added.
