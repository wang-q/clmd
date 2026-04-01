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
use crate::io::writer::Writer;
use crate::options::{OutputFormat, WriterOptions};

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
    use crate::core::nodes::{
        NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeValue,
    };
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
        let text = arena.alloc(Node::with_value(NodeValue::Text("Hello".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        // Add a paragraph
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para_text = arena.alloc(Node::with_value(NodeValue::Text("World".into())));
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
        assert_eq!(writer.mime_type(), "application/rtf");
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

    #[test]
    fn test_rtf_all_heading_levels() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        for level in 1..=6 {
            let mut arena = NodeArena::new();
            let root = arena.alloc(Node::with_value(NodeValue::Document));
            let heading =
                arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
                    level,
                    setext: false,
                    closed: false,
                })));
            let text = arena.alloc(Node::with_value(NodeValue::Text(
                format!("Heading {}", level).into(),
            )));
            TreeOps::append_child(&mut arena, heading, text);
            TreeOps::append_child(&mut arena, root, heading);

            let output = writer.write(&arena, root, &ctx, &options).unwrap();
            assert!(output.contains(&format!("Heading {}", level)));
            assert!(output.contains(r"\b ") || output.contains(r"\b0"));
        }
    }

    #[test]
    fn test_rtf_emphasis() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("italic".into())));
        TreeOps::append_child(&mut arena, emph, text);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains(r"\i "));
        assert!(output.contains(r"\i0"));
        assert!(output.contains("italic"));
    }

    #[test]
    fn test_rtf_strong() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::Text("bold".into())));
        TreeOps::append_child(&mut arena, strong, text);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains(r"\b "));
        assert!(output.contains(r"\b0"));
        assert!(output.contains("bold"));
    }

    #[test]
    fn test_rtf_strikethrough() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strike = arena.alloc(Node::with_value(NodeValue::Strikethrough));
        let text = arena.alloc(Node::with_value(NodeValue::Text("deleted".into())));
        TreeOps::append_child(&mut arena, strike, text);
        TreeOps::append_child(&mut arena, para, strike);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains(r"\strike "));
        assert!(output.contains(r"\strike0"));
        assert!(output.contains("deleted"));
    }

    #[test]
    fn test_rtf_underline() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let underline = arena.alloc(Node::with_value(NodeValue::Underline));
        let text = arena.alloc(Node::with_value(NodeValue::Text("underlined".into())));
        TreeOps::append_child(&mut arena, underline, text);
        TreeOps::append_child(&mut arena, para, underline);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains(r"\ul "));
        assert!(output.contains(r"\ulnone"));
        assert!(output.contains("underlined"));
    }

    #[test]
    fn test_rtf_code() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = NodeValue::Code(Box::new(NodeCode {
            literal: "code".into(),
            num_backticks: 1,
        }));
        let code_node = arena.alloc(Node::with_value(code));
        TreeOps::append_child(&mut arena, para, code_node);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("code"));
    }

    #[test]
    fn test_rtf_code_block() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let code_block = NodeValue::CodeBlock(Box::new(NodeCodeBlock {
            literal: "fn main() {\n    println!();\n}".into(),
            info: "rust".into(),
            fenced: true,
            fence_char: b'`',
            fence_length: 3,
            fence_offset: 0,
            closed: true,
        }));
        let code_node = arena.alloc(Node::with_value(code_block));
        TreeOps::append_child(&mut arena, root, code_node);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("fn main()"));
        assert!(output.contains(r"\line ")); // Line breaks in code
    }

    #[test]
    fn test_rtf_code_block_escape() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let code_block = NodeValue::CodeBlock(Box::new(NodeCodeBlock {
            literal: "{ \\ }".into(),
            info: "".into(),
            fenced: true,
            fence_char: b'`',
            fence_length: 3,
            fence_offset: 0,
            closed: true,
        }));
        let code_node = arena.alloc(Node::with_value(code_block));
        TreeOps::append_child(&mut arena, root, code_node);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains(r"\{")); // Escaped {
        assert!(output.contains(r"\\")); // Escaped \
        assert!(output.contains(r"\}")); // Escaped }
    }

    #[test]
    fn test_rtf_link() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com".into(),
            title: "Example".into(),
        }));
        let link_node = arena.alloc(Node::with_value(link));
        let text = arena.alloc(Node::with_value(NodeValue::Text("click".into())));
        TreeOps::append_child(&mut arena, link_node, text);
        TreeOps::append_child(&mut arena, para, link_node);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("HYPERLINK"));
        assert!(output.contains("https://example.com"));
        assert!(output.contains("click"));
    }

    #[test]
    fn test_rtf_blockquote() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let quote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Quoted".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, quote, para);
        TreeOps::append_child(&mut arena, root, quote);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains(r"\li720 ")); // Indent for blockquote
        assert!(output.contains("Quoted"));
    }

    #[test]
    fn test_rtf_thematic_break() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let hr = arena.alloc(Node::with_value(NodeValue::ThematicBreak));
        TreeOps::append_child(&mut arena, root, hr);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains(r"\brdrb")); // Border
    }

    #[test]
    fn test_rtf_soft_break() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::Text("Line1".into())));
        let soft_break = arena.alloc(Node::with_value(NodeValue::SoftBreak));
        let text2 = arena.alloc(Node::with_value(NodeValue::Text("Line2".into())));
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, soft_break);
        TreeOps::append_child(&mut arena, para, text2);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("Line1 Line2")); // Soft break becomes space
    }

    #[test]
    fn test_rtf_text_escape() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("{ \\ }".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains(r"\{")); // Escaped {
        assert!(output.contains(r"\\")); // Escaped \
        assert!(output.contains(r"\}")); // Escaped }
    }

    #[test]
    fn test_rtf_nested_inline() {
        let writer = RtfWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("bold italic".into())));
        TreeOps::append_child(&mut arena, emph, text);
        TreeOps::append_child(&mut arena, strong, emph);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains(r"\b "));
        assert!(output.contains(r"\i "));
        assert!(output.contains("bold italic"));
    }
}
