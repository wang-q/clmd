//! Core module for clmd.
//!
//! This module provides the foundational types and utilities for the clmd Markdown parser.
//! It includes AST node representations, error handling, memory management, and shared utilities.

// Core AST and node types
pub mod adapter;
pub mod arena;
pub mod nodes;
pub mod traverse;

// Error handling
pub mod error;

// Sandbox mode for security
pub mod sandbox;

// Re-export commonly used types
pub use adapter::{
    CodefenceRendererAdapter, HeadingAdapter, SyntaxHighlighterAdapter, UrlRewriter,
};
pub use arena::{
    AncestorIterator, ChildrenIterator, DescendantIterator, FollowingSiblingsIterator,
    NodeArena, NodeId, PrecedingSiblingsIterator, SiblingsIterator, INVALID_NODE_ID,
};
pub use error::{ClmdError, ClmdResult, LimitKind, ParserLimits, Position};
pub use nodes::NodeValue;
pub use traverse::{Traverse, TraverseEvent, TraverseExt};

// Re-export sandbox types
pub use sandbox::{SandboxMode, SandboxPolicy};

// LogMessage and LogLevel are re-exported from error module
pub use crate::core::error::{LogLevel, LogMessage};
