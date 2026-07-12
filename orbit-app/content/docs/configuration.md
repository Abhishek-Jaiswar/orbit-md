---
title: Configuration
description: Configure Orbit folders, output, and layout with orbit.yaml.
---

# Configuration

Orbit reads `orbit.yaml` from the project root. The file tells the CLI where to find content, templates, and components.

```yaml
title: Orbit
source_dir: content
output_dir: .orbit
template_dir: templates
components_dir: components
layout: base.hbs
```

## Options

| Key | Purpose |
|---|---|
| `title` | Site title exposed to templates as `site_title` |
| `source_dir` | Markdown source folder |
| `output_dir` | Generated static HTML folder |
| `template_dir` | Folder containing page layouts |
| `components_dir` | Folder containing component `.hbs` files |
| `layout` | Default layout file used to wrap page content |

## Layout Variables

Layouts receive page metadata, site metadata, and compiled content.

```handlebars
<title>{{title}} | {{site_title}}</title>
{{#if description}}
  <meta name="description" content="{{description}}">
{{/if}}
<article>{{{content}}}</article>
```

<Alert type="success" title="Deployment">
Run `orbit build`, then publish the configured `output_dir`. No server runtime is required.
</Alert>

<Button href="/" label="Back to Orbit" />
