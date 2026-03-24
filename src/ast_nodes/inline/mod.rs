//! Inline-level AST nodes
//!
//! Inline nodes are the content elements within block-level nodes.
//! They can contain other inline nodes but not block nodes.

// Re-export all inline node types from traits module
pub use crate::ast_nodes::traits::{
    Code, Emph, HtmlInline, Image, InlineNode, LineBreak, Link, SoftBreak, Strong, Text,
};
