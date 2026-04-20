# Plan: Explicit Single-Character Hotkeys For Hinted Entities

**Date:** 2026-04-17
**Status:** Proposed
**Recommended branch:** `entity-hotkeys`

---

## Task

Add optional authored `hotkey:` support for the hint-bearing data types the user wants to target directly:

- sections in the map
- header fields in the wizard
- list items in modals

The author should be able to write:

```yaml
hotkey: "g"
```

and have that entity claim the `g` label in its local hint scope.

This is an explicit, single-character override of generated hint assignment.

---

## Why This Exists

Current hint labels are generated centrally from the shared hint pool in `keybindings.yml`.

That works well for generic navigation, but it does not let the author pin a memorable key to a clinically important section, field, or item. For repeated, frequently used actions, authored mnemonic hotkeys would be faster and more learnable than remembering whichever generated label happens to appear today.

The desired product rule is:

- text entry still behaves like text entry
- explicit authored hotkeys win over generated hint labels
- generated hint labels fill the remaining open slots

---

## Current Hint Inventory

Hints are currently used in these places:

1. Map section rows.
   - Rendered in `src/ui/mod.rs` via `map_pane()`.
   - Resolved in `src/app.rs::handle_map_key()`.

2. Header-field wizard rows.
   - Rendered in `src/ui/mod.rs` via `render_header_state()`.
   - Resolved in `src/app.rs::handle_header_key()`.

3. Standard modal list rows.
   - Rendered in `src/ui/mod.rs` active/preview modal list paths.
   - Resolved in `src/app.rs::handle_modal_key()`.

4. Collection-modal rows.
   - Collection names in the left pane.
   - Collection items in the right pane.
   - Resolved in `src/app.rs::handle_collection_modal_key()`.

Hints are not currently used for:

- free-text section entries
- checklist rows
- list-select wizard rows

### Scope Decision For This Plan

This plan covers:

- `HierarchySection`
- `HierarchyField`
- `HierarchyItem`

That matches the requested v1 behavior: section, field, and list item hotkeys.

This plan does **not** add authored hotkeys to collection names themselves (`HierarchyCollection` / `ResolvedCollectionConfig`), even though collection-name rows currently use generated hints. That can be added later if desired.

---

## Decisions Already Made

1. `hotkey:` is single-character only.
   - No multi-character authored hotkeys in this pass.

2. Text-entry contexts win over hotkeys.
   - Search boxes, manual composition editing, and any other real text-entry path must keep treating printable characters as text.

3. Explicit hotkeys outrank generated hints.
   - If an entity has an authored hotkey, that label is reserved for it in the current local scope.

---

## Recommendation

Treat `hotkey:` as a local reservation, not just a tie-breaker.

That means:

- explicit hotkeys are assigned first
- generated labels skip those reserved characters
- the UI never shows two visible rows with the same label in the same active scope

This is safer than allowing duplicate visible labels and relying on hidden precedence rules.

---

## Goals

1. Let authors pin memorable keys to sections, header fields, and list items.
2. Keep the existing generated-hint system for everything without an authored override.
3. Preserve current text-entry behavior.
4. Make collisions fail clearly during data validation rather than becoming runtime surprises.
5. Keep the behavior explainable from the YAML.

---

## Non-Goals

- Do not redesign the whole hint system.
- Do not add global app hotkeys.
- Do not add multi-character authored hotkeys.
- Do not add `hotkey:` to collection names in this pass.
- Do not add authored hotkeys to free-text, checklist, or list-select rows.

---

## Proposed Authored Schema

Add optional `hotkey:` fields to:

### `HierarchySection`

```yaml
sections:
  - id: subjective_section
    label: Subjective
    nav_label: Signs/Symptoms
    hotkey: "s"
```

### `HierarchyField`

```yaml
fields:
  - id: requested_regions
    label: Requested Regions
    hotkey: "r"
```

### `HierarchyItem`

```yaml
lists:
  - id: region
    items:
      - id: glutes
        label: Glutes
        output: Glutes
        hotkey: "g"
```

Important authoring note:

- string-shorthand items like `- "Glutes"` cannot carry metadata
- any item that needs `hotkey:` must use object form

---

## Resolution Rules

### Rule 1: Scope is local

An authored hotkey only needs to be unique within the active sibling set that currently resolves hints.

Examples:

- section hotkeys must be unique among sections
- field hotkeys must be unique among visible header-field targets in the current section
- item hotkeys must be unique among the currently active modal row set

The same character may be reused in unrelated scopes.

### Rule 2: Text entry always wins

When focus is in:

- modal search input
- free-text editing
- manual composition editing
- any future real text input

printable keys remain text, not hotkey triggers.

### Rule 3: Mode commands still win

Authored entity hotkeys should not silently steal existing command bindings like `n`, `e`, `s`, `t`, `h`, or `i` when those bindings are active in that mode.

If a configured entity hotkey collides with an active command binding for that mode, validation should reject it.

### Rule 4: Explicit beats generated

After command bindings and text-entry exceptions are accounted for:

