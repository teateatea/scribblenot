## Task
#44 - Author-friendly error modal for YAML data load and validation failures

## Context
Data load and validation errors currently surface as a brief flash in the status bar at the bottom of the window. The message is short-lived and space-constrained, which makes YAML authoring errors hard to act on. The user authors YAML data files frequently and wants clear, persistent, actionable error output including the file name, line number, and the exact quoted line where the problem occurred.

Existing error strings already carry fix hints for most structural errors (see `validate_merged_hierarchy`, `hierarchy_to_runtime`, and the `*_fix_hint` helpers in `src/data.rs`). This plan builds on that foundation rather than replacing it.

Key product rules:
- The status bar stays; only data load and validation errors move to the error modal
- The error modal blocks all other interaction until dismissed with a Back keybind (Esc or Backspace)
- The modal fills nearly the full window and is scrollable if content is tall
- On startup load failure the app opens in an error-only state instead of crashing to terminal
- Line numbers and quoted source lines are extracted only when an error actually occurs (not on every successful load)
- Error message wording is authored in `scribblenot/messages/`, one `.yml` file per error category, so the user controls the language

## Approach

Introduce a typed `ErrorKind` enum whose variants map one-to-one to the existing error sites. On error, produce an `ErrorReport` that carries the kind, its parameters (the actual ids, file paths, etc.), and optionally a source location (file, line number, quoted line). The source location is obtained via a lazy two-pass parse: the normal typed-deserialization path runs first; if it fails, a second pass through `serde_yaml::Value` extracts the location of the relevant key before returning the `ErrorReport`.

The `messages/` YAML files map each error kind id to a `title`, `description`, and `fix` field. These are loaded at startup alongside `AppData`. Placeholders like `{id}` or `{file}` in the authored strings are substituted at display time.

The error modal is a new full-width, full-height overlay rendered above everything else. It is not a variant of the existing `Modal` type; it is a separate `App`-level field. When set, the main render function draws only the error modal and suppresses the normal UI.

## Error Kind Categories and Files

### `messages/file_io.yml`
- `data_dir_read_failed` — failed to open the data directory
- `data_dir_enumerate_failed` — failed to list entries in the data directory
- `data_file_read_failed` — failed to read a specific `.yml` file

### `messages/yaml_parse.yml`
- `yaml_parse_failed` — YAML syntax error in a data file document
- `deprecated_repeating_key` — `repeating:` key found; should be `joiner_style:`
- `multiple_templates_in_file` — two `template:` blocks in the same file

### `messages/template.yml`
- `multiple_templates_across_files` — `template:` found in more than one `.yml` file
- `wrong_template_count` — total template count across all files is not exactly 1
- `missing_template` — no template found after loading all files

### `messages/duplicate_ids.yml`
- `duplicate_global_id` — same id registered for two different hierarchy kinds
- `duplicate_boilerplate_id` — two boilerplate entries share an id

### `messages/child_references.yml`
- `missing_child_ref` — `contains:` references an id that does not exist
- `wrong_kind_child_ref` — `contains:` references an id that exists but is the wrong kind
- `invalid_child_kind` — `contains:` includes a kind that is not allowed under this parent

### `messages/field_references.yml`
- `field_unknown_list` — field `lists:` references an id that does not exist
- `field_wrong_kind_list` — field `lists:` references an id that exists as the wrong kind
- `field_unknown_collection` — field `collections:` references an id that does not exist
- `field_wrong_kind_collection` — field `collections:` references an id that exists as the wrong kind
- `field_unknown_format_list` — `format:` placeholder references a list id not attached to the field
- `field_wrong_kind_format_list` — `format:` placeholder references an id of the wrong kind
- `field_cycle` — field `contains:` eventually loops back to itself

### `messages/list_assigns.yml`
- `list_self_assignment` — list item `assigns:` targets its own list
- `list_assigns_unknown_list` — `assigns:` targets a list id that does not exist
- `list_assigns_unknown_item` — `assigns:` targets an item id that does not exist in the target list

## Message YAML Format

Each file is a YAML sequence under a top-level `errors:` key. Placeholders use `{param_name}` syntax and are substituted at display time.

```yaml
errors:
  - id: duplicate_global_id
    title: "Duplicate ID"
    description: "The id '{id}' is used by both a {kind_a} and a {kind_b}. All group, section, collection, field, and list ids must be globally unique."
    fix: "Rename one of the two '{id}' entries so every id appears only once across all hierarchy kinds."
```

Source location fields (`{file}`, `{line}`, `{quoted_line}`) are available in all messages when the two-pass parse succeeds. They can be omitted from a message template if not meaningful for that error kind.

## ErrorReport Structure

```rust
pub struct ErrorReport {
    pub kind_id: &'static str,          // matches the id in messages/*.yml
    pub params: HashMap<String, String>, // substituted into the message template
    pub source: Option<ErrorSource>,
}

pub struct ErrorSource {
    pub file: PathBuf,
    pub line: usize,       // 1-indexed
    pub quoted_line: String,
}
```

## Two-Pass Source Location

The first parse pass uses the existing typed `serde_yaml::Deserializer` path and is unchanged on success. When it returns an error, a second pass deserializes the same content to `serde_yaml::Value`. The `serde_yaml::Value` type carries `Mark` locations; walking the `Value` tree to find the key named in the error extracts its line number. The quoted line is then pulled from the already-in-memory file content string by indexing into its lines.

