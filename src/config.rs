//! Site-wide configuration loaded from YAML at startup.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::OrbitError;

/// Default config filename for Orbit projects.
pub const DEFAULT_CONFIG_FILE: &str = "orbit.yaml";

/// Global site configuration.
///
/// Values may be supplied via [`DEFAULT_CONFIG_FILE`] in the project root. Any
/// omitted field falls back to a sensible default so local development requires
/// minimal setup.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Human-readable site title injected into layout templates.
    #[serde(default = "default_title")]
    pub title: String,

    /// Root directory containing Markdown source files.
    #[serde(default = "default_source_dir")]
    pub source_dir: PathBuf,

    /// Output directory for generated HTML (typically `.orbit/`).
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    /// Directory holding Handlebars layout templates.
    #[serde(default = "default_template_dir")]
    pub template_dir: PathBuf,

    /// Base layout template filename inside [`Config::template_dir`].
    #[serde(default = "default_layout")]
    pub layout: String,

    /// Directory holding reusable Markdown components (`*.hbs`).
    #[serde(default = "default_components_dir")]
    pub components_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: default_title(),
            source_dir: default_source_dir(),
            output_dir: default_output_dir(),
            template_dir: default_template_dir(),
            layout: default_layout(),
            components_dir: default_components_dir(),
        }
    }
}

impl Config {
    /// Loads configuration from `path`, falling back to defaults when the file
    /// is absent.
    ///
    /// # Errors
    ///
    /// Returns [`OrbitError::Config`] when the file exists but cannot be parsed.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, OrbitError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(path).map_err(|source| OrbitError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        serde_yaml::from_str(&contents)
            .map_err(|err| OrbitError::Config(format!("failed to parse {}: {err}", path.display())))
    }
}

fn default_title() -> String {
    "My Site".to_owned()
}

fn default_source_dir() -> PathBuf {
    PathBuf::from("content")
}

fn default_output_dir() -> PathBuf {
    PathBuf::from(".orbit")
}

fn default_template_dir() -> PathBuf {
    PathBuf::from("templates")
}

fn default_layout() -> String {
    "base.hbs".to_owned()
}

fn default_components_dir() -> PathBuf {
    PathBuf::from("components")
}
