# Orbit Markdown Architecture

## Purpose

Orbit Markdown is the next language layer for Orbit.

Today, Orbit users write Markdown and can invoke custom Handlebars components with JSX-style tags:

```md
<Alert type="info">
Hello **world**.
</Alert>
```

That works, but it asks users to understand component files, `.hbs` templates, HTML structure, and CSS. The next version should make Orbit feel like a Markdown-native publishing tool.

The goal is:

```text
Users write only .md files.
Orbit provides layouts, themes, and built-in styled blocks.
```

Orbit Markdown should remain compatible with normal Markdown while adding a small set of Orbit-specific syntax for common documentation and landing-page patterns.

## Product Goal

Orbit should become a Markdown compiler for styled static sites.

The user writes:

```md
---
title: Getting Started
layout: docs
theme: default
---

# Getting Started

:::note
Orbit turns Markdown into a styled static site.
:::

:::steps
1. Install Rust
2. Run `cargo install orbit-md`
3. Start `orbit dev`
:::
```

Orbit compiles that into:

```text
.orbit/docs/getting-started.html
```

The user does not need to create:

```text
components/*.hbs
templates/*.hbs
custom CSS
HTML wrappers
```

Those should become Orbit-provided defaults. Advanced customization can remain possible later, but it should not be required for the common path.

## Language Definition

Orbit Markdown is:

```text
Common Markdown + YAML front matter + Orbit directives
```

Every valid Markdown document should still be valid Orbit Markdown.

Orbit extensions should be opt-in and Markdown-readable.

## Syntax Direction

Orbit should use directive-style blocks with `:::` fences.

Reasons:

- Looks natural in documentation.
- Avoids HTML and JSX.
- Easy to scan in plain text.
- Can support nested Markdown content.
- Can be parsed safely when the parser is Markdown-aware.

### Basic Directive Shape

```md
:::kind
Body content.
:::
```

With attributes:

```md
:::kind title="Hello" tone="info"
Body content.
:::
```

Short form:

```md
:::warning
Be careful.
:::
```

Named type form:

```md
:::callout info title="Heads up"
This is useful.
:::
```

## Front Matter

Front matter remains YAML:

```md
---
title: Getting Started
description: Learn Orbit Markdown
layout: docs
theme: default
date: 2026-07-13
tags:
  - docs
  - rust
draft: false
nav_order: 1
---
```

Initial supported fields:

| Field | Purpose |
|---|---|
| `title` | Page title |
| `description` | Meta description and page summary |
| `date` | Optional display date |
| `tags` | Optional taxonomy |
| `draft` | Skip page when true |
| `layout` | Built-in layout choice |
| `theme` | Built-in theme choice |
| `nav_order` | Optional docs navigation ordering |

## Built-In Directives

Orbit v0.2 should start small. The first language version should support the most useful documentation primitives.

### Callouts

```md
:::note
Useful information.
:::

:::warning title="Careful"
Important warning.
:::

:::success
Everything worked.
:::
```

Canonical internal form:

```rust
OrbitNode::Callout {
    kind: CalloutKind::Warning,
    title: Some("Careful"),
    children,
}
```

Supported callout kinds:

```text
note
info
warning
success
danger
```

### Cards

```md
:::card title="Fast builds"
Orbit compiles Markdown in parallel.
:::
```

Internal form:

```rust
OrbitNode::Card {
    title: Some("Fast builds"),
    children,
}
```

### Steps

```md
:::steps
1. Install Rust
2. Run `cargo install orbit-md`
3. Start `orbit dev`
:::
```

The compiler can render ordered list items as styled steps.

Internal form:

```rust
OrbitNode::Steps {
    items,
}
```

### Feature Lists

For v0.2, use a simple Markdown-list based form:

```md
:::features
- **Markdown-first**: Write plain `.md` files.
- **Rust-powered**: Compile sites quickly.
- **Static output**: Deploy `.orbit/` anywhere.
:::
```

Later, Orbit can support richer structured syntax:

