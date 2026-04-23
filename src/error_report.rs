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
}

impl ErrorReport {
    pub fn generic(kind_id: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Generic { kind_id },
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source(mut self, source: Option<ErrorSource>) -> Self {
        self.source = source;
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

    pub fn params(&self) -> Vec<(&'static str, String)> {
        match &self.kind {
            ErrorKind::Generic { .. } => Vec::new(),
            ErrorKind::LooksLikeListMissingItems {
                id,
                registered_as,
                found_fingerprints,
            } => vec![
                ("id", id.clone()),
                ("registered_as", registered_as.clone()),
                ("found_fingerprints", found_fingerprints.join(", ")),
            ],
            ErrorKind::LooksLikeCollectionMissingKey {
                id,
                registered_as,
                found_fingerprints,
            } => vec![
                ("id", id.clone()),
                ("registered_as", registered_as.clone()),
                ("found_fingerprints", found_fingerprints.join(", ")),
            ],
            ErrorKind::LooksLikeSectionOrGroupMissingKey {
                id,
                inferred_kind,
                registered_as,
                found_fingerprints,
            } => vec![
                ("id", id.clone()),
                ("inferred_kind", inferred_kind.clone()),
                ("registered_as", registered_as.clone()),
                ("found_fingerprints", found_fingerprints.join(", ")),
            ],
        }
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
