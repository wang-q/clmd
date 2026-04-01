//! RevealJS document writer.
//!
//! This module provides a writer for RevealJS (HTML slides) format.
//!
//! # Example
//!
//! ```ignore
//! use clmd::writers::RevealJsWriter;
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
use crate::core::nodes::NodeValue;
use crate::io::writer::Writer;
use crate::options::{OutputFormat, WriterOptions};

/// RevealJS document writer.
#[derive(Debug, Clone, Copy)]
pub struct RevealJsWriter;

impl Writer for RevealJsWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        _options: &WriterOptions,
    ) -> ClmdResult<String> {
        let mut output = String::new();

        // HTML preamble
        output.push_str(REVEALJS_PREAMBLE);

        // Extract title if available
        let title =
            extract_title(arena, root).unwrap_or_else(|| "Presentation".to_string());
        output.push_str(&format!("<title>{}</title>\n", escape_html(&title)));

        output.push_str(REVEALJS_HEAD_END);

        // Slides content
        output.push_str("<div class=\"reveal\">\n");
        output.push_str("<div class=\"slides\">\n");

        render_slides(arena, root, &mut output)?;

        output.push_str("</div>\n");
        output.push_str("</div>\n");

        output.push_str(REVEALJS_FOOTER);

        Ok(output)
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

/// Extract title from the first level 1 heading.
fn extract_title(arena: &NodeArena, root: NodeId) -> Option<String> {
    let root_node = arena.get(root);
    let mut child_opt = root_node.first_child;

    while let Some(child_id) = child_opt {
        let child = arena.get(child_id);
        if let NodeValue::Heading(heading) = &child.value {
            if heading.level == 1 {
                let mut title = String::new();
                let mut text_opt = child.first_child;
                while let Some(text_id) = text_opt {
                    let text_node = arena.get(text_id);
                    if let NodeValue::Text(t) = &text_node.value {
                        title.push_str(t);
                    }
                    text_opt = text_node.next;
                }
                return Some(title);
            }
        }
        child_opt = child.next;
    }

    None
}

/// Render slides from the AST.
fn render_slides(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut String,
) -> ClmdResult<()> {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Document => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_slides(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }

        NodeValue::Heading(heading) => {
            if heading.level == 1 {
                // Level 1 heading - section divider
                let mut title = String::new();
                let mut text_opt = node.first_child;
                while let Some(text_id) = text_opt {
                    let text_node = arena.get(text_id);
                    if let NodeValue::Text(t) = &text_node.value {
                        title.push_str(t);
                    }
                    text_opt = text_node.next;
                }

                output.push_str("<section>\n");
                output.push_str(&format!("<h1>{}</h1>\n", escape_html(&title)));
                output.push_str("</section>\n\n");
            } else {
                // Level 2+ headings become slide titles
                output.push_str("<section>\n");

                let mut title = String::new();
                let mut text_opt = node.first_child;
                while let Some(text_id) = text_opt {
                    let text_node = arena.get(text_id);
                    if let NodeValue::Text(t) = &text_node.value {
                        title.push_str(t);
                    }
                    text_opt = text_node.next;
                }

                output.push_str(&format!("<h2>{}</h2>\n", escape_html(&title)));

                // Render any following content until next heading
                let mut sibling_opt = node.next;
                while let Some(sibling_id) = sibling_opt {
                    let sibling = arena.get(sibling_id);
                    if matches!(sibling.value, NodeValue::Heading(_)) {
                        break;
                    }
                    render_slide_content(arena, sibling_id, output)?;
                    sibling_opt = sibling.next;
                }

                output.push_str("</section>\n\n");
            }
        }

        NodeValue::Paragraph => {
            output.push_str("<section>\n");
            render_slide_content(arena, node_id, output)?;
            output.push_str("</section>\n\n");
        }

        NodeValue::List(_) => {
            output.push_str("<section>\n");
            render_slide_content(arena, node_id, output)?;
            output.push_str("</section>\n\n");
        }

        NodeValue::BlockQuote => {
            output.push_str("<section>\n");
            render_slide_content(arena, node_id, output)?;
            output.push_str("</section>\n\n");
        }

        NodeValue::CodeBlock(code) => {
            output.push_str("<section>\n");
            output.push_str("<pre><code>");
            escape_html_to(&code.literal, output);
            output.push_str("</code></pre>\n");
            output.push_str("</section>\n\n");
        }

        _ => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_slides(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }
    }

    Ok(())
}

