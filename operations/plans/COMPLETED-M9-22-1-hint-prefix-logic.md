## Task
#22 ST1 - Add pure prefix-matching logic to data.rs

## Context
Task #22 implements multi-character hint sequences with a progressive prefix filtering state machine. The app already has `combined_hints(kb: &KeyBindings) -> Vec<&str>` in `src/data.rs` (line 389) that produces an ordered slice of all hints. The sub-task adds the pure data-layer logic that the eventual state machine (later sub-tasks) will call: a result enum, a filter function, and a resolver function.

No UI changes are made in this sub-task. The code lands in `src/data.rs` immediately after the `combined_hints` function.

## Approach
1. Define `HintResolveResult` as a `#[derive(Debug, PartialEq)]` enum with three variants:
   - `Exact(usize)` - exactly one hint matched and the typed string equals that hint in full
   - `Partial(Vec<usize>)` - one or more hints start with the typed prefix but none is an exact full match (or more than one matched exactly)
   - `NoMatch` - zero hints start with the prefix
2. `filter_hints_by_prefix` collects all indices where `hints[i].starts_with(prefix)`, returning a `Vec<usize>`.
3. `resolve_hint` calls `filter_hints_by_prefix`, then:
   - If result is empty: `NoMatch`
   - If result has exactly one index AND `hints[idx] == typed` exactly: `Exact(idx)`
   - Otherwise: `Partial(indices)`

The `Partial` branch covers two cases naturally:
- Multiple hints still share the prefix (user must type more)
- Exactly one hint starts with the prefix but is longer than what was typed (e.g. typed "z", only "zz" left - still Partial because the match is not yet exact)

## Critical Files
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/data.rs` - insertion point after `combined_hints` (line 393), before the `#[cfg(test)]` block (line 395)

## Reuse
- `filter_hints_by_prefix` uses `str::starts_with` from std - no new dependencies.
- The `#[cfg(test)]` block already exists in `data.rs`; new tests are appended inside it.

## Steps

### Step 1 - Add `HintResolveResult` enum
Insert after the `combined_hints` function (after line 393) and before the existing `#[cfg(test)]` block:

```rust
#[derive(Debug, PartialEq)]
pub enum HintResolveResult {
    Exact(usize),
    Partial(Vec<usize>),
    NoMatch,
}
```

### Step 2 - Add `filter_hints_by_prefix`
Insert immediately after the enum:

```rust
/// Returns the indices of all hints that start with `prefix`.
/// An empty `prefix` matches every hint.
pub fn filter_hints_by_prefix(hints: &[&str], prefix: &str) -> Vec<usize> {
    hints
        .iter()
        .enumerate()
        .filter_map(|(i, h)| if h.starts_with(prefix) { Some(i) } else { None })
        .collect()
}
```

### Step 3 - Add `resolve_hint`
Insert immediately after `filter_hints_by_prefix`:

```rust
/// Resolves the current typed string against the hint list.
///
/// - `NoMatch`   - no hint starts with `typed`
/// - `Exact(i)`  - exactly one hint starts with `typed` AND equals `typed` in full
/// - `Partial(v)`- one or more hints share the prefix but none is an exact full match,
///                 or more than one match exists
pub fn resolve_hint(hints: &[&str], typed: &str) -> HintResolveResult {
    let matches = filter_hints_by_prefix(hints, typed);
    match matches.as_slice() {
        [] => HintResolveResult::NoMatch,
        [idx] if hints[*idx] == typed => HintResolveResult::Exact(*idx),
        _ => HintResolveResult::Partial(matches),
    }
}
```

### Step 4 - ~~Add unit tests~~ ALREADY DONE (pre-written by Test Writer in Red phase)

9 tests already exist in `data.rs` at lines 624-693 (including `resolve_hint_partial_one_match_longer_than_typed` added during Reviewer-1 pass). Do NOT re-insert them.

## Verification

### Manual tests
After `cargo test` passes:
1. Confirm `HintResolveResult`, `filter_hints_by_prefix`, and `resolve_hint` are visible from `src/app.rs` via `use crate::data::{HintResolveResult, filter_hints_by_prefix, resolve_hint};` (compile check only - no app changes yet).

### Automated tests
Test names:
- `data::tests::filter_hints_single_char_hit`
- `data::tests::filter_hints_multi_char_partial`
- `data::tests::filter_hints_exact_full_match`
- `data::tests::filter_hints_no_match`
- `data::tests::resolve_hint_single_char_returns_exact`
- `data::tests::resolve_hint_partial_two_matches`
- `data::tests::resolve_hint_partial_one_match_longer_than_typed`
- `data::tests::resolve_hint_exact_two_char`
- `data::tests::resolve_hint_no_match_resets`
- `data::tests::resolve_hint_empty_typed_is_partial_all`

Cargo command:
```
PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" /c/Users/solar/.cargo/bin/cargo.exe test --manifest-path "/c/scribble/Cargo.toml"
```

## Changelog
### Plan - 2026-03-30
- Initial plan
- Reviewer-1 R1: Step 4 marked ALREADY DONE; added `resolve_hint_partial_one_match_longer_than_typed` test to data.rs (critical edge case: one match remaining but longer than typed → Partial not Exact).
