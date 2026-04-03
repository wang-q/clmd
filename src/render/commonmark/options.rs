//! Formatter options configuration
//!
//! This module re-exports format options from the unified `crate::options` module
//! for backward compatibility.
//!
//! The format options provide comprehensive configuration for the Markdown formatter,
//! inspired by flexmark-java's FormatterOptions.

// Re-export all format types from the unified options module
pub use crate::options::format::{
    Alignment, BlockQuoteMarker, BulletMarker, CodeFenceMarker, DiscretionaryText,
    ElementPlacement, ElementPlacementSort, FormatFlags, FormatOptions, HeadingStyle,
    ListSpacing, NumberedMarker, TrailingMarker,
};

// Provide a type alias for backward compatibility
pub use crate::options::format::FormatOptions as FormatterOptions;
