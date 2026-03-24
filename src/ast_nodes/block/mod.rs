//! Block-level AST nodes
//!
//! Block nodes are the top-level structural elements of a Markdown document.
//! They can contain other block nodes or inline nodes.

// Re-export all block node types from traits module
pub use crate::ast_nodes::traits::{
    BlockNode, BlockQuote, CodeBlock, DelimType, Document, Heading, HtmlBlock, Item,
    List, ListType, Paragraph, ThematicBreak,
};
