use crate::data::{HeaderFieldConfig, HierarchyList, JoinerStyle, SectionConfig};
use crate::modal::{
    active_collection_ids, decode_collection_display_value, format_collection_field_value,
    ListValueLookup,
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
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    resolve_field_values(
        std::slice::from_ref(confirmed),
        cfg,
        assigned_values,
        sticky_values,
    )
}

fn merged_slot_assigned_values(
    confirmed: &HeaderFieldValue,
    cfg: &HeaderFieldConfig,
    assigned_values: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut merged = assigned_values.clone();
    merged.extend(crate::modal::confirmed_value_assignments(confirmed, cfg));
    merged
}

pub fn resolve_multifield_value_for_confirmed_slot(
    confirmed: &HeaderFieldValue,
    cfg: &HeaderFieldConfig,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    let merged = merged_slot_assigned_values(confirmed, cfg, assigned_values);
    resolve_multifield_value(confirmed, cfg, &merged, sticky_values)
}

pub fn resolve_field_values(
    confirmed_values: &[HeaderFieldValue],
    cfg: &HeaderFieldConfig,
    assigned_values: &HashMap<String, String>,
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
        return resolve_repeating_values(
            &concrete,
            has_explicit_empty,
            cfg,
            assigned_values,
            sticky_values,
        );
    }

    if let Some(value) = concrete.first() {
        return resolve_single_value(value, cfg, assigned_values, sticky_values);
    }

    if has_explicit_empty {
        ResolvedMultiFieldValue::Empty
    } else {
        resolve_unconfirmed_value(cfg, assigned_values, sticky_values)
    }
}

