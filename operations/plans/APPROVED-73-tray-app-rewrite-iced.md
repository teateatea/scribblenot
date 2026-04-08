## Task

#73 - Rewrite Scribblenot as a native iced desktop tray app

## Status

User Approved for phased implementation.

This rewrite is approved as a **split plan**, not as one monolithic implementation pass.

## Why This File Was Split

The original approved plan mixed three different things:

- implementation-ready desktop rewrite work
- document-model architecture decisions
- an unresolved policy decision around always-on global chord capture

That made it sound more implementation-ready than it really was.

The new structure keeps implementation clear:

- [APPROVED-73-1-document-model-and-iced-port.md](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/operations/plans/APPROVED-73-1-document-model-and-iced-port.md)
- [APPROVED-73-2-tray-hotkey-clipboard-import.md](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/operations/plans/APPROVED-73-2-tray-hotkey-clipboard-import.md)
- [DISCUSSION-73-global-chords-policy.md](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/operations/plans/DISCUSSION-73-global-chords-policy.md)

## Implementation Order

1. Implement Phase 1: document model plus iced port.
2. Implement Phase 2: tray, hotkeys, clipboard import, packaging checks.
3. Do **not** implement global chords until the policy document is explicitly resolved.

## Key Decision

The desktop app's source of truth is the editable note document, not `render_note()` on copy.

That requires a stronger anchor model than the current terminal renderer uses. The approved implementation documents below define that model explicitly.

## Go / No-Go

- Ready now:
  - editable document model
  - iced application shell
  - tray icon
  - show/hide hotkey
  - copy-and-close hotkey
  - clipboard import banner
  - heading validation and repair flow
- Not ready yet:
  - always-on global chord capture via `rdev`

## Notes For The Implementation Instance

- Treat this file as the index.
- Follow the numbered phase docs in order.
- If there is a conflict, the phase doc is more specific than this index.
- The chords document is intentionally not implementation-approved.
