//! Page lifecycle types enforcing compile-state separation.

use std::path::PathBuf;

use crate::models::FrontMatter;

/// A Markdown source file discovered on disk with parsed front matter.
///
/// This is the input to the compilation pipeline. The raw Markdown body has
/// not yet been converted to HTML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePage {
    /// Absolute or project-relative path to the source `.md` file.
    pub source_path: PathBuf,

    /// Path relative to the content root, used to derive output URLs.
    pub relative_path: PathBuf,

    /// Parsed YAML front matter (defaults applied for missing keys).
    pub front_matter: FrontMatter,

    /// Markdown body after front matter extraction.
    pub body: String,
}

/// Newtype marking a [`SourcePage`] that has not yet been parsed.
///
/// Illegal transitions (e.g. writing directly to disk) are prevented because
/// only [`CompiledPage`] and [`RenderedPage`] expose HTML payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UncompiledPage(pub SourcePage);

impl UncompiledPage {
    /// Wraps a discovered source page for the parsing stage.
    pub fn new(page: SourcePage) -> Self {
        Self(page)
    }

    /// Borrows the inner source page.
    pub fn inner(&self) -> &SourcePage {
        &self.0
    }

    /// Consumes the wrapper and returns the inner source page.
    pub fn into_inner(self) -> SourcePage {
        self.0
    }
}

/// Markdown body converted to an HTML fragment, before layout wrapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledPage {
    /// Original source metadata and paths.
    pub source: SourcePage,

    /// HTML generated from the Markdown body (no layout chrome).
    pub content_html: String,
}

impl CompiledPage {
    /// Creates a compiled page from its source and rendered fragment.
    pub fn new(source: SourcePage, content_html: String) -> Self {
        Self {
            source,
            content_html,
        }
    }
}

/// Fully rendered HTML page ready for disk output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedPage {
    /// Destination path under the output directory.
    pub output_path: PathBuf,

    /// Complete HTML document including layout template.
    pub html: String,
}

impl RenderedPage {
    /// Creates a rendered page destined for `output_path`.
    pub fn new(output_path: PathBuf, html: String) -> Self {
        Self { output_path, html }
    }
}
