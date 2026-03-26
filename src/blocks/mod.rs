//! Block-level parsing for CommonMark documents
//!
//! This module implements the block parsing algorithm based on the CommonMark spec.
//! It processes input line by line, building the AST structure using Arena allocation.
//!
//! # Overview
//!
//! Block parsing is the first phase of Markdown processing. It identifies and parses
//! block-level elements:
//!
//! - **Leaf blocks**: Paragraphs, headings, code blocks, HTML blocks
//! - **Container blocks**: Blockquotes, lists, list items
//! - **Document metadata**: Link reference definitions
//!
//! The parser uses a line-by-line approach, maintaining a stack of open blocks
//! and matching each line against potential containers.
//!
//! # Example
//!
//! ```
//! use clmd::{parse_document, render_html, options};
//!
//! let (arena, doc) = parse_document("# Heading\n\nParagraph", options::DEFAULT);
//! let html = render_html(&arena, doc, options::DEFAULT);
//! assert!(html.contains("<h1>Heading</h1>"));
//! assert!(html.contains("<p>Paragraph</p>"));
//! ```

mod info;
pub use info::BlockInfo;

mod parser;
pub use parser::BlockParser;

mod block_info;
mod block_starts;
mod continuation;
mod finalization;
mod helpers;

#[cfg(test)]
mod tests;
