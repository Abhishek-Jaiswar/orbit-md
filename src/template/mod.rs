//! Page layout engine for Orbit v1.0.0.
//!
//! In v1.0.0 the Handlebars template engine has been removed. All pages are
//! rendered through the [`crate::orbit_markdown`] pipeline:
//!
//! 1. `orbit_markdown::pipeline::compile_content` — parses `:::directive`
//!    blocks and renders them to an HTML fragment.
//! 2. `orbit_markdown::layout::render_layout` — wraps the fragment in a
//!    complete `<!DOCTYPE html>` shell using the layout named in
//!    `Config::layout` or the page's `layout:` front-matter key.
//!
//! **Supported built-in layout names** (v1.0.0):
//!
//! | Name      | Description                                      |
//! |-----------|--------------------------------------------------|
//! | `default` | Single-column centred-content page               |
//! | `docs`    | Fixed sidebar + scrollable content (two-column) |
//!
//! Any other layout name causes `render_page` to return a `PageError`.

use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::config::Config;
use crate::error::{OrbitError, PageError};
use crate::models::{CompiledPage, RenderedPage};
use crate::orbit_markdown::layout::{LayoutOptions, render_layout};
use crate::orbit_markdown::theme::theme_css;

// ── TemplateEngine ────────────────────────────────────────────────────────────

/// Orbit v1.0.0 page layout engine.
///
/// Immutable and `Sync` — safe to share across rayon workers without locking.
pub struct TemplateEngine {
    site_title: String,
    default_layout: String,
    /// Theme CSS embedded at startup from the built-in theme registry.
    theme_css: String,
}

impl TemplateEngine {
    /// Builds an engine from the project configuration.
    ///
    /// # Errors
    ///
    /// Returns [`OrbitError::Config`] when `config.theme` names a theme that
    /// does not exist (currently only `"default"` is built-in).
    pub fn from_config(config: &Config) -> Result<Self, OrbitError> {
        let css = theme_css(&config.theme).unwrap_or("").to_owned();

        Ok(Self {
            site_title: config.title.clone(),
            default_layout: config.layout.clone(),
            theme_css: css,
        })
    }

    /// Compiles and wraps a page in the appropriate layout shell.
    ///
    /// The raw `body` of the source page is re-compiled through
    /// `orbit_markdown::pipeline::compile_content` so that `:::directive`
    /// blocks are processed. The legacy `content_html` field of
    /// [`CompiledPage`] is intentionally ignored — it exists only for the
    /// throughput benchmark and is safe to discard.
    ///
    /// # Errors
    ///
    /// Returns `PageError` when:
    /// - A `:::directive` in the source body is invalid (D001–D006), or
    /// - The resolved layout name is not a built-in orbit layout.
    pub fn render_page(
        &self,
        page: CompiledPage,
        output_root: &Path,
    ) -> Result<RenderedPage, PageError> {
        let layout = page
            .source
            .front_matter
            .layout
            .clone()
            .unwrap_or_else(|| self.default_layout.clone());

        let fallback_title = page
            .source
            .relative_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_owned();

        let title = page.source.front_matter.effective_title(&fallback_title);

        use crate::orbit_markdown::parser::parse;
        use crate::orbit_markdown::render::render_nodes;
        use crate::orbit_markdown::ast::OrbitNode;

        // Parse :::directive blocks into nodes.
        let mut nodes = parse(&page.source.body, &page.source.source_path)?;

        // Extract NavGroup nodes for sidebar rendering
        let mut sidebar_nodes = Vec::new();
        nodes.retain(|node| {
            if matches!(node, OrbitNode::NavGroup { .. }) {
                sidebar_nodes.push(node.clone());
                false
            } else {
                true
            }
        });

        let content_html = render_nodes(&nodes);
        let sidebar_html = if sidebar_nodes.is_empty() {
            None
        } else {
            Some(render_nodes(&sidebar_nodes))
        };

        let opts = LayoutOptions {
            page_title: &title,
            site_title: &self.site_title,
            content: &content_html,
            theme_css: &self.theme_css,
            sidebar_html: sidebar_html.as_deref(),
        };

        let html = render_layout(&layout, &opts).ok_or_else(|| {
            PageError::new(
                &page.source.source_path,
                format!(
                    "unknown layout '{layout}' — use 'default' or 'docs' (built-in), \
                     or check your orbit.yaml"
                ),
            )
        })?;

        let output_path = output_path_for(&page.source.relative_path, output_root);
        Ok(RenderedPage::new(output_path, html))
    }
}

