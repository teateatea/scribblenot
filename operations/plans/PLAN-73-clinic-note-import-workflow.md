## Task

Define the clipboard-import workflow for temporary clinic-note editing in the iced tray app.

## Purpose

This plan describes the real user flow behind Phase 2 clipboard import:

- copy an existing patient note from the clinic system
- open Scribblenot
- import the note into a temporary working session
- update or append to the note using Scribblenot
- copy the updated note back into the clinic system

The note content is patient-specific and must not be persisted by Scribblenot.

## Product Intent

Clipboard import is not a saved-draft feature.

Clipboard import is not a general markdown parser.

Clipboard import is a temporary adoption flow for existing plain-text clinic notes that look enough like Scribblenot output to be edited safely.

## Core Rule

Imported clinic-note text is external user content.

Scribblenot may convert that text into its managed editable-document format for in-app editing, but it must treat the clipboard text as untrusted plain note content rather than as an internal document blob.

## User Workflow

1. The user copies a prior patient note from the clinic system.
2. The user opens Scribblenot from the tray or hotkey.
3. Scribblenot inspects the clipboard.
4. If the clipboard looks like a clinic note with recognizable structure, Scribblenot offers import.
5. The user chooses whether to import.
6. On acceptance, Scribblenot creates a temporary in-memory editing session from that note.
7. The user edits the note and may also use structured tools where anchors can be mapped safely.
8. The user presses the copy-and-close hotkey.
9. Scribblenot copies plain note text back to the clipboard and hides.
10. The user pastes the updated note into the clinic system.

## Privacy Boundary

Patient note content must be ephemeral.

Allowed persistence:

- product YAML
- user preferences such as hotkeys, theme, and layout

Disallowed persistence in this phase:

- imported patient note text on disk
- autosave of imported notes
- patient content in config files
- patient content in roadmap notes, logs, fixtures, or debug output

## Format Model

There are two note forms:

1. external plain note text
2. internal managed editable document

External plain note text is what the user copies from and back to the clinic system.

Internal managed editable document is what Scribblenot uses to support safe targeted section updates.

The import pipeline converts from plain note text to managed editable document.

The export pipeline converts from managed editable document back to plain note text.

## Import Pipeline

1. Detect likely note text on the clipboard.
2. Identify recognizable headings or per-section anchors.
3. Partition the plain note text into known sections where possible.
4. Rebuild Scribblenot managed markers around recognized sections.
5. Preserve unmatched text where feasible.
6. Validate the managed editable document before making it active.
7. If validation fails or mapping is too ambiguous, refuse the import rather than guessing.

## Editing Rules After Import

- manual edits in the note editor remain the source of truth
- structured actions may update only sections with valid managed anchors
- unmatched or unparsed text must not be silently dropped
- invalid structure must surface a warning and block unsafe targeted overwrite

## Copy-Out Behavior

- copy-out uses user-facing plain note text
- internal managed markers are never exposed to the user clipboard
- copy-and-close hides the window after successful copy attempt

## Suggested Implementation Slices

### Slice 1: Detection and Offer

- add clipboard inspection on tray-open and hotkey-show
- detect likely clinic-note text by headings and anchor count
- show a dismissible import banner

### Slice 2: Temporary Session State

- store pending import text in memory only
- track whether the active note originated from clipboard import
- ensure session note text is cleared on app restart

### Slice 3: Conversion

- add `src/import.rs`
- parse recognized headings from plain note text
- construct the managed editable-document form
- validate before replacing the current editor content

### Slice 4: Safe Structured Editing

- keep manual editor text authoritative
- only enable section sync when managed anchors are valid
- surface warnings when imported structure is incomplete or damaged

### Slice 5: Export and Privacy Checks

- keep export marker-free
- confirm no patient note text is written to disk
- verify copy-and-close uses exported text, not raw internal text

## Validation

Manual checks:

- import banner appears only when clipboard text resembles a clinic note
- choosing import replaces the working note only after explicit confirmation
- imported text can be edited immediately
- imported text can be copied back out as clean plain note text
- imported text is not restored after app restart

Automated checks:

- unit tests for note detection heuristics
- unit tests for import conversion on representative clinic-note samples
- unit tests that exported text strips managed markers
- startup test confirming the app can still boot headless quickly

## Risks

### Risk 1: over-eager detection

If detection is too loose, random clipboard text could trigger import prompts.

Mitigation:

- require at least two recognized structural anchors
- prefer false negatives over false positives

### Risk 2: unsafe section mapping

If import conversion guesses section boundaries badly, structured tools could overwrite user text.

Mitigation:

- map only clearly recognized sections
- preserve unknown text
- block structured sync when anchors are incomplete

### Risk 3: privacy leakage

If imported note text is autosaved or logged, the feature violates the intended privacy boundary.

Mitigation:

- keep imported note text memory-only
- avoid debug logging of clipboard contents
- test restart behavior explicitly

## Recommendation

Implement this as a minimal safe workflow:

- detect likely clinic-note text
- offer explicit import
- convert only clearly recognized structure
- keep imported content ephemeral
- export clean plain note text back to the clipboard

That is enough to support the real clinic workflow without prematurely designing a full saved-draft or full-parser system.