```md
:::features
- title: Markdown-first
  icon: M
  body: Write plain Markdown.
- title: Rust-powered
  icon: R
  body: Fast static builds.
:::
```

### Buttons

Buttons should use normal Markdown links plus attributes.

```md
[Get started](/docs/getting-started){.button}
[Crate](https://crates.io/crates/orbit-md){.button secondary}
```

This can be a later v0.2.x feature if link attributes require more parser work.

## Non-Goals For First Version

Do not build everything in v0.2.

Postpone:

- tabs
- code groups
- custom user components
- JavaScript islands
- MDX-style JSX
- full custom theme authoring
- syntax highlighting themes
- nested arbitrary directives beyond simple nesting

The first version should prove the compiler architecture.

## Compiler Pipeline

Current Orbit pipeline:

```text
Markdown source
  -> raw string component scanner
  -> Handlebars component expansion
  -> pulldown-cmark Markdown parser
  -> Handlebars layout template
  -> HTML output
```

New Orbit Markdown pipeline:

```text
Markdown source
  -> front matter extraction
  -> Markdown-aware event parsing
  -> Orbit directive recognition
  -> Orbit AST construction
  -> AST to HTML rendering
  -> built-in layout/theme rendering
  -> HTML output
```

The key architectural shift:

```text
from string replacement
to Markdown-aware compilation
```

## Parser Requirements

The parser must understand Markdown context.

Most important rule:

```text
Never parse Orbit directives inside fenced code blocks or inline code.
```

This should render as code:

````md
```md
:::note
Do not parse me.
:::
```
````

This should render as a callout:

```md
:::note
Parse me.
:::
```

## Internal AST

Introduce an Orbit-level AST.

Initial sketch:

```rust
pub enum OrbitNode {
    Markdown(String),
    Html(String),
    Callout {
        kind: CalloutKind,
        title: Option<String>,
        children: Vec<OrbitNode>,
    },
    Card {
        title: Option<String>,
        children: Vec<OrbitNode>,
    },
    Steps {
        items: Vec<StepItem>,
    },
    Features {
        items: Vec<FeatureItem>,
    },
}

pub enum CalloutKind {
    Note,
    Info,
    Warning,
    Success,
    Danger,
}

pub struct StepItem {
    pub body: Vec<OrbitNode>,
}

pub struct FeatureItem {
    pub title: String,
    pub body: Vec<OrbitNode>,
    pub icon: Option<String>,
}
```

The exact representation can change after implementation starts. The important part is that Orbit should have a semantic representation before HTML rendering.

## Rendering Model

Orbit should render built-in directives to semantic HTML with stable CSS classes.

Example:

```md
:::warning title="Careful"
Check your config.
:::
```

Target HTML:

```html
<aside class="orbit-callout orbit-callout-warning">
  <strong class="orbit-callout-title">Careful</strong>
  <div class="orbit-callout-body">
    <p>Check your config.</p>
  </div>
</aside>
```

The built-in theme owns these classes:

```css
.orbit-callout { ... }
.orbit-callout-warning { ... }
```

## Built-In Layouts

Orbit should ship with built-in layouts so users do not need `templates/base.hbs`.

Initial layouts:

| Layout | Purpose |
|---|---|
| `docs` | Documentation pages with readable width and nav |
| `page` | General content pages |
| `landing` | Product-style landing page |

Front matter:

```yaml
layout: docs
```

If missing, use:

```yaml
layout: docs
```

or derive from file path:

```text
content/index.md -> landing
content/docs/*   -> docs
```

## Built-In Themes

Orbit should ship at least one built-in theme.

Initial theme:

```yaml
theme: default
```

Potential future themes:

```text
default
minimal
terminal
editorial
```

Themes should be internal CSS strings or embedded assets at first.

## Relationship To Handlebars

Handlebars should move from default user-facing workflow to advanced customization.

Current:

```text
User creates components/*.hbs and templates/*.hbs
```

Future:

```text
User writes .md only
Orbit provides built-in renderers
Optional advanced template override later
```

Compatibility plan:

