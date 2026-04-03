## Task

#49 - Add repeat_limit: N to multi_field fields (sub-task 2: HeaderState repeat mechanics)

## Context

`HeaderState` in `src/sections/header.rs` currently stores one `String` per field in `values: Vec<String>` and unconditionally increments `field_index` in `advance()`. Sub-task 2 replaces `values` with `repeated_values: Vec<Vec<String>>` (an ordered list of confirmed strings per slot) and adds `repeat_counts: Vec<usize>` so `advance()` can check the current field's `repeat_limit` before deciding whether to re-queue or proceed. The pre-written TDD tests in header.rs (ST49-2-TEST-1 through ST49-2-TEST-11) and the existing app.rs/note.rs/ui.rs call sites all need to compile and pass against the new shape.

## Approach

Rewrite the `HeaderState` struct fields and its three core methods in `src/sections/header.rs`, then update every call site that reads or writes the old `values: Vec<String>` field in `src/app.rs`, `src/note.rs`, and `src/ui.rs`. The `set_current_value` method changes from overwrite to append. `advance()` consults `field_configs[field_index].repeat_limit`; if a limit exists and `repeat_counts[field_index] < limit`, it increments the counter and returns `false` without changing `field_index`; otherwise it increments `field_index` as before. `go_back()` clears `repeat_counts[new_index]` (the slot being returned to) when it moves back.

## Critical Files

- `src/sections/header.rs` - struct definition and all method bodies (lines 3-51)
- `src/app.rs` - call sites reading `s.values` (line 610), test assertion `s.values[0]` (line 1439), `s.set_current_value` (composite go-back lines 1306, 1314; confirm_modal_value lines 820, 828, 840)
- `src/note.rs` - `render_note` has_any check zips `hs.values.iter()` (line 98); `format_header_preview` and `format_header_export` both zip `hs.values.iter()` (lines 386, 414); test helper `make_header_state` sets `hs.values.get_mut(i)` (line 491); direct index assignments `hs.values[1]` etc. (lines 527, 571-574)
- `src/ui.rs` - reads `&state.values[i]` at line 313

## Reuse

- `HeaderFieldConfig::repeat_limit: Option<usize>` already added in sub-task 1 (src/data.rs)
- Existing `advance()` / `go_back()` / `set_current_value()` signatures are preserved; only bodies change
- All eleven pre-written tests in `header.rs` (ST49-2-TEST-1 through ST49-2-TEST-11) drive correctness

## Steps

1. **Rewrite `HeaderState` struct** (`src/sections/header.rs`, lines 3-11):
```diff
-pub struct HeaderState {
-    pub field_configs: Vec<HeaderFieldConfig>,
-    pub values: Vec<String>,
-    pub field_index: usize,
-    pub completed: bool,
-    pub composite_spans: Option<Vec<(String, bool)>>,
-}
+pub struct HeaderState {
+    pub field_configs: Vec<HeaderFieldConfig>,
+    pub repeated_values: Vec<Vec<String>>,
+    pub repeat_counts: Vec<usize>,
+    pub field_index: usize,
+    pub completed: bool,
+    pub composite_spans: Option<Vec<(String, bool)>>,
+}
```

2. **Rewrite `HeaderState::new()`** (lines 14-23):
```diff
-        Self {
-            field_configs: fields,
-            values: vec![String::new(); n],
-            field_index: 0,
-            completed: false,
-            composite_spans: None,
-        }
+        Self {
+            field_configs: fields,
+            repeated_values: vec![Vec::new(); n],
+            repeat_counts: vec![0; n],
+            field_index: 0,
+            completed: false,
+            composite_spans: None,
+        }
```

3. **Rewrite `set_current_value()`** (lines 25-29) - append instead of overwrite:
```diff
-    pub fn set_current_value(&mut self, value: String) {
-        if let Some(v) = self.values.get_mut(self.field_index) {
-            *v = value;
-        }
-    }
+    pub fn set_current_value(&mut self, value: String) {
+        if let Some(slot) = self.repeated_values.get_mut(self.field_index) {
+            slot.push(value);
+        }
+    }
```

