# Project Foundation

## Goals
Mission 8 delivers keyboard navigation improvements to scribblenot's map column and wizard. The mission adds persistent group-jump hotkeys, auto-generated multi-character hint permutations written to keybindings.yml, a progressive prefix-filtering state machine for multi-character hints, and a Shift+Enter super-confirm keybinding that auto-advances through remaining wizard fields using already-confirmed, sticky, and default values.

## Requirements
- When focus is on the map column, a fixed set of reserved keys (e.g. Q, W, F) must always jump to the first section of the corresponding group, regardless of current cursor position
- Reserved group-jump characters must be derived from keybindings.yml and must be excluded from the section-hint pool so no hint is ever assigned a conflicting character
- When the base hint pool is smaller than the number of hints needed, 2-char (and higher) permutations must be generated using n^r logic and written to the hints_permutations section of keybindings.yml; permutations are priority-ordered by adjacency in the base list
- Permutation generation only rewrites keybindings.yml when the hints section has changed; otherwise the existing permutations list is used as-is
- Typing the first character of a multi-char hint must activate a prefix-filtering state: matching hints highlight their typed prefix in white; non-matching hints gray out; a subsequent keypress resolves or resets
- Any non-matching keypress (Escape, Backspace, arrows, Enter, or any key with no remaining match) cancels the hint buffer and returns all hints to normal active state; only a complete valid multi-char sequence navigates
- Single-character hints are unaffected by the prefix-filtering state machine
- Shift+Enter in any wizard field triggers super-confirm: it confirms all remaining fields using already-confirmed (green) values first, then sticky values, then default values, skipping user interaction for fields with a valid answer
- If a field has no confirmed, sticky, or default value, super-confirm skips it (WYSIWYG)
- Shift+Enter must be configurable via keybindings.yml

## Task Priority Order
- #21 - Add persistent group-jump hotkeys in map column
- #23 - Auto-generate multi-character hint permutations
- #22 - Implement multi-character hint sequences with progressive prefix filtering
- #2 - Add Shift+Enter super-confirm keybinding

## Explicit Non-Goals
- Group-jump hotkeys must NOT affect the wizard column or any column other than the map column
- The hint permutation generator must NOT operate in-memory only; results must be written to keybindings.yml
- Super-confirm must NOT attempt to infer or guess a value for a field that has no confirmed, sticky, or default value; it skips silently
- Do not change single-character hint rendering or behavior as part of the multi-char state machine work
- Do not add any new UI chrome, modals, or settings screens

## Constraints
- Reserved group-jump characters must be read from keybindings.yml at runtime, not hardcoded in source
- Permutation adjacency ordering must follow the order characters appear in the hints list in keybindings.yml (not alphabetical or arbitrary)
- keybindings.yml must only be rewritten by the permutation generator when the hints section has actually changed, to avoid unnecessary file churn
- The hint prefix-filter state machine must treat Escape, Backspace, arrow keys, and Enter as cancellation triggers in addition to any unmatched character
- The super-confirm flow is strictly WYSIWYG: confirmed > sticky > default, in that precedence order, with no fallback guessing
- The wizard always has a focused field, so super-confirm always has a specific target; the field-skipping rule applies only to fields with no resolvable value

## Test Criteria
- Pressing a group-jump key (e.g. Q/W/F) while focus is in the map column moves the cursor to the first section of the corresponding group; pressing the same key while focus is in the wizard or any other column has no effect
- No section hint is ever assigned a character that appears in the group-jump key set
- After modifying the hints list in keybindings.yml, running the app regenerates the permutations section; running the app again without changing hints does not rewrite the file
- Typing a valid multi-char hint sequence (e.g. z then x for "zx") navigates to the correct section; typing z then an unmatched key resets all hints to normal magenta active state
- Pressing Shift+Enter in the wizard with some fields already confirmed/sticky auto-completes all resolvable remaining fields without showing their modals; fields with no resolvable value are silently skipped
- All existing single-character hint navigation continues to work without regression

## Coordination
- READY: false
- BEGIN: true
