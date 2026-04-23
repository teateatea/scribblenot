use crate::error_report::ErrorReport;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
struct MessageFile {
    errors: Vec<MessageEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageEntry {
    pub id: String,
    pub title: String,
    pub description: String,
    pub fix: String,
}

#[derive(Debug, Clone, Default)]
pub struct Messages {
    entries: HashMap<String, MessageEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedError {
    pub title: String,
    pub description: String,
    pub fix: String,
    pub source: Option<RenderedErrorSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedErrorSource {
    pub location: String,
    pub quoted_line: Option<String>,
}

impl Messages {
    pub fn load(root: &Path) -> Self {
        let mut messages = Self::default();
        let entries = match fs::read_dir(root) {
            Ok(entries) => entries,
            Err(err) => {
                eprintln!(
                    "Warning: failed to read messages dir '{}': {err}; using raw error text",
                    root.display()
                );
                return messages;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
                continue;
            }
            let content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!(
                        "Warning: failed to read message file '{}': {err}",
                        path.display()
                    );
                    continue;
                }
            };
            let file: MessageFile = match serde_yaml::from_str(&content) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!(
                        "Warning: failed to parse message file '{}': {err}",
                        path.display()
                    );
                    continue;
                }
            };
            for entry in file.errors {
                messages.entries.insert(entry.id.clone(), entry);
            }
        }

        messages
    }

    pub fn render(&self, report: &ErrorReport) -> RenderedError {
        let params = report_params(report);
        let kind_id = report.kind_id();
        let Some(entry) = self.entries.get(kind_id) else {
            return RenderedError {
                title: title_from_kind(kind_id),
                description: report.message.clone(),
                fix: String::new(),
                source: render_source(report),
            };
        };

        RenderedError {
            title: substitute(&entry.title, &params),
            description: substitute(&entry.description, &params),
            fix: substitute(&entry.fix, &params),
            source: render_source(report),
        }
    }
}

fn report_params(report: &ErrorReport) -> HashMap<String, String> {
    let mut params = HashMap::new();
    params.insert("message".to_string(), report.message.clone());
    params.insert("kind_id".to_string(), report.kind_id().to_string());
    if let Some(source) = &report.source {
        params.insert("file".to_string(), source.file.display().to_string());
        params.insert("line".to_string(), source.line.to_string());
        params.insert(
            "quoted_line".to_string(),
            source.quoted_line.clone().unwrap_or_default(),
        );
    }
    for (key, value) in report.params() {
        params.insert(key.to_string(), value);
    }
    params
}

fn render_source(report: &ErrorReport) -> Option<RenderedErrorSource> {
    report.source.as_ref().map(|source| RenderedErrorSource {
        location: format!("{}:{}", source.file.display(), source.line),
        quoted_line: source.quoted_line.clone(),
    })
}

fn substitute(template: &str, params: &HashMap<String, String>) -> String {
    let mut rendered = template.to_string();
    for (key, value) in params {
        rendered = rendered.replace(&format!("{{{key}}}"), value);
    }
    rendered
}

fn title_from_kind(kind_id: &str) -> String {
    kind_id
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_report::{ErrorReport, ErrorSource};
    use std::path::PathBuf;

    #[test]
    fn render_substitutes_raw_message_and_source() {
        let mut messages = Messages::default();
        messages.entries.insert(
            "yaml_parse_failed".to_string(),
            MessageEntry {
                id: "yaml_parse_failed".to_string(),
                title: "YAML Problem".to_string(),
                description: "{message}".to_string(),
                fix: "Check {file}:{line}".to_string(),
            },
        );
        let report = ErrorReport::generic("yaml_parse_failed", "bad yaml").with_source(Some(
            ErrorSource {
                file: PathBuf::from("data/demo.yml"),
                line: 3,
                quoted_line: Some("bad: [".to_string()),
            },
        ));

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "YAML Problem");
        assert_eq!(rendered.description, "bad yaml");
        assert_eq!(rendered.fix, "Check data/demo.yml:3");
        assert_eq!(
            rendered.source.as_ref().map(|source| source.location.as_str()),
            Some("data/demo.yml:3")
        );
    }
}
