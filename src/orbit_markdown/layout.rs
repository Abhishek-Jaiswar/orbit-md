//! Built-in HTML page shell layouts for Orbit Markdown.
//!
//! A layout wraps the rendered page content in a complete `<!DOCTYPE html>`
//! document. Two layouts ship with v1.0.0:
//!
//! | Name      | Structure                                                 |
//! |-----------|-----------------------------------------------------------|
//! | `default` | Single-column page with centred content                   |
//! | `docs`    | Two-column: fixed sidebar + scrollable content area       |
//!
//! # Usage
//!
//! ```
//! use orbit_md::orbit_markdown::layout::{render_layout, LayoutOptions};
//!
//! let opts = LayoutOptions {
//!     page_title: "Getting Started",
//!     site_title: "My Site",
//!     content: "<p>Hello.</p>",
//!     theme_css: "",
//!     sidebar_html: None,
//! };
//! let html = render_layout("default", &opts).unwrap();
//! assert!(html.contains("<!DOCTYPE html>"));
//! assert!(html.contains("Getting Started"));
//! ```

// ── Public types ──────────────────────────────────────────────────────────────

/// Input data for rendering a built-in layout.
#[derive(Debug, Clone)]
pub struct LayoutOptions<'a> {
    /// Page-level title from front matter.
    pub page_title: &'a str,
    /// Site-wide title from `orbit.yaml` — appended after `|` in `<title>`.
    pub site_title: &'a str,
    /// Rendered HTML content (body of the page).
    pub content: &'a str,
    /// CSS string to embed in `<style>`. Pass the output of `theme::theme_css`.
    /// An empty string means no `<style>` block is emitted.
    pub theme_css: &'a str,
    /// Pre-built sidebar HTML for the `docs` layout.
    ///
    /// Typically the rendered output of all `NavGroup` nodes found in the page
    /// or a project-level `_nav.md`. Ignored by the `default` layout.
    /// When `None`, the docs layout renders an empty sidebar.
    pub sidebar_html: Option<&'a str>,
}

/// The names of all built-in page layouts.
pub const BUILT_IN_LAYOUTS: &[&str] = &["default", "docs"];

// ── Public API ────────────────────────────────────────────────────────────────

/// Render a complete `<!DOCTYPE html>` page using a built-in layout.
///
/// Returns `None` when `layout_name` is not a known built-in layout name.
/// The caller should fall back to the Handlebars template pipeline in that
/// case (which handles user-defined `templates/` directories).
///
/// # Examples
///
/// ```
/// use orbit_md::orbit_markdown::layout::{render_layout, LayoutOptions};
///
/// let opts = LayoutOptions {
///     page_title: "Home",
///     site_title: "Orbit Docs",
///     content: "<h1>Welcome</h1>",
///     theme_css: "",
///     sidebar_html: None,
/// };
///
/// let html = render_layout("default", &opts).unwrap();
/// assert!(html.contains("<title>Home | Orbit Docs</title>"));
/// assert!(html.contains("<h1>Welcome</h1>"));
/// ```
pub fn render_layout(layout_name: &str, opts: &LayoutOptions<'_>) -> Option<String> {
    match layout_name {
        "default" => Some(render_default(opts)),
        "docs" => Some(render_docs(opts)),
        _ => None,
    }
}

// ── Layout implementations ────────────────────────────────────────────────────

/// Single-column centred-content layout.
fn render_default(opts: &LayoutOptions<'_>) -> String {
    let head = build_head(opts);
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
{head}
<body>
  <main class="orbit-page">
    {content}
  </main>
</body>
</html>
"#,
        content = opts.content
    )
}

