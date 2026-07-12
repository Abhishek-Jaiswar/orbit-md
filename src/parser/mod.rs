//! Pull-based Markdown-to-HTML conversion via `pulldown-cmark`.

use pulldown_cmark::{Options, Parser, html};

use crate::components::{ComponentRegistry, expand_components};
use crate::error::PageError;
use crate::models::{CompiledPage, UncompiledPage};

/// Shared pulldown-cmark options for page bodies.
fn markdown_options() -> Options {
    Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TASKLISTS
}

/// Converts Markdown source text into an HTML fragment.
pub fn markdown_to_html(body: &str) -> String {
    let parser = Parser::new_ext(body, markdown_options());
    let mut content_html = String::with_capacity(body.len().saturating_mul(2));
    html::push_html(&mut content_html, parser);
    content_html
}

/// Converts an [`UncompiledPage`] into a [`CompiledPage`] HTML fragment.
///
/// JSX-style component tags are expanded before Markdown parsing.
///
/// # Examples
///
/// ```
/// use orbit_md::components::ComponentRegistry;
/// use orbit_md::config::Config;
/// use orbit_md::models::{FrontMatter, SourcePage, UncompiledPage};
/// use orbit_md::parser::compile_markdown;
/// use std::path::PathBuf;
///
/// let registry = ComponentRegistry::from_config(&Config::default()).unwrap();
/// let page = UncompiledPage::new(SourcePage {
///     source_path: PathBuf::from("content/post.md"),
///     relative_path: PathBuf::from("post.md"),
///     front_matter: FrontMatter::default(),
///     body: "# Hello".to_owned(),
/// });
///
/// let compiled = compile_markdown(page, &registry).unwrap();
/// assert!(compiled.content_html.contains("<h1>"));
/// ```
pub fn compile_markdown(
    page: UncompiledPage,
    registry: &ComponentRegistry,
) -> Result<CompiledPage, PageError> {
    let source = page.into_inner();
    let path = source.source_path.clone();

    // NOTE: component expansion first — slots compile MD to HTML inside the registry.
    let expanded = expand_components(&source.body, registry, &path)?;

    // NOTE: Parser borrows `expanded` — no extra clone beyond this stage.
    let parser = Parser::new_ext(&expanded, markdown_options());
    let mut content_html = String::with_capacity(expanded.len().saturating_mul(2));
    html::push_html(&mut content_html, parser);

    Ok(CompiledPage::new(source, content_html))
}

/// Parallel map-reduce over all uncompiled pages.
///
/// Failures are returned as the first error encountered; callers may extend
/// this to collect all errors if needed.
pub fn compile_all(
    pages: Vec<UncompiledPage>,
    registry: &ComponentRegistry,
) -> Result<Vec<CompiledPage>, PageError> {
    use rayon::prelude::*;

    pages
        .into_par_iter()
        .map(|page| compile_markdown(page, registry))
        .collect()
}
