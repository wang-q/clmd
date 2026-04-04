//! Document writers for various output formats.
//!
//! This module provides a unified interface for writing documents to different
//! formats, inspired by Pandoc's Writer system. Writers convert the internal AST
//! representation to the target format.
//!
//! # Example
//!
//! ```ignore
//! use clmd::io::writer::{WriterRegistry, Writer};
//! use clmd::options::{WriterOptions, OutputFormat};
//! use clmd::{parse_document, Options};
//!
//! let registry = WriterRegistry::new();
//! let writer = registry.get_by_name("html").unwrap();
//!
//! let options = Options::default();
//! let (arena, root) = parse_document("# Hello World", &options);
//! let writer_options = WriterOptions::default();
//! let output = writer.write(&arena, root, &ctx, &writer_options).unwrap();
//! ```

pub mod registry;
pub use registry::*;

pub mod shared;
pub use shared::*;

pub mod html_renderer;
pub use html_renderer::*;

pub mod latex_shared;
pub use latex_shared::*;

pub mod beamer;
pub mod bibtex;
pub mod latex;
pub mod man;
pub mod pdf;
pub mod revealjs;
pub mod rtf;
pub mod typst;
pub mod xml;
