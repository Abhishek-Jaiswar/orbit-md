//! Orbit Markdown renderer — `Vec<OrbitNode>` → HTML string.
//!
//! Each `OrbitNode` variant is rendered to a self-contained HTML fragment.
//! `Markdown` chunks are rendered with `pulldown-cmark` (same options as the
//! main Orbit compiler). All generated elements use `orbit-` prefixed class
//! names to avoid collisions with user CSS.
//!
//! # Public API
//!
//! ```
//! use orbit_md::orbit_markdown::ast::{OrbitNode, CalloutKind};
//! use orbit_md::orbit_markdown::render::render_nodes;
//!
//! let nodes = vec![
//!     OrbitNode::Callout {
//!         kind: CalloutKind::Warning,
//!         title: Some("Heads up".to_owned()),
//!         children: vec![OrbitNode::Markdown("Be careful.".to_owned())],
//!     },
//! ];
//! let html = render_nodes(&nodes);
//! assert!(html.contains("orbit-callout--warning"));
//! assert!(html.contains("Heads up"));
//! ```

use pulldown_cmark::{Options, Parser as CmParser, html as cm_html};

use crate::orbit_markdown::ast::OrbitNode;

// ── Public API ────────────────────────────────────────────────────────────────

/// Render a slice of `OrbitNode`s into an HTML string.
///
/// Nodes are concatenated in order. No surrounding wrapper is added — callers
/// insert the result into whatever page shell they need.
pub fn render_nodes(nodes: &[OrbitNode]) -> String {
    nodes.iter().map(render_node).collect()
}

// ── Per-node renderer ─────────────────────────────────────────────────────────

fn render_node(node: &OrbitNode) -> String {
    match node {
        OrbitNode::Markdown(src) => render_markdown(src),
        OrbitNode::Callout {
            kind,
            title,
            children,
        } => render_callout(kind, title, children),
        OrbitNode::Steps { items } => render_steps(items),
        OrbitNode::Card {
            title,
            href,
            children,
        } => render_card(title, href, children),
        OrbitNode::CardGrid(cards) => render_card_grid(cards),
        OrbitNode::Features { items } => render_features(items),
        OrbitNode::Buttons { items } => render_buttons(items),
        OrbitNode::Hero {
            title,
            subtitle,
            body,
            actions,
        } => render_hero(title, subtitle, body, actions),
        OrbitNode::NavGroup { title, links } => render_nav_group(title, links),
        OrbitNode::Figure { src, alt, caption } => render_figure(src, alt, caption),
    }
}

// ── Markdown passthrough ──────────────────────────────────────────────────────

/// Compile a raw Markdown string to HTML using the same options as the main
/// Orbit compiler (tables, strikethrough, footnotes).
fn render_markdown(source: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = CmParser::new_ext(source, opts);
    let mut out = String::with_capacity(source.len().saturating_mul(2));
    cm_html::push_html(&mut out, parser);
    out
}

// ── Directive renderers ───────────────────────────────────────────────────────

fn render_callout(
    kind: &crate::orbit_markdown::ast::CalloutKind,
    title: &Option<String>,
    children: &[OrbitNode],
) -> String {
    let css = kind.css_name();
    let icon = kind.icon();
    let inner = render_nodes(children);

    let title_html = match title {
        Some(t) => format!(
            "<strong class=\"orbit-callout-title\">{}</strong>\n",
            escape_html(t)
        ),
        None => String::new(),
    };

    format!(
        "<div class=\"orbit-callout orbit-callout--{css}\">\
\n  <span class=\"orbit-callout-icon\" aria-hidden=\"true\">{icon}</span>\
\n  <div class=\"orbit-callout-content\">\n{title_html}{inner}  </div>\
\n</div>\n"
    )
}

fn render_steps(items: &[String]) -> String {
    let mut html = String::from("<ol class=\"orbit-steps\">\n");
    for item in items {
        // Each step item may contain inline Markdown (bold, code, links).
        let compiled = render_markdown(item);
        // pulldown-cmark wraps single lines in <p>. Unwrap if it's just one.
        let inner = unwrap_single_paragraph(&compiled);
        html.push_str(&format!("  <li class=\"orbit-step\">{inner}</li>\n"));
    }
    html.push_str("</ol>\n");
    html
}

