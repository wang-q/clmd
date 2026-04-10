//! Text processing utilities for clmd.
//!
//! This module provides utilities for text processing, including string manipulation,
//! HTML utilities, Unicode handling, and text transformations.

// String processing
pub mod asciify;
pub mod char;
pub mod cjk_spacing;
pub mod strings;
pub mod unicode_width;

// Additional text utilities
pub mod html_utils;
pub mod uri;

// Re-export commonly used types
pub use asciify::{asciify, slugify, Transliterator};
pub use char::{count_cjk, has_cjk, is_cjk, is_cjk_punctuation, is_fullwidth};
pub use cjk_spacing::add_cjk_spacing;
pub use html_utils::{escape_html, HtmlBuilder};
pub use strings::decode_entities;
pub use unicode_width::{is_double_width, width};
pub use uri::{normalize_uri, parse_data_uri};
