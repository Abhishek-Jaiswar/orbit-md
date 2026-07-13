//! Built-in CSS themes for Orbit Markdown directive rendering.
//!
//! Themes are embedded as `&'static str` constants so they can be injected
//! directly into `<style>` tags without any file I/O at build time.
//!
//! # Usage
//!
//! ```
//! use orbit_md::orbit_markdown::theme::theme_css;
//!
//! let css = theme_css("default").expect("default theme must exist");
//! assert!(css.contains("orbit-callout"));
//! ```
//!
//! All selectors are prefixed with `orbit-` so they never clash with user
//! stylesheets. Users can override individual selectors in their own CSS.

// ── Public API ────────────────────────────────────────────────────────────────

/// All built-in theme names available in v1.0.0.
pub const BUILT_IN_THEMES: &[&str] = &["default"];

/// Return the CSS string for a named built-in theme.
///
/// Returns `None` for unknown theme names. The caller decides how to handle
/// the fallback (e.g., warn and use "default", or skip the `<style>` tag).
///
/// # Examples
///
/// ```
/// use orbit_md::orbit_markdown::theme::theme_css;
///
/// assert!(theme_css("default").is_some());
/// assert!(theme_css("does-not-exist").is_none());
/// ```
pub fn theme_css(name: &str) -> Option<&'static str> {
    match name {
        "default" => Some(DEFAULT_CSS),
        _ => None,
    }
}

// ── Default theme ─────────────────────────────────────────────────────────────

const DEFAULT_CSS: &str = r#"
/* =============================================================================
   Orbit Markdown — built-in "default" theme
   All selectors use the `orbit-` prefix. Safe to override in your own CSS.
   Compatible with any page background (light or dark mode via @media).
   ============================================================================= */

/* ── Callouts ─────────────────────────────────────────────────────────────── */

.orbit-callout {
  display: flex;
  gap: 0.75rem;
  padding: 1rem 1.25rem;
  border-radius: 0.5rem;
  border-left: 4px solid currentColor;
  margin: 1.5rem 0;
  font-size: 0.95rem;
  line-height: 1.6;
}

.orbit-callout-icon {
  font-size: 1.1rem;
  flex-shrink: 0;
  line-height: 1.6;
}

.orbit-callout-content { flex: 1; min-width: 0; }

.orbit-callout-title {
  display: block;
  font-weight: 600;
  margin-bottom: 0.25rem;
}

