# Orbit v1.0.0 Directive Reference

> **Status: Locked for v1.0.0 implementation.**
>
> This document is the source of truth for every `:::directive` Orbit supports
> in v1.0.0. The parser, renderer, and tests are all built from this spec.
> Do not add or change directives without updating this file first.

---

## Full Directive List

| Directive     | Category      | Required attrs     | Optional attrs     |
|---------------|---------------|--------------------|--------------------|
| `note`        | Callout       | —                  | `title`            |
| `info`        | Callout       | —                  | `title`            |
| `warning`     | Callout       | —                  | `title`            |
| `danger`      | Callout       | —                  | `title`            |
| `success`     | Callout       | —                  | `title`            |
| `tip`         | Callout       | —                  | `title`            |
| `steps`       | Structure     | —                  | —                  |
| `card`        | Structure     | `title`            | `href`             |
| `features`    | Structure     | —                  | —                  |
| `buttons`     | Navigation    | —                  | —                  |
| `hero`        | Navigation    | `title`            | `subtitle`         |
| `nav-group`   | Navigation    | `title`            | —                  |
| `figure`      | Media         | `src`, `alt`       | `caption`          |

---

## Syntax Rules

### Opening line

```
:::directive-name
:::directive-name key="value"
:::directive-name key="value" key2="value2"
```

- Name is lowercase, hyphens allowed (`nav-group`).
- Attributes use `key="value"` syntax. Single quotes also accepted.
- Attribute order does not matter.
- Unknown attributes are ignored (not an error) in v1.0.0.

### Closing line

```
:::
```

A line that is exactly `:::` (after trimming whitespace) closes the innermost
open directive. Nothing else closes a directive.

### Nesting

Directives can contain Markdown. Directives cannot contain other directives
in v1.0.0, with one exception: `hero` may contain a `:::buttons` block.

```md
:::hero title="Orbit"
Write Markdown. Get a styled site.

:::buttons
[Get started](/docs/getting-started) primary
[GitHub](https://github.com/orbit-md) secondary
:::
:::
```

All other nesting is an error (D006).

### Inside fenced code — never parsed

````md
```md
:::note
This is NOT parsed as a directive. It is rendered as code.
:::
```
````

The parser tracks fenced code state. Directives are never recognized inside
a code fence.

### Inside inline code — never parsed

```md
Use `:::note` to create a callout.
```

The text `:::note` above is plain text, not a directive.

---

## Error Codes

| Code | Condition | Example message |
|------|-----------|-----------------|
| D001 | Unknown directive name | `unknown directive 'warnnig'; did you mean 'warning'?` |
| D002 | Missing required attribute | `directive 'card' requires attribute 'title'` |
| D003 | Unclosed directive | `directive 'note' opened on line 14 was never closed` |
| D004 | Invalid attribute syntax | `expected '="value"' after attribute name 'title'` |
| D006 | Illegal nesting | `directive 'card' cannot be nested inside 'note'` |

All errors include `path`, `line`, `column`.

---

## 1. Callouts — `note`, `info`, `warning`, `danger`, `success`, `tip`

Six variants, all same shape.

### Syntax

```md
:::note
Something the reader should know.
:::

:::warning title="Heads up"
This will affect your existing configuration.
:::
```

### Attributes

| Attribute | Type   | Required | Description                    |
|-----------|--------|----------|--------------------------------|
| `title`   | string | no       | Heading shown above the body   |

### Rendered HTML

```html
<aside class="orbit-callout orbit-callout-warning">
  <span class="orbit-callout-icon" aria-hidden="true">⚠️</span>
  <strong class="orbit-callout-title">Heads up</strong>
  <div class="orbit-callout-body">
    <p>This will affect your existing configuration.</p>
  </div>
</aside>
```

When no `title` is given, the `<strong>` element is omitted entirely.

### Icon mapping

| Variant   | Icon | Color token               |
|-----------|------|---------------------------|
| `note`    | 📝   | `--orbit-color-note`      |
| `info`    | ℹ️   | `--orbit-color-info`      |
| `warning` | ⚠️   | `--orbit-color-warning`   |
| `danger`  | 🚨   | `--orbit-color-danger`    |
| `success` | ✅   | `--orbit-color-success`   |
| `tip`     | 💡   | `--orbit-color-tip`       |

### CSS classes used

```
.orbit-callout
.orbit-callout-{note|info|warning|danger|success|tip}
.orbit-callout-icon
.orbit-callout-title
.orbit-callout-body
```

