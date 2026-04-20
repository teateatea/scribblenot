---
Status: Draft
---

# Plan: Config Save Empty-Path Guard

## Context

App tests in `src/app.rs` construct `App` with `PathBuf::new()` as the data_dir, which is an empty path. When a key handler triggers `self.config.save(&self.data_dir)`, Rust resolves the empty path to `"config.yml"` relative to the current working directory. During `cargo test`, CWD is the project root, so `config.yml` is silently written there. This is noisy in the worktree and creates accidental git staging risk.

The actively compiled `composition_span_tests` module (`#[cfg(test)]` at line 3948) is the live source of the issue: its `make_app()` helper (line 4408) passes `PathBuf::new()`, and several tests exercise key handlers that call `config.save`.

## Approach

Add an early return to `Config::save()` when the provided `data_dir` is empty. `PathBuf::new().as_os_str()` is `""`, so the guard catches exactly the test pattern without touching production code paths (`find_data_dir()` always returns a real path in production).

This is a one-file, two-line change. No test code needs updating.

## Critical Files

- `src/config.rs` lines 52-57 — `Config::save()` implementation

## Reuse

- `std::ffi::OsStr::is_empty()` (available on `Path::as_os_str()`) — no new deps

## Steps

1. **Add the empty-path guard to `Config::save`** in `src/config.rs`:

```diff
     pub fn save(&self, data_dir: &Path) -> Result<()> {
+        if data_dir.as_os_str().is_empty() {
+            return Ok(());
+        }
         let path = data_dir.join("config.yml");
         let content = serde_yaml::to_string(self)?;
         fs::write(path, content)?;
         Ok(())
     }
```

2. **Verify no `config.yml` artifact is left** after running the test suite (see Verification).

## Verification

### Manual tests

- Delete any existing `config.yml` from the project root if one is present.
- Run `cargo test` from the project root.
- Confirm `config.yml` does not appear in the project root after the run: `ls config.yml` should report "No such file or directory".
- Confirm `data/config.yml` (the real runtime config) is unchanged if it existed before.

### Automated tests

No new tests are required for this change; the guard is so small it is covered by the existing `composition_span_tests` runs — if a test previously wrote `config.yml` and now does not, that is the passing signal. Optionally, add a unit test directly in `src/config.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_with_empty_path_is_noop() {
        let config = Config::default();
        let result = config.save(std::path::Path::new(""));
        assert!(result.is_ok());
        // confirm no file was written at CWD/config.yml
        assert!(!std::path::Path::new("config.yml").exists());
    }
}
```