fn render_card(title: &Option<String>, href: &Option<String>, children: &[OrbitNode]) -> String {
    let body_html = render_nodes(children);
    let title_html = match title {
        Some(t) => format!("<h3 class=\"orbit-card-title\">{}</h3>\n", escape_html(t)),
        None => String::new(),
    };
    let content = format!("{title_html}<div class=\"orbit-card-body\">\n{body_html}</div>\n");

    match href {
        Some(h) => format!(
            "<a class=\"orbit-card orbit-card--link\" href=\"{}\">\n{content}</a>\n",
            escape_attr(h)
        ),
        None => format!("<div class=\"orbit-card\">\n{content}</div>\n"),
    }
}

fn render_card_grid(cards: &[OrbitNode]) -> String {
    let inner: String = cards.iter().map(render_node).collect();
    format!("<div class=\"orbit-card-grid\">\n{inner}</div>\n")
}

fn render_features(items: &[crate::orbit_markdown::ast::FeatureItem]) -> String {
    let mut html = String::from("<div class=\"orbit-features\">\n");
    for item in items {
        html.push_str("  <div class=\"orbit-feature\">\n");
        if !item.title.is_empty() {
            html.push_str(&format!(
                "    <strong class=\"orbit-feature-title\">{}</strong>\n",
                escape_html(&item.title)
            ));
        }
        if !item.body.is_empty() {
            let body_html = render_markdown(&item.body);
            let inner = unwrap_single_paragraph(&body_html);
            html.push_str(&format!(
                "    <p class=\"orbit-feature-body\">{inner}</p>\n"
            ));
        }
        html.push_str("  </div>\n");
    }
    html.push_str("</div>\n");
    html
}

fn render_buttons(items: &[crate::orbit_markdown::ast::ButtonItem]) -> String {
    let mut html = String::from("<div class=\"orbit-buttons\">\n");
    for item in items {
        let style_class = item.style.css_name();
        html.push_str(&format!(
            "  <a class=\"orbit-btn orbit-btn--{style_class}\" href=\"{}\">{}</a>\n",
            escape_attr(&item.href),
            escape_html(&item.label)
        ));
    }
    html.push_str("</div>\n");
    html
}

fn render_hero(
    title: &str,
    subtitle: &Option<String>,
    body: &[OrbitNode],
    actions: &Option<Box<OrbitNode>>,
) -> String {
    let mut html = String::from("<section class=\"orbit-hero\">\n");

    // Partition body into graphic and text nodes
    let mut text_nodes = Vec::new();
    let mut graphic = None;
    for node in body {
        if matches!(node, OrbitNode::Figure { .. }) && graphic.is_none() {
            graphic = Some(node.clone());
        } else {
            text_nodes.push(node.clone());
        }
    }

    html.push_str("  <div class=\"orbit-hero-content\">\n");
    html.push_str(&format!(
        "    <h1 class=\"orbit-hero-title\">{}</h1>\n",
        escape_html(title)
    ));

    if let Some(sub) = subtitle {
        html.push_str(&format!(
            "    <p class=\"orbit-hero-subtitle\">{}</p>\n",
            escape_html(sub)
        ));
    }

    let body_html = render_nodes(&text_nodes);

    let mut graphic_html = String::new();
    let mut clean_body_html = body_html.clone();

    if let Some(start_idx) = body_html.find("<img") {
        if let Some(end_len) = body_html[start_idx..].find(">") {
            let end_idx = start_idx + end_len + 1;
            graphic_html = body_html[start_idx..end_idx].to_owned();
            
            // Clean up surrounding paragraph tags if any
            let prefix = &body_html[..start_idx];
            let suffix = &body_html[end_idx..];
            let clean_prefix = prefix.trim_end_matches("<p>").trim_end_matches("<p style=\"text-align: center;\">");
            let clean_suffix = suffix.trim_start_matches("</p>");
            
            clean_body_html = format!("{}{}", clean_prefix, clean_suffix);
        }
    }

    html.push_str("  <div class=\"orbit-hero-content\">\n");
    html.push_str(&format!(
        "    <h1 class=\"orbit-hero-title\">{}</h1>\n",
        escape_html(title)
    ));

    if let Some(sub) = subtitle {
        html.push_str(&format!(
            "    <p class=\"orbit-hero-subtitle\">{}</p>\n",
            escape_html(sub)
        ));
    }

    let trimmed_body = clean_body_html.trim();
    if !trimmed_body.is_empty() {
        html.push_str("    <div class=\"orbit-hero-body\">\n");
        html.push_str(trimmed_body);
        html.push_str("\n    </div>\n");
    }

    if let Some(btns) = actions {
        html.push_str("    <div class=\"orbit-hero-actions\">\n");
        html.push_str(&render_node(btns));
        html.push_str("    </div>\n");
    }
    html.push_str("  </div>\n");

    // Render the extracted graphic image if present, otherwise fallback to explicit graphic node
    if !graphic_html.is_empty() {
        html.push_str("  <div class=\"orbit-hero-graphic\">\n");
        html.push_str(&graphic_html);
        html.push_str("\n  </div>\n");
    } else if let Some(g) = graphic {
        html.push_str("  <div class=\"orbit-hero-graphic\">\n");
        html.push_str(&render_node(&g));
        html.push_str("  </div>\n");
    }

    html.push_str("</section>\n");
    html
}

