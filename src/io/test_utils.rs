//! Test utilities for IO module.
//!
//! This module provides common test helpers for creating AST structures
//! and testing writers and readers.

use crate::core::arena::{Node, NodeArena, NodeId, TreeOps};
use crate::core::nodes::{
    ListDelimType, NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeList,
    NodeThematicBreak, NodeValue,
};

/// Create a new test arena with a document root node.
///
/// # Returns
///
/// A tuple of (arena, root_node_id)
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::create_test_arena;
///
/// let (arena, root) = create_test_arena();
/// ```
pub fn create_test_arena() -> (NodeArena, NodeId) {
    let mut arena = NodeArena::new();
    let root = arena.alloc(Node::with_value(NodeValue::Document));
    (arena, root)
}

/// Create a heading node and append it to the parent.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `parent` - The parent node ID
/// * `level` - The heading level (1-6)
/// * `text` - The heading text content
///
/// # Returns
///
/// The ID of the created heading node
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::{create_test_arena, create_heading};
///
/// let (mut arena, root) = create_test_arena();
/// let heading = create_heading(&mut arena, root, 1, "Title");
/// ```
pub fn create_heading(
    arena: &mut NodeArena,
    parent: NodeId,
    level: u8,
    text: &str,
) -> NodeId {
    let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
        level,
        setext: false,
        closed: false,
    })));
    let text_node = arena.alloc(Node::with_value(NodeValue::Text(text.into())));
    TreeOps::append_child(arena, heading, text_node);
    TreeOps::append_child(arena, parent, heading);
    heading
}

/// Create a paragraph node with text content.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `parent` - The parent node ID
/// * `text` - The paragraph text content
///
/// # Returns
///
/// The ID of the created paragraph node
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::{create_test_arena, create_paragraph};
///
/// let (mut arena, root) = create_test_arena();
/// let para = create_paragraph(&mut arena, root, "Hello world");
/// ```
pub fn create_paragraph(arena: &mut NodeArena, parent: NodeId, text: &str) -> NodeId {
    let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
    let text_node = arena.alloc(Node::with_value(NodeValue::Text(text.into())));
    TreeOps::append_child(arena, para, text_node);
    TreeOps::append_child(arena, parent, para);
    para
}

/// Create a text node.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `parent` - The parent node ID
/// * `text` - The text content
///
/// # Returns
///
/// The ID of the created text node
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::{create_test_arena, create_text};
///
/// let (mut arena, root) = create_test_arena();
/// let text = create_text(&mut arena, root, "Hello");
/// ```
pub fn create_text(arena: &mut NodeArena, parent: NodeId, text: &str) -> NodeId {
    let text_node = arena.alloc(Node::with_value(NodeValue::Text(text.into())));
    TreeOps::append_child(arena, parent, text_node);
    text_node
}

/// Create a code block node.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `parent` - The parent node ID
/// * `literal` - The code content
/// * `info` - Optional language info (e.g., "rust")
///
/// # Returns
///
/// The ID of the created code block node
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::{create_test_arena, create_code_block};
///
/// let (mut arena, root) = create_test_arena();
/// let code = create_code_block(&mut arena, root, "fn main() {}", Some("rust"));
/// ```
pub fn create_code_block(
    arena: &mut NodeArena,
    parent: NodeId,
    literal: &str,
    info: Option<&str>,
) -> NodeId {
    let code_block = arena.alloc(Node::with_value(NodeValue::CodeBlock(Box::new(
        NodeCodeBlock {
            literal: literal.into(),
            info: info.unwrap_or("").into(),
            fenced: true,
            fence_char: b'`',
            fence_length: 3,
            fence_offset: 0,
            closed: true,
        },
    ))));
    TreeOps::append_child(arena, parent, code_block);
    code_block
}

/// Create an inline code node.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `parent` - The parent node ID
/// * `literal` - The code content
///
/// # Returns
///
/// The ID of the created code node
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::{create_test_arena, create_code};
///
/// let (mut arena, root) = create_test_arena();
/// let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
/// let code = create_code(&mut arena, para, "println!()");
/// TreeOps::append_child(&mut arena, root, para);
/// ```
pub fn create_code(arena: &mut NodeArena, parent: NodeId, literal: &str) -> NodeId {
    let code = arena.alloc(Node::with_value(NodeValue::Code(Box::new(NodeCode {
        literal: literal.into(),
        num_backticks: 1,
    }))));
    TreeOps::append_child(arena, parent, code);
    code
}

