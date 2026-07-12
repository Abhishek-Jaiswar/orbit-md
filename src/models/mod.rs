//! Data models for configuration, front matter, and page lifecycle states.

mod front_matter;
mod page;

pub use front_matter::FrontMatter;
pub use page::{CompiledPage, RenderedPage, SourcePage, UncompiledPage};
