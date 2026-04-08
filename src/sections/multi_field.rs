use crate::data::{HeaderFieldConfig, HierarchyList};
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

    pub fn is_empty_variant(&self) -> bool {
        matches!(self, Self::Empty)
    }

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
    confirmed: &str,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    if !confirmed.is_empty() {
        return ResolvedMultiFieldValue::Complete(confirmed.to_string());
    }
    if cfg.lists.is_empty() {
        return ResolvedMultiFieldValue::Empty;
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
                || item.label == *default
                || item.output.as_deref() == Some(default.as_str())
        }) {
            return Some(item.output.clone().unwrap_or_else(|| item.label.clone()));
        }
    }

    None
}
