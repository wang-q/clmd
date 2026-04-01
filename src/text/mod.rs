//! Text processing utilities for clmd.
//!
//! This module provides utilities for text processing, including string manipulation,
//! HTML utilities, Unicode handling, and text transformations.

// String processing
pub mod asciify;
pub mod char;
pub mod emoji;

pub mod roff_char;
pub mod strings;
pub mod unicode_width;

// Additional text utilities
pub mod html_utils;
pub mod sequence;
pub mod uri;

// Re-export commonly used types
pub use html_utils::{escape_html, HtmlBuilder};
pub use sequence::BasedSequence;
pub use strings::decode_entities;
pub use uri::{url_decode, url_encode};
