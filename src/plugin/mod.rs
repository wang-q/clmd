//! Plugin system for extending Markdown rendering
//!
//! This module provides a plugin architecture that allows users to customize
//! various aspects of Markdown rendering, such as syntax highlighting,
//! heading rendering, and code block handling.
//!
//! # Example
//!
//! ```ignore
//! use clmd::plugins::{Plugins, SyntaxHighlighterAdapter};
//! use clmd::parse::options::Options;
//!
//! // Create a custom syntax highlighter
//! struct MyHighlighter;
//!
//! impl SyntaxHighlighterAdapter for MyHighlighter {
//!     fn highlight(&self, code: &str, language: Option<&str>) -> String {
//!         format!("<pre><code class=\"lang-{lang}\">{code}</code></pre>",
//!             lang = language.unwrap_or("text"),
//!             code = code)
//!     }
//! }
//!
//! // Configure plugins
//! let mut plugins = Plugins::new();
//! plugins.render.set_syntax_highlighter(&MyHighlighter);
//! ```

// Re-export from parse::options for unified API
pub use crate::parse::options::{Plugins, RenderPlugins};

// Re-export adapter traits for convenience
pub use crate::core::adapter::{
    CodefenceRendererAdapter, HeadingAdapter, SyntaxHighlighterAdapter, UrlRewriter,
};

mod owned;

pub use owned::{DefaultSyntaxHighlighter, OwnedPlugins};

#[cfg(feature = "syntect")]
pub mod syntect;