4. **Rewrite `advance()`** (lines 32-38) - check repeat_limit before advancing:
```diff
-    pub fn advance(&mut self) -> bool {
-        self.field_index += 1;
-        if self.field_index >= self.field_configs.len() {
-            self.completed = true;
-        }
-        self.completed
-    }
+    pub fn advance(&mut self) -> bool {
+        let limit = self.field_configs
+            .get(self.field_index)
+            .and_then(|cfg| cfg.repeat_limit);
+        if let Some(lim) = limit {
+            let count = self.repeat_counts[self.field_index] + 1;
+            self.repeat_counts[self.field_index] = count;
+            if count < lim {
+                return false; // re-queue: stay at same field_index
+            }
+        }
+        self.field_index += 1;
+        if self.field_index >= self.field_configs.len() {
+            self.completed = true;
+        }
+        self.completed
+    }
```

5. **Rewrite `go_back()`** (lines 41-48) - clear repeat_count AND repeated_values for the destination slot:
```diff
-    pub fn go_back(&mut self) -> bool {
-        if self.field_index > 0 {
-            self.field_index -= 1;
-            true
-        } else {
-            false
-        }
-    }
+    pub fn go_back(&mut self) -> bool {
+        if self.field_index > 0 {
+            self.field_index -= 1;
+            self.repeat_counts[self.field_index] = 0;
+            if let Some(slot) = self.repeated_values.get_mut(self.field_index) {
+                slot.clear();
+            }
+            true
+        } else {
+            false
+        }
+    }
```

6. **Update `src/app.rs` call sites** - replace all reads of `s.values` with a compatibility shim. For the `super_confirm` path (line 610) and note preview `s.values.get(s.field_index)`, derive the "current value" from the last element of the current slot (or empty string):
```diff
-                    let confirmed = s.values.get(s.field_index).map(|v| v.as_str()).unwrap_or("");
+                    let confirmed = s.repeated_values.get(s.field_index)
+                        .and_then(|v| v.last())
+                        .map(|v| v.as_str())
+                        .unwrap_or("");
```
   For composite go-back calls to `s.set_current_value(String::new())` at line 1306: this will now push an empty string to the slot. To preserve the intent (clear the in-progress preview rather than accumulate), set the slot directly instead of calling set_current_value:
```diff
-                s.set_current_value(String::new());
+                if let Some(slot) = s.repeated_values.get_mut(s.field_index) {
+                    slot.clear();
+                }
```
   Update the two in-progress composite preview calls `s.set_current_value(preview)` at lines 820 and 1314 - these write a preview that should replace the last entry rather than accumulate. Introduce a `set_preview_value` helper on `HeaderState` that overwrites the last entry (or pushes if the slot is empty). Call it for composite in-progress previews only; keep `set_current_value` for final confirmed values. The replacement at both sites looks like:
```diff
-                        s.set_current_value(preview);
+                        s.set_preview_value(preview);
```
   Implement `set_preview_value` as a second method on `HeaderState`:
```diff
+    /// Overwrite (or set) the last entry in the current slot's vec for live previews.
+    /// Does not add a new confirmed entry.
+    pub fn set_preview_value(&mut self, value: String) {
+        if let Some(slot) = self.repeated_values.get_mut(self.field_index) {
+            if slot.is_empty() {
+                slot.push(value);
+            } else {
+                *slot.last_mut().unwrap() = value;
+            }
+        }
+    }
```
   Then replace preview-only calls (composite NextPart and composite_go_back non-zero part) to use `set_preview_value` instead of `set_current_value`. Confirmed final values (CompositeAdvance::Complete, simple modal confirm, super_confirm) continue to use `set_current_value`. For `CompositeAdvance::Complete`, clear the slot first so the preview entry left by `set_preview_value` does not remain alongside the confirmed value:
```diff
  CompositeAdvance::Complete(final_value) => {
      if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
          s.composite_spans = None;
+         if let Some(slot) = s.repeated_values.get_mut(s.field_index) {
+             slot.clear();
+         }
          s.set_current_value(final_value);
```
  (`src/app.rs`, around line 825-828)
   Also update the test assertion at line 1439 in the `super_confirm_fills_default_and_advances` test:
