//! Configuration for the parser and renderer. Extensions affect both.
//!
//! This module provides a comrak-style Options API for configuring
//! Markdown parsing and rendering behavior.
//!
//! # Example
//!
//! ```
//! use clmd::Options;
//!
//! let mut options = Options::default();
//! options.extension.table = true;
//! options.extension.strikethrough = true;
//! options.render.hardbreaks = true;
//! ```

pub use crate::parser::options::{
    BrokenLinkCallback, BrokenLinkReference, Extension, ListStyleType, Options, Parse,
    Plugins, Render, RenderPlugins, ResolvedReference, URLRewriter, WikiLinksMode,
};
