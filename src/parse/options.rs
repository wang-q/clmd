//! Configuration for the parser and renderer. Extensions affect both.
//!
//! This module re-exports options from the unified `crate::options` module
//! for backward compatibility.
//!
//! # Example
//!
//! ```ignore
//! use clmd::Options;
//!
//! let mut options = Options::default();
//! options.extension.table = true;
//! options.extension.strikethrough = true;
//! options.render.hardbreaks = true;
//! ```

// Re-export all types from the unified options module
pub use crate::options::{
    BrokenLinkCallback, BrokenLinkReference, Extension, InputFormat, ListStyleType, Options,
    OutputFormat, ParseOptions, Plugins, RenderOptions, ResolvedReference, URLRewriter,
    WikiLinksMode, WrapOption,
};

// Re-export reader/writer options
pub use crate::options::{ReaderOptions, WriterOptions};