fn output_path_for(relative: &Path, output_root: &Path) -> PathBuf {
    let mut dest = output_root.to_path_buf();
    dest.push(relative.with_extension("html"));
    dest
}

// ── Public batch API ──────────────────────────────────────────────────────────

/// Renders all compiled pages in parallel using a shared, immutable engine.
pub fn render_all(
    engine: &TemplateEngine,
    pages: Vec<CompiledPage>,
    output_root: &Path,
) -> Result<Vec<RenderedPage>, PageError> {
    pages
        .into_par_iter()
        .map(|page| engine.render_page(page, output_root))
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FrontMatter, SourcePage};
    use std::path::PathBuf;

    fn compiled_page(body: &str, layout: Option<&str>) -> CompiledPage {
        CompiledPage::new(
            SourcePage {
                source_path: PathBuf::from("content/a.md"),
                relative_path: PathBuf::from("a.md"),
                front_matter: FrontMatter {
                    title: Some("Test Page".into()),
                    layout: layout.map(str::to_owned),
                    ..Default::default()
                },
                body: body.to_owned(),
            },
            // content_html is ignored by the orbit engine — the body is
            // re-compiled from source.
            String::new(),
        )
    }

    #[test]
    fn renders_with_default_layout() {
        let config = Config::default();
        let engine = TemplateEngine::from_config(&config).unwrap();
        let page = compiled_page("# Hello\n\nParagraph.", Some("default"));

        let rendered = engine.render_page(page, Path::new("dist")).unwrap();
        assert!(rendered.html.contains("<!DOCTYPE html>"));
        assert!(rendered.html.contains("orbit-page"));
        assert!(rendered.html.contains("<h1>Hello</h1>"));
        assert!(rendered.output_path.ends_with("a.html"));
    }

    #[test]
    fn renders_with_docs_layout() {
        let config = Config::default();
        let engine = TemplateEngine::from_config(&config).unwrap();
        let page = compiled_page("# Docs page.", Some("docs"));

        let rendered = engine.render_page(page, Path::new("dist")).unwrap();
        assert!(rendered.html.contains("orbit-docs-layout"));
        assert!(rendered.html.contains("orbit-docs-sidebar"));
    }

    #[test]
    fn callout_directive_rendered_end_to_end() {
        let config = Config::default();
        let engine = TemplateEngine::from_config(&config).unwrap();
        let page = compiled_page(":::warning\nBe careful!\n:::", Some("default"));

        let rendered = engine.render_page(page, Path::new("dist")).unwrap();
        assert!(rendered.html.contains("orbit-callout--warning"));
        assert!(rendered.html.contains("Be careful!"));
    }

    #[test]
    fn unknown_layout_returns_page_error() {
        let config = Config::default();
        let engine = TemplateEngine::from_config(&config).unwrap();
        let page = compiled_page("# Hi", Some("not-a-real-layout.hbs"));

        let err = engine.render_page(page, Path::new("dist")).unwrap_err();
        assert!(
            err.message.contains("unknown layout"),
            "expected 'unknown layout' error, got: {}",
            err.message
        );
    }

    #[test]
    fn directive_error_propagates_as_page_error() {
        let config = Config::default();
        let engine = TemplateEngine::from_config(&config).unwrap();
        // D001: unknown directive
        let page = compiled_page(":::typo\ncontent\n:::", Some("default"));
        assert!(engine.render_page(page, Path::new("dist")).is_err());
    }

    #[test]
    fn title_appears_in_html_head() {
        let config = Config {
            title: "My Site".into(),
            ..Config::default()
        };
        let engine = TemplateEngine::from_config(&config).unwrap();
        let page = compiled_page("Hello.", Some("default"));
        let rendered = engine.render_page(page, Path::new("dist")).unwrap();
        assert!(rendered.html.contains("Test Page | My Site"));
    }
}
