# Project Structure Guide

This document defines the intended folder and module structure for Orbit as it evolves from a static site generator with Handlebars components into an Orbit Markdown compiler.

## Current Top-Level Structure

```text
.
+-- src/                 Rust crate source
+-- scaffold/            Files embedded into `orbit init`
+-- content/             Demo/documentation site content
+-- components/          Demo-site Handlebars components
+-- templates/           Demo-site Handlebars layouts
+-- docs/                Internal architecture and planning docs
+-- tests/               Integration and CLI tests
+-- architecture.md      High-level repository architecture
+-- Cargo.toml           Crate metadata and dependencies
+-- README.md            Public project README
```

## Current Source Modules

```text
src/
+-- main.rs              CLI binary entry point
+-- lib.rs               Public library API and build pipeline
+-- cli/                 CLI commands and command execution
+-- config.rs            Site configuration loading
+-- discovery/           Markdown discovery and front matter parsing
+-- parser/              Markdown to HTML compilation
+-- components/          Legacy JSX-style component expansion
+-- template/            Handlebars layout rendering
+-- writer/              Output directory cleanup and writes
+-- scaffold/            Project scaffolding and page creation
+-- dev/                 Local dev server and file watcher
+-- models/              Page lifecycle data types
+-- error.rs             Error types
```

## Planned Orbit Markdown Modules

Add a dedicated compiler module instead of spreading language logic through the existing parser and component modules.

```text
src/orbit_markdown/
+-- mod.rs               Public compiler API
+-- ast.rs               Orbit Markdown semantic nodes
+-- parser.rs            Markdown-aware directive parser
+-- directives.rs        Directive definitions and validation
+-- render.rs            AST to HTML renderer
+-- theme.rs             Built-in CSS themes
+-- layout.rs            Built-in page layouts
```

### Responsibilities

`mod.rs`

- Exposes the stable API for compiling Orbit Markdown.
- Should stay small.

`ast.rs`

- Defines semantic structures such as `OrbitNode`, `CalloutKind`, `FeatureItem`, and `StepItem`.
- Should not perform parsing or rendering.

`parser.rs`

- Converts Markdown input into Orbit-aware structures.
- Must be aware of fenced code blocks and inline code.
- Should not contain theme CSS or layout HTML.

`directives.rs`

- Knows directive names, aliases, attributes, and validation rules.
- Good home for parsing `title="..."` style attributes.

`render.rs`

- Converts Orbit nodes into semantic HTML.
- Uses stable `orbit-` CSS class names.

`theme.rs`

- Contains built-in CSS for default themes.
- Avoid mixing theme strings into parser or renderer logic.

`layout.rs`

- Renders the full HTML document shell when users do not provide a custom template.
- Owns site nav, document head, and page wrapper decisions.

## Compatibility Boundary

Existing modules should not disappear immediately.

```text
components/      legacy and advanced component support
template/        legacy and advanced layout support
parser/          base Markdown rendering
orbit_markdown/  new language compiler
```

The v0.2 path should prefer Orbit Markdown, but keep the old Handlebars path working until a deliberate breaking release.

## Maintainer Rules

- Put new language features under `src/orbit_markdown/`.
- Put reusable site docs in `content/docs/`.
- Put internal design notes in `docs/`.
- Keep generated output out of git: `.orbit/`, `dist/`, and `target/`.
- Do not add broad `utils.rs` modules unless there is a very clear shared responsibility.
- Prefer a small module with a specific name over one large mixed module.
