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

    TOPICS
        .iter()
        .enumerate()
        .filter_map(|(idx, topic)| topic_matches(topic, &normalized).then_some(idx))
        .collect()
}

pub fn topic_code_blocks(topic: &HelpTopic) -> Vec<HelpCodeBlock> {
    parse_code_blocks(topic.body)
}

pub fn topic_markdown(topic: &HelpTopic) -> String {
    format!("# {}\n\n{}", topic.title, topic.body.trim())
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

    fn flush_paragraph(
        y: &mut f32,
        paragraph_lines: &mut usize,
        has_previous_element: &mut bool,
    ) {
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

fn topic_matches(topic: &HelpTopic, query: &str) -> bool {
    topic.title.to_ascii_lowercase().contains(query)
        || topic.id.to_ascii_lowercase().contains(query)
        || topic
            .aliases
            .iter()
            .any(|alias| alias.to_ascii_lowercase().contains(query))
        || topic.body.to_ascii_lowercase().contains(query)
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

static TOPICS: [HelpTopic; 3] = [
    HelpTopic {
        id: "item-driven-branching",
        title: "Branching Fields",
        aliases: &[
            "branch fields",
            "branching",
            "item-driven branching",
            "item-driven branching fields",
            "item fields",
            "conditional fields",
            "how do I show fields from an item",
        ],
        body: r#"Use branching fields when one list choice should open an extra follow-up field.

The overview is: put `fields:` on the item that needs follow-up questions. Each referenced field must still be authored as a normal field elsewhere in the YAML data, and the field controls its own lists or collections.

```yaml
fields:
  - id: exercise_recent
    label: Exercise
    format: "Pt describes {exercise_level} in the past 2 weeks{exercise_due_to}."
    contains:
      - list: exercise_level
      - list: exercise_due_to

  - id: exercise_reason_field
    label: Due to
    format: " due to {exercise_reason}"
    contains:
      - list: exercise_reason

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
        output: "{exercise_reason_field}"
        fields:
          - exercise_reason_field

  - id: exercise_reason
    items:
      - "pain"
      - "fatigue"
      - "time constraints"
      - "recent illness"
```

## How To

### Step 1

When starting a split from the item level, the branching item uses `fields:` to open the named follow-up fields.

```yaml
lists:
  - id: exercise_due_to
    items:
      - id: no_due_to
        label: "[no reason]"
        output: ""
      - id: due_to
        label: "DUE TO"
        output: "{exercise_reason_field}"
        fields:
          - exercise_reason_field
```

Use this small fragment when you are adding one branching item to an existing list.

```yaml
- id: example_item
  label: "label_text"
  output: "{followup_field}"
  fields:
    - followup_field
```

### Step 2

If the follow-up field doesn't already exist, set it up now. This is the intermediary step that creates the branch.

```yaml
fields:
  - id: exercise_reason_field
    label: Due to
    format: " due to {exercise_reason}"
    contains:
      - list: exercise_reason
```

### Step 3

The follow-up field is populated with the follow-up answer list. This is where you fill out the options.

```yaml
lists:
  - id: exercise_reason
    items:
      - "pain"
      - "fatigue"
      - "time constraints"
      - "recent illness"
```

### Step 4

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

### Step 5

Write the main answer list. These are the normal choices the field can output.

```yaml
lists:
  - id: exercise_level
    items:
      - "regular exercise"
      - "limited exercise"
      - "no exercise"
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
        id: "format-placeholders",
        title: "Format Placeholders",
        aliases: &[
            "format",
            "placeholders",
            "field format",
            "how do I combine lists",
            "curly braces",
        ],
        body: r#"Use `format:` when a field should combine one or more selected list values into a sentence fragment.

Each `{placeholder}` should match a list id that the field can reach through `contains:`.

```yaml
fields:
  - id: treatment_area_summary
    label: Treatment area summary
    format: "{region} treated with {technique}"
    contains:
      - list: region
      - list: technique
```

Keep literal text outside the braces. Put only the list id inside each placeholder."#,
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
        body: r#"In a help topic, the copy key copies the selected code block. If the topic has no code blocks, it copies the whole topic as markdown.

Use left and right navigation to cycle code blocks. You can also press the hint shown beside a code block, using the same hint-key pool as the rest of Scribblenot.

```yaml
keybindings:
  copy_note: [c]
  hints: [q, w, f, p, a, r, s, t]
```

The help system reads the current `copy_note` and `hints` keybindings, so changing those bindings changes the help modal too."#,
    },
];

#[cfg(test)]
mod tests {
    use super::{
        filtered_topic_indices, topic_code_block_scroll_offset, topic_code_blocks, topics,
    };

    #[test]
    fn help_search_matches_aliases() {
        let matches = filtered_topic_indices("curly braces");
        assert_eq!(topics()[matches[0]].id, "format-placeholders");
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
        assert!(blocks[0].code.contains("exercise_reason_field"));
        assert!(blocks[1].code.contains("exercise_due_to"));
        assert!(blocks[2].code.contains("- id: example_item"));
        assert!(blocks[2].code.contains("{followup_field}"));
        assert!(blocks[6].code.contains("exercise_level"));
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
}
