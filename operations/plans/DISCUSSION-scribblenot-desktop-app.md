# Discussion: Scribblenot Desktop App

**Date:** 2026-04-03
**Status:** Discussion Complete

## Problem

Scribblenot currently runs in a terminal via `cargo run`, which is too much friction for a clinician mid-workflow. The user needs to capture and format patient notes quickly — stopping to open a terminal breaks focus. There is no way to invoke the tool with a hotkey, collapse it out of the way, or pull in existing note text for editing. The terminal also prevents Shift+Enter and other keyboard behaviors the user relies on.

## Goal

A real, distributable desktop application that a clinician can invoke instantly via hotkey, fill in or edit a patient note using ergonomic chord shortcuts, and dismiss — leaving a fully formatted note on the clipboard ready to paste. Quality bar is high enough to share with colleagues.

## Core Concept

Scribblenot runs as a background tray application. A global hotkey brings up the note window instantly. The user types shorthand or chord sequences (e.g. `arstob` expands the Objective section) to build a note. The rendered note is directly editable. A close-and-copy hotkey puts the full note on the clipboard and hides the window. If the user highlights text in another app before invoking scribblenot, that text lands in the editable note pane as a starting point. Global chords can also fire section expansions and open the app in one action.

## Context & Users

Primary user is a clinician (the developer) who needs to document patient encounters quickly without breaking focus. The workflow is: highlight existing note text (optional) → hotkey to open scribblenot → build/edit note using chords and Shift+Enter → hotkey to copy and close → paste into clinic system. Long-term goal is to distribute a polished binary to colleagues.

## Constraints & Anti-goals

- **HIPAA compliance is non-negotiable**: no patient data is saved to disk, logged, or transmitted anywhere. Clipboard is the only output.
- No cloud sync of any kind.
- No note history or session persistence.
- No multi-user or account system.
- No AI-generated text. All content comes from the user or their YAML templates.
- No section-level copy (full note only, for now — polish later).
- Startup must be near-instant; startup latency is a hard UX requirement.
- Windows-first; Linux compatibility is a plus but not a near-term requirement.
- The core logic stays in Rust. The GUI layer may use a non-Rust frontend (e.g. Tauri) if it serves speed and packaging goals.

## Key Decisions

- **Editable preview**: The rendered note pane is directly editable, not just a read-only render of shorthand input. This is a shift from the current terminal model.
- **Highlight-to-import**: Detected via clipboard snapshot on open (capture system selection before invoking app), pre-filled into the editable note pane.
- **Global chords**: Option (a) — firing a chord like `arstob` system-wide should both open scribblenot and expand that section template. The user had not previously considered this was possible; it is directionally correct but not fully specified.
- **Custom themes**: In scope (user explicitly wants this).
- **Shift+Enter**: Must work in the note input; terminal currently cannot support it.

## Open Questions

- Which GUI framework best satisfies instant startup + system tray + global hotkeys + Windows/Linux? Main candidates: Tauri (Rust backend + web frontend), egui (pure Rust immediate-mode), iced (pure Rust retained-mode). Propose-plan should evaluate and recommend.
- How does highlight-to-import detect the selected text? Clipboard snapshot before open is the likely approach but has edge cases (clipboard already has something important). Plan should address.
- Global chord implementation: a background keyboard hook vs. integrating with an existing tool like espanso. Plan should evaluate the trade-offs (reliability, HIPAA implications of a keylogger-adjacent hook, etc.).
- Exact hotkey scheme: which keys trigger open, close-and-copy, and section chords? User has preferences from espanso but these need to be defined and configurable.
- Packaging: how is the app distributed? Single `.exe`, NSIS installer, or something else?
