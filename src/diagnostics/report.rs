use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorSource {
    pub file: PathBuf,
    pub line: usize,
    pub quoted_line: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Generic {
        kind_id: &'static str,
    },
    LooksLikeListMissingItems {
        id: String,
        registered_as: String,
        found_fingerprints: Vec<String>,
    },
    LooksLikeCollectionMissingKey {
        id: String,
        registered_as: String,
        found_fingerprints: Vec<String>,
    },
    LooksLikeSectionOrGroupMissingKey {
        id: String,
        inferred_kind: String,
        registered_as: String,
        found_fingerprints: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorReport {
    pub kind: ErrorKind,
    pub message: String,
    pub source: Option<ErrorSource>,
    pub extra_params: Vec<(String, String)>,
}

impl ErrorReport {
    pub fn generic(kind_id: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Generic { kind_id },
            message: message.into(),
            source: None,
            extra_params: Vec::new(),
        }
    }

    pub fn with_source(mut self, source: Option<ErrorSource>) -> Self {
        self.source = source;
        self
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_params.push((key.into(), value.into()));
        self
    }

    pub fn kind_id(&self) -> &'static str {
        match &self.kind {
            ErrorKind::Generic { kind_id } => kind_id,
            ErrorKind::LooksLikeListMissingItems { .. } => "looks_like_list_missing_items",
            ErrorKind::LooksLikeCollectionMissingKey { .. } => "looks_like_collection_missing_key",
            ErrorKind::LooksLikeSectionOrGroupMissingKey { .. } => {
                "looks_like_section_or_group_missing_key"
            }
        }
    }

    pub fn params(&self) -> Vec<(String, String)> {
        let mut params = match &self.kind {
            ErrorKind::Generic { .. } => Vec::new(),
            ErrorKind::LooksLikeListMissingItems {
                id,
                registered_as,
                found_fingerprints,
            } => vec![
                ("id".to_string(), id.clone()),
                ("registered_as".to_string(), registered_as.clone()),
                (
                    "found_fingerprints".to_string(),
                    found_fingerprints.join(", "),
                ),
            ],
            ErrorKind::LooksLikeCollectionMissingKey {
                id,
                registered_as,
                found_fingerprints,
            } => vec![
                ("id".to_string(), id.clone()),
                ("registered_as".to_string(), registered_as.clone()),
                (
                    "found_fingerprints".to_string(),
                    found_fingerprints.join(", "),
                ),
            ],
            ErrorKind::LooksLikeSectionOrGroupMissingKey {
                id,
                inferred_kind,
                registered_as,
                found_fingerprints,
            } => vec![
                ("id".to_string(), id.clone()),
                ("inferred_kind".to_string(), inferred_kind.clone()),
                ("registered_as".to_string(), registered_as.clone()),
                (
                    "found_fingerprints".to_string(),
                    found_fingerprints.join(", "),
                ),
            ],
        };
        params.extend(self.extra_params.clone());
        params
    }

    #[cfg(test)]
    pub fn contains(&self, pattern: &str) -> bool {
        self.message.contains(pattern)
    }
}

impl fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(source) = &self.source {
            write!(f, " Source: {}:{}", source.file.display(), source.line)?;
            if let Some(quoted_line) = &source.quoted_line {
                write!(f, " (`{}`)", quoted_line)?;
            }
        }
        Ok(())
    }
}

impl Error for ErrorReport {}
