//! RTF document writer.
//!
//! This module provides a writer for RTF (Rich Text Format) format.
//!
//! # Example
//!
//! ```ignore
//! use clmd::writers::RtfWriter;
//! use clmd::options::WriterOptions;
//! use clmd::context::PureContext;
//!
//! let writer = RtfWriter;
//! let ctx = PureContext::new();
//! let output = writer.write(&arena, root, &ctx, &WriterOptions::default()).unwrap();
//! ```

use crate::context::ClmdContext;
use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::core::nodes::NodeValue;
use crate::options::{OutputFormat, WriterOptions};
use crate::writers::Writer;

/// RTF document writer.
#[derive(Debug, Clone, Copy)]
pub struct RtfWriter;

impl Writer for RtfWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        _options: &WriterOptions,
    ) -> ClmdResult<String> {
        let mut output = String::new();

        // RTF header
        output.push_str(r"{\rtf1\ansi\deff0");
        output.push_str(r"{\fonttbl{\f0\fnil\fcharset0 Arial;}}");
        output.push_str(r"{\colortbl ;}");
        output.push_str(r"\viewkind4\uc1");

        // Document content
        render_node(arena, root, &mut output, false)?;

        // RTF footer
        output.push('}');

        Ok(output)
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Rtf
    }

    fn extensions(&self) -> &[&'static str] {
        &["rtf"]
    }

    fn mime_type(&self) -> &'static str {
        "application/rtf"
    }
}

/// Render a node and its children to RTF.
fn render_node(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut String,
    in_block: bool,
) -> ClmdResult<()> {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Document => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output, false)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }

        NodeValue::Heading(heading) => {
            output.push_str("\n\\pard ");

            // Set font size based on heading level
            let font_size = match heading.level {
                1 => "\\fs32\\b ",
                2 => "\\fs28\\b ",
                3 => "\\fs24\\b ",
                4 => "\\fs22\\b ",
                5 => "\\fs20\\b ",
                _ => "\\fs18\\b ",
            };
            output.push_str(font_size);

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("\\b0\\par\n");
        }

        NodeValue::Paragraph => {
            if !in_block {
                output.push_str("\n\\pard ");
            }

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            if !in_block {
                output.push_str("\\par\n");
            }
        }

        NodeValue::BlockQuote => {
            output.push_str("\n\\pard \\li720 ");

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output, true)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("\\par\n");
        }

        NodeValue::CodeBlock(code) => {
            output.push_str("\n\\pard \\f0\\fs20 ");

            // Escape RTF special characters in code
            for c in code.literal.chars() {
                match c {
                    '\\' => output.push_str("\\\\"),
                    '{' => output.push_str("\\{"),
                    '}' => output.push_str("\\}"),
                    '\n' => output.push_str("\\line "),
                    _ => output.push(c),
                }
            }

            output.push_str("\\par\n");
        }

        NodeValue::List(_) => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output, false)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }

        NodeValue::Item(_) => {
            output.push_str("\n\\pard \\li360 \\bullet\\tab ");

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output, true)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("\\par\n");
        }

        NodeValue::ThematicBreak => {
            output.push_str("\n\\pard \\brdrb \\brdrs \\brdrw10 \\par\n");
        }

        _ => {
            // For other node types, render children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output, in_block)?;
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
            // Escape RTF special characters
            for c in text.chars() {
                match c {
                    '\\' => output.push_str("\\\\"),
                    '{' => output.push_str("\\{"),
                    '}' => output.push_str("\\}"),
                    '\n' => output.push(' '),
                    _ => output.push(c),
                }
            }
        }

        NodeValue::SoftBreak | NodeValue::HardBreak => {
            output.push(' ');
        }

        NodeValue::Emph => {
            output.push_str("\\i ");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("\\i0 ");
        }

        NodeValue::Strong => {
            output.push_str("\\b ");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("\\b0 ");
        }

        NodeValue::Code(code) => {
            output.push_str("\\f0\\fs20 ");
            for c in code.literal.chars() {
                match c {
                    '\\' => output.push_str("\\\\"),
                    '{' => output.push_str("\\{"),
                    '}' => output.push_str("\\}"),
                    _ => output.push(c),
                }
            }
            output.push_str("\\f0 ");
        }

        NodeValue::Link(link) => {
            // RTF hyperlink
            output.push_str("{\\field{\\*\\fldinst{HYPERLINK \"");
            for c in link.url.chars() {
                match c {
                    '\\' => output.push_str("\\\\"),
                    '"' => output.push_str("\\\""),
                    _ => output.push(c),
                }
            }
            output.push_str("\"}}{\\fldrslt ");

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("}}");
        }

        NodeValue::Strikethrough => {
            output.push_str("\\strike ");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("\\strike0 ");
        }

        NodeValue::Underline => {
            output.push_str("\\ul ");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("\\ulnone ");
        }

        _ => {
            // For other inline types, render children
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

    fn create_test_document() -> (NodeArena, NodeId) {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Add a heading
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text(
            "Hello".to_string().into_boxed_str(),
        )));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        // Add a paragraph
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para_text = arena.alloc(Node::with_value(NodeValue::Text(
            "World".to_string().into_boxed_str(),
        )));
        TreeOps::append_child(&mut arena, para, para_text);
        TreeOps::append_child(&mut arena, root, para);

        (arena, root)
    }

    #[test]
    fn test_rtf_writer_basic() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let (arena, root) = create_test_document();

        let output = writer.write(&arena, root, &ctx, &options).unwrap();

        // Check for RTF header
        assert!(output.contains(r"{\rtf1"));
        assert!(output.contains("Hello"));
        assert!(output.contains("World"));
        assert!(output.ends_with('}'));
    }

    #[test]
    fn test_rtf_writer_format() {
        let writer = RtfWriter;
        assert_eq!(writer.format(), OutputFormat::Rtf);
        assert!(writer.extensions().contains(&"rtf"));
    }

    #[test]
    fn test_rtf_writer_empty() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains(r"{\rtf1"));
        assert!(output.ends_with('}'));
    }
}
