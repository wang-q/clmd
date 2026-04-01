//! Markdown extensions
//!
//! This module contains various extensions to the CommonMark specification,
//! including GitHub Flavored Markdown (GFM) features and other commonly used extensions.

/// Extension flags management
///
/// This module provides the `ExtensionFlags` bitflags type and `ExtensionKind` enum
/// for managing Markdown extensions.
pub mod flags;

// Re-export commonly used extension types for convenience
pub use flags::{ExtensionFlags, ExtensionKind};

/// GitHub Flavored Markdown (GFM) extensions.
///
/// This module includes GFM-specific extensions:
/// - Tables
/// - Strikethrough
/// - Task lists
/// - Autolinks
/// - Tag filtering
pub mod gfm;

/// Syntax extensions.
///
/// This module includes syntax-related extensions:
/// - Footnotes
/// - Abbreviations
/// - Definition lists
/// - Attributes
pub mod syntax;

/// Metadata extensions.
///
/// This module includes metadata-related extensions:
/// - YAML front matter
/// - Table of contents (TOC)
pub mod metadata;

/// Shortcode extensions.
///
/// This module includes shortcode support for emoji and other content.
pub mod shortcode;
