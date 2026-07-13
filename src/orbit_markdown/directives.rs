//! Directive registry, attribute parsing, and validation.
//!
//! This module is the single source of truth for:
//!
//! - Which directive names are valid (`DirectiveKind`)
//! - Which attributes each directive requires or accepts
//! - How to parse `key="value"` attribute strings
//! - How to produce helpful "did you mean?" suggestions for typos

use std::collections::HashMap;
use std::path::Path;

use crate::error::PageError;

/// All directive names recognised by the Orbit v1.0.0 parser.
///
/// Used for typo suggestions in D001 errors.
const ALL_DIRECTIVES: &[&str] = &[
    "note",
    "info",
    "warning",
    "danger",
    "success",
    "tip",
    "steps",
    "card",
    "features",
    "buttons",
    "hero",
    "nav-group",
    "figure",
];

// ── DirectiveKind ─────────────────────────────────────────────────────────────

/// The kind of an Orbit directive, derived from the name on the opening line.
///
/// Every valid directive in the v1.0.0 spec has exactly one variant here.
/// Unknown names are rejected by [`parse_opening_line`] with a D001 error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectiveKind {
    // ── Callouts ──
    /// `:::note`
    Note,
    /// `:::info`
    Info,
    /// `:::warning`
    Warning,
    /// `:::danger`
    Danger,
    /// `:::success`
    Success,
    /// `:::tip`
    Tip,

    // ── Structure ──
    /// `:::steps`
    Steps,
    /// `:::card`
    Card,
    /// `:::features`
    Features,

    // ── Navigation ──
    /// `:::buttons`
    Buttons,
    /// `:::hero`
    Hero,
    /// `:::nav-group`
    NavGroup,

    // ── Media ──
    /// `:::figure`
    Figure,
}

impl DirectiveKind {
    /// Map a directive name string to a `DirectiveKind`.
    ///
    /// Returns `None` when `name` is not a known directive. The caller is
    /// responsible for turning `None` into a D001 error with a suggestion.
    pub fn from_str(name: &str) -> Option<Self> {
        match name {
            "note" => Some(Self::Note),
            "info" => Some(Self::Info),
            "warning" => Some(Self::Warning),
            "danger" => Some(Self::Danger),
            "success" => Some(Self::Success),
            "tip" => Some(Self::Tip),
            "steps" => Some(Self::Steps),
            "card" => Some(Self::Card),
            "features" => Some(Self::Features),
            "buttons" => Some(Self::Buttons),
            "hero" => Some(Self::Hero),
            "nav-group" => Some(Self::NavGroup),
            "figure" => Some(Self::Figure),
            _ => None,
        }
    }

    /// Returns the canonical string name for this directive kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Success => "success",
            Self::Tip => "tip",
            Self::Steps => "steps",
            Self::Card => "card",
            Self::Features => "features",
            Self::Buttons => "buttons",
            Self::Hero => "hero",
            Self::NavGroup => "nav-group",
            Self::Figure => "figure",
        }
    }

    /// Attribute names that MUST be present for this directive.
    ///
    /// The validator returns a D002 error for any missing required attribute.
    pub fn required_attrs(&self) -> &'static [&'static str] {
        match self {
            Self::Card => &["title"],
            Self::Hero => &["title"],
            Self::NavGroup => &["title"],
            Self::Figure => &["src", "alt"],
            // All other directives have no required attributes.
            _ => &[],
        }
    }

    /// Attribute names that MAY be present for this directive.
    ///
    /// Not enforced by the validator — listed here for documentation and
    /// future tooling (e.g. editor completion).
    pub fn optional_attrs(&self) -> &'static [&'static str] {
        match self {
            Self::Note | Self::Info | Self::Warning | Self::Danger | Self::Success | Self::Tip => {
                &["title"]
            }
            Self::Card => &["href"],
            Self::Hero => &["subtitle"],
            Self::Figure => &["caption"],
            // Steps, Features, Buttons, NavGroup accept no optional attrs.
            _ => &[],
        }
    }

    /// Returns `true` when this directive is self-closing (has no body).
    ///
    /// Self-closing directives still require a closing `:::` line in the
    /// source, but the body between the two markers is ignored.
    pub fn is_self_closing(&self) -> bool {
        matches!(self, Self::Figure)
    }
}

// ── DirectiveLine ─────────────────────────────────────────────────────────────

