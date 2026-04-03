use crate::app::SectionState;
use crate::data::{SectionConfig, SectionGroup};
use crate::sections::multi_field::resolve_multifield_value;
use chrono::Local;
use std::collections::HashMap;

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

    // Header: always first - find the multi_field section
    let header_text = sections.iter().zip(states.iter()).find_map(|(cfg, state)| {
        if cfg.section_type == "multi_field" {
            if let SectionState::Header(hs) = state {
                match &mode {
                    NoteRenderMode::Preview => {
                        let has_any = hs.field_configs.iter().zip(hs.values.iter()).any(|(fcfg, confirmed)| {
                            !resolve_multifield_value(confirmed, fcfg, sticky_values).is_empty_variant()
                        });
                        if has_any {
                            Some(format_header_preview(hs, sticky_values))
                        } else {
                            None
                        }
                    }
                    NoteRenderMode::Export => format_header_export(hs, sticky_values),
                }
            } else {
                None
            }
        } else {
            None
        }
    });

    if let Some(h) = header_text {
        parts.push(h);
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
            let rendered = render_section_content(cfg, state, &today);
            if !rendered.trim().is_empty() {
                tx_parts.push(format!("\n\n\n#### TREATMENT MODIFICATIONS & PREFERENCES\n{}", rendered));
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
            .zip(hs.values.iter())
            .find(|(cfg, _)| cfg.id == id)
            .map(|(cfg, confirmed)| {
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
            .zip(hs.values.iter())
            .find(|(cfg, _)| cfg.id == id)
            .and_then(|(cfg, confirmed)| {
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
            })
            .collect();
        let mut hs = HeaderState::new(configs);
        for (i, val) in values.iter().enumerate() {
            if let Some(v) = hs.values.get_mut(i) {
                *v = val.to_string();
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
        };
        let configs = vec![date_cfg, dur_cfg];
        let mut hs = HeaderState::new(configs);
        hs.values[1] = "60".to_string(); // only duration confirmed
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
        };
        let time_cfg = HeaderFieldConfig {
            id: "start_time".to_string(),
            name: "Start Time".to_string(),
            options: vec![],
            default: None,
            composite: None,
        };
        let dur_cfg = HeaderFieldConfig {
            id: "appointment_duration".to_string(),
            name: "Duration".to_string(),
            options: vec![],
            composite: None,
            default: None,
        };
        let appt_cfg = HeaderFieldConfig {
            id: "appointment_type".to_string(),
            name: "Appointment Type".to_string(),
            options: vec![],
            composite: None,
            default: None,
        };
        let configs = vec![date_cfg, time_cfg, dur_cfg, appt_cfg];
        let mut hs = HeaderState::new(configs);
        hs.values[0] = "2026-04-02".to_string();
        hs.values[1] = "13:00".to_string();
        hs.values[2] = "60".to_string();
        hs.values[3] = "Treatment focused massage".to_string();
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
}
