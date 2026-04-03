use std::collections::HashMap;
use crate::data::{CompositeConfig, CompositePart, HeaderFieldConfig};

/// The resolved value for a single multi_field field.
#[derive(Debug, Clone)]
pub enum ResolvedMultiFieldValue {
    /// No part or field resolved to any value.
    Empty,
    /// Some but not all composite parts resolved.
    Partial,
    /// All parts (or the whole simple field) resolved cleanly.
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

    /// For note preview pane: Complete shows value, everything else shows "--".
    pub fn preview_str(&self) -> &str {
        match self {
            Self::Complete(s) => s.as_str(),
            _ => "--",
        }
    }

    /// For clipboard export: only Complete values are usable.
    pub fn export_value(&self) -> Option<&str> {
        match self {
            Self::Complete(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

fn resolve_part_default(part: &CompositePart) -> Option<String> {
    part.default.as_ref()?;
    let cursor = part.default_cursor();
    part.options.get(cursor).map(|o| o.output().to_string())
}

fn resolve_composite_part_value(
    field_id: &str,
    part: &CompositePart,
    sticky_values: &HashMap<String, String>,
) -> Option<String> {
    if part.sticky {
        let key = format!("{}.{}", field_id, part.id);
        if let Some(v) = sticky_values.get(&key) {
            if !v.is_empty() {
                return Some(v.clone());
            }
        }
    }
    resolve_part_default(part)
}

fn resolve_composite_multifield_value(
    field_id: &str,
    composite: &CompositeConfig,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    if composite.parts.is_empty() {
        return ResolvedMultiFieldValue::Empty;
    }
    let mut result = composite.format.clone();
    let mut resolved_count = 0;
    for part in &composite.parts {
        match resolve_composite_part_value(field_id, part, sticky_values) {
            Some(val) => {
                resolved_count += 1;
                result = result.replace(&format!("{{{}}}", part.id), &val);
            }
            None => {
                result = result.replace(&format!("{{{}}}", part.id), "");
            }
        }
    }
    if resolved_count == composite.parts.len() {
        ResolvedMultiFieldValue::Complete(result)
    } else if resolved_count > 0 {
        ResolvedMultiFieldValue::Partial
    } else {
        ResolvedMultiFieldValue::Empty
    }
}

/// Resolve one header field using: confirmed > sticky > default > empty.
///
/// - Composite fields are resolved part-by-part from sticky then part default.
/// - Simple fields use the field-level sticky key then field default.
/// - Returns Complete only when the value is fully and cleanly resolved.
/// - Returns Partial when some composite parts resolved but not all.
/// - Returns Empty when nothing resolved.
pub fn resolve_multifield_value(
    confirmed: &str,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> ResolvedMultiFieldValue {
    if !confirmed.is_empty() {
        return ResolvedMultiFieldValue::Complete(confirmed.to_string());
    }
    if let Some(ref composite) = cfg.composite {
        resolve_composite_multifield_value(&cfg.id, composite, sticky_values)
    } else {
        if let Some(v) = sticky_values.get(&cfg.id) {
            if !v.is_empty() {
                return ResolvedMultiFieldValue::Complete(v.clone());
            }
        }
        if let Some(ref default) = cfg.default {
            if !default.is_empty() {
                return ResolvedMultiFieldValue::Complete(default.clone());
            }
        }
        ResolvedMultiFieldValue::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{CompositeConfig, CompositePart, HeaderFieldConfig, PartOption};

    fn simple_field(id: &str, default: Option<&str>) -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: id.to_string(),
            name: id.to_string(),
            options: vec![],
            composite: None,
            default: default.map(|s| s.to_string()),
            repeat_limit: None,
        }
    }

    fn composite_field_date() -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: "date".to_string(),
            name: "Date".to_string(),
            options: vec![],
            default: None,
            composite: Some(CompositeConfig {
                format: "{year}-{month}-{day}".to_string(),
                parts: vec![
                    CompositePart {
                        id: "year".to_string(),
                        label: "Year".to_string(),
                        preview: Some("YYYY".to_string()),
                        options: vec![
                            PartOption::Simple("2025".to_string()),
                            PartOption::Simple("2026".to_string()),
                        ],
                        data_file: None,
                        sticky: true,
                        default: None,
                    },
                    CompositePart {
                        id: "month".to_string(),
                        label: "Month".to_string(),
                        preview: Some("MM".to_string()),
                        options: vec![PartOption::Simple("04".to_string())],
                        data_file: None,
                        sticky: true,
                        default: None,
                    },
                    CompositePart {
                        id: "day".to_string(),
                        label: "Day".to_string(),
                        preview: Some("DD".to_string()),
                        options: vec![PartOption::Simple("02".to_string())],
                        data_file: None,
                        sticky: true,
                        default: None,
                    },
                ],
            }),
            repeat_limit: None,
        }
    }

    #[test]
    fn confirmed_beats_sticky_and_default() {
        let cfg = simple_field("dur", Some("60"));
        let mut sticky = HashMap::new();
        sticky.insert("dur".to_string(), "45".to_string());
        let result = resolve_multifield_value("90", &cfg, &sticky);
        assert!(result.is_complete());
        assert_eq!(result.export_value(), Some("90"));
    }

    #[test]
    fn sticky_beats_default_for_simple_field() {
        let cfg = simple_field("dur", Some("60"));
        let mut sticky = HashMap::new();
        sticky.insert("dur".to_string(), "45".to_string());
        let result = resolve_multifield_value("", &cfg, &sticky);
        assert!(result.is_complete());
        assert_eq!(result.export_value(), Some("45"));
    }

    #[test]
    fn default_used_when_no_sticky() {
        let cfg = simple_field("dur", Some("60"));
        let sticky = HashMap::new();
        let result = resolve_multifield_value("", &cfg, &sticky);
        assert!(result.is_complete());
        assert_eq!(result.export_value(), Some("60"));
    }

    #[test]
    fn empty_when_no_confirmed_sticky_or_default() {
        let cfg = simple_field("dur", None);
        let sticky = HashMap::new();
        let result = resolve_multifield_value("", &cfg, &sticky);
        assert!(result.is_empty_variant());
        assert_eq!(result.export_value(), None);
        assert_eq!(result.preview_str(), "--");
    }

    #[test]
    fn composite_all_sticky_parts_resolve_to_complete() {
        let cfg = composite_field_date();
        let mut sticky = HashMap::new();
        sticky.insert("date.year".to_string(), "2026".to_string());
        sticky.insert("date.month".to_string(), "04".to_string());
        sticky.insert("date.day".to_string(), "02".to_string());
        let result = resolve_multifield_value("", &cfg, &sticky);
        assert!(result.is_complete());
        assert_eq!(result.export_value(), Some("2026-04-02"));
    }

    #[test]
    fn composite_partial_sticky_gives_partial() {
        let cfg = composite_field_date();
        let mut sticky = HashMap::new();
        sticky.insert("date.year".to_string(), "2026".to_string());
        // month and day missing
        let result = resolve_multifield_value("", &cfg, &sticky);
        assert!(matches!(result, ResolvedMultiFieldValue::Partial));
        assert_eq!(result.export_value(), None);
        assert_eq!(result.preview_str(), "--");
    }

    #[test]
    fn composite_no_sticky_gives_empty() {
        let cfg = composite_field_date();
        let sticky = HashMap::new();
        let result = resolve_multifield_value("", &cfg, &sticky);
        assert!(result.is_empty_variant());
    }

    #[test]
    fn confirmed_beats_composite_sticky() {
        let cfg = composite_field_date();
        let mut sticky = HashMap::new();
        sticky.insert("date.year".to_string(), "2026".to_string());
        sticky.insert("date.month".to_string(), "04".to_string());
        sticky.insert("date.day".to_string(), "02".to_string());
        let result = resolve_multifield_value("2025-01-01", &cfg, &sticky);
        assert!(result.is_complete());
        assert_eq!(result.export_value(), Some("2025-01-01"));
    }
}