---

## 2. Steps — `steps`

A numbered sequence of how-to instructions.

### Syntax

```md
:::steps
- Install Rust from [rustup.rs](https://rustup.rs)
- Run `cargo install orbit-md`
- Create a site: `orbit init my-site`
- Start the server: `orbit dev`
:::
```

Body must be a Markdown unordered list (`-` bullets). Each list item becomes
one step. Orbit ignores the bullet markers and assigns sequential numbers.

### Attributes

None.

### Rendered HTML

```html
<ol class="orbit-steps">
  <li class="orbit-step">
    <span class="orbit-step-number" aria-hidden="true">1</span>
    <div class="orbit-step-body">
      <p>Install Rust from <a href="https://rustup.rs">rustup.rs</a></p>
    </div>
  </li>
  <li class="orbit-step">
    <span class="orbit-step-number" aria-hidden="true">2</span>
    <div class="orbit-step-body">
      <p>Run <code>cargo install orbit-md</code></p>
    </div>
  </li>
  <!-- ... -->
</ol>
```

### CSS classes used

```
.orbit-steps
.orbit-step
.orbit-step-number
.orbit-step-body
```

---

## 3. Card — `card`

A content card with a title and optional link. Adjacent cards are
automatically grouped into a responsive grid.

### Syntax

```md
:::card title="Parallel Builds"
Orbit compiles every page using Rayon. Large sites build in milliseconds.
:::

:::card title="Zero JavaScript" href="/docs/output"
The default output is static HTML and CSS. No JavaScript unless you add it.
:::

:::card title="Single Binary" href="/docs/install"
`cargo install orbit-md`. No Node, no npm, no runtime.
:::
```

### Attributes

| Attribute | Type   | Required | Description                          |
|-----------|--------|----------|--------------------------------------|
| `title`   | string | **yes**  | Card heading                         |
| `href`    | url    | no       | Makes the entire card a clickable link |

### Card grid rule

When two or more `:::card` blocks appear consecutively (no other directive or
blank lines between them), the renderer wraps them in a `<div class="orbit-card-grid">`.

A single isolated `:::card` renders without the grid wrapper.

### Rendered HTML (single card, no href)

```html
<div class="orbit-card">
  <h3 class="orbit-card-title">Parallel Builds</h3>
  <div class="orbit-card-body">
    <p>Orbit compiles every page using Rayon.</p>
  </div>
</div>
```

### Rendered HTML (card with href)

```html
<a class="orbit-card orbit-card-link" href="/docs/output">
  <h3 class="orbit-card-title">Zero JavaScript</h3>
  <div class="orbit-card-body">
    <p>The default output is static HTML and CSS.</p>
  </div>
</a>
```

### Rendered HTML (grid of three cards)

```html
<div class="orbit-card-grid">
  <div class="orbit-card">...</div>
  <a class="orbit-card orbit-card-link" href="...">...</a>
  <div class="orbit-card">...</div>
</div>
```

### CSS classes used

```
.orbit-card
.orbit-card-link       (added when href is present)
.orbit-card-grid       (wrapper for 2+ consecutive cards)
.orbit-card-title
.orbit-card-body
```

---

## 4. Features — `features`

A feature grid for landing pages. Each list item is a feature entry.
Bold text at the start of an item becomes the feature title.

### Syntax

```md
:::features
- **Markdown-native**: Write plain `.md` files. No templates needed.
- **Rust-powered**: Parallel builds. Sites compile in milliseconds.
- **Static output**: Deploy `.orbit/` to any CDN or static host.
- **Zero JS default**: Clean, fast HTML pages out of the box.
- **Single binary**: `cargo install orbit-md`. Done.
- **Open source**: MIT licensed. Own your content forever.
:::
```

### Parsing rule for items

Each `-` list item is split at the first `:` character (after optional bold
markers). Everything before `:` is the title, everything after is the body.

```
- **Markdown-native**: Write plain `.md` files.
  ↓
title = "Markdown-native"
body  = "Write plain `.md` files."
```

### Attributes

None.

### Rendered HTML

```html
<ul class="orbit-features">
  <li class="orbit-feature">
    <strong class="orbit-feature-title">Markdown-native</strong>
    <p class="orbit-feature-body">Write plain <code>.md</code> files. No templates needed.</p>
  </li>
  <li class="orbit-feature">
    <strong class="orbit-feature-title">Rust-powered</strong>
    <p class="orbit-feature-body">Parallel builds. Sites compile in milliseconds.</p>
  </li>
  <!-- ... -->
</ul>
```

