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

// Re-export for convenience
pub use read as reader;
pub use write as writer;
