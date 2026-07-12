# Orbit Markdown Language Specification — `.omd`

> **Status: Future Plan — Not Yet Implemented**
>
> This document describes the intended design of `.omd`, Orbit's own authoring
> language. It exists to capture the thinking before implementation begins.
> Nothing described here is stable or final. Decisions marked **OPEN** require
> a concrete choice before the relevant phase starts.

---

## 1. Why `.omd`?

The current Orbit pipeline works well for developers comfortable with Handlebars
templates and JSX-style component syntax. But the target user for the next
version is someone who primarily writes, not someone who configures build tools.

The core problem with extending `.md`:

- Every syntax addition has to be compatible with Markdown parsers that know
  nothing about Orbit. This creates permanent compromise.
- Editors render `.md` files as Markdown previews, which makes Orbit extensions
  look like broken Markdown or raw text.
- The styling system can never be first-class because Markdown has no concept
  of design tokens, themes, or layout.

`.omd` solves this by making Orbit the primary authoring tool rather than
a Markdown extension. The language is designed from the ground up for the
use case: writing styled, structured content that compiles to beautiful static
HTML with zero configuration.

The one-sentence pitch:

> `.omd` is what you write. `orbit` turns it into a styled website.

---

## 2. Design Principles

These principles must survive every syntax decision.

1. **Readable as plain text.** An `.omd` file should be understandable when
   opened in Notepad. It should not look like code.

2. **Familiar to Markdown users.** Headings, bold, italic, links, code blocks,
   and lists work exactly as in Markdown. Users migrate by renaming files.

3. **Opt-in complexity.** A plain Markdown file is a valid `.omd` file. Orbit
   features are additive. You never have to use them.

4. **One way to do common things.** Every common documentation pattern (notes,
   warnings, steps, code examples, feature grids) has exactly one canonical
   Orbit syntax. No choices to paralyze the user.

5. **Errors help, not just fail.** Every invalid `.omd` construct should
   produce a message with a file path, line number, and a suggestion.

6. **The style is part of the language.** Themes and layout choices are made
   in the file's front matter. You do not write CSS or HTML to style a page.

---

## 3. File Format

### Extension

```
.omd
```

Files are UTF-8 encoded plain text.

### Compatibility

Every valid Markdown document is a valid `.omd` document. The Orbit compiler
must parse and render standard Markdown correctly before any Orbit extensions
are involved.

### Migration path

Existing `.md` sites can migrate by:

1. Renaming files from `.md` to `.omd`.
2. Adding `layout` and `theme` to front matter.
3. Replacing JSX-style component tags with Orbit block syntax over time.

A future `orbit migrate` CLI command should automate step 1.

---

## 4. Front Matter

Front matter is YAML between `---` delimiters, identical to the current format.

```omd
---
title: Getting Started
description: Learn how to use Orbit in five minutes.
date: 2026-07-13
tags:
  - docs
  - orbit
draft: false
layout: docs
theme: default
nav_order: 1
---
```

### Supported fields

| Field         | Type          | Purpose                                                  |
|---------------|---------------|----------------------------------------------------------|
| `title`       | string        | Page title shown in the browser tab and layout header    |
| `description` | string        | Meta description for SEO and page summary cards          |
| `date`        | string        | Optional publication or update date                      |
| `tags`        | list          | Optional taxonomy tags                                   |
| `draft`       | bool          | When `true`, page is excluded from the build             |
| `layout`      | string        | Built-in layout name or path to custom template          |
| `theme`       | string        | Built-in theme name                                      |
| `nav_order`   | integer       | Position in generated navigation (lower = earlier)       |

### Layout selection rules

When `layout` is absent, Orbit infers the layout from the file path:

```text
content/index.omd        → landing
content/docs/**/*.omd    → docs
everything else          → page
```

### Theme selection rules

When `theme` is absent, Orbit uses `default`. The theme can be set globally in
`orbit.yaml` and overridden per page in front matter.

