#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelpTopic {
    pub id: &'static str,
    pub title: &'static str,
    pub aliases: &'static [&'static str],
    pub body: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpCodeBlock {
    pub language: Option<String>,
    pub code: String,
}

pub fn topics() -> &'static [HelpTopic] {
    &TOPICS
}

pub fn filtered_topic_indices(query: &str) -> Vec<usize> {
    let normalized = query.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return (0..TOPICS.len()).collect();
    }

    let mut ranked = TOPICS
        .iter()
        .enumerate()
        .filter_map(|(idx, topic)| topic_match_rank(topic, &normalized).map(|rank| (rank, idx)))
        .collect::<Vec<_>>();
    ranked.sort_by_key(|(rank, idx)| (*rank, *idx));
    ranked.into_iter().map(|(_, idx)| idx).collect()
}

pub fn topic_code_blocks(topic: &HelpTopic) -> Vec<HelpCodeBlock> {
    parse_code_blocks(topic.body)
}

pub fn topic_markdown(topic: &HelpTopic) -> String {
    let mut markdown = format!("# {}\n\n{}", topic.title, topic.body.trim());
    let see_also = topic_see_also_topics(topic);
    if !see_also.is_empty() {
        markdown.push_str("\n\n## See Also\n\n");
        for related in see_also {
            markdown.push_str("- ");
            markdown.push_str(related.title);
            markdown.push('\n');
        }
    }
    markdown
}

pub fn topic_see_also_topics(topic: &HelpTopic) -> Vec<&'static HelpTopic> {
    topic_see_also_ids(topic)
        .iter()
        .filter_map(|id| TOPICS.iter().find(|topic| topic.id == *id))
        .collect()
}

#[cfg(test)]
pub fn topic_preview_sentence(topic: &HelpTopic) -> String {
    topic_preview_sentence_with_limit(topic, usize::MAX)
}

pub fn topic_preview_sentence_with_limit(topic: &HelpTopic, max_chars: usize) -> String {
    let first_line = topic
        .body
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with("```"))
        .unwrap_or_default();
    let sentence_end = first_line
        .char_indices()
        .find_map(|(idx, ch)| (ch == '.').then_some(idx + ch.len_utf8()))
        .unwrap_or(first_line.len());
    truncate_single_line(&first_line[..sentence_end], max_chars)
}

pub fn topic_code_block_scroll_offset(topic: &HelpTopic, target_block_idx: usize) -> f32 {
    const HELP_TOPIC_FIXED_HEADER_HEIGHT: f32 = 120.0;
    const CODE_BLOCK_HEADER_HEIGHT: f32 = 22.0;
    const CODE_BLOCK_HEADER_TO_TEXTBOX_GAP: f32 = 4.0;
    const BODY_ELEMENT_GAP: f32 = 12.0;
    const PARAGRAPH_LINE_HEIGHT: f32 = 22.0;
    const HEADING_2_HEIGHT: f32 = 32.0;
    const HEADING_3_HEIGHT: f32 = 27.0;
    const CODE_LINE_HEIGHT: f32 = 24.0;
    const CODE_TEXTBOX_VERTICAL_PADDING: f32 = 16.0;

    let mut y = 0.0f32;
    let mut block_idx = 0usize;
    let mut paragraph_lines = 0usize;
    let mut has_previous_element = false;
    let mut lines = topic.body.lines();

    fn add_element_gap(y: &mut f32, has_previous_element: &mut bool) {
        if *has_previous_element {
            *y += BODY_ELEMENT_GAP;
        }
        *has_previous_element = true;
    }

    fn flush_paragraph(y: &mut f32, paragraph_lines: &mut usize, has_previous_element: &mut bool) {
        if *paragraph_lines > 0 {
            add_element_gap(y, has_previous_element);
            *y += *paragraph_lines as f32 * PARAGRAPH_LINE_HEIGHT;
            *paragraph_lines = 0;
        }
    }

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            flush_paragraph(&mut y, &mut paragraph_lines, &mut has_previous_element);
            add_element_gap(&mut y, &mut has_previous_element);
            if block_idx == target_block_idx {
                return (y + CODE_BLOCK_HEADER_HEIGHT + CODE_BLOCK_HEADER_TO_TEXTBOX_GAP
                    - HELP_TOPIC_FIXED_HEADER_HEIGHT)
                    .max(0.0);
            }

            let mut code_line_count = 0usize;
            for code_line in lines.by_ref() {
                if code_line.trim() == "```" {
                    break;
                }
                code_line_count += 1;
            }
            y += CODE_BLOCK_HEADER_HEIGHT
                + CODE_BLOCK_HEADER_TO_TEXTBOX_GAP
                + CODE_TEXTBOX_VERTICAL_PADDING
                + (code_line_count.max(1) as f32 * CODE_LINE_HEIGHT);
            block_idx += 1;
            continue;
        }

        if trimmed.is_empty() {
            flush_paragraph(&mut y, &mut paragraph_lines, &mut has_previous_element);
        } else if trimmed.starts_with("## ") {
            flush_paragraph(&mut y, &mut paragraph_lines, &mut has_previous_element);
            add_element_gap(&mut y, &mut has_previous_element);
            y += HEADING_2_HEIGHT;
        } else if trimmed.starts_with("### ") {
            flush_paragraph(&mut y, &mut paragraph_lines, &mut has_previous_element);
            add_element_gap(&mut y, &mut has_previous_element);
            y += HEADING_3_HEIGHT;
        } else {
            paragraph_lines += line.len().saturating_sub(1) / 78 + 1;
        }
    }

    y
}

