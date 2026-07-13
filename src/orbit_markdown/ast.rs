//! Abstract Syntax Tree types for Orbit Markdown.
//!
//! A parsed `.md` document is represented as a `Vec<OrbitNode>` before any
//! HTML is produced. The parser builds this tree; the renderer walks it.
//!
//! # Design rules
//!
//! - This module contains only types. No parsing, no rendering.
//! - Every directive in the v1.0.0 spec has exactly one `OrbitNode` variant.
//! - `Markdown(String)` is the passthrough for everything that is not an
//!   Orbit directive — plain paragraphs, headings, lists, etc.
//! - Adjacent `Card` nodes are grouped into `CardGrid` by the parser as a
//!   post-parse transformation, not during scanning.

/// A single node in an Orbit Markdown document.
///
/// A document is represented as `Vec<OrbitNode>`. The parser produces this
/// flat list; the renderer walks it to produce the final HTML string.
#[derive(Debug, Clone, PartialEq)]
pub enum OrbitNode {
    /// A chunk of raw Markdown source that is not an Orbit directive.
    ///
    /// Passed verbatim to `pulldown-cmark` during rendering. This covers
    /// headings, paragraphs, lists, code blocks, blockquotes, and any other
    /// standard Markdown constructs.
    Markdown(String),

    // ── Callouts ─────────────────────────────────────────────────────────────
    /// A styled callout block: note, info, warning, danger, success, or tip.
    ///
    /// ```md
    /// :::warning title="Watch out"
    /// This will affect your configuration.
    /// :::
    /// ```
    Callout {
        /// Which callout variant to render.
        kind: CalloutKind,
        /// Optional heading shown above the body. Maps to the `title` attribute.
        title: Option<String>,
        /// Inner document nodes compiled from the body Markdown.
        children: Vec<OrbitNode>,
    },

    // ── Structure blocks ──────────────────────────────────────────────────────
    /// A numbered how-to sequence.
    ///
    /// ```md
    /// :::steps
    /// - Install Rust
    /// - Run `cargo install orbit-md`
    /// - Start `orbit dev`
    /// :::
    /// ```
    ///
    /// Each string in `items` is a Markdown fragment compiled from one list item.
    Steps {
        /// Ordered list items. Each item is a raw Markdown string that the
        /// renderer compiles individually.
        items: Vec<String>,
    },

    /// A content card with an optional clickable link.
    ///
    /// ```md
    /// :::card title="Fast Builds" href="/docs/perf"
    /// Compiles pages in parallel.
    /// :::
    /// ```
    ///
    /// Adjacent `Card` nodes are grouped into [`OrbitNode::CardGrid`] by a
    /// post-parse pass. Do not produce `CardGrid` directly from the scanner.
    Card {
        /// Card heading text. Required by the directive spec.
        title: Option<String>,
        /// When present, the whole card becomes an `<a>` element.
        href: Option<String>,
        /// Inner document nodes compiled from the body Markdown.
        children: Vec<OrbitNode>,
    },

    /// Two or more consecutive `Card` nodes grouped for grid layout.
    ///
    /// Created by a post-parse pass over the flat node list — never produced
    /// directly by the scanner. The renderer wraps these in
    /// `<div class="orbit-card-grid">`.
    CardGrid(Vec<OrbitNode>),

    /// A feature grid for landing pages.
    ///
    /// ```md
    /// :::features
    /// - **Markdown-native**: Write `.md` files.
    /// - **Zero JS**: Static HTML output.
    /// :::
    /// ```
    Features {
        /// Parsed feature entries. Each entry has a title (the bold text at
        /// the start of the list item) and a body.
        items: Vec<FeatureItem>,
    },

    // ── Navigation blocks ─────────────────────────────────────────────────────
    /// A group of CTA link buttons.
    ///
    /// ```md
    /// :::buttons
    /// [Get started](/docs) primary
    /// [GitHub](https://github.com) secondary
    /// :::
    /// ```
    Buttons {
        /// Ordered list of button definitions.
        items: Vec<ButtonItem>,
    },