- explicit `hotkey:` labels are assigned first
- generated labels are assigned to the remaining unclaimed rows

### Rule 5: Matching follows existing hint normalization

`hotkey:` should go through the same case-folding/display rules as current hints:

- `hint_labels_capitalized` still controls display
- `hint_labels_case_sensitive` still controls matching behavior

That preserves current user-configurable hint semantics instead of creating a parallel rule set.

---

## Special Cases

### Repeating header fields

Current wizard hints target visible rows, not just field definitions. Repeating fields can therefore produce multiple visible rows for one authored field.

Recommended v1 rule:

- `field.hotkey` applies to the field as a whole
- pressing it jumps to that field and opens its modal at the field's current active repeat slot
- only the active visible row for that field displays the explicit hotkey
- other visible repeat rows for that same field keep generated labels or stay unlabeled, depending on final rendering choice

This avoids showing the same authored label on multiple rows at once.

### Modal windowing

Current modal hints are window-local: only the visible row window gets labels.

Recommended v1 rule:

- item hotkeys participate only when their row is in the currently visible modal window

That keeps explicit hotkeys aligned with the current visible hint model and avoids selecting off-screen rows with invisible labels.

### Collection modal

`HierarchyItem` hotkeys should work for item rows that appear inside collection modals, because those are still item-backed rows.

This plan does not add authored hotkeys to collection-name rows in the left pane.

---

## Data Validation Rules

Validation should reject:

1. missing or empty `hotkey` values
2. strings longer than one character
3. duplicate explicit hotkeys within one active local scope
4. explicit hotkeys that collide with active command bindings in that mode
5. explicit hotkeys that are not part of the configured base hint pool, if the implementation chooses to keep the visible label set constrained to `keybindings.hints`

### Recommendation On Rule 5

Prefer requiring explicit hotkeys to come from the configured base hint pool.

Pros:

- keeps the visible label vocabulary coherent
- avoids inventing ad hoc labels outside the authored hint alphabet
- makes conflicts with generated labels easier to reason about

Cons:

- slightly reduces author freedom

Recommendation:

- require membership in `keybindings.hints` for v1

That keeps the feature simpler and more predictable.

---

## Implementation Outline

### Step 1. Extend authored schema types

Add optional `hotkey: Option<String>` to:

- `HierarchySection`
- `HierarchyField`
- `HierarchyItem`

Thread the data through any runtime structs that need direct access during UI hint assignment.

### Step 2. Centralize local hint assignment

Introduce a small helper layer that, given:

- the local row set
- any authored hotkeys
- the base generated hint pool

returns the final visible labels for that scope.

This should replace ad hoc "take the first N hints" behavior in:

- map sections
- wizard header fields
- standard modals
- collection-item modal panes where item-backed rows are shown

### Step 3. Add validation

During data load, validate:

- single-character shape
- duplicate local hotkeys
- command-binding collisions
- any base-pool membership rule

Make the loader error text explain the exact entity ID and the conflicting key.

### Step 4. Update resolution paths

Change input handling so each hint-bearing surface resolves against its computed visible labels instead of assuming the visible labels are always just the first N generated hints.

### Step 5. Update rendering

Use the same computed label set in rendering so the UI always matches the actual input behavior.

### Step 6. Add tests

Add focused tests for:

- section hotkey reserves a label and beats generated assignment
- field hotkey jumps correctly, including repeating-field behavior
- item hotkey selects the intended visible row
- duplicate hotkeys fail validation
- command-binding collisions fail validation
- text-entry contexts ignore authored hotkeys and keep treating characters as text

### Step 7. Update docs

Update:

- `README.md`
- YAML authoring reference comments in `data/sections.yml` if that file remains the project example

Document:

- supported entity types
- single-character limit
- shorthand-item limitation
- collision rules

---

## Open Product Question

For repeating header fields, should non-active visible repeat rows of a hotkeyed field:

1. keep generated row hints
2. show no hint label

Recommendation:

Choose option 1.

Pros:

- preserves row-level discoverability
- avoids reducing navigability in long repeating fields

Cons:

- mixes one explicit field-level hotkey with generated row hints in the same field block

This is still the better tradeoff for v1 because it keeps existing row navigation power intact while adding the authored mnemonic.

---

## Manual Verification

1. Add `hotkey:` to a section and confirm the map shows that label and navigation chooses that section.
2. Add `hotkey:` to a header field and confirm the wizard shows that label and opens the correct field.
3. Add `hotkey:` to an item in a list modal and confirm pressing the key selects that visible row.
4. Add two duplicate hotkeys in the same local scope and confirm data loading fails with a clear error.
5. Add a hotkey that collides with an active command binding for that mode and confirm validation rejects it.
6. In modal search, type the same character as a configured item hotkey and confirm it enters search text instead of selecting the row.

---

## Expected Outcome

After this work, the app will still feel like the current hint-driven Scribblenot, but authors will be able to pin stable mnemonic keys onto the most important sections, fields, and items without breaking text entry or creating invisible precedence rules.
