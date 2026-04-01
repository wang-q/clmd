//! IO module for document reading and writing
//!
//! This module provides a unified interface for reading documents from different
//! formats and writing to various output formats, inspired by Pandoc's Reader/Writer system.

/// Document readers for various input formats.
pub mod reader;

/// Document writers for various output formats.
pub mod writer;

/// Format abstraction layer for document formats.
pub mod format;

/// Format conversion from other formats to Markdown.
pub mod convert;

// Internal implementations
pub(crate) mod format_impl;
pub(crate) mod from_impl;

/// Deprecated: Use `reader` instead of `read`.
#[deprecated(since = "0.2.0", note = "Use `reader` instead of `read`")]
pub mod read {
    pub use crate::io::reader::*;
}

/// Deprecated: Use `writer` instead of `write`.
#[deprecated(since = "0.2.0", note = "Use `writer` instead of `write`")]
pub mod write {
    pub use crate::io::writer::*;
}

/// Deprecated: Use `convert` instead of `from`.
#[deprecated(since = "0.2.0", note = "Use `convert` instead of `from`")]
pub mod from {
    pub use crate::io::convert::*;
}

/// Deprecated: Use `format::css` instead.
#[deprecated(since = "0.2.0", note = "Use `format::css` instead")]
pub mod format_css {
    pub use crate::io::format::css::*;
}

/// Deprecated: Use `format::csv` instead.
#[deprecated(since = "0.2.0", note = "Use `format::csv` instead")]
pub mod format_csv {
    pub use crate::io::format::csv::*;
}

/// Deprecated: Use `format::mime` instead.
#[deprecated(since = "0.2.0", note = "Use `format::mime` instead")]
pub mod format_mime {
    pub use crate::io::format::mime::*;
}

/// Deprecated: Use `format::slides` instead.
#[deprecated(since = "0.2.0", note = "Use `format::slides` instead")]
pub mod format_slides {
    pub use crate::io::format::slides::*;
}

/// Deprecated: Use `format::tex` instead.
#[deprecated(since = "0.2.0", note = "Use `format::tex` instead")]
pub mod format_tex {
    pub use crate::io::format::tex::*;
}

/// Deprecated: Use `format::xml` instead.
#[deprecated(since = "0.2.0", note = "Use `format::xml` instead")]
pub mod format_xml {
    pub use crate::io::format::xml::*;
}
