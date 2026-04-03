# Plan: M9-21-1 — Add `group_jump_target` pure function

**Status:** Draft

## Goal

Add a pure function `group_jump_target(groups: &[SectionGroup], g_idx: usize) -> usize` in `src/data.rs` that returns the flat (global) section index of the first section in a given group. Include unit tests covering all group indices, empty-group case, and out-of-bounds case.

## Context

`SectionGroup` is defined in `src/data.rs`:
```rust
pub struct SectionGroup {
    pub id: String,
    pub num: Option<usize>,
    pub name: String,
    pub sections: Vec<SectionConfig>,
}
```

Flat indices are computed by summing `sections.len()` across preceding groups (see `app.rs` line 334: `self.data.groups.iter().take(g_idx).map(|g| g.sections.len()).sum()`). The new function formalizes this as a reusable utility.

## Implementation

### Step 1 — Add `group_jump_target` to `src/data.rs`

Insert after the `combined_hints` function (around line 393) and before the `HintResolveResult` enum:

```rust
/// Returns the flat section index of the first section in `groups[g_idx]`.
///
/// - If `g_idx` is in bounds, returns the sum of `sections.len()` for all preceding groups.
/// - If the group exists but has 0 sections, returns the same start index (which equals the
///   next group's start, or total section count if it is the last group).
/// - If `g_idx >= groups.len()`, returns the total section count (past-the-end sentinel).
pub fn group_jump_target(groups: &[SectionGroup], g_idx: usize) -> usize {
    if g_idx >= groups.len() {
        return groups.iter().map(|g| g.sections.len()).sum();
    }
    groups.iter().take(g_idx).map(|g| g.sections.len()).sum()
}
```

### Step 2 — Add unit tests in the existing `#[cfg(test)] mod tests` block in `src/data.rs`

Append the following test functions inside the `tests` module (before the closing `}`):

```rust
// ---- group_jump_target tests (Task #21 sub-task 1) ----

fn make_groups(sizes: &[usize]) -> Vec<SectionGroup> {
    sizes
        .iter()
        .enumerate()
        .map(|(i, &n)| SectionGroup {
            id: format!("g{i}"),
            num: None,
            name: format!("Group {i}"),
            sections: (0..n)
                .map(|j| SectionConfig {
                    id: format!("s{i}_{j}"),
                    name: format!("Section {i}/{j}"),
                    map_label: format!("{i}/{j}"),
                    section_type: "free_text".to_string(),
                    data_file: None,
                    date_prefix: None,
                    options: vec![],
                    composite: None,
                    fields: None,
                })
                .collect(),
        })
        .collect()
}

#[test]
fn group_jump_target_group0_returns_0() {
    let groups = make_groups(&[3, 2, 4]);
    assert_eq!(group_jump_target(&groups, 0), 0);
}

#[test]
fn group_jump_target_group1_returns_sum_of_group0() {
    let groups = make_groups(&[3, 2, 4]);
    assert_eq!(group_jump_target(&groups, 1), 3);
}

#[test]
fn group_jump_target_group2_returns_sum_of_groups_0_and_1() {
    let groups = make_groups(&[3, 2, 4]);
    assert_eq!(group_jump_target(&groups, 2), 5);
}

#[test]
fn group_jump_target_out_of_bounds_returns_total_count() {
    let groups = make_groups(&[3, 2, 4]);
    // total = 9; g_idx = 3 is out of bounds
    assert_eq!(group_jump_target(&groups, 3), 9);
}

#[test]
fn group_jump_target_far_out_of_bounds_returns_total_count() {
    let groups = make_groups(&[3, 2, 4]);
    assert_eq!(group_jump_target(&groups, 100), 9);
}

#[test]
fn group_jump_target_empty_group_returns_same_as_next_start() {
    // group 1 has 0 sections; its start == group 0's end == 3
    let groups = make_groups(&[3, 0, 4]);
    assert_eq!(group_jump_target(&groups, 1), 3);
    // group 2's start == 3 + 0 == 3 as well
    assert_eq!(group_jump_target(&groups, 2), 3);
}

#[test]
fn group_jump_target_all_empty_groups() {
    let groups = make_groups(&[0, 0, 0]);
    assert_eq!(group_jump_target(&groups, 0), 0);
    assert_eq!(group_jump_target(&groups, 1), 0);
    assert_eq!(group_jump_target(&groups, 2), 0);
    assert_eq!(group_jump_target(&groups, 3), 0); // out of bounds, total = 0
}

#[test]
fn group_jump_target_single_group() {
    let groups = make_groups(&[5]);
    assert_eq!(group_jump_target(&groups, 0), 0);
    assert_eq!(group_jump_target(&groups, 1), 5); // out of bounds
}

#[test]
fn group_jump_target_empty_slice() {
    let groups: Vec<SectionGroup> = vec![];
    assert_eq!(group_jump_target(&groups, 0), 0); // out of bounds, total = 0
}
```

## Manual Tests

None required - all verification is via automated unit tests.

## Verification

Run:
```
PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" /c/Users/solar/.cargo/bin/cargo.exe test --manifest-path "/c/scribble/Cargo.toml"
```

All tests (existing + new) must pass with zero failures.

## Files Modified

- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/data.rs` - add `group_jump_target` function and unit tests
