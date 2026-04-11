//! Core module for clmd.
//!
//! This module provides the foundational types and utilities for the clmd Markdown parser.
//! It includes AST representations, error handling, memory management, and shared utilities.

// Core AST and node types
pub mod adapter;
pub mod arena;
pub mod ast;
pub mod nodes;
pub mod traverse;

// Error handling
pub mod error;

// Monad abstraction for IO operations
pub mod monad;

// Sandbox mode for security
pub mod sandbox;

// State management
pub mod state;

// Shared utilities
pub mod shared;

// Re-export commonly used types
pub use adapter::{
    CodefenceRendererAdapter, HeadingAdapter, SyntaxHighlighterAdapter, UrlRewriter,
};
pub use arena::{
    AncestorIterator, ChildrenIterator, DescendantIterator, FollowingSiblingsIterator,
    NodeArena, NodeId, PrecedingSiblingsIterator, SiblingsIterator, INVALID_NODE_ID,
};
pub use ast::{Block, Document, Inline};
pub use error::{ClmdError, ClmdResult, LimitKind, ParserLimits, Position};
pub use nodes::NodeValue;
pub use traverse::{
    AncestorIter, ChildIter, DescendantIter, EventIterator, SiblingIter, Traverse,
    TraverseEvent, TraverseExt,
};

// Re-export monad types
pub use monad::{ClmdIO, ClmdMonad, ClmdPure, Verbosity};

// Re-export sandbox types
pub use sandbox::{SandboxMode, SandboxPolicy};

// Re-export state types
pub use state::CommonState;

// LogMessage and LogLevel are re-exported from error module
pub use crate::core::error::{LogLevel, LogMessage};
