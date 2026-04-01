//! IO module for document reading and writing
//!
//! This module provides a unified interface for reading documents from different
//! formats and writing to various output formats, inspired by Pandoc's Reader/Writer system.

/// Document readers for various input formats.
pub mod read;

/// Document writers for various output formats.
pub mod write;

/// Format abstraction layer for document formats.
pub mod format;

/// Format conversion from other formats to Markdown.
pub mod from;

// Format submodules
pub mod format_css;
pub mod format_csv;
pub mod format_mime;
pub mod format_slides;
pub mod format_tex;
pub mod format_xml;

// Re-export for convenience
pub use read as reader;
pub use write as writer;