---

## 5. Standard Markdown Elements

The following Markdown features work identically in `.omd`:

- Headings `#` through `######`
- Bold `**text**`, italic `*text*`, strikethrough `~~text~~`
- Inline code `` `code` ``
- Fenced code blocks with language identifiers
- Links `[text](url)`, images `![alt](url)`
- Ordered and unordered lists
- Blockquotes `>`
- Horizontal rules `---`
- Tables (GFM style)
- Footnotes
- Task lists `- [ ]` / `- [x]`

**Orbit block syntax is never parsed inside fenced code blocks or inline code.**
This is an absolute rule.

---

## 6. Orbit Block Syntax

Orbit introduces a **block** syntax for semantic, styled content that cannot
be expressed cleanly in standard Markdown.

### 6.1 Basic shape

```omd
[block-type]
Content goes here.
[/block-type]
```

Or with attributes:

```omd
[block-type key="value" key2="value2"]
Content goes here.
[/block-type]
```

Self-closing (no body):

```omd
[block-type key="value" /]
```

### 6.2 Why `[block]` instead of `:::block`

`[block]` is chosen over `:::block` for these reasons:

- Square brackets are rarely used at the start of a line in Markdown bodies.
- The open/close tags are visually symmetric and easy to scan.
- The self-closing form `[foo /]` is immediately recognizable from HTML.
- Nesting is unambiguous: open and close tags are always explicit.
- It reads naturally in plain text: `[note]`, `[warning]`, `[steps]`.

**OPEN:** Final syntax shape needs a user-testing decision before the parser
is written. The above is the current leading candidate.

---

## 7. Built-In Block Types

### 7.1 Callouts

Used for notes, warnings, tips, and alerts.

```omd
[note]
Orbit supports Markdown natively. No conversion step needed.
[/note]

[warning title="Check your config"]
The `output_dir` setting defaults to `.orbit/`. Make sure this is not tracked
by your deployment tool as a source directory.
[/warning]

[success]
Your site built in 12 ms.
[/success]

[danger title="Breaking change"]
This behavior changed in v0.3. Update your `orbit.yaml` before upgrading.
[/danger]

[info]
You can use any theme with any layout.
[/info]
```

#### Callout kinds

| Kind      | Default icon | Intended use                        |
|-----------|--------------|-------------------------------------|
| `note`    | 📝            | General information                 |
| `info`    | ℹ️            | Contextual background               |
| `warning` | ⚠️            | Potential problems                  |
| `danger`  | 🚨            | Breaking changes, data loss risk    |
| `success` | ✅            | Confirmation, completion            |
| `tip`     | 💡            | Shortcuts and best practices        |

#### Callout attributes

| Attribute | Type   | Purpose                                 |
|-----------|--------|-----------------------------------------|
| `title`   | string | Optional heading above the body         |

#### Rendered HTML

```html
<aside class="orbit-callout orbit-callout-warning">
  <span class="orbit-callout-icon" aria-hidden="true">⚠️</span>
  <strong class="orbit-callout-title">Check your config</strong>
  <div class="orbit-callout-body">
    <p>The <code>output_dir</code> setting defaults to <code>.orbit/</code>.</p>
  </div>
</aside>
```

---

### 7.2 Steps

Used for numbered how-to sequences.

```omd
[steps]
- Install Rust via `rustup.rs`
- Run `cargo install orbit-md`
- Create a site: `orbit init my-site`
- Start the dev server: `orbit dev`
[/steps]
```

Steps are parsed as an ordered list. Each item is compiled to an HTML fragment.

#### Rendered HTML

```html
<ol class="orbit-steps">
  <li class="orbit-step">
    <span class="orbit-step-number">1</span>
    <div class="orbit-step-body">Install Rust via <a href="...">rustup.rs</a></div>
  </li>
  ...
</ol>
```

---

### 7.3 Cards

Used for feature highlights and grid layouts.

