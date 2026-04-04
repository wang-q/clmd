//! Typst writer.
//!
//! This module provides a writer for Typst format.

use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::core::nodes::{ListType, NodeList, NodeValue};
use crate::options::{Options, Plugins, WriterOptions};
use std::fmt;

/// Write a document as Typst.
pub fn write_typst(
    arena: &NodeArena,
    root: NodeId,
    _options: &WriterOptions,
) -> ClmdResult<String> {
    let mut renderer = TypstRenderer::new(arena);
    Ok(renderer.render(root))
}

/// Format an AST as Typst with plugins.
pub fn format_document_with_plugins(
    arena: &NodeArena,
    root: NodeId,
    _options: &Options,
    output: &mut dyn fmt::Write,
    _plugins: &Plugins<'_>,
) -> fmt::Result {
    let mut renderer = TypstRenderer::new(arena);
    let result = renderer.render(root);
    output.write_str(&result)
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
            NodeValue::CodeBlock(code_block) => {
                self.output.push_str("```");
                if !code_block.info.is_empty() {
                    self.output.push_str(&code_block.info);
                }
                self.output.push('\n');
                self.output.push_str(&code_block.literal);
                self.output.push_str("\n```\n\n");
            }
            NodeValue::List(NodeList { list_type, .. }) => {
                self.render_list(node_id, 0, *list_type);
            }
            NodeValue::Item(_) => {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
            }
            NodeValue::BlockQuote => {
                self.output.push_str("#quote[\n");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push_str("]\n\n");
            }
            NodeValue::ThematicBreak(..) => {
                self.output.push_str("#line(length: 100%)\n\n");
            }
            NodeValue::SoftBreak => {
                self.output.push(' ');
            }
            NodeValue::HardBreak => {
                self.output.push_str("\\n");
            }
            NodeValue::Link(link) => {
                self.output.push_str("#link(\"");
                self.output.push_str(&link.url);
                self.output.push_str("\")[");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push(']');
            }
            NodeValue::Image(link) => {
                self.output.push_str("#image(\"");
                self.output.push_str(&link.url);
                self.output.push_str("\")");
            }
            NodeValue::Strikethrough => {
                self.output.push_str("#strike[");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push(']');
            }
            NodeValue::HtmlBlock(html_block) => {
                self.output.push_str(&html_block.literal);
                self.output.push_str("\n\n");
            }
            NodeValue::HtmlInline(html) => {
                self.output.push_str(html);
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

    fn render_list(&mut self, node_id: NodeId, depth: usize, list_type: ListType) {
        let node = self.arena.get(node_id);

        let mut child_opt = node.first_child;
        while let Some(child_id) = child_opt {
            let child = self.arena.get(child_id);
            if matches!(child.value, NodeValue::Item(_)) {
                for _ in 0..depth {
                    self.output.push_str("  ");
                }
                match list_type {
                    ListType::Bullet => self.output.push_str("- "),
                    ListType::Ordered => self.output.push_str("+ "),
                }
                let mut item_child_opt = child.first_child;
                while let Some(item_child_id) = item_child_opt {
                    self.render_node(item_child_id);
                    item_child_opt = self.arena.get(item_child_id).next;
                }
                self.output.push('\n');
            }
            child_opt = child.next;
        }
        self.output.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{NodeCode, NodeCodeBlock, NodeHeading, NodeLink};

    #[test]
    fn test_write_typst_heading() {
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

        let options = WriterOptions::default();
        let typst = write_typst(&arena, root, &options).unwrap();
        assert!(typst.contains("= Title"));
    }

    #[test]
    fn test_write_typst_heading_levels() {
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

        let options = WriterOptions::default();
        let typst = write_typst(&arena, root, &options).unwrap();
        assert!(typst.contains("= H1"));
        assert!(typst.contains("== H2"));
        assert!(typst.contains("=== H3"));
        assert!(typst.contains("==== H4"));
        assert!(typst.contains("===== H5"));
        assert!(typst.contains("====== H6"));
    }

    #[test]
    fn test_write_typst_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let options = WriterOptions::default();
        let typst = write_typst(&arena, root, &options).unwrap();
        assert!(typst.contains("Hello world"));
    }

    #[test]
    fn test_write_typst_emph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("italic")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, emph, text);

        let options = WriterOptions::default();
        let typst = write_typst(&arena, root, &options).unwrap();
        assert!(typst.contains("_italic_"));
    }

    #[test]
    fn test_write_typst_strong() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("bold")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, strong, text);

        let options = WriterOptions::default();
        let typst = write_typst(&arena, root, &options).unwrap();
        assert!(typst.contains("*bold*"));
    }

    #[test]
    fn test_write_typst_code() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = arena.alloc(Node::with_value(NodeValue::Code(Box::new(NodeCode {
            num_backticks: 1,
            literal: "code".to_string(),
        }))));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let options = WriterOptions::default();
        let typst = write_typst(&arena, root, &options).unwrap();
        assert!(typst.contains("`code`"));
    }

    #[test]
    fn test_write_typst_code_block() {
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
    fn test_write_typst_blockquote() {
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
    fn test_write_typst_thematic_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let hr = arena.alloc(Node::with_value(NodeValue::ThematicBreak(
            crate::core::nodes::NodeThematicBreak::default(),
        )));

        TreeOps::append_child(&mut arena, root, hr);

        let options = Options::default();
        let plugins = Plugins::default();
        let mut typst = String::new();
        format_document_with_plugins(&arena, root, &options, &mut typst, &plugins)
            .unwrap();
        assert!(typst.contains("#line(length: 100%)"));
    }

    #[test]
    fn test_write_typst_link() {
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

        let options = WriterOptions::default();
        let typst = write_typst(&arena, root, &options).unwrap();
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
