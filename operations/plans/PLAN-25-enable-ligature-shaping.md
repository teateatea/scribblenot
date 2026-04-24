# PLAN-25: Enable Font Ligatures (Fira Code)

**Status**: Draft

## Background

Iced has two text shaping modes (`text::Shaping`):
- `Basic` (default) - simple glyph lookup, no ligatures
- `Advanced` - uses cosmic-text for full OpenType shaping, enabling Fira Code ligatures like `->`, `=>`, `!=`, `<=`, `>=`, etc.

The codebase never sets `.shaping()` on any text widget, so everything defaults to `Basic`. The `advanced` feature flag is already enabled in `Cargo.toml`, so no dependency changes are needed.

## Steps

### Step 1: Add shaping import to `ui/mod.rs`

Add `text::Shaping` to the existing iced imports at the top of `src/ui/mod.rs`.

### Step 2: Create a `shaped_text()` helper

Add a small wrapper near the existing font helpers (~line 1052):

```rust
fn shaped_text(content: impl ToString) -> iced::widget::Text<'static, ...> {
    text(content.to_string()).shaping(Shaping::Advanced)
}
```

This keeps the diff minimal and gives a single place to change if the approach needs adjusting.

### Step 3: Replace `text(...)` with `shaped_text(...)` selectively

Apply to widgets where ligatures are meaningful - pane content and preview text. Skip purely decorative or single-character text (e.g. modal stub symbols, single letters) where `Advanced` shaping adds overhead with no benefit.

### Step 4: Apply `.shaping(Shaping::Advanced)` to `rich_text`

The `rich_text` widget at ~line 2046 has a separate shaping path. Add `.shaping(Shaping::Advanced)` directly there since it doesn't go through `text()`.

## Trade-offs

| | Basic (current) | Advanced |
|---|---|---|
| Ligatures | No | Yes |
| Unicode / complex scripts | No | Yes |
| Performance | Faster | Slightly slower (cosmic-text layout pass) |
| Scope of change | - | ~20-30 call sites, or 1 helper + selective replacement |

Performance difference is negligible for a note-taking UI at this text volume.

## Estimated Scope

`src/ui/mod.rs` only, ~25 lines changed. No changes to `theme.rs`, `main.rs`, `Cargo.toml`, or data files.
