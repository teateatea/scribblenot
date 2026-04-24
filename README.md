# scribblenot

**v0.5.5** - A structured clinical note authoring tool built with Rust and Iced.

scribblenot is a keyboard-driven desktop app for building formatted clinical notes from YAML-defined structure and picker data. You move through a map of note sections, confirm values in the wizard pane, and watch the note update in the preview pane. Press `c` to copy the exported note to the clipboard.

## Quick Start

```bash
cargo build --release
./target/release/scribblenot
```

What loads at startup:

1. All `.yml` files in `data/`, except `keybindings.yml`, are merged into one runtime hierarchy.
2. `keybindings.yml` is loaded separately as keybinding config.
3. The hierarchy must contain exactly one `template:` block across all data files.

If you edit a YAML file, restart the app to reload it.

Fast validation for authored data:

```bash
cargo validate
cargo run -- --validate-data
cargo run -- --validate
```

That checks the merged hierarchy files in `data/` plus `keybindings.yml` and exits without opening the UI.
`cargo run --validate` will not work because Cargo interprets that flag itself before passing args to the app.
Successful validation prints a bold green `Validation OK:` heading followed by the rest of the line in green. Validation failures print a bold yellow `scribblenot:` prefix, keep the problem text in yellow, and print any bold green `Fix:` guidance in green. Cargo still reports the nonzero exit when you invoke it through `cargo run` or `cargo validate`.


## First Run Mental Model

The UI has three panes:

- **Map**: overall note outline.
- **Wizard**: the current section you are filling in.
- **Preview**: the editable document the app keeps in sync.

Two terms matter immediately:

- A **section** is one stop in the note flow.
- A **field** is one input step inside a multi-field section.

Typical flow:

1. Move to a section.
2. Confirm values in the wizard.
3. Watch the preview update.
4. Press `c` to copy the exported note.

## Data Files Overview

| File | Purpose | Written By App? |
|------|---------|-----------------|
| `config.yml` | App preferences and sticky values | Yes, when settings or sticky values change |
| `keybindings.yml` | Keybinding overrides | No |
| `default-theme.yml` | Theme colors, fonts, spacing, flash durations | No |
| `sections.yml` | Template block, groups, intake data, shared lists | No |
| `subjective.yml` | Subjective section data | No |
| `objective.yml` | Objective section data | No |
| `treatment.yml` | Treatment section data | No |

Notes:

- All hierarchy files in `data/` are merged together.
- `keybindings.yml` is not part of that hierarchy merge.
- `config.yml` is not simply “saved on exit”; it is saved when pane layout changes and when sticky values are updated by field confirmation.

## `config.yml` Reference

Example:

```yaml
pane_layout: default
theme: default-theme
sticky_values:
  year: "2026"
  month: "04"
  day: "09"
hint_labels_capitalized: true
hint_labels_case_sensitive: false
```

| Key | Meaning |
|-----|---------|
| `pane_layout` | `default` or `swapped` |
| `theme` | Theme file name without `.yml` |
| `sticky_values` | Map of `list_id: output_value` |
| `hint_labels_capitalized` | Uppercase or lowercase hint labels |
| `hint_labels_case_sensitive` | Case-sensitive or case-insensitive hint matching |

Notes:

- `pane_layout` can be toggled at runtime with the `swap_panes` binding.
- `sticky_values` are used both for sticky lists and for format placeholders that reference lists not prompted in the current field.

## `keybindings.yml` Reference

Important distinction:

- The checked-in `data/keybindings.yml` is one authored configuration.
- If `keybindings.yml` is missing or invalid, the app uses built-in fallback defaults, which are different in a few places.

Checked-in example:

