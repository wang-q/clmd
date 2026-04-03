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
pub use ast::{Block, Document, Inline, Walkable as AstWalkable};
pub use error::{ClmdError, ClmdResult, LimitKind, ParserLimits, Position, Range};
pub use nodes::NodeValue;
pub use shared::stringify;
pub use traverse::{
    AncestorIter, ArenaIteratorItem, ArenaNodeWalker, ArenaWalkerEvent, ChildIter,
    DescendantIter, EventIterator, IteratorEventType, NodeType, Query, Queryable,
    SiblingIter, Traverse, TraverseContext, TraverseEvent, TraverseExt, WalkDirection,
    Walkable,
};

// Re-export monad types
pub use monad::{share_monad, ClmdIO, ClmdMonad, ClmdPure, SharedMonad, Verbosity};

// Re-export sandbox types
pub use sandbox::{SandboxMode, SandboxPolicy, SandboxedMonad};

// Re-export state types
pub use state::{CommonState, ExtensionData, TrackChanges, Translations};

// LogMessage and LogLevel are re-exported from error module
pub use crate::core::error::{LogLevel, LogMessage};