fn topic_match_rank(topic: &HelpTopic, query: &str) -> Option<usize> {
    if topic.title.to_ascii_lowercase().contains(query) {
        return Some(0);
    }
    if topic
        .aliases
        .iter()
        .any(|alias| alias.to_ascii_lowercase().contains(query))
    {
        return Some(1);
    }
    topic.id.to_ascii_lowercase().contains(query).then_some(2)
}

fn topic_see_also_ids(topic: &HelpTopic) -> &'static [&'static str] {
    match topic.id {
        "yaml-template-class" => &["yaml-group-class"],
        "yaml-group-class" => &[
            "yaml-template-class",
            "yaml-section-class",
            "yaml-collection-class",
            "yaml-boilerplate-class",
        ],
        "yaml-section-class" => &["yaml-group-class", "yaml-field-class", "yaml-list-class"],
        "yaml-collection-class" => &["yaml-group-class", "yaml-field-class", "yaml-list-class"],
        "yaml-field-class" => &[
            "yaml-section-class",
            "yaml-list-item-class",
            "yaml-list-class",
            "yaml-collection-class",
            "format-placeholders",
        ],
        "yaml-list-class" => &[
            "yaml-section-class",
            "yaml-collection-class",
            "yaml-field-class",
            "yaml-list-item-class",
            "item-assigns-target",
            "list-output-affixes",
        ],
        "yaml-list-item-class" => &[
            "yaml-list-class",
            "yaml-field-class",
            "item-driven-branching",
            "item-assigns-target",
        ],
        "yaml-boilerplate-class" => &["yaml-group-class"],
        "item-driven-branching" => &["yaml-list-item-class", "yaml-field-class"],
        "item-assigns-target" => &["yaml-list-item-class", "yaml-list-class"],
        "format-placeholders" => &["yaml-field-class", "list-output-affixes"],
        "list-output-affixes" => &["yaml-list-class"],
        _ => &[],
    }
}

fn parse_code_blocks(body: &str) -> Vec<HelpCodeBlock> {
    let mut blocks = Vec::new();
    let mut lines = body.lines();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        let Some(language) = trimmed.strip_prefix("```") else {
            continue;
        };

        let language = if language.trim().is_empty() {
            None
        } else {
            Some(language.trim().to_string())
        };
        let mut code_lines = Vec::new();

        for code_line in lines.by_ref() {
            if code_line.trim() == "```" {
                blocks.push(HelpCodeBlock {
                    language,
                    code: code_lines.join("\n"),
                });
                break;
            }
            code_lines.push(code_line);
        }
    }

    blocks
}

fn truncate_single_line(value: &str, max_chars: usize) -> String {
    let single_line = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if single_line.chars().count() <= max_chars {
        return single_line;
    }

    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }

    let hard_limit = max_chars - 3;
    let mut boundary = 0usize;
    let mut chars_seen = 0usize;
    for (idx, ch) in single_line.char_indices() {
        if chars_seen >= hard_limit {
            break;
        }
        if ch.is_whitespace() {
            boundary = idx;
        }
        chars_seen += 1;
    }

    if boundary < hard_limit / 2 {
        boundary = single_line
            .char_indices()
            .nth(hard_limit)
            .map(|(idx, _)| idx)
            .unwrap_or(single_line.len());
    }

    let mut truncated = single_line[..boundary].trim_end().to_string();
    truncated.push_str("...");
    truncated
}