/// Two-column docs layout: fixed sidebar + scrollable content area.
fn render_docs(opts: &LayoutOptions<'_>) -> String {
    let head = build_head(opts);
    let sidebar = opts.sidebar_html.unwrap_or("");
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
{head}
<body class="orbit-docs-layout">
  <aside class="orbit-docs-sidebar">
    <div class="orbit-docs-sidebar-inner">
      <a class="orbit-docs-brand" href="/">{site_title}</a>
      <div class="orbit-docs-search">
        <input type="text" class="orbit-docs-search-input" placeholder="Search Reference..." aria-label="Search Reference">
      </div>
      <nav class="orbit-docs-nav" aria-label="Documentation">
        {sidebar}
      </nav>
    </div>
  </aside>
  <div class="orbit-docs-body">
    <header class="orbit-docs-header">
      <div class="orbit-docs-header-env">
        <span class="orbit-docs-env-label">Environment:</span>
        <select class="orbit-docs-env-select">
          <option>main</option>
          <option>production</option>
        </select>
      </div>
      <div class="orbit-docs-header-meta">
        <span class="orbit-docs-meta-link"><span class="orbit-icon">🚀</span> Releases</span>
        <span class="orbit-docs-meta-link"><span class="orbit-icon">❓</span> Help</span>
        <span class="orbit-docs-meta-link"><span class="orbit-icon">🌙</span></span>
      </div>
    </header>
    <main class="orbit-docs-main" id="main-content">
      <article class="orbit-docs-content">
        {content}
      </article>
    </main>
  </div>
</body>
</html>
"#,
        site_title = escape_html(opts.site_title),
        content = opts.content
    )
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Build the `<head>` block common to both layouts.
fn build_head(opts: &LayoutOptions<'_>) -> String {
    let title = build_title(opts.page_title, opts.site_title);
    let style_block = if opts.theme_css.is_empty() {
        String::new()
    } else {
        format!("  <style>\n{}\n  </style>\n", opts.theme_css)
    };
    let layout_css = LAYOUT_CSS;

    format!(
        r#"<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title}</title>
  <style>
{layout_css}
  </style>
{style_block}</head>"#
    )
}

/// Construct the `<title>` string.
///
/// - When `page_title` equals `site_title` (e.g. home page), emit just the
///   site title.
/// - Otherwise emit `"Page Title | Site Title"`.
fn build_title(page_title: &str, site_title: &str) -> String {
    let page = escape_html(page_title);
    let site = escape_html(site_title);
    if page.is_empty() || page == site {
        site
    } else {
        format!("{page} | {site}")
    }
}

/// Escape the five special HTML characters for use in text content.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

// ── Layout structural CSS ─────────────────────────────────────────────────────

