//! Domain-specific error types for Orbit.
//!
//! All recoverable per-file failures are represented as [`PageError`] so the
//! parallel pipeline can skip bad inputs without aborting the entire build.
//!
//! The directive compiler produces [`PageError`] values with line and column
//! information via [`PageError::at`]. Legacy pipeline errors use [`PageError::new`]
//! and leave location fields as `None`.

use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur while building the site.
#[derive(Debug, Error)]
pub enum OrbitError {
    /// Failed to read or write a file on disk.
    #[error("I/O error at {path}: {source}")]
    Io {
        /// Path involved in the failed operation.
        path: PathBuf,
        /// Underlying OS error.
        #[source]
        source: std::io::Error,
    },

    /// Site configuration could not be loaded or validated.
    #[error("configuration error: {0}")]
    Config(String),

    /// A single page failed to compile; other pages may still succeed.
    #[error("{}", _format_page_error(path, *line, *column, message))]
    Page {
        /// Source file associated with the failure.
        path: PathBuf,
        /// 1-indexed source line, when known.
        line: Option<usize>,
        /// 1-indexed source column, when known.
        column: Option<usize>,
        /// Human-readable failure reason.
        message: String,
    },

    /// Template engine failure.
    #[error("template error: {0}")]
    Template(String),

    /// Front matter could not be deserialized.
    #[error("front matter parse error at {path}: {source}")]
    FrontMatter {
        /// Source file whose front matter failed to parse.
        path: PathBuf,
        /// Underlying serde error.
        #[source]
        source: serde_yaml::Error,
    },
}

/// Formats a page error location string for display.
fn _format_page_error(
    path: &PathBuf,
    line: Option<usize>,
    column: Option<usize>,
    message: &str,
) -> String {
    match (line, column) {
        (Some(l), Some(c)) => format!("page error at {}:{l}:{c} — {message}", path.display()),
        (Some(l), None) => format!("page error at {}:{l} — {message}", path.display()),
        _ => format!("page error at {} — {message}", path.display()),
    }
}

/// Per-page failure surfaced during parallel compilation.
///
/// Use [`PageError::new`] for errors without source location (legacy pipeline).
/// Use [`PageError::at`] when the directive parser can provide a line and column.
#[derive(Debug, Error)]
#[error("{}", _format_page_error(&self.path, self.line, self.column, &self.message))]
pub struct PageError {
    /// Source path for diagnostics.
    pub path: PathBuf,
    /// 1-indexed source line where the error occurred, if known.
    pub line: Option<usize>,
    /// 1-indexed source column where the error occurred, if known.
    pub column: Option<usize>,
    /// Human-readable failure description.
    pub message: String,
}

impl PageError {
    /// Creates a page-scoped error without source location.
    ///
    /// Use this for errors coming from the legacy pipeline (discovery,
    /// Handlebars rendering) where line information is not available.
    pub fn new(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            line: None,
            column: None,
            message: message.into(),
        }
    }

    /// Creates a page-scoped error with a precise source location.
    ///
    /// Use this in the directive parser where every error maps to a specific
    /// line (and optionally column) in the source file.
    ///
    /// # Example
    ///
    /// ```
    /// use orbit_md::error::PageError;
    /// use std::path::Path;
    ///
    /// let err = PageError::at(Path::new("content/index.md"), 14, 1, "unclosed directive 'note'");
    /// assert_eq!(err.line, Some(14));
    /// assert_eq!(err.column, Some(1));
    /// ```
    pub fn at(
        path: impl Into<PathBuf>,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            line: Some(line),
            column: Some(column),
            message: message.into(),
        }
    }
}

impl From<PageError> for OrbitError {
    fn from(value: PageError) -> Self {
        Self::Page {
            path: value.path,
            line: value.line,
            column: value.column,
            message: value.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn page_error_new_has_no_location() {
        let err = PageError::new(Path::new("a.md"), "bad");
        assert!(err.line.is_none());
        assert!(err.column.is_none());
        assert!(err.to_string().contains("a.md"));
        assert!(err.to_string().contains("bad"));
    }

    #[test]
    fn page_error_at_carries_line_and_column() {
        let err = PageError::at(
            Path::new("content/index.md"),
            14,
            3,
            "unclosed directive 'note'",
        );
        assert_eq!(err.line, Some(14));
        assert_eq!(err.column, Some(3));
        assert!(err.to_string().contains("14:3"));
        assert!(err.to_string().contains("unclosed directive"));
    }

    #[test]
    fn page_error_converts_to_orbit_error() {
        let page_err = PageError::at(Path::new("p.md"), 5, 1, "oops");
        let orbit_err = OrbitError::from(page_err);
        match orbit_err {
            OrbitError::Page { line, column, .. } => {
                assert_eq!(line, Some(5));
                assert_eq!(column, Some(1));
            }
            _ => panic!("expected OrbitError::Page"),
        }
    }
}