```omd
[card title="Parallel builds"]
Orbit compiles every page in parallel using Rayon. Large sites build in
under a second.
[/card]

[card title="Zero JavaScript"]
The default output is static HTML and CSS. No JavaScript ships unless you
explicitly add it.
[/card]

[card title="Single binary"]
Install with `cargo install orbit-md`. No runtime, no Node, no npm.
[/card]
```

When multiple `[card]` blocks appear in sequence, Orbit wraps them in a grid
container automatically.

#### Card attributes

| Attribute | Type   | Purpose                                        |
|-----------|--------|------------------------------------------------|
| `title`   | string | Card heading                                   |
| `href`    | url    | Makes the card a clickable link                |
| `accent`  | string | Theme accent color name for the card border    |

#### Rendered HTML (single card)

```html
<div class="orbit-card">
  <h3 class="orbit-card-title">Parallel builds</h3>
  <div class="orbit-card-body">
    <p>Orbit compiles every page in parallel using Rayon.</p>
  </div>
</div>
```

#### Rendered HTML (card group, auto-detected)

```html
<div class="orbit-card-grid">
  <div class="orbit-card">...</div>
  <div class="orbit-card">...</div>
  <div class="orbit-card">...</div>
</div>
```

---

### 7.4 Feature Grid

Used for product landing pages.

```omd
[features]
- **Markdown-first**: Write plain `.omd` files. No templates needed.
- **Rust-powered**: Builds complete in milliseconds.
- **Static output**: Deploy `.orbit/` to any CDN or host.
- **Zero JS default**: Clean, fast pages out of the box.
[/features]
```

Each list item is parsed as a feature entry. Bold text at the start becomes
the feature title. The rest of the item is the body.

#### Rendered HTML

```html
<ul class="orbit-features">
  <li class="orbit-feature">
    <strong class="orbit-feature-title">Markdown-first</strong>
    <p class="orbit-feature-body">Write plain <code>.omd</code> files.</p>
  </li>
  ...
</ul>
```

---

### 7.5 Code Groups (future — v0.4+)

**Not in v0.2 or v0.3. Documented here as a reserved block name.**

```omd
[code-group]
```bash title="npm"
npm install orbit-md
```
```bash title="cargo"
cargo install orbit-md
```
[/code-group]
```

---

### 7.6 Tabs (future — v0.4+)

**Not in v0.2 or v0.3. Documented here as a reserved block name.**

```omd
[tabs]
[tab title="macOS"]
Run `brew install orbit`.
[/tab]
[tab title="Windows"]
Run `winget install orbit-md`.
[/tab]
[/tabs]
```

---

## 8. Inline Styling (future — v0.3+)

Standard Markdown inline syntax covers most needs. One addition is planned:

### Link buttons

```omd
[Get started](/docs/getting-started){.button}
[View on GitHub](https://github.com/...){.button .secondary}
```

Link attributes in curly braces add CSS classes to the anchor tag.

**OPEN:** This requires a Markdown-aware parser extension. It is planned for
v0.3 after the block system is stable.

---

## 9. Built-In Styling System

The `.omd` format is designed around an integrated styling system. Users do
not write CSS. Orbit emits the CSS.

### 9.1 Design tokens

Each built-in theme exposes a set of CSS custom properties:

