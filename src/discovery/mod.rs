//! Parallel filesystem discovery and front matter extraction.

use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::error::PageError;
use crate::models::{FrontMatter, SourcePage, UncompiledPage};

/// Discovers Markdown files under `root` and loads them in parallel.
///
/// Each file is read independently; failures are collected without stopping
/// siblings. Draft pages and non-`.md` files are skipped.
///
/// # Examples
///
/// ```no_run
/// use orbit_md::discovery::discover_pages;
/// use std::path::Path;
///
/// let pages = discover_pages(Path::new("content")).expect("discovery ok");
/// assert!(!pages.is_empty());
/// ```
pub fn discover_pages(root: &Path) -> Result<Vec<UncompiledPage>, PageError> {
    // NOTE: collect paths sequentially — metadata only, no file contents yet.
    let markdown_paths: Vec<PathBuf> = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        })
        .map(|entry| entry.into_path())
        .collect();

    // NOTE: rayon parallel map — each task owns its file buffer; no locks.
    let results: Vec<Result<UncompiledPage, PageError>> = markdown_paths
        .par_iter()
        .map(|path| load_page(root, path))
        .collect();

    let mut pages = Vec::with_capacity(results.len());
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(page) if !page.inner().front_matter.draft => pages.push(page),
            Ok(_) => {}
            Err(err) => errors.push(err),
        }
    }

    if let Some(first) = errors.into_iter().next() {
        return Err(first);
    }

    Ok(pages)
}

fn load_page(content_root: &Path, path: &Path) -> Result<UncompiledPage, PageError> {
    let raw = fs::read_to_string(path).map_err(|err| {
        PageError::new(path, format!("failed to read source file: {err}"))
    })?;

    let relative_path = path.strip_prefix(content_root).map_err(|_| {
        PageError::new(path, "source file is outside the content root")
    })?;

    let (front_matter, body) = split_front_matter(path, &raw)?;

    Ok(UncompiledPage::new(SourcePage {
        source_path: path.to_path_buf(),
        relative_path: relative_path.to_path_buf(),
        front_matter,
        body,
    }))
}

/// Splits optional YAML front matter from the Markdown body.
///
/// Malformed YAML returns a per-file error instead of aborting the build.
///
/// # Examples
///
/// ```
/// use orbit_md::discovery::split_front_matter;
/// use std::path::Path;
///
/// let input = "---\ntitle: Hello\n---\n\n# World\n";
/// let (fm, body) = split_front_matter(Path::new("post.md"), input).unwrap();
/// assert_eq!(fm.title.as_deref(), Some("Hello"));
/// assert!(body.contains("# World"));
/// ```
pub fn split_front_matter(
    path: &Path,
    contents: &str,
) -> Result<(FrontMatter, String), PageError> {
    let trimmed = contents.trim_start();
    if !trimmed.starts_with("---") {
        return Ok((FrontMatter::default(), contents.to_owned()));
    }

    let after_open = trimmed.strip_prefix("---").unwrap_or(trimmed);
    let Some(yaml_end) = after_open.find("\n---") else {
        return Ok((FrontMatter::default(), contents.to_owned()));
    };

    let yaml = &after_open[..yaml_end];
    let body_start = yaml_end + 4; // `\n---`
    let body = after_open
        .get(body_start..)
        .unwrap_or("")
        .trim_start_matches('\n')
        .trim_start()
        .to_owned();

    match serde_yaml::from_str::<FrontMatter>(yaml) {
        Ok(front_matter) => Ok((front_matter, body)),
        Err(err) => Err(PageError::new(
            path,
            format!("malformed front matter YAML: {err}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_front_matter_parses_yaml_block() {
        let input = "---\ntitle: Test\ntags:\n  - rust\n---\n\nBody";
        let (fm, body) = split_front_matter(Path::new("x.md"), input).unwrap();
        assert_eq!(fm.title.as_deref(), Some("Test"));
        assert_eq!(fm.tags, vec!["rust"]);
        assert_eq!(body, "Body");
    }

    #[test]
    fn split_front_matter_without_delimiters() {
        let (fm, body) = split_front_matter(Path::new("x.md"), "# Hi").unwrap();
        assert_eq!(fm, FrontMatter::default());
        assert_eq!(body, "# Hi");
    }
}
