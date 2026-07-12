//! Local development server with watch-and-rebuild.
//!
//! `orbit dev` serves the configured output directory and rebuilds when source files change.

mod server;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::build;
use crate::config::Config;
use crate::error::OrbitError;

/// Options for the development server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevOptions {
    /// Path to `orbit.yaml`.
    pub config_path: PathBuf,
    /// Host to bind (default `127.0.0.1`).
    pub host: String,
    /// Port to bind (default `3000`).
    pub port: u16,
    /// Open the site in the default browser on startup.
    pub open: bool,
}

impl Default for DevOptions {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from(crate::config::DEFAULT_CONFIG_FILE),
            host: "127.0.0.1".to_owned(),
            port: 3000,
            open: false,
        }
    }
}

/// Runs the dev server: initial build, static HTTP server, and file watching.
pub fn run(options: &DevOptions) -> Result<(), OrbitError> {
    let config = Config::load(&options.config_path)?;
    let url = format!("http://{}:{}/", options.host, options.port);

    println!("Building site...");
    let report = build(&config)?;
    println!(
        "Built {} pages in {} ms",
        report.pages_written, report.elapsed_ms
    );

    let output_dir = config.output_dir.clone();
    let server_host = options.host.clone();
    let server_port = options.port;

    thread::spawn(move || {
        if let Err(err) = server::serve(&output_dir, &server_host, server_port) {
            eprintln!("dev server error: {err}");
        }
    });

    // NOTE: brief pause so the server thread binds before we print the URL.
    thread::sleep(Duration::from_millis(100));

    println!();
    println!("  Orbit dev server running at {url}");
    println!("  Watching for changes — press Ctrl+C to stop");
    println!();

    if options.open {
        open_browser(&url);
    }

    watch_and_rebuild(&options.config_path)
}

fn watch_and_rebuild(config_path: &Path) -> Result<(), OrbitError> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |result| {
            if let Ok(event) = result {
                let _ = tx.send(event);
            }
        },
        NotifyConfig::default(),
    )
    .map_err(|err| OrbitError::Config(format!("failed to start file watcher: {err}")))?;

    let config = Config::load(config_path)?;
    watch_project_paths(&mut watcher, config_path, &config)?;

    let debounce = Duration::from_millis(250);
    let mut pending = false;
    let mut last_event = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    pending = true;
                    last_event = Instant::now();
                }
                _ => {}
            },
            Err(RecvTimeoutError::Timeout) => {
                if pending && last_event.elapsed() >= debounce {
                    pending = false;
                    rebuild(config_path);
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    Ok(())
}

fn rebuild(config_path: &Path) {
    match Config::load(config_path).and_then(|config| build(&config).map(|report| (config, report)))
    {
        Ok((config, report)) => {
            println!(
                "Rebuilt {} pages in {} ms → {}",
                report.pages_written,
                report.elapsed_ms,
                config.output_dir.display()
            );
        }
        Err(err) => {
            eprintln!("Rebuild failed: {err}");
        }
    }
}

fn watch_project_paths(
    watcher: &mut RecommendedWatcher,
    config_path: &Path,
    config: &Config,
) -> Result<(), OrbitError> {
    let paths = [
        config_path.to_path_buf(),
        config.source_dir.clone(),
        config.components_dir.clone(),
        config.template_dir.clone(),
    ];

    for path in paths {
        if !path.exists() {
            continue;
        }
        let mode = if path.is_dir() {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        watcher.watch(&path, mode).map_err(|err| {
            OrbitError::Config(format!("failed to watch {}: {err}", path.display()))
        })?;
    }

    Ok(())
}

fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn();

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(url).spawn();

    #[cfg(all(unix, not(target_os = "macos")))]
    let result = std::process::Command::new("xdg-open").arg(url).spawn();

    if let Err(err) = result {
        eprintln!("Could not open browser: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_options_default() {
        let opts = DevOptions::default();
        assert_eq!(opts.port, 3000);
        assert_eq!(opts.host, "127.0.0.1");
    }
}