/// The parsed result of a directive opening line.
///
/// Produced by [`parse_opening_line`]. Consumed by the parser to open a new
/// directive context and by the validator to check required attributes.
#[derive(Debug, Clone)]
pub struct DirectiveLine {
    /// Which directive was opened.
    pub kind: DirectiveKind,
    /// All parsed key/value attributes from the opening line.
    pub attrs: HashMap<String, String>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Try to parse a directive opening line such as `:::warning title="X"`.
///
/// Returns:
/// - `Ok(None)` — the line does not start with `:::`, or is a bare `:::`
///   closing marker.
/// - `Ok(Some(DirectiveLine))` — successfully parsed.
/// - `Err(PageError)` — the name is not a known directive (D001 error).
///
/// # Examples
///
/// ```
/// use orbit_md::orbit_markdown::directives::parse_opening_line;
/// use std::path::Path;
///
/// let path = Path::new("content/index.md");
/// let result = parse_opening_line(":::warning title=\"Heads up\"", 5, path).unwrap();
/// let line = result.unwrap();
/// assert_eq!(line.attrs.get("title").map(String::as_str), Some("Heads up"));
/// ```
pub fn parse_opening_line(
    line: &str,
    line_num: usize,
    path: &Path,
) -> Result<Option<DirectiveLine>, PageError> {
    let trimmed = line.trim();

    // Must start with `:::`.
    let rest = match trimmed.strip_prefix(":::") {
        Some(r) => r.trim_start(),
        None => return Ok(None),
    };

    // Bare `:::` is a closing marker — not an opener.
    if rest.is_empty() {
        return Ok(None);
    }

    // Split into the name token and the rest (attribute string).
    let (name, attr_str) = rest
        .split_once(|c: char| c.is_ascii_whitespace())
        .unwrap_or((rest, ""));

    // Resolve the name to a known directive, or emit D001.
    let kind = DirectiveKind::from_str(name).ok_or_else(|| {
        let hint = suggest(name)
            .map(|s| format!("; did you mean '{s}'?"))
            .unwrap_or_default();
        PageError::at(
            path,
            line_num,
            1,
            format!("unknown directive '{name}'{hint}"),
        )
    })?;

    let attrs = parse_attrs(attr_str.trim());

    Ok(Some(DirectiveLine { kind, attrs }))
}

/// Validate that all required attributes for `kind` are present in `attrs`.
///
/// Returns a D002 error naming the first missing required attribute.
///
/// # Examples
///
/// ```
/// use orbit_md::orbit_markdown::directives::{DirectiveKind, validate_attrs};
/// use std::collections::HashMap;
/// use std::path::Path;
///
/// // card requires 'title'
/// let attrs = HashMap::new();
/// let err = validate_attrs(&DirectiveKind::Card, &attrs, Path::new("p.md"), 3).unwrap_err();
/// assert!(err.to_string().contains("title"));
/// ```
pub fn validate_attrs(
    kind: &DirectiveKind,
    attrs: &HashMap<String, String>,
    path: &Path,
    line_num: usize,
) -> Result<(), PageError> {
    for &required in kind.required_attrs() {
        if !attrs.contains_key(required) {
            return Err(PageError::at(
                path,
                line_num,
                1,
                format!(
                    "directive '{}' requires attribute '{required}'",
                    kind.as_str()
                ),
            ));
        }
    }
    Ok(())
}

/// Parse a `key="value" key2='value2' flag` attribute string into a map.
///
/// - Double-quoted values: `key="hello world"`
/// - Single-quoted values: `key='hello world'`
/// - Bare values (no spaces): `key=hello`
/// - Boolean flags (no `=`): `disabled` → stored as `"true"`
///
/// # Examples
///
/// ```
/// use orbit_md::orbit_markdown::directives::parse_attrs;
///
/// let attrs = parse_attrs(r#"title="My Title" href="/docs" flag"#);
/// assert_eq!(attrs.get("title").map(String::as_str), Some("My Title"));
/// assert_eq!(attrs.get("href").map(String::as_str),  Some("/docs"));
/// assert_eq!(attrs.get("flag").map(String::as_str),  Some("true"));
/// ```
pub fn parse_attrs(input: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    let mut cursor = input.trim();

    while !cursor.is_empty() {
        cursor = cursor.trim_start();
        if cursor.is_empty() {
            break;
        }

        // Read the key — ends at `=` or whitespace.
        let key_end = cursor
            .find(|c: char| c == '=' || c.is_ascii_whitespace())
            .unwrap_or(cursor.len());

        let key = &cursor[..key_end];
        if key.is_empty() {
            break;
        }
        cursor = &cursor[key_end..];

        // No `=` means this is a boolean flag.
        if !cursor.starts_with('=') {
            attrs.insert(key.to_owned(), "true".to_owned());
            continue;
        }

        cursor = &cursor[1..]; // skip `=`

        let (value, rest) = read_value(cursor);
        attrs.insert(key.to_owned(), value);
        cursor = rest;
    }

    attrs
}

/// Suggest the most similar known directive name for an unrecognised one.
///
/// Used in D001 error messages to guide authors past typos:
/// `unknown directive 'warnnig'; did you mean 'warning'?`
///
/// Returns `None` when no known name is within edit distance 2.
pub fn suggest(unknown: &str) -> Option<&'static str> {
    let mut best: Option<(&'static str, usize)> = None;
    for &name in ALL_DIRECTIVES {
        let dist = edit_distance(unknown, name);
        if dist <= 2 {
            match best {
                Some((_, d)) if dist >= d => {}
                _ => best = Some((name, dist)),
            }
        }
    }
    best.map(|(name, _)| name)
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Read one attribute value from the start of `input`.
///
/// Returns `(value, remaining_input)`. Handles double-quoted, single-quoted,
/// and bare (no whitespace) values.
fn read_value(input: &str) -> (String, &str) {
    // Double-quoted.
    if let Some(rest) = input.strip_prefix('"') {
        let end = rest.find('"').unwrap_or(rest.len());
        return (rest[..end].to_owned(), rest.get(end + 1..).unwrap_or(""));
    }
    // Single-quoted.
    if let Some(rest) = input.strip_prefix('\'') {
        let end = rest.find('\'').unwrap_or(rest.len());
        return (rest[..end].to_owned(), rest.get(end + 1..).unwrap_or(""));
    }
    // Bare value — ends at the next whitespace.
    let end = input
        .find(|c: char| c.is_ascii_whitespace())
        .unwrap_or(input.len());
    (input[..end].to_owned(), &input[end..])
}

/// Levenshtein edit distance between two strings.
///
/// Used by [`suggest`] to find the closest known directive name.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());

    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            dp[i][j] = if a[i - 1] == b[j - 1] {
                dp[i - 1][j - 1]
            } else {
                1 + dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1])
            };
        }
    }
    dp[m][n]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // ── DirectiveKind ──

    #[test]
    fn from_str_recognises_all_directives() {
        for name in ALL_DIRECTIVES {
            assert!(DirectiveKind::from_str(name).is_some(), "missing: {name}");
        }
    }

    #[test]
    fn from_str_rejects_unknown_name() {
        assert!(DirectiveKind::from_str("unknown").is_none());
        assert!(DirectiveKind::from_str("").is_none());
        assert!(DirectiveKind::from_str("NOTE").is_none()); // case-sensitive
    }

    #[test]
    fn as_str_round_trips() {
        for name in ALL_DIRECTIVES {
            let kind = DirectiveKind::from_str(name).unwrap();
            assert_eq!(kind.as_str(), *name);
        }
    }

    #[test]
    fn required_attrs_card_needs_title() {
        assert!(DirectiveKind::Card.required_attrs().contains(&"title"));
    }

    #[test]
    fn required_attrs_figure_needs_src_and_alt() {
        let req = DirectiveKind::Figure.required_attrs();
        assert!(req.contains(&"src"));
        assert!(req.contains(&"alt"));
    }

    #[test]
    fn callouts_have_no_required_attrs() {
        for kind in [
            DirectiveKind::Note,
            DirectiveKind::Info,
            DirectiveKind::Warning,
            DirectiveKind::Danger,
            DirectiveKind::Success,
            DirectiveKind::Tip,
        ] {
            assert!(
                kind.required_attrs().is_empty(),
                "{} should have no required attrs",
                kind.as_str()
            );
        }
    }

    #[test]
    fn figure_is_self_closing() {
        assert!(DirectiveKind::Figure.is_self_closing());
        assert!(!DirectiveKind::Note.is_self_closing());
        assert!(!DirectiveKind::Card.is_self_closing());
    }

    // ── parse_attrs ──

    #[test]
    fn parse_attrs_double_quoted() {
        let attrs = parse_attrs(r#"title="Hello World""#);
        assert_eq!(attrs.get("title").map(String::as_str), Some("Hello World"));
    }

    #[test]
    fn parse_attrs_single_quoted() {
        let attrs = parse_attrs("title='Hello World'");
        assert_eq!(attrs.get("title").map(String::as_str), Some("Hello World"));
    }

    #[test]
    fn parse_attrs_bare_value() {
        let attrs = parse_attrs("href=/docs");
        assert_eq!(attrs.get("href").map(String::as_str), Some("/docs"));
    }

    #[test]
    fn parse_attrs_boolean_flag() {
        let attrs = parse_attrs("disabled");
        assert_eq!(attrs.get("disabled").map(String::as_str), Some("true"));
    }

    #[test]
    fn parse_attrs_multiple_attributes() {
        let attrs = parse_attrs(r#"title="Fast Builds" href="/docs/perf" disabled"#);
        assert_eq!(attrs.get("title").map(String::as_str), Some("Fast Builds"));
        assert_eq!(attrs.get("href").map(String::as_str), Some("/docs/perf"));
        assert_eq!(attrs.get("disabled").map(String::as_str), Some("true"));
    }

    #[test]
    fn parse_attrs_empty_string() {
        let attrs = parse_attrs("");
        assert!(attrs.is_empty());
    }

    // ── parse_opening_line ──

    #[test]
    fn opening_line_bare_closing_marker_returns_none() {
        let result = parse_opening_line(":::", 1, Path::new("p.md")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn opening_line_not_a_directive_returns_none() {
        let result = parse_opening_line("# Heading", 1, Path::new("p.md")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn opening_line_bare_note_parses() {
        let result = parse_opening_line(":::note", 1, Path::new("p.md"))
            .unwrap()
            .unwrap();
        assert_eq!(result.kind, DirectiveKind::Note);
        assert!(result.attrs.is_empty());
    }

    #[test]
    fn opening_line_warning_with_title_parses() {
        let result = parse_opening_line(
            ":::warning title=\"Heads up\"",
            3,
            Path::new("content/index.md"),
        )
        .unwrap()
        .unwrap();
        assert_eq!(result.kind, DirectiveKind::Warning);
        assert_eq!(
            result.attrs.get("title").map(String::as_str),
            Some("Heads up")
        );
    }

    #[test]
    fn opening_line_unknown_directive_returns_error() {
        let err = parse_opening_line(":::warnnig", 7, Path::new("p.md")).unwrap_err();
        assert_eq!(err.line, Some(7));
        assert!(err.to_string().contains("warnnig"));
    }

    #[test]
    fn opening_line_unknown_directive_suggests_correction() {
        let err = parse_opening_line(":::warnnig", 1, Path::new("p.md")).unwrap_err();
        // Should suggest 'warning'
        assert!(err.to_string().contains("warning"));
    }

    #[test]
    fn opening_line_leading_whitespace_is_trimmed() {
        let result = parse_opening_line("  :::note  ", 1, Path::new("p.md"))
            .unwrap()
            .unwrap();
        assert_eq!(result.kind, DirectiveKind::Note);
    }

    // ── validate_attrs ──

    #[test]
    fn validate_attrs_card_without_title_errors() {
        let attrs = HashMap::new();
        let err = validate_attrs(&DirectiveKind::Card, &attrs, Path::new("p.md"), 5).unwrap_err();
        assert_eq!(err.line, Some(5));
        assert!(err.to_string().contains("title"));
    }

    #[test]
    fn validate_attrs_card_with_title_passes() {
        let mut attrs = HashMap::new();
        attrs.insert("title".to_owned(), "My Card".to_owned());
        validate_attrs(&DirectiveKind::Card, &attrs, Path::new("p.md"), 5).unwrap();
    }

    #[test]
    fn validate_attrs_figure_requires_both_src_and_alt() {
        // Only src — should fail on alt.
        let mut attrs = HashMap::new();
        attrs.insert("src".to_owned(), "/img.png".to_owned());
        let err = validate_attrs(&DirectiveKind::Figure, &attrs, Path::new("p.md"), 2).unwrap_err();
        assert!(err.to_string().contains("alt"));
    }

    // ── suggest ──

    #[test]
    fn suggest_common_typos() {
        assert_eq!(suggest("warnnig"), Some("warning"));
        assert_eq!(suggest("noe"), Some("note"));
        assert_eq!(suggest("crad"), Some("card"));
    }

    #[test]
    fn suggest_returns_none_for_gibberish() {
        assert!(suggest("xyzabc123").is_none());
    }

    #[test]
    fn suggest_exact_match_returns_itself() {
        assert_eq!(suggest("warning"), Some("warning"));
    }

    // ── edit_distance ──

    #[test]
    fn edit_distance_identical_strings() {
        assert_eq!(edit_distance("warning", "warning"), 0);
    }

    #[test]
    fn edit_distance_one_insertion() {
        assert_eq!(edit_distance("warnnig", "warning"), 2);
    }

    #[test]
    fn edit_distance_empty_strings() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("abc", ""), 3);
        assert_eq!(edit_distance("", "abc"), 3);
    }
}
