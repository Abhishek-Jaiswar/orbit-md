//! Strongly typed YAML front matter extracted from Markdown sources.

use serde::Deserialize;

/// Metadata block at the top of a Markdown file.
///
/// All fields are optional at the serde layer so missing or partial front
/// matter never panics the compiler. Malformed YAML is reported per file.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct FrontMatter {
    /// Page title; falls back to filename stem when absent.
    #[serde(default)]
    pub title: Option<String>,

    /// Optional publication date string (opaque to the engine).
    #[serde(default)]
    pub date: Option<String>,

    /// Taxonomy tags rendered in the layout when present.
    #[serde(default)]
    pub tags: Vec<String>,

    /// When `true`, the page is skipped during compilation.
    #[serde(default)]
    pub draft: bool,

    /// Optional layout override relative to the template directory.
    #[serde(default)]
    pub layout: Option<String>,

    /// Arbitrary string metadata preserved for template helpers.
    #[serde(default)]
    pub description: Option<String>,
}

impl FrontMatter {
    /// Returns the effective title, using `fallback` when none was declared.
    pub fn effective_title(&self, fallback: &str) -> String {
        self.title
            .clone()
            .unwrap_or_else(|| fallback.to_owned())
    }
}
