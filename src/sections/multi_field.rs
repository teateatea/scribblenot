use crate::data::{HeaderFieldConfig, HierarchyList, JoinerStyle};
use crate::modal::{
    active_collection_ids, decode_collection_display_value, format_collection_field_value,
};
use crate::sections::collection::CollectionState;
use crate::sections::header::{HeaderFieldValue, HeaderState};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ResolvedMultiFieldValue {
    Empty,
    Partial(String),
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

    pub fn display_value(&self) -> Option<&str> {
        match self {
            Self::Partial(value) | Self::Complete(value) if !value.is_empty() => {
                Some(value.as_str())
            }
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn preview_str(&self) -> &str {
        self.display_value().unwrap_or("--")
    }

    pub fn export_value(&self) -> Option<&str> {
        match self {
            Self::Complete(value) => Some(value.as_str()),
            _ => None,
        }
    }
}

pub fn resolve_multifield_value(
    confirmed: &HeaderFieldValue,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    resolve_field_values(std::slice::from_ref(confirmed), cfg, sticky_values)
}

pub fn resolve_field_values(
    confirmed_values: &[HeaderFieldValue],
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    let has_explicit_empty = confirmed_values
        .iter()
        .any(|value| matches!(value, HeaderFieldValue::ExplicitEmpty));
    let concrete: Vec<&HeaderFieldValue> = confirmed_values
        .iter()
        .filter(|value| !matches!(value, HeaderFieldValue::ExplicitEmpty))
        .collect();

    if cfg.max_entries.is_some() {
        return resolve_repeating_values(&concrete, has_explicit_empty, cfg, sticky_values);
    }

    if let Some(value) = concrete.first() {
        return resolve_single_value(value, cfg, sticky_values);
    }

    if has_explicit_empty {
        ResolvedMultiFieldValue::Empty
    } else {
        resolve_unconfirmed_value(cfg, sticky_values)
    }
}

pub fn render_field_display(
    confirmed_values: &[HeaderFieldValue],
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> String {
    match resolve_field_values(confirmed_values, cfg, sticky_values) {
        ResolvedMultiFieldValue::Empty => "[empty]".to_string(),
        ResolvedMultiFieldValue::Partial(value) | ResolvedMultiFieldValue::Complete(value) => value,
    }
}

fn display_template(cfg: &HeaderFieldConfig, prefer_preview: bool) -> String {
    let template = if prefer_preview {
        cfg.preview.as_ref().or(cfg.format.as_ref())
    } else {
        cfg.format.as_ref().or(cfg.preview.as_ref())
    };
    template.cloned().unwrap_or_else(|| {
        cfg.lists
            .first()
            .map(|list| format!("{{{}}}", list.id))
            .or_else(|| cfg.fields.first().map(|field| format!("{{{}}}", field.id)))
            .unwrap_or_default()
    })
}

pub fn resolve_field_label(
    confirmed: &HeaderFieldValue,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> String {
    if !cfg.name.contains('{') {
        return cfg.name.clone();
    }

    if !cfg.fields.is_empty() {
        let nested_state = match confirmed {
            HeaderFieldValue::NestedState(state) => Some(state.as_ref()),
            _ => None,
        };
        let placeholders = cfg
            .fields
            .iter()
            .enumerate()
            .map(|(idx, child)| {
                let value = nested_state
                    .and_then(|state| state.repeated_values.get(idx))
                    .map(|values| resolve_field_values(values, child, sticky_values))
                    .unwrap_or_else(|| resolve_field_values(&[], child, sticky_values))
                    .display_value()
                    .map(str::to_string)
                    .or_else(|| child.preview.clone());
                (child.id.clone(), value)
            })
            .collect::<Vec<_>>();
        return render_template(&cfg.name, &placeholders, &cfg.format_lists, sticky_values);
    }

    let mut placeholders = Vec::new();
    for list in &cfg.lists {
        let value = if matches!(confirmed, HeaderFieldValue::Text(value) if !value.is_empty())
            && cfg.lists.len() == 1
        {
            confirmed.as_text().map(ToOwned::to_owned)
        } else {
            resolve_list_value(list, sticky_values)
        }
        .or_else(|| list.preview.clone())
        .or_else(|| Some(format!("{{{}}}", list.id)));
        placeholders.push((list.id.clone(), value));
    }

    for collection in &cfg.collections {
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
        .or_else(|| Some(format!("{{{}}}", collection.id)));
        placeholders.push((collection.id.clone(), value));
    }

    render_template(&cfg.name, &placeholders, &cfg.format_lists, sticky_values)
}

fn resolve_repeating_values(
    concrete: &[&HeaderFieldValue],
    has_explicit_empty: bool,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    let mut values = Vec::new();
    let mut saw_partial = false;

    for value in concrete {
        match resolve_single_value(value, cfg, sticky_values) {
            ResolvedMultiFieldValue::Complete(value) => values.push(value),
            ResolvedMultiFieldValue::Partial(value) => {
                if !value.is_empty() {
                    values.push(value);
                }
                saw_partial = true;
            }
            ResolvedMultiFieldValue::Empty => {}
        }
    }

    if values.is_empty() {
        if has_explicit_empty {
            return ResolvedMultiFieldValue::Empty;
        }
        return resolve_unconfirmed_value(cfg, sticky_values);
    }

    let combined = if let Some(style) = cfg.joiner_style.as_ref() {
        join_repeat_values(&values, style)
    } else if cfg.format.is_none() {
        values.join("\n")
    } else {
        values.join(", ")
    };

    if saw_partial {
        ResolvedMultiFieldValue::Partial(combined)
    } else {
        ResolvedMultiFieldValue::Complete(combined)
    }
}

fn resolve_single_value(
    confirmed: &HeaderFieldValue,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    match confirmed {
        HeaderFieldValue::ExplicitEmpty => ResolvedMultiFieldValue::Empty,
        HeaderFieldValue::CollectionState(value)
            if !cfg.collections.is_empty() && cfg.lists.is_empty() =>
        {
            decode_collection_display_value(value, cfg)
                .filter(|value| !value.is_empty())
                .map(ResolvedMultiFieldValue::Complete)
                .unwrap_or(ResolvedMultiFieldValue::Empty)
        }
        HeaderFieldValue::Text(value) if !value.is_empty() => {
            ResolvedMultiFieldValue::Complete(value.to_string())
        }
        HeaderFieldValue::ListState(value) if !cfg.lists.is_empty() => {
            resolve_list_state(value, cfg, sticky_values)
        }
        HeaderFieldValue::NestedState(state) if !cfg.fields.is_empty() => {
            resolve_nested_state(state, cfg, sticky_values)
        }
        _ => resolve_unconfirmed_value(cfg, sticky_values),
    }
}

fn resolve_list_state(
    value: &crate::sections::header::ListFieldValue,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    let mut placeholders = Vec::new();
    let mut all_complete = true;
    let mut any_display = false;

    for (idx, list) in cfg.lists.iter().enumerate() {
        let display = if let Some(saved) = value.values.get(idx) {
            if saved.is_empty() {
                None
            } else {
                Some(saved.clone())
            }
        } else if idx == value.list_idx && !value.repeat_values.is_empty() {
            joined_repeating_value(value, list).filter(|joined| !joined.is_empty())
        } else {
            None
        }
        .or_else(|| {
            if value.values.is_empty() && value.repeat_values.is_empty() {
                resolve_list_value(list, sticky_values)
            } else {
                None
            }
        })
        .or_else(|| {
            if value.values.is_empty() && value.repeat_values.is_empty() {
                list.preview.clone()
            } else {
                None
            }
        });

        if display.is_some() {
            any_display = true;
        }
        let resolved_for_completion = value.values.get(idx).is_some_and(|saved| !saved.is_empty())
            || (value.values.is_empty()
                && value.repeat_values.is_empty()
                && resolve_list_value(list, sticky_values).is_some());
        if !resolved_for_completion {
            all_complete = false;
        }
        placeholders.push((list.id.clone(), display.or_else(|| Some(String::new()))));
    }

    if !any_display {
        return resolve_unconfirmed_value(cfg, sticky_values);
    }

    let template = display_template(cfg, false);
    let rendered = render_template(&template, &placeholders, &cfg.format_lists, sticky_values);

    if all_complete && value.list_idx >= cfg.lists.len() {
        ResolvedMultiFieldValue::Complete(rendered)
    } else {
        ResolvedMultiFieldValue::Partial(rendered)
    }
}

fn joined_repeating_value(
    value: &crate::sections::header::ListFieldValue,
    list: &HierarchyList,
) -> Option<String> {
    if value.repeat_values.is_empty() {
        return None;
    }
    let style = list
        .joiner_style
        .as_ref()
        .or(if list.max_entries.is_some() {
            Some(&JoinerStyle::Comma)
        } else {
            None
        });
    style
        .map(|style| join_repeat_values(&value.repeat_values, style))
        .or_else(|| Some(value.repeat_values.join(", ")))
}

fn resolve_nested_state(
    state: &HeaderState,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    let mut placeholders: Vec<(String, Option<String>)> = Vec::new();
    let mut any_display = false;
    let mut all_complete = true;

    for (idx, child) in cfg.fields.iter().enumerate() {
        let values = state
            .repeated_values
            .get(idx)
            .map(|values| values.as_slice())
            .unwrap_or(&[]);
        let resolved = resolve_field_values(values, child, sticky_values);
        let display = resolved
            .display_value()
            .map(str::to_string)
            .or_else(|| child.preview.clone());
        if display.is_some() {
            any_display = true;
        }
        if !matches!(resolved, ResolvedMultiFieldValue::Complete(_)) {
            all_complete = false;
        }
        placeholders.push((child.id.clone(), display));
    }

    if !any_display {
        if let Some(preview) = cfg.preview.clone() {
            return ResolvedMultiFieldValue::Partial(preview);
        }
        if cfg.format_lists.is_empty() {
            return ResolvedMultiFieldValue::Empty;
        }
        let template = display_template(cfg, true);
        let rendered = render_template(&template, &placeholders, &cfg.format_lists, sticky_values);
        return if rendered.is_empty() {
            ResolvedMultiFieldValue::Empty
        } else {
            ResolvedMultiFieldValue::Partial(rendered)
        };
    }

    let template = display_template(cfg, false);
    let rendered = render_template(&template, &placeholders, &cfg.format_lists, sticky_values);

    if all_complete {
        ResolvedMultiFieldValue::Complete(rendered)
    } else {
        ResolvedMultiFieldValue::Partial(rendered)
    }
}

fn resolve_unconfirmed_value(
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    if !cfg.fields.is_empty() {
        return resolve_nested_state(&HeaderState::new(cfg.fields.clone()), cfg, sticky_values);
    }

    if cfg.lists.is_empty() && cfg.collections.is_empty() {
        if let Some(preview) = cfg.preview.clone() {
            return ResolvedMultiFieldValue::Partial(preview);
        }
        return ResolvedMultiFieldValue::Empty;
    }

    if !cfg.collections.is_empty() && cfg.lists.is_empty() {
        let state = CollectionState::new(cfg.collections.clone());
        let value = format_collection_field_value(&state.collections, cfg.format.is_some());
        if !value.is_empty() {
            return ResolvedMultiFieldValue::Complete(value);
        }
        return cfg
            .preview
            .clone()
            .map(ResolvedMultiFieldValue::Partial)
            .unwrap_or(ResolvedMultiFieldValue::Empty);
    }

    let mut placeholders = Vec::new();
    let mut real_count = 0usize;
    let mut any_display = false;
    for list in &cfg.lists {
        let value = resolve_list_value(list, sticky_values)
            .map(|value| {
                real_count += 1;
                value
            })
            .or_else(|| list.preview.clone());
        if value.is_some() {
            any_display = true;
        }
        placeholders.push((list.id.clone(), value));
    }

    if !any_display {
        if let Some(preview) = cfg.preview.clone() {
            return ResolvedMultiFieldValue::Partial(preview);
        }
        if cfg.format_lists.is_empty() {
            return ResolvedMultiFieldValue::Empty;
        }
        let template = display_template(cfg, true);
        let blank_placeholders = cfg
            .lists
            .iter()
            .map(|list| (list.id.clone(), Some(String::new())))
            .collect::<Vec<_>>();
        let rendered = render_template(
            &template,
            &blank_placeholders,
            &cfg.format_lists,
            sticky_values,
        );
        return if rendered.is_empty() {
            ResolvedMultiFieldValue::Empty
        } else {
            ResolvedMultiFieldValue::Partial(rendered)
        };
    }

    let template = display_template(cfg, false);
    let rendered = render_template(&template, &placeholders, &cfg.format_lists, sticky_values);
    if real_count == cfg.lists.len() && !cfg.lists.is_empty() {
        ResolvedMultiFieldValue::Complete(rendered)
    } else {
        ResolvedMultiFieldValue::Partial(rendered)
    }
}

fn render_template(
    template: &str,
    placeholders: &[(String, Option<String>)],
    format_lists: &[HierarchyList],
    sticky_values: &HashMap<String, String>,
) -> String {
    let mut result = template.to_string();
    for (id, value) in placeholders {
        let replacement = value.clone().unwrap_or_else(|| format!("{{{id}}}"));
        result = result.replace(&format!("{{{id}}}"), &replacement);
    }
    for list in format_lists {
        let placeholder = format!("{{{}}}", list.id);
        if !result.contains(&placeholder) {
            continue;
        }
        let replacement = resolve_list_value(list, sticky_values)
            .or_else(|| list.preview.clone())
            .unwrap_or_default();
        result = result.replace(&placeholder, &replacement);
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
            let value = item.output().to_string();
            if value.is_empty() {
                return None;
            }
            return Some(value);
        }
    }

    None
}

fn join_repeat_values(values: &[String], style: &JoinerStyle) -> String {
    let values = dedupe_values(values);
    match style {
        JoinerStyle::Comma => values.join(", "),
        JoinerStyle::Semicolon => values.join("; "),
        JoinerStyle::Newline => values.join("\n"),
        JoinerStyle::CommaAnd => join_with_final(&values, ", ", " and ", ", and "),
        JoinerStyle::CommaOr => join_with_final(&values, ", ", " or ", ", or "),
        JoinerStyle::CommaAndThe => {
            let prefixed: Vec<String> = values.iter().map(|value| format!("the {value}")).collect();
            join_with_final(&prefixed, ", ", " and ", ", and ")
        }
        JoinerStyle::Slash => values.join(" / "),
    }
}

fn dedupe_values(values: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for value in values {
        if value.trim().is_empty() {
            continue;
        }
        if !result.iter().any(|existing| existing == value) {
            result.push(value.clone());
        }
    }
    result
}

fn join_with_final(values: &[String], separator: &str, two: &str, final_separator: &str) -> String {
    match values {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first}{two}{second}"),
        _ => {
            let last = values.last().cloned().unwrap_or_default();
            let head = &values[..values.len() - 1];
            format!("{}{final_separator}{last}", head.join(separator))
        }
    }
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
            preview: None,
            fields: Vec::new(),
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
            joiner_style: None,
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

    #[test]
    fn nested_field_with_empty_children_does_not_recurse_forever() {
        let child = HeaderFieldConfig {
            id: "place".to_string(),
            name: "Place".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "place".to_string(),
                label: Some("Place".to_string()),
                preview: None,
                sticky: false,
                default: Some("empty_space".to_string()),
                modal_start: crate::data::ModalStart::Search,
                joiner_style: None,
                max_entries: None,
                items: vec![crate::data::HierarchyItem {
                    id: "empty_space".to_string(),
                    label: Some("Empty".to_string()),
                    default_enabled: true,
                    output: Some(String::new()),
                    fields: None,
                    branch_fields: Vec::new(),
                }],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let cfg = HeaderFieldConfig {
            id: "requested_region".to_string(),
            name: "Requested Region".to_string(),
            format: Some("{place}".to_string()),
            preview: None,
            fields: vec![child],
            lists: Vec::new(),
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let resolved = resolve_multifield_value(
            &crate::sections::header::HeaderFieldValue::Text(String::new()),
            &cfg,
            &HashMap::new(),
        );

        assert!(matches!(resolved, ResolvedMultiFieldValue::Empty));
    }

    #[test]
    fn empty_string_default_does_not_mark_nested_field_complete() {
        let child = HeaderFieldConfig {
            id: "place".to_string(),
            name: "Place".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "place".to_string(),
                label: Some("Place".to_string()),
                preview: None,
                sticky: false,
                default: Some("empty_space".to_string()),
                modal_start: crate::data::ModalStart::Search,
                joiner_style: None,
                max_entries: None,
                items: vec![crate::data::HierarchyItem {
                    id: "empty_space".to_string(),
                    label: Some("Empty".to_string()),
                    default_enabled: true,
                    output: Some(String::new()),
                    fields: None,
                    branch_fields: Vec::new(),
                }],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let region = HeaderFieldConfig {
            id: "region".to_string(),
            name: "Region".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "region".to_string(),
                label: Some("Region".to_string()),
                preview: None,
                sticky: false,
                default: Some("shoulder".to_string()),
                modal_start: crate::data::ModalStart::Search,
                joiner_style: None,
                max_entries: None,
                items: vec![crate::data::HierarchyItem {
                    id: "shoulder".to_string(),
                    label: Some("Shoulder".to_string()),
                    default_enabled: true,
                    output: Some("Shoulder".to_string()),
                    fields: None,
                    branch_fields: Vec::new(),
                }],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let cfg = HeaderFieldConfig {
            id: "single_region".to_string(),
            name: "Requested Region".to_string(),
            format: Some("{place}{region}".to_string()),
            preview: None,
            fields: vec![region, child],
            lists: Vec::new(),
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let resolved = resolve_multifield_value(
            &crate::sections::header::HeaderFieldValue::Text(String::new()),
            &cfg,
            &HashMap::new(),
        );

        assert!(matches!(
            resolved,
            ResolvedMultiFieldValue::Partial(value) if value.contains("Shoulder")
        ));
    }
}
