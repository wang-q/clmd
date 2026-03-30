//! CommonMark renderer

use crate::arena::{NodeArena, NodeId};
use crate::formatter::{CommonMarkNodeFormatter, Formatter, FormatterOptions};

/// Render a node tree as CommonMark
///
/// This function uses the new CommonMarkNodeFormatter via the Formatter framework,
/// which provides a flexible, node-based approach to rendering CommonMark output.
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
/// * `_options` - Rendering options (currently unused, kept for API compatibility)
/// * `wrap_width` - Maximum line width for wrapping (0 = no wrapping)
///
/// # Returns
///
/// The CommonMark output as a String
///
/// # Example
///
/// ```
/// use clmd::render::commonmark::render;
/// use clmd::arena::{NodeArena, Node, TreeOps};
/// use clmd::nodes::NodeValue;
///
/// let mut arena = NodeArena::new();
/// let root = arena.alloc(Node::with_value(NodeValue::Document));
/// let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
/// let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello world")));
///
/// TreeOps::append_child(&mut arena, root, para);
/// TreeOps::append_child(&mut arena, para, text);
///
/// let cm = render(&arena, root, 0, 0);
/// assert!(cm.contains("Hello world"));
/// ```
pub fn render(
    arena: &NodeArena,
    root: NodeId,
    _options: u32,
    wrap_width: usize,
) -> String {
    let options = FormatterOptions::new().with_right_margin(wrap_width);
    let mut formatter = Formatter::with_options(options);
    formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));
    formatter.render(arena, root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};
    use crate::nodes::{NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeValue};

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let cm = render(&arena, root, 0, 0);
        assert!(cm.contains("Hello world"));
    }

    #[test]
    fn test_render_emph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("emphasized")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, emph, text);

        let cm = render(&arena, root, 0, 0);
        assert!(cm.contains("*emphasized*"));
    }

    #[test]
    fn test_render_strong() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("strong")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, strong, text);

        let cm = render(&arena, root, 0, 0);
        assert!(cm.contains("**strong**"));
    }

    #[test]
    fn test_render_code() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = arena.alloc(Node::with_value(NodeValue::Code(Box::new(NodeCode {
            num_backticks: 1,
            literal: "code".to_string(),
        }))));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let cm = render(&arena, root, 0, 0);
        assert!(cm.contains("`code`"));
    }

    #[test]
    fn test_render_code_with_backticks() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = arena.alloc(Node::with_value(NodeValue::Code(Box::new(NodeCode {
            num_backticks: 1,
            literal: "code `with` backticks".to_string(),
        }))));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let cm = render(&arena, root, 0, 0);
        assert!(cm.contains("``code `with` backticks``"));
    }

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Heading")));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let cm = render(&arena, root, 0, 0);
        assert!(cm.contains("##"));
        assert!(cm.contains("Heading"));
    }

    #[test]
    fn test_render_link() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com".to_string(),
            title: "".to_string(),
        }))));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("link")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let cm = render(&arena, root, 0, 0);
        assert!(cm.contains("[link](https://example.com)"));
    }

    #[test]
    fn test_render_image() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let image =
            arena.alloc(Node::with_value(NodeValue::Image(Box::new(NodeLink {
                url: "image.png".to_string(),
                title: "".to_string(),
            }))));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("alt")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, image);
        TreeOps::append_child(&mut arena, image, text);

        let cm = render(&arena, root, 0, 0);
        assert!(cm.contains("![alt](image.png)"));
    }

    #[test]
    fn test_render_blockquote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let blockquote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Quote")));

        TreeOps::append_child(&mut arena, root, blockquote);
        TreeOps::append_child(&mut arena, blockquote, para);
        TreeOps::append_child(&mut arena, para, text);

        let cm = render(&arena, root, 0, 0);
        assert!(cm.contains("> Quote"));
    }

    #[test]
    fn test_render_code_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let code_block = arena.alloc(Node::with_value(NodeValue::CodeBlock(Box::new(
            NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            },
        ))));

        TreeOps::append_child(&mut arena, root, code_block);

        let cm = render(&arena, root, 0, 0);
        assert!(cm.contains("```rust"));
        assert!(cm.contains("fn main() {}"));
        assert!(cm.contains("```"));
    }

    #[test]
    fn test_render_thematic_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let hr = arena.alloc(Node::with_value(NodeValue::ThematicBreak));

        TreeOps::append_child(&mut arena, root, hr);

        let cm = render(&arena, root, 0, 0);
        assert!(cm.contains("***"));
    }
}