```css
:root {
  /* Typography */
  --orbit-font-body: 'Inter', system-ui, sans-serif;
  --orbit-font-mono: 'JetBrains Mono', 'Fira Code', monospace;
  --orbit-font-size-base: 1rem;
  --orbit-line-height: 1.7;

  /* Spacing */
  --orbit-spacing-xs: 0.25rem;
  --orbit-spacing-sm: 0.5rem;
  --orbit-spacing-md: 1rem;
  --orbit-spacing-lg: 2rem;
  --orbit-spacing-xl: 4rem;

  /* Colors — neutral */
  --orbit-color-bg: #ffffff;
  --orbit-color-surface: #f8f9fa;
  --orbit-color-border: #e2e8f0;
  --orbit-color-text: #1a202c;
  --orbit-color-muted: #718096;

  /* Colors — accent */
  --orbit-color-primary: #5c6bc0;
  --orbit-color-primary-light: #e8eaf6;

  /* Colors — semantic */
  --orbit-color-note: #3b82f6;
  --orbit-color-info: #06b6d4;
  --orbit-color-warning: #f59e0b;
  --orbit-color-danger: #ef4444;
  --orbit-color-success: #22c55e;
  --orbit-color-tip: #a855f7;

  /* Radius */
  --orbit-radius-sm: 0.25rem;
  --orbit-radius-md: 0.5rem;
  --orbit-radius-lg: 1rem;

  /* Shadows */
  --orbit-shadow-sm: 0 1px 2px rgba(0,0,0,0.05);
  --orbit-shadow-md: 0 4px 6px rgba(0,0,0,0.07);
}
```

### 9.2 Dark mode

The default theme supports dark mode via `prefers-color-scheme`:

```css
@media (prefers-color-scheme: dark) {
  :root {
    --orbit-color-bg: #0f172a;
    --orbit-color-surface: #1e293b;
    --orbit-color-border: #334155;
    --orbit-color-text: #f1f5f9;
    --orbit-color-muted: #94a3b8;
  }
}
```

### 9.3 Themes

The built-in theme set for v0.2:

| Theme name  | Description                                        |
|-------------|----------------------------------------------------|
| `default`   | Clean, readable. Blue accent. Light + dark mode.   |

Planned themes for future releases:

| Theme name  | Description                                        |
|-------------|----------------------------------------------------|
| `minimal`   | No color, maximum readability. Editorial feel.     |
| `terminal`  | Dark background, monospace everywhere. Hacker vibe.|
| `editorial` | Serif body font. Long-form writing optimized.      |

Themes are Rust string constants embedded in the binary. No external CSS file
is required.

### 9.4 CSS class naming convention

All Orbit-generated classes are prefixed with `orbit-`:

```
orbit-callout
orbit-callout-warning
orbit-callout-title
orbit-callout-body
orbit-card
orbit-card-grid
orbit-steps
orbit-step
orbit-features
orbit-feature
```

This namespace prevents collisions with user CSS when custom templates are used.

---

## 10. Built-In Layouts

Layouts define the HTML document shell: `<head>`, `<body>`, navigation,
footer, and content area.

### Layout: `docs`

```text
┌─────────────────────────────────────────────┐
│ Site header with title and top nav           │
├──────────────┬──────────────────────────────┤
│ Sidebar nav  │  Article content              │
│ (from pages  │  (max-width: 70ch)            │
│  nav_order)  │                               │
└──────────────┴──────────────────────────────┘
```

- Left sidebar generated from all pages with `layout: docs`, sorted by
  `nav_order` then alphabetically.
- Table of contents generated from `##` and `###` headings on the right side
  (on wide screens).

### Layout: `page`

```text
┌─────────────────────────────────────────────┐
│ Site header                                  │
├─────────────────────────────────────────────┤
│ Content (max-width: 70ch, centered)          │
└─────────────────────────────────────────────┘
```

General purpose. No sidebar.

### Layout: `landing`

```text
┌─────────────────────────────────────────────┐
│ Site header with nav                         │
├─────────────────────────────────────────────┤
│ Hero section (full-width)                    │
├─────────────────────────────────────────────┤
│ Content blocks (full-width, max 1200px)      │
├─────────────────────────────────────────────┤
│ Footer                                       │
└─────────────────────────────────────────────┘
```

Optimized for product landing pages. Feature grids and cards look best here.

### Custom layout override

If `templates/base.hbs` exists, Orbit uses it instead of the built-in layout.
This preserves backward compatibility for existing projects.

The custom template receives the same context:

