//! React-like Markdown component layer.
//!
//! Authors invoke reusable UI pieces with JSX-style tags inside `.md` files.
//! Components are Handlebars templates in `components/*.hbs`.
//!
//! # Island contract (future hydration)
//!
//! When a component tag includes `client="load"`, the rendered HTML is wrapped:
//!
//! ```html
//! <div data-orbit-island="Counter" data-props='{"initial":"0"}'>
//!   <!-- static fallback from Counter.hbs -->
//! </div>
//! ```
//!
//! A future `/islands/loader.js` can read these attributes and mount widgets.

mod parser;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use handlebars::Handlebars;
use pulldown_cmark::{Options, Parser, html};
use serde_json::{Map, Value};

pub use parser::{ParsedComponent, find_next_component, is_component_name};

use crate::config::Config;
use crate::error::{OrbitError, PageError};

/// Pre-compiled Handlebars templates for Markdown components.
///
/// Immutable and `Sync` — shared across rayon workers without locking.
pub struct ComponentRegistry {
    handlebars: Handlebars<'static>,
    template_names: Vec<String>,
}

impl ComponentRegistry {
    /// Loads every `.hbs` file from `config.components_dir`.
    ///
    /// # Errors
    ///
    /// Returns [`OrbitError::Template`] when templates cannot be read or compiled.
    pub fn from_config(config: &Config) -> Result<Self, OrbitError> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        let mut template_names = Vec::new();

        if config.components_dir.exists() {
            for entry in fs::read_dir(&config.components_dir).map_err(|source| OrbitError::Io {
                path: config.components_dir.clone(),
                source,
            })? {
                let entry = entry.map_err(|source| OrbitError::Io {
                    path: config.components_dir.clone(),
                    source,
                })?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "hbs") {
                    let file_stem = path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .ok_or_else(|| OrbitError::Template("invalid component filename".into()))?;
                    let source = fs::read_to_string(&path).map_err(|source| OrbitError::Io {
                        path: path.clone(),
                        source,
                    })?;
                    handlebars
                        .register_template_string(file_stem, source)
                        .map_err(|err| OrbitError::Template(err.to_string()))?;
                    template_names.push(file_stem.to_owned());
                }
            }
        }

        Ok(Self {
            handlebars,
            template_names,
        })
    }

    /// Returns the names of loaded component templates.
    pub fn template_names(&self) -> &[String] {
        &self.template_names
    }

    /// Returns `true` when a component template is registered.
    pub fn has_component(&self, name: &str) -> bool {
        self.handlebars.has_template(name)
    }

    /// Renders a component with props and pre-compiled children HTML.
    pub fn render(
        &self,
        name: &str,
        attrs: &HashMap<String, String>,
        children_html: &str,
        path: &Path,
    ) -> Result<String, PageError> {
        if !self.has_component(name) {
            return Err(PageError::new(
                path,
                format!("unknown component `{name}` — expected components/{name}.hbs"),
            ));
        }

        let mut context = Map::new();
        for (key, value) in attrs {
            context.insert(key.clone(), Value::String(value.clone()));
        }
        context.insert(
            "children".to_owned(),
            Value::String(children_html.to_owned()),
        );

        let html = self
            .handlebars
            .render(name, &Value::Object(context))
            .map_err(|err| {
                PageError::new(path, format!("component `{name}` render failed: {err}"))
            })?;

        if attrs.contains_key("client") {
            Ok(wrap_island(name, attrs, &html))
        } else {
            Ok(html)
        }
    }
}

/// Expands JSX-style component tags in `body` into HTML fragments.
///
/// Nested components are expanded inside-out. Slot Markdown is compiled to
/// HTML before being passed as the `children` prop.
///
/// # Examples
///
/// ```no_run
/// use orbit_md::components::{ComponentRegistry, expand_components};
/// use orbit_md::config::Config;
/// use std::path::Path;
///
/// let config = Config::default();
/// let registry = ComponentRegistry::from_config(&config).unwrap();
/// let body = r#"<Alert type="info">Hello</Alert>"#;
/// let expanded = expand_components(body, &registry, Path::new("page.md")).unwrap();
/// assert!(expanded.contains("alert"));
/// ```
pub fn expand_components(
    body: &str,
    registry: &ComponentRegistry,
    path: &Path,
) -> Result<String, PageError> {
    expand_fragment(body, registry, path)
}

