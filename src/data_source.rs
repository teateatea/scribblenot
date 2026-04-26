// Source-index helpers extracted from data.rs.
// Owns the YAML source mapping types (SourceIndex, SourceNode, the anchor
// types, and the LoadedHierarchy bundle) plus the small helpers that scan raw
// YAML text for line/column anchors. The actual YAML directory loader and the
// `build_source_index` glue that depends on `YamlDocument` stay in data.rs
// during slice 3 and will move into data_load.rs in slice 4.
//
// Private items are kept `pub(crate)` so the loader code still in data.rs can
// construct anchors and call helpers by their bare names. data.rs continues to
// be the single public surface via `pub use crate::data_source::*`.

use crate::data_model::{HierarchyChildRef, HierarchyFile, TypeTag};
use crate::diagnostics::ErrorSource;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct SourceIndex {
    pub nodes: HashMap<String, SourceNode>,
    child_refs: HashMap<ChildRefSourceKey, ErrorSource>,
}

impl SourceIndex {
    pub(crate) fn insert(&mut self, id: String, node: SourceNode) {
        self.nodes.entry(id).or_insert(node);
    }

    pub(crate) fn merge(&mut self, other: SourceIndex) {
        for (id, node) in other.nodes {
            self.insert(id, node);
        }
        for (key, source) in other.child_refs {
            self.child_refs.entry(key).or_insert(source);
        }
    }

    pub(crate) fn source_for(&self, id: &str) -> Option<ErrorSource> {
        self.nodes.get(id).map(|node| ErrorSource {
            file: node.file.clone(),
            line: node.line,
            quoted_line: node.quoted_line.clone(),
        })
    }

    pub(crate) fn insert_child_ref(
        &mut self,
        owner_id: &str,
        child: &HierarchyChildRef,
        source: ErrorSource,
    ) {
        self.child_refs
            .entry(ChildRefSourceKey {
                owner_id: owner_id.to_string(),
                child_kind: child.kind(),
                child_id: child.id().to_string(),
            })
            .or_insert(source);
    }

