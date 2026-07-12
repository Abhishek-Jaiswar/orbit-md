//! CLI entry point for Orbit.

use std::process;

use clap::Parser;
use orbit_md::cli::{self, Cli};

fn main() {
    if let Err(err) = cli::execute(Cli::parse()) {
        eprintln!("error: {err:#}");
        process::exit(1);
    }
}
