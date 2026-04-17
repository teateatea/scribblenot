---
Status: Draft
---

## Context

When focus moves onto the active modal, two visual jitters appear:

1. **Search bar shrinks slightly.** The inactive preview uses `preview_modal_search_strip`, which is a `container(text(...))` with explicit `.padding([6.0, 8.0])`. The active modal uses `text_input(...)` with no explicit padding, so iced applies its default (5px all sides). The 1px difference top and bottom means the active search bar is 2px shorter, causing a visible shrink on focus.

2. **List rows are taller/more spaced in the active modal.** The inactive preview wraps each row in `container(row_content)` with no padding. The active modal wraps each row in `button(button_label)`, and iced buttons default to 5px padding on all sides. This adds 10px of vertical height per row, making the list look bigger and more spread out.

Both issues are purely about matching widget padding between the active and inactive renderers. Color is unaffected.

## Approach

Add explicit padding to the `text_input` in `active_simple_modal_content` to match the preview strip's `[6.0, 8.0]`, and add `.padding(0)` to the list buttons to match the preview's zero-padding containers.

## Critical Files

- `src/ui/mod.rs`
  - `preview_modal_search_strip` (line 1487) - reference for correct padding values
  - `active_simple_modal_content` (line 1645) - where both fixes go
    - `text_input` call (line 1663) - needs `.padding([6, 8])`
    - `button(button_label)` call (line 1727) - needs `.padding(0)`

## Reuse

- `preview_modal_search_strip` padding values (`[6.0, 8.0]`) are the canonical source of truth for search bar height.
- No new helpers needed.

## Steps

1. **Fix search bar padding.** In `active_simple_modal_content`, add `.padding([6, 8])` to the `text_input` so its rendered height matches the preview strip's `container(text(...)).padding([6.0, 8.0])`.

```diff
     text_input("Search", &modal.query)
         .on_input(Message::ModalQueryChanged)
         .font(ui_theme.font_modal)
         .width(Length::Fill)
+        .padding([6, 8])
         .style(move |_theme, status| modal_input_style(&app_theme, status))
         .into(),
```

2. **Fix list row padding.** In `active_simple_modal_content`, add `.padding(0)` to the `button` so each row's height matches the preview's `container(row_content)` which has no padding.

```diff
             let app_theme = ui_theme.clone();
             list_items.push(
                 button(button_label)
                     .width(Length::Fill)
+                    .padding(0)
                     .on_press(Message::ModalSelect(window_pos))
                     .style(move |_theme, status| modal_item_button_style(&app_theme, status))
                     .into(),
```

## Verification

### Manual tests

1. Open the app and trigger a modal sequence with at least two panels visible side by side.
2. Confirm the search bar height is identical between the inactive (preview) and active modal - no shrink or grow when focus moves between panels.
3. Confirm the list row height and line spacing are identical between inactive and active modals.
4. Confirm colors are unaffected (active/inactive color contrast still works).
5. Navigate forward and backward through modal units and confirm no jitter occurs during or after the transition animation.
6. Type in the search bar of the active modal and confirm the input still functions correctly.

### Automated tests

No existing tests cover rendered widget dimensions. A realistic option: snapshot/golden tests using `iced_test` or a headless render harness that measures the bounding box of the search bar widget in both preview and active states and asserts they match.
