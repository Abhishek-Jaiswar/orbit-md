---
title: Getting Started
description: Learn how to use built-in Markdown directives.
---

# Getting Started

Write pages in Markdown and use built-in directives for callouts, cards, buttons, layouts and more.

:::card title="Quick Example"
This card directive is a container for grouped information. Below we have some primary actions.
:::

:::buttons
[Back home](/) primary
:::

:::tip title="Tip"
Directives can contain any Markdown content. Inner Markdown is compiled automatically.
:::

## How it works

1. Add a directive using the `:::` syntax (like the tip and card examples above).
2. Run `orbit build` — HTML lands in `.orbit/`.

## Commands

| Command | Description |
|---|---|
| `orbit dev` | Local dev server with watch-and-rebuild |
| `orbit build` | Compile Markdown pages to HTML |
| `orbit new page path/to/page.md` | Create a new Markdown page |
| `orbit init my-site` | Scaffold a new project |
