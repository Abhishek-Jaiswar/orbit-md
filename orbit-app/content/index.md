---
title: Welcome
description: Orbit is a Rust static site generator for Markdown pages and reusable Handlebars components.
---

<section class="hero">
  <div class="hero-inner">
    <div>
      <p class="eyebrow">Rust static sites</p>
      <h1>Orbit</h1>
      <p class="hero-copy">Build fast documentation sites from Markdown, reusable components, and one small YAML file. Orbit gives you JSX-style authoring without shipping a client-side app.</p>
      <div class="hero-actions">
        <a class="btn" href="/docs/getting-started.html">Start building</a>
        <a class="btn btn-secondary" href="https://crates.io/crates/orbit-md">View crate</a>
      </div>
      <div class="hero-proof">
        <div class="proof-item"><strong>Markdown first</strong> Write normal content with front matter.</div>
        <div class="proof-item"><strong>Components</strong> Drop reusable Handlebars UI into pages.</div>
        <div class="proof-item"><strong>Static output</strong> Ship plain HTML from `.orbit/`.</div>
      </div>
    </div>
    <div class="orbit-stage" aria-hidden="true">
      <div class="orbit-ring one"></div>
      <div class="orbit-ring two"></div>
      <div class="orbit-ring three"></div>
      <span class="planet blue"></span>
      <span class="planet green"></span>
      <span class="planet rose"></span>
      <div class="terminal">
        <div class="terminal-bar">
          <span class="dot"></span><span class="dot"></span><span class="dot"></span>
          orbit build
        </div>
        <pre><code>$ cargo install orbit-md
$ orbit init docs
$ cd docs
$ orbit build
<span class="ok">Built 4 pages into .orbit/</span></code></pre>
      </div>
    </div>
  </div>
</section>

<section class="band">
  <div class="band-inner">
    <p class="section-kicker">Why Orbit</p>
    <h2 class="section-title">A docs workflow that stays close to the files you already understand.</h2>
    <p class="section-lede">Orbit keeps the authoring model small: Markdown pages for content, Handlebars templates for layout, and component partials for repeated UI. The result is easy to inspect, easy to deploy, and friendly to version control.</p>

    <div class="feature-grid">
      <div class="feature">
        <div class="feature-icon">M</div>
        <h3>Markdown pages</h3>
        <p>Use front matter for metadata and write the body in plain Markdown.</p>
      </div>
      <div class="feature">
        <div class="feature-icon">C</div>
        <h3>JSX-style components</h3>
        <p>Compose pages with tags like `Alert`, `Card`, and `Button` backed by `.hbs` files.</p>
      </div>
      <div class="feature">
        <div class="feature-icon">R</div>
        <h3>Rust CLI</h3>
        <p>Install once with Cargo, then build or serve your site from the terminal.</p>
      </div>
    </div>
  </div>
</section>

## Install

```bash
cargo install orbit-md
```

## Core Commands

<section class="command-grid">
  <div class="command-card">
    <h3>Create a site</h3>
    <p><code>orbit init my-site</code></p>
    <p>Scaffold a new content, component, and template structure.</p>
  </div>
  <div class="command-card">
    <h3>Build static HTML</h3>
    <p><code>orbit build</code></p>
    <p>Compile Markdown pages into the configured output directory.</p>
  </div>
  <div class="command-card">
    <h3>Run locally</h3>
    <p><code>orbit dev</code></p>
    <p>Start the development server with watch-and-rebuild.</p>
  </div>
  <div class="command-card">
    <h3>Add a page</h3>
    <p><code>orbit new page docs/api.md</code></p>
    <p>Create a Markdown page under the content directory.</p>
  </div>
</section>

<Alert type="success" title="Built for documentation">
Orbit is a good fit for project docs, package landing pages, internal guides, and small sites that should stay fast and source-controlled.
</Alert>
