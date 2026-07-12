//! Orbit Markdown directive parser.
//!
//! Converts a `.md` source string into a flat `Vec<OrbitNode>` by scanning
//! line-by-line for `:::directive` blocks. Standard Markdown between
//! directives is returned as `OrbitNode::Markdown` chunks for `pulldown-cmark`.
//!
//! # Scanner states
//!
//! ```text
//! Normal ──(:::name)──→ InDirective ──(:::)──→ Normal
//!   │                                              │
//!   └──(fence open)──→ CodeFence ──(fence close)──┘
//! ```
//!
//! Directives are **never** recognised inside a fenced code block.
//! The only permitted nesting is `:::buttons` inside `:::hero`.
//!
//! # Public API
//!
//! ```
//! use orbit_md::orbit_markdown::parser::parse;
//! use orbit_md::orbit_markdown::ast::OrbitNode;
//! use std::path::Path;
//!
//! let src = "# Hello\n\n:::note\nSomething.\n:::\n";
//! let nodes = parse(src, Path::new("index.md")).unwrap();
//! assert_eq!(nodes.len(), 2); // Markdown chunk + Callout
//! ```

use std::path::Path;

use crate::error::PageError;
use crate::orbit_markdown::ast::{
    ButtonItem, ButtonStyle, CalloutKind, FeatureItem, OrbitNode,
};
use crate::orbit_markdown::directives::{DirectiveKind, parse_opening_line, validate_attrs};

// ── Public entry point ────────────────────────────────────────────────────────

/// Parse a Markdown source string with Orbit directive extensions.
///
/// Returns a flat `Vec<OrbitNode>`. A post-parse pass groups consecutive
/// `Card` nodes into `CardGrid` nodes automatically.
///
/// # Errors
///
/// Returns the first structural error found:
/// - **D001** — unknown directive name (with typo suggestion)
/// - **D002** — missing required attribute
/// - **D003** — directive is still open at end of file
/// - **D006** — directive illegally nested inside another
pub fn parse(source: &str, path: &Path) -> Result<Vec<OrbitNode>, PageError> {
    let nodes = scan(source, path)?;
    Ok(group_cards(nodes))
}

// ── Scanner state machine ─────────────────────────────────────────────────────

/// All possible states the line scanner can be in.
enum ScanState {
    /// Standard context — both Markdown and `:::directives` are active.
    Normal,
    /// Inside a fenced code block. Directives are suppressed until the
    /// matching fence close.
    CodeFence { marker: char, width: usize },
    /// Accumulating the body of an open `:::directive` block.
    InDirective(DirectiveCtx),
}

/// All data accumulated while inside an open directive.
struct DirectiveCtx {
    kind: DirectiveKind,
    attrs: std::collections::HashMap<String, String>,
    /// 1-indexed line where the opening `:::name` appeared. Used in D003.
    open_line: usize,
    /// Body lines between the opener and the closing `:::`.
    body_lines: Vec<String>,
    /// Populated when a `:::buttons` block is opened inside `:::hero`.
    nested_buttons: Option<NestedButtonsCtx>,
}

/// Context for a `:::buttons` block nested inside `:::hero`.
struct NestedButtonsCtx {
    /// 1-indexed line where `:::buttons` appeared (reserved for future errors).
    #[allow(dead_code)]
    open_line: usize,
    /// Lines collected between `:::buttons` and its closing `:::`.
    body_lines: Vec<String>,
    /// `true` once the inner `:::` that closes `:::buttons` is seen.
    closed: bool,
}

// ── Scanner loop ──────────────────────────────────────────────────────────────

fn scan(source: &str, path: &Path) -> Result<Vec<OrbitNode>, PageError> {
    let mut nodes: Vec<OrbitNode> = Vec::new();
    let mut md_buf = String::new();
    let mut state = ScanState::Normal;

    for (idx, line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        state = match state {
            ScanState::Normal => {
                handle_normal(line, trimmed, line_num, path, &mut nodes, &mut md_buf)?
            }
            ScanState::CodeFence { marker, width } => {
                handle_code_fence(line, trimmed, marker, width, &mut md_buf)?
            }
            ScanState::InDirective(ctx) => {
                handle_in_directive(line, trimmed, line_num, path, ctx, &mut nodes)?
            }
        };
    }

    // D003 — directive was never closed.
    if let ScanState::InDirective(ctx) = &state {
        return Err(PageError::at(
            path,
            ctx.open_line,
            1,
            format!(
                "directive '{}' opened on line {} was never closed",
                ctx.kind.as_str(),
                ctx.open_line
            ),
        ));
    }

    // Flush any trailing Markdown.
    if !md_buf.is_empty() {
        nodes.push(OrbitNode::Markdown(md_buf));
    }

    Ok(nodes)
}

