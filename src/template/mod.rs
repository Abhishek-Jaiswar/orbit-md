//! Pre-compiled Handlebars layout engine.

use std::fs;
use std::path::{Path, PathBuf};

use handlebars::Handlebars;
use serde::Serialize;

use crate::config::Config;
use crate::error::{OrbitError, PageError};
use crate::models::{CompiledPage, RenderedPage};

/// Template context passed to layout partials.
#[derive(Debug, Serialize)]
struct PageContext<'a> {
    title: String,
    site_title: &'a str,
    content: &'a str,
    date: Option<&'a str>,
    description: Option<&'a str>,
    tags: &'a [String],
}

/// Handlebars registry with templates loaded at startup.
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
    site_title: String,
    default_layout: String,
}

impl TemplateEngine {
    /// Loads and pre-compiles every `.hbs` file under `template_dir`.
    ///
    /// # Errors
    ///
    /// Returns [`OrbitError::Template`] when templates cannot be read or compiled.
    pub fn from_config(config: &Config) -> Result<Self, OrbitError> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        if config.template_dir.exists() {
            for entry in fs::read_dir(&config.template_dir).map_err(|source| OrbitError::Io {
                path: config.template_dir.clone(),
                source,
            })? {
                let entry = entry.map_err(|source| OrbitError::Io {
                    path: config.template_dir.clone(),
                    source,
                })?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "hbs") {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .ok_or_else(|| OrbitError::Template("invalid template filename".into()))?;
                    let source = fs::read_to_string(&path).map_err(|source| OrbitError::Io {
                        path: path.clone(),
                        source,
                    })?;
                    handlebars
                        .register_template_string(name, source)
                        .map_err(|err| OrbitError::Template(err.to_string()))?;
                }
            }
        }

        Ok(Self {
            handlebars,
            site_title: config.title.clone(),
            default_layout: config.layout.clone(),
        })
    }

    /// Wraps a compiled page fragment in the site layout.
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

        let context = PageContext {
            title: title.clone(),
            site_title: &self.site_title,
            content: &page.content_html,
            date: page.source.front_matter.date.as_deref(),
            description: page.source.front_matter.description.as_deref(),
            tags: &page.source.front_matter.tags,
        };

        let html = self.handlebars.render(&layout, &context).map_err(|err| {
            PageError::new(
                &page.source.source_path,
                format!("template render failed: {err}"),
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

use rayon::prelude::*;

/// Renders all compiled pages in parallel using a shared, immutable engine.
///
/// The engine is `Sync` and borrowed immutably — no mutex on the hot path.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FrontMatter, SourcePage};
    use std::path::PathBuf;

    #[test]
    fn renders_with_default_layout() {
        let mut config = Config::default();
        let dir = tempfile::tempdir().unwrap();
        config.template_dir = dir.path().to_path_buf();
        config.layout = "base.hbs".to_owned();
        std::fs::write(
            dir.path().join("base.hbs"),
            "<html><title>{{title}}</title><body>{{{content}}}</body></html>",
        )
        .unwrap();

        let engine = TemplateEngine::from_config(&config).unwrap();
        let compiled = CompiledPage::new(
            SourcePage {
                source_path: PathBuf::from("content/a.md"),
                relative_path: PathBuf::from("a.md"),
                front_matter: FrontMatter {
                    title: Some("Page".into()),
                    ..Default::default()
                },
                body: String::new(),
            },
            "<p>Hi</p>".to_owned(),
        );

        let rendered = engine.render_page(compiled, Path::new("dist")).unwrap();
        assert!(rendered.html.contains("<p>Hi</p>"));
        assert!(rendered.output_path.ends_with("a.html"));
    }
}