/// Create a list node.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `parent` - The parent node ID
/// * `ordered` - Whether the list is ordered
///
/// # Returns
///
/// The ID of the created list node
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::{create_test_arena, create_list, create_list_item};
///
/// let (mut arena, root) = create_test_arena();
/// let list = create_list(&mut arena, root, false);
/// create_list_item(&mut arena, list, "Item 1");
/// create_list_item(&mut arena, list, "Item 2");
/// ```
pub fn create_list(arena: &mut NodeArena, parent: NodeId, ordered: bool) -> NodeId {
    let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
        list_type: if ordered {
            crate::core::nodes::ListType::Ordered
        } else {
            crate::core::nodes::ListType::Bullet
        },
        marker_offset: 0,
        padding: 0,
        start: 1,
        tight: true,
        delimiter: ListDelimType::Period,
        bullet_char: b'-',
        is_task_list: false,
    })));
    TreeOps::append_child(arena, parent, list);
    list
}

/// Create a list item node with text content.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `parent` - The parent list node ID
/// * `text` - The item text content
///
/// # Returns
///
/// The ID of the created list item node
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::{create_test_arena, create_list, create_list_item};
///
/// let (mut arena, root) = create_test_arena();
/// let list = create_list(&mut arena, root, false);
/// create_list_item(&mut arena, list, "Item 1");
/// ```
pub fn create_list_item(arena: &mut NodeArena, parent: NodeId, text: &str) -> NodeId {
    let item = arena.alloc(Node::with_value(NodeValue::Item(
        crate::core::nodes::NodeList::default(),
    )));
    let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
    let text_node = arena.alloc(Node::with_value(NodeValue::Text(text.into())));
    TreeOps::append_child(arena, para, text_node);
    TreeOps::append_child(arena, item, para);
    TreeOps::append_child(arena, parent, item);
    item
}

/// Create a link node.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `parent` - The parent node ID
/// * `url` - The link URL
/// * `title` - The link title
/// * `text` - The link text content
///
/// # Returns
///
/// The ID of the created link node
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::{create_test_arena, create_link};
///
/// let (mut arena, root) = create_test_arena();
/// let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
/// let link = create_link(&mut arena, para, "https://example.com", "Example", "Click here");
/// TreeOps::append_child(&mut arena, root, para);
/// ```
pub fn create_link(
    arena: &mut NodeArena,
    parent: NodeId,
    url: &str,
    title: &str,
    text: &str,
) -> NodeId {
    let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
        url: url.into(),
        title: title.into(),
    }))));
    let text_node = arena.alloc(Node::with_value(NodeValue::Text(text.into())));
    TreeOps::append_child(arena, link, text_node);
    TreeOps::append_child(arena, parent, link);
    link
}

/// Create an emphasis node.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `parent` - The parent node ID
/// * `text` - The emphasized text content
///
/// # Returns
///
/// The ID of the created emphasis node
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::{create_test_arena, create_emphasis};
///
/// let (mut arena, root) = create_test_arena();
/// let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
/// let emph = create_emphasis(&mut arena, para, "emphasized");
/// TreeOps::append_child(&mut arena, root, para);
/// ```
pub fn create_emphasis(arena: &mut NodeArena, parent: NodeId, text: &str) -> NodeId {
    let emph = arena.alloc(Node::with_value(NodeValue::Emph));
    let text_node = arena.alloc(Node::with_value(NodeValue::Text(text.into())));
    TreeOps::append_child(arena, emph, text_node);
    TreeOps::append_child(arena, parent, emph);
    emph
}

