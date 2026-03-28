//! Unified AST node value types for CommonMark documents
//!
//! This module is deprecated. Use `clmd::nodes` instead.
//!
//! # Deprecated
//!
//! This module is kept for backward compatibility. Please migrate to using
//! `clmd::nodes` which provides the same types with an improved API.
//!
//! # Migration Guide
//!
//! ```rust,ignore
//! // Old API (deprecated)
//! use clmd::node_value::{NodeValue, NodeHeading};
//!
//! // New API
//! use clmd::nodes::{NodeValue, NodeHeading};
//! ```

// Re-export all types from the new nodes module for backward compatibility
pub use crate::nodes::{
    can_contain_type, AlertType, Ast, AstNode, LineColumn, ListDelimType, ListType,
    Node as AstNodeRef, NodeAlert, NodeCode, NodeCodeBlock, NodeDescriptionItem,
    NodeFootnoteDefinition, NodeFootnoteReference, NodeHeading, NodeHtmlBlock, NodeLink,
    NodeList, NodeMath, NodeMultilineBlockQuote, NodeTable, NodeTaskItem, NodeValue,
    NodeWikiLink, SourcePos, TableAlignment,
};

// Re-export NodeValueListType alias for backward compatibility
/// Alias for ListType, kept for backward compatibility.
pub type NodeValueListType = ListType;
