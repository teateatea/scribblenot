use crate::data::{HeaderFieldConfig, HierarchyList};
use crate::modal::{active_collection_ids, decode_collection_display_value, format_collection_field_value};
use crate::sections::header::HeaderFieldValue;
use crate::sections::collection::CollectionState;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ResolvedMultiFieldValue {
    Empty,
    Partial,
    Complete(String),
}

impl ResolvedMultiFieldValue {
    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete(_))
    }

    #[allow(dead_code)]
    pub fn is_empty_variant(&self) -> bool {
        matches!(self, Self::Empty)
    }

    #[allow(dead_code)]
    pub fn preview_str(&self) -> &str {
        match self {
            Self::Complete(s) => s.as_str(),
            _ => "--",
        }
    }

    pub fn export_value(&self) -> Option<&str> {
        match self {
            Self::Complete(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

pub fn resolve_multifield_value(
    confirmed: &HeaderFieldValue,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    match confirmed {
        HeaderFieldValue::ExplicitEmpty => return ResolvedMultiFieldValue::Empty,
        HeaderFieldValue::CollectionState(value) if !cfg.collections.is_empty() && cfg.lists.is_empty() => {
            if let Some(decoded) = decode_collection_display_value(value, cfg) {
                if decoded.is_empty() {
                    return ResolvedMultiFieldValue::Empty;
                }
                return ResolvedMultiFieldValue::Complete(decoded);
            }
            return ResolvedMultiFieldValue::Empty;
        }
        HeaderFieldValue::Text(value) if !value.is_empty() => {
            return ResolvedMultiFieldValue::Complete(value.to_string())
        }
        HeaderFieldValue::CollectionState(_) => return ResolvedMultiFieldValue::Empty,
        HeaderFieldValue::Text(_) => {}
    }
    if cfg.lists.is_empty() && cfg.collections.is_empty() {
        return ResolvedMultiFieldValue::Empty;
    }
    if !cfg.collections.is_empty() && cfg.lists.is_empty() {
        let state = CollectionState::new(cfg.collections.clone());
        let value = format_collection_field_value(&state.collections, cfg.format.is_some());
        if value.is_empty() {
            return ResolvedMultiFieldValue::Empty;
        }
        return ResolvedMultiFieldValue::Complete(value);
    }

    let mut values: Vec<Option<String>> = Vec::new();
    for list in &cfg.lists {
        values.push(resolve_list_value(list, sticky_values));
    }

    let resolved_count = values.iter().filter(|value| value.is_some()).count();
    if resolved_count == 0 {
        return ResolvedMultiFieldValue::Empty;
    }
    if resolved_count < cfg.lists.len() {
        return ResolvedMultiFieldValue::Partial;
    }

    if let Some(format) = &cfg.format {
        let mut result = format.clone();
        for (idx, value) in values.into_iter().enumerate() {
            if let Some(value) = value {
                result = result.replace(&format!("{{{}}}", cfg.lists[idx].id), &value);
            }
        }
        for list in &cfg.format_lists {
            let placeholder = format!("{{{}}}", list.id);
            if !result.contains(&placeholder) {
                continue;
            }
            let value = resolve_list_value(list, sticky_values).unwrap_or_default();
            result = result.replace(&placeholder, &value);
        }
        ResolvedMultiFieldValue::Complete(result)
    } else {
        ResolvedMultiFieldValue::Complete(values[0].clone().unwrap_or_default())
    }
}

pub fn resolve_field_label(
    confirmed: &HeaderFieldValue,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> String {
    if !cfg.name.contains('{') {
        return cfg.name.clone();
    }

    let mut result = cfg.name.clone();

    for list in &cfg.lists {
        let placeholder = format!("{{{}}}", list.id);
        if !result.contains(&placeholder) {
            continue;
        }
        let value = if matches!(confirmed, HeaderFieldValue::Text(value) if !value.is_empty()) && cfg.lists.len() == 1 {
            confirmed.as_text().map(ToOwned::to_owned)
        } else {
            resolve_list_value(list, sticky_values)
        }
        .unwrap_or_else(|| list.preview.clone().unwrap_or_else(|| placeholder.clone()));
        result = result.replace(&placeholder, &value);
    }

    for collection in &cfg.collections {
        let placeholder = format!("{{{}}}", collection.id);
        if !result.contains(&placeholder) {
            continue;
        }
        let value = if cfg.collections.len() == 1 {
            match confirmed {
                HeaderFieldValue::CollectionState(value) => {
                    let active_ids = active_collection_ids(value);
                    active_ids
                        .iter()
                        .find(|id| *id == &collection.id)
                        .map(|_| collection.label.clone())
                        .or_else(|| decode_collection_display_value(value, cfg))
                }
                HeaderFieldValue::Text(value) if !value.is_empty() => Some(value.to_string()),
                _ => None,
            }
        } else if collection.default_enabled {
            Some(collection.label.clone())
        } else {
            None
        }
        .unwrap_or(placeholder.clone());
        result = result.replace(&placeholder, &value);
    }

    result
}

fn resolve_list_value(
    list: &HierarchyList,
    sticky_values: &HashMap<String, String>,
) -> Option<String> {
    if list.sticky {
        if let Some(value) = sticky_values.get(&list.id) {
            if !value.is_empty() {
                return Some(value.clone());
            }
        }
    }

    if let Some(default) = &list.default {
        if let Some(item) = list.items.iter().find(|item| {
            item.id == *default
                || item.ui_label() == *default
                || item.output.as_deref() == Some(default.as_str())
        }) {
            return Some(item.output().to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ResolvedCollectionConfig;

    #[test]
    fn explicit_empty_overrides_collection_default_enabled() {
        let cfg = HeaderFieldConfig {
            id: "back".to_string(),
            name: "BACK".to_string(),
            format: None,
            lists: Vec::new(),
            collections: vec![ResolvedCollectionConfig {
                id: "all_back".to_string(),
                label: "ALL BACK".to_string(),
                note_label: Some("#### ALL BACK".to_string()),
                default_enabled: true,
                joiner_style: None,
                lists: Vec::new(),
            }],
            format_lists: Vec::new(),
            max_entries: None,
            max_actives: None,
        };

        let resolved = resolve_multifield_value(
            &crate::sections::header::HeaderFieldValue::ExplicitEmpty,
            &cfg,
            &HashMap::new(),
        );

        assert!(matches!(resolved, ResolvedMultiFieldValue::Empty));
    }
}
