//! Typst renderer
//!
//! This module provides Typst output generation for CommonMark documents.
//! Typst is a modern markup-based typesetting system.
//!
//! # Example
//!
//! ```
//! use clmd::{parse_document, format_typst, Options};
//!
//! let options = Options::default();
//! let (arena, root) = parse_document("# Hello\n\nWorld", &options);
//! let mut typst = String::new();
//! format_typst(&arena, root, &options, &mut typst).unwrap();
//! assert!(typst.contains("= Hello"));
//! ```

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::{ListType, NodeList, NodeValue};
use crate::parser::options::{Options, Plugins};
use std::fmt;

/// Render an AST as Typst.
///
/// This is a convenience function that doesn't use plugins.
///
/// # Example
///
/// ```ignore
/// use clmd::{parse_document, format_typst, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("# Heading\n\nParagraph", &options);
/// let mut typst = String::new();
/// format_typst(&arena, root, &options, &mut typst).unwrap();
/// assert!(typst.contains("= Heading"));
/// ```ignore
pub fn render(arena: &NodeArena, root: NodeId, _options: u32) -> String {
    let mut renderer = TypstRenderer::new(arena);
    renderer.render(root)
}

/// Format an AST as Typst with plugins.
///
/// This function renders the AST to Typst format, supporting all CommonMark
/// elements and selected extensions.
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `_options` - Configuration options (currently unused)
/// * `output` - The output buffer to write to
/// * `_plugins` - Plugins for customizing rendering (currently unused)
///
/// # Returns
///
/// A `fmt::Result` indicating success or failure
///
/// # Example
///
/// ```ignore
/// use clmd::{parse_document, format_typst_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let (arena, root) = parse_document("# Hello\n\n**Bold** text", &options);
/// let mut typst = String::new();
/// format_typst_with_plugins(&arena, root, &options, &mut typst, &plugins).unwrap();
/// assert!(typst.contains("= Hello"));
/// assert!(typst.contains("*Bold*"));
/// ```ignore
pub fn format_document_with_plugins(
    arena: &NodeArena,
    root: NodeId,
    _options: &Options,
    output: &mut dyn fmt::Write,
    _plugins: &Plugins<'_>,
) -> fmt::Result {
    format_node_typst(arena, root, output, 0)
}

/// Escape special Typst characters in text.
fn escape_typst_text(text: &str) -> String {
    text.replace('*', "\\*")
        .replace('_', "\\_")
        .replace('#', "\\#")
        .replace('@', "\\@")
        .replace('$', "\\$")
        .replace('<', "\\<")
        .replace('>', "\\>")
}

fn format_node_typst(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut dyn fmt::Write,
    list_depth: usize,
) -> fmt::Result {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Document => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output, list_depth)?;
                child_opt = arena.get(child_id).next;
            }
        }
        NodeValue::Paragraph => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output, list_depth)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("\n\n")?;
        }
        NodeValue::Text(text) => {
            output.write_str(&escape_typst_text(text))?;
        }
        NodeValue::Heading(heading) => {
            let level = heading.level as usize;
            let prefix = match level {
                1 => "= ",
                2 => "== ",
                3 => "=== ",
                4 => "==== ",
                5 => "===== ",
                6 => "====== ",
                _ => "= ",
            };
            output.write_str(prefix)?;
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output, list_depth)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("\n\n")?;
        }
        NodeValue::Emph => {
            output.write_str("_")?;
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output, list_depth)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("_")?;
        }
        NodeValue::Strong => {
            output.write_str("*")?;
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output, list_depth)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("*")?;
        }
        NodeValue::Code(code) => {
            output.write_str("`")?;
            output.write_str(&code.literal)?;
            output.write_str("`")?;
        }
        NodeValue::CodeBlock(code_block) => {
            if code_block.fenced {
                output.write_str("```")?;
                if !code_block.info.is_empty() {
                    output.write_str(&code_block.info)?;
                }
                output.write_str("\n")?;
                output.write_str(&code_block.literal)?;
                output.write_str("\n```\n\n")?;
            } else {
                // Indented code block
                output.write_str("```\n")?;
                output.write_str(&code_block.literal)?;
                output.write_str("\n```\n\n")?;
            }
        }
        NodeValue::List(NodeList { list_type, .. }) => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_list_item(arena, child_id, output, list_depth, *list_type)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("\n")?;
        }
        NodeValue::Item(_) => {
            // Items are handled by format_list_item
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output, list_depth)?;
                child_opt = arena.get(child_id).next;
            }
        }
        NodeValue::BlockQuote => {
            output.write_str("#quote[\n")?;
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output, list_depth)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("]\n\n")?;
        }
        NodeValue::ThematicBreak => {
            output.write_str("#line(length: 100%)\n\n")?;
        }
        NodeValue::SoftBreak => {
            output.write_str(" ")?;
        }
        NodeValue::HardBreak => {
            output.write_str("\\n")?;
        }
        NodeValue::Link(link) => {
            output.write_str("#link(\"")?;
            output.write_str(&link.url)?;
            output.write_str("\")[")?;
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output, list_depth)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("]")?;
        }
        NodeValue::Image(link) => {
            output.write_str("#image(\"")?;
            output.write_str(&link.url)?;
            output.write_str("\")")?;
        }
        NodeValue::Strikethrough => {
            output.write_str("#strike[")?;
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output, list_depth)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("]")?;
        }
        NodeValue::HtmlBlock(html_block) => {
            // HTML blocks are output as raw text in Typst
            output.write_str(&html_block.literal)?;
            output.write_str("\n\n")?;
        }
        NodeValue::HtmlInline(html) => {
            output.write_str(html)?;
        }
        _ => {
            // For other nodes, just render children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output, list_depth)?;
                child_opt = arena.get(child_id).next;
            }
        }
    }

    Ok(())
}

