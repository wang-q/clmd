//! Text processing utilities for clmd.
//!
//! This module provides utilities for text processing, including string manipulation,
//! HTML utilities, Unicode handling, and text transformations.

// String processing
pub mod char;
pub mod unicode;

// Additional text utilities
pub mod html_utils;
pub mod uri;

// Re-export commonly used types
pub use char::is_cjk_punctuation;
pub use unicode::{add_cjk_spacing, is_cjk};
pub use unicode::{is_double_width, width};
pub use uri::{normalize_uri, parse_data_uri};
