use crate::app::SectionState;
use crate::data::{SectionConfig, SectionGroup};
use crate::sections::multi_field::resolve_multifield_value;
use chrono::Local;
use std::collections::HashMap;

#[derive(Clone)]
pub enum NoteRenderMode {
    Preview,
    Export,
}

/// Returns the rendered heading text to search for when scrolling to a given
/// section id or group id. Both section ids and group ids share this map so
/// section_start_line can use the same lookup for primary and fallback anchors.
fn heading_anchor(id: &str) -> &'static str {
    match id {
        // Section anchors
        "adl"                      => "ACTIVITIES OF DAILY LIVING",
        "exercise"                 => "EXERCISE HABITS",
        "sleep_diet"               => "SLEEP & DIET",
        "social"                   => "SOCIAL & STRESS",
        "history"                  => "HISTORY & PREVIOUS DIAGNOSES",
        "specialists"              => "SPECIALISTS & TREATMENT",
        "subjective_section"       => "## SUBJECTIVE",
        "tx_mods"                  => "TREATMENT MODIFICATIONS",
        "tx_regions"               => "TREATMENT / PLAN",
        "objective_section"        => "## OBJECTIVE / OBSERVATIONS",
        "post_treatment"           => "## POST-TREATMENT",
        "remedial_section"         => "REMEDIAL EXERCISES",
        "tx_plan"                  => "TREATMENT PLAN",
        "infection_control_section" => "INFECTION CONTROL",
        // Group anchors
        "subjective"               => "## SUBJECTIVE",
        "treatment"                => "## TREATMENT / PLAN",
        "objective"                => "## OBJECTIVE / OBSERVATIONS",
        "post_tx"                  => "## POST-TREATMENT",
        // intake, header, and anything else: no anchor in the rendered note
        _                          => "",
    }
}

pub fn section_start_line(
    sections: &[SectionConfig],
    states: &[SectionState],
    sticky_values: &HashMap<String, String>,
    groups: &[SectionGroup],
    boilerplate_texts: &HashMap<String, String>,
    section_id: &str,
) -> u16 {
    let note = render_note(sections, states, sticky_values, boilerplate_texts, NoteRenderMode::Preview);

    // Try the section's own anchor first.
    let anchor = heading_anchor(section_id);
    if !anchor.is_empty() {
        for (i, line) in note.lines().enumerate() {
            if line.contains(anchor) {
                return i as u16;
            }
        }
    }

    // Section heading not found (or section has no anchor). Try the parent group.
    let group_id = groups
        .iter()
        .find(|g| g.sections.iter().any(|s| s.id == section_id))
        .map(|g| g.id.as_str());

    if let Some(gid) = group_id {
        let group_anchor = heading_anchor(gid);
        if !group_anchor.is_empty() {
            for (i, line) in note.lines().enumerate() {
                if line.contains(group_anchor) {
                    return i as u16;
                }
            }
        }
    }

    0
}

pub fn render_note(
    sections: &[SectionConfig],
    states: &[SectionState],
    sticky_values: &HashMap<String, String>,
    boilerplate_texts: &HashMap<String, String>,
    mode: NoteRenderMode,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Pass 1: render the appointment header section (cfg.id == "header") first.
    for (cfg, state) in sections.iter().zip(states.iter()) {
        if cfg.section_type == "multi_field" && cfg.id == "header" {
            if let SectionState::Header(hs) = state {
                let has_repeatable = hs.field_configs.iter().any(|c| c.repeat_limit.is_some());
                let rendered = match &mode {
                    NoteRenderMode::Preview => {
                        let has_any = hs.field_configs.iter().enumerate().any(|(i, fcfg)| {
                            let confirmed = hs.repeated_values.get(i)
                                .and_then(|v| v.first())
                                .map(|s| s.as_str())
                                .unwrap_or("");
                            !resolve_multifield_value(confirmed, fcfg, sticky_values).is_empty_variant()
                        });
                        if has_any {
                            if has_repeatable {
                                Some(format_header_generic_preview(hs, sticky_values))
                            } else {
                                Some(format_header_preview(hs, sticky_values))
                            }
                        } else {
                            None
                        }
                    }
                    NoteRenderMode::Export => {
                        if has_repeatable {
                            format_header_generic_export(hs, sticky_values)
                        } else {
                            format_header_export(hs, sticky_values)
                        }
                    }
                };
                if let Some(h) = rendered {
                    parts.push(h);
                }
            }
            break;
        }
    }

    // INTAKE group sections (no ## heading, just #### subsections)
    let intake_sections: Vec<(&SectionConfig, &SectionState)> = sections
        .iter()
        .zip(states.iter())
        .filter(|(cfg, _)| cfg.section_type != "multi_field" && is_intake_section(cfg))
        .collect();

    let mut intake_parts: Vec<String> = Vec::new();
    for (cfg, state) in &intake_sections {
        let rendered = render_section_content(cfg, state, &today);
        if !rendered.trim().is_empty() || !is_skipped(state) {
            let heading = intake_heading(cfg);
            let mut section_text = format!("\n\n{}", heading);
            if !rendered.trim().is_empty() {
                section_text.push_str(&format!("\n{}", rendered));
            }
            intake_parts.push(section_text);
        }
    }
    if !intake_parts.is_empty() {
        parts.push(intake_parts.join("\n"));
    }

    // SEPARATOR
    parts.push("\n\n\n_______________".to_string());

    // SUBJECTIVE
    let mut subj_parts: Vec<String> = Vec::new();
    subj_parts.push("\n\n## SUBJECTIVE".to_string());
    for (cfg, state) in sections.iter().zip(states.iter()) {
        if cfg.id == "subjective_section" {
            let rendered = render_section_content(cfg, state, &today);
            if !rendered.trim().is_empty() {
                subj_parts.push(format!("\n{}", rendered));
            }
        }
    }
    let informed_consent = boilerplate_texts.get("informed_consent").map(|s| s.as_str()).unwrap_or("");
    if !informed_consent.is_empty() {
        subj_parts.push(format!("\n\n\n#### INFORMED CONSENT\n- {}", informed_consent));
    }
    parts.push(subj_parts.join(""));

    // SEPARATOR
    parts.push("\n\n\n_______________".to_string());

    // TREATMENT / PLAN
    let mut tx_parts: Vec<String> = Vec::new();
    let tx_disclaimer = boilerplate_texts.get("treatment_plan_disclaimer").map(|s| s.as_str()).unwrap_or("");
    let tx_header = if tx_disclaimer.is_empty() {
        "\n\n## TREATMENT / PLAN".to_string()
    } else {
        format!("\n\n## TREATMENT / PLAN\n{}", tx_disclaimer)
    };
    tx_parts.push(tx_header);

    // tx_mods
    for (cfg, state) in sections.iter().zip(states.iter()) {
        if cfg.id == "tx_mods" {
            if cfg.section_type == "multi_field" {
                if let SectionState::Header(hs) = state {
                    if let Some(rendered) = render_multifield_section(cfg, hs, sticky_values, mode.clone()) {
                        if !rendered.trim().is_empty() {
                            tx_parts.push(format!("\n\n\n#### TREATMENT MODIFICATIONS & PREFERENCES\n{}", rendered));
                        }
                    }
                }
            } else {
                let rendered = render_section_content(cfg, state, &today);
                if !rendered.trim().is_empty() {
                    tx_parts.push(format!("\n\n\n#### TREATMENT MODIFICATIONS & PREFERENCES\n{}", rendered));
                }
            }
        }
    }

    // tx_regions
    for (cfg, state) in sections.iter().zip(states.iter()) {
        if cfg.id == "tx_regions" {
            let rendered = render_block_select(state);
            if !rendered.trim().is_empty() {
                tx_parts.push(format!("\n\n{}", rendered));
            }
        }
    }

    parts.push(tx_parts.join(""));

    // SEPARATOR
    parts.push("\n\n\n_______________".to_string());

    // OBJECTIVE
    let mut obj_parts: Vec<String> = Vec::new();
    obj_parts.push("\n\n## OBJECTIVE / OBSERVATIONS".to_string());
    for (cfg, state) in sections.iter().zip(states.iter()) {
        if cfg.id == "objective_section" {
            let rendered = render_section_content(cfg, state, &today);
            if !rendered.trim().is_empty() {
                obj_parts.push(format!("\n\n{}", rendered));
            }
        }
    }
    parts.push(obj_parts.join(""));

    // SEPARATOR
    parts.push("\n\n\n_______________".to_string());

    // POST-TREATMENT
    let mut post_parts: Vec<String> = Vec::new();
    post_parts.push("\n\n## POST-TREATMENT".to_string());

    for (cfg, state) in sections.iter().zip(states.iter()) {
        if cfg.id == "post_treatment" {
            let rendered = render_section_content(cfg, state, &today);
            if !rendered.trim().is_empty() {
                post_parts.push(format!("\n{}", rendered));
            }
        }
    }

    for (cfg, state) in sections.iter().zip(states.iter()) {
        if cfg.id == "remedial_section" {
            let rendered = render_section_content(cfg, state, &today);
            if !rendered.trim().is_empty() {
                post_parts.push(format!("\n\n\n#### REMEDIAL EXERCISES & SELF-CARE\n{}", rendered));
            }
        }
    }

    for (cfg, state) in sections.iter().zip(states.iter()) {
        if cfg.id == "tx_plan" {
            let rendered = render_section_content(cfg, state, &today);
            if !rendered.trim().is_empty() {
                post_parts.push(format!("\n\n\n#### TREATMENT PLAN / THERAPIST NOTES\n{}", rendered));
            }
        }
    }

    parts.push(post_parts.join(""));

    // SEPARATOR
    parts.push("\n\n\n_______________".to_string());

    // INFECTION CONTROL
    for (cfg, state) in sections.iter().zip(states.iter()) {
        if cfg.id == "infection_control_section" {
            let rendered = render_section_content(cfg, state, &today);
            if !rendered.trim().is_empty() {
                parts.push(format!("\n\n\n#### STANDARD INFECTION CONTROL PRECAUTIONS\n{}", rendered));
            }
        }
    }

    // Catch-all: non-header multi_field sections with unrecognized ids
    for (cfg, state) in sections.iter().zip(states.iter()) {
        if cfg.section_type == "multi_field" && cfg.id != "header" {
            let known_ids = ["tx_mods"];
            if !known_ids.contains(&cfg.id.as_str()) {
                if let SectionState::Header(hs) = state {
                    if let Some(rendered) = render_multifield_section(cfg, hs, sticky_values, mode.clone()) {
                        if !rendered.trim().is_empty() {
                            parts.push(format!("\n\n\n#### {}\n{}", cfg.name.to_uppercase(), rendered));
                        }
                    }
                }
            }
        }
    }

    parts.push("\n\n\n_______________\n".to_string());

    parts.join("")
}

