//! XML writer.
//!
//! This module provides XML output generation for documents parsed using the Arena-based parser.
//! Useful for debugging and AST inspection.

use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::core::nodes::{ListDelimType, ListType, NodeValue};
use crate::io::format::xml::escape_xml;
use crate::options::{Options, Plugins, WriterOptions};
use std::fmt;

/// Write a document as XML.
pub fn write_xml(
    arena: &NodeArena,
    root: NodeId,
    _options: &WriterOptions,
) -> ClmdResult<String> {
    let mut renderer = XmlRenderer::new(arena);
    Ok(renderer.render(root))
}

/// Render an AST as XML.
///
/// This is a convenience function that doesn't use plugins.
pub fn render(arena: &NodeArena, root: NodeId, _options: u32) -> String {
    let mut renderer = XmlRenderer::new(arena);
    renderer.render(root)
}

/// Format an AST as XML.
///
/// This is a convenience function that doesn't use plugins.
pub fn format_document(
    arena: &NodeArena,
    root: NodeId,
    options: &Options,
    output: &mut dyn fmt::Write,
) -> fmt::Result {
    format_document_with_plugins(arena, root, options, output, &Plugins::default())
}

/// Format an AST as XML with plugins (comrak-style API).
///
/// This implementation uses the new NodeArena-based AST.
pub fn format_document_with_plugins(
    arena: &NodeArena,
    root: NodeId,
    options: &Options,
    output: &mut dyn fmt::Write,
    _plugins: &Plugins<'_>,
) -> fmt::Result {
    output.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")?;
    output.write_str("<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n")?;
    format_node_xml(arena, root, options, output)
}

fn format_node_xml(
    arena: &NodeArena,
    node_id: NodeId,
    options: &Options,
    output: &mut dyn fmt::Write,
) -> fmt::Result {
    let node = arena.get(node_id);
    let tag_name = node.value.xml_node_name();

    output.write_str("<")?;
    output.write_str(tag_name)?;

    if options.render.sourcepos && node.source_pos.start.line > 0 {
        write!(
            output,
            " sourcepos=\"{}:{}-{}:{}\"",
            node.source_pos.start.line,
            node.source_pos.start.column,
            node.source_pos.end.line,
            node.source_pos.end.column
        )?;
    }

    match &node.value {
        NodeValue::List(list) => {
            match list.list_type {
                ListType::Bullet => {
                    output.write_str(" type=\"bullet\"")?;
                }
                ListType::Ordered => {
                    output.write_str(" type=\"ordered\"")?;
                    if list.start != 1 {
                        write!(output, " start=\"{}\"", list.start)?;
                    }
                    match list.delimiter {
                        ListDelimType::Period => {
                            output.write_str(" delim=\"period\"")?;
                        }
                        ListDelimType::Paren => {
                            output.write_str(" delim=\"paren\"")?;
                        }
                    }
                }
            }
            if list.tight {
                output.write_str(" tight=\"true\"")?;
            }
        }
        NodeValue::Heading(heading) => {
            write!(output, " level=\"{}\"", heading.level)?;
        }
        NodeValue::CodeBlock(code) => {
            if !code.info.is_empty() {
                write!(output, " info=\"{}\"", escape_xml(&code.info))?;
            }
        }
        NodeValue::Link(link) | NodeValue::Image(link) => {
            write!(output, " destination=\"{}\"", escape_xml(&link.url))?;
            if !link.title.is_empty() {
                write!(output, " title=\"{}\"", escape_xml(&link.title))?;
            }
        }
        NodeValue::ShortCode(shortcode) => {
            write!(output, " code=\"{}\"", escape_xml(&shortcode.code))?;
        }
        _ => {}
    }

    if node.value.is_leaf() {
        match &node.value {
            NodeValue::Text(text) => {
                if !text.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(text))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::HtmlInline(html) | NodeValue::Raw(html) => {
                if !html.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(html))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::CodeBlock(code) => {
                if !code.literal.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(&code.literal))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::HtmlBlock(html) => {
                if !html.literal.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(&html.literal))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::Code(code) => {
                if !code.literal.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(&code.literal))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::ShortCode(shortcode) => {
                if !shortcode.emoji.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(&shortcode.emoji))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            _ => {
                output.write_str(" />")?;
            }
        }
    } else {
        output.write_str(">\n")?;

        let mut child_opt = node.first_child;
        while let Some(child_id) = child_opt {
            format_node_xml(arena, child_id, options, output)?;
            child_opt = arena.get(child_id).next;
        }

        writeln!(output, "</{tag_name}>")?;
    }

    Ok(())
}

/// XML renderer state
struct XmlRenderer<'a> {
    arena: &'a NodeArena,
    output: String,
}

impl<'a> XmlRenderer<'a> {
    fn new(arena: &'a NodeArena) -> Self {
        XmlRenderer {
            arena,
            output: String::new(),
        }
    }