// ── State handlers ────────────────────────────────────────────────────────────

/// Handle one source line in the `Normal` state.
fn handle_normal(
    line: &str,
    trimmed: &str,
    line_num: usize,
    path: &Path,
    nodes: &mut Vec<OrbitNode>,
    md_buf: &mut String,
) -> Result<ScanState, PageError> {
    // Code fence opener?
    if let Some((marker, width)) = detect_fence_open(trimmed) {
        md_buf.push_str(line);
        md_buf.push('\n');
        return Ok(ScanState::CodeFence { marker, width });
    }

    // Directive opener?
    if trimmed.starts_with(":::") {
        if let Some(dir_line) = parse_opening_line(line, line_num, path)? {
            validate_attrs(&dir_line.kind, &dir_line.attrs, path, line_num)?;
            // Flush buffered Markdown before entering the directive.
            if !md_buf.is_empty() {
                nodes.push(OrbitNode::Markdown(std::mem::take(md_buf)));
            }
            return Ok(ScanState::InDirective(DirectiveCtx {
                kind: dir_line.kind,
                attrs: dir_line.attrs,
                open_line: line_num,
                body_lines: Vec::new(),
                nested_buttons: None,
            }));
        }
        // Bare `:::` in Normal context is treated as plain Markdown text.
    }

    md_buf.push_str(line);
    md_buf.push('\n');
    Ok(ScanState::Normal)
}

/// Handle one source line in the `CodeFence` state.
///
/// Everything is raw text until the matching close fence is found.
fn handle_code_fence(
    line: &str,
    trimmed: &str,
    marker: char,
    width: usize,
    md_buf: &mut String,
) -> Result<ScanState, PageError> {
    md_buf.push_str(line);
    md_buf.push('\n');
    if is_fence_close(trimmed, marker, width) {
        Ok(ScanState::Normal)
    } else {
        Ok(ScanState::CodeFence { marker, width })
    }
}

/// Handle one source line in the `InDirective` state.
///
/// Takes ownership of `ctx` and returns the new state (often the same
/// `InDirective` with an updated context, or `Normal` after closing).
fn handle_in_directive(
    line: &str,
    trimmed: &str,
    line_num: usize,
    path: &Path,
    mut ctx: DirectiveCtx,
    nodes: &mut Vec<OrbitNode>,
) -> Result<ScanState, PageError> {
    // ── Closing marker `:::` ──────────────────────────────────────────────────
    if trimmed == ":::" {
        // If a nested :::buttons is still open, this closes it first.
        let closes_inner = ctx
            .nested_buttons
            .as_ref()
            .map(|nb| !nb.closed)
            .unwrap_or(false);

        if closes_inner {
            if let Some(ref mut nb) = ctx.nested_buttons {
                nb.closed = true;
            }
            return Ok(ScanState::InDirective(ctx));
        }

        // Close this directive and emit its node.
        nodes.push(build_node(ctx, path)?);
        return Ok(ScanState::Normal);
    }

    // ── Accumulate into open nested :::buttons ────────────────────────────────
    let in_open_nested = ctx
        .nested_buttons
        .as_ref()
        .map(|nb| !nb.closed)
        .unwrap_or(false);

    if in_open_nested {
        // No further nesting is allowed inside :::buttons.
        if trimmed.starts_with(":::") {
            if let Some(inner) = parse_opening_line(line, line_num, path)? {
                return Err(PageError::at(
                    path,
                    line_num,
                    1,
                    format!(
                        "directive '{}' cannot be nested inside 'buttons'",
                        inner.kind.as_str()
                    ),
                ));
            }
        }
        if let Some(ref mut nb) = ctx.nested_buttons {
            nb.body_lines.push(line.to_owned());
        }
        return Ok(ScanState::InDirective(ctx));
    }

    // ── Nested directive opener ───────────────────────────────────────────────
    if trimmed.starts_with(":::") {
        if let Some(inner) = parse_opening_line(line, line_num, path)? {
            // The only allowed nesting: :::buttons inside :::hero.
            if ctx.kind == DirectiveKind::Hero
                && inner.kind == DirectiveKind::Buttons
                && ctx.nested_buttons.is_none()
            {
                ctx.nested_buttons = Some(NestedButtonsCtx {
                    open_line: line_num,
                    body_lines: Vec::new(),
                    closed: false,
                });
                return Ok(ScanState::InDirective(ctx));
            }
            // D006 — all other nesting is illegal.
            return Err(PageError::at(
                path,
                line_num,
                1,
                format!(
                    "directive '{}' cannot be nested inside '{}'",
                    inner.kind.as_str(),
                    ctx.kind.as_str()
                ),
            ));
        }
    }

    // ── Regular body line ─────────────────────────────────────────────────────
    ctx.body_lines.push(line.to_owned());
    Ok(ScanState::InDirective(ctx))
}

