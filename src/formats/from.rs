//! Format converters for importing content to Markdown
//!
//! This module provides conversion from various formats back to Markdown.
//! These are reverse operations of the `render` module.
//!
//! Supported formats:
//! - **HTML**: Convert HTML content to Markdown
//!
//! # Example
//!
//! ```
//! use clmd::from::html::convert;
//!
//! let html = "<h1>Title</h1><p>Paragraph with <strong>bold</strong> text.</p>";
//! let markdown = convert(html);
//! assert!(markdown.contains("# Title"));
//! assert!(markdown.contains("**bold**"));
//! ```

pub mod html;

// Re-export commonly used functions
pub use html::convert as html_to_markdown;
pub use html::convert_with_rules as html_to_markdown_with_rules;
pub use html::is_html;
