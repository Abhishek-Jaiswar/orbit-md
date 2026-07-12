# Architecture

## What This Project Is

`orbit-md` is a Rust-powered static site generator. It builds documentation-style websites from Markdown files, Handlebars templates, and reusable JSX-like Markdown components.

The installed command is `orbit`.

The project has two roles:

- It is the source code for the `orbit-md` Rust crate and `orbit` CLI.
- It also contains a small demo Orbit site in the repository root using `content/`, `components/`, `templates/`, and `orbit.yaml`.

Orbit is designed to feel familiar to developers who have used tools like React or modern docs frameworks:

- Pages are written as `.md` files.
- Page metadata is written as YAML front matter.
- Reusable UI blocks are written as Handlebars component templates.
- Markdown pages can invoke components with PascalCase JSX-style tags such as `<Alert>` and `<Button />`.
- The final output is static HTML in `.orbit/`.

## Main User Workflows

### Create a new site

```bash
orbit init my-site
```

This creates a starter project with:

- `orbit.yaml`
- `content/`
- `components/`
- `templates/`

The starter files come from the repository's `scaffold/` directory and are embedded into the binary at compile time.

### Build a site

```bash
orbit build
```

This reads `orbit.yaml`, discovers Markdown pages, expands components, renders templates, and writes static HTML to `.orbit/`.

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
  components/          JSX-style component expansion and component rendering
  parser/              Markdown to HTML conversion
  template/            Handlebars page layout rendering
  writer/              Output directory cleanup and HTML file writing
  scaffold/            Project initialization and new-page creation
  models/              Page lifecycle data types
  error.rs             Domain-specific error types

scaffold/              Embedded starter project files for `orbit init`
content/               Demo site Markdown content
components/            Demo site component templates
templates/             Demo site page layout templates
tests/                 CLI and end-to-end integration tests
.orbit/                Generated demo output
target/                Rust build output
test-app/              Generated local test project, currently untracked
```

## High-Level Architecture

The core architecture is a staged pipeline:

```text
Config
  |
  v
Load Components + Load Templates
  |
  v
Discover Markdown Pages
  |
  v
Expand Components
  |
  v
Compile Markdown to HTML
  |
  v
Render Page Layouts
  |
  v
Write HTML Files
```

The main implementation lives in `src/lib.rs`:

```rust
pub fn build(config: &Config) -> Result<BuildReport, OrbitError>
```

This is the public build API used by the CLI and tests.

## Build Pipeline Details

### 1. Load configuration

Module: `src/config.rs`

Orbit loads site settings from `orbit.yaml`.

Default config filename:

```rust
pub const DEFAULT_CONFIG_FILE: &str = "orbit.yaml";
```

Supported config fields:

- `title`
- `source_dir`
- `output_dir`
- `template_dir`
- `components_dir`
- `layout`

If `orbit.yaml` is missing, Orbit falls back to defaults:

```yaml
title: My Site
source_dir: content
output_dir: .orbit
template_dir: templates
components_dir: components
layout: base.hbs
```

### 2. Load components

Module: `src/components/mod.rs`

Orbit reads every `.hbs` file from `components_dir` and registers it with Handlebars.

For example:

```text
components/Alert.hbs
components/Button.hbs
components/Card.hbs
```

become component names:

```text
Alert
Button
Card
```

Markdown can then use:

```md
<Alert type="info" title="Hello">
This is **Markdown** inside a component.
</Alert>