// ── Card grouping pass ────────────────────────────────────────────────────────

/// Group consecutive `Card` nodes into a `CardGrid` node.
///
/// A single isolated `Card` is left unwrapped. Only two or more adjacent
/// cards become a `CardGrid`.
fn group_cards(nodes: Vec<OrbitNode>) -> Vec<OrbitNode> {
    let mut result: Vec<OrbitNode> = Vec::with_capacity(nodes.len());
    let mut card_run: Vec<OrbitNode> = Vec::new();

    for node in nodes {
        if matches!(node, OrbitNode::Card { .. }) {
            card_run.push(node);
        } else {
            flush_card_run(&mut card_run, &mut result);
            result.push(node);
        }
    }
    flush_card_run(&mut card_run, &mut result);
    result
}

fn flush_card_run(run: &mut Vec<OrbitNode>, out: &mut Vec<OrbitNode>) {
    match run.len() {
        0 => {}
        1 => out.push(run.remove(0)),
        _ => out.push(OrbitNode::CardGrid(std::mem::take(run))),
    }
}

// ── Node builder ──────────────────────────────────────────────────────────────

/// Convert a completed `DirectiveCtx` into the corresponding `OrbitNode`.
fn build_node(ctx: DirectiveCtx, _path: &Path) -> Result<OrbitNode, PageError> {
    // Destructure so all fields are available after matching on `kind`.
    let DirectiveCtx {
        kind,
        attrs,
        body_lines: body,
        nested_buttons,
        ..
    } = ctx;

    let node = match kind {
        // ── Callouts ──────────────────────────────────────────────────────────
        DirectiveKind::Note => OrbitNode::Callout {
            kind: CalloutKind::Note,
            title: attrs.get("title").cloned(),
            children: body_to_children(&body),
        },
        DirectiveKind::Info => OrbitNode::Callout {
            kind: CalloutKind::Info,
            title: attrs.get("title").cloned(),
            children: body_to_children(&body),
        },
        DirectiveKind::Warning => OrbitNode::Callout {
            kind: CalloutKind::Warning,
            title: attrs.get("title").cloned(),
            children: body_to_children(&body),
        },
        DirectiveKind::Danger => OrbitNode::Callout {
            kind: CalloutKind::Danger,
            title: attrs.get("title").cloned(),
            children: body_to_children(&body),
        },
        DirectiveKind::Success => OrbitNode::Callout {
            kind: CalloutKind::Success,
            title: attrs.get("title").cloned(),
            children: body_to_children(&body),
        },
        DirectiveKind::Tip => OrbitNode::Callout {
            kind: CalloutKind::Tip,
            title: attrs.get("title").cloned(),
            children: body_to_children(&body),
        },

        // ── Structure ─────────────────────────────────────────────────────────
        DirectiveKind::Steps => OrbitNode::Steps {
            items: parse_list_items(&body),
        },
        DirectiveKind::Card => OrbitNode::Card {
            title: attrs.get("title").cloned(),
            href: attrs.get("href").cloned(),
            children: body_to_children(&body),
        },
        DirectiveKind::Features => OrbitNode::Features {
            items: parse_feature_items(&parse_list_items(&body)),
        },

        // ── Navigation ────────────────────────────────────────────────────────
        DirectiveKind::Buttons => OrbitNode::Buttons {
            items: parse_button_items(&body),
        },
        DirectiveKind::Hero => {
            let actions = nested_buttons.map(|nb| {
                Box::new(OrbitNode::Buttons {
                    items: parse_button_items(&nb.body_lines),
                })
            });
            OrbitNode::Hero {
                title: attrs.get("title").cloned().unwrap_or_default(),
                subtitle: attrs.get("subtitle").cloned(),
                body: body_to_children(&body),
                actions,
            }
        }
        DirectiveKind::NavGroup => OrbitNode::NavGroup {
            title: attrs.get("title").cloned().unwrap_or_default(),
            links: parse_nav_links(&parse_list_items(&body)),
        },

        // ── Media ─────────────────────────────────────────────────────────────
        DirectiveKind::Figure => OrbitNode::Figure {
            src: attrs.get("src").cloned().unwrap_or_default(),
            alt: attrs.get("alt").cloned().unwrap_or_default(),
            caption: attrs.get("caption").cloned(),
        },
    };

    Ok(node)
}

