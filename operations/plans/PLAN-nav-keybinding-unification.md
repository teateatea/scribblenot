# Plan: Navigation Keybinding Unification

**Date:** 2026-04-17
**Status:** Proposed
**Recommended branch:** `nav-keybinding-unification`

---

## Task

Update directional navigation so the app uses one consistent keybinding vocabulary:

- rename `navigate_down` -> `nav_down`
- rename `navigate_up` -> `nav_up`
- rename `focus_left` -> `nav_left`
- rename `focus_right` -> `nav_right`

Then extend those directional bindings so they work in modals the same way arrow keys do now, with one exception:

- while focus is in a modal search bar, raw arrow keys should keep their current behavior
- character bindings like `n`, `e`, `h`, and `i` must remain text input there

Also add a new wizard behavior:

- when focus is on a wizard field that can open a modal, `nav_right` should open that modal
- this is additive; `confirm: [enter]` stays unchanged

---

## Why This Exists

The current behavior is inconsistent in three ways:

1. The naming splits "navigate" and "focus" even though both are really directional movement.
2. Map and wizard already respect the configured directional bindings, but modals still hard-code raw arrow-key behavior in important paths.
3. The wizard already uses leftward movement to back out of modals, so opening a field modal with `nav_right` is the natural symmetric action.

The only place that cannot blindly adopt character aliases is the modal search bar, because `n/e/h/i` are valid text input there.

---

## Decisions Already Made

1. Old key names will break on purpose.
   - No backward-compatibility aliasing.
   - Older `keybindings.yml` files should fail validation instead of silently mapping to the new names.

2. `nav_right` should prioritize modal entry over pane movement.
   - If the current wizard field can open a modal, `nav_right` opens it first.
   - This takes priority over map-pane movement because map navigation is already ergonomic via hint keys.

---

## Goals

1. Standardize directional keybinding names across config, code, docs, and tests.
2. Make modal list navigation respect configured directional bindings, not just raw arrows.
3. Preserve safe text-entry behavior in modal search bars.
4. Add `nav_right` as a wizard-to-modal shortcut.
5. Keep existing confirm and back behavior intact unless this task explicitly changes it.

---

## Non-Goals

- Do not preserve old keybinding field names.
- Do not redesign pane layout or focus order.
- Do not change hint-key navigation behavior.
- Do not change `confirm`, `back`, `select`, or other non-directional bindings.
- Do not broaden the wizard-open shortcut beyond fields that already support modal opening.

---

## Current Facts In The Codebase

- `data/keybindings.yml` still uses:
  - `navigate_down`
  - `navigate_up`
  - `focus_left`
  - `focus_right`
- `src/data.rs` defines the same old field names in `KeyBindings` and in the built-in defaults.
- `src/app.rs` already centralizes keybinding checks through helper methods:
  - `is_navigate_down`
  - `is_navigate_up`
  - `is_focus_left`
  - `is_focus_right`
- Map and wizard handlers mostly use those helpers already.
- Modal handlers do not consistently use them:
  - `handle_modal_key()` still hard-codes `AppKey::Left`, `AppKey::Right`, `AppKey::Up`, and `AppKey::Down` in several branches
  - `handle_collection_modal_key()` also hard-codes arrow-key behavior
- Modal search and modal list are already distinct focus states via `ModalFocus::SearchBar` and `ModalFocus::List`.
- Header fields already open modals with `Enter` via `open_header_modal()`.

---

## Design Rules

### Rule 1: One directional vocabulary

Every config key, helper method, doc example, and test fixture in this task should use:

- `nav_up`
- `nav_down`
- `nav_left`
- `nav_right`

Avoid mixing old and new names in comments or APIs once the rename lands.

### Rule 2: Search-bar text wins over character aliases

When modal focus is `SearchBar`:

- raw arrow keys keep their current navigation behavior
- `n/e/h/i` are treated as ordinary typed characters

This exception is local to modal search focus. Outside modal search, the configured directional bindings should behave normally.

### Rule 3: Wizard `nav_right` opens before it moves panes

When focus is in the wizard and the current field supports modal opening:

- `nav_right` opens the modal

Only if no modal can be opened should normal pane-navigation behavior continue.

### Rule 4: Enter remains the canonical confirm

`confirm: [enter]` stays unchanged. `nav_right` is an extra entry path for modal-backed fields, not a replacement for confirm.

---

## Implementation Plan

### Step 1. Rename the keybinding schema

Update the authored and built-in keybinding names in:

- `data/keybindings.yml`
- `src/data.rs`
- `README.md`
- any validation fixtures or test YAML strings

The new schema should be:

```yaml
nav_down: [down, n]
nav_up: [up, e]
nav_left: [left, h]
nav_right: [right, i]
```

### Step 2. Rename app-side directional helpers

In `src/app.rs`, rename the helper methods and their call sites so the code reads consistently:

- `is_navigate_down` -> `is_nav_down`
- `is_navigate_up` -> `is_nav_up`
- `is_focus_left` -> `is_nav_left`
- `is_focus_right` -> `is_nav_right`

This is mostly a terminology cleanup, but it also makes the modal changes easier to reason about.

### Step 3. Keep the hard break explicit