```text
title          string
site_title     string
content        string  (compiled HTML body)
date           string?
description    string?
tags           [string]
theme_css      string  (built-in theme CSS, empty if user provides own CSS)
```

---

## 11. Compiler Architecture for `.omd`

### Module layout

```text
src/orbit_markdown/
  mod.rs          Public API: compile_omd(source, config) -> Result<RenderedPage>
  ast.rs          OrbitNode enum and supporting types
  parser.rs       .omd source → OrbitNode tree
  directives.rs   Block type registry, attribute parsing, validation
  render.rs       OrbitNode tree → HTML fragment
  theme.rs        Built-in CSS theme strings
  layout.rs       Built-in HTML document shells
```

### AST types

```rust
pub enum OrbitNode {
    /// Raw Markdown fragment, passed to pulldown-cmark.
    Markdown(String),

    /// Pre-rendered HTML passthrough.
    Html(String),

    /// Callout block (note, warning, etc.).
    Callout {
        kind: CalloutKind,
        title: Option<String>,
        children: Vec<OrbitNode>,
    },

    /// Card with optional link.
    Card {
        title: Option<String>,
        href: Option<String>,
        accent: Option<String>,
        children: Vec<OrbitNode>,
    },

    /// Sequence of adjacent cards to be wrapped in a grid.
    CardGrid {
        cards: Vec<OrbitNode>,
    },

    /// Numbered how-to steps.
    Steps {
        items: Vec<StepItem>,
    },

    /// Feature grid for landing pages.
    Features {
        items: Vec<FeatureItem>,
    },
}

pub enum CalloutKind {
    Note,
    Info,
    Warning,
    Danger,
    Success,
    Tip,
}

pub struct StepItem {
    pub body: Vec<OrbitNode>,
}

pub struct FeatureItem {
    pub title: String,
    pub body: Vec<OrbitNode>,
}
```

### Parse pipeline

```text
Raw .omd source
  → split front matter
  → scan for fenced code blocks (mark as opaque)
  → scan for Orbit blocks [block]...[/block]
  → build OrbitNode tree
  → pass Markdown segments to pulldown-cmark
  → merge into final HTML fragment
  → wrap in built-in layout
  → inject theme CSS
  → RenderedPage
```

### Context awareness rule

The parser must track these states as it scans:

```text
NORMAL         — Orbit blocks and Markdown both active
IN_CODE_FENCE  — neither Orbit blocks nor Markdown extensions active
IN_INLINE_CODE — Orbit blocks inactive
```

This is implemented as a state machine over lines, not a regex scan.

### Card grouping rule

Adjacent `[card]` blocks at the same nesting level are automatically grouped
into a `CardGrid` node during AST construction. This happens after parsing,
not during.

```text
[card] [card] [card]  →  CardGrid([card, card, card])
[card] [note] [card]  →  Card, Callout, Card (no grouping across other blocks)
```

---

## 12. Error Reporting

Errors produced by the `.omd` compiler must include:

```text
path     — absolute or project-relative file path
line     — 1-indexed line number of the offending content
column   — 1-indexed column (best effort)
message  — human-readable description
hint     — optional suggestion
```

### Example error messages

```
error[E001]: unclosed block
  --> content/docs/start.omd:14
   |
14 | [warning title="Watch out"]
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^ opened here
   |
   = hint: add [/warning] to close this block

error[E002]: unknown block type
  --> content/index.omd:22
   |
22 | [warnnig]
   | ^^^^^^^^^ unknown block type `warnnig`
   |
   = hint: did you mean `warning`?

error[E003]: missing required attribute
  --> content/docs/api.omd:8
   |
 8 | [card]
   | ^^^^^^ block `card` requires a `title` attribute
   |
   = hint: try [card title="Your title"]
```

### Error code registry

| Code  | Meaning                          |
|-------|----------------------------------|
| E001  | Unclosed block                   |
| E002  | Unknown block type               |
| E003  | Missing required attribute       |
| E004  | Invalid attribute value          |
| E005  | Unexpected closing tag           |
| E006  | Illegal nesting                  |
| E007  | Malformed front matter YAML      |