fn format_list_item(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut dyn fmt::Write,
    depth: usize,
    list_type: ListType,
) -> fmt::Result {
    let node = arena.get(node_id);

    // Write indentation
    for _ in 0..depth {
        output.write_str("  ")?;
    }

    // Write list marker
    match list_type {
        ListType::Bullet => {
            output.write_str("- ")?;
        }
        ListType::Ordered => {
            output.write_str("+ ")?;
        }
    }

    // Render item content
    let mut child_opt = node.first_child;
    while let Some(child_id) = child_opt {
        format_node_typst(arena, child_id, output, depth + 1)?;
        child_opt = arena.get(child_id).next;
    }

    output.write_str("\n")?;
    Ok(())
}

/// Typst renderer state
struct TypstRenderer<'a> {
    arena: &'a NodeArena,
    output: String,
}

impl<'a> TypstRenderer<'a> {
    fn new(arena: &'a NodeArena) -> Self {
        TypstRenderer {
            arena,
            output: String::new(),
        }
    }

    fn render(&mut self, root: NodeId) -> String {
        self.render_node(root);
        self.output.clone()
    }

    fn render_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        match &node.value {
            NodeValue::Document => {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
            }
            NodeValue::Paragraph => {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push_str("\n\n");
            }
            NodeValue::Text(text) => {
                self.output.push_str(&escape_typst_text(text));
            }
            NodeValue::Heading(heading) => {
                let level = heading.level as usize;
                let prefix = match level {
                    1 => "= ",
                    2 => "== ",
                    3 => "=== ",
                    4 => "==== ",
                    5 => "===== ",
                    6 => "====== ",
                    _ => "= ",
                };
                self.output.push_str(prefix);
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push_str("\n\n");
            }
            NodeValue::Emph => {
                self.output.push('_');
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push('_');
            }
            NodeValue::Strong => {
                self.output.push('*');
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push('*');
            }
            NodeValue::Code(code) => {
                self.output.push('`');
                self.output.push_str(&code.literal);
                self.output.push('`');
            }
            _ => {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};
    use crate::nodes::{NodeCode, NodeCodeBlock, NodeHeading, NodeLink};

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Title")));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let typst = render(&arena, root, 0);
        assert!(typst.contains("= Title"));
    }

    #[test]
    fn test_render_heading_levels() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        for level in 1..=6 {
            let heading =
                arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
                    level,
                    setext: false,
                    closed: false,
                })));
            let text = arena.alloc(Node::with_value(NodeValue::make_text(format!(
                "H{}",
                level
            ))));
            TreeOps::append_child(&mut arena, root, heading);
            TreeOps::append_child(&mut arena, heading, text);
        }

        let typst = render(&arena, root, 0);
        assert!(typst.contains("= H1"));
        assert!(typst.contains("== H2"));
        assert!(typst.contains("=== H3"));
        assert!(typst.contains("==== H4"));
        assert!(typst.contains("===== H5"));
        assert!(typst.contains("====== H6"));
    }

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let typst = render(&arena, root, 0);
        assert!(typst.contains("Hello world"));
    }

    #[test]
    fn test_render_emph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("italic")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, emph, text);

        let typst = render(&arena, root, 0);
        assert!(typst.contains("_italic_"));
    }

    #[test]
    fn test_render_strong() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("bold")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, strong, text);

        let typst = render(&arena, root, 0);
        assert!(typst.contains("*bold*"));
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

        let typst = render(&arena, root, 0);
        assert!(typst.contains("`code`"));
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

        let options = Options::default();
        let plugins = Plugins::default();
        let mut typst = String::new();
        format_document_with_plugins(&arena, root, &options, &mut typst, &plugins)
            .unwrap();
        assert!(typst.contains("```rust"));
        assert!(typst.contains("fn main() {}"));
        assert!(typst.contains("```"));
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

        let options = Options::default();
        let plugins = Plugins::default();
        let mut typst = String::new();
        format_document_with_plugins(&arena, root, &options, &mut typst, &plugins)
            .unwrap();
        assert!(typst.contains("#quote["));
        assert!(typst.contains("Quote"));
        assert!(typst.contains("]"));
    }

    #[test]
    fn test_render_thematic_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let hr = arena.alloc(Node::with_value(NodeValue::ThematicBreak));

        TreeOps::append_child(&mut arena, root, hr);

        let options = Options::default();
        let plugins = Plugins::default();
        let mut typst = String::new();
        format_document_with_plugins(&arena, root, &options, &mut typst, &plugins)
            .unwrap();
        assert!(typst.contains("#line(length: 100%)"));
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

        let options = Options::default();
        let plugins = Plugins::default();
        let mut typst = String::new();
        format_document_with_plugins(&arena, root, &options, &mut typst, &plugins)
            .unwrap();
        assert!(typst.contains("#link(\"https://example.com\")["));
        assert!(typst.contains("link"));
        assert!(typst.contains("]"));
    }

    #[test]
    fn test_escape_typst_characters() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text(
            "Use *bold* and #heading",
        )));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let typst = render(&arena, root, 0);
        assert!(typst.contains("Use \\*bold\\* and \\#heading"));
    }

    #[test]
    fn test_format_with_plugins() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Test")));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let options = Options::default();
        let plugins = Plugins::default();
        let mut output = String::new();

        format_document_with_plugins(&arena, root, &options, &mut output, &plugins)
            .unwrap();

        assert!(output.contains("= Test"));
    }
}
