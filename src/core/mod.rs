//! Core module for clmd.
//!
//! This module provides the foundational types and utilities for the clmd Markdown parser.
//! It includes AST node representations, error handling, memory management, and shared utilities.

// Core AST and node types
pub mod adapter;
pub mod arena;
pub mod nodes;

// Error handling
pub mod error;

// Re-export commonly used types
pub use adapter::{
    CodefenceRendererAdapter, HeadingAdapter, SyntaxHighlighterAdapter, UrlRewriter,
};
pub use arena::{
    AncestorIter, AncestorIterator, ChildIter, ChildrenIterator, DescendantIterator,
    FollowingSiblingsIterator, NodeArena, NodeId, PrecedingSiblingsIterator,
    SiblingsIterator, TraverseExt, INVALID_NODE_ID,
};
pub use error::{ClmdError, ClmdResult, LimitKind, ParserLimits, Position};
pub use nodes::NodeValue;

// LogMessage and LogLevel are re-exported from error module
pub use crate::core::error::{LogLevel, LogMessage};
