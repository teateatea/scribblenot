use crate::app::SectionState;
use crate::data::SectionConfig;
use chrono::Local;

pub fn section_start_line(sections: &[SectionConfig], states: &[SectionState], section_id: &str) -> u16 {
    let note = render_note(sections, states);
    let search = heading_search_text(section_id);
    if search.is_empty() {
        return 0;
    }
    for (i, line) in note.lines().enumerate() {
        if line.contains(search) {
            return i as u16;
        }
    }
    0
}

fn heading_search_text(id: &str) -> &'static str {
    match id {
        "adl" => "ACTIVITIES OF DAILY LIVING",
        "exercise" => "EXERCISE HABITS",
        "sleep_diet" => "SLEEP & DIET",
        "social" => "SOCIAL & STRESS",
        "history" => "HISTORY & PREVIOUS DIAGNOSES",
        "specialists" => "SPECIALISTS & TREATMENT",
        "subjective" => "## SUBJECTIVE",
        "tx_mods" => "TREATMENT MODIFICATIONS",
        "tx_regions" => "TREATMENT / PLAN",
        "objective" => "## OBJECTIVE",
        "post_treatment" => "## POST-TREATMENT",
        "remedial" => "REMEDIAL EXERCISES",
        "tx_plan" => "TREATMENT PLAN",
        "infection_control" => "INFECTION CONTROL",
        _ => "",
    }
}

pub fn render_note(sections: &[SectionConfig], states: &[SectionState]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Header is always first - find it
    let header_text = sections.iter().zip(states.iter()).find_map(|(cfg, state)| {
        if cfg.section_type == "multi_field" {
            if let SectionState::Header(hs) = state {
                if hs.completed {
                    return Some(format_header(hs));
                }
            }
        }
        None
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
        if cfg.id == "subjective" {
            let rendered = render_section_content(cfg, state, &today);
            if !rendered.trim().is_empty() {
                subj_parts.push(format!("\n{}", rendered));
            }
        }
    }
    subj_parts.push("\n\n\n#### INFORMED CONSENT\n- Patient has been informed of the risks and benefits of massage therapy, and has given informed consent to assessment and treatment.".to_string());
    parts.push(subj_parts.join(""));

    // SEPARATOR
    parts.push("\n\n\n_______________".to_string());

    // TREATMENT / PLAN
    let mut tx_parts: Vec<String> = Vec::new();
    tx_parts.push("\n\n## TREATMENT / PLAN\nRegions and locations are bilateral unless indicated otherwise.\nPatient is pillowed under ankles when prone, and under knees when supine.".to_string());

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
        if cfg.id == "objective" {
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
        if cfg.id == "remedial" {
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
        if cfg.id == "infection_control" {
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
        for region_state in &s.regions {
            let selected: Vec<String> = region_state
                .technique_selected
                .iter()
                .enumerate()
                .filter(|(_, &sel)| sel)
                .filter_map(|(i, _)| region_state.techniques.get(i))
                .map(|t| t.output.clone())
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

fn format_header(hs: &crate::sections::header::HeaderState) -> String {
    let date_str = format_header_date(hs.get_value("date"));
    let time_str = format_header_time(hs.get_value("start_time"));
    let dur_str = hs.get_value("duration");
    let appt_str = hs.get_value("appointment_type");
    format!("{} at {} ({} min)\n{}", date_str, time_str, dur_str, appt_str)
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
