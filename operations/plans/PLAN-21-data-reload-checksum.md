## Task
#21 - Skip full data rebuild in `reload_data()` when YAML content is unchanged

## Context
`reload_data()` unconditionally reads all YAML files, parses them, rebuilds the runtime template,
navigation, section states, and editable document on every Ctrl+R — even when nothing on disk has
changed. This pays the full `AppData::load()` derivation cost (file I/O + YAML parsing + template
walk) for a no-op. Caching a hash of the loader inputs lets `reload_data()` bail before the YAML
parse, runtime rebuild, and state-reset work starts.

The three triggers for modal-unit layout derivation are modal open, YAML reload, and viewport
resize. This plan addresses the YAML reload trigger. The resize trigger is already cheap (geometry
pass only, no data read). The open trigger is unavoidable.

## Approach
Add `data_content_hash: Option<u64>` to `App`. Add a private `hash_app_data_inputs(dir) ->
io::Result<u64>` helper in `src/app.rs` that mirrors the actual input boundary of `AppData::load`:

- hash every `.yml` file that `read_hierarchy_dir()` would read
- include `keybindings.yml` explicitly if it exists
- process files in sorted path order, just like the loader
- hash each file's relative path plus its bytes so file renames, reorderings, and file-boundary
  changes invalidate the cache instead of only raw concatenated bytes

In `reload_data()`, compute the fresh hash first; if it matches the stored value, return `Ok(())`
immediately. If it differs (or no hash is stored yet), run the existing full load body and store
the new hash only after the reload succeeds.

No new crate dependency is needed. `std::hash::Hasher` +
`std::collections::hash_map::DefaultHasher` is deterministic within a session, which is enough for
within-session no-op detection. The win here is not "zero I/O" - the checksum helper still reads
the loader inputs - but skipping the more expensive YAML parse, runtime rebuild, navigation reset,
document rebuild, and modal-unit recalculation when the inputs are unchanged.

Important safety note: today `read_hierarchy_dir()` scans every non-`keybindings.yml` `.yml` file
in `data_dir`, including files that are semantically inert for hierarchy building such as
`config.yml` and `default-theme.yml`. That is not ideal product scoping, but the checksum must stay
aligned with the current loader behavior for this task. Otherwise the fast path could incorrectly
skip a reload that `AppData::load()` would have treated as changed or even errored on. If the team
wants to stop treating config/theme YAML as data-loader inputs, that should be a separate
loader-scope cleanup done in lockstep with the checksum boundary.

If `hash_app_data_inputs` returns an I/O error, `new_hash` is `None`, the stored hash is not
consulted, and the existing load path runs unchanged so the user still gets the real
`AppData::load()` error instead of a silent cache decision.

## Critical Files
- `src/app.rs`:
  - `App` struct (~line 271): add `data_content_hash: Option<u64>`
  - helper area near `reload_data()` (~line 1382): add `hash_app_data_inputs`
  - `App::new()` (~line 378): initialize field with `hash_app_data_inputs(&data_dir).ok()`; note that
    `data_dir` is moved into the struct at line 393, so compute the hash before that point
  - `reload_data()` (~line 1382): add hash check at the top; store new hash at the bottom
- `src/app.rs` tests (`mod tests` near file end): add focused checksum-scope coverage

## Steps

### Step 1 - Add `data_content_hash` field to `App`

In `src/app.rs`, in the `App` struct (after `modal_unit_layout`, around line 299), add:

```rust
/// Hash of the last successful loader input set, for no-op reload detection.
data_content_hash: Option<u64>,
```

The field is private (no `pub`) because it is an internal cache with no external readers.

### Step 2 - Add `hash_app_data_inputs` free function

Add this private function near `reload_data()` in `src/app.rs`:

