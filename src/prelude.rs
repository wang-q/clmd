//! Prelude module for convenient imports
//!
//! This module re-exports the most commonly used types and functions from clmd.
//! Using this module can simplify your imports:
//!
//! ```
//! use clmd::prelude::*;
//!
//! let options = Options::default();
//! let html = markdown_to_html("Hello **world**!", &options);
//! ```

pub use crate::core::nodes::NodeValue;
pub use crate::{
    format_commonmark, format_commonmark_with_plugins, format_html,
    format_html_with_plugins, format_xml, format_xml_with_plugins,
    markdown_to_commonmark, markdown_to_commonmark_with_plugins,
    markdown_to_commonmark_xml, markdown_to_commonmark_xml_with_plugins,
    markdown_to_html, markdown_to_html_with_plugins, parse_document, Arena, NodeId,
    Options, Plugins,
};

// Re-export from options module
pub use crate::options::{
    BrokenLinkCallback, BrokenLinkReference, Extension, InputFormat, ListStyleType,
    OutputFormat, ParseOptions, ReaderOptions, RenderOptions, RenderPlugins,
    ResolvedReference, URLRewriter, WikiLinksMode, WrapOption, WriterOptions,
};

// Re-export extension types
pub use crate::ext::{ExtensionFlags, ExtensionKind};
