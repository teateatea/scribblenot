## Task

#73 Phase 1 - Editable document model and iced port

## Goal

Replace the terminal UI with an iced desktop app while changing the source of truth from generated preview text to an editable markdown note document.

This phase is implementation-ready.

## Scope

- replace `ratatui` and `crossterm` with `iced`
- introduce editable document state
- preserve existing section state machines as structured input tools
- make structured actions update the editable note instead of regenerating the entire note on copy
- keep copy/export behavior out of scope for this phase except where needed to support the new document model

## Core Architecture

The desktop app uses two linked representations:

- structured state: the existing `App`, section configs, and section state machines
- editable document: a markdown string stored in `App::editable_note`

The editable document is the user-visible source of truth.

Structured actions are allowed to update only their own anchored section content inside that document.

## Anchor Model

This is the missing contract that must exist before implementation.

Top-level canonical headings remain stable:

- `## SUBJECTIVE`
- `## TREATMENT / PLAN`
- `## OBJECTIVE / OBSERVATIONS`
- `## POST-TREATMENT`

But top-level headings are **not** enough for safe section replacement because multiple runtime sections share each heading in [note.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/note.rs).

Therefore the editable document model uses two anchor levels:

1. top-level canonical headings, for coarse validation
2. per-section subheadings, for targeted replacement

Per-section replacement contract:

- each structured runtime section that can be updated independently must own a unique rendered subheading
- if a section already renders under a unique `####` heading, that heading is its anchor
- if a section currently renders as free body text inside a shared top-level section, Phase 1 must introduce a stable per-section subheading for it before that section becomes editable through structured actions
- manual edits inside a section body are allowed and preserved
- missing, renamed, or duplicated anchors must set `note_headings_valid = false` and block targeted overwrite until repaired

This is more important than minimizing markdown changes. Safe reconciliation matters more than matching the old renderer byte-for-byte.

## Required App Changes

In [app.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/app.rs):

- add `editable_note: String`
- add `note_headings_valid: bool`
- add `show_window: bool`
- add `clipboard_import: Option<String>`

Initialize `editable_note` from a one-time document builder on startup.

After startup, do not treat `render_note()` as the export authority.

## Required New Module

Add `src/document.rs`.

Required helpers:

- `build_initial_document(...)`
- `parse_document_headings(...)`
- `validate_canonical_headings(...)`
- `find_section_bounds(...)`
- `replace_section_body(...)`
- `repair_document_structure(...)`

Expected behavior:

- `build_initial_document(...)` creates the initial editable markdown from current structured state
- `find_section_bounds(...)` locates a section by its stable anchor
- `replace_section_body(...)` updates only that section body
- `validate_canonical_headings(...)` detects missing, renamed, or duplicate anchors
- `repair_document_structure(...)` restores required anchors without silently discarding unrelated manual edits outside the damaged section when feasible

## Input Migration

Removing `crossterm` breaks nearly all key handling in [app.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/app.rs).

Define:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AppKey {
    Char(char),
    CtrlChar(char),
    Enter,
    ShiftEnter,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Backspace,
    Tab,
}
```

Then migrate:

- `match_binding_str`
- `matches_key`
- all `is_*` helpers
- `handle_key`
- `handle_map_key`
- `handle_modal_key`
- `handle_header_key`
- `handle_free_text_key`
- `handle_list_select_key`
- `handle_block_select_key`
- `handle_checklist_key`
- `try_navigate_to_map_via_hint`
- the existing `#[cfg(test)]` keybinding tests

Add a conversion function from iced keyboard events to `AppKey`.

## UI Port

Replace [ui.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/ui.rs) with iced `view()` logic.

High-level mapping:

- map pane: left or right navigation pane
- wizard pane: structured controls
- preview pane: real editable note widget bound to `editable_note`

The preview pane is now an editor, not a rendered paragraph.

Keyboard-driven structured actions continue to delegate through `app.handle_key(AppKey::...)`.

Direct editor changes mutate `editable_note` directly.

## Main Bootstrap

Replace [main.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/main.rs) terminal bootstrap with iced application startup.

Also add:

- `SCRIBBLENOT_HEADLESS=1` fast-start path
- initial hidden window setting
- app wrapper type for subscriptions and future tray/hotkey wiring

## Theme Migration

Replace [theme.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/theme.rs) with iced-compatible color constants.

The semantic palette (ACTIVE, SELECTED, HINT, MODAL, MUTED, ERROR, DISPLACED) is worth preserving - rename nothing.

What changes:

- replace `ratatui::style::{Color, Modifier, Style}` imports with `iced::Color`
- keep all `pub const` color definitions, converting `Color::Yellow` etc. to `iced::Color` equivalents
- remove all `pub fn` style helpers (`active()`, `active_bold()`, `selected()`, etc.) - iced does not use a `Style` struct; color is applied per-widget via `.style()` or direct color arguments
- any usage of `theme::active()` in the new `view()` logic must be replaced with direct `theme::ACTIVE` constant references

Do not rename the module. Callers in the new `view()` function should use the color constants directly.

## Modal Rendering

[modal.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/modal.rs) contains only state (`SearchModal`, `CompositeModal`, `ModalFocus`, `CompositeAdvance`) and has no ratatui imports. No changes required to the state module itself.

The modal *rendering* currently lives in [ui.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/ui.rs) as an overlay drawn over the terminal layout. That rendering must be ported to iced.

Replacement approach for the modal overlay in `view()`:

- when `app.modal` is `Some(SearchModal)`, render an iced `container` centered in the window
- the container holds a `column` with: a `text_input` (search bar, bound to `SearchModal::query`), then a `scrollable` `column` of item rows
- each item row is a `button` that emits a select message; the row at `list_cursor` is highlighted using `theme::SELECTED`
- `ModalFocus::SearchBar` vs `ModalFocus::List` controls which widget receives keyboard input
- when `app.modal` is `None`, render the normal three-pane layout

The composite progression (`CompositeModal`, `CompositeAdvance`) is handled by the existing `app.rs` logic; the iced view only needs to show the current part prompt and the search results for that part.

No new state is required in `modal.rs` for this phase. The rendering is entirely in `view()`.

## Dependencies

Update [Cargo.toml](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/Cargo.toml):

```diff
- ratatui = "0.29"
- crossterm = "0.28"
+ iced = { version = "0.13", features = ["tokio", "multi-window"] }
```

Do not add tray or chord crates in this phase unless needed by a compile boundary in the next approved phase.

## Verification

Manual checks:

- app builds and launches into iced without terminal setup
- editable note pane shows initial document
- manual edits in one section persist after structured edits in another section
- deleting or renaming a required anchor shows invalid-structure warning instead of silent overwrite
- Shift+Enter still triggers the intended structured action

Automated checks:

- existing `app.rs` keybinding tests updated and passing
- new unit tests for `find_section_bounds`
- new unit tests for `replace_section_body`
- new unit tests for heading validation and repair

## Exit Criteria

This phase is complete when:

- the app runs under iced
- `editable_note` is the active source of truth
- targeted section replacement is safe
- missing anchors surface as a repair problem, not a silent mutation problem