/* Callout colour variants */
.orbit-callout--note    { background: #f0f4ff; color: #3b5bdb; }
.orbit-callout--info    { background: #e8f4fd; color: #0c63e4; }
.orbit-callout--warning { background: #fff8e1; color: #e67700; }
.orbit-callout--danger  { background: #fff0f0; color: #c92a2a; }
.orbit-callout--success { background: #ebfbee; color: #2f9e44; }
.orbit-callout--tip     { background: #f3f0ff; color: #7048e8; }

/* Inner content resets */
.orbit-callout-content > *:first-child { margin-top: 0; }
.orbit-callout-content > *:last-child  { margin-bottom: 0; }

/* ── Steps ────────────────────────────────────────────────────────────────── */

.orbit-steps {
  counter-reset: orbit-step;
  list-style: none;
  padding: 0;
  margin: 1.5rem 0;
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}

.orbit-step {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  padding: 0.875rem 1.25rem;
  background: #f8f9fa;
  border-radius: 0.5rem;
  line-height: 1.6;
  font-size: 0.95rem;
}

.orbit-step::before {
  counter-increment: orbit-step;
  content: counter(orbit-step);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  min-width: 1.75rem;
  border-radius: 50%;
  background: #228be6;
  color: #fff;
  font-size: 0.78rem;
  font-weight: 700;
}

/* ── Cards ────────────────────────────────────────────────────────────────── */

.orbit-card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1.25rem;
  margin: 1.5rem 0;
}

.orbit-card {
  padding: 1.25rem 1.5rem;
  border: 1px solid #dee2e6;
  border-radius: 0.625rem;
  background: #ffffff;
  transition: box-shadow 0.15s ease, border-color 0.15s ease;
}

.orbit-card--link {
  display: block;
  text-decoration: none;
  color: inherit;
  cursor: pointer;
}

.orbit-card--link:hover {
  border-color: #228be6;
  box-shadow: 0 4px 16px rgba(34, 139, 230, 0.12);
}

.orbit-card-title {
  font-size: 1.05rem;
  font-weight: 600;
  margin: 0 0 0.5rem;
  color: #1a1a2e;
}

.orbit-card-body {
  font-size: 0.9rem;
  color: #495057;
  line-height: 1.6;
}

.orbit-card-body > *:first-child { margin-top: 0; }
.orbit-card-body > *:last-child  { margin-bottom: 0; }

/* ── Features ─────────────────────────────────────────────────────────────── */

.orbit-features {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 1.5rem;
  margin: 1.5rem 0;
}

.orbit-feature {
  padding: 1rem;
  border-left: 3px solid #228be6;
  background: #f8f9fa;
  border-radius: 0 0.5rem 0.5rem 0;
}

.orbit-feature-title {
  display: block;
  font-weight: 600;
  margin-bottom: 0.375rem;
  color: #1a1a2e;
}

.orbit-feature-body {
  font-size: 0.9rem;
  color: #495057;
  margin: 0;
  line-height: 1.6;
}

/* ── Buttons ──────────────────────────────────────────────────────────────── */

.orbit-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  margin: 1.5rem 0;
}

.orbit-btn {
  display: inline-flex;
  align-items: center;
  padding: 0.6rem 1.25rem;
  border-radius: 0.5rem;
  font-size: 0.925rem;
  font-weight: 600;
  text-decoration: none;
  border: 2px solid transparent;
  transition: opacity 0.15s ease, box-shadow 0.15s ease, transform 0.1s ease;
  cursor: pointer;
  white-space: nowrap;
}

.orbit-btn:hover  { opacity: 0.88; transform: translateY(-1px); }
.orbit-btn:active { transform: translateY(0); }

.orbit-btn--primary   { background: #228be6; color: #fff; border-color: #228be6; }
.orbit-btn--secondary { background: transparent; color: #228be6; border-color: #228be6; }
.orbit-btn--ghost     { background: transparent; color: #495057; border-color: transparent; }
.orbit-btn--ghost:hover { background: #f1f3f5; opacity: 1; }
.orbit-btn--danger    { background: #fa5252; color: #fff; border-color: #fa5252; }

.orbit-hero {
  display: grid;
  grid-template-columns: 1fr;
  gap: 2rem;
  align-items: center;
  padding: 3rem 2.5rem;
  margin: 0 0 2rem;
  border-radius: 1rem;
  background: linear-gradient(135deg, #f3f0ff 0%, #f8f0fc 100%);
  border: 1px solid #e8dbfc;
  text-align: left;
}

@media (min-width: 768px) {
  .orbit-hero {
    grid-template-columns: 1.15fr 0.85fr;
  }
}

.orbit-hero-content {
  display: flex;
  flex-direction: column;
}

.orbit-hero-title {
  font-size: clamp(1.8rem, 4vw, 2.5rem);
  font-weight: 800;
  line-height: 1.2;
  margin: 0 0 0.75rem;
  color: #7048e8;
  letter-spacing: -0.02em;
}

.orbit-hero-subtitle {
  font-size: clamp(0.95rem, 2vw, 1.15rem);
  color: #495057;
  margin: 0 0 1.5rem;
  font-weight: 400;
  line-height: 1.5;
}

.orbit-hero-body {
  margin: 0 0 1.5rem;
  color: #495057;
  font-size: 0.95rem;
  line-height: 1.6;
}

.orbit-hero-body > *:first-child { margin-top: 0; }
.orbit-hero-body > *:last-child  { margin-bottom: 0; }

.orbit-hero-actions .orbit-buttons { justify-content: flex-start; }

.orbit-hero-graphic {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
}

.orbit-hero-graphic .orbit-figure {
  margin: 0;
  display: flex;
  justify-content: center;
  align-items: center;
}

.orbit-hero-graphic img {
  max-width: 100%;
  height: auto;
  border-radius: 0.5rem;
  box-shadow: none !important;
}

/* ── NavGroup ─────────────────────────────────────────────────────────────── */

.orbit-nav-group { margin: 0.5rem 0 1rem; }

.orbit-nav-group-title {
  display: block;
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: #868e96;
  padding: 0.75rem 0.5rem 0.375rem;
}

.orbit-nav-group-links {
  list-style: none;
  padding: 0;
  margin: 0;
}

.orbit-nav-group-links li { margin: 0; }

.orbit-nav-group-links a {
  display: block;
  padding: 0.35rem 0.625rem;
  border-radius: 0.375rem;
  text-decoration: none;
  font-size: 0.9rem;
  color: #343a40;
  transition: background 0.1s ease, color 0.1s ease;
}

.orbit-nav-group-links a:hover {
  background: #e9ecef;
  color: #228be6;
}

.orbit-nav-group-links a[aria-current="page"] {
  background: #dbe4ff;
  color: #3b5bdb;
  font-weight: 600;
}

/* ── Figure ───────────────────────────────────────────────────────────────── */

.orbit-figure {
  margin: 2rem auto;
  text-align: center;
}

.orbit-figure img {
  max-width: 100%;
  height: auto;
  border-radius: 0.5rem;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  display: block;
  margin: 0 auto;
}

.orbit-figure-caption {
  display: block;
  margin-top: 0.75rem;
  font-size: 0.875rem;
  color: #868e96;
  font-style: italic;
  line-height: 1.5;
}

/* ── Dark mode ────────────────────────────────────────────────────────────── */

@media (prefers-color-scheme: dark) {
  .orbit-callout--note    { background: #1c2340; color: #748ffc; }
  .orbit-callout--info    { background: #0d2137; color: #4dabf7; }
  .orbit-callout--warning { background: #2a1f00; color: #fcc419; }
  .orbit-callout--danger  { background: #2a0d0d; color: #ff6b6b; }
  .orbit-callout--success { background: #0d2b14; color: #69db7c; }
  .orbit-callout--tip     { background: #1e1340; color: #b197fc; }

  .orbit-card {
    background: #1a1b1e;
    border-color: #2c2e33;
  }
  .orbit-card-title { color: #c1c2c5; }
  .orbit-card-body  { color: #909296; }

  .orbit-step { background: #1a1b1e; }

  .orbit-feature { background: #1a1b1e; }
  .orbit-feature-title { color: #c1c2c5; }
  .orbit-feature-body  { color: #909296; }

  .orbit-hero {
    background: linear-gradient(135deg, #1b172a 0%, #1e132c 100%);
    border-color: #312547;
  }
  .orbit-hero-title    { color: #b197fc; }
  .orbit-hero-subtitle { color: #c1c2c5; }
  .orbit-hero-body     { color: #909296; }

  .orbit-nav-group-links a { color: #c1c2c5; }
  .orbit-nav-group-links a:hover { background: #25262b; color: #4dabf7; }
  .orbit-nav-group-links a[aria-current="page"] { background: #1c2340; color: #748ffc; }
}
"#;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_exists() {
        let css = theme_css("default");
        assert!(css.is_some());
    }

    #[test]
    fn unknown_theme_returns_none() {
        assert!(theme_css("does-not-exist").is_none());
        assert!(theme_css("").is_none());
    }

    #[test]
    fn default_theme_contains_all_orbit_classes() {
        let css = theme_css("default").unwrap();
        let required = [
            "orbit-callout",
            "orbit-callout--note",
            "orbit-callout--info",
            "orbit-callout--warning",
            "orbit-callout--danger",
            "orbit-callout--success",
            "orbit-callout--tip",
            "orbit-steps",
            "orbit-step",
            "orbit-card",
            "orbit-card-grid",
            "orbit-card--link",
            "orbit-features",
            "orbit-feature",
            "orbit-buttons",
            "orbit-btn--primary",
            "orbit-btn--secondary",
            "orbit-btn--ghost",
            "orbit-btn--danger",
            "orbit-hero",
            "orbit-hero-title",
            "orbit-hero-subtitle",
            "orbit-nav-group",
            "orbit-figure",
            "orbit-figure-caption",
        ];
        for selector in required {
            assert!(
                css.contains(selector),
                "default theme missing selector: {selector}"
            );
        }
    }

    #[test]
    fn built_in_themes_list_is_non_empty() {
        assert!(!BUILT_IN_THEMES.is_empty());
        assert!(BUILT_IN_THEMES.contains(&"default"));
    }

    #[test]
    fn every_built_in_theme_name_resolves() {
        for &name in BUILT_IN_THEMES {
            assert!(
                theme_css(name).is_some(),
                "BUILT_IN_THEMES lists '{name}' but theme_css returned None"
            );
        }
    }
}
