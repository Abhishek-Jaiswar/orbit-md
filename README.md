# orbit-md

**Orbit** — a fast static site generator with **React-like components in Markdown**. Write pages in `.md` files, use JSX-style tags for UI components, and compile to production-ready HTML.

Install the CLI globally, scaffold a project, and build — just like Create React App.

## Install

```bash
cargo install orbit-md
```

Requires [Rust](https://rustup.rs/) 1.70+.

## Quick start

```bash
orbit init my-site
cd my-site
orbit dev
```

Open `http://127.0.0.1:3000` — edit files in `content/` and the site rebuilds automatically.

One-off build without the dev server:

```bash
orbit build
```

## Commands

| Command | Description |
|---|---|
| `orbit init <path>` | Scaffold a new project |
| `orbit init my-site --title "My Blog"` | Init with a custom site title |
| `orbit build` | Compile Markdown → HTML (`.orbit/`) |
| `orbit dev` | Local dev server with auto-rebuild on save |
| `orbit dev --open` | Dev server and open browser |
| `orbit dev --port 8080` | Dev server on a custom port |
| `orbit build --config orbit.yaml` | Build with a custom config path |
| `orbit new page blog/hello` | Create a new page under `content/` |

## Project structure

After `orbit init`, your site looks like this:

```
my-site/
├── orbit.yaml            # Site config
├── content/              # Markdown pages
│   └── index.md
├── components/           # Reusable UI components (Handlebars)
│   ├── Alert.hbs
│   ├── Button.hbs
│   └── Card.hbs
├── templates/            # Page layouts
│   └── base.hbs
└── .orbit/               # Generated HTML (after orbit build)
```

## Write pages with components

In any `.md` file under `content/`:

```md
---
title: Welcome
---

<Alert type="info" title="Hello">
You write **Markdown**, not HTML.
</Alert>

<Button href="/docs" label="Get started" />
```

Components live in `components/YourComponent.hbs` and are invoked with PascalCase JSX-style tags.

## Configuration

`orbit.yaml`:

```yaml
title: My Site
source_dir: content
output_dir: .orbit
template_dir: templates
components_dir: components
layout: base.hbs
```

## Publish to crates.io

```bash
cargo login          # one-time
cargo publish        # publishes crate `orbit-md`, installs as `orbit`
```

Users install with `cargo install orbit-md` and run the `orbit` command.

## Development

This repository is both the **Orbit tool source** and a demo site:

```bash
cargo run -- build          # build the demo site in ./.orbit
cargo test                  # run tests
```

## License

MIT
