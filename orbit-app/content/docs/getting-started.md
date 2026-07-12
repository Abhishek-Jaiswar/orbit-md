---
title: Getting Started
description: Install Orbit, create a site, and build your first static pages.
---

# Getting Started

Orbit is published as the Rust crate `orbit-md` and installs the `orbit` command.

## Install

```bash
cargo install orbit-md
```

## Create a site

```bash
orbit init my-site
cd my-site
orbit dev
```

The development server watches your files, rebuilds the site, and serves the generated HTML locally.

## Project structure

```text
my-site/
  orbit.yaml
  content/
    index.md
  components/
    Alert.hbs
    Button.hbs
    Card.hbs
  templates/
    base.hbs
```

<Card title="The Orbit model">
Markdown owns your writing, Handlebars owns your HTML, and `orbit.yaml` connects the folders together.
</Card>

## Write a page

Every page is a Markdown file with optional front matter:

```markdown
---
title: Welcome
description: My first Orbit page.
---

# Welcome

<Alert type="info" title="Hello">
This content is Markdown inside a reusable component.
</Alert>
```

## Build

```bash
orbit build
```

Orbit writes static HTML to `.orbit/` by default. Deploy that directory to any static host.

<Alert type="info" title="Next step">
Learn how components work, then tune `orbit.yaml` for your site structure.
</Alert>

## Commands Reference

| Command | Description |
|---|---|
| `orbit dev` | Local dev server with watch-and-rebuild |
| `orbit build` | Compile Markdown pages to HTML |
| `orbit new page path/to/page.md` | Create a new Markdown page |
| `orbit init my-site` | Scaffold a new project |

<Button href="/docs/components.html" label="Read components guide" />
