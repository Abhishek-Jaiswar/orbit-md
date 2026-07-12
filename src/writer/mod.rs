//! Parallel disk writer for rendered HTML pages.

use std::fs;
use std::path::Path;

use rayon::prelude::*;

use crate::error::PageError;
use crate::models::RenderedPage;

/// Writes all rendered pages to disk in parallel.
///
/// Parent directories are created as needed. Each write is independent, so
/// no global lock serializes the hot path.
pub fn write_all(pages: &[RenderedPage]) -> Result<(), PageError> {
    pages.par_iter().try_for_each(write_one)
}

fn write_one(page: &RenderedPage) -> Result<(), PageError> {
    if let Some(parent) = page.output_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            PageError::new(&page.output_path, format!("failed to create directory: {err}"))
        })?;
    }

    // NOTE: single write call — OS page cache handles streaming; no extra buffer.
    fs::write(&page.output_path, page.html.as_bytes()).map_err(|err| {
        PageError::new(
            &page.output_path,
            format!("failed to write HTML output: {err}"),
        )
    })
}

/// Ensures the output directory exists before compilation begins.
pub fn prepare_output_dir(path: &Path) -> Result<(), PageError> {
    fs::create_dir_all(path).map_err(|err| {
        PageError::new(path, format!("failed to create output directory: {err}"))
    })
}

/// Removes and recreates the output directory for clean builds.
pub fn clean_output_dir(path: &Path) -> Result<(), PageError> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|err| {
            PageError::new(path, format!("failed to clean output directory: {err}"))
        })?;
    }
    prepare_output_dir(path)
}
