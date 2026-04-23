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
