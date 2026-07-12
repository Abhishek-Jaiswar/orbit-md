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

    /// Built-in theme name for this page (e.g. `"default"`).
    ///
    /// When set, overrides the global `theme` in `orbit.yaml` for this page
    /// only. When absent, the global config theme is used.
    #[serde(default)]
    pub theme: Option<String>,

    /// Position of this page in the generated docs sidebar navigation.
    ///
    /// Pages are sorted ascending by this value. Pages without `nav_order`
    /// appear after all ordered pages, sorted alphabetically by title.
    #[serde(default)]
    pub nav_order: Option<u32>,

    /// Sidebar section this page belongs to (docs layout only).
    ///
    /// Pages sharing the same `nav_group` string are grouped under a labeled
    /// section heading in the sidebar. Pages without `nav_group` are placed
    /// in an ungrouped section at the top of the nav.
    #[serde(default)]
    pub nav_group: Option<String>,
}

impl FrontMatter {
    /// Returns the effective title, using `fallback` when none was declared.
    pub fn effective_title(&self, fallback: &str) -> String {
        self.title.clone().unwrap_or_else(|| fallback.to_owned())
    }
}
