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
- The core logic stays in Rust. No web-based GUI (Tauri excluded) -- too much frontend pixel-pushing. Pure Rust GUI only (egui or iced).

## Key Decisions

- **Editable preview**: The rendered note pane is directly editable, not just a read-only render of shorthand input. This is a shift from the current terminal model.
- **Highlight-to-import**: Since the app runs continuously in the tray, it can capture the clipboard at the moment the open-hotkey fires. That clipboard snapshot pre-fills the editable note pane. Edge case (user's clipboard already has unrelated content) to be resolved in planning.
- **Global chords**: Option (a) -- firing a chord like `arstob` system-wide should both open scribblenot and expand that section template. The chord detector maintains only a rolling buffer of the last 6-10 keypresses and never persists anything; user believes this does not constitute a HIPAA violation (no patient data stored, no logging). Plan should confirm.
- **HIPAA keylogger boundary**: A minimal rolling keypress buffer (6-10 chars, never written to disk, never transmitted) is considered acceptable. Plan must verify this interpretation is safe.
- **Custom themes**: In scope (user explicitly wants this).
- **Shift+Enter**: Must work in the note input; terminal currently cannot support it.

## Open Questions

- **GUI framework**: Tauri ruled out (web frontend). Candidates are egui (immediate-mode, very fast, simpler) and iced (retained-mode, more polished, closer to traditional UI feel). User leans toward iced but is not well-informed yet. Propose-plan should benchmark startup time and tray/hotkey support for both and make a recommendation.
- **Clipboard edge case on import**: If the user's clipboard already holds something important when they open scribblenot, the snapshot approach could silently discard it. Plan should propose a safe resolution (e.g. only import if clipboard contains plain text that looks like a note, or always prompt).
- **Chord implementation approach**: Rolling in-process keypress buffer vs. delegating to espanso. Given the user already uses espanso, integration may be natural -- but an in-process hook gives more control and avoids a runtime dependency. Plan should weigh.
- **Exact hotkey scheme**: TBD during development. Should be fully configurable.
- **Packaging and distribution**: Informed by framework choice. Single `.exe` is the minimum; installer (e.g. NSIS or WiX) for colleague distribution. Plan to address after framework is selected.
