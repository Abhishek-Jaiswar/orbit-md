---
title: Getting Started
description: Learn how to use built-in Markdown directives.
layout: docs
---

# Getting Started

Welcome to your new Orbit documentation page! Orbit allows you to structure docs using built-in directives without writing any templates or styles.

:::nav-group title="Documentation"
- [Home](/)
- [Getting Started](/docs/getting-started.html)
:::

## Directives Overview

Here are the most common blocks you can use:

### 1. Callouts
Use callouts to highlight tips, warnings, or info:

:::tip title="Pro Tip"
Use `:::tip` or `:::note` to style informative callouts for readers.
:::

:::warning title="Important warning"
Watch out for nesting violations! Directives cannot be nested except for buttons inside hero.
:::

### 2. Ordered Steps
Show a sequence of instructions using the steps directive:

:::steps
1. **Edit content**: Open `content/index.md` or this file.
2. **Preview site**: Run `orbit dev` and navigate to `http://localhost:3000`.
3. **Build site**: Run `orbit build` to compile production static HTML under `.orbit/`.
:::

### 3. Cards & Actions
Group info together and add CTAs:

:::card title="Quick Resource"
Check out the CLI command reference to learn all options.
:::

:::buttons
[CLI Commands](#commands-reference) primary
:::

## Commands Reference

| Command | Description |
|---|---|
| `orbit dev` | Local dev server with watch-and-rebuild |
| `orbit build` | Compile Markdown pages to HTML |
| `orbit new page <path>` | Create a new Markdown page |
| `orbit init <dir>` | Scaffold a new project |
