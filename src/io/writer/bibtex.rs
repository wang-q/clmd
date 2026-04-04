//! BibTeX document writer.
//!
//! This module provides a writer for BibTeX format.
//!
//! # Example
//!
//! ```ignore
//! use clmd::writers::BibTeXWriter;
//! use clmd::options::WriterOptions;
//! use clmd::context::PureContext;
//!
//! let writer = BibTeXWriter;
//! let ctx = PureContext::new();
//! let output = writer.write(&arena, root, &ctx, &WriterOptions::default()).unwrap();
//! ```

use crate::context::ClmdContext;
use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::core::nodes::NodeValue;
use crate::io::writer::shared::collect_text;
use crate::io::writer::Writer;
use crate::options::{OutputFormat, WriterOptions};

/// BibTeX document writer.
#[derive(Debug, Clone, Copy)]
pub struct BibTeXWriter;

impl Writer for BibTeXWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        _options: &WriterOptions,
    ) -> ClmdResult<String> {
        let mut output = String::new();
        render_node(arena, root, &mut output)?;
        Ok(output)
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Bibtex
    }

    fn extensions(&self) -> &[&'static str] {
        &["bib", "bibtex"]
    }

    fn mime_type(&self) -> &'static str {
        "text/x-bibtex"
    }
}

/// Render a node and its children to BibTeX.
fn render_node(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut String,
) -> ClmdResult<()> {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Document => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }

        NodeValue::Heading(_heading) => {
            // Get the heading text from children
            let mut title = String::new();
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                title.push_str(&collect_text(arena, child_id));
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            if !title.is_empty() {
                // Create a comment for the heading
                output.push_str("% ");
                output.push_str(&title);
                output.push('\n');
            }
        }

        NodeValue::Paragraph => {
            let mut text = String::new();
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, &mut text)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            if !text.is_empty() {
                output.push_str("% ");
                output.push_str(&text);
                output.push('\n');
            }
        }

        NodeValue::CodeBlock(code) => {
            // Try to parse as BibTeX entry
            output.push_str(&code.literal);
            output.push('\n');
            output.push('\n');
        }

        NodeValue::List(_) => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }

        NodeValue::Item(_) => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }

        NodeValue::Text(text) => {
            output.push_str(text);
        }

        _ => {
            // For other node types, render children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }
    }

    Ok(())
}