```diff
-            assert_eq!(s.values[0], "hello", "field 0 should be filled with its default");
+            assert_eq!(s.repeated_values[0].last().map(|s| s.as_str()).unwrap_or(""), "hello", "field 0 should be filled with its default");
```

7. **Update `src/note.rs`** - replace all `hs.values.iter()` zips and direct `hs.values[i]` index writes with the new shape. There is also a `has_any` check at line 98 (inside `render_note`) that zips `hs.values.iter()` and must be updated first:
```diff
-                        let has_any = hs.field_configs.iter().zip(hs.values.iter()).any(|(fcfg, confirmed)| {
-                            !resolve_multifield_value(confirmed, fcfg, sticky_values).is_empty_variant()
-                        });
+                        let has_any = hs.field_configs.iter().enumerate().any(|(i, fcfg)| {
+                            let confirmed = hs.repeated_values.get(i)
+                                .and_then(|v| v.last())
+                                .map(|s| s.as_str())
+                                .unwrap_or("");
+                            !resolve_multifield_value(confirmed, fcfg, sticky_values).is_empty_variant()
+                        });
```
   In `format_header_preview` (line 386), update the `field_preview` closure to use `repeated_values[index].last()`:
```diff
  // src/note.rs:386 (inside field_preview closure in format_header_preview)
-            .zip(hs.values.iter())
-            .find(|(cfg, _)| cfg.id == id)
-            .map(|(cfg, confirmed)| {
-                resolve_multifield_value(confirmed, cfg, sticky_values)
+            .enumerate()
+            .find(|(_, cfg)| cfg.id == id)
+            .map(|(i, cfg)| {
+                let confirmed = hs.repeated_values.get(i)
+                    .and_then(|v| v.last())
+                    .map(|s| s.as_str())
+                    .unwrap_or("");
+                resolve_multifield_value(confirmed, cfg, sticky_values)
```
   Apply the same change in `format_header_export` (line 414), updating the `field_export` closure identically:
```diff
  // src/note.rs:414 (inside field_export closure in format_header_export)
-            .zip(hs.values.iter())
-            .find(|(cfg, _)| cfg.id == id)
-            .and_then(|(cfg, confirmed)| {
-                resolve_multifield_value(confirmed, cfg, sticky_values)
+            .enumerate()
+            .find(|(_, cfg)| cfg.id == id)
+            .and_then(|(i, cfg)| {
+                let confirmed = hs.repeated_values.get(i)
+                    .and_then(|v| v.last())
+                    .map(|s| s.as_str())
+                    .unwrap_or("");
+                resolve_multifield_value(confirmed, cfg, sticky_values)
``` In the test helper `make_header_state`, change to push into `repeated_values`:
```diff
-        for (i, val) in values.iter().enumerate() {
-            if let Some(v) = hs.values.get_mut(i) {
-                *v = val.to_string();
-            }
-        }
+        for (i, val) in values.iter().enumerate() {
+            if let Some(slot) = hs.repeated_values.get_mut(i) {
+                if !val.is_empty() {
+                    slot.push(val.to_string());
+                }
+            }
+        }
```
   Update `hs.values[1] = "60".to_string();` (line 527) to push into repeated_values:
```diff
-        hs.values[1] = "60".to_string();
+        hs.repeated_values[1].push("60".to_string());
```
   Update the four-field assignments (lines 571-574) the same way:
```diff
-        hs.values[0] = "2026-04-02".to_string();
-        hs.values[1] = "13:00".to_string();
-        hs.values[2] = "60".to_string();
-        hs.values[3] = "Treatment focused massage".to_string();
+        hs.repeated_values[0].push("2026-04-02".to_string());
+        hs.repeated_values[1].push("13:00".to_string());
+        hs.repeated_values[2].push("60".to_string());
+        hs.repeated_values[3].push("Treatment focused massage".to_string());
```

8. **Update `src/ui.rs`** (line 313) - derive display value from last entry of the slot:
```diff
-        let value = &state.values[i];
+        let last_val;
+        let value: &str = {
+            last_val = state.repeated_values.get(i)
+                .and_then(|v| v.last())
+                .map(|s| s.as_str())
+                .unwrap_or("");
+            last_val
+        };
```
   Also update `has_value` check (line 315) - it currently calls `!value.is_empty()` which still works after the above change.