```rust
fn hash_app_data_inputs(dir: &std::path::Path) -> std::io::Result<u64> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    let mut entries = std::fs::read_dir(dir)?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|e| e.path());
    let mut hasher = DefaultHasher::new();
    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("keybindings.yml") {
            continue;
        }
        hash_loader_input(&mut hasher, dir, &path)?;
    }
    let keybindings_path = dir.join("keybindings.yml");
    if keybindings_path.exists() {
        hash_loader_input(&mut hasher, dir, &keybindings_path)?;
    }
    Ok(hasher.finish())
}
```

Add a tiny private helper such as `hash_loader_input(...)` so each file contributes:

- its relative path from `data_dir`
- a separator / length marker
- the file bytes

That keeps the hash aligned with loader ordering semantics instead of treating the directory like
one anonymous byte stream.

Important scope rule: mirror the current loader exactly. With today's code, that means all
non-`keybindings.yml` `.yml` files plus `keybindings.yml`, even when some of those files are only
incidentally tolerated by the hierarchy loader.

### Step 3 - Initialize the field in `App::new()`

In `App::new()`, compute the hash before `data_dir` is moved into the struct. After the
`ui_theme` initialization block (around line 376) and before `Self { ... }`, add:

```rust
let data_content_hash = hash_app_data_inputs(&data_dir).ok();
```

Then in the `Self { ... }` literal, after `modal_unit_layout: None,` (around line 403), add:

```rust
    data_content_hash,
```

### Step 4 - Update `reload_data()` to use the hash

In `reload_data()` (~line 1382), insert a fast-path check at the very top, before
`previous_section_id` is computed:

```rust
pub fn reload_data(&mut self) -> anyhow::Result<()> {
    let new_hash = hash_app_data_inputs(&self.data_dir).ok();
    if let (Some(old), Some(new)) = (self.data_content_hash, new_hash) {
        if old == new {
            return Ok(());
        }
    }

    // existing body starts here ...
    let previous_section_id = ...
```

At the end of `reload_data()`, just before the final `Ok(())`, store the new hash:

```rust
    self.data_content_hash = new_hash;
    Ok(())
}
```

`new_hash` is still in scope because it was bound at the top of the function.

### Step 5 - Add targeted tests for checksum scope

Add focused tests in `src/app.rs` so the cache boundary is locked to real loader inputs:

- changing a hierarchy YAML file changes `hash_app_data_inputs`
- changing `keybindings.yml` changes `hash_app_data_inputs`
- changing `config.yml` or `default-theme.yml` also changes `hash_app_data_inputs`, matching the
  current loader scope rather than an idealized future scope

If the implementation stays simple enough, also add one app-level regression test that:

- builds an `App` from a temp data dir
- mutates some in-memory state that a real reload would normally clear
- calls `reload_data()` without changing files
- asserts that the in-memory state is preserved, proving the no-op fast path actually fired

## Verification

### Automated tests
```
cargo test
```
All existing tests must pass, plus the new focused checksum tests above. Most existing
test-constructed `App` instances use `PathBuf::new()`, so the new field should naturally be `None`
there and remain behaviorally inert.

### Manual tests
```
cargo run
```
1. Open the app. Press Ctrl+R. Confirm the status bar shows "Data refreshed from YAML." (hash was
   stored during `new()`, so `reload_data()` should short-circuit on this first press if no loader
   inputs have changed; the success message still appears because the caller sets it on any
   `Ok(())`).
2. Without editing any YAML, press Ctrl+R again. Confirm the message still appears and the app
   state is unchanged.
3. Edit a real app-data input such as `data/sections.yml` or `data/keybindings.yml`, save, then
   press Ctrl+R. Confirm the app picks up the change correctly.
4. Revert that edit and press Ctrl+R again. Confirm the app reloads again because the input hash
   changed back.
5. Optional safety check: edit `data/config.yml` or `data/default-theme.yml`, press Ctrl+R, and
   confirm the app still takes the reload path instead of the no-op fast path. That matches the
   current loader scope and avoids hiding loader-visible changes behind the checksum gate.
