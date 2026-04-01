//! Core module for clmd.
//!
//! This module provides the foundational types and utilities for the clmd Markdown parser.
//! It includes AST representations, error handling, memory management, and shared utilities.

// Core AST and node types
pub mod adapters;
pub mod arena;
pub mod ast;
pub mod nodes;
pub mod tree;
pub mod walk;

// Error handling
pub mod error;

// State and monad types
pub mod iterator;
pub mod monad;
pub mod sandbox;
pub mod state;

// Shared utilities
pub mod shared;

// Re-export commonly used types
pub use adapters::{from_pandoc_ast, to_pandoc_ast, SyntaxHighlighterAdapter, HeadingAdapter, CodefenceRendererAdapter, UrlRewriter};
pub use arena::{NodeArena, NodeId};
pub use ast::{Block, Document, Inline, Walkable};
pub use error::{ClmdError, ClmdResult, ParseError, ParseResult, Position, Range};
pub use nodes::NodeValue;
pub use shared::stringify;