fn render_nav_group(title: &str, links: &[(String, String)]) -> String {
    let mut html = String::from("<nav class=\"orbit-nav-group\">\n");
    html.push_str(&format!(
        "  <strong class=\"orbit-nav-group-title\">{}</strong>\n",
        escape_html(title)
    ));
    html.push_str("  <ul class=\"orbit-nav-group-links\">\n");
    for (label, href) in links {
        html.push_str(&format!(
            "    <li><a href=\"{}\">{}</a></li>\n",
            escape_attr(href),
            escape_html(label)
        ));
    }
    html.push_str("  </ul>\n");
    html.push_str("</nav>\n");
    html
}

fn render_figure(src: &str, alt: &str, caption: &Option<String>) -> String {
    let img = format!(
        "  <img src=\"{}\" alt=\"{}\">\n",
        escape_attr(src),
        escape_attr(alt)
    );
    let cap_html = match caption {
        Some(c) => format!(
            "  <figcaption class=\"orbit-figure-caption\">{}</figcaption>\n",
            escape_html(c)
        ),
        None => String::new(),
    };
    format!("<figure class=\"orbit-figure\">\n{img}{cap_html}</figure>\n")
}

// ── Utilities ─────────────────────────────────────────────────────────────────

/// Escape the five special HTML characters in text content.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Escape characters that are special inside an HTML attribute value.
fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// If `html` is a single `<p>…</p>` block, return just the inner HTML.
///
/// pulldown-cmark wraps single-line Markdown in a `<p>` tag. For inline
/// contexts (step items, feature bodies) we strip that wrapper so the
/// surrounding element provides the block semantics.
fn unwrap_single_paragraph(html: &str) -> &str {
    let trimmed = html.trim();
    if trimmed.starts_with("<p>") && trimmed.ends_with("</p>") {
        let inner = &trimmed[3..trimmed.len() - 4];
        // Only unwrap if there's no second block element inside.
        if !inner.contains("<p>") {
            return inner;
        }
    }
    html
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orbit_markdown::ast::*;

    fn html(nodes: &[OrbitNode]) -> String {
        render_nodes(nodes)
    }

    // ── Markdown passthrough ──────────────────────────────────────────────────

    #[test]
    fn markdown_node_renders_to_html() {
        let out = html(&[OrbitNode::Markdown("**Bold**".to_owned())]);
        assert!(out.contains("<strong>Bold</strong>"));
    }

    // ── Callouts ─────────────────────────────────────────────────────────────

    #[test]
    fn callout_note_has_correct_class() {
        let out = html(&[OrbitNode::Callout {
            kind: CalloutKind::Note,
            title: None,
            children: vec![],
        }]);
        assert!(out.contains("orbit-callout--note"));
        assert!(out.contains("📝"));
    }

    #[test]
    fn callout_warning_has_correct_class_and_icon() {
        let out = html(&[OrbitNode::Callout {
            kind: CalloutKind::Warning,
            title: Some("Watch out".to_owned()),
            children: vec![OrbitNode::Markdown("Be careful.".to_owned())],
        }]);
        assert!(out.contains("orbit-callout--warning"));
        assert!(out.contains("⚠️"));
        assert!(out.contains("Watch out"));
        assert!(out.contains("Be careful."));
    }

    #[test]
    fn callout_without_title_has_no_title_element() {
        let out = html(&[OrbitNode::Callout {
            kind: CalloutKind::Info,
            title: None,
            children: vec![],
        }]);
        assert!(!out.contains("orbit-callout-title"));
    }

    #[test]
    fn all_callout_kinds_render_their_css_class() {
        for kind in [
            CalloutKind::Note,
            CalloutKind::Info,
            CalloutKind::Warning,
            CalloutKind::Danger,
            CalloutKind::Success,
            CalloutKind::Tip,
        ] {
            let css = kind.css_name();
            let out = html(&[OrbitNode::Callout {
                kind: kind.clone(),
                title: None,
                children: vec![],
            }]);
            assert!(
                out.contains(&format!("orbit-callout--{css}")),
                "missing class for {css}"
            );
        }
    }

    // ── Steps ─────────────────────────────────────────────────────────────────

    #[test]
    fn steps_renders_ordered_list() {
        let out = html(&[OrbitNode::Steps {
            items: vec!["Install Rust".to_owned(), "Run cargo".to_owned()],
        }]);
        assert!(out.contains("<ol class=\"orbit-steps\">"));
        assert!(out.contains("orbit-step"));
        assert!(out.contains("Install Rust"));
        assert!(out.contains("Run cargo"));
    }

    // ── Card ──────────────────────────────────────────────────────────────────

    #[test]
    fn card_without_href_renders_div() {
        let out = html(&[OrbitNode::Card {
            title: Some("Fast Builds".to_owned()),
            href: None,
            children: vec![],
        }]);
        assert!(out.contains("<div class=\"orbit-card\">"));
        assert!(out.contains("Fast Builds"));
        assert!(!out.contains("<a "));
    }

    #[test]
    fn card_with_href_renders_anchor() {
        let out = html(&[OrbitNode::Card {
            title: Some("Docs".to_owned()),
            href: Some("/docs".to_owned()),
            children: vec![],
        }]);
        assert!(out.contains("<a class=\"orbit-card orbit-card--link\""));
        assert!(out.contains("href=\"/docs\""));
    }

    // ── CardGrid ──────────────────────────────────────────────────────────────

    #[test]
    fn card_grid_wraps_in_grid_div() {
        let out = html(&[OrbitNode::CardGrid(vec![
            OrbitNode::Card {
                title: Some("A".to_owned()),
                href: None,
                children: vec![],
            },
            OrbitNode::Card {
                title: Some("B".to_owned()),
                href: None,
                children: vec![],
            },
        ])]);
        assert!(out.contains("orbit-card-grid"));
        assert!(out.contains("orbit-card"));
    }

    // ── Features ──────────────────────────────────────────────────────────────

    #[test]
    fn features_renders_titles_and_bodies() {
        let out = html(&[OrbitNode::Features {
            items: vec![FeatureItem {
                title: "Markdown-native".to_owned(),
                body: "Write plain files.".to_owned(),
            }],
        }]);
        assert!(out.contains("orbit-features"));
        assert!(out.contains("orbit-feature-title"));
        assert!(out.contains("Markdown-native"));
        assert!(out.contains("Write plain files."));
    }

    // ── Buttons ───────────────────────────────────────────────────────────────

    #[test]
    fn buttons_renders_anchor_tags_with_style_class() {
        let out = html(&[OrbitNode::Buttons {
            items: vec![
                ButtonItem {
                    label: "Start".to_owned(),
                    href: "/docs".to_owned(),
                    style: ButtonStyle::Primary,
                },
                ButtonItem {
                    label: "Code".to_owned(),
                    href: "https://github.com".to_owned(),
                    style: ButtonStyle::Secondary,
                },
            ],
        }]);
        assert!(out.contains("orbit-btn--primary"));
        assert!(out.contains("orbit-btn--secondary"));
        assert!(out.contains("href=\"/docs\""));
        assert!(out.contains("Start"));
    }

    // ── Hero ──────────────────────────────────────────────────────────────────

    #[test]
    fn hero_renders_section_with_h1() {
        let out = html(&[OrbitNode::Hero {
            title: "Orbit".to_owned(),
            subtitle: Some("Fast sites".to_owned()),
            body: vec![],
            actions: None,
        }]);
        assert!(out.contains("<section class=\"orbit-hero\">"));
        assert!(out.contains("<h1 class=\"orbit-hero-title\">Orbit</h1>"));
        assert!(out.contains("orbit-hero-subtitle"));
        assert!(out.contains("Fast sites"));
    }

    #[test]
    fn hero_without_subtitle_has_no_subtitle_element() {
        let out = html(&[OrbitNode::Hero {
            title: "Orbit".to_owned(),
            subtitle: None,
            body: vec![],
            actions: None,
        }]);
        assert!(!out.contains("orbit-hero-subtitle"));
    }

    #[test]
    fn hero_with_actions_renders_buttons_inside() {
        let out = html(&[OrbitNode::Hero {
            title: "Orbit".to_owned(),
            subtitle: None,
            body: vec![],
            actions: Some(Box::new(OrbitNode::Buttons {
                items: vec![ButtonItem {
                    label: "Start".to_owned(),
                    href: "/docs".to_owned(),
                    style: ButtonStyle::Primary,
                }],
            })),
        }]);
        assert!(out.contains("orbit-hero-actions"));
        assert!(out.contains("orbit-btn--primary"));
        assert!(out.contains("Start"));
    }

    // ── NavGroup ──────────────────────────────────────────────────────────────

    #[test]
    fn nav_group_renders_nav_with_links() {
        let out = html(&[OrbitNode::NavGroup {
            title: "Reference".to_owned(),
            links: vec![
                ("Config".to_owned(), "/docs/config".to_owned()),
                ("CLI".to_owned(), "/docs/cli".to_owned()),
            ],
        }]);
        assert!(out.contains("<nav class=\"orbit-nav-group\">"));
        assert!(out.contains("orbit-nav-group-title"));
        assert!(out.contains("Reference"));
        assert!(out.contains("href=\"/docs/config\""));
        assert!(out.contains("Config"));
    }

    // ── Figure ────────────────────────────────────────────────────────────────

    #[test]
    fn figure_renders_img_and_figcaption() {
        let out = html(&[OrbitNode::Figure {
            src: "/img.png".to_owned(),
            alt: "Diagram".to_owned(),
            caption: Some("Fig 1.".to_owned()),
        }]);
        assert!(out.contains("<figure class=\"orbit-figure\">"));
        assert!(out.contains("src=\"/img.png\""));
        assert!(out.contains("alt=\"Diagram\""));
        assert!(out.contains("orbit-figure-caption"));
        assert!(out.contains("Fig 1."));
    }

    #[test]
    fn figure_without_caption_has_no_figcaption() {
        let out = html(&[OrbitNode::Figure {
            src: "/img.png".to_owned(),
            alt: "Alt".to_owned(),
            caption: None,
        }]);
        assert!(!out.contains("figcaption"));
    }

    // ── HTML escaping ─────────────────────────────────────────────────────────

    #[test]
    fn html_in_title_is_escaped() {
        let out = html(&[OrbitNode::Callout {
            kind: CalloutKind::Note,
            title: Some("<script>alert(1)</script>".to_owned()),
            children: vec![],
        }]);
        assert!(!out.contains("<script>"));
        assert!(out.contains("&lt;script&gt;"));
    }

    #[test]
    fn html_in_href_is_escaped() {
        let out = html(&[OrbitNode::Card {
            title: Some("Card".to_owned()),
            href: Some("/path?a=1&b=2".to_owned()),
            children: vec![],
        }]);
        assert!(out.contains("&amp;"));
    }
}