The `PageError` type must be extended to include `line` and `column` before
the `.omd` parser ships.

---

## 13. Editor Support Plan

`.omd` files get no built-in editor support on day one. This is the main
adoption risk. The mitigation plan:

### Phase A — Syntax highlighting (ship with v0.2)

Publish a TextMate grammar (`omd.tmLanguage.json`). This powers syntax
highlighting in:

- VS Code (via extension or `editors/` directory in the repo)
- Sublime Text
- Any editor using TextMate grammar bundles

The grammar needs to cover:

- Front matter block (YAML)
- Orbit blocks `[block-type]...[/block-type]`
- Standard Markdown within block bodies
- Fenced code blocks inside `.omd`

A minimal grammar is approximately 150-200 lines of JSON. This is achievable
in a single sitting.

### Phase B — VS Code extension (v0.3)

A dedicated VS Code extension providing:

- Syntax highlighting (from Phase A)
- Block type completion (type `[` → suggestions appear)
- Attribute completion per block type
- Hover documentation (e.g., hover over `[warning]` shows supported attributes)
- Lint: underline unknown block types
- Preview panel (run `orbit build` in background, show HTML output)

The extension lives in `editors/vscode/` in the repository.

### Phase C — Language Server (v0.4+)

A Language Server Protocol (LSP) implementation in Rust, powering:

- All Phase B features
- Go-to definition for cross-page links
- Rename refactor for page slugs
- Diagnostics streamed while typing
- Works in any LSP-compatible editor (Neovim, Helix, Emacs, Zed)

The language server reuses the parser from `src/orbit_markdown/parser.rs`.

---

## 14. Implementation Phases

### v0.2 — Core Language

**Goal:** Ship `.omd` as the primary format for new Orbit projects.

- [ ] `src/orbit_markdown/` module scaffold
- [ ] Front matter parsing (reuse existing, extend for `layout`, `theme`, `nav_order`)
- [ ] Orbit block parser: `[block]...[/block]`, attributes, self-closing
- [ ] Callout blocks: `note`, `info`, `warning`, `danger`, `success`, `tip`
- [ ] Steps block
- [ ] Card block + automatic card grid grouping
- [ ] Feature grid block
- [ ] AST → HTML renderer with `orbit-` CSS classes
- [ ] Built-in `default` theme CSS embedded in binary
- [ ] Built-in `docs` layout HTML
- [ ] Built-in `page` layout HTML
- [ ] Error reporting with line numbers
- [ ] E001–E007 error codes
- [ ] Parser rule: never parse blocks inside fenced code or inline code
- [ ] Extended `PageError` with `line` and `column`
- [ ] Scaffold updated: new `orbit init` generates `.omd` files only
- [ ] `orbit build` detects `.omd` files and routes to new compiler
- [ ] `orbit new page` creates `.omd` files
- [ ] TextMate grammar for syntax highlighting
- [ ] Tests: all block types, error messages, code-fence rule, front matter

### v0.3 — Polish and Tooling

**Goal:** Make the day-to-day authoring experience smooth.

- [ ] Built-in `landing` layout
- [ ] Dark mode in `default` theme
- [ ] `minimal` theme
- [ ] Link attributes `[text](url){.button}` for styled links
- [ ] VS Code extension (Phase B above)
- [ ] `orbit migrate` command: rename `.md` → `.omd` and convert JSX tags
- [ ] Auto-generated sidebar navigation for `docs` layout
- [ ] Table of contents sidebar for `docs` layout
- [ ] `nav_order` sorting for sidebar
- [ ] Improved card grid: responsive column count

### v0.4 — Advanced Features

**Goal:** Cover remaining common patterns.