pub fn render_field_display(
    confirmed_values: &[HeaderFieldValue],
    cfg: &HeaderFieldConfig,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> String {
    match resolve_field_values(confirmed_values, cfg, assigned_values, sticky_values) {
        ResolvedMultiFieldValue::Empty => "[empty]".to_string(),
        ResolvedMultiFieldValue::Partial(value) | ResolvedMultiFieldValue::Complete(value) => value,
    }
}

pub fn render_field_display_for_confirmed_values(
    confirmed_values: &[HeaderFieldValue],
    cfg: &HeaderFieldConfig,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> String {
    if let [confirmed] = confirmed_values {
        match resolve_multifield_value_for_confirmed_slot(
            confirmed,
            cfg,
            assigned_values,
            sticky_values,
        ) {
            ResolvedMultiFieldValue::Empty => "[empty]".to_string(),
            ResolvedMultiFieldValue::Partial(value) | ResolvedMultiFieldValue::Complete(value) => {
                value
            }
        }
    } else {
        render_field_display(confirmed_values, cfg, assigned_values, sticky_values)
    }
}

pub fn renders_without_field_label(section: &SectionConfig, field: &HeaderFieldConfig) -> bool {
    !section.show_field_labels
        || (!field.collections.is_empty() && field.lists.is_empty() && field.format.is_none())
}

pub fn render_note_line(
    section: &SectionConfig,
    field: &HeaderFieldConfig,
    value: &HeaderFieldValue,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> Option<String> {
    let resolved = resolve_multifield_value(value, field, assigned_values, sticky_values);
    let rendered = resolved.export_value()?;
    if renders_without_field_label(section, field) {
        Some(rendered.to_string())
    } else {
        let label = resolve_field_label(value, field, assigned_values, sticky_values);
        Some(format!("{label}: {rendered}"))
    }
}

pub fn render_note_line_for_confirmed_slot(
    section: &SectionConfig,
    field: &HeaderFieldConfig,
    value: &HeaderFieldValue,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> Option<String> {
    let merged = merged_slot_assigned_values(value, field, assigned_values);
    render_note_line(section, field, value, &merged, sticky_values)
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
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> String {
    let confirmed = confirmed.source_value();
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
                    .map(|values| {
                        resolve_field_values(values, child, assigned_values, sticky_values)
                    })
                    .unwrap_or_else(|| {
                        resolve_field_values(&[], child, assigned_values, sticky_values)
                    })
                    .display_value()
                    .map(str::to_string)
                    .or_else(|| child.preview.clone());
                (child.id.clone(), value)
            })
            .collect::<Vec<_>>();
        return render_template(
            &cfg.name,
            &placeholders,
            &cfg.format_lists,
            assigned_values,
            sticky_values,
        );
    }

    let mut placeholders = Vec::new();
    for list in &cfg.lists {
        let value = if cfg.lists.len() == 1 {
            match confirmed {
                HeaderFieldValue::ListState(value) => {
                    { resolve_list_state(value, cfg, assigned_values, sticky_values) }
                        .display_value()
                        .map(str::to_string)
                }
                value if value.as_text().is_some_and(|text| !text.is_empty()) => {
                    value.as_text().map(ToOwned::to_owned)
                }
                _ => resolve_list_value(list, assigned_values, sticky_values),
            }
        } else {
            resolve_list_value(list, assigned_values, sticky_values)
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
                value if value.as_text().is_some_and(|text| !text.is_empty()) => {
                    value.as_text().map(ToOwned::to_owned)
                }
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

    render_template(
        &cfg.name,
        &placeholders,
        &cfg.format_lists,
        assigned_values,
        sticky_values,
    )
}

pub fn resolve_field_label_for_confirmed_slot(
    confirmed: &HeaderFieldValue,
    cfg: &HeaderFieldConfig,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> String {
    let merged = merged_slot_assigned_values(confirmed, cfg, assigned_values);
    resolve_field_label(confirmed, cfg, &merged, sticky_values)
}

fn resolve_repeating_values(
    concrete: &[&HeaderFieldValue],
    has_explicit_empty: bool,
    cfg: &HeaderFieldConfig,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    let mut values = Vec::new();
    let mut saw_partial = false;

    for value in concrete {
        match resolve_single_value(value, cfg, assigned_values, sticky_values) {
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
        return resolve_unconfirmed_value(cfg, assigned_values, sticky_values);
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
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    match confirmed {
        HeaderFieldValue::ExplicitEmpty => ResolvedMultiFieldValue::Empty,
        HeaderFieldValue::ManualOverride { text, .. } => {
            if text.trim().is_empty() {
                ResolvedMultiFieldValue::Empty
            } else {
                ResolvedMultiFieldValue::Complete(text.clone())
            }
        }
        HeaderFieldValue::CollectionState(value)
            if !cfg.collections.is_empty() && cfg.lists.is_empty() =>
        {
            decode_collection_display_value(value, cfg)
                .filter(|value| !value.is_empty())
                .map(ResolvedMultiFieldValue::Complete)
                .unwrap_or(ResolvedMultiFieldValue::Empty)
        }
        value if value.as_text().is_some_and(|text| !text.is_empty()) => {
            ResolvedMultiFieldValue::Complete(value.as_text().unwrap().to_string())
        }
        HeaderFieldValue::ListState(value) if !cfg.lists.is_empty() => {
            resolve_list_state(value, cfg, assigned_values, sticky_values)
        }
        HeaderFieldValue::NestedState(state) if !cfg.fields.is_empty() => {
            resolve_nested_state(state, cfg, assigned_values, sticky_values)
        }
        _ => resolve_unconfirmed_value(cfg, assigned_values, sticky_values),
    }
}

fn resolve_list_state(
    value: &crate::sections::header::ListFieldValue,
    cfg: &HeaderFieldConfig,
    assigned_values: &HashMap<String, String>,
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
                resolve_list_value(list, assigned_values, sticky_values)
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
        let resolved_for_completion = value.values.get(idx).is_some()
            || (value.values.is_empty()
                && value.repeat_values.is_empty()
                && resolve_list_value(list, assigned_values, sticky_values).is_some());
        if !resolved_for_completion {
            all_complete = false;
        }
        placeholders.push((list.id.clone(), display.or_else(|| Some(String::new()))));
    }

    if !any_display {
        return resolve_unconfirmed_value(cfg, assigned_values, sticky_values);
    }

    let template = display_template(cfg, false);
    let rendered = render_template(
        &template,
        &placeholders,
        &cfg.format_lists,
        assigned_values,
        sticky_values,
    );

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
    assigned_values: &HashMap<String, String>,
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
        let resolved = resolve_field_values(values, child, assigned_values, sticky_values);
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
        let rendered = render_template(
            &template,
            &placeholders,
            &cfg.format_lists,
            assigned_values,
            sticky_values,
        );
        return if rendered.is_empty() {
            ResolvedMultiFieldValue::Empty
        } else {
            ResolvedMultiFieldValue::Partial(rendered)
        };
    }

    let template = display_template(cfg, false);
    let rendered = render_template(
        &template,
        &placeholders,
        &cfg.format_lists,
        assigned_values,
        sticky_values,
    );

    if all_complete {
        ResolvedMultiFieldValue::Complete(rendered)
    } else {
        ResolvedMultiFieldValue::Partial(rendered)
    }
}

fn resolve_unconfirmed_value(
    cfg: &HeaderFieldConfig,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    if !cfg.fields.is_empty() {
        return resolve_nested_state(
            &HeaderState::new(cfg.fields.clone()),
            cfg,
            assigned_values,
            sticky_values,
        );
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
        let value = resolve_list_value(list, assigned_values, sticky_values)
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
            assigned_values,
            sticky_values,
        );
        return if rendered.is_empty() {
            ResolvedMultiFieldValue::Empty
        } else {
            ResolvedMultiFieldValue::Partial(rendered)
        };
    }

    let template = display_template(cfg, false);
    let rendered = render_template(
        &template,
        &placeholders,
        &cfg.format_lists,
        assigned_values,
        sticky_values,
    );
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
    assigned_values: &HashMap<String, String>,
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
        let replacement = explicit_or_fallback_list_value(list, assigned_values, sticky_values)
            .or_else(|| list.preview.clone())
            .unwrap_or_default();
        result = result.replace(&placeholder, &replacement);
    }
    result
}

fn explicit_or_fallback_list_value(
    list: &HierarchyList,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> Option<String> {
    if let Some(value) = assigned_values.get(&list.id) {
        return Some(value.clone());
    }
    resolve_list_value(list, assigned_values, sticky_values)
}

fn resolve_list_value(
    list: &HierarchyList,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> Option<String> {
    ListValueLookup::new(assigned_values, sticky_values).fallback_value(list)
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
                    assigns: Vec::new(),
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
                    assigns: Vec::new(),
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
                    assigns: Vec::new(),
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
            &HashMap::new(),
        );

        assert!(matches!(
            resolved,
            ResolvedMultiFieldValue::Partial(value) if value.contains("Shoulder")
        ));
    }

    #[test]
    fn manual_override_uses_override_text_for_output() {
        let cfg = HeaderFieldConfig {
            id: "region".to_string(),
            name: "Region".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: Vec::new(),
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let resolved = resolve_multifield_value(
            &crate::sections::header::HeaderFieldValue::ManualOverride {
                text: "Manual shoulder".to_string(),
                source: Box::new(crate::sections::header::HeaderFieldValue::Text(
                    "Shoulder".to_string(),
                )),
            },
            &cfg,
            &HashMap::new(),
            &HashMap::new(),
        );

        assert!(matches!(
            resolved,
            ResolvedMultiFieldValue::Complete(value) if value == "Manual shoulder"
        ));
    }

    #[test]
    fn empty_manual_override_renders_as_empty() {
        let cfg = HeaderFieldConfig {
            id: "region".to_string(),
            name: "Region".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: Vec::new(),
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let resolved = resolve_multifield_value(
            &crate::sections::header::HeaderFieldValue::ManualOverride {
                text: String::new(),
                source: Box::new(crate::sections::header::HeaderFieldValue::Text(
                    "Shoulder".to_string(),
                )),
            },
            &cfg,
            &HashMap::new(),
            &HashMap::new(),
        );

        assert!(matches!(resolved, ResolvedMultiFieldValue::Empty));
    }

    #[test]
    fn manual_override_keeps_field_label_from_structured_source() {
        let cfg = HeaderFieldConfig {
            id: "requested_region".to_string(),
            name: "Requested {region}".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "region".to_string(),
                label: Some("Region".to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: crate::data::ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items: vec![crate::data::HierarchyItem {
                    id: "shoulder".to_string(),
                    label: Some("Shoulder".to_string()),
                    default_enabled: true,
                    output: Some("Shoulder".to_string()),
                    fields: None,
                    branch_fields: Vec::new(),
                    assigns: Vec::new(),
                }],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let value = crate::sections::header::HeaderFieldValue::ManualOverride {
            text: "Manual shoulder".to_string(),
            source: Box::new(crate::sections::header::HeaderFieldValue::ListState(
                crate::sections::header::ListFieldValue {
                    values: vec!["Shoulder".to_string()],
                    item_ids: vec!["shoulder".to_string()],
                    list_idx: 0,
                    repeat_values: Vec::new(),
                    repeat_item_ids: Vec::new(),
                },
            )),
        };

        let label = resolve_field_label(&value, &cfg, &HashMap::new(), &HashMap::new());

        assert_eq!(label, "Requested Shoulder");
    }

    #[test]
    fn committed_empty_list_part_can_still_resolve_complete_output() {
        let cfg = HeaderFieldConfig {
            id: "single_region".to_string(),
            name: "Requested Region".to_string(),
            format: Some("{place}{region}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "place".to_string(),
                    label: Some("Place".to_string()),
                    preview: None,
                    sticky: false,
                    default: Some("empty_space".to_string()),
                    modal_start: crate::data::ModalStart::Search,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![
                        crate::data::HierarchyItem {
                            id: "empty_space".to_string(),
                            label: Some("Empty".to_string()),
                            default_enabled: true,
                            output: Some(String::new()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                        crate::data::HierarchyItem {
                            id: "left".to_string(),
                            label: Some("Left".to_string()),
                            default_enabled: true,
                            output: Some("Left ".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                    ],
                },
                HierarchyList {
                    id: "region".to_string(),
                    label: Some("Region".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
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
                        assigns: Vec::new(),
                    }],
                },
            ],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let value = crate::sections::header::HeaderFieldValue::ListState(
            crate::sections::header::ListFieldValue {
                values: vec![String::new(), "Shoulder".to_string()],
                item_ids: vec!["empty_space".to_string(), "shoulder".to_string()],
                list_idx: 2,
                repeat_values: Vec::new(),
                repeat_item_ids: Vec::new(),
            },
        );

        let resolved = resolve_multifield_value(&value, &cfg, &HashMap::new(), &HashMap::new());

        assert!(matches!(
            resolved,
            ResolvedMultiFieldValue::Complete(value) if value == "Shoulder"
        ));
    }

    #[test]
    fn assigned_format_lists_render_when_declared_on_field() {
        let cfg = HeaderFieldConfig {
            id: "subjective".to_string(),
            name: "Subjective".to_string(),
            format: Some(
                "{starting_frequency}{duration_unit}{pluralizer}{starting_ago}".to_string(),
            ),
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "starting_frequency".to_string(),
                    label: Some("Frequency".to_string()),
                    preview: None,
                    sticky: false,
                    default: Some("freq_2".to_string()),
                    modal_start: crate::data::ModalStart::Search,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![crate::data::HierarchyItem {
                        id: "freq_2".to_string(),
                        label: Some("2".to_string()),
                        default_enabled: true,
                        output: Some(" 2".to_string()),
                        fields: None,
                        branch_fields: Vec::new(),
                        assigns: vec![crate::data::ItemAssignment {
                            list_id: "pluralizer".to_string(),
                            item_id: "plural".to_string(),
                            output: "s".to_string(),
                        }],
                    }],
                },
                HierarchyList {
                    id: "duration_unit".to_string(),
                    label: Some("Unit".to_string()),
                    preview: None,
                    sticky: false,
                    default: Some("week".to_string()),
                    modal_start: crate::data::ModalStart::Search,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![crate::data::HierarchyItem {
                        id: "week".to_string(),
                        label: Some("week{pluralizer}".to_string()),
                        default_enabled: true,
                        output: Some(" week".to_string()),
                        fields: None,
                        branch_fields: Vec::new(),
                        assigns: Vec::new(),
                    }],
                },
                HierarchyList {
                    id: "starting_since".to_string(),
                    label: Some("Since".to_string()),
                    preview: None,
                    sticky: false,
                    default: Some("starting".to_string()),
                    modal_start: crate::data::ModalStart::Search,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![crate::data::HierarchyItem {
                        id: "starting".to_string(),
                        label: Some("starting".to_string()),
                        default_enabled: true,
                        output: Some(String::new()),
                        fields: None,
                        branch_fields: Vec::new(),
                        assigns: vec![crate::data::ItemAssignment {
                            list_id: "starting_ago".to_string(),
                            item_id: "ago".to_string(),
                            output: " ago".to_string(),
                        }],
                    }],
                },
            ],
            collections: Vec::new(),
            format_lists: vec![
                HierarchyList {
                    id: "pluralizer".to_string(),
                    label: Some("Pluralizer".to_string()),
                    preview: None,
                    sticky: false,
                    default: Some("singular".to_string()),
                    modal_start: crate::data::ModalStart::Search,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![
                        crate::data::HierarchyItem {
                            id: "singular".to_string(),
                            label: Some("singular".to_string()),
                            default_enabled: true,
                            output: Some(String::new()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                        crate::data::HierarchyItem {
                            id: "plural".to_string(),
                            label: Some("plural".to_string()),
                            default_enabled: true,
                            output: Some("s".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                    ],
                },
                HierarchyList {
                    id: "starting_ago".to_string(),
                    label: Some("Ago".to_string()),
                    preview: None,
                    sticky: false,
                    default: Some("empty_space".to_string()),
                    modal_start: crate::data::ModalStart::Search,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![
                        crate::data::HierarchyItem {
                            id: "empty_space".to_string(),
                            label: Some("empty".to_string()),
                            default_enabled: true,
                            output: Some(String::new()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                        crate::data::HierarchyItem {
                            id: "ago".to_string(),
                            label: Some("ago".to_string()),
                            default_enabled: true,
                            output: Some(" ago".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                    ],
                },
            ],
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let value = crate::sections::header::HeaderFieldValue::ListState(
            crate::sections::header::ListFieldValue {
                values: vec![" 2".to_string(), " week".to_string(), String::new()],
                item_ids: vec![
                    "freq_2".to_string(),
                    "week".to_string(),
                    "starting".to_string(),
                ],
                list_idx: 3,
                repeat_values: Vec::new(),
                repeat_item_ids: Vec::new(),
            },
        );

        let resolved = resolve_multifield_value_for_confirmed_slot(
            &value,
            &cfg,
            &HashMap::new(),
            &HashMap::new(),
        );

        assert!(matches!(
            resolved,
            ResolvedMultiFieldValue::Complete(value) if value == " 2 weeks ago"
        ));
    }

    #[test]
    fn empty_assigned_format_lists_clear_placeholder_previews() {
        let cfg = HeaderFieldConfig {
            id: "subjective".to_string(),
            name: "Subjective".to_string(),
            format: Some("{region}{pluralizer}{starting_ago}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "region".to_string(),
                    label: Some("Region".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
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
                        assigns: Vec::new(),
                    }],
                },
                HierarchyList {
                    id: "starting_since".to_string(),
                    label: Some("Since".to_string()),
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
                        assigns: vec![crate::data::ItemAssignment {
                            list_id: "starting_ago".to_string(),
                            item_id: "empty_space".to_string(),
                            output: String::new(),
                        }],
                    }],
                },
                HierarchyList {
                    id: "starting_frequency".to_string(),
                    label: Some("Frequency".to_string()),
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
                        assigns: vec![crate::data::ItemAssignment {
                            list_id: "pluralizer".to_string(),
                            item_id: "singular".to_string(),
                            output: String::new(),
                        }],
                    }],
                },
            ],
            collections: Vec::new(),
            format_lists: vec![
                HierarchyList {
                    id: "pluralizer".to_string(),
                    label: Some("Pluralizer".to_string()),
                    preview: Some(" [plural]".to_string()),
                    sticky: false,
                    default: Some("singular".to_string()),
                    modal_start: crate::data::ModalStart::Search,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![
                        crate::data::HierarchyItem {
                            id: "singular".to_string(),
                            label: Some("singular".to_string()),
                            default_enabled: true,
                            output: Some(String::new()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                        crate::data::HierarchyItem {
                            id: "plural".to_string(),
                            label: Some("plural".to_string()),
                            default_enabled: true,
                            output: Some("s".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                    ],
                },
                HierarchyList {
                    id: "starting_ago".to_string(),
                    label: Some("Ago".to_string()),
                    preview: Some(" [ago]".to_string()),
                    sticky: false,
                    default: Some("empty_space".to_string()),
                    modal_start: crate::data::ModalStart::Search,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![
                        crate::data::HierarchyItem {
                            id: "empty_space".to_string(),
                            label: Some("empty".to_string()),
                            default_enabled: true,
                            output: Some(String::new()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                        crate::data::HierarchyItem {
                            id: "ago".to_string(),
                            label: Some("ago".to_string()),
                            default_enabled: true,
                            output: Some(" ago".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                    ],
                },
            ],
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let value = crate::sections::header::HeaderFieldValue::ListState(
            crate::sections::header::ListFieldValue {
                values: vec!["Shoulder".to_string(), String::new(), String::new()],
                item_ids: vec![
                    "shoulder".to_string(),
                    "empty_space".to_string(),
                    "empty_space".to_string(),
                ],
                list_idx: 3,
                repeat_values: Vec::new(),
                repeat_item_ids: Vec::new(),
            },
        );

        let resolved = resolve_multifield_value_for_confirmed_slot(
            &value,
            &cfg,
            &HashMap::new(),
            &HashMap::new(),
        );

        assert!(matches!(
            resolved,
            ResolvedMultiFieldValue::Complete(value) if value == "Shoulder"
        ));
    }
}