```yaml
nav_down: [down, n]
nav_up: [up, e]
select: [space]
confirm: [enter]
add_entry: [d]
back: [esc, backspace]
swap_panes: ['`']
help: ['?']
quit: [ctrl+q]
nav_left: [left, h]
nav_right: [right, i]
hints: [a, r, s, t, q, w, f, p, 1, 2, 3, 4]
super_confirm: [shift+enter]
copy_note: [c]
```

Built-in fallback defaults:

```yaml
nav_down: [down, n]
nav_up: [up, e]
select: [space, s]
confirm: [enter, t]
add_entry: [a, d]
back: [esc]
swap_panes: ['`']
help: ['?']
quit: [ctrl+q]
nav_left: [left, h]
nav_right: [right, i]
hints: [1, 2, 3, 4, 5, 6, 7, 8, 9]
super_confirm: [shift+enter]
copy_note: [c]
```

| Action | What It Does |
|--------|--------------|
| `nav_down` / `nav_up` | Move cursor or selection |
| `select` | Toggle current item in multi-select or collection flows |
| `confirm` | Confirm current value or advance |
| `add_entry` | Start a new free-text entry; it is not used for reusable list authoring, and in list-select sections it currently shows an error because custom list entry creation was removed |
| `back` | Go back, dismiss a modal, or leave a sub-context |
| `swap_panes` | Toggle pane layout |
| `help` | Show help overlay |
| `quit` | Quit app |
| `nav_left` / `nav_right` | Move left or right in the current context: pane movement, modal browsing between field parts, or wizard modal entry |
| `super_confirm` | Confirm using defaults or sticky fallbacks where possible |
| `copy_note` | Copy exported note |
| `hints` | Hint key pool used to generate quick-select labels |

Key string syntax:

Modal search-bar exception:

- In modal search bars, raw arrow keys still perform navigation.
- Character aliases such as `n`, `e`, `h`, and `i` stay as typed text there.

Modal reopen behavior:

- Reopening a confirmed list-based field restores the modal cursors from the last confirmed structured choices when that state is available.
- In reopened list modals, the originally confirmed row stays highlighted while the live cursor can move away from it.
- `nav_left` and `nav_right` browse between modal parts without committing intermediate rows. At the terminal confirm edge, `nav_right` commits each part from its explicitly confirmed choice when available, otherwise it falls back to that modal's current cursor position.
- Confirmed list items gain the thin confirmation border as soon as each list part is confirmed, including during first-pass modal entry and when revisiting a completed field.

- Single characters: `a`, `1`, `/`
- Special keys: `up`, `down`, `left`, `right`, `enter`, `esc`, `space`, `backspace`, `tab`
- Modifier combos: `ctrl+q`, `shift+enter`

## Theme Reference (`default-theme.yml`)

Theme files define colors, fonts, widths, spacing, and flash durations.

Custom colors:

```yaml
custom_colors:
  my-dark: 0F1217
  my-accent: 4FB3D9
```

Use them elsewhere:

```yaml
pane_active_border: "#my-accent"
scroll_rail: "#my-dark"
```

Main theme keys:

- Text and highlight: `text`, `active`, `active_preview`, `selected`, `selected_dark`, `hint`, `hint_prefix`, `modal`, `muted`, `error`, `displaced`
- Pane layout: `background`, `pane_background`, `pane_active_background`, `pane_inactive_background`, `pane_active_border`, `pane_inactive_border`, `pane_border_width`, `pane_buffer_width`
- Modal: `modal_background`, `modal_panel_background`, `modal_active_background`, `modal_inactive_background`, `modal_stub_background`, `modal_item_background`, `modal_item_hovered_background`, `modal_text`, `modal_selected_text`, `modal_muted_text`, `modal_hint_text`, `modal_input_background`, `modal_input_text`, `modal_input_placeholder`, `modal_input_border`, `modal_spacer_width`, `modal_stub_width`, `modal_stream_transition_duration_ms`, `modal_stream_transition_easing`
- Preview and status: `sticky_default_preview`, `confirmed_muted_preview`, `preview_copy_flash_background`, `preview_copy_flash_duration_ms`, `text_color_flash`, `text_color_flash_duration`, `status_background`
- Scroll: `scroll_rail`, `scroll_scroller`, `scroll_rail_hovered`, `scroll_scroller_hovered`, `scroll_rail_dragged`, `scroll_scroller_dragged`, `scroll_gap`, `scroll_border_width`, `scroll_width`, `scroll_spacing`
- Fonts: `font_pane`, `font_heading`, `font_preview`, `font_modal`, `font_status`

Fonts must exist locally. If the requested font cannot be parsed, the app falls back.

## Data Authoring Guide

### Hierarchy At A Glance

```text
template
  -> groups
    -> sections or collections
      -> sections may contain fields and/or lists
      -> collections contain lists
      -> fields may contain fields, lists, and/or collections
      -> lists contain items