static TOPICS: [HelpTopic; 13] = [
    HelpTopic {
        id: "item-driven-branching",
        title: "Item Branching",
        aliases: &[
            "branching fields",
            "branch fields",
            "branching",
            "item-driven branching",
            "item-driven branching fields",
            "item fields",
            "conditional fields",
            "how do I show fields from an item",
        ],
        body: r#"Use item branching when one list choice should open extra follow-up choices.

The overview is: the item itself becomes a small inline field. Use `format:` for the branch output template, and use typed `contains:` refs for the follow-up field/list/collection choices. Do not use item `fields:`; that old shape is no longer valid.

```yaml
fields:
  - id: exercise_recent
    label: Exercise
    format: "Pt describes {exercise_level} in the past 2 weeks{exercise_due_to}."
    contains:
      - list: exercise_level
      - list: exercise_due_to

lists:
  - id: exercise_level
    items:
      - "regular exercise"
      - "limited exercise"
      - "no exercise"

  - id: exercise_due_to
    items:
      - id: no_due_to
        label: "[no reason]"
        output: ""
      - id: due_to
        label: "DUE TO"
        format: " due to {exercise_reason}"
        contains:
          - list: exercise_reason

  - id: exercise_reason
    items:
      - "pain"
      - "fatigue"
      - "time constraints"
      - "recent illness"
```

## How To

### Step 1

When starting a split from the item level, make the selected item a branch item. A branch item uses `format:` and `contains:`.

```yaml
lists:
  - id: exercise_due_to
    items:
      - id: no_due_to
        label: "[no reason]"
        output: ""
      - id: due_to
        label: "DUE TO"
        format: " due to {exercise_reason}"
        contains:
          - list: exercise_reason
```

Use this small fragment when you are adding one branching item to an existing list.

```yaml
- id: example_item
  label: "label_text"
  format: "{followup_list}"
  contains:
    - list: followup_list
```

### Step 2

Add the follow-up answer list. This is where you fill out the branch options.

```yaml
lists:
  - id: exercise_reason
    items:
      - "pain"
      - "fatigue"
      - "time constraints"
      - "recent illness"
```

### Step 3

Connect the branching list to the main field. This field asks about recent exercise and includes both the main answer list and the small branching list.

```yaml
fields:
  - id: exercise_recent
    label: Exercise
    format: "Pt describes {exercise_level} in the past 2 weeks{exercise_due_to}."
    contains:
      - list: exercise_level
      - list: exercise_due_to
```

### Step 4

Write the main answer list. These are the normal choices the field can output.

```yaml
lists:
  - id: exercise_level
    items:
      - "regular exercise"
      - "limited exercise"
      - "no exercise"
```

### Step 5

Choose one item shape. A leaf item uses `output:`. A branch item uses `format:` plus `contains:`. Do not mix `output:` with `format:` or `contains:`.

```yaml
# Leaf item
- id: no_due_to
  label: "[no reason]"
  output: ""

# Branch item
- id: due_to
  label: "DUE TO"
  format: " due to {exercise_reason}"
  contains:
    - list: exercise_reason
```

## Example Outputs

```text
Pt describes regular exercise in the past 2 weeks.
```

```text
Pt describes limited exercise in the past 2 weeks due to pain.
```"#,
    },
    HelpTopic {
        id: "yaml-field-class",
        title: "Field Class",
        aliases: &[
            "field",
            "fields",
            "field class",
            "field properties",
            "field keys",
            "yaml field",
            "id",
            "label",
            "nav_label",
            "hotkey",
            "format",
            "preview",
            "contains",
            "joiner_style",
            "max_entries",
            "max_actives",
        ],
        body: r########"Use `fields:` for reusable prompts inside multi-field sections and as `field:` children inside item branches. Fields can contain lists, nested fields, or collections.

---

## Property Keys

### id (required)
Stable identity for the field. Must be globally unique.

### label (required)
Prompt name shown to the user.

### nav_label (optional)
Provides a shorter navigation label.

### hotkey (optional)
Direct navigation key. Must be exactly one visible character.

### format (optional)
Combines selected child output with placeholders such as `{list_id}` or `{field_id}`.

### preview (optional)
Short placeholder text shown in the multi-field UI.

### contains (optional)
Lists child inputs. Allowed child refs: `field`, `list`, `collection`.

### joiner_style (optional)
Controls how repeated field output is joined. Options: `comma_and`, `comma_and_the`, `comma_or`, `comma`, `space`, `semicolon`, `slash`, `newline`.

### max_entries (optional)
Caps repeated entries before auto-advance.

### max_actives (optional)
Caps the number of active nested values.

---

## Required-Only Field

```yaml
fields:
  - id: field_id
    label: Field Label
```

## Optional Field

```yaml
fields:
  - id: field_id
    label: Field Label
    nav_label: Field
    hotkey: f
    format: "{list_id}"
    preview: "[value]"
    contains:
      - field: child_field_id
      - list: list_id
      - collection: collection_id
    joiner_style: [comma_and|comma_and_the|comma_or|comma|space|semicolon|slash|newline]
    max_entries: 3
    max_actives: 1
```"########,
    },
    HelpTopic {
        id: "yaml-list-class",
        title: "List Class",
        aliases: &[
            "list",
            "lists",
            "list class",
            "list properties",
            "list keys",
            "yaml list",
            "id",
            "label",
            "preview",
            "output_prefix",
            "output_suffix",
            "sticky",
            "default",
            "modal_start",
            "joiner_style",
            "max_entries",
            "items",
        ],
        body: r########"Use `lists:` for selectable options. Lists are the option source for fields, list-select sections, and collections.

---

## Property Keys

### id (required)
Stable identity for the list. Must be globally unique.

### label (optional)
Human-facing list name.

### preview (optional)
Short placeholder text shown in multi-field UI.

### output_prefix (optional)
Added only when the selected list output is not empty.

### output_suffix (optional)
Added only when the selected list output is not empty.

### sticky (optional)
Defaults to `false`. Sticky lists remember their chosen output across notes.

### default (optional)
Names the item id selected by default.

### modal_start (optional)
Defaults to `list`. Options: `list`, `search`.

### joiner_style (optional)
Controls how repeated list output is joined. Options: `comma_and`, `comma_and_the`, `comma_or`, `comma`, `space`, `semicolon`, `slash`, `newline`.

### max_entries (optional)
Caps repeated selected values.

### items (optional)
Lists selectable items. A useful list should contain items.

---

## Required-Only List

```yaml
lists:
  - id: list_id
```

## Optional List

```yaml
lists:
  - id: list_id
    label: List Label
    preview: "[choice]"
    output_prefix: " "
    output_suffix: "."
    sticky: [true|false]
    default: item_id
    modal_start: [list|search]
    joiner_style: [comma_and|comma_and_the|comma_or|comma|space|semicolon|slash|newline]
    max_entries: 3
    items:
      - id: item_id
        label: Item Label
        output: item output
```"########,
    },
    HelpTopic {
        id: "yaml-list-item-class",
        title: "Item Class (Lists)",
        aliases: &[
            "item",
            "items",
            "list item",
            "list items",
            "item class",
            "item properties",
            "item keys",
            "yaml item",
            "plain string item",
            "simple string item",
            "string shorthand",
            "id",
            "label",
            "default_enabled",
            "output",
            "hotkey",
            "format",
            "contains",
            "assigns",
            "assign",
            "list",
        ],
        body: r########"Use list items under `lists[].items`. An item may be a plain string entry or a full object.

A plain string entry is the shortest way to write an item:

```yaml
items:
  - "Simple item"
```

That form has no explicit keys. Scribblenot uses the string as both the label and output, and auto-generates the item id.

---

## Property Keys

For a full object item, use at least one content key: `id`, `label`, `output`, or `format`. Branch behavior is authored with `format:` plus `contains:`. `assigns:` adds a side effect to an item, but it should not be the only thing that identifies what the item is.

### id (optional)
Stable item identifier. It is used by list `default:` values, `assigns:` targets, saved selections, branch/assignment restoration, hotkey lookup, and validation messages. If omitted, it is generated from `label` or `output`.

You usually only need to write it yourself when another place must point at this item, such as `default: item_id` or `assigns: [{ item: item_id }]`, or when you want a stable name that will not change if the label/output text changes.

### label (optional)
UI/search text. Falls back to output.

### output (optional)
Note/export text for a leaf item. Falls back to label.

### format (optional)
Branch output template. Use this with `contains:` when selecting the item should open follow-up choices.

### contains (optional)
Typed branch children. Allowed child refs: `field`, `list`, `collection`.

### default_enabled (optional)
Defaults to `true`. Controls whether collection items start enabled.

### hotkey (optional)
Must be exactly one visible character.

### assigns (optional)
When this item is selected, also selects an item in another list. Each assign entry has `list` and `item`, both required. The `item` value must be the target item's id.

---

## Required-Only List Item

```yaml
lists:
  - id: list_id
    items:
      - "Item label"
```

This is the plain string form. It is valid because the string supplies the item's content.

## Leaf List Item

```yaml
lists:
  - id: list_id
    items:
      - id: item_id
        label: Item Label
        output: item output
        default_enabled: [true|false]
        hotkey: i
        assigns:
          - list: target_list_id
            item: target_item_id
```

## Branch List Item

```yaml
lists:
  - id: list_id
    items:
      - id: item_id
        label: Item Label
        format: "{child_list_id}"
        contains:
          - list: child_list_id
        default_enabled: [true|false]
        hotkey: i
        assigns:
          - list: target_list_id
            item: target_item_id
```"########,
    },
    HelpTopic {
        id: "item-assigns-target",
        title: "Item Assigns Target Item to Target List",
        aliases: &[
            "assigns",
            "item assigns",
            "assign target",
            "target list",
            "target item",
            "assign another list",
            "select another list item",
        ],
        body: r########"Use `assigns:` when choosing one item should also choose an item in another list.

Plain English version: the source item says, "when I am selected, set this other list to this other item."

```yaml
lists:
  - id: side
    items:
      - id: left
        label: Left
        output: left
        assigns:
          - list: side_word
            item: left_word

  - id: side_word
    items:
      - id: left_word
        label: Left word
        output: left-sided
      - id: right_word
        label: Right word
        output: right-sided
```

In that example:

- `side.left` is the source item.
- `side_word` is the target list.
- `left_word` is the target item id inside that target list.

`assigns:` is for cross-list side effects. It cannot assign back into the same list, and the target `item:` value must be an existing item id in the target list.

## Shape

```yaml
items:
  - id: source_item_id
    label: Source Item
    output: source output
    assigns:
      - list: target_list_id
        item: target_item_id
```

Each assign entry needs both keys:

- `list`: the target list id.
- `item`: the target item id inside that target list.

Write explicit item ids when you plan to target them from `assigns:`. Auto-generated ids can work, but explicit ids are safer because changing the label or output will not accidentally change the reference."########,
    },
    HelpTopic {
        id: "format-placeholders",
        title: "Format Placeholders (Fields)",
        aliases: &[
            "format",
            "placeholders",
            "field format",
            "how do I combine lists",
            "curly braces",
        ],
        body: r#"Use `format:` when a field should combine one or more selected list values into a sentence fragment.

Each `{placeholder}` should match a list id that the field can reach through `contains:`.

Lists are presented to the user in the order they appear under `contains:`. The `format:` string only controls where the confirmed values land in the final output.

```yaml
fields:
  - id: treatment_area_summary
    label: Treatment area summary
    format: "{region} treated with {technique}"
    contains:
      - list: region
      - list: technique
```

In that example, Scribblenot asks for `region` first, then `technique`, even though the format could place those placeholders in any order.

Keep literal text outside the braces. Put only the list id inside each placeholder.

## Optional Values

A list item can intentionally output an empty string. When that empty item is selected, Scribblenot replaces the placeholder with nothing. It does not remove surrounding words, spaces, or punctuation from `format:`.

```yaml
fields:
  - id: exercise_brief
    label: Exercise
    format: "{exercise_level}{exercise_detail}."
    contains:
      - list: exercise_level
      - list: exercise_detail

lists:
  - id: exercise_level
    items:
      - "walking"
      - "regular exercise"

  - id: exercise_detail
    output_prefix: " "
    items:
      - id: no_detail
        label: "[no detail]"
        output: ""
      - "3x/week"
```

## Example Outputs

```text
walking.
```

```text
walking 3x/week.
```

If the format were `"{exercise_level} with {exercise_detail}."`, choosing the empty detail would render `walking with .`. Put optional spacing or punctuation in the optional list output, or use `output_prefix:` and `output_suffix:` on that list."#,
    },
    HelpTopic {
        id: "list-output-affixes",
        title: "Output Prefixes & Suffixes (Lists)",
        aliases: &[
            "output_prefix",
            "output_suffix",
            "prefix",
            "suffix",
            "conditional punctuation",
            "optional punctuation",
            "leading space",
            "parentheses",
        ],
        body: r#"Use `output_prefix:` and `output_suffix:` when a list should bring punctuation or spacing only when that list contributes real output.

These properties belong on a `list`, not on each item. Scribblenot applies them after the selected item output is resolved and after repeated values are joined. If the final list output is empty, both affixes are skipped.

```yaml
fields:
  - id: exercise_brief
    label: Exercise Brief
    format: "{exercise_level}{exercise_details}."
    contains:
      - list: exercise_level
      - list: exercise_details

lists:
  - id: exercise_level
    items:
      - "regular exercise"
      - "limited exercise"

  - id: exercise_details
    output_prefix: " "
    max_entries: 3
    joiner_style: comma
    items:
      - id: empty
        label: "[none]"
        output: ""
      - "3x/week"
      - "walking"
```

## Example Outputs

```text
regular exercise.
```

```text
regular exercise 3x/week, walking.
```

For comma-style optional detail, use a comma and a space:

```yaml
lists:
  - id: symptom_details
    output_prefix: ", "
    items:
      - id: empty
        label: "[none]"
        output: ""
      - "worse with rotation"
```

For parenthetical detail, use both properties:

```yaml
lists:
  - id: exercise_context
    output_prefix: " ("
    output_suffix: ")"
    items:
      - id: empty
        label: "[none]"
        output: ""
      - "3x/week"
```

This renders `Walking.` when empty, and `Walking (3x/week).` when the list has output."#,
    },
    HelpTopic {
        id: "yaml-section-class",
        title: "Section Class",
        aliases: &[
            "section",
            "sections",
            "section class",
            "section properties",
            "section keys",
            "yaml section",
            "free text section",
            "multi field section",
            "list select section",
            "id",
            "label",
            "nav_label",
            "hotkey",
            "show_field_labels",
            "contains",
            "note",
            "note_label",
        ],
        body: r########"Use `sections:` for note areas that the user visits directly. The section kind is inferred from `contains`: field refs make a multi-field section, list refs make a list-select section, and no children makes a free-text section.

---

## Property Keys

### id (required)
Stable identity for the section. Must be globally unique.

### label (optional)
Human-facing section name.

### nav_label (optional)
Controls the map/sidebar label.

### hotkey (optional)
Direct navigation key. Must be exactly one visible character.

### show_field_labels (optional)
Defaults to `true`. Use `false` to render multi-field values without `Label:`.

### contains (optional)
Defines the section kind. Allowed child refs: `field`, `list`.

### note (optional)
Supports `note_label`, the rendered managed subsection heading. Use an empty string when the section should have no heading.

---

## Required-Only Section

```yaml
sections:
  - id: section_id
```

## Optional Section

```yaml
sections:
  - id: section_id
    label: Section Label
    nav_label: SECTION
    hotkey: s
    show_field_labels: true
    contains:
      - field: field_id
      - list: list_id
    note:
      note_label: "#### SECTION HEADING"
```"########,
    },
    HelpTopic {
        id: "yaml-collection-class",
        title: "Collection Class",
        aliases: &[
            "collection",
            "collections",
            "collection class",
            "collection properties",
            "collection keys",
            "yaml collection",
            "id",
            "label",
            "nav_label",
            "note_label",
            "default_enabled",
            "joiner_style",
            "contains",
            "note",
        ],
        body: r########"Use `collections:` for grouped list pickers that behave as a first-class selectable node. Collections can appear under groups or inside fields.

---

## Property Keys

### id (required)
Stable identity for the collection. Must be globally unique.

### label (optional)
Human-facing collection name.

### nav_label (optional)
Controls the map/sidebar label when the collection is in a group.

### note_label (optional)
Legacy shorthand for the rendered note heading.

### default_enabled (optional)
Defaults to `true`. Controls whether the collection starts enabled.

### joiner_style (optional)
Controls how repeated collection output is joined. Options: `comma_and`, `comma_and_the`, `comma_or`, `comma`, `space`, `semicolon`, `slash`, `newline`.

### contains (optional)
Lists the collection's lists. Allowed child ref: `list`.

### note (optional)
Supports `note_label`, the rendered managed subsection heading.

---

## Required-Only Collection

```yaml
collections:
  - id: collection_id
```

## Optional Collection

```yaml
collections:
  - id: collection_id
    label: Collection Label
    nav_label: COLLECTION
    note_label: "#### COLLECTION HEADING"
    default_enabled: [true|false]
    joiner_style: [comma_and|comma_and_the|comma_or|comma|space|semicolon|slash|newline]
    contains:
      - list: list_id
    note:
      note_label: "#### COLLECTION HEADING"
```"########,
    },
    HelpTopic {
        id: "yaml-group-class",
        title: "Group Class",
        aliases: &[
            "group",
            "groups",
            "group class",
            "group properties",
            "group keys",
            "yaml group",
            "id",
            "nav_label",
            "note_label",
            "contains",
        ],
        body: r########"Use `groups:` for the major workflow areas shown in the map and rendered as top-level note headings.

---

## Property Keys

### id (required)
Stable identity for the group. Must be globally unique.

### nav_label (optional)
Controls the map/sidebar label. If omitted, the id is used.

### note_label (optional)
Renders a top-level note heading for the group, such as `## SUBJECTIVE`.

### contains (optional)
Lists the group's children. Allowed child refs: `section`, `collection`, `boilerplate`.

---

## Required-Only Group

```yaml
groups:
  - id: group_id
```

## Optional Group

```yaml
groups:
  - id: group_id
    nav_label: GROUP
    note_label: "## GROUP HEADING"
    contains:
      - section: section_id
      - collection: collection_id
      - boilerplate: boilerplate_id
```"########,
    },
    HelpTopic {
        id: "copy-help-code-blocks",
        title: "Copying Help Code Blocks",
        aliases: &[
            "copy",
            "code block",
            "clipboard",
            "qwf",
            "q w f p",
            "hint keys",
        ],
        body: r#"In a help topic, the section copy key copies the selected code block. The full-note copy key copies the whole topic as markdown.

Use left and right navigation to cycle code blocks. You can also press the hint shown beside a code block, using the same hint-key pool as the rest of Scribblenot.

```yaml
keybindings:
  copy_note: [shift+c]
  copy_section: [c]
  hints: [q, w, f, p, a, r, s, t]
```

The help system reads the current `copy_note` and `hints` keybindings, so changing those bindings changes the help modal too."#,
    },
    HelpTopic {
        id: "yaml-boilerplate-class",
        title: "Boilerplate Class",
        aliases: &[
            "boilerplate",
            "boilerplates",
            "boilerplate class",
            "boilerplate properties",
            "boilerplate keys",
            "yaml boilerplate",
            "id",
            "text",
        ],
        body: r########"Use `boilerplates:` for static authored text that can be inserted through a group's `contains` list.

---

## Property Keys

### id (required)
Stable identity for the boilerplate entry. Must be globally unique.

### text (required)
Exact note text inserted when referenced.

---

## Required-Only Boilerplate

```yaml
boilerplates:
  - id: boilerplate_id
    text: "Boilerplate text."
```

## Optional Boilerplate

```yaml
boilerplates:
  - id: boilerplate_id
    text: |
      First line of boilerplate text.
      Second line of boilerplate text.
```"########,
    },
    HelpTopic {
        id: "yaml-template-class",
        title: "Template Class",
        aliases: &[
            "template",
            "template class",
            "template properties",
            "template keys",
            "yaml template",
            "id",
            "contains",
        ],
        body: r########"Use `template:` once across the loaded data files. It defines the ordered group list for the note workflow.

---

## Property Keys

### id (optional)
Template identity used internally.

### contains (optional)
Lists the groups that appear in the note workflow. Allowed child ref: `group`.

---

## Required-Only Template

```yaml
template: {}
```

## Optional Template

```yaml
template:
  id: template_id
  contains:
    - group: group_id
```"########,
    },
];