/// Render inline content.
fn render_inline(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut String,
) -> ClmdResult<()> {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Text(text) => {
            output.push_str(text);
        }

        NodeValue::SoftBreak | NodeValue::HardBreak => {
            output.push(' ');
        }

        NodeValue::Emph => {
            output.push_str("\\emph{");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push('}');
        }

        NodeValue::Strong => {
            output.push_str("\\textbf{");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push('}');
        }

        NodeValue::Code(code) => {
            output.push_str("\\texttt{");
            output.push_str(&code.literal);
            output.push('}');
        }

        NodeValue::Link(link) => {
            output.push_str("\\url{");
            output.push_str(&link.url);
            output.push('}');
        }

        _ => {
            // For other inline types, collect text from children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::PureContext;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{NodeHeading, NodeValue};
    use crate::options::WriterOptions;

    #[test]
    fn test_bibtex_writer_basic() {
        let writer = BibTeXWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.is_empty() || output == "\n");
    }

    #[test]
    fn test_bibtex_writer_format() {
        let writer = BibTeXWriter;
        assert_eq!(writer.format(), OutputFormat::Bibtex);
        assert!(writer.extensions().contains(&"bib"));
        assert!(writer.extensions().contains(&"bibtex"));
        assert_eq!(writer.mime_type(), "text/x-bibtex");
    }

    #[test]
    fn test_bibtex_writer_with_heading() {
        let writer = BibTeXWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create a heading
        let heading = NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        });
        let heading_node = arena.alloc(Node::with_value(heading));

        // Add text to heading
        let text = arena.alloc(Node::with_value(NodeValue::Text("References".into())));
        TreeOps::append_child(&mut arena, heading_node, text);
        TreeOps::append_child(&mut arena, root, heading_node);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("% References"));
    }

    #[test]
    fn test_bibtex_writer_with_code_block() {
        let writer = BibTeXWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create a code block with BibTeX content
        let code = NodeValue::CodeBlock(Box::new(crate::core::nodes::NodeCodeBlock {
            literal: "@article{key,\n  author = {Author},\n  title = {Title}\n}".into(),
            info: "bibtex".into(),
            fenced: true,
            fence_char: b'`',
            fence_length: 3,
            fence_offset: 0,
            closed: true,
        }));
        let code_node = arena.alloc(Node::with_value(code));
        TreeOps::append_child(&mut arena, root, code_node);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("@article{key"));
        assert!(output.contains("author = {Author}"));
    }

    #[test]
    fn test_bibtex_writer_with_paragraph() {
        let writer = BibTeXWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create a paragraph with text
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Some notes".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("% Some notes"));
    }

    #[test]
    fn test_bibtex_writer_with_emphasis() {
        let writer = BibTeXWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create a paragraph with emphasized text
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("emphasized".into())));
        TreeOps::append_child(&mut arena, emph, text);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("\\emph{emphasized}"));
    }

    #[test]
    fn test_bibtex_writer_with_strong() {
        let writer = BibTeXWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create a paragraph with strong text
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::Text("bold".into())));
        TreeOps::append_child(&mut arena, strong, text);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("\\textbf{bold}"));
    }

    #[test]
    fn test_bibtex_writer_with_code() {
        let writer = BibTeXWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create a paragraph with inline code
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = NodeValue::Code(Box::new(crate::core::nodes::NodeCode {
            literal: "code".into(),
            num_backticks: 1,
        }));
        let code_node = arena.alloc(Node::with_value(code));
        TreeOps::append_child(&mut arena, para, code_node);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("\\texttt{code}"));
    }

    #[test]
    fn test_bibtex_writer_with_link() {
        let writer = BibTeXWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create a paragraph with link
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = NodeValue::Link(Box::new(crate::core::nodes::NodeLink {
            url: "https://example.com".into(),
            title: "Example".into(),
        }));
        let link_node = arena.alloc(Node::with_value(link));
        TreeOps::append_child(&mut arena, para, link_node);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("\\url{https://example.com}"));
    }

    #[test]
    fn test_bibtex_writer_with_list() {
        let writer = BibTeXWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create a list with items - list items typically contain paragraphs
        let list = NodeValue::List(crate::core::nodes::NodeList {
            list_type: crate::core::nodes::ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 1,
            delimiter: crate::core::nodes::ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        });
        let list_node = arena.alloc(Node::with_value(list));

        let item = NodeValue::Item(crate::core::nodes::NodeList {
            list_type: crate::core::nodes::ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 1,
            delimiter: crate::core::nodes::ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        });
        let item_node = arena.alloc(Node::with_value(item));

        // Item needs a paragraph to contain the text
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Item 1".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, item_node, para);
        TreeOps::append_child(&mut arena, list_node, item_node);
        TreeOps::append_child(&mut arena, root, list_node);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("% Item 1"));
    }

    #[test]
    fn test_bibtex_writer_with_soft_break() {
        let writer = BibTeXWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create a paragraph with soft break
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::Text("Line 1".into())));
        let soft_break = arena.alloc(Node::with_value(NodeValue::SoftBreak));
        let text2 = arena.alloc(Node::with_value(NodeValue::Text("Line 2".into())));

        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, soft_break);
        TreeOps::append_child(&mut arena, para, text2);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("% Line 1 Line 2"));
    }

    #[test]
    fn test_bibtex_writer_empty_heading() {
        let writer = BibTeXWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create an empty heading
        let heading = NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        });
        let heading_node = arena.alloc(Node::with_value(heading));
        TreeOps::append_child(&mut arena, root, heading_node);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        // Empty heading should not produce output
        assert!(!output.contains("% "));
    }
}
