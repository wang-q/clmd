//! AST Node types
//!
//! Provides concrete node type implementations using a trait-based system.
//! This module is designed to be similar to flexmark-java's AST architecture.
//!
//! # Architecture
//!
//! The node system is organized around several key traits:
//!
//! - `NodeType`: Core trait for all nodes, defines basic properties
//! - `BlockNode`: Marker trait for block-level nodes
//! - `InlineNode`: Marker trait for inline-level nodes
//! - `Visitor`: Trait for visiting nodes in a type-safe manner
//!
//! # Example
//!
//! ```
//! use clmd::ast_nodes::{Document, Paragraph, Text, NodeType, BlockNode, InlineNode};
//!
//! let doc = Document;
//! assert!(doc.is_block());
//!
//! let text = Text { literal: "Hello".to_string() };
//! assert!(text.is_inline());
//! ```

pub mod block;
pub mod extensions;
pub mod inline;
pub mod traits;

// Re-export all node types and traits from traits module
pub use traits::*;

// Re-export block nodes

// Re-export inline nodes

// Re-export extensions
pub use extensions::{factory, BlockNodeExt, InlineNodeExt, ListExt};
