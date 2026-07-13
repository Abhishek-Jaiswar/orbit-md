//! End-to-end Orbit Markdown compilation pipeline.
//!
//! This module wires [`parser::parse`] and [`render::render_nodes`] into a
//! single function that converts a raw Orbit Markdown body into an HTML
//! fragment, ready to be embedded in a page layout.
//!
//! The pipeline is *content-only* — it does not wrap the fragment in a
//! `<!DOCTYPE html>` shell. For that, pass the returned string to
//! [`layout::render_layout`].
//!
//! # Usage
//!
//! ```no_run
//! use std::path::Path;
//! use orbit_md::orbit_markdown::pipeline::compile_content;
//!
//! let body = "# Hello\n\n:::note\nOrbit is fast.\n:::\n";
//! let html = compile_content(body, Path::new("content/index.md")).unwrap();
//! assert!(html.contains("orbit-callout--note"));
//! ```

use std::path::Path;

use crate::error::PageError;
use crate::orbit_markdown::parser::parse;
use crate::orbit_markdown::render::render_nodes;

// ── Public API ────────────────────────────────────────────────────────────────

/// Parse an Orbit Markdown body and render it to an HTML fragment.
///
/// Plain Markdown is always valid — `parse` will emit a single
/// [`OrbitNode::Markdown`] node and `render_nodes` will pass it through
/// `pulldown-cmark` untouched.
///
/// # Errors
///
/// Returns the first structural directive error found in the source:
/// - **D001** — unknown directive name
/// - **D002** — missing required attribute
/// - **D003** — unclosed directive block at end of file
/// - **D006** — directive illegally nested inside another
///
/// The `path` argument is attached to any error for source-location reporting.
pub fn compile_content(body: &str, path: &Path) -> Result<String, PageError> {
    let nodes = parse(body, path)?;
    Ok(render_nodes(&nodes))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const DUMMY_PATH: &str = "content/test.md";

    fn path() -> &'static Path {
        Path::new(DUMMY_PATH)
    }

    #[test]
    fn plain_markdown_passes_through() {
        let html = compile_content("# Hello\n\nParagraph.", path()).unwrap();
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>Paragraph.</p>"));
    }

    #[test]
    fn callout_directive_renders_to_html() {
        let src = ":::note\nBe aware.\n:::";
        let html = compile_content(src, path()).unwrap();
        assert!(html.contains("orbit-callout--note"));
        assert!(html.contains("Be aware."));
    }

    #[test]
    fn hero_directive_renders_to_html() {
        let src = ":::hero title=\"Orbit\"\nFast sites.\n:::";
        let html = compile_content(src, path()).unwrap();
        assert!(html.contains("orbit-hero"));
        assert!(html.contains("Orbit"));
    }

    #[test]
    fn unknown_directive_returns_error() {
        let src = ":::typo\nContent.\n:::";
        let err = compile_content(src, path()).unwrap_err();
        // D001 message: "unknown directive 'typo'"
        assert!(err.message.contains("unknown directive"));
    }

    #[test]
    fn unclosed_directive_returns_error() {
        let src = ":::note\nNo closing marker.";
        let err = compile_content(src, path()).unwrap_err();
        // D003 message: "directive 'note' opened on line 1 was never closed"
        assert!(err.message.contains("never closed"));
    }

    #[test]
    fn directives_inside_code_fence_are_ignored() {
        let src = "```\n:::note\nshould be ignored\n:::\n```";
        let html = compile_content(src, path()).unwrap();
        // Should have a code block, not an orbit callout.
        assert!(!html.contains("orbit-callout"));
        assert!(html.contains("<code>"));
    }

    #[test]
    fn markdown_before_and_after_directive_is_included() {
        let src = "Intro.\n\n:::note\nBody.\n:::\n\nOutro.";
        let html = compile_content(src, path()).unwrap();
        assert!(html.contains("Intro."));
        assert!(html.contains("orbit-callout--note"));
        assert!(html.contains("Outro."));
    }

    #[test]
    fn error_carries_source_path() {
        let src = ":::oops\n:::";
        let err = compile_content(src, Path::new("content/my-page.md")).unwrap_err();
        assert!(
            err.path.ends_with("my-page.md"),
            "path not attached: {:?}",
            err.path
        );
    }
}