    fn render(&mut self, root: NodeId) -> String {
        self.output
            .push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        self.output
            .push_str("<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n");
        self.render_node(root);
        self.output.clone()
    }

    fn render_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        let tag_name = node.value.xml_node_name();

        self.output.push('<');
        self.output.push_str(tag_name);

        match &node.value {
            NodeValue::List(list) => match list.list_type {
                ListType::Bullet => {
                    self.output.push_str(" type=\"bullet\"");
                }
                ListType::Ordered => {
                    self.output.push_str(" type=\"ordered\"");
                    if list.start != 1 {
                        self.output.push_str(&format!(" start=\"{}\"", list.start));
                    }
                }
            },
            NodeValue::Heading(heading) => {
                self.output
                    .push_str(&format!(" level=\"{}\"", heading.level));
            }
            NodeValue::ShortCode(shortcode) => {
                self.output
                    .push_str(&format!(" code=\"{}\"", escape_xml(&shortcode.code)));
            }
            _ => {}
        }

        if node.value.is_leaf() {
            match &node.value {
                NodeValue::Text(text) => {
                    if !text.is_empty() {
                        self.output.push('>');
                        self.output.push_str(&escape_xml(text));
                        self.output.push_str(&format!("</{tag_name}>"));
                    } else {
                        self.output.push_str(" />");
                    }
                }
                NodeValue::ShortCode(shortcode) => {
                    if !shortcode.emoji.is_empty() {
                        self.output.push('>');
                        self.output.push_str(&escape_xml(&shortcode.emoji));
                        self.output.push_str(&format!("</{tag_name}>"));
                    } else {
                        self.output.push_str(" />");
                    }
                }
                _ => {
                    self.output.push_str(" />");
                }
            }
        } else {
            self.output.push_str(">\n");

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                self.render_node(child_id);
                child_opt = self.arena.get(child_id).next;
            }

