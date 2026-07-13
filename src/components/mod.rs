//! Component layer — superseded in v1.0.0 by `:::directive` blocks.
//!
//! In v1.0.0, Markdown components (JSX-style `.hbs` templates) are replaced
//! by the directive parser in [`crate::orbit_markdown`]. This module is kept
//! as a compatibility shim so the public API compiles and existing callers
//! (`parser::compile_all`, `lib::build`) require no changes.
//!
//! The `ComponentRegistry::from_config` constructor always succeeds.
//! `expand_components` returns the source string unchanged.

use std::path::Path;

use crate::config::Config;
use crate::error::{OrbitError, PageError};

/// No-op component registry.
///
/// In v1.0.0, JSX-style `.hbs` components are superseded by `:::directive`
/// blocks processed by [`crate::orbit_markdown::pipeline`]. This registry
/// does nothing but is kept for API compatibility.
#[derive(Debug, Default, Clone)]
pub struct ComponentRegistry;

impl ComponentRegistry {
    /// Always succeeds — no templates are loaded in v1.0.0.
    pub fn from_config(_config: &Config) -> Result<Self, OrbitError> {
        Ok(Self)
    }
}

/// No-op expansion — returns `body` unchanged.
///
/// In v1.0.0, all content transformations (callouts, cards, steps, hero, …)
/// are handled by the orbit directive parser, not by Handlebars templates.
/// This function is retained so that `parser::compile_markdown` compiles
/// without changes.
pub fn expand_components(
    body: &str,
    _registry: &ComponentRegistry,
    _path: &Path,
) -> Result<String, PageError> {
    Ok(body.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn registry_always_created() {
        let registry = ComponentRegistry::from_config(&Config::default()).unwrap();
        let _ = registry;
    }

    #[test]
    fn expand_components_is_a_no_op() {
        let body = "# Hello\n\n:::note\nOrbit directives, not HBS.\n:::";
        let registry = ComponentRegistry::default();
        let out = expand_components(body, &registry, Path::new("test.md")).unwrap();
        assert_eq!(out, body, "expand_components must return source unchanged");
    }
}
