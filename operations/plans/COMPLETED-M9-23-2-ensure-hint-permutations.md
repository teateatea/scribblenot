## Task
#23 ST2 - Add `ensure_hint_permutations` and call it from AppData::load

## Context
Task #23 requires auto-generating multi-character hint permutations from the base `hints:` list in
keybindings.yml. Sub-task 1 added `generate_hint_permutations` and the `hint_permutations` field on
`KeyBindings`. Sub-task 2 adds the cache-management layer: a pure function
`ensure_hint_permutations(kb: &mut KeyBindings)` that computes how many permutations are needed, and
regenerates `hint_permutations` only when it is empty or stale (i.e., the hints list changed). The
function is called from `AppData::load` after keybindings are loaded.

Staleness is detected by comparing `hint_permutations.len()` against `count_needed`. If the
`hints:` list changes length, `count_needed` changes, and the cached vec is considered stale.
Because the function has no access to runtime section data, `count_needed` is derived purely from
`kb.hints.len()`: we generate enough permutations to cover the full r=2 space of the base hints
list (`hints.len() * hints.len()`). This gives a deterministic count that changes whenever the
hints list changes length, satisfying the staleness contract.

## Approach
1. Define `pub fn ensure_hint_permutations(kb: &mut KeyBindings)` in data.rs.
2. Compute `count_needed = kb.hints.len() * kb.hints.len()`. If `kb.hints` is empty, return early.
3. Check freshness: if `kb.hint_permutations.len() == count_needed`, do nothing (already fresh).
4. Otherwise, regenerate: `kb.hint_permutations = generate_hint_permutations(&kb.hints, count_needed)`.
5. In `AppData::load`, call `ensure_hint_permutations(&mut keybindings)` immediately after
   `keybindings` is resolved (whether from file or default).

The staleness invariant: `hint_permutations.len() == hints.len()^2` when fresh. Any change to
`hints.len()` causes a mismatch and triggers regeneration. An empty `hint_permutations` (serde
default) also mismatches (0 != n^2 for n>0), triggering generation on first run.

## Critical Files
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/data.rs`
  - `generate_hint_permutations` at line 305
  - `KeyBindings` struct at lines 144-165 (hints at 160, hint_permutations at 163-164)
  - `AppData::load` at lines 213-275; keybindings resolved at lines 252-264, `Ok(Self {...})` at 266
  - `#[cfg(test)]` block starts at line 369

## Reuse
- Follow the existing pattern of `pub fn generate_hint_permutations(...)` for the new public function.
- Tests in the existing `mod tests` block at line 369 - append new tests there.
- Use the same `Vec<String>` mutation pattern as the rest of the module.

## Steps

### Step 1 - Add `ensure_hint_permutations` function (data.rs, after `generate_hint_permutations`)

Insert after the closing `}` of `generate_hint_permutations` (after line 367), before the
`#[cfg(test)]` block (line 369):

```rust
/// Ensures `kb.hint_permutations` is populated and up-to-date.
///
/// count_needed is `hints.len()^2` (the full r=2 space). Regeneration is triggered when
/// `hint_permutations` is empty or its length does not match count_needed (staleness).
pub fn ensure_hint_permutations(kb: &mut KeyBindings) {
    let n = kb.hints.len();
    if n == 0 {
        return;
    }
    let count_needed = n * n;
    if kb.hint_permutations.len() == count_needed {
        return; // already fresh
    }
    kb.hint_permutations = generate_hint_permutations(&kb.hints, count_needed);
}
```

### Step 2 - Call `ensure_hint_permutations` from `AppData::load` (data.rs, lines 252-275)

After the `keybindings` binding is resolved (the `if kb_path.exists() { ... } else { ... }` block
ending at line 264), add one line before the `Ok(Self {` at line 266:

```rust
        ensure_hint_permutations(&mut keybindings);
```

The resulting block (lines 252-275) should look like:

