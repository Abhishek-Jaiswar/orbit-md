//! JSX-style component tag scanner for Markdown bodies.
//!
//! Detects PascalCase tags (React-like components) and leaves lowercase HTML
//! tags untouched.

use std::collections::HashMap;
use std::ops::Range;

use crate::error::PageError;

/// A parsed component invocation inside Markdown source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedComponent {
    /// Component name (PascalCase), e.g. `Alert`.
    pub name: String,
    /// Attribute key/value pairs from the opening tag.
    pub attrs: HashMap<String, String>,
    /// Inner Markdown for block components; empty for self-closing tags.
    pub inner: String,
    /// Whether the tag is self-closing (`<Button />`).
    pub self_closing: bool,
    /// Byte range of the entire tag invocation in the source string.
    pub span: Range<usize>,
}

/// Returns `true` when `name` is a valid PascalCase component identifier.
pub fn is_component_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_uppercase() {
        return false;
    }
    name.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Finds the next PascalCase component tag at or after `from`.
pub fn find_next_component(source: &str, from: usize) -> Option<ParsedComponent> {
    let bytes = source.as_bytes();
    let mut index = from;

    while index < bytes.len() {
        if bytes[index] != b'<' {
            index += 1;
            continue;
        }

        if let Some(component) = try_parse_component(source, index) {
            return Some(component);
        }

        index += 1;
    }

    None
}

fn try_parse_component(source: &str, open: usize) -> Option<ParsedComponent> {
    let rest = &source[open + 1..];

    // Closing tags and HTML comments are not components.
    if rest.starts_with('/') || rest.starts_with('!') {
        return None;
    }

    let (name, after_name) = read_identifier(rest)?;
    if !is_component_name(name) {
        return None;
    }

    let (attrs, after_attrs) = parse_attributes(after_name)?;
    let after_attrs = skip_whitespace(after_attrs);

    if after_attrs.starts_with("/>") {
        let end = open + 1 + (after_attrs.as_ptr() as usize - rest.as_ptr() as usize) + 2;
        return Some(ParsedComponent {
            name: name.to_owned(),
            attrs,
            inner: String::new(),
            self_closing: true,
            span: open..end,
        });
    }

    if !after_attrs.starts_with('>') {
        return None;
    }

    let content_start = open + 1 + (after_attrs.as_ptr() as usize - rest.as_ptr() as usize) + 1;
    let close_tag = format!("</{name}>");
    let (inner, close_start) = read_until_matching_close(source, content_start, name).ok()?;
    let end = close_start + close_tag.len();

    Some(ParsedComponent {
        name: name.to_owned(),
        attrs,
        inner,
        self_closing: false,
        span: open..end,
    })
}

fn read_identifier(input: &str) -> Option<(&str, &str)> {
    let mut end = 0;
    for (i, ch) in input.char_indices() {
        if ch.is_ascii_alphanumeric() {
            end = i + ch.len_utf8();
        } else {
            break;
        }
    }

    if end == 0 {
        return None;
    }

    Some((&input[..end], &input[end..]))
}

fn skip_whitespace(input: &str) -> &str {
    input.trim_start()
}

fn parse_attributes(input: &str) -> Option<(HashMap<String, String>, &str)> {
    let mut attrs = HashMap::new();
    let mut cursor = skip_whitespace(input);

    loop {
        cursor = skip_whitespace(cursor);
        if cursor.is_empty() || cursor.starts_with('>') || cursor.starts_with('/') {
            return Some((attrs, cursor));
        }

        let (key, after_key) = read_attribute_key(cursor)?;
        cursor = skip_whitespace(after_key);

        if cursor.starts_with('=') {
            cursor = skip_whitespace(&cursor[1..]);
            let (value, after_value) = read_attribute_value(cursor)?;
            attrs.insert(key.to_owned(), value);
            cursor = after_value;
        } else {
            attrs.insert(key.to_owned(), "true".to_owned());
        }
    }
}

fn read_attribute_key(input: &str) -> Option<(&str, &str)> {
    let mut end = 0;
    for (i, ch) in input.char_indices() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            end = i + ch.len_utf8();
        } else {
            break;
        }
    }

    if end == 0 {
        return None;
    }

    Some((&input[..end], &input[end..]))
}

fn read_attribute_value(input: &str) -> Option<(String, &str)> {
    if let Some(rest) = input.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some((rest[..end].to_owned(), &rest[end + 1..]));
    }

    if let Some(rest) = input.strip_prefix('\'') {
        let end = rest.find('\'')?;
        return Some((rest[..end].to_owned(), &rest[end + 1..]));
    }

    let end = input
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(input.len());
    Some((input[..end].to_owned(), &input[end..]))
}

fn read_until_matching_close(
    source: &str,
    start: usize,
    name: &str,
) -> Result<(String, usize), ()> {
    let open_needle = format!("<{name}");
    let close_tag = format!("</{name}>");
    let mut depth = 0;
    let mut cursor = start;

    while cursor < source.len() {
        let Some(next_open) = source[cursor..].find('<') else {
            break;
        };
        let abs = cursor + next_open;

        if source[abs..].starts_with(&close_tag) {
            if depth == 0 {
                let inner = source[start..abs].to_owned();
                return Ok((inner, abs));
            }
            depth -= 1;
            cursor = abs + close_tag.len();
            continue;
        }

        if source[abs..].starts_with(&open_needle) {
            let after = &source[abs + open_needle.len()..];
            let boundary = after
                .chars()
                .next()
                .is_none_or(|c| c.is_whitespace() || c == '>' || c == '/');
            if boundary {
                depth += 1;
            }
        }

        cursor = abs + 1;
    }

    Err(())
}

/// Validates a parsed component and returns a [`PageError`] on malformed tags.
pub fn validate_component(
    component: &ParsedComponent,
    path: &std::path::Path,
) -> Result<(), PageError> {
    if component.name.is_empty() {
        return Err(PageError::new(
            path,
            "encountered empty component tag name".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pascal_case_names() {
        assert!(is_component_name("Alert"));
        assert!(is_component_name("Button"));
        assert!(!is_component_name("div"));
        assert!(!is_component_name(""));
    }

    #[test]
    fn parses_self_closing_component() {
        let source = r#"Intro <Button href="/docs" label="Go" /> tail"#;
        let component = find_next_component(source, 0).unwrap();
        assert_eq!(component.name, "Button");
        assert!(component.self_closing);
        assert_eq!(component.attrs.get("href"), Some(&"/docs".to_owned()));
        assert_eq!(component.attrs.get("label"), Some(&"Go".to_owned()));
    }

    #[test]
    fn parses_block_component_with_attributes() {
        let source = r#"<Alert type="warning" title="Heads up">Hello **world**</Alert>"#;
        let component = find_next_component(source, 0).unwrap();
        assert_eq!(component.name, "Alert");
        assert!(!component.self_closing);
        assert_eq!(component.inner, "Hello **world**");
        assert_eq!(component.attrs.get("type"), Some(&"warning".to_owned()));
    }

    #[test]
    fn ignores_lowercase_html_tags() {
        let source = "<div class=\"x\">plain</div>";
        assert!(find_next_component(source, 0).is_none());
    }

    #[test]
    fn parses_nested_components_inner_first() {
        let source = r#"<Card title="Outer"><Alert type="info">Nested</Alert></Card>"#;
        let outer = find_next_component(source, 0).unwrap();
        assert_eq!(outer.name, "Card");
        assert!(outer.inner.contains("<Alert"));
        let inner = find_next_component(&outer.inner, 0).unwrap();
        assert_eq!(inner.name, "Alert");
    }
}