- Keep existing `.hbs` component support for now.
- Mark it as advanced or legacy in docs later.
- Prefer Orbit directives in scaffolded projects.
- Do not remove old support until a major version.

## Error Reporting

Orbit Markdown should produce clear errors.

Examples:

```text
content/docs/start.md:12: unclosed directive `note`
content/docs/start.md:18: unknown directive `warnnig`; did you mean `warning`?
content/docs/start.md:24: invalid attribute syntax in directive `card`
```

Error type should include:

```rust
path
line
column
message
```

Current `PageError` only has path and message. We may extend it later.

## File Extensions

Keep `.md`.

Do not introduce `.omd` yet.

Reason:

- Users already understand Markdown files.
- Editor support works immediately.
- Orbit Markdown should be Markdown-compatible.

Potential future:

```text
.orbit.md
```

Only if we need editor/tooling distinction.

## Implementation Plan For v0.2

### Phase 1: Add Compiler Module

Add:

```text
src/orbit_markdown/
  mod.rs
  ast.rs
  parser.rs
  directives.rs
  render.rs
  theme.rs
  layout.rs
```

Responsibilities:

- parse Orbit directives
- construct AST or transformed Markdown
- render built-in directive HTML
- provide built-in theme CSS
- provide built-in page layouts

Keep this as a dedicated compiler boundary. Do not scatter Orbit Markdown parsing through the legacy `components/` module or the base Markdown `parser/` module.

### Phase 2: Directive Parser

Implement:

```text
:::note
...
:::
```

and:

```text
:::warning title="Careful"
...
:::
```

Must skip fenced code blocks.

### Phase 3: Built-In HTML Renderers

Render:

- callouts
- cards
- steps
- features

Use stable CSS class names prefixed with `orbit-`.

### Phase 4: Built-In Layout

Add internal layout renderer so a site can build without `templates/base.hbs`.

Short-term option:

- keep existing template path if present
- otherwise use built-in layout

### Phase 5: Update Scaffold

Generated sites should contain only:

```text
orbit.yaml
content/index.md
content/docs/getting-started.md
.gitignore
```

No required:

```text
components/
templates/
```

### Phase 6: Compatibility

Keep old component expansion behind the existing path for one release:

```text
Orbit Markdown directives first
legacy JSX-style components second
Markdown compile third
```

Or choose the simpler v0.2 route:

```text
legacy components stay as-is
new directives are implemented as a preprocessor before legacy components
```

The final design can evolve after the first implementation spike.

## Example End State

Input:

```md
---
title: Orbit
layout: landing
theme: default
---

# Orbit

Fast static sites from Markdown.

[Get started](/docs/getting-started){.button}

:::features
- **Markdown-only**: No templates needed for common sites.
- **Rust-powered**: Fast builds with a small CLI.
- **Static output**: Deploy `.orbit/` anywhere.
:::
```

Output:

```text
.orbit/index.html
```

The HTML includes:

- page layout
- theme CSS
- rendered Markdown
- rendered feature grid
- static links

## Open Questions

1. Should directive attributes use only `key="value"` or also shorthand tokens?
2. Should callouts use `:::note` or `:::callout note` as canonical syntax?
3. Should built-in layouts fully replace `templates/` in v0.2 or coexist?
4. Should the compiler build a full AST immediately or begin with event-level transforms?
5. Should link attributes be part of v0.2 or delayed?
6. Should HTML be allowed in user Markdown by default?

## Recommended First Decision

Start with:

```md
:::note
...
:::
```

instead of:

```md
:::callout note
...
:::
```

Reason:

- More readable.
- Less typing.
- Easier for beginners.
- Maps directly to common docs language.

Support aliases later if needed.

## Summary

Orbit Markdown is a Markdown-compatible source language for building styled static sites.

The compiler should move Orbit away from user-authored HTML and `.hbs` files for the default experience. Users should write Markdown only, while Orbit provides built-in directives, themes, and layouts.

This turns Orbit into:

```text
Markdown authoring experience
Compiler architecture
Static HTML output
```

That is the right foundation for the next version.