```rust
        let kb_path = data_dir.join("keybindings.yml");
        let mut keybindings = if kb_path.exists() {
            let kb_content = fs::read_to_string(&kb_path)?;
            match serde_yaml::from_str(&kb_content) {
                Ok(kb) => kb,
                Err(e) => {
                    eprintln!("Warning: keybindings.yml parse error ({}), using defaults", e);
                    KeyBindings::default()
                }
            }
        } else {
            KeyBindings::default()
        };

        ensure_hint_permutations(&mut keybindings);

        Ok(Self {
            ...
        })
```

Note: `keybindings` must be `let mut keybindings` (add `mut`).

### Step 3 - ~~Add unit tests~~ ALREADY DONE (pre-written by Test Writer in Red phase)

The three tests below were already inserted into data.rs at lines 488-543 during the Red phase.
**Do NOT re-insert them.** They reference `ensure_hint_permutations` which will be resolved by Step 1.

```rust
    // ---- ensure_hint_permutations tests (Task #23 sub-task 2) ----

    /// Regeneration is triggered when hint_permutations is empty.
    #[test]
    fn ensure_hint_permutations_populates_when_empty() {
        let mut kb = KeyBindings::default(); // hint_permutations = []
        assert!(kb.hint_permutations.is_empty(), "precondition: starts empty");
        ensure_hint_permutations(&mut kb);
        let expected_count = kb.hints.len() * kb.hints.len();
        assert_eq!(
            kb.hint_permutations.len(),
            expected_count,
            "hint_permutations should be populated after ensure call"
        );
    }

    /// Regeneration is triggered when hints list changes (staleness by hints.len() change).
    #[test]
    fn ensure_hint_permutations_regenerates_when_hints_change() {
        let mut kb = KeyBindings::default();
        ensure_hint_permutations(&mut kb);
        let original_len = kb.hint_permutations.len();

        // Simulate hints list change: add an extra hint
        kb.hints.push("z".to_string());
        // hint_permutations.len() is now stale (doesn't equal new hints.len()^2)
        ensure_hint_permutations(&mut kb);

        let new_expected = kb.hints.len() * kb.hints.len();
        assert_ne!(
            kb.hint_permutations.len(),
            original_len,
            "hint_permutations should be regenerated after hints list change"
        );
        assert_eq!(
            kb.hint_permutations.len(),
            new_expected,
            "regenerated hint_permutations should match new count_needed"
        );
    }

    /// No regeneration when hint_permutations is already fresh (idempotent).
    #[test]
    fn ensure_hint_permutations_no_regen_when_fresh() {
        let mut kb = KeyBindings::default();
        ensure_hint_permutations(&mut kb);
        let populated = kb.hint_permutations.clone();

        // Call again - should not change anything
        ensure_hint_permutations(&mut kb);
        assert_eq!(
            kb.hint_permutations,
            populated,
            "ensure_hint_permutations should be idempotent when already fresh"
        );
    }
```

## Verification

### Manual tests
- Run the app; confirm keybindings.yml gets a populated `hint_permutations:` list written (once
  sub-task 3 adds write-back). For now, verify in tests only.

### Automated tests

Test names and what they verify:
- `ensure_hint_permutations_populates_when_empty` - empty vec triggers generation; length equals hints.len()^2
- `ensure_hint_permutations_regenerates_when_hints_change` - adding a hint to kb.hints makes the
  cached count stale; ensure call regenerates to new size
- `ensure_hint_permutations_no_regen_when_fresh` - calling ensure twice leaves hint_permutations
  identical (idempotent / no spurious regeneration)

Cargo test command:
```
PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" /c/Users/solar/.cargo/bin/cargo.exe test --manifest-path "/c/scribble/Cargo.toml"
```

All existing tests must continue to pass.

## Changelog
### Plan - 2026-03-30
- Initial plan
- Prefect R1: Step 3 (tests) marked ALREADY DONE — tests were pre-written in Red phase at data.rs lines 488-543; implementer must skip re-insertion.
