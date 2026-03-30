## Task

#2 - Add Shift+Enter super-confirm keybinding to auto-complete remaining fields

## Context

The modal path for super-confirm is complete. When the user presses Shift+Enter while a header section is focused but no modal is open, the wizard should auto-fill the current field using the best available value (sticky first, then field default), advance to the next field, and repeat until all fields are filled or a field has no resolvable value. If the current field has no resolvable value, skip it silently and leave the wizard on that field for manual input.

## Approach

Add an `is_super_confirm` branch near the top of `handle_header_key`, before the hint-key block. Extract the focused field config and resolve a value via sticky lookup (`self.config.sticky_values.get(&field.id)`) then `field.default`. If a value is found, call `set_current_value` + `advance()`. If `advance()` returns true, call `advance_section()` and return. If no value is found, return without changing state (the user must open the modal manually). Do not loop over subsequent fields - fire once per keypress, consistent with how one Shift+Enter press confirms one modal in the modal path.

## Critical Files

- `src/app.rs` - `handle_header_key` function, lines 490-581
- `src/sections/header.rs` - `set_current_value`, `advance` methods, lines 25-38
- `src/config.rs` - `sticky_values: HashMap<String, String>`, line 11
- `src/data.rs` - `HeaderFieldConfig.id`, `HeaderFieldConfig.default`, lines 71-78

## Reuse

- `self.is_super_confirm(&key)` - already defined on `impl App` in `src/app.rs` line 228
- `HeaderState::set_current_value(&mut self, value: String)` - `src/sections/header.rs` line 25
- `HeaderState::advance(&mut self) -> bool` - `src/sections/header.rs` line 32
- `self.advance_section()` - `src/app.rs` line 476
- `self.config.sticky_values` - `HashMap<String, String>` keyed by field id for simple fields

## Steps

1. In `handle_header_key` (src/app.rs, line 490), add a super-confirm branch immediately after the hint-key block ends (after line 541) and before the `is_back` check (line 544):

```diff
+        if self.is_super_confirm(&key) {
+            let idx = self.current_idx;
+            // Resolve value: sticky first, then field default
+            let resolved = if let Some(SectionState::Header(s)) = self.section_states.get(idx) {
+                s.field_configs.get(s.field_index).and_then(|cfg| {
+                    self.config.sticky_values.get(&cfg.id).cloned()
+                        .or_else(|| cfg.default.clone())
+                })
+            } else {
+                None
+            };
+            if let Some(value) = resolved {
+                if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
+                    s.set_current_value(value);
+                    let done = s.advance();
+                    if done {
+                        self.advance_section();
+                    }
+                }
+            }
+            return;
+        }
```

2. Add unit tests to the `#[cfg(test)]` block in `src/app.rs` (after the existing three tests):

