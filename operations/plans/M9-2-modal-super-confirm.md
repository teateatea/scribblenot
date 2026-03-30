## Task

#2 - Add Shift+Enter super-confirm keybinding to auto-complete remaining fields

## Context

Sub-task 2 of 3 for task #2. Sub-task 1 added the `super_confirm` keybinding to `KeyBindings` and the `is_super_confirm` helper method. This sub-task wires `is_super_confirm` into `handle_modal_key` so that when a modal is open and the user presses Shift+Enter, the value is confirmed immediately using the best available value: typed query text (if non-empty), else the currently highlighted list item's output, else close the modal silently without advancing.

The path through `confirm_modal_value` already handles both simple and composite modals correctly. We just need to intercept `is_super_confirm` early in `handle_modal_key`, resolve a value (or bail), and call `confirm_modal_value`.

## Approach

Add an `is_super_confirm` check at the top of `handle_modal_key`, before the `focus` match, mirroring the pattern used by the `Esc` early-return that already sits there. Resolve value in priority order: (1) non-empty `query.trim()`, (2) `selected_value()` from the current filtered list (the highlighted item). If neither is available, close the modal silently (`self.modal = None`) and return. Otherwise call `confirm_modal_value(value)`.

## Critical Files

- `src/app.rs` - `handle_modal_key` (line 616), `confirm_modal_value` (line 703), `is_super_confirm` helper (line 228)
- `src/modal.rs` - `SearchModal::selected_value` (line 156), `SearchModal::query` field (line 23)

## Reuse

- `self.is_super_confirm(&key)` - already defined in `src/app.rs` line 228
- `self.confirm_modal_value(value)` - already handles both simple and composite modals
- `modal.selected_value()` - returns `Option<&str>` for the highlighted list item
- `modal.query.trim()` - gives the typed search text

## Steps

1. In `src/app.rs`, inside `handle_modal_key`, add an `is_super_confirm` branch immediately after the `Esc` early-return (before the `focus` match):

```diff
     fn handle_modal_key(&mut self, key: KeyEvent) {
         let hints = self.data.keybindings.hints.clone();

         if key.code == KeyCode::Esc {
             self.modal = None;
             return;
         }

+        if self.is_super_confirm(&key) {
+            let value = {
+                let modal = self.modal.as_ref().unwrap();
+                let q = modal.query.trim().to_string();
+                if !q.is_empty() {
+                    Some(q)
+                } else {
+                    modal.selected_value().map(String::from)
+                }
+            };
+            match value {
+                Some(v) => self.confirm_modal_value(v),
+                None => { self.modal = None; }
+            }
+            return;
+        }
+
         let focus = match &self.modal {
```

## Verification

### Manual tests

1. Open the app and navigate to a header field. Press Enter to open a simple (non-composite) modal. Without typing anything, press Shift+Enter. The highlighted item should be confirmed and the modal should close, advancing to the next field.

2. Open the same modal, type a partial query to filter the list (e.g. "ma"). The list should narrow. Press Shift+Enter. The currently highlighted filtered item should be confirmed using `query.trim()` (the typed text is used first).

3. Open a composite modal (a field with multiple parts). Press Shift+Enter on the first part without a query. The highlighted item for part 1 should be confirmed. The modal should then show part 2. Press Shift+Enter again. Part 2 is confirmed. This repeats until all parts are complete and the modal closes.

4. Open any modal. Navigate the list to a non-first item with arrow keys (or n/e), then press Shift+Enter with no query. The highlighted (non-first) item should be confirmed, not the first item.

5. Open a modal with an empty filtered list (type a query string that matches nothing). Press Shift+Enter. The modal should close silently without advancing the field or producing an error.

### Automated tests

No unit tests are planned (TUI modal state, as specified). The manual tests above cover all branching paths.

## Progress
- Step 1: Added `is_super_confirm` branch in `handle_modal_key` after Esc check, resolving value from query or selected_value, closing modal silently if neither available
