# Project Foundation

## Goals
This mission implements the full multi-character hint system for scribblenot's map column navigation, from permutation generation through progressive input filtering, plus ergonomic group-jump hotkeys and a super-confirm keybinding for the wizard. Together these changes make the hint system scale to any number of sections and reduce keystrokes for common confirmation workflows.

## Requirements
- Auto-generate multi-character hint permutations (2-char, 3-char+) from the base hint characters in keybindings.yml and persist them as a `hint_permutations:` field; regenerate only when the `hints:` field changes
- Permutations must be priority-ordered by adjacency in the base list (adjacent pairs first, distant pairs last) up to the count needed to cover all sections
- Multi-character hints consume their prefix: any character sequence that is a proper prefix of an existing multi-char hint cannot itself be assigned as a standalone hint
- Implement a hint-input state machine: typing the first character of a multi-char hint highlights matching prefixes in white and grays out non-matching hints; an unresolvable keypress resets all hints to normal active state
- Add group-jump hotkeys hardcoded in source (Q=Intake, W=Subjective, F=Treatment, etc.) that are always active when focus is on the map column and are excluded from the section-hint pool
- Add a Shift+Enter super-confirm keybinding (configurable in keybindings.yml) that confirms the currently focused wizard field using its visible value (typed, sticky, or default); if the field is empty it is skipped; Shift+Enter confirms only the focused field and the wizard advances normally

## Task Priority Order
- #23 - Auto-generate multi-character hint permutations from base hint characters for overflow assignment
- #22 - Implement multi-character hint sequences with progressive prefix filtering
- #21 - Add persistent group-jump hotkeys in map column
- #2 - Add Shift+Enter super-confirm keybinding to auto-complete remaining fields

## Explicit Non-Goals
- Group-jump hotkey assignments must not be user-configurable via keybindings.yml
- Permutations must not be regenerated on every startup if the `hints:` field has not changed
- The hint state machine must not affect single-character hints or any behavior outside the map column hint system
- Super-confirm must not auto-advance through all remaining wizard fields in a single press; it confirms only the currently focused field

## Constraints
- `hint_permutations:` is written back to keybindings.yml as a persistent field; it is regenerated only when `hints:` changes
- Multi-char hint prefix exclusion rule: if hints `zz` and `zx` exist, `z` can never be a standalone hint
- Group key assignments (Q, W, F, etc.) are hardcoded in source; never read from keybindings.yml
- Shift+Enter super-confirm applies WYSIWYG: if the focused field has no typed value, no sticky, and no default, it is skipped rather than erroring
- Group-reserved characters must be excluded from the section-hint pool

## Test Criteria
- Given a `hints:` list smaller than the number of sections, `hint_permutations:` is populated in keybindings.yml on first run and is not rewritten on subsequent runs unless `hints:` is edited
- Permutations appear in adjacency-priority order (e.g. `qq, qw, wq, ww` before `qp`)
- No hint in `hint_permutations:` shares a value with any prefix of another multi-char hint
- Typing the first character of a multi-char hint grays out non-matching hints and highlights the matching prefix in white; typing an unresolvable second character resets all hints to active magenta
- Single-character hints resolve immediately with no buffering delay
- Pressing Q/W/F (or other group keys) while focused on the map column jumps to the first section of the corresponding group regardless of current cursor position
- Group-jump keys never appear as section hints
- Shift+Enter in the wizard confirms the currently focused field with its visible value and advances the wizard; an empty field is skipped silently
- Shift+Enter binding is present and functional when configured in keybindings.yml

## Coordination
- READY: false
- BEGIN: true
