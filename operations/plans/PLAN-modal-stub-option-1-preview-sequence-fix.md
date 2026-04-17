# Plan: Modal Stub Option 1 - Preview Sequence Fix

**Date:** 2026-04-17
**Status:** Proposed
**Scope:** Focused fix
**Related Issue:** repeat-joiner list (`obmuscle_field` / `muscle`) renders a right confirm stub even when real field flow continues to downstream lists

## Purpose

Fix the immediate UI mismatch where the modal stream shows a green confirm stub (`+`) on a list that is not actually terminal from the user's point of view.

This plan keeps the existing modal architecture intact. It only changes how the preview sequence is derived for the modal stream.

## Plain-English Summary

Today, the stub renderer decides whether the right edge is `>` or `+` based on whether the preview system can synthesize a next modal snapshot.

That shortcut breaks for repeat-joiner lists like `muscle` in `obmuscle_field`:

- the real field flow can still continue to `place`
- but the preview builder refuses to synthesize downstream previews for repeat-joiner lists
- so the stream thinks `muscle` is the last previewable modal
- that makes the right stub render as confirm (`+`) instead of navigation (`>`)

This option fixes the bug by teaching the preview builder how to produce a downstream preview snapshot for repeat-joiner lists when finishing that list would advance to another authored list.

## Recommendation

Use this option if the goal is to fix the user-facing mismatch with minimal risk.

Why:

- the current bug is caused by preview-generation logic, not by the underlying field engine
- the real confirm / advance behavior already works
- the stub renderer can stay unchanged if the preview sequence becomes accurate

## Current Grounding

Relevant current behavior:

- `data/objective.yml` defines `obmuscle_field` as a multi-part field with downstream lists after `muscle`
- `muscle` is authored as a repeating joiner list (`joiner_style: comma`, `repeat_limit: 12`, `modal_start: search`)
- `src/modal.rs` blocks `peek_next_list_view()` and `peek_next_list_views()` for any list where `effective_joiner_style(list).is_some()`
- `src/ui/modal_unit.rs` treats the final preview snapshot as a confirm stub boundary
- `src/app.rs` still uses the real modal engine for right-arrow / confirm behavior, so actual progression remains correct

## In Scope

- repeat-joiner list handling in simple modal preview generation
- modal-stream sequence correctness for downstream list previews
- tests covering the `obmuscle_field`-style case

## Out Of Scope

- changing real confirm behavior
- redefining the semantic meaning of stubs across the whole modal system
- broad refactors to nested, branch, or collection modal semantics

## Desired Product Rule

If the current list is a repeat-joiner list and finishing that list would move the user to another authored list, the modal stream should still show a right navigation stub (`>`) rather than a confirm stub (`+`).

More concretely:

- `+` means the real field flow would complete from here
- `>` means the real field flow can still advance to another modal after the current list is finished

Under this option, that rule is satisfied indirectly by making the preview sequence include the real downstream modal.

## Implementation Strategy

### 1. Extend forward preview generation for repeat-joiner lists

In `src/modal.rs`, update:

- `peek_next_list_view()`
- `peek_next_list_views()`

Current behavior:

- both helpers bail out immediately when `effective_joiner_style(list).is_some()`

Planned behavior:

- if the current list is a repeat-joiner list, simulate the "finish current repeating list" path
- if that simulated finish produces `FieldAdvance::NextList`, capture the resulting next-list snapshot
- if it produces `FieldAdvance::Complete(_)` or remains on the same list, stop as before

### 2. Reuse real modal-advance logic

Do not invent a second custom rule for repeat lists.

The preview path should call the same core transition logic the real modal uses:

- finalize current repeat values
- resolve the joined value
- call `finish_current_list(...)`
- inspect whether the result is `NextList` or `Complete`

This keeps the preview sequence aligned with real authored behavior.

### 3. Preserve current input behavior

No changes should be made to:

- `AppKey::Right` handling in `src/app.rs`
- `confirm_modal_value(...)`
- repeat-list confirmation behavior in `advance_active_leaf_field(...)`

The point of this option is to correct the stream preview without changing the live field flow.

### 4. Let existing stub rendering stay unchanged

If the preview sequence now includes the downstream `place` list, the existing stub-kind logic in `src/ui/modal_unit.rs` should naturally switch from:

- `ModalStubKind::Confirm`

to:

- `ModalStubKind::NavRight`

No direct UI-side stub override should be introduced in this option unless testing proves it is necessary.

## Expected Code Areas

- `src/modal.rs`
- `src/ui/modal_unit.rs` only if a small supporting adjustment is needed
- tests in `src/modal.rs` and/or `src/ui/mod.rs`

## Testing Plan

Add focused automated coverage for:

1. A field shaped like `obmuscle_field` where:
- first list is repeat-joiner
- downstream list exists
- preview sequence includes a next snapshot after the repeating list

2. A rendered modal unit for that case where:
- the right stub is `NavRight`
- not `Confirm`

3. A true terminal repeat-joiner field where:
- no downstream list exists
- the right stub remains `Confirm`

4. Existing non-repeat list preview behavior remains unchanged.

## Manual Verification

1. Open `obmuscle_field`.
2. Navigate to the `muscle` list.
3. Confirm that the right edge of the modal stream shows `>`, not green `+`.
4. Confirm that moving right still lands in `place` exactly as before.
5. Confirm that a true final modal still shows green `+`.

## Risks

- The preview system may accidentally imply more certainty than the real field flow allows in other repeat-list edge cases.
- Empty terminator behavior must remain consistent with current authored defaults.
- Branch items inside repeating lists need care so preview generation does not skip required branch flow.

## Tradeoffs

### Pros

- Smallest implementation surface
- Low regression risk
- Fixes the visible bug directly
- Preserves existing architecture

### Cons

- Stub meaning is still derived from preview-sequence shape rather than explicit modal semantics
- Similar mismatches could reappear if future modal types are not previewable
- Repeat-list logic in preview helpers becomes more special-case aware

## Exit Criteria

This option is complete when:

- `obmuscle_field` shows a right nav stub on `muscle`
- actual modal progression remains unchanged
- true terminal confirm stubs still render correctly
- automated coverage exists for both downstream-repeat and terminal-repeat cases