```

### What Can Contain What

| Parent | Allowed Children |
|--------|------------------|
| `template` | `group` refs only |
| `group` | `section` refs and `collection` refs |
| `section` | `field` refs and `list` refs |
| `collection` | `list` refs only |
| `field.contains` | `field`, `list`, and `collection` refs |

Quick reading guide:

- Use `section -> field` when you want a guided wizard flow.
- Use `section -> list` when you want a flat list-select section.
- Use `field -> field` when you want a composite field built from smaller child fields.
- Use `field -> collection` when you want switchable bundles of list options.

Child ref syntax:

```yaml
contains:
  - { group: some_group_id }
  - { section: some_section_id }
  - { collection: some_collection_id }
  - { field: some_field_id }
  - { list: some_list_id }
```

Global rules:

- Exactly one `template` block across all hierarchy data files.
- Group, section, collection, field, and list IDs share one global namespace.
- Unknown keys on authored hierarchy blocks are rejected.
- `repeating` is rejected; use `joiner_style`.
- `format_lists` is not part of the authored schema; extra `format` placeholders are discovered automatically.

### Authored `hotkey`

You can add an optional single-character `hotkey:` to:

- `section`
- `field`
- object-form list `item`

Example:

```yaml
sections:
  - id: subjective_section
    label: Subjective
    hotkey: "s"

fields:
  - id: requested_regions
    label: Requested Regions
    hotkey: "r"

lists:
  - id: region
    items:
      - id: glutes
        label: Glutes
        output: Glutes
        hotkey: "g"
```

Rules:

- `hotkey` must be exactly one character.
- Shorthand string items like `- Shoulder` cannot carry `hotkey`; use object form instead.
- Duplicate authored hotkeys in the same live hint scope are expanded into multi-character labels automatically.
- In modal search bars and other text-entry contexts, typed characters still go to text entry instead of firing item hotkeys.
- Collection-name rows still use generated hints in this version; only item-backed rows inside collection modals can use authored `hotkey`.

### Section Type Inference

The runtime decides section type from resolved contents:

| Resolved Contents | Runtime Type |
|-------------------|--------------|
| One or more fields | `multi_field` |
| No fields, one or more lists | `list_select` |
| No fields, no lists | `free_text` |

If a section contains both field refs and list refs, the runtime still becomes `multi_field` because fields take priority.

### `template` block

```yaml
template:
  id: patient_standard_template
  contains:
    - { group: intake_group }
    - { group: subjective_group }
```

| Key | Required | Meaning |
|-----|----------|---------|
| `id` | no | Template ID; defaults to `default_template` |
| `contains` | yes | Ordered group refs |

### `group` block

```yaml
groups:
  - id: subjective_group
    nav_label: SUBJECTIVE
    note_label: "## SUBJECTIVE"
    boilerplate_refs:
      - intake_header
    contains:
      - { section: subjective_section }
```

| Key | Required | Meaning |
|-----|----------|---------|
| `id` | yes | Unique group ID |
| `contains` | yes | Ordered `section` and `collection` refs |
| `nav_label` | no | Map label, used as authored |
| `note_label` | no | Group heading in the note |
| `boilerplate_refs` | no | Boilerplate IDs inserted after group heading |

If `note_label` is omitted, the runtime falls back to `nav_label`, then `id`. In practice, groups always render some heading text.

### `section` block

```yaml
sections:
  - id: appointment_section
    label: Appointment
    nav_label: APPOINTMENT
    show_field_labels: false
    contains:
      - { field: date_field }
    note:
      note_label: "#### APPOINTMENT"
```

| Key | Required | Meaning |
|-----|----------|---------|
| `id` | yes | Unique section ID |
| `label` | no | Wizard title |
| `nav_label` | no | Map label |
| `show_field_labels` | no | Defaults to `true`; set `false` to render multi-field values without `{label}: ` prefixes |
| `contains` | no | Child refs |
| `note.note_label` | no | Section heading in the note |

If `note.note_label` is omitted, it defaults to `#### {LABEL_UPPERCASED}`. Set `note_label: ""` to suppress the section heading.

### `field` block

Fields are the input steps inside a `multi_field` section. A field can be a simple leaf picker, or it can be a composite field that contains child fields and formats their resolved outputs into one phrase.