// ── Body parsing helpers ──────────────────────────────────────────────────────

/// Wrap body lines as a single `Markdown` child, or return empty vec.
fn body_to_children(body: &[String]) -> Vec<OrbitNode> {
    let text = body.join("\n");
    if text.trim().is_empty() {
        vec![]
    } else {
        vec![OrbitNode::Markdown(text)]
    }
}

/// Extract content from Markdown list items (`- `, `* `, `+ ` markers).
///
/// Non-list lines are silently skipped.
fn parse_list_items(body: &[String]) -> Vec<String> {
    body.iter()
        .filter_map(|line| {
            let t = line.trim();
            t.strip_prefix("- ")
                .or_else(|| t.strip_prefix("* "))
                .or_else(|| t.strip_prefix("+ "))
                .map(str::to_owned)
        })
        .collect()
}

/// Parse `:::features` list items into `FeatureItem` values.
///
/// Expected format per item: `**Title**: body text`.
/// The bold text up to the closing `**` becomes the title; the rest (after
/// any leading `:` or `-`) becomes the body. Items that don't follow the
/// pattern get an empty title.
fn parse_feature_items(items: &[String]) -> Vec<FeatureItem> {
    items
        .iter()
        .map(|item| {
            if let Some(rest) = item.strip_prefix("**") {
                if let Some(end) = rest.find("**") {
                    let title = rest[..end].to_owned();
                    let after = rest[end + 2..]
                        .trim_start_matches(|c: char| c == ':' || c == '-' || c == ' ');
                    return FeatureItem {
                        title,
                        body: after.to_owned(),
                    };
                }
            }
            FeatureItem {
                title: String::new(),
                body: item.clone(),
            }
        })
        .collect()
}

/// Parse `:::buttons` body lines into `ButtonItem` values.
///
/// Expected format per line: `[Label](href) style`.
/// Lines that do not start with `[` are silently skipped.
fn parse_button_items(lines: &[String]) -> Vec<ButtonItem> {
    lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let (label, href, rest) = parse_md_link_with_rest(line.trim())?;
            let style = ButtonStyle::from_str(rest.trim());
            Some(ButtonItem { label, href, style })
        })
        .collect()
}

/// Parse `:::nav-group` list items into `(label, href)` pairs.
///
/// Expected format per item: `[Label](href)`.
/// Items that do not parse as a Markdown link are silently skipped.
fn parse_nav_links(items: &[String]) -> Vec<(String, String)> {
    items
        .iter()
        .filter_map(|item| {
            let (label, href, _) = parse_md_link_with_rest(item.trim())?;
            Some((label, href))
        })
        .collect()
}

/// Parse a Markdown inline link `[label](href)` from the start of `s`.
///
/// Returns `(label, href, remaining_text)`.
/// Returns `None` when `s` does not start with `[`.
fn parse_md_link_with_rest(s: &str) -> Option<(String, String, &str)> {
    let after_open = s.strip_prefix('[')?;
    let label_end = after_open.find(']')?;
    let label = after_open[..label_end].to_owned();
    let after_label = after_open[label_end + 1..].strip_prefix('(')?;
    let href_end = after_label.find(')')?;
    let href = after_label[..href_end].to_owned();
    let rest = &after_label[href_end + 1..];
    Some((label, href, rest))
}

