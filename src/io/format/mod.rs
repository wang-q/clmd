//! Format abstraction layer for document formats.
//!
//! This module provides types and utilities for working with different document formats,
//! including format detection, MIME type handling, and format-specific metadata.
//!
//! ## Module Organization
//!
//! This module provides **general-purpose utilities** for format handling:
//!
//! - **`mime`**: MIME type utilities for media resources (images, fonts, etc.)
//! - **`slides`**: Slide data structures for presentation formats
//! - **`css`**: CSS parsing utilities
//! - **`csv`**: CSV/TSV parsing utilities
//! - **`tex`**: TeX tokenization utilities
//! - **`xml`**: XML building utilities
//!
//! ## Format vs Writer Distinction
//!
//! - **`io::format`**: Provides format types, detection, and utilities
//! - **`io::writer`**: Provides actual document rendering to specific formats
//!
//! For example:
//! - `format::xml` provides `XmlBuilder` for building XML structures
//! - `writer::xml` provides `XmlWriter` for rendering AST to CommonMark XML format

// Re-export format-specific modules
pub mod css;
pub mod csv;
pub mod mime;
pub mod slides;
pub mod tex;
pub mod xml;