fn expand_fragment(
    body: &str,
    registry: &ComponentRegistry,
    path: &Path,
) -> Result<String, PageError> {
    let mut output = String::with_capacity(body.len());
    let mut cursor = 0;

    while cursor < body.len() {
        let Some(component) = find_next_component(body, cursor) else {
            output.push_str(&body[cursor..]);
            break;
        };

        output.push_str(&body[cursor..component.span.start]);
        parser::validate_component(&component, path)?;

        let children_md = if component.self_closing {
            String::new()
        } else {
            expand_fragment(&component.inner, registry, path)?
        };

        let children_html = markdown_fragment_to_html(&children_md);
        let rendered = registry.render(&component.name, &component.attrs, &children_html, path)?;
        output.push_str(&rendered);
        cursor = component.span.end;
    }

    Ok(output)
}

/// Compiles a Markdown fragment to an HTML string for component slots.
pub fn markdown_fragment_to_html(markdown: &str) -> String {
    if markdown.is_empty() {
        return String::new();
    }

    // NOTE: keep options aligned with the main parser for consistent output.
    let options = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(markdown, options);
    let mut html = String::with_capacity(markdown.len().saturating_mul(2));
    html::push_html(&mut html, parser);
    html
}

/// Wraps rendered HTML in the island contract for future client hydration.
fn wrap_island(name: &str, attrs: &HashMap<String, String>, html: &str) -> String {
    let mut props = Map::new();
    for (key, value) in attrs {
        if key == "client" {
            continue;
        }
        props.insert(key.clone(), Value::String(value.clone()));
    }

    let props_json =
        serde_json::to_string(&Value::Object(props)).unwrap_or_else(|_| "{}".to_owned());

    format!(r#"<div data-orbit-island="{name}" data-props='{props_json}'>{html}</div>"#)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_registry(dir: &Path) -> ComponentRegistry {
        let config = Config {
            components_dir: dir.to_path_buf(),
            ..Config::default()
        };
        ComponentRegistry::from_config(&config).unwrap()
    }

    fn write_component(dir: &Path, name: &str, source: &str) {
        std::fs::write(dir.join(format!("{name}.hbs")), source).unwrap();
    }

    #[test]
    fn expands_block_and_self_closing_components() {
        let dir = tempfile::tempdir().unwrap();
        write_component(
            dir.path(),
            "Alert",
            r#"<div class="alert alert-{{type}}">{{#if title}}<strong>{{title}}</strong>{{/if}}{{{children}}}</div>"#,
        );
        write_component(
            dir.path(),
            "Button",
            r#"<a class="btn" href="{{href}}">{{label}}</a>"#,
        );

        let registry = test_registry(dir.path());
        let body = r#"<Alert type="info" title="Hi">Hello **world**</Alert>

<Button href="/docs" label="Go" />"#;

        let expanded = expand_components(body, &registry, Path::new("test.md")).unwrap();
        assert!(expanded.contains(r#"class="alert alert-info""#));
        assert!(expanded.contains("<strong>Hi</strong>"));
        assert!(expanded.contains("<strong>world</strong>"));
        assert!(expanded.contains(r#"class="btn" href="/docs""#));
    }

    #[test]
    fn island_wrapper_adds_data_attributes() {
        let dir = tempfile::tempdir().unwrap();
        write_component(
            dir.path(),
            "Counter",
            r#"<span class="counter">{{initial}}</span>"#,
        );

        let registry = test_registry(dir.path());
        let body = r#"<Counter client="load" initial="0" />"#;
        let expanded = expand_components(body, &registry, Path::new("test.md")).unwrap();

        assert!(expanded.contains(r#"data-orbit-island="Counter""#));
        assert!(expanded.contains(r#"data-props='{"initial":"0"}'"#));
        assert!(expanded.contains(r#"<span class="counter">0</span>"#));
    }

    #[test]
    fn unknown_component_returns_page_error() {
        let dir = tempfile::tempdir().unwrap();
        let registry = test_registry(dir.path());
        let result = expand_components("<Missing />", &registry, Path::new("test.md"));
        assert!(result.is_err());
    }
}
