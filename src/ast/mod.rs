//! AST module
//!
//! Provides the core AST node types and traversal utilities.
//! Design inspired by flexmark-java's AST architecture.

pub mod node;
pub mod visitor;

pub use node::{ChildrenIterator, DescendantIterator, Node, SourcePos};
pub use visitor::{CollectingVisitor, FindVisitor, NodeVisitor, TransformVisitor, Visitor};