- [ ] `terminal` and `editorial` themes
- [ ] Code groups block
- [ ] Tabs block
- [ ] LSP implementation
- [ ] Site search (static index emitted at build time)
- [ ] Custom theme authoring (override CSS custom properties in `orbit.yaml`)
- [ ] Full deprecation notices for legacy `.hbs` component workflow

---

## 15. Relationship to Current `.md` Pipeline

During the transition period, `orbit build` handles both formats:

```text
content/
  index.omd         → compiled by orbit_markdown compiler
  blog/post.md      → compiled by legacy Markdown pipeline
```

The compiler chooses the pipeline by file extension:

```rust
match path.extension() {
    Some("omd") => orbit_markdown::compile(source, config),
    Some("md")  => legacy::compile(source, config),
    _           => skip,
}
```

This means existing sites are never broken. Users migrate on their own
schedule.

The legacy `.md` + Handlebars path is maintained until at least v1.0.

---

## 16. Open Questions

These must be decided before the Phase is implemented. Do not start
implementation with unresolved items.

| # | Question | Blocking |
|---|---|---|
| 1 | Is `[block]` the right delimiter, or should we test another? | v0.2 parser |
| 2 | Should `[card]` require `title` or make it optional? | v0.2 renderer |
| 3 | Do we auto-group cards, or require an explicit `[card-grid]`? | v0.2 AST |
| 4 | Should `[warning]` and `[callout warning]` both be valid, or only one form? | v0.2 parser |
| 5 | Should front matter `theme` apply to the whole site or only the page? | v0.2 layout |
| 6 | Is the `default` theme light-mode-only at launch, or does dark mode ship with v0.2? | v0.2 theme |
| 7 | Should inline HTML be allowed in `.omd` by default? | v0.2 parser |
| 8 | Does `orbit dev` hot-reload work with `.omd` from day one? | v0.2 dev server |

---

## 17. What This Is Not

To keep the design honest, these are explicit non-goals:

- **Not an MDX replacement.** `.omd` does not support JavaScript components,
  JSX, or React. It is a writing tool, not a component system.
- **Not a full CMS.** There is no database, no user authentication, and no
  admin UI. It compiles files to static HTML.
- **Not a full CSS framework.** The built-in styling system is for Orbit's
  own generated elements. It does not try to style arbitrary user HTML.
- **Not a competitor to Tailwind.** Users who want fine-grained CSS control
  should use the custom template override path.

---

## 18. Example: Full `.omd` Document

```omd
---
title: Orbit
layout: landing
theme: default
description: A fast, Markdown-native static site generator written in Rust.
---

# Orbit

Write `.omd` files. Get a styled static site. No templates required.

[Get started](/docs/getting-started){.button}
[View source](https://github.com/Abhishek-Jaiswar/orbit-md){.button .secondary}

[features]
- **Markdown-native**: `.omd` is Markdown. Your editor already knows it.
- **Built-in styling**: Callouts, cards, steps. No CSS needed.
- **Rust-powered**: Parallel compilation. Sites build in milliseconds.
- **Zero JavaScript**: Static HTML and CSS by default.
- **Single binary**: `cargo install orbit-md`. Done.
- **Open source**: MIT licensed. Own your content.
[/features]

## How it works

[steps]
- Write your content in `.omd` files
- Run `orbit build`
- Deploy the `.orbit/` folder anywhere
[/steps]

[note]
Orbit keeps all output in `.orbit/`. You can host it on Netlify, GitHub Pages,
Vercel, an S3 bucket, or any static file host.
[/note]

## Built for documentation

[card title="Callouts" href="/docs/callouts"]
Note, warning, success, and danger blocks that look great out of the box.
[/card]

[card title="Steps" href="/docs/steps"]
Numbered how-to sequences with styled step numbers.
[/card]

[card title="Themes" href="/docs/themes"]
Switch themes in one line of front matter. Dark mode included.
[/card]
```

---

*Last updated: 2026-07-13*
*Author: orbit-md contributors*
*Status: Future plan — implementation has not started*