```rust
    #[test]
    fn super_confirm_fills_default_and_advances() {
        use crate::data::{AppData, HeaderFieldConfig, KeyBindings, SectionConfig, SectionGroup};
        use crate::config::Config;
        use std::path::PathBuf;

        let fields = vec![
            HeaderFieldConfig { id: "f1".to_string(), name: "F1".to_string(), options: vec![], composite: None, default: Some("hello".to_string()) },
            HeaderFieldConfig { id: "f2".to_string(), name: "F2".to_string(), options: vec![], composite: None, default: None },
        ];
        let section = SectionConfig {
            id: "s1".to_string(), name: "S1".to_string(), map_label: "S1".to_string(),
            section_type: "header".to_string(), data_file: None, date_prefix: None,
            options: vec![], composite: None, fields: Some(fields),
        };
        let group = SectionGroup { id: "g1".to_string(), num: None, name: "G1".to_string(), sections: vec![section.clone()] };
        let data = AppData {
            groups: vec![group], sections: vec![section],
            list_data: Default::default(), checklist_data: Default::default(),
            region_data: Default::default(), keybindings: KeyBindings::default(),
            data_dir: PathBuf::new(),
        };
        let mut app = App::new(data, Config::default(), PathBuf::new());
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);
        app.handle_header_key(key);
        if let Some(SectionState::Header(s)) = app.section_states.get(0) {
            assert_eq!(s.values[0], "hello", "field 0 should be filled with its default");
            assert_eq!(s.field_index, 1, "field_index should advance to 1");
        } else {
            panic!("expected Header state at index 0");
        }
    }

    #[test]
    fn super_confirm_no_op_when_no_default() {
        use crate::data::{AppData, HeaderFieldConfig, KeyBindings, SectionConfig, SectionGroup};
        use crate::config::Config;
        use std::path::PathBuf;

        let fields = vec![
            HeaderFieldConfig { id: "f1".to_string(), name: "F1".to_string(), options: vec![], composite: None, default: None },
        ];
        let section = SectionConfig {
            id: "s1".to_string(), name: "S1".to_string(), map_label: "S1".to_string(),
            section_type: "header".to_string(), data_file: None, date_prefix: None,
            options: vec![], composite: None, fields: Some(fields),
        };
        let group = SectionGroup { id: "g1".to_string(), num: None, name: "G1".to_string(), sections: vec![section.clone()] };
        let data = AppData {
            groups: vec![group], sections: vec![section],
            list_data: Default::default(), checklist_data: Default::default(),
            region_data: Default::default(), keybindings: KeyBindings::default(),
            data_dir: PathBuf::new(),
        };
        let mut app = App::new(data, Config::default(), PathBuf::new());
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);
        app.handle_header_key(key);
        if let Some(SectionState::Header(s)) = app.section_states.get(0) {
            assert_eq!(s.field_index, 0, "field_index should stay at 0 when no default");
        } else {
            panic!("expected Header state at index 0");
        }
    }
```

3. Run `cargo build` to confirm no compilation errors:
   ```
   PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" /c/Users/solar/.cargo/bin/cargo.exe -C "/c/scribble" build
   ```

4. Run `cargo test` to confirm no regressions:
   ```
   PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" /c/Users/solar/.cargo/bin/cargo.exe test --manifest-path "/c/scribble/Cargo.toml"
   ```

## Verification

### Manual tests

- Open the app with a header section that has fields with defaults configured. Navigate to the header section. Press Shift+Enter: the current field should be filled with its default value and the wizard should advance to the next field.
- For a field that has a sticky value set (from a prior session), press Shift+Enter: the sticky value should be used in preference to the default.
- For a field with neither a sticky value nor a default, press Shift+Enter: nothing should happen (the field stays focused, no crash).
- Press Shift+Enter on the last field of a header section that has a default: the section should complete and the wizard should advance to the next section.

### Automated tests

- `super_confirm_fills_default_and_advances`: constructs a minimal `App` with a header section containing two fields (first has a default, second does not), simulates Shift+Enter, and asserts the first field's value equals the default and `field_index` advances to 1. Added in Step 2.
- `super_confirm_no_op_when_no_default`: constructs a minimal `App` with a single field with no default, simulates Shift+Enter, and asserts `field_index` stays at 0. Added in Step 2.

## Changelog

### Review - 2026-03-30
- #1: Added Step 2 with two unit tests (`super_confirm_fills_default_and_advances`, `super_confirm_no_op_when_no_default`) - the Verification section required these tests but no Step implemented them; renumbered old Steps 2-3 to Steps 3-4; updated Automated tests section to reference Step 2.

## Progress
- Step 1: Added `is_super_confirm` branch in `handle_header_key` (src/app.rs) after hint-key block, resolving sticky then default, calling `set_current_value` + `advance()`, then `advance_section()` if done
- Step 2: Added unit tests `super_confirm_fills_default_and_advances` and `super_confirm_no_op_when_no_default` to `#[cfg(test)]` block in src/app.rs
- Step 3: cargo check passed with no errors (cargo build blocked by running exe; check confirms clean compilation)
- Step 4: cargo test passed - 7 tests ok, 0 failed

## Implementation
Complete - 2026-03-30
