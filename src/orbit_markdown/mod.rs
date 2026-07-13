//! Orbit Markdown compiler — `:::directive` extension layer for `.md` files.
//!
//! # Module structure
//!
//! | Module          | Responsibility                                    |
//! |-----------------|---------------------------------------------------|
//! | [`ast`]         | `OrbitNode` tree and supporting types             |
//! | `directives`    | Directive registry, attribute parsing, validation |
//! | `parser`        | Markdown-aware `:::directive` scanner             |
//! | `render`        | `OrbitNode` tree → HTML fragment                  |
//! | `theme`         | Built-in CSS themes (embedded strings)            |
//! | `layout`        | Built-in HTML page shells                         |
//!
//! # Status
//!
//! Phase 2 — `ast` module complete. Remaining modules are stubs pending
//! implementation in subsequent phases.

pub mod ast;
pub mod directives;
pub mod layout;
pub mod parser;
pub mod pipeline;
pub mod render;
pub mod theme;