Only the parse and structural-validation errors benefit from source location. I/O errors (file not found, permission denied) carry the OS message directly and do not need a second pass.

## Critical Files

- `src/data.rs` — introduce `ErrorKind`, `ErrorReport`, `ErrorSource`; update `load_hierarchy_dir`, `validate_merged_hierarchy`, and related helpers to return `ErrorReport`; add two-pass location extraction helper
- `src/app.rs` — add `error_modal: Option<ErrorReport>` to `App`; wire `reload_data` failure to set it; wire startup failure to produce an error-state `App`; add Back-keybind handler to clear it
- `src/ui/mod.rs` — add `error_modal_view(report, messages)` render function; gate the normal UI render behind `app.error_modal.is_none()`
- `src/main.rs` — catch `AppData::load` failure at startup and build `App` in error state
- `messages/*.yml` — the seven authored message files listed above
- new `src/messages.rs` — load and index `messages/*.yml`; provide `fn render_report(report, messages) -> RenderedError` that substitutes params and injects source location fields

## Steps

### Step 1 - Define `ErrorReport`, `ErrorKind`, and `ErrorSource`
In a new `src/error_report.rs` (or inline in `src/data.rs`), define the three structs above. `ErrorKind` can start as a `&'static str` id rather than a full enum; a full enum is a later refactor if needed.

### Step 2 - Author `messages/*.yml`
Create the seven files listed above under `scribblenot/messages/`. Write a complete entry for every error kind id, with `title`, `description`, and `fix`. Use `{param_name}` placeholders consistently. Existing fix-hint strings from the `*_fix_hint` helpers are a good starting draft for the `fix` fields.

### Step 3 - Add `src/messages.rs` loader
Load all `.yml` files in the `messages/` directory at startup. Parse each into a `Vec<MessageEntry>` keyed by `id`. Expose `fn lookup(id: &str) -> Option<&MessageEntry>` and `fn render(report: &ErrorReport) -> RenderedError` that substitutes params into `description` and `fix`, appending source location when present.

If `messages/` is missing or a file fails to parse, log a warning and fall back to the raw error string from the `ErrorReport`. This ensures the app never panics over its own message files.

### Step 4 - Update data-load error returns
Update `load_hierarchy_dir`, `validate_merged_hierarchy`, `hierarchy_to_runtime`, and `parse_hierarchy_file_documents` to return `ErrorReport` instead of bare `String`. Populate `kind_id` and `params` at each error site to match the message ids from Step 2. The existing fix-hint helper functions (`missing_child_fix_hint`, etc.) can be retired once the YAML messages cover the same ground, or kept as internal fallback text.

### Step 5 - Add two-pass source location extraction
Add a helper `fn locate_in_yaml(content: &str, key: &str) -> Option<(usize, String)>` that:
1. Deserializes `content` to `serde_yaml::Value`
2. Walks the value tree looking for a mapping key matching `key`
3. Returns the 1-indexed line number and the trimmed source line from `content`

Call this helper from the error-producing sites in Step 4 that benefit from location info (parse errors and structural validation errors). Do not call it from I/O error sites.

### Step 6 - Add `error_modal` field to `App`
Add `error_modal: Option<ErrorReport>` to `App`. In `reload_data`, replace the current `StatusMsg::error(format!("Data refresh failed: {err}"))` with `self.error_modal = Some(report)` when the load returns an `ErrorReport`. Add a handler for the Back keybind that clears `self.error_modal` when it is set, before any other key dispatch.

### Step 7 - Render the error modal
In `src/ui/mod.rs`, add `fn error_modal_view(report: &ErrorReport, messages: &Messages) -> Element<Message>`. The layout:
- Full window width and height, rendered above the normal UI
- Header row: `title` from the message entry (or the kind id as fallback)
- Body: `description` with params substituted
- If source location is present: `file:line` on one line, then the `quoted_line` in a monospace block
- Fix section: `fix` with params substituted
- Footer: "Press Esc or Backspace to dismiss"

In `view()`, check `app.error_modal` first. If set, render only the error modal overlay. If not set, render the normal UI as today.

### Step 8 - Wire startup load failure to error-only state
In `src/main.rs`, when `AppData::load` fails at startup, build an `App` with `error_modal: Some(report)` and all other fields in their zero/empty state. The app window opens, shows only the error modal, and the user can dismiss it (which would leave a blank app — that is acceptable; they can then fix the YAML and relaunch, or trigger a live reload if they fix it in time).

### Step 9 - Tests and verification
- Unit test: `locate_in_yaml` returns the correct line number and quoted text for a known YAML string with a duplicate key
- Unit test: `render` substitutes all params and includes source location fields when present
- Unit test: `reload_data` sets `error_modal` on a deliberately broken YAML fixture
- Manual test: break a real data file with a duplicate id, reload, confirm the modal appears with the correct title, quoted line, and fix text; dismiss with Esc; fix the file, reload, confirm the modal clears

### Step 10 - Final verification
```powershell
cargo test
cargo run -- --validate-data
$env:SCRIBBLENOT_HEADLESS="1"; cargo run
cargo run  # visual check: break a yml, reload, inspect modal
```
