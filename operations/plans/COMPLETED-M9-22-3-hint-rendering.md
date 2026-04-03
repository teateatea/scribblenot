## Task
Update hint rendering in map and header TUI code (ratatui spans) to consume `hint_buffer` from `App`: for each rendered hint, if the buffer is non-empty split the hint into a white-colored confirmed prefix span and a normal-colored remainder span; gray out hints whose prefix does not match the buffer. Single-char hints with no buffer involvement render unchanged.

## Context
`app.hint_buffer: String` (App struct, `src/app.rs` line 79) holds the characters typed so far during hint navigation. It is already updated by sub-task 2.

Three hint rendering sites exist in `src/ui.rs`:

1. **Group hints** (`render_section_map`, line ~100-106): one `Span::styled(format!("{} ", group_hint_display), Style::default().fg(group_hint_color))`
2. **Section hints** (`render_section_map`, line ~158-162): one `Span::styled(format!("{} ", section_hint_display), Style::default().fg(section_hint_color))`
3. **Header field hints** (`render_header_widget`, line ~270-278): `Span::styled(format!(" {} ", hint_str), Style::default().fg(hint_color))` in the block title

Current active color = `theme::HINT` (magenta). Inactive/grayed = `theme::MUTED` (DarkGray).

The `hint_buffer` logic applies only when a hint-navigation session is active. The existing code already gates hints with `group_hint_color`, `section_hint_color`, and `hint_color` — these already go `MUTED` in modal / non-map modes. The buffer-split effect must be layered on top of those color decisions, not replace them.

Key rule: when `hint_buffer` is empty, behavior is identical to today (no change).

## Approach
Extract a small helper function `hint_spans(hint: &str, hint_buffer: &str, active_color: Color) -> Vec<Span<'static>>` that returns the appropriate span(s) for a single hint string:

- If `hint_buffer` is empty or hint is a single char with empty buffer: return `[Span::styled(hint.to_string(), Style::default().fg(active_color))]` (unchanged behavior)
- Else if `hint.to_lowercase().starts_with(&hint_buffer.to_lowercase())`:
  - prefix span: `hint[..hint_buffer.len()]` styled `Style::default().fg(Color::White).add_modifier(Modifier::BOLD)`
  - remainder span: `hint[hint_buffer.len()..]` styled `Style::default().fg(active_color)`
  - return `[prefix_span, remainder_span]`