#[cfg(test)]
mod tests {
    use super::{
        filtered_topic_indices, topic_code_block_scroll_offset, topic_code_blocks, topic_markdown,
        topic_preview_sentence, topic_preview_sentence_with_limit, topic_see_also_topics, topics,
    };

    #[test]
    fn help_search_matches_aliases() {
        let matches = filtered_topic_indices("curly braces");
        assert_eq!(topics()[matches[0]].id, "format-placeholders");
    }

    #[test]
    fn empty_help_search_uses_curated_topic_order() {
        let ids = filtered_topic_indices("")
            .into_iter()
            .take(4)
            .map(|idx| topics()[idx].id)
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                "item-driven-branching",
                "yaml-field-class",
                "yaml-list-class",
                "yaml-list-item-class"
            ]
        );
        assert_eq!(
            topics().last().map(|topic| topic.id),
            Some("yaml-template-class")
        );
        assert_eq!(
            topics()
                .get(topics().len().saturating_sub(2))
                .map(|topic| topic.id),
            Some("yaml-boilerplate-class")
        );
    }

    #[test]
    fn help_search_finds_list_output_affixes() {
        let matches = filtered_topic_indices("output_prefix");
        assert!(matches
            .iter()
            .any(|idx| topics()[*idx].id == "list-output-affixes"));
    }

    #[test]
    fn help_search_does_not_match_topic_body_text() {
        let matches = filtered_topic_indices("first-class selectable");
        assert!(matches.is_empty());
    }

    #[test]
    fn help_search_prioritizes_title_matches_before_alias_matches() {
        let matches = filtered_topic_indices("field");
        let matched_ids = matches
            .into_iter()
            .map(|idx| topics()[idx].id)
            .collect::<Vec<_>>();

        assert_eq!(matched_ids[0], "yaml-field-class");
        assert_eq!(matched_ids[1], "format-placeholders");
        let section_class_pos = matched_ids
            .iter()
            .position(|id| *id == "yaml-section-class")
            .expect("section class should still match by alias");
        let format_placeholders_pos = matched_ids
            .iter()
            .position(|id| *id == "format-placeholders")
            .expect("format placeholders should match by title");
        assert!(section_class_pos > format_placeholders_pos);
        let branching_pos = matched_ids
            .iter()
            .position(|id| *id == "item-driven-branching")
            .expect("item branching should still match by alias");
        assert!(branching_pos > format_placeholders_pos);
    }

    #[test]
    fn help_topic_preview_uses_first_sentence() {
        let topic = topics()
            .iter()
            .find(|topic| topic.id == "yaml-list-class")
            .expect("topic should exist");

        assert_eq!(
            topic_preview_sentence(topic),
            "Use `lists:` for selectable options."
        );
    }

    #[test]
    fn help_topic_preview_truncates_at_word_boundary() {
        let topic = topics()
            .iter()
            .find(|topic| topic.id == "yaml-group-class")
            .expect("topic should exist");

        assert_eq!(
            topic_preview_sentence_with_limit(topic, 56),
            "Use `groups:` for the major workflow areas shown in..."
        );
    }

    #[test]
    fn help_topic_code_blocks_extract_language_and_code() {
        let topic = topics()
            .iter()
            .find(|topic| topic.id == "item-driven-branching")
            .expect("topic should exist");
        let blocks = topic_code_blocks(topic);
        assert_eq!(blocks.len(), 9);
        assert_eq!(blocks[0].language.as_deref(), Some("yaml"));
        assert!(blocks[0].code.contains("fields:"));
        assert!(blocks[0].code.contains("exercise_recent"));
        assert!(blocks[0]
            .code
            .contains("format: \" due to {exercise_reason}\""));
        assert!(blocks[1].code.contains("exercise_due_to"));
        assert!(blocks[2].code.contains("- id: example_item"));
        assert!(blocks[2].code.contains("contains:"));
        assert!(blocks[2].code.contains("- list: followup_list"));
        assert!(blocks[6].code.contains("# Leaf item"));
        assert!(blocks[6].code.contains("# Branch item"));
    }

    #[test]
    fn help_topic_code_block_scroll_offsets_increase_by_block() {
        let topic = topics()
            .iter()
            .find(|topic| topic.id == "item-driven-branching")
            .expect("topic should exist");

        assert!(
            topic_code_block_scroll_offset(topic, 4) > topic_code_block_scroll_offset(topic, 1)
        );
    }

    #[test]
    fn branching_fields_see_also_links_to_field_class() {
        let topic = topics()
            .iter()
            .find(|topic| topic.id == "item-driven-branching")
            .expect("topic should exist");
        let related = topic_see_also_topics(topic);

        assert_eq!(related.len(), 2);
        assert_eq!(related[0].id, "yaml-list-item-class");
        assert_eq!(related[0].title, "Item Class (Lists)");
        assert_eq!(related[1].id, "yaml-field-class");
        assert_eq!(related[1].title, "Field Class");
    }

    #[test]
    fn help_topic_markdown_includes_see_also() {
        let topic = topics()
            .iter()
            .find(|topic| topic.id == "item-driven-branching")
            .expect("topic should exist");

        let markdown = topic_markdown(topic);

        assert!(markdown.contains("## See Also"));
        assert!(markdown.contains("- Item Class (Lists)"));
        assert!(markdown.contains("- Field Class"));
    }

    #[test]
    fn format_placeholders_topic_explains_contains_order_and_empty_values() {
        let topic = topics()
            .iter()
            .find(|topic| topic.id == "format-placeholders")
            .expect("topic should exist");

        assert!(topic.body.contains(
            "Lists are presented to the user in the order they appear under `contains:`"
        ));
        assert!(topic.body.contains("replaces the placeholder with nothing"));
        assert!(topic.body.contains("walking with ."));
    }

    #[test]
    fn class_topics_see_also_direct_parents_and_children() {
        let cases = [
            ("yaml-template-class", vec!["Group Class"]),
            (
                "yaml-group-class",
                vec![
                    "Template Class",
                    "Section Class",
                    "Collection Class",
                    "Boilerplate Class",
                ],
            ),
            (
                "yaml-section-class",
                vec!["Group Class", "Field Class", "List Class"],
            ),
            (
                "yaml-collection-class",
                vec!["Group Class", "Field Class", "List Class"],
            ),
            (
                "yaml-field-class",
                vec![
                    "Section Class",
                    "Item Class (Lists)",
                    "List Class",
                    "Collection Class",
                    "Format Placeholders (Fields)",
                ],
            ),
            (
                "yaml-list-class",
                vec![
                    "Section Class",
                    "Collection Class",
                    "Field Class",
                    "Item Class (Lists)",
                    "Item Assigns Target Item to Target List",
                    "Output Prefixes & Suffixes (Lists)",
                ],
            ),
            (
                "yaml-list-item-class",
                vec![
                    "List Class",
                    "Field Class",
                    "Item Branching",
                    "Item Assigns Target Item to Target List",
                ],
            ),
            ("yaml-boilerplate-class", vec!["Group Class"]),
        ];

        for (topic_id, expected_titles) in cases {
            let topic = topics()
                .iter()
                .find(|topic| topic.id == topic_id)
                .expect("topic should exist");
            let titles = topic_see_also_topics(topic)
                .into_iter()
                .map(|topic| topic.title)
                .collect::<Vec<_>>();

            assert_eq!(
                titles, expected_titles,
                "unexpected See Also for {topic_id}"
            );
        }
    }

    #[test]
    fn related_help_topics_see_also_their_class_topics() {
        let cases = [
            (
                "format-placeholders",
                vec!["Field Class", "Output Prefixes & Suffixes (Lists)"],
            ),
            ("list-output-affixes", vec!["List Class"]),
            (
                "item-assigns-target",
                vec!["Item Class (Lists)", "List Class"],
            ),
        ];

        for (topic_id, expected_titles) in cases {
            let topic = topics()
                .iter()
                .find(|topic| topic.id == topic_id)
                .expect("topic should exist");
            let titles = topic_see_also_topics(topic)
                .into_iter()
                .map(|topic| topic.title)
                .collect::<Vec<_>>();

            assert_eq!(
                titles, expected_titles,
                "unexpected See Also for {topic_id}"
            );
        }
    }
}