```yaml
fields:
  - id: appointment_requested_field
    label: Request
    format: "{year}-{month}-{day}: Pt requested a {appointment_type_list}{the_region}."
    contains:
      - { list: appointment_type_list }
      - { list: the_region }
```

| Key | Required | Meaning |
|-----|----------|---------|
| `id` | yes | Unique field ID |
| `label` | yes | Wizard label |
| `format` | no | Output template |
| `contains` | no | Typed child refs. May include `field`, `list`, and `collection` refs |
| `joiner_style` | no | Join style for repeated completed field outputs |
| `max_entries` | no | Repeat cap before auto-advance |
| `max_actives` | no | Maximum simultaneously active collections |

Legacy field-level `lists:` and `collections:` keys are rejected. Use typed `contains:` refs instead.

#### Nested fields

If a field contains child fields, the child fields are prompted in authored order and the parent field's `format` is applied to the completed child values.

That means you can model:

- a full sentence field
- a repeatable phrase field inside that sentence
- a smaller child field that builds one phrase like `left shoulder`

Example:

```yaml
fields:
  - id: appointment_requested
    label: Request
    format: "Pt requested a {appointment_type}{requested_regions}."
    contains:
      - { list: appointment_type }
      - { field: requested_regions }

  - id: requested_regions
    label: Requested Regions
    format: "{requested_region}"
    joiner_style: comma_and_the
    max_entries: 6
    contains:
      - { field: requested_region }

  - id: requested_region
    label: Requested Region
    format: "{side}{body_part}"
    contains:
      - { field: side }
      - { list: body_part }

  - id: side
    label: Side
    contains:
      - { list: side_list }
```

In that pattern:

- `requested_region` builds one phrase
- `requested_regions` repeats that completed phrase and joins it
- `appointment_requested` wraps the final joined phrase in the outer sentence

#### Field Placeholders

Both `label` and `format` can contain placeholders like `{year}`, `{the_region}`, or `{requested_regions}`.

Prompted vs resolved behavior:

| Placeholder Source | What Happens |
|--------------------|--------------|
| Listed in `contains` as a child field or child list | The user is prompted for it in authored order |
| Listed in `lists` on a simple field | The user is prompted for it in listed order |
| Referenced only in `format` | No picker opens for it; the runtime resolves it from sticky values or list defaults |

That is the easiest way to reuse sticky date values across many fields.

### `list` block

Lists define picker options.

```yaml
lists:
  - id: the_region
    label: "Region"
    default: empty_space
    modal_start: search
    joiner_style: comma_and_the
    max_entries: 3
    items:
      - id: empty_space
        label: "1 (Empty Space)"
        output: ""
      - label: "Head, Neck, Shoulders"
        output: "Head, Neck, Shoulders"
```

| Key | Required | Meaning |
|-----|----------|---------|
| `id` | yes | Unique list ID and placeholder name |
| `items` | yes | List entries |
| `label` | no | Picker title |
| `preview` | no | Short preview label |
| `sticky` | no | Save confirmed output in `config.yml` |
| `default` | no | Default item ID, label, or output value |
| `modal_start` | no | `list` or `search` |
| `joiner_style` | no | Join style for repeated output |
| `max_entries` | no | Repeat cap |

Joiner styles:

| Value | Example Output |
|-------|----------------|
| `comma` | `Neck, Shoulders, Back` |
| `comma_and` | `Neck, Shoulders, and Back` |
| `comma_and_the` | `Neck, Shoulders, and the Back` |
| `comma_or` | `Neck, Shoulders, or Back` |
| `semicolon` | `Neck; Shoulders; Back` |
| `slash` | `Neck/Shoulders/Back` |
| `newline` | one value per line |

Notes:

- If `joiner_style` is set, the list behaves as repeating multi-select.
- If `joiner_style` is omitted but `max_entries` is set, the runtime still treats the list as repeating and uses a default comma-and joiner.
- If both are omitted, the list is single-select.
- Repeating at the list level repeats raw list outputs. Repeating at the field level repeats the completed field output after its `format` has been applied.

That last distinction matters:

- Repeating list: choose several raw values like `Neck`, `Shoulders`, `Back`
- Repeating field: build a completed phrase like `left shoulder`, then repeat that completed phrase

### list items

String shorthand:

