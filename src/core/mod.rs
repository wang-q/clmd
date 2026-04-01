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
pub use error::{
    ClmdError, ClmdResult, LimitKind, ParseError, ParseResult, ParserLimits, Position,
    Range,
};
pub use nodes::NodeValue;
pub use shared::stringify;
pub use traverse::{
    AncestorIter, ArenaIteratorItem, ArenaNodeWalker, ArenaWalkerEvent, ChildIter,
    DescendantIter, EventIterator, IteratorEventType, NodeType, Query, Queryable,
    SiblingIter, Traverse, TraverseEvent, TraverseExt, WalkDirection, Walkable,
};