    pub(crate) fn source_for_child_ref(
        &self,
        owner_id: &str,
        child: &HierarchyChildRef,
    ) -> Option<ErrorSource> {
        self.child_refs
            .get(&ChildRefSourceKey {
                owner_id: owner_id.to_string(),
                child_kind: child.kind(),
                child_id: child.id().to_string(),
            })
            .cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ChildRefSourceKey {
    owner_id: String,
    child_kind: TypeTag,
    child_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceNode {
    pub file: PathBuf,
    pub line: usize,
    pub quoted_line: Option<String>,
    pub raw: serde_yaml::Value,
}

#[derive(Debug, Clone)]
pub struct LoadedHierarchy {
    pub hierarchy: HierarchyFile,
    pub source_index: SourceIndex,
}

#[derive(Debug, Clone)]
pub(crate) struct SourceAnchor {
    pub(crate) line: usize,
    pub(crate) quoted_line: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct EntryAnchor {
    pub(crate) anchor: SourceAnchor,
    pub(crate) start_idx: usize,
    pub(crate) end_idx: usize,
}

pub(crate) fn quoted_line(text: &str, relative_line: usize) -> Option<String> {
    text.lines()
        .nth(relative_line.saturating_sub(1))
        .map(|line| line.trim().to_string())
}

pub(crate) fn top_level_block_range(lines: &[&str], key: &str) -> Option<(usize, usize)> {
    let mut start_idx = None;
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if leading_spaces(line) == 0 && trimmed.starts_with(&format!("{key}:")) {
            start_idx = Some(idx);
            continue;
        }
        if start_idx.is_some()
            && leading_spaces(line) == 0
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            return Some((start_idx?, idx));
        }
    }
    start_idx.map(|start| (start, lines.len()))
}

pub(crate) fn find_mapping_anchor(
    doc_text: &str,
    start_line: usize,
    top_level_key: &str,
    id: &str,
) -> SourceAnchor {
    let mut current_key = None::<&str>;
    for (idx, line) in doc_text.lines().enumerate() {
        let absolute_line = start_line + idx;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if leading_spaces(line) == 0 {
            current_key = top_level_key_name(trimmed);
            continue;
        }
        if current_key == Some(top_level_key) && trimmed.starts_with("id:") {
            let maybe_id = trimmed
                .trim_start_matches("id:")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if maybe_id == id {
                return SourceAnchor {
                    line: absolute_line,
                    quoted_line: Some(trimmed.to_string()),
                };
            }
        }
    }
    SourceAnchor {
        line: start_line,
        quoted_line: None,
    }
}

pub(crate) fn collect_top_level_entry_anchors(
    doc_text: &str,
    start_line: usize,
) -> HashMap<String, Vec<EntryAnchor>> {
    let lines: Vec<&str> = doc_text.lines().collect();
    let mut anchors: HashMap<String, Vec<EntryAnchor>> = HashMap::new();
    let mut current_key: Option<String> = None;
    let mut idx = 0usize;

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            idx += 1;
            continue;
        }

        if leading_spaces(line) == 0 {
            current_key = top_level_key_name(trimmed)
                .filter(|key| {
                    matches!(
                        *key,
                        "groups" | "sections" | "collections" | "fields" | "lists" | "boilerplate"
                    )
                })
                .map(str::to_string);
            idx += 1;
            continue;
        }

        if leading_spaces(line) == 2 && trimmed.starts_with("- ") {
            if let Some(key) = current_key.clone() {
                let start_idx = idx;
                idx += 1;
                while idx < lines.len() {
                    let next = lines[idx];
                    let next_trimmed = next.trim();
                    if next_trimmed.is_empty() || next_trimmed.starts_with('#') {
                        idx += 1;
                        continue;
                    }
                    if leading_spaces(next) == 0
                        || (leading_spaces(next) == 2 && next_trimmed.starts_with("- "))
                    {
                        break;
                    }
                    idx += 1;
                }
                let anchor = find_entry_anchor(&lines[start_idx..idx], start_line + start_idx)
                    .unwrap_or(SourceAnchor {
                        line: start_line + start_idx,
                        quoted_line: Some(lines[start_idx].trim().to_string()),
                    });
                anchors.entry(key).or_default().push(EntryAnchor {
                    anchor,
                    start_idx,
                    end_idx: idx,
                });
                continue;
            }
        }

        idx += 1;
    }

    anchors
}

pub(crate) fn collect_child_ref_anchors(
    entry_lines: &[&str],
    start_line: usize,
) -> Vec<SourceAnchor> {
    entry_lines
        .iter()
        .enumerate()
        .filter_map(|(offset, line)| {
            let trimmed = line.trim();
            let matched = [
                "- group:",
                "- section:",
                "- collection:",
                "- field:",
                "- list:",
            ]
            .iter()
            .any(|prefix| trimmed.starts_with(prefix));
            matched.then(|| SourceAnchor {
                line: start_line + offset,
                quoted_line: Some(trimmed.to_string()),
            })
        })
        .collect()
}

pub(crate) fn child_ref_from_value(value: &serde_yaml::Value) -> Option<HierarchyChildRef> {
    serde_yaml::from_value(value.clone()).ok()
}

pub(crate) fn find_entry_anchor(entry_lines: &[&str], start_line: usize) -> Option<SourceAnchor> {
    for (offset, line) in entry_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("- id:") || trimmed.starts_with("id:") || trimmed.contains("{ id:") {
            return Some(SourceAnchor {
                line: start_line + offset,
                quoted_line: Some(trimmed.to_string()),
            });
        }
    }
    None
}

pub(crate) fn top_level_key_name(line: &str) -> Option<&str> {
    line.split_once(':').map(|(key, _)| key)
}

pub(crate) fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
}
