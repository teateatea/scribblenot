## Task

#23 - Auto-generate multi-character hint permutations from base hint characters for overflow assignment

## Context

When the base hint pool (e.g. `[q, w, f, p]`) is smaller than the number of hints needed, there are currently no fallback hints assigned. This plan adds a `hint_permutations: Vec<String>` field to `KeyBindings` (serde-defaulting to empty) and a pure function `generate_hint_permutations` that builds adjacency-priority-ordered n^r permutations up to a requested count. Seven TDD tests are already compiled into `src/data.rs` (lines 328-419) and fail at compile time because neither the field nor the function exist yet.

## Approach

Add the field directly to `KeyBindings` with `#[serde(default)]`, update `KeyBindings::default()` to include it, then implement `generate_hint_permutations` as a free public function in `src/data.rs`. The adjacency ordering is achieved by iterating over "distance bands" (0, 1, 2, ...) where the distance between indices `i` and `j` is `(i as isize - j as isize).abs()`, collecting all `(first, second)` pairs at that distance before moving to the next, with repeats (`distance=0`, i.e. `aa`, `bb`) treated as distance 0 (most adjacent). For r=3 fallback, extend each r=2 entry with all base characters in the same adjacency-priority order.

## Critical Files

- `src/data.rs` lines 144-197: `KeyBindings` struct and `Default` impl - field and default added here
- `src/data.rs` line 300: after `AppData` impl block - new function added before the `#[cfg(test)]` block
- `src/data.rs` lines 328-419: TDD tests already written, drive the implementation

## Reuse

- Existing `#[serde(default)]` pattern already used on `focus_left`, `focus_right`, `hints`, and `super_confirm` fields in `KeyBindings` (lines 155-162) - follow the same pattern.
- Existing `default_hints()` helper function (line 175) as a style reference for the default helper.

## Steps

1. Add `hint_permutations` field to `KeyBindings` struct with serde default:

```
- pub super_confirm: Vec<String>,
+ pub super_confirm: Vec<String>,
+ #[serde(default)]
+ pub hint_permutations: Vec<String>,
```

2. Add `hint_permutations: vec![]` to `KeyBindings::default()`:

```
-             super_confirm: default_super_confirm(),
+             super_confirm: default_super_confirm(),
+             hint_permutations: vec![],
```

3. Implement `generate_hint_permutations` as a public free function immediately before the `#[cfg(test)]` block (around line 302). The function builds results in adjacency-band order:

```rust
pub fn generate_hint_permutations(base: &[String], count_needed: usize) -> Vec<String> {
    let n = base.len();
    if n == 0 || count_needed == 0 {
        return vec![];
    }

    let mut result: Vec<String> = Vec::with_capacity(count_needed);

    // r=1: single characters (band 0 only - each char is its own "pair")
    // Skip r=1; hints field already covers single chars.
    // r=2: iterate distance bands 0..n
    'outer: for dist in 0..n {
        for i in 0..n {
            // j = i + dist (wrap is not meaningful for linear adjacency - skip wrapping)
            if dist == 0 {
                // Same-index pairs: "qq", "ww", etc.
                let entry = format!("{}{}", base[i], base[i]);
                result.push(entry);
                if result.len() >= count_needed {
                    break 'outer;
                }
            } else {
                // (i, i+dist) and (i+dist, i) - both directions
                let j = i + dist;
                if j < n {
                    result.push(format!("{}{}", base[i], base[j]));
                    if result.len() >= count_needed { break 'outer; }
                    result.push(format!("{}{}", base[j], base[i]));
                    if result.len() >= count_needed { break 'outer; }
                }
            }
        }
    }

    if result.len() >= count_needed {
        return result;
    }

    // r=3 fallback: extend each r=2 entry with all base chars in adjacency order
    let r2_complete = result.clone();
    'r3: for prefix in &r2_complete {
        for dist in 0..n {
            for i in 0..n {
                if dist == 0 {
                    let entry = format!("{}{}", prefix, base[i]);
                    result.push(entry);
                    if result.len() >= count_needed { break 'r3; }
                } else {
                    let j = i + dist;
                    if j < n {
                        result.push(format!("{}{}", prefix, base[i]));
                        if result.len() >= count_needed { break 'r3; }
                        result.push(format!("{}{}", prefix, base[j]));
                        if result.len() >= count_needed { break 'r3; }
                    }
                }
            }
        }
    }

    result.truncate(count_needed);
    result
}
```

Note: The r=3 fallback above appends a third character to each r=2 prefix using the same adjacency-band walk over base. This satisfies the test requirement that entries with `len() == 3` appear when `count_needed > n^2`. In the tested case (n=4, count_needed=20), `dist=0` produces all 4 needed r=3 entries before the `dist > 0` branch is reached; the `dist > 0` branch would re-emit already-seen `base[i]` values and produce duplicates if more than `n` r=3 entries per prefix were needed, but the tests do not exercise that path.

4. Run the tests to confirm all 7 pass:

```
PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" /c/Users/solar/.cargo/bin/cargo.exe test --manifest-path "/c/scribble/Cargo.toml"
```

Note: `/c/scribble` is a Windows junction pointing to the project root (`C:\Users\solar\Documents\Claude Projects\scribblenot`). The junction is mandatory because the GNU toolchain (mingw) cannot handle spaces in paths. Do NOT substitute the full path with spaces here.

Expected: all 7 tests in the `hint_permutations` group pass; the 2 pre-existing `super_confirm` tests continue to pass.

## Verification

### Manual tests

None required - the changes are pure data-layer logic with no UI surface.

### Automated tests

The 7 TDD tests already in `src/data.rs` lines 328-419 serve as the complete automated test suite:

- `hint_permutations_capped_at_count_needed` - output capped at count_needed
- `hint_permutations_r2_from_4_element_base` - full r=2 space (16 entries), each len==2
- `hint_permutations_adjacency_ordering_adjacent_before_distant` - qq/qw/wq/ww all before qp
- `hint_permutations_r3_fallback_when_r2_not_enough` - 20 entries with at least one len==3
- `keybindings_hint_permutations_field_defaults_empty` - `KeyBindings::default()` has empty field
- `keybindings_hint_permutations_serde_default_empty` - absent from YAML deserializes to empty
- `keybindings_hint_permutations_serde_explicit_value` - explicit YAML values deserialized correctly

Run with the build command in Step 4. All 7 must pass with zero compile errors.

## Changelog

### Review - 2026-03-30
- #1 (nit): Clarified the Note after Step 3 to document that the `dist > 0` branch in the r=3 fallback is unreachable in the tested scenario (n=4, count_needed=20) and would produce duplicates if more than n r=3 entries per prefix were required; the existing tests do not exercise that path.
- #2 (Prefect): Added note to Step 4 clarifying that `/c/scribble` is a Windows junction (spaces-in-path workaround) and must not be substituted with the full path.
