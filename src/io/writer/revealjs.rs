//! RevealJS document writer.
//!
//! This module provides a writer for RevealJS (HTML slides) format.
//! It uses the shared HTML renderer from `html_renderer` module.
//!
//! # Example
//!
//! ```ignore
//! use clmd::io::writer::RevealJsWriter;
//! use clmd::options::WriterOptions;
//! use clmd::context::PureContext;
//!
//! let writer = RevealJsWriter;
//! let ctx = PureContext::new();
//! let output = writer.write(&arena, root, &ctx, &WriterOptions::default()).unwrap();
//! ```

use crate::context::ClmdContext;
use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::io::writer::{Writer, HtmlMode, HtmlRenderer};
use crate::options::{OutputFormat, WriterOptions};

/// RevealJS document writer.
///
/// This writer uses the shared HTML renderer in RevealJs mode.
#[derive(Debug, Clone, Copy)]
pub struct RevealJsWriter;

impl Writer for RevealJsWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        // Use the shared HTML renderer in RevealJs mode
        HtmlRenderer::new(arena, options, HtmlMode::RevealJs).render(root)
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::RevealJs
    }

    fn extensions(&self) -> &[&'static str] {
        &["html", "revealjs"]
    }

    fn mime_type(&self) -> &'static str {
        "text/html"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::PureContext;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{
        NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeList, NodeValue,
    };
    use crate::core::nodes::{ListType, ListDelimType};
    use crate::options::WriterOptions;

    fn create_test_presentation() -> (NodeArena, NodeId) {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Add a title heading
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text =
            arena.alloc(Node::with_value(NodeValue::Text("My Presentation".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        // Add a slide heading
        let slide = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));
        let slide_text =
            arena.alloc(Node::with_value(NodeValue::Text("First Slide".into())));
        TreeOps::append_child(&mut arena, slide, slide_text);
        TreeOps::append_child(&mut arena, root, slide);

        // Add a paragraph
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para_text =
            arena.alloc(Node::with_value(NodeValue::Text("Hello World".into())));
        TreeOps::append_child(&mut arena, para, para_text);
        TreeOps::append_child(&mut arena, root, para);

        (arena, root)
    }

    #[test]
    fn test_revealjs_writer_basic() {
        let writer = RevealJsWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let (arena, root) = create_test_presentation();

        let output = writer.write(&arena, root, &ctx, &options).unwrap();

        // Check for RevealJS structure
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("reveal.js"));
        assert!(output.contains("<div class=\"reveal\">"));
        assert!(output.contains("<div class=\"slides\">"));
        assert!(output.contains("<section>"));
        assert!(output.contains("My Presentation"));
        assert!(output.contains("Hello World"));
    }

    #[test]
    fn test_revealjs_writer_format() {
        let writer = RevealJsWriter;
        assert_eq!(writer.format(), OutputFormat::RevealJs);
        assert!(writer.extensions().contains(&"html"));
        assert!(writer.extensions().contains(&"revealjs"));
        assert_eq!(writer.mime_type(), "text/html");
    }

    #[test]
    fn test_revealjs_empty_document() {
        let writer = RevealJsWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_revealjs_level1_heading_slide() {
        let writer = RevealJsWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Title Slide".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("<section>"));
        assert!(output.contains("<h1>Title Slide</h1>"));
        assert!(output.contains("</section>"));
    }

    #[test]
    fn test_revealjs_level2_heading_slide() {
        let writer = RevealJsWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));
        let text =
            arena.alloc(Node::with_value(NodeValue::Text("Content Slide".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("<section>"));
        assert!(output.contains("<h2>Content Slide</h2>"));
        assert!(output.contains("</section>"));
    }

    #[test]
    fn test_revealjs_paragraph_slide() {
        let writer = RevealJsWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text(
            "Paragraph content".into(),
        )));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("<section>"));
        assert!(output.contains("<p>Paragraph content</p>"));
        assert!(output.contains("</section>"));
    }

    #[test]
    fn test_revealjs_code_block_slide() {
        let writer = RevealJsWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
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

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("<section>"));
        assert!(output.contains("<pre><code>"));
        assert!(output.contains("fn main() {}"));
        assert!(output.contains("</code></pre>"));
        assert!(output.contains("</section>"));
    }

    #[test]
    fn test_revealjs_emphasis() {
        let writer = RevealJsWriter;
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
        assert!(output.contains("<em>italic</em>"));
    }

    #[test]
    fn test_revealjs_strong() {
        let writer = RevealJsWriter;
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
        assert!(output.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_revealjs_code_inline() {
        let writer = RevealJsWriter;
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
        assert!(output.contains("<code>code</code>"));
    }

    #[test]
    fn test_revealjs_link() {
        let writer = RevealJsWriter;
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
        assert!(output.contains(r#"<a href="https://example.com">click</a>"#));
    }

    #[test]
    fn test_revealjs_strikethrough() {
        let writer = RevealJsWriter;
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
        assert!(output.contains("<del>deleted</del>"));
    }

    #[test]
    fn test_revealjs_underline() {
        let writer = RevealJsWriter;
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
        assert!(output.contains("<u>underlined</u>"));
    }

    #[test]
    fn test_revealjs_soft_break() {
        let writer = RevealJsWriter;
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
        assert!(output.contains("Line1 Line2"));
    }

    #[test]
    fn test_revealjs_hard_break() {
        let writer = RevealJsWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::Text("Line1".into())));
        let hard_break = arena.alloc(Node::with_value(NodeValue::HardBreak));
        let text2 = arena.alloc(Node::with_value(NodeValue::Text("Line2".into())));
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, hard_break);
        TreeOps::append_child(&mut arena, para, text2);
        TreeOps::append_child(&mut arena, root, para);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("Line1<br/>Line2"));
    }

    #[test]
    fn test_revealjs_blockquote() {
        let writer = RevealJsWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let quote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Quoted text".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, quote, para);
        TreeOps::append_child(&mut arena, root, quote);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("<section>"));
        assert!(output.contains("<blockquote>"));
        assert!(output.contains("<p>Quoted text</p>"));
        assert!(output.contains("</blockquote>"));
        assert!(output.contains("</section>"));
    }

    #[test]
    fn test_revealjs_list() {
        let writer = RevealJsWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let list = NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 1,
            delimiter: ListDelimType::Period,
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
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        });
        let item_node = arena.alloc(Node::with_value(item));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Item 1".into())));
        TreeOps::append_child(&mut arena, item_node, text);
        TreeOps::append_child(&mut arena, list_node, item_node);
        TreeOps::append_child(&mut arena, root, list_node);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("<section>"));
        assert!(output.contains("<ul>"));
        assert!(output.contains("<li>Item 1</li>"));
        assert!(output.contains("</ul>"));
        assert!(output.contains("</section>"));
    }

    #[test]
    fn test_revealjs_nested_inline() {
        let writer = RevealJsWriter;
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
        assert!(output.contains("<strong><em>bold italic</em></strong>"));
    }
}
