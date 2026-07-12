//! Command-line interface for Orbit.

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::build;
use crate::config::{Config, DEFAULT_CONFIG_FILE};
use crate::scaffold::{self, title_from_slug, InitOptions};

/// Fast static site generator with React-like Markdown components.
#[derive(Debug, Parser)]
#[command(
    name = "orbit",
    bin_name = "orbit",
    version,
    about = "Orbit — Markdown static sites with React-like components",
    long_about = "Orbit (orbit-md) compiles Markdown pages and JSX-style components into static HTML.\n\nInstall: cargo install orbit-md\nInit:    orbit init my-site\nBuild:   orbit build"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Available CLI commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new Orbit site in a directory.
    Init {
        /// Project directory to create (e.g. `my-site`).
        path: PathBuf,

        /// Site title used in config and starter pages.
        #[arg(short, long)]
        title: Option<String>,
    },

    /// Compile Markdown pages to HTML in `dist/`.
    Build {
        /// Path to the site config file.
        #[arg(short, long, default_value = DEFAULT_CONFIG_FILE)]
        config: PathBuf,
    },

    /// Create a new Markdown page under `content/`.
    New {
        #[command(subcommand)]
        target: NewTarget,
    },
}

/// Targets for `orbit new`.
#[derive(Debug, Subcommand)]
pub enum NewTarget {
    /// Create a new page file.
    Page {
        /// Page path relative to `content/` (extension optional).
        path: PathBuf,

        /// Page title for front matter.
        #[arg(short, long)]
        title: Option<String>,
    },
}

/// Executes the parsed CLI command.
pub fn execute(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Init { path, title } => {
            let mut options = InitOptions::from_path(path.clone());
            if let Some(custom_title) = title {
                options.title = custom_title;
            }

            scaffold::init_project(&options).map_err(anyhow::Error::from)?;
            print_init_success(&path);
        }
        Command::Build { config } => {
            let site_config = Config::load(&config)
                .map_err(anyhow::Error::from)
                .map_err(|err| anyhow::anyhow!("loading config from {}: {err}", config.display()))?;

            let report = build(&site_config).map_err(anyhow::Error::from)?;
            println!(
                "Built {} pages in {} ms → {}",
                report.pages_written,
                report.elapsed_ms,
                site_config.output_dir.display()
            );
        }
        Command::New { target } => match target {
            NewTarget::Page { path, title } => {
                let config = Config::load(DEFAULT_CONFIG_FILE).map_err(anyhow::Error::from)?;
                let page_title = title.unwrap_or_else(|| default_page_title(&path));
                let created =
                    scaffold::new_page(&config.source_dir, &path, &page_title).map_err(anyhow::Error::from)?;
                println!("Created {}", created.display());
            }
        },
    }

    Ok(())
}

fn print_init_success(path: &Path) {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("your-site");

    println!("Created Orbit project at {}", path.display());
    println!();
    println!("  cd {name}");
    println!("  orbit build");
    println!();
    println!("Edit Markdown in content/ and components in components/.");
}

fn default_page_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(title_from_slug)
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Untitled".to_owned())
}
