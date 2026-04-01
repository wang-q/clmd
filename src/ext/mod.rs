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

/// Deprecated: Use `gfm::table` instead.
#[deprecated(since = "0.2.0", note = "Use `gfm::table` instead")]
pub mod tables {
    pub use crate::ext::gfm::table::*;
}

/// Deprecated: Use `gfm::strikethrough` instead.
#[deprecated(since = "0.2.0", note = "Use `gfm::strikethrough` instead")]
pub mod strikethrough {
    pub use crate::ext::gfm::strikethrough::*;
}

/// Deprecated: Use `gfm::tasklist` instead.
#[deprecated(since = "0.2.0", note = "Use `gfm::tasklist` instead")]
pub mod tasklist {
    pub use crate::ext::gfm::tasklist::*;
}

/// Deprecated: Use `gfm::autolink` instead.
#[deprecated(since = "0.2.0", note = "Use `gfm::autolink` instead")]
pub mod autolink {
    pub use crate::ext::gfm::autolink::*;
}

/// Deprecated: Use `gfm::tagfilter` instead.
#[deprecated(since = "0.2.0", note = "Use `gfm::tagfilter` instead")]
pub mod tagfilter {
    pub use crate::ext::gfm::tagfilter::*;
}

/// Deprecated: Use `syntax::footnote` instead.
#[deprecated(since = "0.2.0", note = "Use `syntax::footnote` instead")]
pub mod footnotes {
    pub use crate::ext::syntax::footnote::*;
}

/// Deprecated: Use `syntax::abbreviation` instead.
#[deprecated(since = "0.2.0", note = "Use `syntax::abbreviation` instead")]
pub mod abbreviation {
    pub use crate::ext::syntax::abbreviation::*;
}

/// Deprecated: Use `syntax::definition` instead.
#[deprecated(since = "0.2.0", note = "Use `syntax::definition` instead")]
pub mod definition {
    pub use crate::ext::syntax::definition::*;
}

/// Deprecated: Use `syntax::attribute` instead.
#[deprecated(since = "0.2.0", note = "Use `syntax::attribute` instead")]
pub mod attributes {
    pub use crate::ext::syntax::attribute::*;
}

/// Deprecated: Use `metadata::toc` instead.
#[deprecated(since = "0.2.0", note = "Use `metadata::toc` instead")]
pub mod toc {
    pub use crate::ext::metadata::toc::*;
}

/// Deprecated: Use `metadata::yaml_front_matter` instead.
#[deprecated(since = "0.2.0", note = "Use `metadata::yaml_front_matter` instead")]
pub mod yaml_front_matter {
    pub use crate::ext::metadata::yaml_front_matter::*;
}

/// Deprecated: Use `shortcode::parser` instead.
#[deprecated(since = "0.2.0", note = "Use `shortcode::parser` instead")]
pub mod shortcodes {
    pub use crate::ext::shortcode::parser::*;
}

/// Deprecated: Use `shortcode::data` instead.
#[deprecated(since = "0.2.0", note = "Use `shortcode::data` instead")]
pub mod shortcodes_data {
    pub use crate::ext::shortcode::data::*;
}
