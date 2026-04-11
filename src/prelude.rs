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
    format_commonmark, format_html, format_xml, markdown_to_commonmark,
    markdown_to_commonmark_xml, markdown_to_html, parse_document, Arena, NodeId,
    Options,
};

// Re-export from options module
pub use crate::options::{
    BrokenLinkCallback, BrokenLinkReference, ListStyleType, OutputFormat, ParseOptions,
    RenderOptions, ResolvedReference, URLRewriter, WrapOption, WriterOptions,
};

// Re-export extension types
pub use crate::ext::{ExtensionFlags, ExtensionKind};
