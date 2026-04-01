// flat_file.rs — flat YAML data structures for scribblenot form definitions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum FlatBlock {
    Box { id: String, #[serde(default)] children: Vec<String> },
    Group { id: String, #[serde(default)] children: Vec<String> },
    Section { id: String, #[serde(default)] children: Vec<String> },
    Field { id: String, #[serde(default)] children: Vec<String> },
    OptionsList { id: String, #[serde(default)] children: Vec<String> },
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
        let block = FlatBlock::Group { id: "grp1".to_string(), children: vec![] };
        match &block {
            FlatBlock::Group { id, .. } => assert_eq!(id, "grp1"),
            _ => panic!("expected Group variant"),
        }
    }

    #[test]
    fn flat_block_section_variant_has_id() {
        let block = FlatBlock::Section { id: "sec1".to_string(), children: vec![] };
        match &block {
            FlatBlock::Section { id, .. } => assert_eq!(id, "sec1"),
            _ => panic!("expected Section variant"),
        }
    }

    #[test]
    fn flat_block_field_variant_has_id() {
        let block = FlatBlock::Field { id: "fld1".to_string(), children: vec![] };
        match &block {
            FlatBlock::Field { id, .. } => assert_eq!(id, "fld1"),
            _ => panic!("expected Field variant"),
        }
    }

    #[test]
    fn flat_block_options_list_variant_has_id() {
        let block = FlatBlock::OptionsList { id: "opt1".to_string(), children: vec![] };
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
                FlatBlock::Section { id: "s1".to_string(), children: vec![] },
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
        let block = FlatBlock::Field { id: String::from("field_id"), children: vec![] };
        let _id: String = match block {
            FlatBlock::Field { id, .. } => id,
            _ => unreachable!(),
        };
    }
}