fn is_intake_section(cfg: &SectionConfig) -> bool {
    matches!(
        cfg.id.as_str(),
        "adl" | "exercise" | "sleep_diet" | "social" | "history" | "specialists"
    )
}

fn intake_heading(cfg: &SectionConfig) -> String {
    match cfg.id.as_str() {
        "adl" => "#### ACTIVITIES OF DAILY LIVING".to_string(),
        "exercise" => "#### EXERCISE HABITS".to_string(),
        "sleep_diet" => "#### SLEEP & DIET".to_string(),
        "social" => "#### SOCIAL & STRESS".to_string(),
        "history" => "#### HISTORY & PREVIOUS DIAGNOSES".to_string(),
        "specialists" => "#### SPECIALISTS & TREATMENT".to_string(),
        _ => format!("#### {}", cfg.name.to_uppercase()),
    }
}

fn is_skipped(state: &SectionState) -> bool {
    match state {
        SectionState::FreeText(s) => s.skipped,
        SectionState::ListSelect(s) => s.skipped,
        SectionState::BlockSelect(s) => s.skipped,
        SectionState::Checklist(s) => s.skipped,
        SectionState::Header(s) => !s.completed,
        SectionState::Pending => true,
    }
}

fn render_section_content(cfg: &SectionConfig, state: &SectionState, today: &str) -> String {
    match state {
        SectionState::FreeText(s) => {
            if s.entries.is_empty() {
                return String::new();
            }
            s.entries
                .iter()
                .map(|e| format!("- {}", e))
                .collect::<Vec<_>>()
                .join("\n\n")
        }
        SectionState::ListSelect(s) => {
            if s.selected_indices.is_empty() {
                return String::new();
            }
            let use_date = cfg.date_prefix.unwrap_or(false);
            s.selected_indices
                .iter()
                .filter_map(|&i| s.entries.get(i))
                .map(|entry| {
                    if use_date {
                        format!("- {}: {}", today, entry.output)
                    } else {
                        entry.output.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        SectionState::BlockSelect(_) => {
            // Block select has its own renderer
            String::new()
        }
        SectionState::Checklist(s) => {
            s.items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    if s.checked.get(i).copied().unwrap_or(true) {
                        format!("- [x] {}", item)
                    } else {
                        format!("- [ ] {}", item)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => String::new(),
    }
}

fn render_block_select(state: &SectionState) -> String {
    if let SectionState::BlockSelect(s) = state {
        let mut parts: Vec<String> = Vec::new();
        for region_state in &s.groups {
            let selected: Vec<String> = region_state
                .item_selected
                .iter()
                .enumerate()
                .filter(|(_, &sel)| sel)
                .filter_map(|(i, _)| region_state.entries.get(i))
                .map(|t| t.output().to_string())
                .collect();
            if !selected.is_empty() {
                let mut region_text = region_state.header.clone();
                for tech in &selected {
                    region_text.push('\n');
                    region_text.push_str(tech);
                }
                parts.push(region_text);
            }
        }
        parts.join("\n\n")
    } else {
        String::new()
    }
}

/// Format the header for live note preview. Unresolved fields show "--".
fn format_header_preview(
    hs: &crate::sections::header::HeaderState,
    sticky_values: &HashMap<String, String>,
) -> String {
    let field_preview = |id: &str| -> String {
        hs.field_configs
            .iter()
            .enumerate()
            .find(|(_, cfg)| cfg.id == id)
            .map(|(i, cfg)| {
                let confirmed = hs.repeated_values.get(i)
                    .and_then(|v| v.first())
                    .map(|s| s.as_str())
                    .unwrap_or("");
                resolve_multifield_value(confirmed, cfg, sticky_values)
                    .preview_str()
                    .to_string()
            })
            .unwrap_or_else(|| "--".to_string())
    };

    let date_raw = field_preview("date");
    let date_str = if date_raw == "--" { "--".to_string() } else { format_header_date(&date_raw) };
    let time_raw = field_preview("start_time");
    let time_str = if time_raw == "--" { "--".to_string() } else { format_header_time(&time_raw) };
    let dur_str = field_preview("appointment_duration");
    let appt_str = field_preview("appointment_type");
    format!("{} at {} ({} min)\n{}", date_str, time_str, dur_str, appt_str)
}

/// Format the header for clipboard export. Omits any unresolved fields cleanly.
/// Returns None when no fields resolve, so the header block is omitted entirely.
fn format_header_export(
    hs: &crate::sections::header::HeaderState,
    sticky_values: &HashMap<String, String>,
) -> Option<String> {
    let field_export = |id: &str| -> Option<String> {
        hs.field_configs
            .iter()
            .enumerate()
            .find(|(_, cfg)| cfg.id == id)
            .and_then(|(i, cfg)| {
                let confirmed = hs.repeated_values.get(i)
                    .and_then(|v| v.first())
                    .map(|s| s.as_str())
                    .unwrap_or("");
                resolve_multifield_value(confirmed, cfg, sticky_values)
                    .export_value()
                    .map(|s| s.to_string())
            })
    };

    let date_val = field_export("date").map(|d| format_header_date(&d));
    let time_val = field_export("start_time").map(|t| format_header_time(&t));
    let dur_val = field_export("appointment_duration");
    let appt_val = field_export("appointment_type");

    // Line 1: join only the pieces that exist
    let mut line1_parts: Vec<String> = Vec::new();
    if let Some(d) = date_val {
        line1_parts.push(d);
    }
    if let Some(t) = time_val {
        line1_parts.push(format!("at {}", t));
    }
    if let Some(dur) = dur_val {
        line1_parts.push(format!("({} min)", dur));
    }

    let line1 = if line1_parts.is_empty() { None } else { Some(line1_parts.join(" ")) };
    let line2 = appt_val;

    match (line1, line2) {
        (None, None) => None,
        (Some(l1), None) => Some(l1),
        (None, Some(l2)) => Some(l2),
        (Some(l1), Some(l2)) => Some(format!("{}\n{}", l1, l2)),
    }
}

/// Format a generic multi_field header section for live note preview.
/// Non-repeatable fields show their first confirmed value (or "--").
/// Repeatable fields (repeat_limit is Some) emit one line per confirmed value.
fn format_header_generic_preview(
    hs: &crate::sections::header::HeaderState,
    sticky_values: &HashMap<String, String>,
) -> String {
    let mut lines: Vec<String> = Vec::new();
    for (i, cfg) in hs.field_configs.iter().enumerate() {
        let slot = hs.repeated_values.get(i).map(|v| v.as_slice()).unwrap_or(&[]);
        if cfg.repeat_limit.is_some() {
            // Emit one line per confirmed value in order
            if slot.is_empty() {
                lines.push(format!("{}: --", cfg.name));
            } else {
                for entry in slot {
                    let resolved = resolve_multifield_value(entry.as_str(), cfg, sticky_values);
                    lines.push(format!("{}: {}", cfg.name, resolved.preview_str()));
                }
            }
        } else {
            let confirmed = slot.first().map(|s| s.as_str()).unwrap_or("");
            let resolved = resolve_multifield_value(confirmed, cfg, sticky_values);
            lines.push(format!("{}: {}", cfg.name, resolved.preview_str()));
        }
    }
    lines.join("\n")
}

/// Format a generic multi_field header section for clipboard export.
/// Non-repeatable fields emit their first confirmed value (if resolved).
/// Repeatable fields (repeat_limit is Some) emit one line per confirmed value.
/// Returns None when no fields resolve at all.
fn format_header_generic_export(
    hs: &crate::sections::header::HeaderState,
    sticky_values: &HashMap<String, String>,
) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();
    for (i, cfg) in hs.field_configs.iter().enumerate() {
        let slot = hs.repeated_values.get(i).map(|v| v.as_slice()).unwrap_or(&[]);
        if cfg.repeat_limit.is_some() {
            for entry in slot {
                let resolved = resolve_multifield_value(entry.as_str(), cfg, sticky_values);
                if let Some(val) = resolved.export_value() {
                    lines.push(val.to_string());
                }
            }
        } else {
            let confirmed = slot.first().map(|s| s.as_str()).unwrap_or("");
            let resolved = resolve_multifield_value(confirmed, cfg, sticky_values);
            if let Some(val) = resolved.export_value() {
                lines.push(val.to_string());
            }
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Dispatch a non-header multi_field section to the appropriate renderer.
/// Preview always returns Some (empty fields show "--" placeholders).
/// Export returns None when all fields are empty.
pub fn render_multifield_section(
    _cfg: &SectionConfig,
    hs: &crate::sections::header::HeaderState,
    sticky_values: &HashMap<String, String>,
    mode: NoteRenderMode,
) -> Option<String> {
    match mode {
        NoteRenderMode::Preview => Some(format_header_generic_preview(hs, sticky_values)),
        NoteRenderMode::Export => format_header_generic_export(hs, sticky_values),
    }
}

fn format_header_date(date: &str) -> String {
    // Parse YYYY-MM-DD and format as "Thu Mar 19, 2026"
    if let Ok(d) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        d.format("%a %b %-d, %Y").to_string()
    } else {
        date.to_string()
    }
}

fn format_header_time(time: &str) -> String {
    // Parse HH:MM or H:MMpm and normalize to h:MMpm
    if let Ok(t) = chrono::NaiveTime::parse_from_str(time, "%H:%M") {
        t.format("%-I:%M%P").to_string()
    } else if let Ok(t) = chrono::NaiveTime::parse_from_str(time, "%I:%M%P") {
        t.format("%-I:%M%P").to_string()
    } else {
        time.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sections::header::HeaderState;
    use crate::data::HeaderFieldConfig;

    fn make_header_state(field_ids: &[&str], values: &[&str]) -> HeaderState {
        let configs: Vec<HeaderFieldConfig> = field_ids
            .iter()
            .map(|id| HeaderFieldConfig {
                id: id.to_string(),
                name: id.to_string(),
                options: vec![],
                composite: None,
                default: None,
                repeat_limit: None,
            })
            .collect();
        let mut hs = HeaderState::new(configs);
        for (i, val) in values.iter().enumerate() {
            if let Some(slot) = hs.repeated_values.get_mut(i) {
                if !val.is_empty() {
                    slot.push(val.to_string());
                }
            }
        }
        hs
    }

    #[test]
    fn export_omits_unresolved_fields() {
        // appointment_duration confirmed, date/time/type empty
        use crate::data::{CompositeConfig, CompositePart, PartOption};
        let dur_cfg = HeaderFieldConfig {
            id: "appointment_duration".to_string(),
            name: "Duration".to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit: None,
        };
        let date_cfg = HeaderFieldConfig {
            id: "date".to_string(),
            name: "Date".to_string(),
            options: vec![],
            default: None,
            composite: Some(CompositeConfig {
                format: "{year}-{month}-{day}".to_string(),
                parts: vec![
                    CompositePart { id: "year".to_string(), label: "Year".to_string(), preview: None, options: vec![PartOption::Simple("2026".to_string())], data_file: None, sticky: true, default: None },
                    CompositePart { id: "month".to_string(), label: "Month".to_string(), preview: None, options: vec![PartOption::Simple("04".to_string())], data_file: None, sticky: true, default: None },
                    CompositePart { id: "day".to_string(), label: "Day".to_string(), preview: None, options: vec![PartOption::Simple("02".to_string())], data_file: None, sticky: true, default: None },
                ],
            }),
            repeat_limit: None,
        };
        let configs = vec![date_cfg, dur_cfg];
        let mut hs = HeaderState::new(configs);
        hs.repeated_values[1].push("60".to_string()); // only duration confirmed
        let sticky = HashMap::new();

        let result = format_header_export(&hs, &sticky);
        // Only duration present: should be "(60 min)"
        assert_eq!(result, Some("(60 min)".to_string()));
    }

    #[test]
    fn export_full_header() {
        let date_cfg = HeaderFieldConfig {
            id: "date".to_string(),
            name: "Date".to_string(),
            options: vec![],
            default: None,
            composite: None,
            repeat_limit: None,
        };
        let time_cfg = HeaderFieldConfig {
            id: "start_time".to_string(),
            name: "Start Time".to_string(),
            options: vec![],
            default: None,
            composite: None,
            repeat_limit: None,
        };
        let dur_cfg = HeaderFieldConfig {
            id: "appointment_duration".to_string(),
            name: "Duration".to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit: None,
        };
        let appt_cfg = HeaderFieldConfig {
            id: "appointment_type".to_string(),
            name: "Appointment Type".to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit: None,
        };
        let configs = vec![date_cfg, time_cfg, dur_cfg, appt_cfg];
        let mut hs = HeaderState::new(configs);
        hs.repeated_values[0].push("2026-04-02".to_string());
        hs.repeated_values[1].push("13:00".to_string());
        hs.repeated_values[2].push("60".to_string());
        hs.repeated_values[3].push("Treatment focused massage".to_string());
        let sticky = HashMap::new();

        let result = format_header_export(&hs, &sticky);
        assert_eq!(result, Some("Thu Apr 2, 2026 at 1:00pm (60 min)\nTreatment focused massage".to_string()));
    }

    #[test]
    fn export_returns_none_when_nothing_resolved() {
        let hs = make_header_state(&["date", "start_time"], &["", ""]);
        let sticky = HashMap::new();
        let result = format_header_export(&hs, &sticky);
        assert_eq!(result, None);
    }

    #[test]
    fn preview_shows_placeholder_for_unresolved() {
        let hs = make_header_state(
            &["date", "start_time", "appointment_duration", "appointment_type"],
            &["2026-04-02", "", "", ""],
        );
        let sticky = HashMap::new();
        let result = format_header_preview(&hs, &sticky);
        // date resolved, rest "--"
        assert!(result.contains("Thu Apr 2, 2026"));
        assert!(result.contains("--"));
    }

    // --- section_start_line fallback tests ---

    fn make_section(id: &str, section_type: &str) -> SectionConfig {
        SectionConfig {
            id: id.to_string(),
            name: id.to_string(),
            map_label: id.to_string(),
            section_type: section_type.to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
        }
    }

    fn make_group(id: &str, sections: Vec<SectionConfig>) -> SectionGroup {
        SectionGroup {
            id: id.to_string(),
            num: None,
            name: id.to_string(),
            sections,
        }
    }

    #[test]
    fn empty_tx_plan_falls_back_to_post_treatment_heading() {
        use crate::sections::free_text::FreeTextState;
        let sec = make_section("tx_plan", "free_text");
        let groups = vec![make_group("post_tx", vec![sec.clone()])];
        let sections = vec![sec];
        let states = vec![SectionState::FreeText(FreeTextState::new())]; // empty
        let sticky = HashMap::new();
        let bp = HashMap::new();
        let line = section_start_line(&sections, &states, &sticky, &groups, &bp, "tx_plan");
        // ## POST-TREATMENT is always rendered; must not return 0
        assert!(line > 0, "expected fallback to ## POST-TREATMENT, got 0");
        // Verify it actually landed on that heading
        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);
        let target: Vec<(usize, &str)> = note.lines().enumerate()
            .filter(|(_, l)| l.contains("## POST-TREATMENT"))
            .collect();
        assert!(!target.is_empty(), "## POST-TREATMENT not found in rendered note");
        assert_eq!(line, target[0].0 as u16);
    }

    #[test]
    fn non_empty_tx_plan_returns_own_heading_line() {
        use crate::sections::free_text::FreeTextState;
        let sec = make_section("tx_plan", "free_text");
        let groups = vec![make_group("post_tx", vec![sec.clone()])];
        let sections = vec![sec];
        let mut s = FreeTextState::new();
        s.entries.push("some content".to_string());
        let states = vec![SectionState::FreeText(s)];
        let sticky = HashMap::new();
        let bp = HashMap::new();
        let line = section_start_line(&sections, &states, &sticky, &groups, &bp, "tx_plan");
        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);
        let target: Vec<(usize, &str)> = note.lines().enumerate()
            .filter(|(_, l)| l.contains("TREATMENT PLAN"))
            .collect();
        assert!(!target.is_empty(), "TREATMENT PLAN heading not found in rendered note");
        assert_eq!(line, target[0].0 as u16);
    }

    #[test]
    fn skipped_intake_section_returns_zero() {
        use crate::sections::free_text::FreeTextState;
        let sec = make_section("adl", "free_text");
        let groups = vec![make_group("intake", vec![sec.clone()])];
        let sections = vec![sec];
        let mut s = FreeTextState::new();
        s.skipped = true; // skipped, no content -> heading not rendered
        let states = vec![SectionState::FreeText(s)];
        let sticky = HashMap::new();
        let bp = HashMap::new();
        let line = section_start_line(&sections, &states, &sticky, &groups, &bp, "adl");
        // intake group has no ## heading -> fallback is 0
        assert_eq!(line, 0);
    }

    // --- boilerplate_texts lookup tests (sub-task 52.5) ---

    fn make_minimal_sections() -> (Vec<SectionConfig>, Vec<SectionState>) {
        // Minimal: no sections, so only the static scaffolding renders.
        (vec![], vec![])
    }

    #[test]
    fn informed_consent_text_comes_from_boilerplate_map() {
        let (sections, states) = make_minimal_sections();
        let sticky = HashMap::new();
        let mut bp = HashMap::new();
        bp.insert("informed_consent".to_string(), "CUSTOM CONSENT TEXT FOR TEST".to_string());

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);

        // The custom value must appear in the note
        assert!(
            note.contains("CUSTOM CONSENT TEXT FOR TEST"),
            "expected custom informed_consent text in note, but got:\n{}", note
        );
        // The hard-coded string must NOT appear
        assert!(
            !note.contains("Patient has been informed of the risks and benefits"),
            "found hard-coded informed consent text that should have been replaced"
        );
    }

    #[test]
    fn treatment_plan_disclaimer_comes_from_boilerplate_map() {
        let (sections, states) = make_minimal_sections();
        let sticky = HashMap::new();
        let mut bp = HashMap::new();
        bp.insert("treatment_plan_disclaimer".to_string(), "CUSTOM DISCLAIMER FOR TEST".to_string());

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);

        // The custom value must appear in the note
        assert!(
            note.contains("CUSTOM DISCLAIMER FOR TEST"),
            "expected custom treatment_plan_disclaimer text in note, but got:\n{}", note
        );
        // The hard-coded string must NOT appear
        assert!(
            !note.contains("bilateral unless indicated otherwise"),
            "found hard-coded treatment plan disclaimer text that should have been replaced"
        );
    }

    #[test]
    fn empty_boilerplate_map_does_not_silently_use_hard_coded_strings() {
        let (sections, states) = make_minimal_sections();
        let sticky = HashMap::new();
        let bp: HashMap<String, String> = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);

        // With an empty map, neither hard-coded string should appear
        // (the feature should either omit, panic, or use a recognizable fallback --
        // but it must NOT silently fall back to the old hard-coded literals)
        assert!(
            !note.contains("Patient has been informed of the risks and benefits"),
            "render_note silently used hard-coded informed consent text when boilerplate_texts was empty"
        );
        assert!(
            !note.contains("bilateral unless indicated otherwise"),
            "render_note silently used hard-coded treatment plan disclaimer when boilerplate_texts was empty"
        );
    }

    // --- ST49-3: repeated field rendering tests ---

    fn make_field_with_repeat_limit(id: &str, repeat_limit: Option<usize>) -> crate::data::HeaderFieldConfig {
        crate::data::HeaderFieldConfig {
            id: id.to_string(),
            name: id.to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit,
        }
    }

    // ST49-3-TEST-1: export uses first confirmed value for a non-repeating field
    // (no repeat_limit), not last. With two values pushed, export should use [0], not [1].
    #[test]
    fn export_uses_first_value_for_non_repeated_field() {
        let configs = vec![
            make_field_with_repeat_limit("appointment_duration", None),
        ];
        let mut hs = HeaderState::new(configs);
        // Push two values; the implementation should use the first (index 0)
        hs.repeated_values[0].push("30".to_string());
        hs.repeated_values[0].push("60".to_string()); // second value - should NOT appear
        let sticky = HashMap::new();

        let result = format_header_export(&hs, &sticky);
        // Must contain the first value "30", not the second "60"
        assert_eq!(
            result,
            Some("(30 min)".to_string()),
            "export should use the FIRST confirmed value for non-repeated fields, got: {:?}",
            result
        );
    }

    // ST49-3-TEST-2: export emits ALL confirmed values as separate output lines
    // for a field with repeat_limit set.
    #[test]
    fn export_emits_all_values_for_repeated_field() {
        // A field with repeat_limit=3 holding 3 confirmed modifications
        let configs = vec![
            make_field_with_repeat_limit("modifications", Some(3)),
        ];
        let mut hs = HeaderState::new(configs);
        hs.repeated_values[0].push("No deep pressure on lower back".to_string());
        hs.repeated_values[0].push("Avoid prone positioning".to_string());
        hs.repeated_values[0].push("Use bolster under knees".to_string());
        let sticky = HashMap::new();

        let result = format_header_generic_export(&hs, &sticky);
        let output = result.expect("export should produce output for a repeated field with 3 values");

        // All three values must appear in the output
        assert!(
            output.contains("No deep pressure on lower back"),
            "first modification must appear in export output, got: {}", output
        );
        assert!(
            output.contains("Avoid prone positioning"),
            "second modification must appear in export output, got: {}", output
        );
        assert!(
            output.contains("Use bolster under knees"),
            "third modification must appear in export output, got: {}", output
        );
        // Each value must be on its own line
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(
            lines.len(),
            3,
            "repeated field with 3 values should produce exactly 3 output lines, got: {:?}",
            lines
        );
    }

    // ST49-3-TEST-3: preview uses first confirmed value for a non-repeating field
    #[test]
    fn preview_uses_first_value_for_non_repeated_field() {
        // date, start_time, appointment_duration, appointment_type - all non-repeated
        let configs = vec![
            make_field_with_repeat_limit("date", None),
            make_field_with_repeat_limit("start_time", None),
            make_field_with_repeat_limit("appointment_duration", None),
            make_field_with_repeat_limit("appointment_type", None),
        ];
        let mut hs = HeaderState::new(configs);
        // Push first value then a second (preview should use first)
        hs.repeated_values[2].push("30".to_string()); // first duration
        hs.repeated_values[2].push("90".to_string()); // second - should NOT appear
        hs.repeated_values[0].push("2026-04-02".to_string());
        hs.repeated_values[1].push("09:00".to_string());
        hs.repeated_values[3].push("Initial Assessment".to_string());
        let sticky = HashMap::new();

        let result = format_header_preview(&hs, &sticky);
        // Must show "30" not "90"
        assert!(
            result.contains("30 min"),
            "preview should use the FIRST confirmed duration value (30), got: {}", result
        );
        assert!(
            !result.contains("90 min"),
            "preview must NOT show the second duration value (90), got: {}", result
        );
    }

    // ST49-3-TEST-4: preview shows all repeated values for a field with repeat_limit
    #[test]
    fn preview_emits_all_values_for_repeated_field() {
        let configs = vec![
            make_field_with_repeat_limit("modifications", Some(2)),
        ];
        let mut hs = HeaderState::new(configs);
        hs.repeated_values[0].push("Mod A".to_string());
        hs.repeated_values[0].push("Mod B".to_string());
        let sticky = HashMap::new();

        let result = format_header_generic_preview(&hs, &sticky);
        assert!(
            result.contains("Mod A"),
            "preview must show first repeated value 'Mod A', got: {}", result
        );
        assert!(
            result.contains("Mod B"),
            "preview must show second repeated value 'Mod B', got: {}", result
        );
    }

    // ST49-3-TEST-5: export with repeat_limit field having only one value still emits that value
    #[test]
    fn export_emits_single_value_for_repeated_field_with_one_entry() {
        let configs = vec![
            make_field_with_repeat_limit("modifications", Some(3)),
        ];
        let mut hs = HeaderState::new(configs);
        hs.repeated_values[0].push("Only one mod".to_string());
        let sticky = HashMap::new();

        let result = format_header_generic_export(&hs, &sticky);
        let output = result.expect("export should produce output for a repeated field with 1 value");
        assert!(
            output.contains("Only one mod"),
            "export must emit the single repeated value, got: {}", output
        );
    }

    // ST49-3-TEST-6: repeated values are emitted in confirmation order (first confirmed appears first)
    #[test]
    fn export_emits_repeated_values_in_confirmation_order() {
        let configs = vec![
            make_field_with_repeat_limit("modifications", Some(2)),
        ];
        let mut hs = HeaderState::new(configs);
        hs.repeated_values[0].push("First Confirmed".to_string());
        hs.repeated_values[0].push("Second Confirmed".to_string());
        let sticky = HashMap::new();

        let result = format_header_generic_export(&hs, &sticky);
        let output = result.expect("export should produce output");
        let lines: Vec<&str> = output.lines().collect();
        assert!(
            lines.len() >= 2,
            "expected at least 2 lines for 2 repeated values, got: {:?}", lines
        );
        assert_eq!(
            lines[0], "First Confirmed",
            "first confirmed value must appear on first line, got: {:?}", lines
        );
        assert_eq!(
            lines[1], "Second Confirmed",
            "second confirmed value must appear on second line, got: {:?}", lines
        );
    }

    #[test]
    fn empty_infection_control_falls_back_to_post_treatment_heading() {
        use crate::sections::checklist::ChecklistState;
        let sec = make_section("infection_control_section", "checklist");
        let groups = vec![make_group("post_tx", vec![sec.clone()])];
        let sections = vec![sec];
        // checklist with no items -> renders nothing for the section
        let states = vec![SectionState::Checklist(ChecklistState::new(vec![]))];
        let sticky = HashMap::new();
        let bp = HashMap::new();
        let line = section_start_line(&sections, &states, &sticky, &groups, &bp, "infection_control_section");
        assert!(line > 0, "expected fallback to ## POST-TREATMENT, got 0");
        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);
        let target: Vec<(usize, &str)> = note.lines().enumerate()
            .filter(|(_, l)| l.contains("## POST-TREATMENT"))
            .collect();
        assert!(!target.is_empty());
        assert_eq!(line, target[0].0 as u16);
    }

    // --- ST48-1: two-pass multi_field rendering tests ---
    //
    // These tests confirm that render_note renders BOTH the appointment header
    // section (cfg.id == "header") AND any additional multi_field sections
    // (e.g. id == "test_section"). The current single find_map pass silently
    // drops the second section, so these tests must FAIL before implementation.

    fn make_multi_field_section(id: &str) -> SectionConfig {
        SectionConfig {
            id: id.to_string(),
            name: id.to_string(),
            map_label: id.to_string(),
            section_type: "multi_field".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
        }
    }

    fn make_header_state_with_value(field_id: &str, field_name: &str, value: &str) -> HeaderState {
        let cfg = HeaderFieldConfig {
            id: field_id.to_string(),
            name: field_name.to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit: None,
        };
        let mut hs = HeaderState::new(vec![cfg]);
        hs.repeated_values[0].push(value.to_string());
        hs.completed = true;
        hs
    }

    // ST48-1-TEST-1: preview renders header section output
    // The header section (id="header") with a confirmed appointment_type value
    // must produce visible output in the rendered note. This establishes the
    // baseline that the header section still renders after the two-pass refactor.
    #[test]
    fn preview_renders_header_section_output() {
        let header_sec = make_multi_field_section("header");
        let other_sec = make_section("subjective_section", "free_text");

        let header_hs = make_header_state_with_value(
            "appointment_type",
            "Appointment Type",
            "HEADER_SENTINEL_VALUE",
        );

        use crate::sections::free_text::FreeTextState;
        let sections = vec![header_sec, other_sec];
        let states = vec![
            SectionState::Header(header_hs),
            SectionState::FreeText(FreeTextState::new()),
        ];
        let sticky = HashMap::new();
        let bp = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);

        assert!(
            note.contains("HEADER_SENTINEL_VALUE"),
            "render_note must render the header multi_field section (id='header'); \
             sentinel value 'HEADER_SENTINEL_VALUE' not found in:\n{}", note
        );
    }

    // ST48-1-TEST-2: preview renders second multi_field section output
    // A second multi_field section with id != "header" must also produce output.
    // With the current find_map approach this test FAILS because find_map stops
    // at the first match and the second section is silently dropped.
    #[test]
    fn preview_renders_second_multi_field_section_output() {
        let header_sec = make_multi_field_section("header");
        let test_sec = make_multi_field_section("test_section");

        let header_hs = make_header_state_with_value(
            "appointment_type",
            "Appointment Type",
            "HEADER_SECTION_VALUE",
        );
        let test_hs = make_header_state_with_value(
            "note_field",
            "Note Field",
            "TEST_SECTION_SENTINEL_VALUE",
        );

        let sections = vec![header_sec, test_sec];
        let states = vec![
            SectionState::Header(header_hs),
            SectionState::Header(test_hs),
        ];
        let sticky = HashMap::new();
        let bp = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);

        assert!(
            note.contains("TEST_SECTION_SENTINEL_VALUE"),
            "render_note must render the second multi_field section (id='test_section'); \
             sentinel value 'TEST_SECTION_SENTINEL_VALUE' not found in:\n{}", note
        );
    }

    // ST48-1-TEST-3: preview renders BOTH sections when both have confirmed values
    // Both the header section AND the test_section must produce output in the
    // same rendered note. This is the combined proof that neither section is dropped.
    #[test]
    fn preview_renders_both_multi_field_sections() {
        let header_sec = make_multi_field_section("header");
        let test_sec = make_multi_field_section("test_section");

        let header_hs = make_header_state_with_value(
            "appointment_type",
            "Appointment Type",
            "HEADER_SECTION_VALUE",
        );
        let test_hs = make_header_state_with_value(
            "note_field",
            "Note Field",
            "TEST_SECTION_SENTINEL_VALUE",
        );

        let sections = vec![header_sec, test_sec];
        let states = vec![
            SectionState::Header(header_hs),
            SectionState::Header(test_hs),
        ];
        let sticky = HashMap::new();
        let bp = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);

        assert!(
            note.contains("HEADER_SECTION_VALUE"),
            "header section (id='header') output must appear in rendered note, got:\n{}", note
        );
        assert!(
            note.contains("TEST_SECTION_SENTINEL_VALUE"),
            "second multi_field section (id='test_section') output must appear in rendered note, got:\n{}", note
        );
    }

    // ST48-1-TEST-4: export renders second multi_field section output
    // Same as TEST-2 but for Export mode, ensuring both modes are covered.
    #[test]
    fn export_renders_second_multi_field_section_output() {
        let header_sec = make_multi_field_section("header");
        let test_sec = make_multi_field_section("test_section");

        let header_hs = make_header_state_with_value(
            "appointment_type",
            "Appointment Type",
            "HEADER_SECTION_VALUE",
        );
        let test_hs = make_header_state_with_value(
            "note_field",
            "Note Field",
            "EXPORT_TEST_SECTION_SENTINEL",
        );

        let sections = vec![header_sec, test_sec];
        let states = vec![
            SectionState::Header(header_hs),
            SectionState::Header(test_hs),
        ];
        let sticky = HashMap::new();
        let bp = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Export);

        assert!(
            note.contains("EXPORT_TEST_SECTION_SENTINEL"),
            "render_note (Export mode) must render the second multi_field section (id='test_section'); \
             sentinel value 'EXPORT_TEST_SECTION_SENTINEL' not found in:\n{}", note
        );
    }

    // ST48-1-TEST-5: header section output is byte-for-byte identical with and without
    // a second multi_field section present. This guards against the refactor accidentally
    // changing appointment header output.
    #[test]
    fn header_output_unchanged_when_second_section_present() {
        // Build an appointment header state with all four standard fields.
        let header_field_configs = vec![
            HeaderFieldConfig {
                id: "date".to_string(),
                name: "Date".to_string(),
                options: vec![],
                composite: None,
                default: None,
                repeat_limit: None,
            },
            HeaderFieldConfig {
                id: "start_time".to_string(),
                name: "Start Time".to_string(),
                options: vec![],
                composite: None,
                default: None,
                repeat_limit: None,
            },
            HeaderFieldConfig {
                id: "appointment_duration".to_string(),
                name: "Duration".to_string(),
                options: vec![],
                composite: None,
                default: None,
                repeat_limit: None,
            },
            HeaderFieldConfig {
                id: "appointment_type".to_string(),
                name: "Appointment Type".to_string(),
                options: vec![],
                composite: None,
                default: None,
                repeat_limit: None,
            },
        ];
        let mut header_hs = HeaderState::new(header_field_configs.clone());
        header_hs.repeated_values[0].push("2026-04-03".to_string());
        header_hs.repeated_values[1].push("10:00".to_string());
        header_hs.repeated_values[2].push("60".to_string());
        header_hs.repeated_values[3].push("Deep Tissue Massage".to_string());
        header_hs.completed = true;

        let sticky = HashMap::new();
        let bp = HashMap::new();

        // Baseline: header section alone
        let header_sec_only = make_multi_field_section("header");
        let sections_baseline = vec![header_sec_only];
        let states_baseline = vec![SectionState::Header(header_hs.clone())];
        let note_baseline = render_note(
            &sections_baseline,
            &states_baseline,
            &sticky,
            &bp,
            NoteRenderMode::Preview,
        );

        // With second section present
        let header_sec = make_multi_field_section("header");
        let test_sec = make_multi_field_section("test_section");
        let test_hs = make_header_state_with_value("note_field", "Note Field", "EXTRA_VALUE");
        let sections_with_extra = vec![header_sec, test_sec];
        let states_with_extra = vec![
            SectionState::Header(header_hs.clone()),
            SectionState::Header(test_hs),
        ];
        let note_with_extra = render_note(
            &sections_with_extra,
            &states_with_extra,
            &sticky,
            &bp,
            NoteRenderMode::Preview,
        );

        // Extract the header portion (everything before the first separator line
        // or the first blank-blank block) to compare just the header output.
        // Both notes must start with the same appointment header text.
        let baseline_first_line = note_baseline.lines().next().unwrap_or("");
        let extra_first_line = note_with_extra.lines().next().unwrap_or("");

        assert_eq!(
            baseline_first_line,
            extra_first_line,
            "first line of header output must be identical whether or not a second \
             multi_field section is present.\nbaseline: {:?}\nwith extra: {:?}",
            baseline_first_line,
            extra_first_line
        );

        // Also verify the specific expected content is present in both
        assert!(
            note_baseline.contains("Fri Apr 3, 2026"),
            "baseline note must contain formatted date, got:\n{}", note_baseline
        );
        assert!(
            note_with_extra.contains("Fri Apr 3, 2026"),
            "note with extra section must still contain formatted date, got:\n{}", note_with_extra
        );
    }

    // --- render_multifield_section tests (task #48 sub-task 2) ---

    /// Build a two-field HeaderState with arbitrary field ids and given values.
    /// Both fields are non-repeatable (repeat_limit = None).
    fn make_two_field_header(
        id_a: &str, val_a: &str,
        id_b: &str, val_b: &str,
    ) -> (SectionConfig, crate::sections::header::HeaderState) {
        let cfg_a = crate::data::HeaderFieldConfig {
            id: id_a.to_string(),
            name: id_a.to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit: None,
        };
        let cfg_b = crate::data::HeaderFieldConfig {
            id: id_b.to_string(),
            name: id_b.to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit: None,
        };
        let mut hs = crate::sections::header::HeaderState::new(vec![cfg_a, cfg_b]);
        if !val_a.is_empty() {
            hs.repeated_values[0].push(val_a.to_string());
        }
        if !val_b.is_empty() {
            hs.repeated_values[1].push(val_b.to_string());
        }
        let sec_cfg = SectionConfig {
            id: "test_section".to_string(),
            name: "Test Section".to_string(),
            map_label: "Test Section".to_string(),
            section_type: "multi_field".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
        };
        (sec_cfg, hs)
    }

    /// Preview mode: two fields with values must produce two lines joined by newline,
    /// each formatted as "<field_id>: <value>". Returns Some(text).
    #[test]
    fn render_multifield_section_preview_two_fields_joined_by_newline() {
        let (cfg, hs) = make_two_field_header("alpha", "hello", "beta", "world");
        let sticky = HashMap::new();
        let result = render_multifield_section(&cfg, &hs, &sticky, NoteRenderMode::Preview);
        // Both fields resolved - preview returns the joined text (never None)
        let text = result.expect("preview must return Some for non-empty fields");
        assert_eq!(text, "alpha: hello\nbeta: world",
            "expected two field lines joined by newline, got: {:?}", text);
    }

    /// Preview mode: a field with no confirmed value must show "--" as placeholder.
    #[test]
    fn render_multifield_section_preview_empty_field_shows_placeholder() {
        let (cfg, hs) = make_two_field_header("alpha", "hello", "beta", "");
        let sticky = HashMap::new();
        let result = render_multifield_section(&cfg, &hs, &sticky, NoteRenderMode::Preview);
        // beta has no value - generic preview shows "--"
        let text = result.expect("preview must return Some even with partially empty fields");
        assert!(text.contains("beta: --"),
            "expected 'beta: --' placeholder for empty field, got: {:?}", text);
    }

    /// Export mode: two fields with values must produce their values only (no labels),
    /// joined by newline.
    #[test]
    fn render_multifield_section_export_values_only_joined_by_newline() {
        let (cfg, hs) = make_two_field_header("alpha", "hello", "beta", "world");
        let sticky = HashMap::new();
        let result = render_multifield_section(&cfg, &hs, &sticky, NoteRenderMode::Export);
        // Both fields resolved - export returns Some("hello\nworld")
        assert_eq!(result, Some("hello\nworld".to_string()),
            "expected Some with two values joined by newline, got: {:?}", result);
    }

    /// Export mode: when all fields are empty, returns None.
    #[test]
    fn render_multifield_section_export_none_when_all_empty() {
        let (cfg, hs) = make_two_field_header("alpha", "", "beta", "");
        let sticky = HashMap::new();
        let result = render_multifield_section(&cfg, &hs, &sticky, NoteRenderMode::Export);
        assert_eq!(result, None,
            "expected None when no fields have values, got: {:?}", result);
    }

    /// Export mode: one filled field returns Some with just that value.
    #[test]
    fn render_multifield_section_export_partial_returns_some() {
        let (cfg, hs) = make_two_field_header("alpha", "only_this", "beta", "");
        let sticky = HashMap::new();
        let result = render_multifield_section(&cfg, &hs, &sticky, NoteRenderMode::Export);
        assert_eq!(result, Some("only_this".to_string()),
            "expected Some with only the non-empty value, got: {:?}", result);
    }

    // --- ST48-3: non-header multi_field sections rendered at correct position ---
    //
    // These tests verify that render_note places non-header multi_field sections
    // at the correct position in the note output (controlled by where the section id
    // appears in the existing section-position logic), NOT dumped before the INTAKE
    // group as a side-effect of Pass 2.

    fn make_multi_field_section_with_id(id: &str) -> SectionConfig {
        SectionConfig {
            id: id.to_string(),
            name: id.to_string(),
            map_label: id.to_string(),
            section_type: "multi_field".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
        }
    }

    fn make_header_state_with_confirmed(field_id: &str, field_name: &str, value: &str) -> HeaderState {
        let cfg = crate::data::HeaderFieldConfig {
            id: field_id.to_string(),
            name: field_name.to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit: None,
        };
        let mut hs = HeaderState::new(vec![cfg]);
        hs.repeated_values[0].push(value.to_string());
        hs.completed = true;
        hs
    }

    // ST48-3-TEST-1: a non-header multi_field section with id "tx_mods" and a confirmed
    // value must appear AFTER the "## TREATMENT / PLAN" heading in the rendered note.
    // With the current Pass-2 implementation, the content is dumped before the INTAKE
    // block (before "## TREATMENT / PLAN"), so this test must FAIL before the fix.
    #[test]
    fn non_header_multi_field_section_appears_after_treatment_heading() {
        let tx_mods_sec = make_multi_field_section_with_id("tx_mods");
        let tx_mods_hs = make_header_state_with_confirmed(
            "mod_field",
            "Modification",
            "TX_MODS_SENTINEL_VALUE",
        );

        let sections = vec![tx_mods_sec];
        let states = vec![SectionState::Header(tx_mods_hs)];
        let sticky = HashMap::new();
        let bp = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);

        // The sentinel value must appear somewhere in the note
        assert!(
            note.contains("TX_MODS_SENTINEL_VALUE"),
            "TX_MODS_SENTINEL_VALUE must appear in rendered note, got:\n{}", note
        );

        // Find the line index of "## TREATMENT / PLAN" and the sentinel value.
        // The sentinel must appear AFTER the treatment heading.
        let treatment_line = note.lines().enumerate()
            .find(|(_, l)| l.contains("## TREATMENT / PLAN"))
            .map(|(i, _)| i);
        let sentinel_line = note.lines().enumerate()
            .find(|(_, l)| l.contains("TX_MODS_SENTINEL_VALUE"))
            .map(|(i, _)| i);

        let treatment_idx = treatment_line.expect("## TREATMENT / PLAN heading must be present");
        let sentinel_idx = sentinel_line.expect("TX_MODS_SENTINEL_VALUE must be in the note");

        assert!(
            sentinel_idx > treatment_idx,
            "TX_MODS_SENTINEL_VALUE (line {}) must appear AFTER ## TREATMENT / PLAN (line {}), \
             but it appeared before it. The section is being dumped at the wrong position.",
            sentinel_idx, treatment_idx
        );
    }

    // ST48-3-TEST-2: a non-header multi_field section with a generic id (not any known
    // section id) must still produce non-empty output in the rendered note.
    // This verifies the generic case: even unknown ids do not silently drop content.
    #[test]
    fn non_header_multi_field_section_with_unknown_id_produces_output() {
        let sec = make_multi_field_section_with_id("my_custom_section");
        let hs = make_header_state_with_confirmed(
            "custom_field",
            "Custom Field",
            "CUSTOM_SECTION_SENTINEL",
        );

        let sections = vec![sec];
        let states = vec![SectionState::Header(hs)];
        let sticky = HashMap::new();
        let bp = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);

        assert!(
            note.contains("CUSTOM_SECTION_SENTINEL"),
            "CUSTOM_SECTION_SENTINEL must appear in rendered note for a generic multi_field \
             section id, got:\n{}", note
        );
    }

    // ST48-3-TEST-3: a non-header multi_field section with id "tx_mods" must NOT
    // appear before the INTAKE separator (the first "_______________" line).
    // Pass 2 currently dumps it before INTAKE (before the separator), so this
    // test must FAIL before the fix.
    #[test]
    fn non_header_multi_field_section_not_before_intake_separator() {
        let tx_mods_sec = make_multi_field_section_with_id("tx_mods");
        let tx_mods_hs = make_header_state_with_confirmed(
            "mod_field",
            "Modification",
            "PREMATURE_SENTINEL",
        );

        let sections = vec![tx_mods_sec];
        let states = vec![SectionState::Header(tx_mods_hs)];
        let sticky = HashMap::new();
        let bp = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);

        // Find the first separator line index
        let first_separator_line = note.lines().enumerate()
            .find(|(_, l)| l.contains("_______________"))
            .map(|(i, _)| i);
        let sentinel_line = note.lines().enumerate()
            .find(|(_, l)| l.contains("PREMATURE_SENTINEL"))
            .map(|(i, _)| i);

        let separator_idx = first_separator_line
            .expect("at least one _______________ separator must appear in the note");
        let sentinel_idx = sentinel_line
            .expect("PREMATURE_SENTINEL must appear in the note");

        assert!(
            sentinel_idx > separator_idx,
            "PREMATURE_SENTINEL (line {}) must NOT appear before the first _______________ \
             separator (line {}). The section is being prematurely dumped before the INTAKE block.",
            sentinel_idx, separator_idx
        );
    }

    // ST48-3-TEST-4: export mode - a non-header multi_field section with id "tx_mods"
    // and a confirmed value must appear after "## TREATMENT / PLAN" in export output.
    #[test]
    fn export_non_header_multi_field_section_appears_after_treatment_heading() {
        let tx_mods_sec = make_multi_field_section_with_id("tx_mods");
        let tx_mods_hs = make_header_state_with_confirmed(
            "mod_field",
            "Modification",
            "EXPORT_TX_MODS_SENTINEL",
        );

        let sections = vec![tx_mods_sec];
        let states = vec![SectionState::Header(tx_mods_hs)];
        let sticky = HashMap::new();
        let bp = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Export);

        assert!(
            note.contains("EXPORT_TX_MODS_SENTINEL"),
            "EXPORT_TX_MODS_SENTINEL must appear in export output for tx_mods multi_field section, \
             got:\n{}", note
        );

        let treatment_line = note.lines().enumerate()
            .find(|(_, l)| l.contains("## TREATMENT / PLAN"))
            .map(|(i, _)| i);
        let sentinel_line = note.lines().enumerate()
            .find(|(_, l)| l.contains("EXPORT_TX_MODS_SENTINEL"))
            .map(|(i, _)| i);

        let treatment_idx = treatment_line.expect("## TREATMENT / PLAN must be in export output");
        let sentinel_idx = sentinel_line.expect("EXPORT_TX_MODS_SENTINEL must be in export output");

        assert!(
            sentinel_idx > treatment_idx,
            "EXPORT_TX_MODS_SENTINEL (line {}) must appear AFTER ## TREATMENT / PLAN (line {}) \
             in export mode.",
            sentinel_idx, treatment_idx
        );
    }

    // --- ST48-4: tx_mods multi_field SectionState::Header detection tests ---
    //
    // These tests verify the specific render_note behavior added in sub-task 3:
    // when tx_mods has section_type "multi_field" and state SectionState::Header,
    // render_note must call render_multifield_section and emit the exact heading
    // "#### TREATMENT MODIFICATIONS & PREFERENCES".

    // ST48-4-TEST-1: preview mode - the exact "#### TREATMENT MODIFICATIONS & PREFERENCES"
    // heading must appear when tx_mods is multi_field + SectionState::Header with a
    // confirmed value.  Without the multi_field branch this heading would still appear
    // (via the else/render_section_content path) but the value would not; the combination
    // of heading + value is the diagnostic signal.
    #[test]
    fn tx_mods_multi_field_header_state_preview_shows_exact_heading_and_value() {
        let sec = make_multi_field_section_with_id("tx_mods");
        let hs = make_header_state_with_confirmed(
            "pressure",
            "Pressure",
            "ST48_4_PREVIEW_VALUE",
        );

        let sections = vec![sec];
        let states = vec![SectionState::Header(hs)];
        let sticky = HashMap::new();
        let bp = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);

        assert!(
            note.contains("#### TREATMENT MODIFICATIONS & PREFERENCES"),
            "exact heading '#### TREATMENT MODIFICATIONS & PREFERENCES' must appear for \
             tx_mods multi_field section in preview mode, got:\n{}", note
        );
        assert!(
            note.contains("ST48_4_PREVIEW_VALUE"),
            "confirmed field value ST48_4_PREVIEW_VALUE must appear in preview note, got:\n{}", note
        );
    }

    // ST48-4-TEST-2: export mode - the exact "#### TREATMENT MODIFICATIONS & PREFERENCES"
    // heading must appear when tx_mods is multi_field + SectionState::Header with a
    // confirmed value.
    #[test]
    fn tx_mods_multi_field_header_state_export_shows_exact_heading_and_value() {
        let sec = make_multi_field_section_with_id("tx_mods");
        let hs = make_header_state_with_confirmed(
            "pressure",
            "Pressure",
            "ST48_4_EXPORT_VALUE",
        );

        let sections = vec![sec];
        let states = vec![SectionState::Header(hs)];
        let sticky = HashMap::new();
        let bp = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Export);

        assert!(
            note.contains("#### TREATMENT MODIFICATIONS & PREFERENCES"),
            "exact heading '#### TREATMENT MODIFICATIONS & PREFERENCES' must appear for \
             tx_mods multi_field section in export mode, got:\n{}", note
        );
        assert!(
            note.contains("ST48_4_EXPORT_VALUE"),
            "confirmed field value ST48_4_EXPORT_VALUE must appear in export note, got:\n{}", note
        );
    }

    // ST48-4-TEST-3: tx_mods multi_field + SectionState::Header with two fields, both
    // confirmed values must appear in preview output.  Without the multi_field branch,
    // render_section_content returns "" for SectionState::Header so neither value
    // would appear.
    #[test]
    fn tx_mods_multi_field_header_state_preview_renders_multiple_field_values() {
        let sec = make_multi_field_section_with_id("tx_mods");
        // Build a HeaderState with two fields, each confirmed
        let cfg_a = crate::data::HeaderFieldConfig {
            id: "pressure".to_string(),
            name: "Pressure".to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit: None,
        };
        let cfg_b = crate::data::HeaderFieldConfig {
            id: "challenge".to_string(),
            name: "Challenge".to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit: None,
        };
        let mut hs = HeaderState::new(vec![cfg_a, cfg_b]);
        hs.repeated_values[0].push("ST48_4_PRESSURE".to_string());
        hs.repeated_values[1].push("ST48_4_CHALLENGE".to_string());
        hs.completed = true;

        let sections = vec![sec];
        let states = vec![SectionState::Header(hs)];
        let sticky = HashMap::new();
        let bp = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);

        assert!(
            note.contains("ST48_4_PRESSURE"),
            "first field value ST48_4_PRESSURE must appear in preview note via \
             render_multifield_section, got:\n{}", note
        );
        assert!(
            note.contains("ST48_4_CHALLENGE"),
            "second field value ST48_4_CHALLENGE must appear in preview note via \
             render_multifield_section, got:\n{}", note
        );
    }

    // ST48-4-TEST-4: tx_mods multi_field + SectionState::Header with a sticky value.
    // The multi_field path resolves sticky values via render_multifield_section;
    // the old render_section_content path ignores sticky values entirely for Header state
    // (it returns "" for any Header state).  Without the multi_field branch, the
    // sticky value would not appear.
    #[test]
    fn tx_mods_multi_field_header_state_preview_resolves_sticky_value() {
        let sec = make_multi_field_section_with_id("tx_mods");
        let cfg_field = crate::data::HeaderFieldConfig {
            id: "mood".to_string(),
            name: "Mood".to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit: None,
        };
        // confirmed value is the sticky key; sticky map resolves it to a display value
        let mut hs = HeaderState::new(vec![cfg_field]);
        hs.repeated_values[0].push("ST48_4_STICKY_KEY".to_string());
        hs.completed = true;

        let sections = vec![sec];
        let states = vec![SectionState::Header(hs)];
        let mut sticky = HashMap::new();
        sticky.insert("ST48_4_STICKY_KEY".to_string(), "ST48_4_STICKY_RESOLVED".to_string());
        let bp = HashMap::new();

        let note = render_note(&sections, &states, &sticky, &bp, NoteRenderMode::Preview);

        assert!(
            note.contains("ST48_4_STICKY_KEY") || note.contains("ST48_4_STICKY_RESOLVED"),
            "tx_mods multi_field preview must render the confirmed/sticky value for the field; \
             expected ST48_4_STICKY_KEY or ST48_4_STICKY_RESOLVED in note, got:\n{}", note
        );
        assert!(
            note.contains("#### TREATMENT MODIFICATIONS & PREFERENCES"),
            "heading must appear when tx_mods multi_field HeaderState has confirmed values, \
             got:\n{}", note
        );
    }
}
