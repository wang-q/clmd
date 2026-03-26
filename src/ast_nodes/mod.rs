//! AST Node types (DEPRECATED)
//!
//! **DEPRECATED**: This module is deprecated and will be removed in a future version.
//! Please use the `node` module instead, which provides the actual AST node types
//! used by the parser.
//!
//! This module was an experimental trait-based node system that was never
//! integrated into the main parser. The `node` module contains the real
//! `NodeType` enum and `NodeData` structures used throughout the codebase.
//!
//! # Migration
//!
//! Instead of using types from this module, use:
//! - `crate::node::NodeType` - The actual node type enum
//! - `crate::node::NodeData` - Node data variants
//! - `crate::Node` - The node structure (re-exported at crate root)
//! - `crate::arena::NodeId` - Node identifiers
//!
//! # Example
//!
//! ```
//! use clmd::node::{NodeType, NodeData};
//! use clmd::{Node, NodeArena, NodeId};
//!
//! let mut arena = NodeArena::new();
//! let doc_id = arena.alloc(Node::new(NodeType::Document));
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
