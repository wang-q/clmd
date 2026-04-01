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
use crate::options::{OutputFormat, WriterOptions};
use crate::io::writer::Writer;

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
    use crate::core::nodes::{NodeHeading, NodeValue};
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
        let text = arena.alloc(Node::with_value(NodeValue::Text(
            "My Presentation".to_string().into_boxed_str(),
        )));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        // Add a slide heading
        let slide = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));
        let slide_text = arena.alloc(Node::with_value(NodeValue::Text(
            "First Slide".to_string().into_boxed_str(),
        )));
        TreeOps::append_child(&mut arena, slide, slide_text);
        TreeOps::append_child(&mut arena, root, slide);

        // Add a paragraph
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para_text = arena.alloc(Node::with_value(NodeValue::Text(
            "Hello World".to_string().into_boxed_str(),
        )));
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
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(escape_html("hello"), "hello");
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quote\""), "&quot;quote&quot;");
    }
}
