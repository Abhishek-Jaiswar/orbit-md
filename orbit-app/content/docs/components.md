---
title: Components
description: Build reusable Orbit UI components with Handlebars templates.
---

# Components

Orbit components are `.hbs` files stored in your configured components directory. You use them from Markdown with JSX-style tags.

## Define a component

Create `components/Alert.hbs`:

```handlebars
<div class="alert alert-{{type}}">
  {{#if title}}<strong>{{title}}</strong>{{/if}}
  <div>{{{children}}}</div>
</div>
```

## Use it in Markdown

```markdown
<Alert type="warning" title="Careful">
Inner **Markdown** is compiled before it is passed to the component.
</Alert>
```

<Alert type="warning" title="Careful">
Inner **Markdown** is compiled before it is passed to the component.
</Alert>

## Component Data

Attributes become template variables. Content between the opening and closing tag becomes `children`.

<section class="steps">
  <div class="step">
    <div class="step-number">1</div>
    <h3>Name the file</h3>
    <p>`components/Card.hbs` becomes the `Card` tag.</p>
  </div>
  <div class="step">
    <div class="step-number">2</div>
    <h3>Pass attributes</h3>
    <p>`title="Quick start"` is available as `{{title}}`.</p>
  </div>
  <div class="step">
    <div class="step-number">3</div>
    <h3>Render children</h3>
    <p>Use triple braces with `{{{children}}}` for compiled inner markup.</p>
  </div>
</section>

<Card title="Self-closing components">
Components without children can use self-closing syntax:

```markdown
<Button href="/" label="Back home" />
```
</Card>

<Button href="/docs/configuration.html" label="Read configuration guide" />