```yaml
items:
  - "Adhesion"
  - "Atrophy"
```

Full object:

```yaml
items:
  - id: bilateral
    label: "2 Bilateral"
    output: "BL "
    default_enabled: true
```

| Key | Required | Meaning |
|-----|----------|---------|
| `id` | no | Item ID; auto-generated if omitted |
| `label` | no | UI and search text |
| `output` | no | Exported text |
| `default_enabled` | no | Initial enabled state inside collections |
| `fields` | no | Referenced branch field IDs |

### `collection` block

Collections group one or more lists into a toggleable bundle.

```yaml
collections:
  - id: back_all_prone_collection
    label: "ALL - UPPER MIDDLE & LOW BACK"
    default_enabled: true
    contains:
      - { list: back_all_prone }
```

| Key | Required | Meaning |
|-----|----------|---------|
| `id` | yes | Unique collection ID |
| `contains` | yes | List refs only |
| `label` | no | Wizard label |
| `nav_label` | no | Map label |
| `default_enabled` | no | Starts active |
| `joiner_style` | no | Join style across enabled outputs |
| `note.note_label` | no | Heading used when rendering as a block |

If a field attaches multiple collections and sets `max_actives: 1`, they behave like radio groups.

### `boilerplate` block

```yaml
boilerplate:
  - id: intake_header
    text: |
      **Patient Intake**
      ---
```

| Key | Required | Meaning |
|-----|----------|---------|
| `id` | yes | Boilerplate ID |
| `text` | yes | Literal text inserted after group heading |

## Common Patterns

Sticky date field:

```yaml
fields:
  - id: date_field
    label: "{year}-{month}-{day}"
    format: ""
    contains:
      - { list: day }
      - { list: month }
      - { list: year }
```

Reusing sticky date values later:

```yaml
fields:
  - id: appointment_requested_field
    label: Request
    format: "{year}-{month}-{day}: Pt requested a {appointment_type_list}{the_region}."
    contains:
      - { list: appointment_type_list }
      - { list: the_region }
```

Collection switching:

```yaml
fields:
  - id: prone_back_field
    label: "BACK"
    max_actives: 1
    contains:
      - { collection: back_all_prone_collection }
      - { collection: back_upper_mid_prone_collection }
      - { collection: back_lower_prone_collection }
```

Repeat a completed phrase instead of repeating one raw list:

```yaml
fields:
  - id: requested_regions
    label: Requested Regions
    format: "{requested_region}"
    joiner_style: comma_and_the
    max_entries: 6
    contains:
      - { field: requested_region }

  - id: requested_region
    label: Requested Region
    format: "{side}{body_part}"
    contains:
      - { list: side_list }
      - { list: body_part }
```

## Note Rendering

The preview is an editable document with managed markers:

```html
#### SUBJECTIVE
<!-- scribblenot:section id=subjective_section:start -->
2026-04-09: BL Head, Neck, Shoulders: Pt describes minor pain
<!-- scribblenot:section id=subjective_section:end -->
```

On export:

- Managed markers are stripped.
- Empty managed section bodies are omitted along with their section heading.
- Group headings and boilerplate remain.

Rendered body by section type:

| Section Type | Exported Body |
|--------------|---------------|
| `multi_field` | One rendered line per confirmed field entry, plus fallback-rendered lines when defaults or sticky values resolve a value |
| `free_text` | One line per entry |
| `list_select` | Selected item outputs, one per line |
| `collection` | Active collection outputs |
| `checklist` | Checked item labels, one per line |

Multi-field output rules:

- If a field resolves to a value and has `format`, that rendered string is used.
- If the section sets `show_field_labels: false`, field labels are suppressed for that section.
- If a field has collections only, with no lists and no `format`, field labels are also suppressed.
- Otherwise output is `{label}: {value}`.

## Known Limitations And Gotchas

- `repeating` is rejected. Use `joiner_style`.
- IDs for groups, sections, collections, fields, and lists must be globally unique.
- Group headings always resolve to something because the runtime falls back from `note_label` to `nav_label` to `id`.
- `section_type` is inferred; you do not declare it directly in YAML.
- `max_entries` without `joiner_style` still makes a list repeating.
- `max_entries` with `joiner_style` on a field repeats the completed field output, including nested child-field formats.
- Theme fonts depend on local font availability.