/// Create a strong (bold) node.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `parent` - The parent node ID
/// * `text` - The bold text content
///
/// # Returns
///
/// The ID of the created strong node
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::{create_test_arena, create_strong};
///
/// let (mut arena, root) = create_test_arena();
/// let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
/// let strong = create_strong(&mut arena, para, "bold");
/// TreeOps::append_child(&mut arena, root, para);
/// ```
pub fn create_strong(arena: &mut NodeArena, parent: NodeId, text: &str) -> NodeId {
    let strong = arena.alloc(Node::with_value(NodeValue::Strong));
    let text_node = arena.alloc(Node::with_value(NodeValue::Text(text.into())));
    TreeOps::append_child(arena, strong, text_node);
    TreeOps::append_child(arena, parent, strong);
    strong
}

/// Create a thematic break (horizontal rule) node.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `parent` - The parent node ID
///
/// # Returns
///
/// The ID of the created thematic break node
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::{create_test_arena, create_thematic_break};
///
/// let (mut arena, root) = create_test_arena();
/// create_thematic_break(&mut arena, root);
/// ```
pub fn create_thematic_break(arena: &mut NodeArena, parent: NodeId) -> NodeId {
    let hr = arena.alloc(Node::with_value(NodeValue::ThematicBreak(
        NodeThematicBreak { marker: '-' },
    )));
    TreeOps::append_child(arena, parent, hr);
    hr
}

/// Create a blockquote node.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `parent` - The parent node ID
///
/// # Returns
///
/// The ID of the created blockquote node
///
/// # Example
///
/// ```ignore
/// use clmd::io::test_utils::{create_test_arena, create_blockquote, create_paragraph};
///
/// let (mut arena, root) = create_test_arena();
/// let quote = create_blockquote(&mut arena, root);
/// create_paragraph(&mut arena, quote, "Quoted text");
/// ```
pub fn create_blockquote(arena: &mut NodeArena, parent: NodeId) -> NodeId {
    let quote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
    TreeOps::append_child(arena, parent, quote);
    quote
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_arena() {
        let (arena, root) = create_test_arena();
        assert_eq!(arena.len(), 1);
        let node = arena.get(root);
        assert!(matches!(node.value, NodeValue::Document));
    }

    #[test]
    fn test_create_heading() {
        let (mut arena, root) = create_test_arena();
        let heading = create_heading(&mut arena, root, 1, "Title");

        let node = arena.get(heading);
        assert!(matches!(node.value, NodeValue::Heading(_)));
        assert_eq!(arena.len(), 3); // Document + Heading + Text
    }

    #[test]
    fn test_create_paragraph() {
        let (mut arena, root) = create_test_arena();
        let para = create_paragraph(&mut arena, root, "Hello world");

        let node = arena.get(para);
        assert!(matches!(node.value, NodeValue::Paragraph));
    }

    #[test]
    fn test_create_code_block() {
        let (mut arena, root) = create_test_arena();
        let code = create_code_block(&mut arena, root, "fn main() {}", Some("rust"));

        let node = arena.get(code);
        assert!(matches!(node.value, NodeValue::CodeBlock(_)));
    }

    #[test]
    fn test_create_list() {
        let (mut arena, root) = create_test_arena();
        let list = create_list(&mut arena, root, false);
        create_list_item(&mut arena, list, "Item 1");
        create_list_item(&mut arena, list, "Item 2");

        let node = arena.get(list);
        assert!(matches!(node.value, NodeValue::List(_)));
    }

    #[test]
    fn test_create_link() {
        let (mut arena, root) = create_test_arena();
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = create_link(
            &mut arena,
            para,
            "https://example.com",
            "Example",
            "Click here",
        );
        TreeOps::append_child(&mut arena, root, para);

        let node = arena.get(link);
        assert!(matches!(node.value, NodeValue::Link(_)));
    }

    #[test]
    fn test_create_emphasis_and_strong() {
        let (mut arena, root) = create_test_arena();
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        create_emphasis(&mut arena, para, "emphasized");
        create_strong(&mut arena, para, "bold");
        TreeOps::append_child(&mut arena, root, para);

        assert_eq!(arena.len(), 6); // Document + Paragraph + Emph + Text + Strong + Text
    }

    #[test]
    fn test_create_blockquote() {
        let (mut arena, root) = create_test_arena();
        let quote = create_blockquote(&mut arena, root);
        create_paragraph(&mut arena, quote, "Quoted text");

        let node = arena.get(quote);
        assert!(matches!(node.value, NodeValue::BlockQuote));
    }
}