- Else (hint doesn't match buffer):
  - return `[Span::styled(hint.to_string(), Style::default().fg(theme::MUTED))]`

The caller is responsible for adding the surrounding space padding (` ` prefix/suffix) and passing the correct `active_color` — either `theme::HINT` or `theme::MUTED` depending on existing group/section hint color logic. The buffer-split only activates when the hint color would otherwise be `theme::HINT` (i.e. the hint is in the active set). When `active_color == theme::MUTED` the caller should skip calling this function and just emit the existing muted span as before.

For the **header field hints** site, `hint_buffer` needs to be threaded into `render_header_widget`. This is currently called from `render_wizard_widget`, which already takes `app: &App`. Add `hint_buffer: &str` as a new parameter to `render_header_widget` and pass `&app.hint_buffer`.

## Critical Files
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/ui.rs` - all three render sites + new helper
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/app.rs` - source of `hint_buffer` (read-only reference)

## Reuse
- `theme::HINT`, `theme::MUTED`, `theme::dim()` - existing constants from `src/theme.rs`
- `Color::White`, `Modifier::BOLD` - ratatui primitives already in scope via `use ratatui::style::{Color, Modifier, Style}`

## Steps

### Step 1 - Add `Color` to the style import in `src/ui.rs`

Line 10 currently reads `style::{Modifier, Style}`. Change to:
```rust
use ratatui::style::{Color, Modifier, Style};
```

### Step 2 - Add helper function `hint_spans` to `src/ui.rs`

Insert before `fn render_section_map`:

```rust
/// Returns the styled span(s) for a hint label given the current hint_buffer.
/// `active_color` should be the color that would be used when the hint is active (HINT).
/// When `hint_buffer` is empty, returns a single span with `active_color` (no change).
/// When `hint_buffer` is non-empty and hint starts with buffer: prefix in White/Bold + remainder in active_color.
/// When `hint_buffer` is non-empty and hint does NOT start with buffer: single span in MUTED.
fn hint_spans(hint: &str, hint_buffer: &str, active_color: Color) -> Vec<Span<'static>> {
    if hint_buffer.is_empty() {
        return vec![Span::styled(hint.to_string(), Style::default().fg(active_color))];
    }
    let hint_lower = hint.to_lowercase();
    let buf_lower = hint_buffer.to_lowercase();
    if hint_lower.starts_with(&buf_lower) {
        let prefix = hint[..hint_buffer.len()].to_string();
        let remainder = hint[hint_buffer.len()..].to_string();
        let prefix_span = Span::styled(
            prefix,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        );
        if remainder.is_empty() {
            vec![prefix_span]
        } else {
            vec![
                prefix_span,
                Span::styled(remainder, Style::default().fg(active_color)),
            ]
        }
    } else {
        vec![Span::styled(hint.to_string(), Style::default().fg(theme::MUTED))]
    }
}
```

Note: The function takes `&str` and returns `Vec<Span<'static>>` by cloning the string slices into owned `String`s, which is necessary because `ListItem` and `Line` require owned data.

### Step 3 - Update group hint rendering in `render_section_map`

Current code (lines ~100-106):
```rust
items.push(ListItem::new(Line::from(vec![
    Span::styled(
        format!("{} ", group_hint_display),
        Style::default().fg(group_hint_color),
    ),
    Span::styled(group.name.clone(), group_name_style),
])));
```

Replace with:
```rust
let group_hint_spans: Vec<Span> = if group_hint_color == theme::HINT && !app.hint_buffer.is_empty() {
    let mut spans = hint_spans(&group_hint_display, &app.hint_buffer, theme::HINT);
    // Append trailing space as part of last span's text
    if let Some(last) = spans.last_mut() {
        let s = last.content.to_string() + " ";
        *last = Span::styled(s, last.style);
    }
    spans
} else {
    vec![Span::styled(
        format!("{} ", group_hint_display),
        Style::default().fg(group_hint_color),
    )]
};
let mut group_line_spans = group_hint_spans;
group_line_spans.push(Span::styled(group.name.clone(), group_name_style));
items.push(ListItem::new(Line::from(group_line_spans)));
```

### Step 4 - Update section hint rendering in `render_section_map`

Current code (line ~158-162):
```rust
items.push(ListItem::new(Line::from(vec![
    Span::styled(cursor_char.to_string(), entry_style),
    Span::styled(format!("{} ", section_hint_display), Style::default().fg(section_hint_color)),
    Span::styled(section.map_label.clone(), entry_style),
])));
```

Replace with:
```rust
let section_hint_spans: Vec<Span> = if section_hint_color == theme::HINT && !app.hint_buffer.is_empty() {
    let mut spans = hint_spans(&section_hint_display, &app.hint_buffer, theme::HINT);
    if let Some(last) = spans.last_mut() {
        let s = last.content.to_string() + " ";
        *last = Span::styled(s, last.style);
    }
    spans
} else {
    vec![Span::styled(
        format!("{} ", section_hint_display),
        Style::default().fg(section_hint_color),
    )]
};
let mut section_line_spans = vec![Span::styled(cursor_char.to_string(), entry_style)];
section_line_spans.extend(section_hint_spans);
section_line_spans.push(Span::styled(section.map_label.clone(), entry_style));
items.push(ListItem::new(Line::from(section_line_spans)));
```

### Step 5 - Thread `hint_buffer` into `render_header_widget`

In `render_wizard_widget`, change the call:
```rust
render_header_widget(f, area, s, map_preview, &field_hints, hints_active, &app.config.sticky_values, modal_for_header)
```
to:
```rust
render_header_widget(f, area, s, map_preview, &field_hints, hints_active, &app.hint_buffer, &app.config.sticky_values, modal_for_header)
```

Update function signature from:
```rust
fn render_header_widget(
    f: &mut Frame,
    area: Rect,
    state: &crate::sections::header::HeaderState,
    map_preview: bool,
    field_hints: &[String],
    hints_active: bool,
    sticky_values: &HashMap<String, String>,
    active_modal: Option<&SearchModal>,
)
```
to:
```rust
fn render_header_widget(
    f: &mut Frame,
    area: Rect,
    state: &crate::sections::header::HeaderState,
    map_preview: bool,
    field_hints: &[String],
    hints_active: bool,
    hint_buffer: &str,
    sticky_values: &HashMap<String, String>,
    active_modal: Option<&SearchModal>,
)
```

### Step 6 - Update header field hint rendering in `render_header_widget`

Current code (lines ~270-278):
```rust
let hint_str = field_hints.get(i).map(String::as_str).unwrap_or("");
let field_title = if !hint_str.is_empty() {
    Line::from(vec![
        Span::styled(format!(" {} ", hint_str), Style::default().fg(hint_color)),
        Span::raw(format!("{} ", cfg.name)),
    ])
} else {
    Line::from(Span::raw(format!(" {} ", cfg.name)))
};
```

Replace with:
```rust
let hint_str = field_hints.get(i).map(String::as_str).unwrap_or("");
let field_title = if !hint_str.is_empty() {
    let hint_title_spans: Vec<Span> = if hints_active && !hint_buffer.is_empty() {
        // Leading space
        let mut spans = vec![Span::raw(" ")];
        spans.extend(hint_spans(hint_str, hint_buffer, theme::HINT));
        spans.push(Span::raw(" "));
        spans
    } else {
        vec![Span::styled(format!(" {} ", hint_str), Style::default().fg(hint_color))]
    };
    let mut title_spans = hint_title_spans;
    title_spans.push(Span::raw(format!("{} ", cfg.name)));
    Line::from(title_spans)
} else {
    Line::from(Span::raw(format!(" {} ", cfg.name)))
};
```

### Step 7 - Build and verify compilation

```bash
PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" /c/Users/solar/.cargo/bin/cargo.exe build --manifest-path "/c/scribble/Cargo.toml"
```

Fix any type errors (e.g. `Span::styled` taking owned vs borrowed strings, `last_mut` returning `Option<&mut Span>` requiring careful reborrow).

## Verification

### Manual tests

1. **Empty hint_buffer (baseline)**: Open app, do not type any hint characters. Verify all hints render exactly as before (magenta for active, dark gray for inactive). No visual change.

2. **Group hint prefix highlight**: Navigate to Map focus with Groups level. Type first character of a group hint (e.g. "a"). Verify:
   - Group "a" hint: "a" portion renders White/Bold, remaining chars render magenta
   - Other group hints: entire hint renders dark gray (MUTED)

3. **Section hint prefix highlight**: Press a group hint to enter Sections level. Type first character of a section hint. Verify:
   - Matching section hint: typed prefix is White/Bold, remainder is magenta
   - Non-matching section hints: render dark gray

4. **Multi-char buffer**: Type two characters of a multi-char hint. Verify both characters render White/Bold and the remainder is magenta.

5. **Full match**: Type all characters of a hint. Verify entire hint renders White/Bold (remainder is empty, only prefix span shown).

6. **Header field hints**: In wizard mode (non-map), type a character matching a header field hint. Verify prefix turns White/Bold, remainder stays magenta, non-matching hints gray out.

7. **No buffer - header**: In wizard mode with empty buffer, header hints render as plain magenta (unchanged).

8. **Backspace**: After typing one hint char, press Backspace. Verify buffer clears and all hints return to normal magenta rendering.

### Automated tests
None - TDD is infeasible for ratatui terminal rendering.

## Changelog
### Plan - 2026-03-30
- Initial plan
- Prefect R1: Added Step 1 to add `Color` to the style import (was missing from ui.rs); renumbered subsequent steps.
