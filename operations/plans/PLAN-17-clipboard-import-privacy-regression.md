# Plan: Clipboard-Import Config-Persistence Privacy Regressions (#17)

**Date:** 2026-04-19
**Status:** Proposed

## Purpose

Add regression tests that verify imported patient note text is not serialized through the current config-save paths. The clipboard-import flow defined in `APPROVED-73-2-tray-hotkey-clipboard-import.md` is intentionally memory-only, and leaking patient text into persisted user config would be a high-impact privacy failure.

The tests serve a dual purpose:
- They document the current config-persistence privacy contract explicitly in code so any future change that breaks isolation fails immediately.
- They provide a temp-dir-backed harness that the Phase 2 implementation team can expand as the import feature is built.

## Background

The clipboard-import feature (`APPROVED-73-2`) is planned but not yet implemented. The relevant persistence facts as of today:

- `editable_note: String` in `App` is session-only; it is never written to `config.yml`.
- `Config::save()` at `src/config.rs:52` writes only: `pane_layout`, `theme`, `sticky_values`, `hint_labels_capitalized`, `hint_labels_case_sensitive`.
- `config.save()` is called from six sites in `src/app.rs` (lines 1399, 2221, 2262, 2287, 2323, 2846).
- No structured logging exists; `eprintln!`/`println!` are the only diagnostic outputs, and this plan does not attempt to capture or assert on process output.
- `src/import.rs` does not exist yet.
- Existing `App` tests often construct `App` with `PathBuf::new()` as `data_dir`, which resolves `config.yml` writes relative to the working directory during tests. Item `#20` addresses that specific empty-path hazard.

## Scope

- New unit tests in `src/app.rs` — verify that current `config.save()` paths do not serialize clipboard-derived note text into `config.yml`.
- New file `src/import.rs` — privacy regression tests live here alongside the future implementation; a minimal stub for `try_parse_clipboard_note` is added so the tests compile and pass.
- No changes to the import feature itself; that belongs to Phase 2 (`#73`).
- No attempt to prove a global “writes nowhere” or “logs nowhere” property. Those require broader instrumentation than this plan introduces.

## Implementation Steps

### 1. Add a temp-dir-backed app test helper

Before adding new assertions, stop relying on `PathBuf::new()` for any new tests that exercise `config.save()` paths. Use a real temporary directory so the save target is explicit and isolated.

Add a helper in the `src/app.rs` test module that:
- creates a `tempfile::TempDir`
- constructs an `App` with that directory path as `data_dir`
- returns both the `App` and the temp dir so tests can inspect the written `config.yml`

This plan depends on explicit temp-dir-backed setup rather than implicitly depending on item `#20`, though the two items are adjacent and complementary.

### 2. Add a test helper: `save_config_to_temp`

In the test module of `src/app.rs`, add a helper that:
- accepts a `Config` and a temp-dir path
- calls `Config::save()` with the temp dir path
- reads back the written `config.yml` bytes
- returns the raw file content as a `String`

This helper is the foundation for every privacy assertion: load sentinel content into a field, trigger a save, check the output.

```rust
fn save_config_to_temp(cfg: &Config, dir: &std::path::Path) -> String {
    cfg.save(dir).expect("config save failed");
    std::fs::read_to_string(dir.join("config.yml")).expect("config.yml read failed")
}
```

### 3. Add privacy assertion helper

Add a helper that panics with a useful message if sentinel text is found anywhere in a string:

```rust
fn assert_no_patient_text(haystack: &str, sentinel: &str, label: &str) {
    assert!(
        !haystack.contains(sentinel),
        "{label}: sentinel patient text found in persisted output\nsentinel: {sentinel:?}\noutput:\n{haystack}"
    );
}
```

### 4. Tests in `src/app.rs`

Add to the existing test module. Each test uses the temp-dir-backed `App` helper above so real save-triggering code writes only inside an isolated temporary directory.

#### 4a. `config_save_does_not_include_editable_note`

- Build an `App` via the new temp-dir-backed helper.
- Set `app.editable_note` to a sentinel string containing a plausible patient note fragment (e.g., `"PATIENT: Jane Doe DOB 1990-01-01 SENTINEL"`).
- Call `save_config_to_temp` directly on `app.config` (without routing through message dispatch, to isolate config structure).
- Assert the sentinel is absent from the written file.

#### 4b. `config_save_does_not_include_editable_note_after_modal_confirm`

