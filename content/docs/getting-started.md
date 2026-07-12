---
title: Getting Started
description: Learn how to use JSX-style components in Markdown.
---

# Getting Started

Write pages in Markdown and drop in reusable components — just like React, but static.

<Card title="Quick Example">

<Alert type="warning" title="Tip">
Nest components freely. Inner Markdown is compiled automatically.
</Alert>

Use self-closing tags for simple widgets:

<Button href="/" label="Back home" />

</Card>

## How it works

1. Create a component in `components/YourComponent.hbs`
2. Use it in any `.md` file with JSX-style tags (see the Alert example above)
3. Run `orbit build` — HTML lands in `dist/`

## Commands

| Command | Description |
|---|---|
| `orbit build` | Compile Markdown pages to HTML |
| `orbit new page path/to/page.md` | Create a new Markdown page |
| `orbit init my-site` | Scaffold a new project |