Do not add serde aliases or migration logic in `src/data.rs`.

Expected consequence:

- an old `keybindings.yml` with the previous field names should fail validation
- the app should not silently reinterpret it as the new schema

The validation test that currently uses an invalid keybindings fixture should be updated so it still exercises a real parse failure under the renamed schema.

### Step 4. Convert standard modal handling to directional helpers

In `src/app.rs::handle_modal_key()`:

- replace raw-arrow modal list movement with `is_nav_up`, `is_nav_down`, `is_nav_left`, and `is_nav_right`
- apply that to:
  - list navigation
  - back-out behavior
  - rightward confirm/advance behavior where appropriate

The intent is that a modal list should respond the same way whether the user presses arrows or `n/e/h/i`.

### Step 5. Apply the same directional routing to collection modals

In `src/app.rs::handle_collection_modal_key()`:

- map left/right/up/down behavior onto the renamed directional helpers
- keep the current collection semantics the same

This should make collection modals match standard modals and the rest of the app.

### Step 6. Preserve the modal search-bar exception

In the `ModalFocus::SearchBar` branch of `handle_modal_key()`:

- keep raw `AppKey::Up` / `Down` behavior as it works today
- keep any raw left/right search-specific behavior unchanged if present
- do not let `is_nav_up/down/left/right` consume character aliases there

Practical effect:

- `down` arrow may leave the search bar for the list
- `n` must type into the query instead
- `i` must type into the query instead of acting like right-arrow confirm

This branch should remain explicit rather than relying on a broad global rule.

### Step 7. Add wizard `nav_right` modal entry

In the header-field wizard flow in `src/app.rs::handle_header_key()`:

- add `nav_right` as another way to call `open_header_modal()`
- keep the current `Enter` path

Priority rule:

- if the current wizard field can open a modal, `nav_right` opens it
- this should run before pane-navigation logic takes effect for that keypress

This should only apply to wizard fields that already support modal opening. It should not invent modal behavior for other section types.

### Step 8. Update docs and keybinding reference text

Update `README.md` so it reflects:

- the new key names
- the directional naming rationale in plain language
- the fact that modal search bars are the one context where arrow keys and character aliases intentionally differ

The checked-in `data/keybindings.yml` should be the canonical example.

### Step 9. Add focused regression coverage

Add or update tests for:

1. Keybinding deserialization and defaults use the renamed fields.
2. Old field names no longer deserialize successfully.
3. Standard modal list navigation works with `n/e/h/i`.
4. Collection modal navigation works with `n/e/h/i`.
5. In modal search focus, arrow keys still perform their current navigation behavior.
6. In modal search focus, `n/e/h/i` are inserted as text rather than triggering navigation.
7. In the wizard, `nav_right` opens a header-field modal.

Prefer targeted app tests around `handle_modal_key()`, `handle_collection_modal_key()`, and `handle_header_key()` rather than broad end-to-end tests only.

---

## Suggested File-Level Changes

### `src/data.rs`

- rename `KeyBindings` directional fields
- rename default helper functions for left/right
- update `Default for KeyBindings`
- update validation tests and keybinding fixtures

### `data/keybindings.yml`

- replace old directional action names with the new ones

### `src/app.rs`

- rename directional helper methods
- replace modal raw-arrow routing where configured directional bindings should apply
- keep the modal-search exception explicit
- add `nav_right` wizard modal opening

### `README.md`

- update keybinding examples
- update action reference table
- mention the modal-search exception clearly

---

## Validation

### Automated

Run:

```powershell
cargo test
cargo run -- --validate-data
```

Expected outcomes:

- tests pass with the new key names
- `keybindings.yml` validation passes with the renamed schema
- old-name fixtures fail where the tests intentionally check for that

### Manual

1. Wizard header field:
   - focus a field with a modal
   - press `i`
   - verify the modal opens

2. Standard modal list focus:
   - use `n/e` to move up and down
   - use `h/i` to back out / confirm where arrow keys currently do that

3. Collection modal:
   - verify `n/e/h/i` mirror the current arrow-key behavior

4. Modal search bar:
   - type `n`, `e`, `h`, `i`
   - verify they appear in the query instead of navigating
   - press arrow keys and verify current search-bar navigation still works

5. Map and wizard baseline:
   - verify renamed directional bindings still navigate as before outside modals

---

## Risks / Watchouts

- The pane-navigation branch currently runs before section-specific wizard handlers, so the `nav_right` modal-open path must be wired carefully or it will never take priority.
- Modal input handling mixes global keybinding helpers with raw `AppKey` matches today; partial conversion could leave behavior inconsistent.
- Search-bar handling is the one context where character aliases must not behave like arrows. That exception should stay obvious in code comments and tests.
- Because old key names are intentionally breaking, docs and the checked-in example must be updated in the same change or the app will become confusing to configure.

---

## Recommendation Summary

Implement this as one focused change set:

- rename the directional schema to `nav_*`
- hard-break the old names
- make modals use directional bindings consistently
- keep modal search bars arrow-only for navigation
- let wizard `nav_right` reopen modal-backed fields

That keeps the behavior model simple: directional bindings work everywhere directional movement makes sense, except inside a modal search bar where typed characters must remain text.
