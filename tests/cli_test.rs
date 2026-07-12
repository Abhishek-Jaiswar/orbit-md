//! CLI integration tests.

use std::process::Command;

use clap::Parser;
use orbit_md::cli::{self, Cli};

#[test]
fn cli_init_and_build_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("my-site");

    cli::execute(Cli::parse_from([
        "orbit",
        "init",
        project.to_str().unwrap(),
        "--title",
        "My Site",
    ]))
    .expect("init");

    assert!(project.join("orbit.yaml").exists());
    assert!(project.join("content/index.md").exists());

    let status = Command::new(env!("CARGO_BIN_EXE_orbit"))
        .arg("build")
        .current_dir(&project)
        .status()
        .expect("run orbit build");

    assert!(status.success());
    assert!(project.join(".orbit/index.html").exists());

    let html = std::fs::read_to_string(project.join(".orbit/index.html")).unwrap();
    assert!(html.contains("Welcome to My Site"));
}

#[test]
fn cli_new_page_creates_markdown_file() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("site");
    cli::execute(Cli::parse_from([
        "orbit",
        "init",
        project.to_str().unwrap(),
    ]))
    .unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_orbit"))
        .args(["new", "page", "blog/post"])
        .current_dir(&project)
        .status()
        .unwrap();

    assert!(status.success());
    assert!(project.join("content/blog/post.md").exists());
}