            self.output.push_str(&format!("</{tag_name}>\n"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{
        NodeCode, NodeCodeBlock, NodeFootnoteDefinition, NodeFootnoteReference,
        NodeHeading, NodeHtmlBlock, NodeLink, NodeList, NodeTable, NodeValue,
    };

    #[test]
    fn test_render_empty_document() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let output = render(&arena, root, 0);
        assert!(output.contains("<?xml version="));
        assert!(output.contains("<!DOCTYPE document"));
        assert!(output.contains("<document>"));
        assert!(output.contains("</document>"));
    }

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Hello world".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let output = render(&arena, root, 0);
        assert!(output.contains("<paragraph>"));
        assert!(output.contains("<text>Hello world</text>"));
        assert!(output.contains("</paragraph>"));
    }

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));
        let text =
            arena.alloc(Node::with_value(NodeValue::Text("Section Title".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let output = render(&arena, root, 0);
        assert!(output.contains("<heading level=\"2\">"));
        assert!(output.contains("<text>Section Title</text>"));
        assert!(output.contains("</heading>"));
    }

    #[test]
    fn test_render_all_heading_levels() {
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
                format!("H{}", level).into(),
            )));
            TreeOps::append_child(&mut arena, heading, text);
            TreeOps::append_child(&mut arena, root, heading);

            let output = render(&arena, root, 0);
            assert!(output.contains(&format!("level=\"{}\"", level)));
        }
    }

    #[test]
    fn test_render_emphasis() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("emphasized".into())));
        TreeOps::append_child(&mut arena, emph, text);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, root, para);

        let output = render(&arena, root, 0);
        assert!(output.contains("<emph>"));
        assert!(output.contains("<text>emphasized</text>"));
        assert!(output.contains("</emph>"));
    }

    #[test]
    fn test_render_strong() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::Text("bold".into())));
        TreeOps::append_child(&mut arena, strong, text);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, root, para);

        let output = render(&arena, root, 0);
        assert!(output.contains("<strong>"));
        assert!(output.contains("<text>bold</text>"));
        assert!(output.contains("</strong>"));
    }

    #[test]
    fn test_render_code() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = NodeValue::Code(Box::new(NodeCode {
            literal: "code snippet".into(),
            num_backticks: 1,
        }));
        let code_node = arena.alloc(Node::with_value(code));
        TreeOps::append_child(&mut arena, para, code_node);
        TreeOps::append_child(&mut arena, root, para);

        let options = Options::default();
        let mut output = String::new();
        format_document(&arena, root, &options, &mut output).unwrap();
        assert!(output.contains("<code>"));
        assert!(output.contains("code snippet"));
        assert!(output.contains("</code>"));
    }

    #[test]
    fn test_render_code_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let code_block = NodeValue::CodeBlock(Box::new(NodeCodeBlock {
            literal: "fn main() {}".into(),
            info: "rust".into(),
            fenced: true,
            fence_char: b'`',
            fence_length: 3,
            fence_offset: 0,
            closed: true,
        }));
        let code_node = arena.alloc(Node::with_value(code_block));
        TreeOps::append_child(&mut arena, root, code_node);

        let options = Options::default();
        let mut output = String::new();
        format_document(&arena, root, &options, &mut output).unwrap();
        assert!(output.contains("<code_block"));
        assert!(output.contains("info=\"rust\""));
        assert!(output.contains("fn main() {}"));
        assert!(output.contains("</code_block>"));
    }

    #[test]
    fn test_render_code_block_empty() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let code_block = NodeValue::CodeBlock(Box::new(NodeCodeBlock {
            literal: "".into(),
            info: "".into(),
            fenced: true,
            fence_char: b'`',
            fence_length: 3,
            fence_offset: 0,
            closed: true,
        }));
        let code_node = arena.alloc(Node::with_value(code_block));
        TreeOps::append_child(&mut arena, root, code_node);

        let options = Options::default();
        let mut output = String::new();
        format_document(&arena, root, &options, &mut output).unwrap();
        assert!(output.contains("<code_block"));
        assert!(output.contains(" />"));
    }

    #[test]
    fn test_render_link() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com".into(),
            title: "Example".into(),
        }));
        let link_node = arena.alloc(Node::with_value(link));
        let text = arena.alloc(Node::with_value(NodeValue::Text("click here".into())));
        TreeOps::append_child(&mut arena, link_node, text);
        TreeOps::append_child(&mut arena, para, link_node);
        TreeOps::append_child(&mut arena, root, para);

        let options = Options::default();
        let mut output = String::new();
        format_document(&arena, root, &options, &mut output).unwrap();
        assert!(output.contains("<link"));
        assert!(output.contains("destination=\"https://example.com\""));
        assert!(output.contains("title=\"Example\""));
        assert!(output.contains("<text>click here</text>"));
        assert!(output.contains("</link>"));
    }

    #[test]
    fn test_render_image() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let image = NodeValue::Image(Box::new(NodeLink {
            url: "image.png".into(),
            title: "Alt text".into(),
        }));
        let image_node = arena.alloc(Node::with_value(image));
        TreeOps::append_child(&mut arena, para, image_node);
        TreeOps::append_child(&mut arena, root, para);

        let options = Options::default();
        let mut output = String::new();
        format_document(&arena, root, &options, &mut output).unwrap();
        assert!(output.contains("<image"));
        assert!(output.contains("destination=\"image.png\""));
        assert!(output.contains("title=\"Alt text\""));
    }

    #[test]
    fn test_render_bullet_list() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let list = NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 1,
            delimiter: crate::core::nodes::ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        });
        let list_node = arena.alloc(Node::with_value(list));

        let item = NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 1,
            delimiter: crate::core::nodes::ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        });
        let item_node = arena.alloc(Node::with_value(item));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Item 1".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, item_node, para);
        TreeOps::append_child(&mut arena, list_node, item_node);
        TreeOps::append_child(&mut arena, root, list_node);

        let options = Options::default();
        let mut output = String::new();
        format_document(&arena, root, &options, &mut output).unwrap();
        assert!(output.contains("<list type=\"bullet\""));
        assert!(output.contains("tight=\"true\""));
        assert!(output.contains("<item>"));
        assert!(output.contains("<text>Item 1</text>"));
    }

    #[test]
    fn test_render_ordered_list() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let list = NodeValue::List(NodeList {
            list_type: ListType::Ordered,
            marker_offset: 0,
            padding: 2,
            start: 5,
            delimiter: crate::core::nodes::ListDelimType::Period,
            bullet_char: b'.',
            tight: false,
            is_task_list: false,
        });
        let list_node = arena.alloc(Node::with_value(list));
        TreeOps::append_child(&mut arena, root, list_node);

        let options = Options::default();
        let mut output = String::new();
        format_document(&arena, root, &options, &mut output).unwrap();
        assert!(output.contains("<list type=\"ordered\""));
        assert!(output.contains("start=\"5\""));
        assert!(output.contains("delim=\"period\""));
    }

    #[test]
    fn test_render_blockquote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let quote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Quoted text".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, quote, para);
        TreeOps::append_child(&mut arena, root, quote);

        let output = render(&arena, root, 0);
        assert!(output.contains("<block_quote>"));
        assert!(output.contains("<text>Quoted text</text>"));
        assert!(output.contains("</block_quote>"));
    }

    #[test]
    fn test_render_thematic_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let hr = arena.alloc(Node::with_value(NodeValue::ThematicBreak(
            crate::core::nodes::NodeThematicBreak::default(),
        )));
        TreeOps::append_child(&mut arena, root, hr);

        let output = render(&arena, root, 0);
        assert!(output.contains("<thematic_break />"));
    }

    #[test]
    fn test_render_soft_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::Text("Line 1".into())));
        let soft_break = arena.alloc(Node::with_value(NodeValue::SoftBreak));
        let text2 = arena.alloc(Node::with_value(NodeValue::Text("Line 2".into())));
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, soft_break);
        TreeOps::append_child(&mut arena, para, text2);
        TreeOps::append_child(&mut arena, root, para);

        let output = render(&arena, root, 0);
        assert!(output.contains("<softbreak />"));
    }

    #[test]
    fn test_render_hard_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::Text("Line 1".into())));
        let hard_break = arena.alloc(Node::with_value(NodeValue::HardBreak));
        let text2 = arena.alloc(Node::with_value(NodeValue::Text("Line 2".into())));
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, hard_break);
        TreeOps::append_child(&mut arena, para, text2);
        TreeOps::append_child(&mut arena, root, para);

        let output = render(&arena, root, 0);
        assert!(output.contains("<linebreak />"));
    }

    #[test]
    fn test_render_strikethrough() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strike = arena.alloc(Node::with_value(NodeValue::Strikethrough));
        let text = arena.alloc(Node::with_value(NodeValue::Text("deleted".into())));
        TreeOps::append_child(&mut arena, strike, text);
        TreeOps::append_child(&mut arena, para, strike);
        TreeOps::append_child(&mut arena, root, para);

        let output = render(&arena, root, 0);
        assert!(output.contains("<strikethrough>"));
        assert!(output.contains("<text>deleted</text>"));
        assert!(output.contains("</strikethrough>"));
    }

    #[test]
    fn test_render_footnote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let footnote_def =
            NodeValue::FootnoteDefinition(Box::new(NodeFootnoteDefinition {
                name: "note1".into(),
                total_references: 1,
            }));
        let def_node = arena.alloc(Node::with_value(footnote_def));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text =
            arena.alloc(Node::with_value(NodeValue::Text("Footnote content".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, def_node, para);
        TreeOps::append_child(&mut arena, root, def_node);

        let footnote_ref =
            NodeValue::FootnoteReference(Box::new(NodeFootnoteReference {
                name: "note1".into(),
                ref_num: 1,
                ix: 0,
            }));
        let ref_node = arena.alloc(Node::with_value(footnote_ref));
        TreeOps::append_child(&mut arena, root, ref_node);

        let output = render(&arena, root, 0);
        assert!(output.contains("<footnote_definition"));
        assert!(output.contains("<footnote_reference"));
    }

    #[test]
    fn test_render_empty_text() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let output = render(&arena, root, 0);
        assert!(output.contains("<text />"));
    }

    #[test]
    fn test_format_document_with_options() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Test".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let options = Options::default();
        let mut output = String::new();
        format_document(&arena, root, &options, &mut output).unwrap();

        assert!(output.contains("<?xml version="));
        assert!(output.contains("<document>"));
        assert!(output.contains("<text>Test</text>"));
    }

    #[test]
    fn test_render_nested_structure() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let list = NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 1,
            delimiter: crate::core::nodes::ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        });
        let list_node = arena.alloc(Node::with_value(list));

        let item = NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 1,
            delimiter: crate::core::nodes::ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        });
        let item_node = arena.alloc(Node::with_value(item));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("nested".into())));

        TreeOps::append_child(&mut arena, emph, text);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, item_node, para);
        TreeOps::append_child(&mut arena, list_node, item_node);
        TreeOps::append_child(&mut arena, root, list_node);

        let output = render(&arena, root, 0);
        assert!(output.contains("<list"));
        assert!(output.contains("<item>"));
        assert!(output.contains("<emph>"));
        assert!(output.contains("<text>nested</text>"));
    }

    #[test]
    fn test_render_html_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let html_block = NodeValue::HtmlBlock(Box::new(NodeHtmlBlock {
            literal: "<div>content</div>".into(),
            block_type: 6,
        }));
        let html_node = arena.alloc(Node::with_value(html_block));
        TreeOps::append_child(&mut arena, root, html_node);

        let options = Options::default();
        let mut output = String::new();
        format_document(&arena, root, &options, &mut output).unwrap();
        assert!(output.contains("<html_block>"));
        assert!(output.contains("&lt;div&gt;content&lt;/div&gt;"));
        assert!(output.contains("</html_block>"));
    }

    #[test]
    fn test_render_table() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let table = NodeValue::Table(Box::new(NodeTable {
            alignments: vec![],
            num_columns: 2,
            num_rows: 1,
            num_nonempty_cells: 2,
        }));
        let table_node = arena.alloc(Node::with_value(table));
        TreeOps::append_child(&mut arena, root, table_node);

        let output = render(&arena, root, 0);
        assert!(output.contains("<table>"));
        assert!(output.contains("</table>"));
    }
}
