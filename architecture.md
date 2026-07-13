# Architecture

## What This Project Is

`orbit-md` is a Rust-powered static site generator. It builds documentation-style websites from Markdown files using built-in themes, layouts, and rich directives (`:::directive`).

The installed command is `orbit`.

The project has two roles:

- It is the source code for the `orbit-md` Rust crate and `orbit` CLI.
- It also contains a demo Orbit site in the repository root using `content/` and `orbit.yaml`.

Orbit is designed to feel familiar to documentation writers and developers:

- Pages are written as `.md` files.
- Page metadata is written as YAML front matter.
- Common UI blocks (callouts, cards, buttons, heroes, steps) are written using a native `:::directive` syntax.
- The final output is static HTML in `.orbit/`.

## Main User Workflows

### Create a new site

```bash
orbit init my-site
```

This creates a starter project with:

- `orbit.yaml`
- `.gitignore`
- `content/index.md`
- `content/docs/getting-started.md`

The starter files come from the repository's `scaffold/` directory and are embedded into the binary at compile time.

### Build a site

```bash
orbit build
```

This reads `orbit.yaml`, discovers Markdown pages, parses directives, renders styling and templates using built-in page layouts, and writes static HTML to `.orbit/`.

### Create a new page

```bash
orbit new page blog/hello
```

This creates `content/blog/hello.md` with starter front matter and Markdown content.

## Repository Layout

```text
src/
  main.rs              CLI binary entry point
  lib.rs               Public library API and full build pipeline
  cli/                 clap command definitions and command execution
  config.rs            orbit.yaml loading and defaults
  discovery/           Markdown file discovery and front matter parsing
  parser/              Markdown compilation wrapper
  template/            Built-in page layout wrapper and injection engine
  writer/              Output directory cleanup and HTML file writing
  scaffold/            Project initialization and new-page creation
  models/              Page lifecycle data types
  error.rs             Domain-specific error types
  orbit_markdown/      Built-in directive registry, scanner, renderer, theme, and layouts

scaffold/              Embedded starter project files for `orbit init`
content/               Demo site Markdown content
tests/                 CLI and end-to-end integration tests
.orbit/                Generated demo output
target/                Rust build output
```

## High-Level Architecture

The core architecture is a staged pipeline:

```text
Config
  |
  v
Discover Markdown Pages
  |
  v
Parse Orbit AST (directives + Markdown)
  |
  v
Render Directives & Markdown to HTML
  |
  v
Wrap in Page Layout (default or docs) with Theme CSS
  |
  v
Write HTML to Output Dir (.orbit/)
```

### Staged Pipeline

#### 1. Load configuration

Module: `src/config.rs`

Orbit reads `orbit.yaml` from the project root. If the file is missing, it falls back to sensible default values.

Example configuration:

```yaml
title: My Site
source_dir: content
output_dir: .orbit
layout: default
theme: default
```

#### 2. Discover Markdown pages

Module: `src/discovery/mod.rs`

Orbit recursively walks `source_dir` and collects `.md` files.

For each Markdown file it:

- reads the file
- extracts optional YAML front matter
- stores the remaining Markdown body
- skips the page if `draft: true`

The discovery stage returns `Vec<UncompiledPage>`.

#### 3. Parse Orbit AST

Modules:

- `src/orbit_markdown/ast.rs`
- `src/orbit_markdown/parser.rs`
- `src/orbit_markdown/directives.rs`

Orbit parses the page body into AST nodes (`OrbitNode`). It handles code block boundaries to avoid parsing directives inside fenced or inline code, maps directive properties, groups adjacent cards into a card grid, and enforces nesting validation rules.

#### 4. Render HTML Fragments

Module: `src/orbit_markdown/render.rs`

The AST is compiled to clean HTML. Standard Markdown nodes are compiled using `pulldown-cmark`, and directive nodes are converted into structured HTML with stable `orbit-` prefixed CSS classes.

#### 5. Apply Layout & Theme

Modules:

- `src/orbit_markdown/layout.rs`
- `src/orbit_markdown/theme.rs`
- `src/template/mod.rs`

The generated HTML content is wrapped in the specified page layout (`default` or `docs`). The layout inserts the page title, site title, viewport metadata, layout structural CSS, and the CSS for the chosen theme.

#### 6. Clean and Write Output

Module: `src/writer/mod.rs`

The output directory is cleaned and new HTML pages are written to disk.

## Testing Strategy

- **Unit Tests**: Found in each module. Cover AST parsing, scanner state machine transitions, directive validations, theme contents, and layout rendering.
- **CLI Tests**: In `tests/cli_test.rs`. Exercises init, build, and new commands end-to-end.
- **Integration Tests**: In `tests/integration_test.rs`. Exercises full parallel pipelines, malformed front-matter handling, and deep nested compilation throughput.
