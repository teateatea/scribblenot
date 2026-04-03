// flat_file.rs — flat YAML data structures for scribblenot form definitions.

use serde::{Deserialize, Serialize};
use crate::data::{CompositeConfig, PartOption};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum FlatBlock {
    Box { id: String, #[serde(default)] children: Vec<String> },
    Group {
        id: String,
        #[serde(default)] children: Vec<String>,
        #[serde(default)] name: Option<String>,
        #[serde(default)] num: Option<usize>,
    },
    Section {
        id: String,
        #[serde(default)] children: Vec<String>,
        #[serde(default)] name: Option<String>,
        #[serde(default)] map_label: Option<String>,
        #[serde(default)] section_type: Option<String>,
        #[serde(default)] data_file: Option<String>,
        #[serde(default)] date_prefix: Option<bool>,
    },
    Field {
        id: String,
        #[serde(default)] children: Vec<String>,
        #[serde(default)] name: Option<String>,
        #[serde(default)] options: Vec<String>,
        #[serde(default)] composite: Option<CompositeConfig>,
        #[serde(default)] default: Option<String>,
    },
    OptionsList {
        id: String,
        #[serde(default)] children: Vec<String>,
        #[serde(default)] entries: Vec<PartOption>,
    },
    Boilerplate {
        id: String,
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatFile {
    pub blocks: Vec<FlatBlock>,
}

#[cfg(test)]
mod tests {
    // Import the types that should exist in this module once implemented.
    // These tests verify the structure defined in task #45 sub-task 1:
    //   - FlatBlock: an enum/struct with a `type` discriminant and an `id: String` field,
    //     covering box, group, section, field, and options-list variants.
    //   - FlatFile: a wrapper that serde_yaml can deserialize a single yml file into
    //     (essentially a list of FlatBlock items).

    use super::{FlatBlock, FlatFile};

    #[test]
    fn flat_block_box_variant_has_id() {
        // A FlatBlock of type "box" must carry an id field.
        let block = FlatBlock::Box { id: "box1".to_string(), children: vec![] };
        match &block {
            FlatBlock::Box { id, .. } => assert_eq!(id, "box1"),
            _ => panic!("expected Box variant"),
        }
    }

    #[test]
    fn flat_block_group_variant_has_id() {
        let block = FlatBlock::Group { id: "grp1".to_string(), children: vec![], name: None, num: None };
        match &block {
            FlatBlock::Group { id, .. } => assert_eq!(id, "grp1"),
            _ => panic!("expected Group variant"),
        }
    }

    #[test]
    fn flat_block_section_variant_has_id() {
        let block = FlatBlock::Section { id: "sec1".to_string(), children: vec![], name: None, map_label: None, section_type: None, data_file: None, date_prefix: None };
        match &block {
            FlatBlock::Section { id, .. } => assert_eq!(id, "sec1"),
            _ => panic!("expected Section variant"),
        }
    }

    #[test]
    fn flat_block_field_variant_has_id() {
        let block = FlatBlock::Field { id: "fld1".to_string(), children: vec![], name: None, options: vec![], composite: None, default: None };
        match &block {
            FlatBlock::Field { id, .. } => assert_eq!(id, "fld1"),
            _ => panic!("expected Field variant"),
        }
    }

    #[test]
    fn flat_block_options_list_variant_has_id() {
        let block = FlatBlock::OptionsList { id: "opt1".to_string(), children: vec![], entries: vec![] };
        match &block {
            FlatBlock::OptionsList { id, .. } => assert_eq!(id, "opt1"),
            _ => panic!("expected OptionsList variant"),
        }
    }

    #[test]
    fn flat_file_holds_list_of_blocks() {
        // FlatFile must contain a Vec<FlatBlock> (the flat list of blocks from one yml file).
        let file = FlatFile {
            blocks: vec![
                FlatBlock::Box { id: "b1".to_string(), children: vec![] },
                FlatBlock::Section { id: "s1".to_string(), children: vec![], name: None, map_label: None, section_type: None, data_file: None, date_prefix: None },
            ],
        };
        assert_eq!(file.blocks.len(), 2);
    }

    #[test]
    fn flat_file_deserializes_from_yaml() {
        // FlatFile must be deserializable by serde_yaml from a yml list.
        // The yaml uses a `type` discriminant field to pick the FlatBlock variant.
        let yaml = r#"
blocks:
  - type: box
    id: box_a
  - type: group
    id: grp_a
  - type: section
    id: sec_a
  - type: field
    id: fld_a
  - type: options-list
    id: opt_a
"#;
        let file: FlatFile = serde_yaml::from_str(yaml).expect("deserialization failed");
        assert_eq!(file.blocks.len(), 5);
    }

    #[test]
    fn flat_block_id_is_string() {
        // Verify at compile time that id is a String (not &str or numeric).
        let block = FlatBlock::Field { id: String::from("field_id"), children: vec![], name: None, options: vec![], composite: None, default: None };
        let _id: String = match block {
            FlatBlock::Field { id, .. } => id,
            _ => unreachable!(),
        };
    }

    // --- Tests for extended FlatBlock metadata fields (task #45 sub-task 1) ---
    // These tests FAIL until FlatBlock variants are extended with the new fields.

    #[test]
    fn group_block_deserializes_name_and_num() {
        // Group variant must carry `name: Option<String>` and `num: Option<usize>`.
        let yaml = r#"
blocks:
  - type: group
    id: grp_meta
    name: "Injuries"
    num: 3
"#;
        let file: FlatFile = serde_yaml::from_str(yaml).expect("deserialization failed");
        match &file.blocks[0] {
            FlatBlock::Group { id, name, num, .. } => {
                assert_eq!(id, "grp_meta");
                assert_eq!(name.as_deref(), Some("Injuries"));
                assert_eq!(*num, Some(3usize));
            }
            _ => panic!("expected Group variant"),
        }
    }

    #[test]
    fn section_block_deserializes_name_map_label_section_type() {
        // Section variant must carry `name`, `map_label`, and `section_type` fields.
        let yaml = r#"
blocks:
  - type: section
    id: sec_meta
    name: "Head Injuries"
    map_label: "Head"
    section_type: "multi_field"
"#;
        let file: FlatFile = serde_yaml::from_str(yaml).expect("deserialization failed");
        match &file.blocks[0] {
            FlatBlock::Section { id, name, map_label, section_type, .. } => {
                assert_eq!(id, "sec_meta");
                assert_eq!(name.as_deref(), Some("Head Injuries"));
                assert_eq!(map_label.as_deref(), Some("Head"));
                assert_eq!(section_type.as_deref(), Some("multi_field"));
            }
            _ => panic!("expected Section variant"),
        }
    }

    #[test]
    fn options_list_block_deserializes_entries() {
        // OptionsList variant must carry `entries: Vec<PartOption>` where each entry
        // is a PartOption-shaped value (untagged: simple string OR {label, output}).
        let yaml = r#"
blocks:
  - type: options-list
    id: opts_meta
    entries:
      - "simple option"
      - label: "Labeled"
        output: "labeled_output"
"#;
        let file: FlatFile = serde_yaml::from_str(yaml).expect("deserialization failed");
        match &file.blocks[0] {
            FlatBlock::OptionsList { id, entries, .. } => {
                assert_eq!(id, "opts_meta");
                assert_eq!(entries.len(), 2);
            }
            _ => panic!("expected OptionsList variant"),
        }
    }

    // --- Tests for FlatBlock::Boilerplate variant (task #52 sub-task 1) ---

    #[test]
    fn boilerplate_variant_deserializes_from_yaml() {
        // A FlatBlock of type "boilerplate" must deserialize with id and text fields.
        let yaml = r#"
blocks:
  - type: boilerplate
    id: bp_intro
    text: "This is the boilerplate text content."
"#;
        let file: FlatFile = serde_yaml::from_str(yaml).expect("deserialization failed");
        assert_eq!(file.blocks.len(), 1);
        match &file.blocks[0] {
            FlatBlock::Boilerplate { id, text } => {
                assert_eq!(id, "bp_intro");
                assert_eq!(text, "This is the boilerplate text content.");
            }
            _ => panic!("expected Boilerplate variant"),
        }
    }

    #[test]
    fn boilerplate_variant_id_and_text_are_correct() {
        // Verify that id and text are independently extracted correctly.
        let block = FlatBlock::Boilerplate {
            id: "my_bp".to_string(),
            text: "Hello, world!".to_string(),
        };
        match &block {
            FlatBlock::Boilerplate { id, text } => {
                assert_eq!(id, "my_bp");
                assert_eq!(text, "Hello, world!");
            }
            _ => panic!("expected Boilerplate variant"),
        }
    }

    #[test]
    fn boilerplate_missing_id_fails_deserialization() {
        // Deserialization must fail when the required `id` field is absent.
        let yaml = r#"
blocks:
  - type: boilerplate
    text: "Missing id field."
"#;
        let result: Result<FlatFile, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "expected deserialization to fail when id is missing");
    }

    #[test]
    fn boilerplate_missing_text_fails_deserialization() {
        // Deserialization must fail when the required `text` field is absent.
        let yaml = r#"
blocks:
  - type: boilerplate
    id: bp_no_text
"#;
        let result: Result<FlatFile, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "expected deserialization to fail when text is missing");
    }
}
