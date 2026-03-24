//! AST module
//!
//! Provides the core AST node types and traversal utilities.
//! Design inspired by flexmark-java's AST architecture.

pub mod node;
pub mod render_compat;
pub mod util;
pub mod visitor;

pub use node::{ChildrenIterator, DescendantIterator, Node, SourcePos};
pub use render_compat::{RenderAdapter, RenderExt, to_old_node};
pub use util::{
    collect_nodes, find_node, get_siblings, get_text_content, is_ancestor, make_source_pos,
    merge_source_pos, node_depth, node_path_depths, replace_node,
};
pub use visitor::{CollectingVisitor, FindVisitor, NodeVisitor, TransformVisitor, Visitor};