/// Render content inside a slide.
fn render_slide_content(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut String,
) -> ClmdResult<()> {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Paragraph => {
            output.push_str("<p>");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("</p>\n");
        }

        NodeValue::List(_) => {
            output.push_str("<ul>\n");

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                let child = arena.get(child_id);
                if matches!(child.value, NodeValue::Item(_)) {
                    output.push_str("<li>");
                    let mut item_child_opt = child.first_child;
                    while let Some(item_child_id) = item_child_opt {
                        render_inline(arena, item_child_id, output)?;
                        let item_child = arena.get(item_child_id);
                        item_child_opt = item_child.next;
                    }
                    output.push_str("</li>\n");
                }
                child_opt = child.next;
            }

            output.push_str("</ul>\n");
        }

        NodeValue::BlockQuote => {
            output.push_str("<blockquote>\n");

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_slide_content(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("</blockquote>\n");
        }

        _ => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_slide_content(arena, child_id, output)?;
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
            escape_html_to(text, output);
        }

        NodeValue::SoftBreak => {
            output.push(' ');
        }

        NodeValue::HardBreak => {
            output.push_str("<br/>");
        }

        NodeValue::Emph => {
            output.push_str("<em>");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("</em>");
        }

        NodeValue::Strong => {
            output.push_str("<strong>");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("</strong>");
        }

        NodeValue::Code(code) => {
            output.push_str("<code>");
            escape_html_to(&code.literal, output);
            output.push_str("</code>");
        }

        NodeValue::Link(link) => {
            output.push_str(&format!(r#"<a href="{}">"#, escape_html(&link.url)));
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("</a>");
        }

        NodeValue::Strikethrough => {
            output.push_str("<del>");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("</del>");
        }

        NodeValue::Underline => {
            output.push_str("<u>");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("</u>");
        }

        _ => {
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

/// Escape HTML special characters.
fn escape_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    escape_html_to(text, &mut result);
    result
}

/// Escape HTML special characters to output.
fn escape_html_to(text: &str, output: &mut String) {
    for c in text.chars() {
        match c {
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '&' => output.push_str("&amp;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27;"),
            _ => output.push(c),
        }
    }
}

/// RevealJS HTML preamble.
const REVEALJS_PREAMBLE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
"#;

const REVEALJS_HEAD_END: &str = r#"    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/reveal.js@4.5.0/dist/reveal.css">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/reveal.js@4.5.0/dist/theme/white.css">
    <style>
        .reveal h1, .reveal h2, .reveal h3 {
            text-transform: none;
        }
        .reveal pre {
            background: #f5f5f5;
            padding: 1em;
            border-radius: 4px;
        }
        .reveal code {
            font-family: 'Fira Code', 'Consolas', monospace;
            background: #f5f5f5;
            padding: 0.1em 0.3em;
            border-radius: 3px;
        }
        .reveal pre code {
            background: transparent;
            padding: 0;
        }
        .reveal blockquote {
            background: #f9f9f9;
            border-left: 4px solid #ccc;
            padding: 0.5em 1em;
            font-style: italic;
        }
        .reveal ul {
            list-style-type: disc;
        }
        .reveal li {
            margin-bottom: 0.5em;
        }
    </style>
</head>
<body>
"#;

const REVEALJS_FOOTER: &str = r#"    <script src="https://cdn.jsdelivr.net/npm/reveal.js@4.5.0/dist/reveal.js"></script>
    <script>
        Reveal.initialize({
            hash: true,
            slideNumber: 'c/t',
            showSlideNumber: 'all',
            transition: 'slide',
            width: 1200,
            height: 700,
            margin: 0.04
        });
    </script>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::PureContext;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{
        NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeValue,
    };
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
        assert!(output.contains("<title>Presentation</title>"));
    }

    #[test]
    fn test_revealjs_extract_title() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Slide Title".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let title = extract_title(&arena, root);
        assert_eq!(title, Some("Slide Title".to_string()));
    }

    #[test]
    fn test_revealjs_extract_title_no_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Just text".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let title = extract_title(&arena, root);
        assert_eq!(title, None);
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

    #[test]
    fn test_html_escape() {
        assert_eq!(escape_html("hello"), "hello");
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quote\""), "&quot;quote&quot;");
        assert_eq!(escape_html("it's"), "it&#x27;s");
    }

    #[test]
    fn test_escape_html_to() {
        let mut output = String::new();
        escape_html_to("<>&\"'", &mut output);
        assert_eq!(output, "&lt;&gt;&amp;&quot;&#x27;");
    }
}