/// Minimal structural CSS for the built-in layouts.
///
/// This is separate from the theme CSS (which styles the directive elements).
/// These rules only control page-level layout geometry.
const LAYOUT_CSS: &str = r#"
    /* ── Orbit layout reset ── */
    *, *::before, *::after { box-sizing: border-box; }
    html { font-size: 16px; -webkit-text-size-adjust: 100%; }
    body {
      margin: 0;
      font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
      line-height: 1.65;
      color: #1a1a2e;
      background: #ffffff;
    }

    /* ── Default (single-column) layout ── */
    .orbit-page {
      max-width: 72ch;
      margin: 0 auto;
      padding: 2rem 1.5rem 4rem;
    }

    /* ── Docs (two-column) layout ── */
    .orbit-docs-layout {
      display: flex;
      min-height: 100vh;
    }

    .orbit-docs-sidebar {
      position: sticky;
      top: 0;
      height: 100vh;
      width: 260px;
      min-width: 260px;
      overflow-y: auto;
      border-right: 1px solid #dee2e6;
      background: #f8f9fa;
      flex-shrink: 0;
    }

    .orbit-docs-sidebar-inner {
      padding: 1.5rem 1rem 2rem;
    }

    .orbit-docs-brand {
      display: block;
      font-size: 1rem;
      font-weight: 700;
      color: #1a1a2e;
      text-decoration: none;
      padding: 0 0.5rem 1rem;
      margin-bottom: 0.5rem;
      border-bottom: 1px solid #dee2e6;
    }

    .orbit-docs-brand:hover { color: #228be6; }

    /* ── Sidebar Search ── */
    .orbit-docs-search {
      padding: 0.5rem 0.5rem 1rem;
    }
    .orbit-docs-search-input {
      width: 100%;
      padding: 0.5rem 0.75rem;
      font-size: 0.85rem;
      border: 1px solid #dee2e6;
      border-radius: 0.375rem;
      background: #ffffff;
      color: #1a1a2e;
      outline: none;
      transition: border-color 0.15s ease;
    }
    .orbit-docs-search-input:focus {
      border-color: #7048e8;
    }

    .orbit-docs-body {
      flex: 1;
      min-width: 0;
      overflow-y: auto;
      display: flex;
      flex-direction: column;
    }

    /* ── Content Header bar ── */
    .orbit-docs-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 0.75rem 2rem;
      border-bottom: 1px solid #dee2e6;
      background: #ffffff;
      font-size: 0.85rem;
      flex-shrink: 0;
    }
    .orbit-docs-header-env {
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }
    .orbit-docs-env-label {
      font-weight: 500;
      color: #495057;
    }
    .orbit-docs-env-select {
      padding: 0.25rem 0.5rem;
      border: 1px solid #dee2e6;
      border-radius: 0.25rem;
      background: #f8f9fa;
      color: #495057;
      outline: none;
    }
    .orbit-docs-header-meta {
      display: flex;
      align-items: center;
      gap: 1.5rem;
    }
    .orbit-docs-meta-link {
      color: #495057;
      text-decoration: none;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 0.25rem;
    }
    .orbit-docs-meta-link:hover {
      color: #7048e8;
    }

    .orbit-docs-main {
      max-width: 76ch;
      padding: 2.5rem 2rem 4rem;
      flex: 1;
    }

    .orbit-docs-content > *:first-child { margin-top: 0; }

    /* ── Dark mode geometry adjustments ── */
    @media (prefers-color-scheme: dark) {
      body { color: #c1c2c5; background: #141517; }
      .orbit-docs-sidebar { background: #1a1b1e; border-color: #2c2e33; }
      .orbit-docs-brand   { color: #c1c2c5; border-color: #2c2e33; }
      .orbit-docs-search-input {
        background: #2c2e33;
        border-color: #373a40;
        color: #c1c2c5;
      }
      .orbit-docs-search-input:focus {
        border-color: #9072f8;
      }
      .orbit-docs-header {
        background: #141517;
        border-color: #2c2e33;
      }
      .orbit-docs-env-label, .orbit-docs-meta-link {
        color: #909296;
      }
      .orbit-docs-env-select {
        background: #25262b;
        border-color: #2c2e33;
        color: #c1c2c5;
      }
      .orbit-docs-meta-link:hover {
        color: #9072f8;
      }
    }

    /* ── Responsive: collapse sidebar on small screens ── */
    @media (max-width: 720px) {
      .orbit-docs-layout { flex-direction: column; }
      .orbit-docs-sidebar {
        position: static;
        height: auto;
        width: 100%;
        border-right: none;
        border-bottom: 1px solid #dee2e6;
      }
      .orbit-docs-header {
        padding: 0.75rem 1rem;
      }
      .orbit-docs-main { padding: 1.5rem 1rem 3rem; }
    }
"#;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn opts<'a>(content: &'a str) -> LayoutOptions<'a> {
        LayoutOptions {
            page_title: "Getting Started",
            site_title: "Orbit",
            content,
            theme_css: "",
            sidebar_html: None,
        }
    }

    // ── render_layout dispatch ────────────────────────────────────────────────

    #[test]
    fn unknown_layout_returns_none() {
        assert!(render_layout("hbs", &opts("")).is_none());
        assert!(render_layout("", &opts("")).is_none());
    }

    #[test]
    fn all_built_in_layouts_resolve() {
        for &name in BUILT_IN_LAYOUTS {
            assert!(
                render_layout(name, &opts("")).is_some(),
                "BUILT_IN_LAYOUTS lists '{name}' but render_layout returned None"
            );
        }
    }

    // ── Shared <head> ─────────────────────────────────────────────────────────

    #[test]
    fn page_title_uses_pipe_separator() {
        let html = render_layout("default", &opts("")).unwrap();
        assert!(html.contains("<title>Getting Started | Orbit</title>"));
    }

    #[test]
    fn matching_page_and_site_title_emits_single_title() {
        let o = LayoutOptions {
            page_title: "Orbit",
            site_title: "Orbit",
            content: "",
            theme_css: "",
            sidebar_html: None,
        };
        let html = render_layout("default", &o).unwrap();
        // Should contain exactly "Orbit" and NOT "Orbit | Orbit"
        assert!(html.contains("<title>Orbit</title>"));
        assert!(!html.contains("Orbit | Orbit"));
    }

    #[test]
    fn theme_css_is_embedded_in_style_tag_when_present() {
        let o = LayoutOptions {
            page_title: "Test",
            site_title: "Test",
            content: "",
            theme_css: ".orbit-callout { color: red; }",
            sidebar_html: None,
        };
        let html = render_layout("default", &o).unwrap();
        assert!(html.contains(".orbit-callout { color: red; }"));
    }

    #[test]
    fn empty_theme_css_produces_no_extra_style_tag() {
        let html = render_layout("default", &opts("")).unwrap();
        // The layout CSS block is always present, but the extra theme block is not.
        // Count <style> tags — should be exactly 1 (the layout CSS).
        let count = html.matches("<style>").count();
        assert_eq!(count, 1, "expected exactly 1 <style> block, got {count}");
    }

    #[test]
    fn html_in_title_is_escaped() {
        let o = LayoutOptions {
            page_title: "<script>alert(1)</script>",
            site_title: "Safe",
            content: "",
            theme_css: "",
            sidebar_html: None,
        };
        let html = render_layout("default", &o).unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    // ── Default layout ────────────────────────────────────────────────────────

    #[test]
    fn default_layout_contains_doctype() {
        let html = render_layout("default", &opts("")).unwrap();
        assert!(html.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn default_layout_wraps_content_in_orbit_page() {
        let html = render_layout("default", &opts("<p>Hello.</p>")).unwrap();
        assert!(html.contains("orbit-page"));
        assert!(html.contains("<p>Hello.</p>"));
    }

    #[test]
    fn default_layout_has_no_sidebar() {
        let html = render_layout("default", &opts("")).unwrap();
        // The <aside> element is only emitted by the docs layout.
        assert!(
            !html.contains("<aside"),
            "default layout should not contain <aside>"
        );
    }

    // ── Docs layout ───────────────────────────────────────────────────────────

    #[test]
    fn docs_layout_contains_sidebar_element() {
        let html = render_layout("docs", &opts("")).unwrap();
        assert!(html.contains("orbit-docs-sidebar"));
        assert!(html.contains("orbit-docs-nav"));
    }

    #[test]
    fn docs_layout_renders_sidebar_html() {
        let o = LayoutOptions {
            page_title: "Docs",
            site_title: "Orbit",
            content: "<p>Doc content.</p>",
            theme_css: "",
            sidebar_html: Some("<nav>sidebar</nav>"),
        };
        let html = render_layout("docs", &o).unwrap();
        assert!(html.contains("<nav>sidebar</nav>"));
        assert!(html.contains("<p>Doc content.</p>"));
    }

    #[test]
    fn docs_layout_without_sidebar_html_still_renders() {
        let html = render_layout("docs", &opts("<p>content</p>")).unwrap();
        assert!(html.contains("orbit-docs-main"));
        assert!(html.contains("<p>content</p>"));
    }

    #[test]
    fn docs_layout_shows_site_title_as_brand_link() {
        let html = render_layout("docs", &opts("")).unwrap();
        assert!(html.contains("orbit-docs-brand"));
        assert!(html.contains("Orbit"));
    }

    #[test]
    fn docs_layout_has_main_content_id() {
        let html = render_layout("docs", &opts("")).unwrap();
        assert!(html.contains("id=\"main-content\""));
    }
}
