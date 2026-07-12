# Agent Guidelines For Orbit

These instructions are for coding agents working in this repository.

Orbit is a Rust CLI and library for compiling Markdown-based sites. The next major direction is Orbit Markdown: a Markdown-compatible language with built-in directives, layouts, and themes.

## Working Principles

- Keep user-facing authoring simple: users should write `.md` files first.
- Prefer clear compiler stages over broad string replacement.
- Keep modules small and named by responsibility.
- Preserve existing behavior unless the task explicitly changes it.
- Add tests for parser/compiler behavior before or alongside implementation.
- Run `cargo fmt` and `cargo test` before considering work complete.
- Do not remove user work or generated experiments unless asked.

## Code Style

- Write straightforward Rust with explicit types where they help readability.
- Prefer small pure functions for parsing, transformation, and rendering.
- Avoid large functions that parse, transform, and render in one pass.
- Use domain names consistently:
  - `directive` for `:::` Orbit Markdown blocks
  - `node` for Orbit AST nodes
  - `render` for AST or model to HTML conversion
  - `theme` for built-in CSS and visual tokens
  - `layout` for full-page HTML structure
- Keep comments useful and sparse. Comment why a parser rule exists, not what every line does.
- Use `PageError` for source-page failures. Extend it carefully if line/column reporting is added.

## Parser And Compiler Rules

- Do not parse Orbit directives inside fenced code blocks.
- Do not parse Orbit directives inside inline code.
- Keep valid Markdown valid. Orbit Markdown is Markdown plus extensions.
- Prefer Markdown-aware parsing over raw text scanning.
- Add regression tests for every parser edge case.
- Use stable CSS class names prefixed with `orbit-` for built-in rendered HTML.

## Module Organization

When building Orbit Markdown, prefer this structure:

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

- `ast.rs`: semantic Orbit Markdown nodes and supporting types.
- `parser.rs`: Markdown-aware parsing and directive recognition.
- `directives.rs`: directive names, attributes, validation, and errors.
- `render.rs`: convert Orbit nodes to HTML.
- `theme.rs`: built-in CSS/theme assets.
- `layout.rs`: built-in page shell rendering.
- `mod.rs`: public API only; avoid implementation bloat.

If a module grows too large, split by responsibility instead of creating generic utility files.

## Testing Expectations

Parser/compiler tests should cover:

- plain Markdown still works
- callouts render correctly
- card blocks render correctly
- steps render correctly
- directives inside fenced code are ignored
- directives inside inline code are ignored
- malformed directives return useful errors
- nested directive behavior is defined and tested

Use integration tests when the behavior crosses the full build pipeline.

## Documentation Expectations

When adding a language feature, update:

- `docs/orbit-markdown-architecture.md` for design-level decisions
- user docs under `content/docs/` when the feature is user-facing
- `architecture.md` when the high-level repository architecture changes

Examples in docs should avoid unsupported syntax. If a limitation exists, name it clearly.

## Release Hygiene

- Bump `Cargo.toml` version for every crates.io release after `0.1.0`.
- Run:

```bash
cargo fmt
cargo test
cargo publish --dry-run
```

- Published crates.io versions are immutable. Never assume a published version can be overwritten.

