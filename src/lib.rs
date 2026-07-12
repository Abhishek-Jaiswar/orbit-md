//! Orbit — high-speed parallel static site generator.
//!
//! Markdown pages with JSX-style components, compiled to static HTML.
//!
//! Architecture:
//! - [`discovery`] — parallel filesystem crawl + front matter extraction
//! - [`components`] — JSX-style Markdown component expansion
//! - [`parser`] — `pulldown-cmark` HTML fragment generation
//! - [`template`] — pre-compiled Handlebars layouts
//! - [`writer`] — parallel flush to `.orbit/`

#![deny(unsafe_code)]

pub mod cli;
pub mod components;
pub mod config;
pub mod dev;
pub mod discovery;
pub mod error;
pub mod models;
pub mod parser;
pub mod scaffold;
pub mod template;
pub mod writer;

use std::path::Path;
use std::time::Instant;

use rayon::prelude::*;

use crate::components::ComponentRegistry;
use crate::config::Config;
use crate::discovery::discover_pages;
use crate::error::{OrbitError, PageError};
use crate::models::RenderedPage;
use crate::parser::{compile_all, compile_markdown};
use crate::template::{TemplateEngine, render_all};
use crate::writer::{clean_output_dir, write_all};

/// Build statistics returned after a successful compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildReport {
    /// Number of pages written to disk.
    pub pages_written: usize,
    /// Wall-clock duration of the full pipeline.
    pub elapsed_ms: u128,
}

/// Runs the full discover → compile → render → write pipeline.
///
/// # Errors
///
/// Returns the first per-page or configuration error encountered.
pub fn build(config: &Config) -> Result<BuildReport, OrbitError> {
    let started = Instant::now();

    let components = ComponentRegistry::from_config(config)?;
    let engine = TemplateEngine::from_config(config)?;
    clean_output_dir(&config.output_dir).map_err(OrbitError::from)?;

    let uncompiled = discover_pages(&config.source_dir).map_err(OrbitError::from)?;
    let compiled = compile_all(uncompiled, &components).map_err(OrbitError::from)?;
    let rendered = render_all(&engine, compiled, &config.output_dir).map_err(OrbitError::from)?;

    let count = rendered.len();
    write_all(&rendered).map_err(OrbitError::from)?;

    Ok(BuildReport {
        pages_written: count,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

/// Map-reduce pipeline exposing each stage for testing and benchmarking.
///
/// Accepts pre-discovered pages and returns rendered output without writing.
pub fn compile_pipeline(
    config: &Config,
    content_root: &Path,
) -> Result<Vec<RenderedPage>, PageError> {
    let components = ComponentRegistry::from_config(config)
        .map_err(|err| PageError::new(content_root, err.to_string()))?;
    let engine = TemplateEngine::from_config(config)
        .map_err(|err| PageError::new(content_root, err.to_string()))?;

    let uncompiled = discover_pages(content_root)?;
    let compiled = compile_all(uncompiled, &components)?;
    render_all(&engine, compiled, &config.output_dir)
}

/// Parallel markdown-only compilation used in throughput benchmarks.
pub fn compile_markdown_batch(
    pages: Vec<crate::models::UncompiledPage>,
    registry: &ComponentRegistry,
) -> Result<usize, PageError> {
    let count = pages
        .into_par_iter()
        .map(|page| compile_markdown(page, registry))
        .collect::<Result<Vec<_>, _>>()?
        .len();
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FrontMatter, SourcePage, UncompiledPage};
    use std::path::PathBuf;

    #[test]
    fn compile_markdown_batch_parallel() {
        let registry = ComponentRegistry::from_config(&Config::default()).unwrap();
        let pages: Vec<_> = (0..32)
            .map(|i| {
                UncompiledPage::new(SourcePage {
                    source_path: PathBuf::from(format!("content/{i}.md")),
                    relative_path: PathBuf::from(format!("{i}.md")),
                    front_matter: FrontMatter::default(),
                    body: format!("# Post {i}\n\nParagraph."),
                })
            })
            .collect();

        let count = compile_markdown_batch(pages, &registry).unwrap();
        assert_eq!(count, 32);
    }
}