9. **Run `cargo test`** and fix any remaining compilation errors or test failures before marking complete.

## Verification

### Manual tests

- Launch the app and navigate to a multi_field header section. Confirm a field value in a simple (non-repeat) field; verify the field advances normally and the entered value is shown in the field block.
- Set `repeat_limit: 2` on a test field in sections.yml. Confirm a value; verify the same field re-appears (field_index stays). Confirm a second value; verify field advances. Back-navigate to the field; verify the counter is reset and you can enter a new value.
- Composite fields: confirm a multi-part composite value; verify the final assembled value appears in the header block (not duplicate previews).

### Automated tests

- `cargo test` - all eleven ST49-2 tests in `src/sections/header.rs` must pass, plus all existing tests in `src/app.rs` and `src/note.rs`.
- The two app.rs integration tests (`super_confirm_fills_default_and_advances`, `super_confirm_no_op_when_no_default`) must be updated: `s.values[0]` becomes `s.repeated_values[0].last().map(|s| s.as_str()).unwrap_or("")`, or the assertions rewritten to check `repeated_values[0]` directly.

## Prefect-2 Report

### Issues found (R2)

**Minor**

- **#P6** `src/sections/header.rs` (missing placement label for `set_preview_value`) - Step 6 is titled "Update `src/app.rs` call sites" and introduces the `set_preview_value` method via a `+` diff block, but provides no explicit instruction to add it to `src/sections/header.rs`. An implementer following the steps may look only in app.rs and miss that this is a new method on `HeaderState`. The diff block should be prefaced with "Add the following method to `src/sections/header.rs`'s `impl HeaderState` block:" or a separate numbered step should be created for it.

- **#P7** `src/app.rs:636-639` (out-of-bounds back-navigation path bypasses `go_back()` cleanup) - The out-of-bounds normalization block (lines 636-639) fires when `field_index >= field_configs.len()` (i.e., re-entering a completed header). It resets `field_index` directly without calling `go_back()`, so after Step 5's changes to `go_back()` this path will NOT clear `repeated_values` or `repeat_counts` for the last field. Re-entering a completed header leaves the last slot dirty. The plan should include the following diff for `src/app.rs` lines 636-639:
```diff
                if s.field_index >= s.field_configs.len() && !s.field_configs.is_empty() {
                    s.field_index = s.field_configs.len() - 1;
+                   s.repeat_counts[s.field_index] = 0;
+                   if let Some(slot) = s.repeated_values.get_mut(s.field_index) {
+                       slot.clear();
+                   }
                    s.completed = false;
                    true
```
  (`src/app.rs`, around line 636-639)

**Nit**

- **#P8** Verification section (line 286) - States "The two app.rs integration tests (`super_confirm_fills_default_and_advances`, `super_confirm_no_op_when_no_default`) must be updated" but `super_confirm_no_op_when_no_default` does not reference `s.values` at all (it only asserts `s.field_index`). Only `super_confirm_fills_default_and_advances` (app.rs:1439) needs updating.

### Issues found (R1)

**Minor**

- **#P3** `src/app.rs:828` (composite `Complete` branch) - Stale preview entry left in slot after composite confirm. When `CompositeAdvance::NextPart` fires, `set_preview_value` pushes one entry into the slot. When `CompositeAdvance::Complete` fires immediately after, the plan keeps `set_current_value(final_value)` which **appends** onto that preview entry, leaving the slot as `["<preview_string>", "<final_confirmed>"]`. All `last()` lookups return the correct value, but the slot accumulates a stale entry for every composite field confirm. The fix is to clear the slot before calling `set_current_value` in the `Complete` arm:
```diff
  CompositeAdvance::Complete(final_value) => {
      if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
          s.composite_spans = None;
+         if let Some(slot) = s.repeated_values.get_mut(s.field_index) {
+             slot.clear();
+         }
          s.set_current_value(final_value);
```
  (`src/app.rs`, around line 825-828)

