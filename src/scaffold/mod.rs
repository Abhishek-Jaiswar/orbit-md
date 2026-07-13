//! Project scaffolding for `orbit init`.
//!
//! Templates are embedded at compile time from the `scaffold/` directory so
//! the CLI works after `cargo install orbit-md` with no external files required.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::OrbitError;

/// A file written during project initialization.
struct ScaffoldFile {
    relative_path: &'static str,
    contents: &'static str,
}

const SCAFFOLD_FILES: &[ScaffoldFile] = &[
    ScaffoldFile {
        relative_path: "orbit.yaml",
        contents: include_str!("../../scaffold/orbit.yaml"),
    },
    ScaffoldFile {
        relative_path: ".gitignore",
        contents: include_str!("../../scaffold/.gitignore"),
    },
    ScaffoldFile {
        relative_path: "content/index.md",
        contents: include_str!("../../scaffold/content/index.md"),
    },
    ScaffoldFile {
        relative_path: "content/docs/getting-started.md",
        contents: include_str!("../../scaffold/content/docs/getting-started.md"),
    },
];

/// Options for creating a new site project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitOptions {
    /// Destination directory for the new project.
    pub path: PathBuf,
    /// Human-readable site title injected into config and pages.
    pub title: String,
}

impl InitOptions {
    /// Creates init options from a directory path, deriving the title from the folder name.
    pub fn from_path(path: PathBuf) -> Self {
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(title_from_slug)
            .unwrap_or_else(|| "My Site".to_owned());

        Self { path, title }
    }
}

/// Initializes a new Orbit project at `options.path`.
///
/// # Errors
///
/// Returns [`OrbitError::Config`] when the destination already exists and is not empty.
pub fn init_project(options: &InitOptions) -> Result<(), OrbitError> {
    let dest = &options.path;

    if dest.exists() {
        let mut entries = fs::read_dir(dest).map_err(|source| OrbitError::Io {
            path: dest.clone(),
            source,
        })?;
        if entries.next().is_some() {
            return Err(OrbitError::Config(format!(
                "directory {} is not empty — choose an empty folder or a new name",
                dest.display()
            )));
        }
    } else {
        fs::create_dir_all(dest).map_err(|source| OrbitError::Io {
            path: dest.clone(),
            source,
        })?;
    }

    for file in SCAFFOLD_FILES {
        let target = dest.join(file.relative_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| OrbitError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let contents = substitute_placeholders(file.contents, &options.title);
        fs::write(&target, contents).map_err(|source| OrbitError::Io {
            path: target,
            source,
        })?;
    }

    Ok(())
}

/// Creates a new Markdown page under `content/` with starter front matter.
pub fn new_page(content_root: &Path, relative: &Path, title: &str) -> Result<PathBuf, OrbitError> {
    let normalized = normalize_page_path(relative);
    let target = content_root.join(&normalized);

    if target.exists() {
        return Err(OrbitError::Config(format!(
            "page already exists at {}",
            target.display()
        )));
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|source| OrbitError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let stem = normalized
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled");

    let body = format!(
        "---\ntitle: {title}\n---\n\n# {heading}\n\nWrite your content here.\n",
        title = title,
        heading = stem
    );

    fs::write(&target, body).map_err(|source| OrbitError::Io {
        path: target.clone(),
        source,
    })?;

    Ok(target)
}

fn substitute_placeholders(input: &str, title: &str) -> String {
    input.replace("{{title}}", title)
}

/// Derives a human-readable title from a slug-like name.
pub fn title_from_slug(slug: &str) -> String {
    slug.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_page_path(relative: &Path) -> PathBuf {
    let mut path = relative.to_path_buf();
    if path.extension().is_none() {
        path.set_extension("md");
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_from_slug_formats_name() {
        assert_eq!(title_from_slug("my-blog"), "My Blog");
        assert_eq!(title_from_slug("docs_site"), "Docs Site");
    }

    #[test]
    fn init_creates_project_tree() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("demo-site");
        init_project(&InitOptions {
            path: path.clone(),
            title: "Demo Site".into(),
        })
        .unwrap();

        assert!(path.join("orbit.yaml").exists());
        assert!(path.join("content/index.md").exists());

        let config = std::fs::read_to_string(path.join("orbit.yaml")).unwrap();
        assert!(config.contains("title: Demo Site"));
    }

    #[test]
    fn init_rejects_nonempty_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("existing.txt"), "data").unwrap();
        let result = init_project(&InitOptions {
            path: dir.path().to_path_buf(),
            title: "X".into(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn new_page_adds_md_extension() {
        let dir = tempfile::tempdir().unwrap();
        let content = dir.path().join("content");
        std::fs::create_dir_all(&content).unwrap();

        let created = new_page(&content, Path::new("blog/post"), "My Post").unwrap();
        assert_eq!(created, content.join("blog/post.md"));
        assert!(created.exists());
    }
}