// ── Code fence helpers ────────────────────────────────────────────────────────

/// Detect a code fence opening from a trimmed line.
///
/// Returns `(marker, width)` for a run of ≥ 3 identical `` ` `` or `~`
/// characters. Returns `None` for anything else.
fn detect_fence_open(trimmed: &str) -> Option<(char, usize)> {
    for marker in ['`', '~'] {
        if trimmed.starts_with(marker) {
            let width = trimmed.chars().take_while(|&c| c == marker).count();
            if width >= 3 {
                return Some((marker, width));
            }
        }
    }
    None
}

/// Return `true` when `trimmed` is a valid closing fence for `(marker, width)`.
///
/// A closing fence consists entirely of `marker` characters and must be at
/// least as wide as the opening fence.
fn is_fence_close(trimmed: &str, marker: char, width: usize) -> bool {
    let count = trimmed.chars().take_while(|&c| c == marker).count();
    count >= width && trimmed.chars().all(|c| c == marker)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orbit_markdown::ast::*;

    /// Shorthand: parse source, expect success.
    fn p(src: &str) -> Vec<OrbitNode> {
        parse(src, Path::new("test.md")).expect("parse failed")
    }

    /// Shorthand: parse source, expect a `PageError`.
    fn p_err(src: &str) -> PageError {
        parse(src, Path::new("test.md")).expect_err("expected an error")
    }

    // ── Plain Markdown ────────────────────────────────────────────────────────

    #[test]
    fn plain_markdown_is_single_node() {
        let nodes = p("# Hello\n\nSome text.\n");
        assert_eq!(nodes.len(), 1);
        assert!(matches!(nodes[0], OrbitNode::Markdown(_)));
    }

    #[test]
    fn empty_source_produces_no_nodes() {
        assert!(p("").is_empty());
    }

    // ── Callouts ──────────────────────────────────────────────────────────────

    #[test]
    fn note_without_title() {
        let nodes = p(":::note\nSomething.\n:::\n");
        match &nodes[0] {
            OrbitNode::Callout { kind, title, children } => {
                assert_eq!(*kind, CalloutKind::Note);
                assert!(title.is_none());
                assert_eq!(children.len(), 1);
            }
            _ => panic!("expected Callout"),
        }
    }

    #[test]
    fn warning_with_title() {
        let nodes = p(":::warning title=\"Watch out\"\nDangerous.\n:::\n");
        match &nodes[0] {
            OrbitNode::Callout { kind, title, .. } => {
                assert_eq!(*kind, CalloutKind::Warning);
                assert_eq!(title.as_deref(), Some("Watch out"));
            }
            _ => panic!("expected Callout"),
        }
    }

    #[test]
    fn all_six_callout_kinds_parse() {
        for (name, expected) in [
            ("note",    CalloutKind::Note),
            ("info",    CalloutKind::Info),
            ("warning", CalloutKind::Warning),
            ("danger",  CalloutKind::Danger),
            ("success", CalloutKind::Success),
            ("tip",     CalloutKind::Tip),
        ] {
            let src = format!(":::{name}\nbody\n:::\n");
            let nodes = p(&src);
            match &nodes[0] {
                OrbitNode::Callout { kind, .. } => assert_eq!(*kind, expected, "kind mismatch for {name}"),
                _ => panic!("expected Callout for {name}"),
            }
        }
    }

    // ── Steps ─────────────────────────────────────────────────────────────────

    #[test]
    fn steps_parses_list_items() {
        let nodes = p(":::steps\n- Install Rust\n- Run cargo\n:::\n");
        match &nodes[0] {
            OrbitNode::Steps { items } => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], "Install Rust");
                assert_eq!(items[1], "Run cargo");
            }
            _ => panic!("expected Steps"),
        }
    }

    // ── Card and CardGrid ─────────────────────────────────────────────────────

    #[test]
    fn single_card_not_wrapped_in_grid() {
        let nodes = p(":::card title=\"One\"\nBody.\n:::\n");
        assert!(matches!(nodes[0], OrbitNode::Card { .. }));
    }

    #[test]
    fn two_adjacent_cards_become_grid() {
        let nodes = p(":::card title=\"A\"\nA.\n:::\n:::card title=\"B\"\nB.\n:::\n");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            OrbitNode::CardGrid(cards) => assert_eq!(cards.len(), 2),
            _ => panic!("expected CardGrid"),
        }
    }

    #[test]
    fn three_adjacent_cards_become_grid() {
        let nodes = p(
            ":::card title=\"A\"\nA.\n:::\n\
             :::card title=\"B\"\nB.\n:::\n\
             :::card title=\"C\"\nC.\n:::\n",
        );
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            OrbitNode::CardGrid(cards) => assert_eq!(cards.len(), 3),
            _ => panic!("expected CardGrid"),
        }
    }

    #[test]
    fn cards_broken_by_other_node_are_not_grouped() {
        // card — callout — card: two separate cards, no grid.
        let nodes = p(
            ":::card title=\"A\"\nA.\n:::\n\
             :::note\nBreaker.\n:::\n\
             :::card title=\"B\"\nB.\n:::\n",
        );
        assert_eq!(nodes.len(), 3);
        assert!(matches!(nodes[0], OrbitNode::Card { .. }));
        assert!(matches!(nodes[1], OrbitNode::Callout { .. }));
        assert!(matches!(nodes[2], OrbitNode::Card { .. }));
    }

    #[test]
    fn card_with_href_stored() {
        let nodes = p(":::card title=\"Docs\" href=\"/docs\"\nBody.\n:::\n");
        match &nodes[0] {
            OrbitNode::Card { href, .. } => assert_eq!(href.as_deref(), Some("/docs")),
            _ => panic!("expected Card"),
        }
    }

    // ── Features ──────────────────────────────────────────────────────────────

    #[test]
    fn features_parses_bold_title_colon_body() {
        let nodes = p(
            ":::features\n\
             - **Markdown-native**: Write plain files.\n\
             - **Zero JS**: Static HTML.\n\
             :::\n",
        );
        match &nodes[0] {
            OrbitNode::Features { items } => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].title, "Markdown-native");
                assert_eq!(items[0].body,  "Write plain files.");
                assert_eq!(items[1].title, "Zero JS");
                assert_eq!(items[1].body,  "Static HTML.");
            }
            _ => panic!("expected Features"),
        }
    }

    // ── Buttons ───────────────────────────────────────────────────────────────

    #[test]
    fn buttons_parses_label_href_style() {
        let nodes = p(
            ":::buttons\n\
             [Get started](/docs) primary\n\
             [GitHub](https://github.com) secondary\n\
             :::\n",
        );
        match &nodes[0] {
            OrbitNode::Buttons { items } => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].label, "Get started");
                assert_eq!(items[0].href,  "/docs");
                assert_eq!(items[0].style, ButtonStyle::Primary);
                assert_eq!(items[1].style, ButtonStyle::Secondary);
            }
            _ => panic!("expected Buttons"),
        }
    }

    #[test]
    fn buttons_missing_style_defaults_to_primary() {
        let nodes = p(":::buttons\n[Go](/go)\n:::\n");
        match &nodes[0] {
            OrbitNode::Buttons { items } => assert_eq!(items[0].style, ButtonStyle::Primary),
            _ => panic!("expected Buttons"),
        }
    }

    // ── Hero ──────────────────────────────────────────────────────────────────

    #[test]
    fn hero_title_and_subtitle() {
        let nodes = p(":::hero title=\"Orbit\" subtitle=\"Fast sites\"\nDesc.\n:::\n");
        match &nodes[0] {
            OrbitNode::Hero { title, subtitle, body, actions } => {
                assert_eq!(title, "Orbit");
                assert_eq!(subtitle.as_deref(), Some("Fast sites"));
                assert_eq!(body.len(), 1);
                assert!(actions.is_none());
            }
            _ => panic!("expected Hero"),
        }
    }

    #[test]
    fn hero_with_nested_buttons() {
        let src = ":::hero title=\"Orbit\"\nDesc.\n:::buttons\n[Start](/docs) primary\n:::\n:::\n";
        let nodes = p(src);
        match &nodes[0] {
            OrbitNode::Hero { actions, .. } => match actions.as_deref() {
                Some(OrbitNode::Buttons { items }) => {
                    assert_eq!(items.len(), 1);
                    assert_eq!(items[0].label, "Start");
                }
                _ => panic!("expected Buttons inside Hero"),
            },
            _ => panic!("expected Hero"),
        }
    }

    // ── NavGroup ──────────────────────────────────────────────────────────────

    #[test]
    fn nav_group_parses_links() {
        let nodes = p(
            ":::nav-group title=\"Docs\"\n\
             - [Config](/docs/config)\n\
             - [CLI](/docs/cli)\n\
             :::\n",
        );
        match &nodes[0] {
            OrbitNode::NavGroup { title, links } => {
                assert_eq!(title, "Docs");
                assert_eq!(links.len(), 2);
                assert_eq!(links[0], ("Config".into(), "/docs/config".into()));
            }
            _ => panic!("expected NavGroup"),
        }
    }

    // ── Figure ────────────────────────────────────────────────────────────────

    #[test]
    fn figure_all_attrs() {
        let nodes =
            p(":::figure src=\"/img.png\" alt=\"Diagram\" caption=\"Fig 1.\"\n:::\n");
        match &nodes[0] {
            OrbitNode::Figure { src, alt, caption } => {
                assert_eq!(src, "/img.png");
                assert_eq!(alt, "Diagram");
                assert_eq!(caption.as_deref(), Some("Fig 1."));
            }
            _ => panic!("expected Figure"),
        }
    }

    #[test]
    fn figure_without_caption() {
        let nodes = p(":::figure src=\"/img.png\" alt=\"Alt\"\n:::\n");
        match &nodes[0] {
            OrbitNode::Figure { caption, .. } => assert!(caption.is_none()),
            _ => panic!("expected Figure"),
        }
    }

    // ── Code fence rule ───────────────────────────────────────────────────────

    #[test]
    fn directive_inside_backtick_fence_is_ignored() {
        let src = "```\n:::note\nThis is code.\n:::\n```\n";
        let nodes = p(src);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            OrbitNode::Markdown(text) => assert!(text.contains(":::note")),
            _ => panic!("expected Markdown passthrough"),
        }
    }

    #[test]
    fn directive_inside_tilde_fence_is_ignored() {
        let src = "~~~\n:::warning\nCode.\n:::\n~~~\n";
        let nodes = p(src);
        assert_eq!(nodes.len(), 1);
        assert!(matches!(nodes[0], OrbitNode::Markdown(_)));
    }

    #[test]
    fn four_backtick_fence_recognised() {
        let src = "````\n:::note\nCode.\n:::\n````\n";
        let nodes = p(src);
        assert!(matches!(nodes[0], OrbitNode::Markdown(_)));
    }

    // ── Mixed Markdown and directives ─────────────────────────────────────────

    #[test]
    fn markdown_before_and_after_directive() {
        let src = "# Before\n\n:::note\nNote.\n:::\n\n# After\n";
        let nodes = p(src);
        assert_eq!(nodes.len(), 3);
        assert!(matches!(nodes[0], OrbitNode::Markdown(_)));
        assert!(matches!(nodes[1], OrbitNode::Callout { .. }));
        assert!(matches!(nodes[2], OrbitNode::Markdown(_)));
    }

    // ── Error cases ───────────────────────────────────────────────────────────

    #[test]
    fn unclosed_directive_d003() {
        let err = p_err(":::note\nNever closed.\n");
        assert_eq!(err.line, Some(1));
        assert!(err.to_string().contains("was never closed"));
    }

    #[test]
    fn unknown_directive_d001_with_suggestion() {
        let err = p_err(":::warnnig\nbody\n:::\n");
        assert!(err.to_string().contains("warnnig"));
        // Suggestion for 'warnnig' should be 'warning'.
        assert!(err.to_string().contains("warning"));
    }

    #[test]
    fn missing_required_attr_d002() {
        let err = p_err(":::card\nbody\n:::\n");
        assert!(err.to_string().contains("title"));
    }

    #[test]
    fn illegal_nesting_d006() {
        let err = p_err(":::note\n:::warning\nnested\n:::\n:::\n");
        assert!(err.to_string().contains("cannot be nested"));
    }
}