### CSS classes used

```
.orbit-features
.orbit-feature
.orbit-feature-title
.orbit-feature-body
```

---

## 5. Buttons — `buttons`

A group of CTA (call-to-action) link buttons. Used on landing pages and
at the end of guide pages.

### Syntax

```md
:::buttons
[Get started](/docs/getting-started) primary
[View on GitHub](https://github.com/orbit-md) secondary
[Read the docs](/docs) ghost
:::
```

Each line inside `:::buttons` is a single button defined as:
```
[Label](url) style
```

Where `style` is one of: `primary`, `secondary`, `ghost`, `danger`.
If style is omitted, `primary` is used.

### Attributes

None on the opening `:::buttons` line. Style is per-button on each body line.

### Button styles

| Style       | Appearance                            |
|-------------|---------------------------------------|
| `primary`   | Filled, brand color background        |
| `secondary` | Outlined, brand color border + text   |
| `ghost`     | No border, muted text                 |
| `danger`    | Filled, danger color background       |

### Rendered HTML

```html
<div class="orbit-buttons">
  <a href="/docs/getting-started" class="orbit-btn orbit-btn-primary">Get started</a>
  <a href="https://github.com/orbit-md" class="orbit-btn orbit-btn-secondary">View on GitHub</a>
  <a href="/docs" class="orbit-btn orbit-btn-ghost">Read the docs</a>
</div>
```

### CSS classes used

```
.orbit-buttons
.orbit-btn
.orbit-btn-primary
.orbit-btn-secondary
.orbit-btn-ghost
.orbit-btn-danger
```

---

## 6. Hero — `hero`

A full-width hero section for landing pages. Typically the first block in
`content/index.md`. Can contain a `:::buttons` block for CTAs.

### Syntax

```md
:::hero title="Orbit" subtitle="Fast static sites from Markdown"
Write `.md` files. Orbit turns them into a styled, fast static site.
No templates. No configuration. No Node.

:::buttons
[Get started](/docs/getting-started) primary
[GitHub](https://github.com/orbit-md) secondary
:::
:::
```

### Attributes

| Attribute  | Type   | Required | Description                         |
|------------|--------|----------|-------------------------------------|
| `title`    | string | **yes**  | Main headline of the hero           |
| `subtitle` | string | no       | Supporting line below the headline  |

### Body

The body is Markdown text rendered as the hero description. A nested
`:::buttons` block is the only nesting allowed inside `:::hero`.

### Rendered HTML

```html
<section class="orbit-hero">
  <div class="orbit-hero-inner">
    <h1 class="orbit-hero-title">Orbit</h1>
    <p class="orbit-hero-subtitle">Fast static sites from Markdown</p>
    <div class="orbit-hero-body">
      <p>Write <code>.md</code> files. Orbit turns them into a styled, fast static site.</p>
    </div>
    <div class="orbit-hero-actions">
      <!-- rendered :::buttons content -->
      <a href="/docs/getting-started" class="orbit-btn orbit-btn-primary">Get started</a>
      <a href="https://github.com/orbit-md" class="orbit-btn orbit-btn-secondary">GitHub</a>
    </div>
  </div>
</section>
```

### CSS classes used

```
.orbit-hero
.orbit-hero-inner
.orbit-hero-title
.orbit-hero-subtitle
.orbit-hero-body
.orbit-hero-actions
```

---

## 7. Nav Group — `nav-group`

Labels a section of the auto-generated sidebar navigation. Applies only to
the `docs` layout. Has no rendered HTML output on the page itself — it
only affects the sidebar.

### Syntax

In `content/docs/getting-started.md`:

```md
---
title: Getting Started
layout: docs
nav_order: 1
nav_group: "Getting Started"
---
```

Or as a standalone directive on any docs page:

```md
:::nav-group title="Reference"
- [Config options](/docs/config)
- [CLI commands](/docs/cli)
- [Front matter](/docs/front-matter)
:::
```

### Two forms

**Form A — front matter field (recommended):**

Add `nav_group: "Section Name"` to the page's front matter. The page appears
under that group in the sidebar. Pages without `nav_group` appear in an
ungrouped section at the top.

**Form B — directive:**

A `:::nav-group` block explicitly lists links that appear under a labeled
section. The links in the body do not need to be pages in the current site
— they can be external links too.

### Attributes