    /// A landing-page hero section.
    ///
    /// ```md
    /// :::hero title="Orbit" subtitle="Fast static sites from Markdown"
    /// Write `.md` files. No templates.
    ///
    /// :::buttons
    /// [Get started](/docs) primary
    /// :::
    /// :::
    /// ```
    Hero {
        /// Main headline. Maps to `<h1 class="orbit-hero-title">`.
        title: String,
        /// Supporting line below the headline. Maps to `orbit-hero-subtitle`.
        subtitle: Option<String>,
        /// Description paragraphs between the subtitle and the action buttons.
        body: Vec<OrbitNode>,
        /// Optional nested `Buttons` node for CTAs. Rendered in
        /// `<div class="orbit-hero-actions">`.
        actions: Option<Box<OrbitNode>>,
    },

    /// A labeled sidebar navigation section (docs layout only).
    ///
    /// ```md
    /// :::nav-group title="Reference"
    /// - [Config options](/docs/config)
    /// - [CLI commands](/docs/cli)
    /// :::
    /// ```
    ///
    /// Does not render HTML on the page itself — it feeds the sidebar
    /// generation in the `docs` layout.
    NavGroup {
        /// Section heading shown in the sidebar.
        title: String,
        /// List of (display label, href) pairs.
        links: Vec<(String, String)>,
    },

    // ── Media blocks ──────────────────────────────────────────────────────────
    /// A semantic image with required alt text and an optional caption.
    ///
    /// ```md
    /// :::figure src="/images/diagram.png" alt="The build pipeline" caption="Figure 1."
    /// :::
    /// ```
    Figure {
        /// Image source path or URL.
        src: String,
        /// Alt text — required for accessibility.
        alt: String,
        /// Optional caption shown below the image in a `<figcaption>`.
        caption: Option<String>,
    },
}

// ── Supporting types ──────────────────────────────────────────────────────────

/// Which visual style a callout block uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalloutKind {
    /// 📝 General information.
    Note,
    /// ℹ️ Background context.
    Info,
    /// ⚠️ Potential problem.
    Warning,
    /// 🚨 Breaking change or destructive action.
    Danger,
    /// ✅ Confirmation or completion.
    Success,
    /// 💡 Best practice or shortcut.
    Tip,
}

impl CalloutKind {
    /// Returns the CSS modifier class suffix (e.g. `"warning"`).
    pub fn css_name(&self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Success => "success",
            Self::Tip => "tip",
        }
    }

    /// Returns the emoji icon for this callout kind.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Note => "📝",
            Self::Info => "ℹ️",
            Self::Warning => "⚠️",
            Self::Danger => "🚨",
            Self::Success => "✅",
            Self::Tip => "💡",
        }
    }
}

/// A single entry in a `:::features` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureItem {
    /// Feature title — parsed from the bold text at the start of a list item.
    pub title: String,
    /// Feature description — the rest of the list item after the `:` separator.
    pub body: String,
}

/// A single button inside a `:::buttons` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonItem {
    /// Visible button label.
    pub label: String,
    /// Button link target.
    pub href: String,
    /// Visual style of the button.
    pub style: ButtonStyle,
}

/// Visual style applied to a button inside `:::buttons`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ButtonStyle {
    /// Filled button with the brand color background. Used for the primary CTA.
    #[default]
    Primary,
    /// Outlined button with brand color border and text.
    Secondary,
    /// Text-only button with no background or border.
    Ghost,
    /// Filled button with the danger color — for destructive or warning CTAs.
    Danger,
}