- Build an App with a temp `data_dir`, set `editable_note` to sentinel.
- Drive at least one modal save-triggering path that currently calls `config.save(&self.data_dir)` after syncing UI state, such as a path that reaches `FieldAdvance::NextList` or `FieldAdvance::Complete`.
- Read the resulting temp `config.yml` and assert the sentinel is absent.

This test ensures a real `App` save path remains isolated from note content, not just the bare serializer.

#### 4c. `modal_save_paths_do_not_persist_editable_note`

- Build an App with a temp `data_dir`. Set `editable_note` to sentinel.
- Exercise the save-triggering app flows this plan intentionally covers:
  - one non-modal config save path such as pane swap
  - one modal path that saves on intermediate advancement
  - one modal path that saves on completion
- After each flow, read `config.yml` and assert the sentinel is absent.

#### 4d. `sticky_values_do_not_contain_imported_text_after_save`

- Build an App with a temp `data_dir`. Set `editable_note` to sentinel.
- Assert `app.config.sticky_values` does not already contain the sentinel before any save-triggering action.
- Run a save-triggering app path.
- Save or reread `config.yml` and assert sentinel absent from output.

This test remains a narrow canary for the current config shape: if a future import path incorrectly copies clipboard-derived text into `sticky_values`, config persistence will expose it here.

### 5. Create `src/import.rs`

Add the file declared in `APPROVED-73-2`. This step only adds a stub implementation with privacy-focused tests; the real heuristic belongs to Phase 2.

```rust
use crate::data::SectionConfig;

/// Try to parse clipboard text as a prior clinic note.
/// Returns the recognized note text if the content looks like a structured note,
/// or None if the clipboard content should be ignored.
pub fn try_parse_clipboard_note(
    _text: &str,
    _sections: &[SectionConfig],
) -> Option<String> {
    // Stub: always returns None until Phase 2 implementation.
    None
}
```

Add `pub mod import;` to `src/lib.rs` or `src/main.rs` (whichever pattern the project uses for module declarations).

#### 5a. Tests in `src/import.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// try_parse_clipboard_note must never produce output that contains
    /// content it did not receive as input (no injection from config or files).
    #[test]
    fn parse_result_is_subset_of_input() {
        let input = "Assessment: stable\nPlan: continue";
        let result = try_parse_clipboard_note(input, &[]);
        if let Some(text) = result {
            for line in text.lines() {
                assert!(
                    input.contains(line.trim()) || line.trim().is_empty(),
                    "parse output contains content not present in input: {line:?}"
                );
            }
        }
    }

    /// The function must never return a non-empty string for empty input.
    #[test]
    fn empty_input_returns_none() {
        assert_eq!(try_parse_clipboard_note("", &[]), None);
    }

    /// The function must not panic on arbitrary binary-ish content.
    #[test]
    fn does_not_panic_on_garbage_input() {
        let garbage = "\x00\xFF\xFE binary \n junk \t \r\n";
        let _ = try_parse_clipboard_note(garbage, &[]);
    }

}
```

Do not add a “does not write files anywhere” test here unless the implementation is refactored to accept an injectable filesystem target. A temp dir that is never passed into `try_parse_clipboard_note` cannot prove anything useful.

### 6. Add `tempfile` as a dev-dependency

If not already present, add to `Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
```

Check `Cargo.toml` first; `tempfile` may already be used by existing tests.

### 7. Run the full test suite

```
cargo test --all
```

All new tests must pass. Any failure indicates either a test bug or an existing privacy regression that must be fixed before this plan closes.

## Validation

### Automated

- All new tests in `src/app.rs` pass with `cargo test`.
- All new tests in `src/import.rs` pass with `cargo test`.
- No existing tests regress.

### Manual

None required for this plan beyond checking that tests write only inside their temp directories.

## Non-Goals

- Implementing the clipboard-import feature (Phase 2 scope).
- Adding UI for import banners or `ImportYes`/`ImportNo` messages (Phase 2 scope).
- Proving that clipboard parsing or future import flows write nowhere on disk under any circumstance.
- Testing log output (no structured logging exists; add dedicated capture tests here if `tracing` or richer diagnostics are introduced).

## Expected Outcome

After this plan is merged, any future change that causes clipboard-derived patient note text to appear in `config.yml` through the current save paths will fail one of these regression tests immediately rather than silently reaching production.
