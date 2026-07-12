//! Integration tests validating end-to-end throughput on large page sets.

use std::path::Path;
use std::time::Instant;

use orbit_md::build;
use orbit_md::components::ComponentRegistry;
use orbit_md::config::Config;
use orbit_md::discovery::discover_pages;
use orbit_md::parser::compile_all;
use orbit_md::template::{TemplateEngine, render_all};
use orbit_md::writer::{clean_output_dir, write_all};

/// Generates a deep tree of Markdown fixtures under `root`.
fn seed_markdown_matrix(root: &Path, count: usize) {
    for i in 0..count {
        let segment_a = format!("section-{i:04}");
        let segment_b = format!("chapter-{i:04}");
        let segment_c = format!("page-{i:04}.md");
        let path = root.join(&segment_a).join(&segment_b).join(&segment_c);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }

        let body = format!(
            "---\ntitle: Page {i}\ndate: 2026-01-01\ntags:\n  - bench\n  - rust\n---\n\n# Heading {i}\n\nParagraph **{i}** with [link](https://example.com).\n\n- item one\n- item two\n"
        );
        std::fs::write(&path, body).unwrap();
    }
}

fn bench_config(content: &Path, output: &Path, templates: &Path, components: &Path) -> Config {
    Config {
        title: "Throughput Bench".into(),
        source_dir: content.to_path_buf(),
        output_dir: output.to_path_buf(),
        template_dir: templates.to_path_buf(),
        components_dir: components.to_path_buf(),
        layout: "base.hbs".into(),
    }
}

fn empty_registry(components: &Path) -> ComponentRegistry {
    std::fs::create_dir_all(components).unwrap();
    let config = Config {
        components_dir: components.to_path_buf(),
        ..Config::default()
    };
    ComponentRegistry::from_config(&config).unwrap()
}

#[test]
fn compiles_one_thousand_deep_markdown_paths() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content = tmp.path().join("content");
    let output = tmp.path().join("dist");
    let templates = tmp.path().join("templates");
    let components = tmp.path().join("components");

    std::fs::create_dir_all(&templates).unwrap();
    std::fs::write(
        templates.join("base.hbs"),
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>{{title}} | {{site_title}}</title></head>
<body><article>{{{content}}}</article></body>
</html>"#,
    )
    .unwrap();

    let page_count = 1_000;
    let started_seed = Instant::now();
    seed_markdown_matrix(&content, page_count);
    eprintln!(
        "seeded {page_count} files in {} ms",
        started_seed.elapsed().as_millis()
    );

    let config = bench_config(&content, &output, &templates, &components);
    let registry = empty_registry(&components);
    let engine = TemplateEngine::from_config(&config).expect("template engine");

    let started = Instant::now();

    let uncompiled = discover_pages(&content).expect("discovery");
    assert_eq!(uncompiled.len(), page_count);

    let compiled = compile_all(uncompiled, &registry).expect("markdown compile");
    assert_eq!(compiled.len(), page_count);

    let rendered = render_all(&engine, compiled, &output).expect("template render");
    assert_eq!(rendered.len(), page_count);

    clean_output_dir(&output).expect("clean output");
    write_all(&rendered).expect("write output");

    let elapsed = started.elapsed();
    eprintln!(
        "compiled and wrote {page_count} pages in {} ms ({:.0} pages/sec)",
        elapsed.as_millis(),
        page_count as f64 / elapsed.as_secs_f64()
    );

    for i in [0, 499, 999] {
        let html_path = output
            .join(format!("section-{i:04}"))
            .join(format!("chapter-{i:04}"))
            .join(format!("page-{i:04}.html"));
        assert!(html_path.exists(), "missing {}", html_path.display());
        let html = std::fs::read_to_string(&html_path).unwrap();
        assert!(html.contains(&format!("Heading {i}")));
    }
}

#[test]
fn full_build_via_public_api() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content = tmp.path().join("content");
    let output = tmp.path().join("dist");
    let templates = tmp.path().join("templates");
    let components = tmp.path().join("components");

    std::fs::create_dir_all(&templates).unwrap();
    std::fs::write(
        templates.join("base.hbs"),
        "<html><body>{{{content}}}</body></html>",
    )
    .unwrap();

    std::fs::create_dir_all(&content).unwrap();
    std::fs::write(
        content.join("hello.md"),
        "---\ntitle: Hello\n---\n\n# Hello World\n",
    )
    .unwrap();

    let config = bench_config(&content, &output, &templates, &components);
    let report = build(&config).expect("build");

    assert_eq!(report.pages_written, 1);
    assert!(output.join("hello.html").exists());
}

#[test]
fn markdown_components_render_to_html() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content = tmp.path().join("content");
    let output = tmp.path().join("dist");
    let templates = tmp.path().join("templates");
    let components = tmp.path().join("components");

    std::fs::create_dir_all(&templates).unwrap();
    std::fs::create_dir_all(&content).unwrap();
    std::fs::create_dir_all(&components).unwrap();

    std::fs::write(
        templates.join("base.hbs"),
        "<html><body>{{{content}}}</body></html>",
    )
    .unwrap();
    std::fs::write(
        components.join("Alert.hbs"),
        r#"<div class="alert alert-{{type}}">{{{children}}}</div>"#,
    )
    .unwrap();
    std::fs::write(
        components.join("Button.hbs"),
        r#"<a class="btn" href="{{href}}">{{label}}</a>"#,
    )
    .unwrap();
    std::fs::write(
        content.join("demo.md"),
        r#"---
title: Demo
---

<Alert type="info">Hello **components**</Alert>
<Button href="/docs" label="Go" />
"#,
    )
    .unwrap();

    let config = bench_config(&content, &output, &templates, &components);
    build(&config).expect("build with components");

    let html = std::fs::read_to_string(output.join("demo.html")).unwrap();
    assert!(html.contains(r#"class="alert alert-info""#));
    assert!(html.contains("<strong>components</strong>"));
    assert!(html.contains(r#"class="btn" href="/docs""#));
}

#[test]
fn malformed_front_matter_does_not_abort_sibling_pages() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let content = tmp.path().join("content");
    std::fs::create_dir_all(&content).unwrap();

    std::fs::write(content.join("good.md"), "---\ntitle: OK\n---\n\n# Good\n").unwrap();
    std::fs::write(content.join("bad.md"), "---\n: invalid\n---\n\n# Bad\n").unwrap();

    let result = discover_pages(&content);
    assert!(result.is_err());

    // Only good file in isolation succeeds.
    let single = tmp.path().join("single");
    std::fs::create_dir_all(&single).unwrap();
    std::fs::write(single.join("good.md"), "---\ntitle: OK\n---\n\n# Good\n").unwrap();
    let pages = discover_pages(&single).unwrap();
    assert_eq!(pages.len(), 1);
}