| Attribute | Type   | Required | Description           |
|-----------|--------|----------|-----------------------|
| `title`   | string | **yes**  | Section heading in the sidebar |

### Body (Form B only)

A Markdown unordered list of links:

```md
:::nav-group title="Reference"
- [Config options](/docs/config)
- [CLI commands](/docs/cli)
:::
```

### Rendered HTML (sidebar, generated by the docs layout)

```html
<nav class="orbit-sidebar">
  <div class="orbit-nav-group">
    <span class="orbit-nav-group-title">Getting Started</span>
    <ul class="orbit-nav-group-links">
      <li><a class="orbit-nav-link" href="/docs/getting-started.html">Getting Started</a></li>
      <li><a class="orbit-nav-link" href="/docs/quickstart.html">Quickstart</a></li>
    </ul>
  </div>
  <div class="orbit-nav-group">
    <span class="orbit-nav-group-title">Reference</span>
    <ul class="orbit-nav-group-links">
      <li><a class="orbit-nav-link" href="/docs/config.html">Config options</a></li>
      <li><a class="orbit-nav-link" href="/docs/cli.html">CLI commands</a></li>
    </ul>
  </div>
</nav>
```

### CSS classes used

```
.orbit-sidebar
.orbit-nav-group
.orbit-nav-group-title
.orbit-nav-group-links
.orbit-nav-link
.orbit-nav-link--active   (added to the current page's link)
```

---

## 8. Figure — `figure`

An image with an optional caption. A semantic upgrade over raw Markdown
image syntax for important diagrams and screenshots.

### Syntax

```md
:::figure src="/images/pipeline.png" alt="The Orbit build pipeline" caption="Figure 1: Orbit compiles Markdown in four stages."
:::
```

Self-closing — no body content needed. The `:::` closing line is still required.

### Attributes

| Attribute | Type   | Required | Description                                  |
|-----------|--------|----------|----------------------------------------------|
| `src`     | url    | **yes**  | Image source path or URL                     |
| `alt`     | string | **yes**  | Alt text for accessibility                   |
| `caption` | string | no       | Caption text shown below the image           |

### Rendered HTML (with caption)

```html
<figure class="orbit-figure">
  <img
    class="orbit-figure-img"
    src="/images/pipeline.png"
    alt="The Orbit build pipeline"
  >
  <figcaption class="orbit-figure-caption">
    Figure 1: Orbit compiles Markdown in four stages.
  </figcaption>
</figure>
```

### Rendered HTML (no caption)

```html
<figure class="orbit-figure">
  <img
    class="orbit-figure-img"
    src="/images/pipeline.png"
    alt="The Orbit build pipeline"
  >
</figure>
```

### CSS classes used

```
.orbit-figure
.orbit-figure-img
.orbit-figure-caption
```

---

## Deferred to v2.0.0

These are explicitly not in v1.0.0. Do not implement them now.

| Directive      | Reason deferred                               |
|----------------|-----------------------------------------------|
| `tabs`         | Requires JavaScript for tab switching         |
| `code-group`   | Requires JavaScript for language switching    |
| `section`      | Landing layout can be designed without it     |
| `breadcrumb`   | Auto-generated from file path in the layout   |
| `badge`        | Low priority, can be done with plain HTML     |
| `video`        | Low priority for v1.0.0                       |

---

## Complete CSS Class Reference

All classes produced by the v1.0.0 directive renderer.

```
orbit-callout
orbit-callout-note
orbit-callout-info
orbit-callout-warning
orbit-callout-danger
orbit-callout-success
orbit-callout-tip
orbit-callout-icon
orbit-callout-title
orbit-callout-body

orbit-steps
orbit-step
orbit-step-number
orbit-step-body

orbit-card
orbit-card-link
orbit-card-grid
orbit-card-title
orbit-card-body

orbit-features
orbit-feature
orbit-feature-title
orbit-feature-body

orbit-buttons
orbit-btn
orbit-btn-primary
orbit-btn-secondary
orbit-btn-ghost
orbit-btn-danger

orbit-hero
orbit-hero-inner
orbit-hero-title
orbit-hero-subtitle
orbit-hero-body
orbit-hero-actions

orbit-sidebar
orbit-nav-group
orbit-nav-group-title
orbit-nav-group-links
orbit-nav-link
orbit-nav-link--active

orbit-figure
orbit-figure-img
orbit-figure-caption
```

---

*Last updated: 2026-07-13*
*Status: Locked — v1.0.0 implementation spec*