impl ButtonStyle {
    /// Parses a style token from the `:::buttons` body line.
    ///
    /// Accepts `"primary"`, `"secondary"`, `"ghost"`, `"danger"`.
    /// Returns `ButtonStyle::Primary` for any unknown token so the parser
    /// never needs to produce an error for a style mismatch.
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "secondary" => Self::Secondary,
            "ghost" => Self::Ghost,
            "danger" => Self::Danger,
            _ => Self::Primary,
        }
    }

    /// Returns the CSS modifier class suffix (e.g. `"secondary"`).
    pub fn css_name(&self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Ghost => "ghost",
            Self::Danger => "danger",
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callout_kind_css_name_and_icon_are_consistent() {
        let cases = [
            (CalloutKind::Note, "note", "📝"),
            (CalloutKind::Info, "info", "ℹ️"),
            (CalloutKind::Warning, "warning", "⚠️"),
            (CalloutKind::Danger, "danger", "🚨"),
            (CalloutKind::Success, "success", "✅"),
            (CalloutKind::Tip, "tip", "💡"),
        ];
        for (kind, css, icon) in cases {
            assert_eq!(kind.css_name(), css);
            assert_eq!(kind.icon(), icon);
        }
    }

    #[test]
    fn button_style_from_str_parses_known_tokens() {
        assert_eq!(ButtonStyle::from_str("primary"), ButtonStyle::Primary);
        assert_eq!(ButtonStyle::from_str("secondary"), ButtonStyle::Secondary);
        assert_eq!(ButtonStyle::from_str("ghost"), ButtonStyle::Ghost);
        assert_eq!(ButtonStyle::from_str("danger"), ButtonStyle::Danger);
    }

    #[test]
    fn button_style_from_str_falls_back_to_primary() {
        assert_eq!(ButtonStyle::from_str("unknown"), ButtonStyle::Primary);
        assert_eq!(ButtonStyle::from_str(""), ButtonStyle::Primary);
        assert_eq!(ButtonStyle::from_str("PRIMARY"), ButtonStyle::Primary);
    }

    #[test]
    fn button_style_css_name_matches_from_str() {
        for style in [
            ButtonStyle::Primary,
            ButtonStyle::Secondary,
            ButtonStyle::Ghost,
            ButtonStyle::Danger,
        ] {
            // Round-trip: css_name() should be parseable back to the same style.
            assert_eq!(ButtonStyle::from_str(style.css_name()), style);
        }
    }

    #[test]
    fn orbit_node_callout_stores_kind_title_children() {
        let node = OrbitNode::Callout {
            kind: CalloutKind::Warning,
            title: Some("Heads up".to_owned()),
            children: vec![OrbitNode::Markdown("Be careful.".to_owned())],
        };
        match node {
            OrbitNode::Callout {
                kind,
                title,
                children,
            } => {
                assert_eq!(kind, CalloutKind::Warning);
                assert_eq!(title.as_deref(), Some("Heads up"));
                assert_eq!(children.len(), 1);
            }
            _ => panic!("expected Callout"),
        }
    }

    #[test]
    fn orbit_node_card_grid_wraps_cards() {
        let cards = vec![
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
        ];
        let grid = OrbitNode::CardGrid(cards);
        match grid {
            OrbitNode::CardGrid(inner) => assert_eq!(inner.len(), 2),
            _ => panic!("expected CardGrid"),
        }
    }

    #[test]
    fn orbit_node_hero_stores_all_fields() {
        let hero = OrbitNode::Hero {
            title: "Orbit".to_owned(),
            subtitle: Some("Fast static sites".to_owned()),
            body: vec![OrbitNode::Markdown("Description.".to_owned())],
            actions: Some(Box::new(OrbitNode::Buttons { items: vec![] })),
        };
        match hero {
            OrbitNode::Hero {
                title,
                subtitle,
                body,
                actions,
            } => {
                assert_eq!(title, "Orbit");
                assert!(subtitle.is_some());
                assert_eq!(body.len(), 1);
                assert!(actions.is_some());
            }
            _ => panic!("expected Hero"),
        }
    }
}