<Button href="/docs/getting-started.html" label="Read docs" />
```

Component props are passed to the Handlebars template as string values. Component children are compiled from Markdown to HTML and passed as `children`.

### 3. Load templates

Module: `src/template/mod.rs`

Orbit reads `.hbs` files from `template_dir` and registers them with Handlebars.

The default layout is usually:

```text
templates/base.hbs
```

The template receives a page context with:

- `title`
- `site_title`
- `content`
- `date`
- `description`
- `tags`

The page content is already compiled HTML, so layouts should render it with triple braces:

```hbs
<article>{{{content}}}</article>
```

### 4. Clean output directory

Module: `src/writer/mod.rs`

Before building, Orbit removes and recreates the configured output directory.

Default:

```text
.orbit/
```

This makes builds deterministic because old output files are removed.

### 5. Discover Markdown pages

Module: `src/discovery/mod.rs`

Orbit recursively walks `source_dir` and collects `.md` files.

For each Markdown file it:

- reads the file
- extracts optional YAML front matter
- stores the remaining Markdown body
- skips the page if `draft: true`

The discovery stage returns `Vec<UncompiledPage>`.

Front matter example:

```md
---
title: Welcome
date: 2026-07-13
tags:
  - rust
  - orbit
description: A demo page.
---

# Welcome
```

### 6. Expand components

Modules:

- `src/components/mod.rs`
- `src/components/parser.rs`

Before Markdown is compiled, Orbit scans the Markdown body for PascalCase component tags.

Lowercase HTML tags are ignored:

```html
<div>Regular HTML</div>
```

PascalCase tags are treated as Orbit components:

```md
<Alert type="warning">Careful</Alert>
```

The component parser supports:

- block components: `<Alert>children</Alert>`
- self-closing components: `<Button />`
- quoted attributes
- unquoted attribute values
- boolean-like attributes, represented as `"true"`
- nested components with matching names

Nested components are expanded inside-out.

### 7. Compile Markdown

Module: `src/parser/mod.rs`

Orbit uses `pulldown-cmark` to convert Markdown to HTML.

Enabled Markdown extensions:

- strikethrough
- tables
- footnotes
- task lists

This stage turns `UncompiledPage` into `CompiledPage`.

### 8. Render full HTML pages

Module: `src/template/mod.rs`

The template engine wraps each compiled HTML fragment in the configured page layout.

This stage turns `CompiledPage` into `RenderedPage`.

Output paths preserve the content directory structure while changing `.md` to `.html`.

Example:

```text
content/index.md                  -> .orbit/index.html
content/docs/getting-started.md   -> .orbit/docs/getting-started.html
```

### 9. Write HTML files

Module: `src/writer/mod.rs`

Orbit writes rendered pages to disk in parallel. Parent directories are created as needed.

This is the final stage of `orbit build`.

## Data Model

Module: `src/models/`

Orbit models each page as it moves through the pipeline.

### `SourcePage`

Represents a Markdown file after discovery and front matter extraction.

Contains:

- source path
- path relative to the content root
- parsed front matter
- raw Markdown body

### `UncompiledPage`

A wrapper around `SourcePage` marking that the page has not yet been converted to HTML.

### `CompiledPage`

Contains the original source metadata plus the compiled HTML fragment.

This is HTML content only, not a complete document.

### `RenderedPage`

Contains:

- final output path
- complete HTML document

This is ready to write to disk.

## CLI Architecture

Modules:

- `src/main.rs`
- `src/cli/mod.rs`

`src/main.rs` is intentionally small. It parses CLI args and delegates to `cli::execute`.

The CLI is built with `clap`.

Commands:

```text
orbit init <path>
orbit build --config orbit.yaml
orbit new page <path>
```

The CLI does not contain the build logic directly. It loads config and calls the library API:

```rust
build(&site_config)
```

This separation keeps the build system testable without invoking the binary.

## Component System

Orbit components are Handlebars templates invoked from Markdown.

Example component template:

```hbs
<div class="alert alert-{{type}}">
  {{#if title}}<strong>{{title}}</strong>{{/if}}
  {{{children}}}
</div>
```

Example Markdown usage:

```md
<Alert type="info" title="Heads up">
This supports **Markdown**.
</Alert>
```

Rendered behavior:

1. The parser finds `<Alert>`.
2. Attributes become Handlebars context values.
3. Children Markdown is compiled to HTML.
4. `Alert.hbs` is rendered with the context.
5. The rendered component HTML is inserted back into the page body.
6. The full page is compiled as Markdown.

### Island Contract

The component system has a future-facing island hydration contract.

If a component includes a `client` attribute:

```md
<Counter client="load" initial="0" />
```

Orbit wraps the rendered output like this:

```html
<div data-orbit-island="Counter" data-props='{"initial":"0"}'>
  ...
</div>
```

There is currently no client-side hydration runtime in this repository. The wrapper is a contract for future JavaScript island support.

## Parallelism

Orbit uses `rayon` to parallelize independent work:

- Markdown page compilation
- template rendering
- HTML writing
- benchmark-style batch compilation

The pipeline keeps shared state immutable where possible:

- component registry is loaded once
- template engine is loaded once
- pages are transformed independently

This makes the system simple to parallelize without locks in the hot path.

## Error Handling

Module: `src/error.rs`

Orbit has two main error types:

- `OrbitError`: top-level application errors
- `PageError`: per-page pipeline errors

`PageError` is used for failures tied to a specific source or output path, such as:

- malformed front matter
- unknown component
- template render failure
- output write failure

`PageError` can be converted into `OrbitError`.

The current pipeline generally returns the first encountered page error.

## Scaffolding

Module: `src/scaffold/mod.rs`

The `orbit init` command writes a starter project using files embedded with `include_str!`.

Embedded files come from:

```text
scaffold/
```

This is important: after users install the binary with `cargo install`, Orbit can still create a new project without needing external template files.

`orbit new page` also lives in this module. It normalizes paths so:

```bash
orbit new page blog/post
```

creates:

```text
content/blog/post.md
```

## Testing Strategy

Tests cover:

- component parser behavior
- component expansion
- front matter splitting
- template rendering
- scaffold generation
- CLI init/build/new workflows
- full public API build
- large page-set compilation

Important test files:

```text
tests/cli_test.rs
tests/integration_test.rs
```

Useful command:

```bash
cargo test
```

At the time this document was created, the full test suite passed.

## Important Extension Points

### Add a new CLI command

Start in:

```text
src/cli/mod.rs
```

Add a new variant to `Command` or `NewTarget`, then implement it in `execute`.

### Change Markdown behavior

Start in:

```text
src/parser/mod.rs
```

The Markdown options are centralized in `markdown_options`.

### Change component parsing

Start in:

```text
src/components/parser.rs
```

This is a hand-written scanner. It intentionally recognizes PascalCase component names and ignores lowercase HTML tags.

### Change component rendering

Start in:

```text
src/components/mod.rs
```

Look at `ComponentRegistry::render` and `expand_fragment`.

### Change page layout data

Start in:

```text
src/template/mod.rs
src/models/front_matter.rs
```

Add fields to `FrontMatter`, then pass them into `PageContext`.

### Change generated starter project

Edit files in:

```text
scaffold/
```

Then make sure `src/scaffold/mod.rs` includes any new scaffold file in `SCAFFOLD_FILES`.

## Current Design Assumptions

- Components are `.hbs` files.
- Component names are derived from file stems.
- Component invocations must use PascalCase names.
- Component attributes are strings.
- Front matter is YAML.
- Output files mirror content paths and use `.html`.
- Builds clean the output directory before writing.
- The first page error stops the build.
- There is no routing layer beyond static file paths.
- There is no live dev server yet.
- The island wrapper exists, but client hydration is not implemented yet.

## Mental Model for Future Agents

Think of Orbit as a compiler:

```text
Markdown source + front matter + components + layout
        |
        v
static HTML documents
```

Most changes should preserve the pipeline boundaries:

- discovery should only find and load source pages
- component expansion should only transform Markdown component tags into HTML
- Markdown parsing should only produce HTML fragments
- template rendering should only wrap fragments in layouts
- writing should only handle files on disk

Keeping those stages separate is what makes the project easy to test, parallelize, and extend.
