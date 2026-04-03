**Task**: #51 Move hard-coded section metadata into sections.yml

**Context**: `SectionConfig` in `src/data.rs` (lines 101-114) is missing four fields that
downstream logic and TDD tests require. Four tests at `src/data.rs:1968` currently produce
compile errors because the fields do not exist. After the struct is extended and
`data/sections.yml` is populated for the two target sections (`adl`, `tx_mods`), all four
tests must pass with zero regressions in the full suite.

**Approach**: Add four new fields to `SectionConfig` in `src/data.rs` with serde defaults.
Then populate the corresponding values in `data/sections.yml` for the `adl` and `tx_mods`
sections so the runtime assertions in the TDD tests resolve to their expected values.
No loader changes needed -- serde handles deserialization automatically for these simple types.

**Critical Files**:
- `src/data.rs` lines 101-114 (`SectionConfig` struct -- add 4 fields)
- `src/data.rs` lines 1968+  (ST51-1 tests -- must compile and pass after change)
- `data/sections.yml` (populate new fields for `adl` and `tx_mods` sections)

**Steps**:

1. **Extend `SectionConfig` in `src/data.rs`** -- add four fields after `pub fields: Option<Vec<HeaderFieldConfig>>`:

```diff
 pub composite: Option<CompositeConfig>,
 pub fields: Option<Vec<HeaderFieldConfig>>,
+    #[serde(default)]
+    pub is_intake: bool,
+    #[serde(default)]
+    pub heading_search_text: Option<String>,
+    #[serde(default)]
+    pub heading_label: Option<String>,
+    #[serde(default)]
+    pub note_render_slot: Option<String>,
 }
```

   - `#[serde(default)]` on `bool` defaults to false when key absent from YAML
   - `#[serde(default)]` on `Option<String>` defaults to None when key absent from YAML
   - No other struct changes; no loader changes.

2. **Populate `data/sections.yml`** for the two sections the tests inspect:

   For the `adl` section, add:
   ```yaml
   is_intake: true
   heading_label: "#### ACTIVITIES OF DAILY LIVING"
   ```

   For the `tx_mods` section, add:
   ```yaml
   heading_search_text: "TREATMENT MODIFICATIONS"
   note_render_slot: "tx_mods"
   ```

   All other sections need no changes; serde defaults supply false/None automatically.

3. Run `cargo test adl_is_intake_is_true`, `cargo test tx_mods_heading_search_text_is_set`, `cargo test adl_heading_label_is_set`, `cargo test tx_mods_note_render_slot_is_set` individually to confirm all pass.

4. Run `cargo test --manifest-path "C:/Users/solar/Documents/Claude Projects/scribblenot/Cargo.toml"` to confirm full suite passes.

5. Commit `src/data.rs` and `data/sections.yml` with message:
   `Implement task #51 sub-task 51.1: add metadata fields to SectionConfig`

**Verification**:

### Automated tests
- `cargo test adl_is_intake_is_true` -- currently fails (compile error), must pass after
- `cargo test tx_mods_heading_search_text_is_set` -- must pass after
- `cargo test adl_heading_label_is_set` -- must pass after
- `cargo test tx_mods_note_render_slot_is_set` -- must pass after
- `cargo test` (full suite) -- zero regressions

## Progress

Implementation complete. Commit: 847c5b5

All 4 named tests pass:
- adl_is_intake_is_true: ok
- tx_mods_heading_search_text_is_set: ok
- adl_heading_label_is_set: ok
- tx_mods_note_render_slot_is_set: ok

Full suite: 176 passed, 0 failed.

Additional changes required beyond the plan:
- `FlatBlock::Section` in `src/flat_file.rs` lacked the 4 new fields; added them with `#[serde(default)]`
- Loader reconstruction pass in `data.rs` was updated to destructure and propagate the new fields instead of hard-coding false/None
- 9 struct literal sites across `src/app.rs`, `src/data.rs`, and `src/note.rs` required the new fields to be explicitly set (Rust requires exhaustive struct literals)
