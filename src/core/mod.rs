//! Core module for clmd.
//!
//! This module provides the foundational types and utilities for the clmd Markdown parser.
//! It includes AST representations, error handling, memory management, and shared utilities.

// Core AST and node types
pub mod adapter;
pub mod arena;
pub mod ast;
pub mod nodes;
pub mod tree;
pub mod walk;

// Error handling
pub mod error;

// Iterator types
pub mod iterator;

// Shared utilities
pub mod shared;

// Re-export commonly used types
pub use adapter::{
    from_pandoc_ast, to_pandoc_ast, CodefenceRendererAdapter, HeadingAdapter,
    SyntaxHighlighterAdapter, UrlRewriter,
};
pub use arena::{
    AncestorIterator, ChildrenIterator, DescendantIterator, FollowingSiblingsIterator,
    NodeArena, NodeId, PrecedingSiblingsIterator, SiblingsIterator, INVALID_NODE_ID,
};
pub use ast::{Block, Document, Inline, Walkable};
pub use error::{
    ClmdError, ClmdResult, LimitKind, ParseError, ParseResult, ParserLimits, Position,
    Range,
};
pub use iterator::{ArenaNodeIterator, EventType};
pub use nodes::NodeValue;
pub use shared::stringify;
