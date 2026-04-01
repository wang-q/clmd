//! Format abstraction layer for document formats.
//!
//! This module provides types and utilities for working with different document formats,
//! including format detection, MIME type handling, and format-specific metadata.

// Re-export from the original format.rs
pub use crate::io::format_impl::*;

// Re-export format-specific modules
pub mod css;
pub mod csv;
pub mod mime;
pub mod slides;
pub mod tex;
pub mod xml;
