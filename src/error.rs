//! Domain-specific error types for Orbit.
//!
//! All recoverable per-file failures are represented as [`PageError`] so the
//! parallel pipeline can skip bad inputs without aborting the entire build.

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
    #[error("page error at {path}: {message}")]
    Page {
        /// Source file associated with the failure.
        path: PathBuf,
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

/// Per-page failure surfaced during parallel compilation.
#[derive(Debug, Error)]
#[error("{message}")]
pub struct PageError {
    /// Source path for diagnostics.
    pub path: PathBuf,
    /// Failure description.
    pub message: String,
}

impl PageError {
    /// Creates a new page-scoped error.
    pub fn new(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl From<PageError> for OrbitError {
    fn from(value: PageError) -> Self {
        Self::Page {
            path: value.path,
            message: value.message,
        }
    }
}