- **#P4** `src/sections/header.rs` Step 5 (`go_back()`) - `go_back()` clears `repeat_counts[field_index]` but does NOT clear `repeated_values[field_index]`. After going back to a slot and re-entering a value, `set_current_value` appends to the existing entries, accumulating stale values across go-back cycles. The `last()` lookup still returns the correct most-recent entry for display, but old entries pile up indefinitely. Step 5 should also clear the slot:
```diff
  pub fn go_back(&mut self) -> bool {
      if self.field_index > 0 {
          self.field_index -= 1;
          self.repeat_counts[self.field_index] = 0;
+         if let Some(slot) = self.repeated_values.get_mut(self.field_index) {
+             slot.clear();
+         }
          true
```
  (`src/sections/header.rs`, around line 118-122)

- **#P5** `src/note.rs:386` and `src/note.rs:414` - Step 7 provides only one diff block for the `.zip`-to-`.enumerate()` migration but applies it to two distinct closures (`field_preview` inside `format_header_preview` and `field_export` inside `format_header_export`). The diff text says "Update both ... the same way" but only shows the change once without labeling which function. An implementer applying diffs mechanically may only update one. The plan should provide two labeled diffs (one per function) or explicitly label the single diff as applying to both closures at lines 386 and 414.

## Prefect-1 Report

### Issues found and fixed (R1)

**Minor**
- **#P1** `src/note.rs:98` - Missing call site: `render_note` `has_any` check uses `hs.values.iter()` zip but was not listed in Critical Files or Step 7. Added diff to Step 7 and updated Critical Files entry. (note.rs:98)
- **#P2** `src/app.rs:1439` - Missing call site: test assertion `s.values[0]` in `super_confirm_fills_default_and_advances` was mentioned in Verification but had no diff in Step 6. Added diff to Step 6. (app.rs:1439)

## Changelog

### Review - 2026-04-03
- #1: Step 6 - removed dangling/broken diff that referenced non-existent `s.current_preview_slot()` method; replaced with clear `set_preview_value` call diff and description
- #2: Critical Files - removed stale reference to note.rs line 491 from app.rs entry (line 491 is in note.rs, not app.rs)

### Review - 2026-04-03 (Prefect R1)
- #P1: Critical Files + Step 7 - added missing note.rs line 98 `has_any` call site with update diff
- #P2: Step 6 - added missing diff for app.rs line 1439 test assertion `s.values[0]` -> `repeated_values` form

### Review - 2026-04-03 (Reviewer R2)
- #P3: Step 6 - added slot.clear() diff for CompositeAdvance::Complete branch before set_current_value to prevent stale preview entry accumulation
- #P4: Step 5 - added slot.clear() to go_back() diff so repeated_values is also cleared on back-navigation
- #P5: Step 7 - split single .zip-to-.enumerate() diff into two labeled diffs (format_header_preview at line 386, format_header_export at line 414) with correct and_then/map chain for the export closure

### Review - 2026-04-03 (Reviewer #4 R2)
- #R4-1: Step 7 - added explicit diff for the four-field assignments at note.rs lines 571-574 (hs.values[0-3] -> hs.repeated_values[0-3].push()); previously only the line-527 single-field example was shown, leaving the four-field block undiffed

## Progress
- Step 1: Rewrote HeaderState struct - replaced values with repeated_values and added repeat_counts
- Step 2: Rewrote new() - initializes repeated_values as Vec<Vec<String>> and repeat_counts as Vec<usize>
- Step 3: Rewrote set_current_value() to append to slot; added set_preview_value() method on HeaderState
- Step 4: Rewrote advance() to check repeat_limit before advancing field_index
- Step 5: Rewrote go_back() to clear repeat_counts and repeated_values for destination slot
- Step 6: Updated app.rs call sites - super_confirm, out-of-bounds back-nav (P7), composite preview/complete/go-back, test assertion
- Step 7: Updated note.rs - has_any check, format_header_preview, format_header_export, make_header_state helper, all direct index writes
- Step 8: Updated ui.rs - derive display value from repeated_values.last(); fixed value.clone() to value.to_string() for type compatibility
- Step 9: cargo test - all 136 tests pass (including all 11 ST49-2 tests)

## Implementation
Complete - 2026-04-03
