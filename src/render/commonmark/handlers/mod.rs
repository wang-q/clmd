//! Node handlers for CommonMark formatting
//!
//! This module contains the handlers for different node types used in
//! CommonMark formatting. Each handler is responsible for formatting
//! a specific type of node.
//!
//! # Submodules
//!
//! - `block`: Block-level element rendering functions (code blocks, HTML blocks)
//! - `container`: Container element rendering functions (paragraphs, headings, blockquotes)
//! - `inline`: Inline element rendering functions (links, images)
//! - `list`: List element rendering and utility functions
//! - `table`: Table element rendering functions
//! - `registration`: Handler registration organized by functional domain

pub mod block;
pub mod container;
pub mod inline;
pub mod list;
pub mod registration;
pub mod table;

pub use block::*;
pub use container::*;
pub use inline::*;
pub use list::*;
pub use table::*;
