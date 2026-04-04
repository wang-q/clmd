//! Shared HTML renderer for both standard HTML and Reveal.js output.
//!
//! This module provides a unified HTML rendering system inspired by Pandoc's
//! LaTeX/Beamer architecture. It uses a state flag to differentiate between
//! standard HTML and Reveal.js slide output.
//!
//! # Design
//!
//! - **Shared Core**: Both HTML and Reveal.js share the same rendering logic
//! - **State-Driven**: `is_revealjs` flag controls format-specific behavior
//! - **Minimal Duplication**: Common code is shared, only differences are branched
//!
//! # Example
//!
//! ```ignore
//! use clmd::io::writer::html_renderer::{HtmlRenderer, HtmlMode};
//!
//! // Standard HTML mode
//! let html_output = HtmlRenderer::render_html(&arena, root, &options);
//!
//! // Reveal.js mode
//! let revealjs_output = HtmlRenderer::render_revealjs(&arena, root, &options);
//! ```

use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::core::nodes::NodeValue;
use crate::io::writer::shared::extract_title;
use crate::options::WriterOptions;

/// HTML rendering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlMode {
    /// Standard HTML output.
    Standard,
    /// Reveal.js slide presentation output.
    RevealJs,
}

impl HtmlMode {
    /// Check if this is Reveal.js mode.
    pub fn is_revealjs(&self) -> bool {
        matches!(self, HtmlMode::RevealJs)
    }

    /// Get the mode name.
    pub fn as_str(&self) -> &'static str {
        match self {
            HtmlMode::Standard => "html",
            HtmlMode::RevealJs => "revealjs",
        }
    }
}

/// Shared HTML renderer that supports both standard HTML and Reveal.js.
///
/// This renderer uses a mode flag to control format-specific behavior,
/// similar to Pandoc's `stBeamer` flag for LaTeX/Beamer.
#[derive(Debug)]
pub struct HtmlRenderer<'a> {
    arena: &'a NodeArena,
    #[allow(dead_code)]
    options: &'a WriterOptions,
    mode: HtmlMode,
    output: String,
    slide_level: usize,
}

impl<'a> HtmlRenderer<'a> {
    /// Create a new HTML renderer with the specified mode.
    pub fn new(
        arena: &'a NodeArena,
        options: &'a WriterOptions,
        mode: HtmlMode,
    ) -> Self {
        Self {
            arena,
            options,
            mode,
            output: String::with_capacity(arena.len() * 64),
            slide_level: 2, // Default: h1 = title slide, h2+ = content slides
        }
    }

    /// Set the slide level for Reveal.js mode.
    ///
    /// Headings at this level or higher start new slides.
    /// Default is 2 (h1 = title, h2+ = slides).
    pub fn with_slide_level(mut self, level: usize) -> Self {
        self.slide_level = level;
        self
    }

    /// Render the document and return the output.
    pub fn render(mut self, root: NodeId) -> ClmdResult<String> {
        match self.mode {
            HtmlMode::Standard => self.render_standard(root),
            HtmlMode::RevealJs => self.render_revealjs(root),
        }
    }

    /// Render in standard HTML mode.
    fn render_standard(&mut self, root: NodeId) -> ClmdResult<String> {
        self.write_preamble(None);
        self.render_body(root)?;
        self.write_footer();
        Ok(self.output.clone())
    }

    /// Render in Reveal.js mode.
    fn render_revealjs(&mut self, root: NodeId) -> ClmdResult<String> {
        let title = extract_title(self.arena, root);
        self.write_preamble(title.as_deref());
        self.write_revealjs_head();
        self.output.push_str("<body>\n");
        self.write_revealjs_setup();
        self.render_slides(root)?;
        self.write_revealjs_footer();
        self.write_footer();
        Ok(self.output.clone())
    }

    /// Write the HTML preamble (DOCTYPE, head start).
    fn write_preamble(&mut self, title: Option<&str>) {
        self.output.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
"#);

        if let Some(t) = title {
            self.output.push_str(&format!("    <title>{}</title>\n", escape_html(t)));
        }
    }

    /// Write the head end and body start for standard HTML.
    fn write_standard_head_end(&mut self) {
        self.output.push_str("</head>\n<body>\n");
    }

    /// Write Reveal.js specific head content.
    fn write_revealjs_head(&mut self) {
        self.output.push_str(r#"    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/reveal.js@4.5.0/dist/reveal.css">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/reveal.js@4.5.0/dist/theme/white.css">
    <style>
        .reveal h1, .reveal h2, .reveal h3 {
            text-transform: none;
        }
        .reveal section img {
            border: none;
            box-shadow: none;
        }
    </style>
</head>
"#);
    }

    /// Write Reveal.js setup HTML.
    fn write_revealjs_setup(&mut self) {
        self.output.push_str(r#"<div class="reveal">
    <div class="slides">
"#);
    }

    /// Write the standard footer.
    fn write_footer(&mut self) {
        self.output.push_str("</body>\n</html>\n");
    }

    /// Write Reveal.js footer with initialization script.
    fn write_revealjs_footer(&mut self) {
        self.output.push_str(r#"    </div>
</div>
<script src="https://cdn.jsdelivr.net/npm/reveal.js@4.5.0/dist/reveal.js"></script>
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
"#);
    }

    /// Render the document body in standard HTML mode.
    fn render_body(&mut self, root: NodeId) -> ClmdResult<()> {
        self.write_standard_head_end();

        let root_node = self.arena.get(root);
        let mut child_opt = root_node.first_child;

        while let Some(child_id) = child_opt {
            self.render_block(child_id)?;
            let child = self.arena.get(child_id);
            child_opt = child.next;
        }

        Ok(())
    }

    /// Render content as slides for Reveal.js.
    fn render_slides(&mut self, root: NodeId) -> ClmdResult<()> {
        let root_node = self.arena.get(root);

        // Collect top-level sections (h1, h2, etc.)
        let mut slides: Vec<Vec<NodeId>> = Vec::new();
        let mut current_slide: Vec<NodeId> = Vec::new();

        let mut child_opt = root_node.first_child;
        while let Some(child_id) = child_opt {
            let child = self.arena.get(child_id);

            // Check if this is a heading that starts a new slide
            let is_slide_boundary = if let NodeValue::Heading(h) = &child.value {
                h.level as usize >= self.slide_level
            } else {
                false
            };

            if is_slide_boundary && !current_slide.is_empty() {
                slides.push(current_slide);
                current_slide = Vec::new();
            }

            current_slide.push(child_id);
            child_opt = child.next;
        }

        if !current_slide.is_empty() {
            slides.push(current_slide);
        }

        // Render each slide
        for slide_content in slides {
            self.output.push_str("        <section>\n");
            for node_id in slide_content {
                self.render_slide_content(node_id)?;
            }
            self.output.push_str("        </section>\n");
        }

        Ok(())
    }

    /// Render content within a slide.
    fn render_slide_content(&mut self, node_id: NodeId) -> ClmdResult<()> {
        let node = self.arena.get(node_id);

        match &node.value {
            NodeValue::Heading(h) => {
                let tag = format!("h{}", h.level);
                self.output.push_str(&format!("            <{}>", tag));

                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }

                self.output.push_str(&format!("</{}>\n", tag));
            }

            NodeValue::Paragraph => {
                self.output.push_str("            <p>");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }
                self.output.push_str("</p>\n");
            }

            NodeValue::List(_) => {
                self.render_list(node_id, 3)?;
            }

            NodeValue::BlockQuote => {
                self.output.push_str("            <blockquote>\n");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_slide_content(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }
                self.output.push_str("            </blockquote>\n");
            }

            NodeValue::CodeBlock(code) => {
                self.output.push_str("            <pre><code>");
                escape_html_to(&code.literal, &mut self.output);
                self.output.push_str("</code></pre>\n");
            }

            _ => {
                // For other nodes, render children
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_slide_content(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }
            }
        }

        Ok(())
    }

    /// Render a block element.
    fn render_block(&mut self, node_id: NodeId) -> ClmdResult<()> {
        let node = self.arena.get(node_id);

        match &node.value {
            NodeValue::Heading(h) => {
                let tag = format!("h{}", h.level);
                self.output.push_str(&format!("<{}>", tag));

                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }

                self.output.push_str(&format!("</{}>\n", tag));
            }

            NodeValue::Paragraph => {
                self.output.push_str("<p>");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }
                self.output.push_str("</p>\n");
            }

            NodeValue::List(_) => {
                self.render_list(node_id, 0)?;
            }

            NodeValue::BlockQuote => {
                self.output.push_str("<blockquote>\n");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_block(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }
                self.output.push_str("</blockquote>\n");
            }

            NodeValue::CodeBlock(code) => {
                self.output.push_str("<pre><code>");
                escape_html_to(&code.literal, &mut self.output);
                self.output.push_str("</code></pre>\n");
            }

            NodeValue::ThematicBreak(_) => {
                self.output.push_str("<hr/>\n");
            }

            _ => {
                // For other nodes, render children
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_block(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }
            }
        }

        Ok(())
    }

    /// Render a list.
    fn render_list(&mut self, node_id: NodeId, indent: usize) -> ClmdResult<()> {
        let node = self.arena.get(node_id);
        let indent_str = " ".repeat(indent);

        self.output.push_str(&format!("{}<ul>\n", indent_str));

        let mut child_opt = node.first_child;
        while let Some(child_id) = child_opt {
            let child = self.arena.get(child_id);
            if matches!(child.value, NodeValue::Item(_)) {
                self.output.push_str(&format!("{}    <li>", indent_str));
                let mut item_child_opt = child.first_child;
                while let Some(item_child_id) = item_child_opt {
                    self.render_inline(item_child_id)?;
                    let item_child = self.arena.get(item_child_id);
                    item_child_opt = item_child.next;
                }
                self.output.push_str("</li>\n");
            }
            child_opt = child.next;
        }

        self.output.push_str(&format!("{}</ul>\n", indent_str));
        Ok(())
    }

    /// Render inline content.
    fn render_inline(&mut self, node_id: NodeId) -> ClmdResult<()> {
        let node = self.arena.get(node_id);

        match &node.value {
            NodeValue::Text(text) => {
                escape_html_to(text, &mut self.output);
            }

            NodeValue::SoftBreak => {
                self.output.push(' ');
            }

            NodeValue::HardBreak => {
                self.output.push_str("<br/>");
            }

            NodeValue::Emph => {
                self.output.push_str("<em>");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }
                self.output.push_str("</em>");
            }

            NodeValue::Strong => {
                self.output.push_str("<strong>");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }
                self.output.push_str("</strong>");
            }

            NodeValue::Code(code) => {
                self.output.push_str("<code>");
                escape_html_to(&code.literal, &mut self.output);
                self.output.push_str("</code>");
            }

            NodeValue::Link(link) => {
                self.output.push_str(&format!(r#"<a href="{}">"#, escape_html(&link.url)));
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }
                self.output.push_str("</a>");
            }

            NodeValue::Strikethrough => {
                self.output.push_str("<del>");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }
                self.output.push_str("</del>");
            }

            NodeValue::Underline => {
                self.output.push_str("<u>");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }
                self.output.push_str("</u>");
            }

            NodeValue::Image(link) => {
                self.output.push_str(&format!(
                    r#"<img src="{}" alt="{}" />"#,
                    escape_html(&link.url),
                    escape_html(&link.title)
                ));
            }

            _ => {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id)?;
                    let child = self.arena.get(child_id);
                    child_opt = child.next;
                }
            }
        }

        Ok(())
    }

}

/// Escape HTML special characters.
fn escape_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    escape_html_to(text, &mut result);
    result
}

/// Escape HTML special characters to an existing output buffer.
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

/// Render a document to standard HTML.
///
/// This is a convenience function that creates an HtmlRenderer in Standard mode.
pub fn render_html(
    arena: &NodeArena,
    root: NodeId,
    options: &WriterOptions,
) -> ClmdResult<String> {
    HtmlRenderer::new(arena, options, HtmlMode::Standard).render(root)
}

/// Render a document to Reveal.js HTML.
///
/// This is a convenience function that creates an HtmlRenderer in RevealJs mode.
pub fn render_revealjs(
    arena: &NodeArena,
    root: NodeId,
    options: &WriterOptions,
) -> ClmdResult<String> {
    HtmlRenderer::new(arena, options, HtmlMode::RevealJs).render(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{NodeHeading, NodeValue};
    use crate::options::WriterOptions;

    fn create_test_document() -> (NodeArena, NodeId) {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // h1 heading
        let h1 = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let h1_text = arena.alloc(Node::with_value(NodeValue::Text("Title".into())));
        TreeOps::append_child(&mut arena, h1, h1_text);
        TreeOps::append_child(&mut arena, root, h1);

        // Paragraph
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para_text = arena.alloc(Node::with_value(NodeValue::Text("Hello world".into())));
        TreeOps::append_child(&mut arena, para, para_text);
        TreeOps::append_child(&mut arena, root, para);

        (arena, root)
    }

    #[test]
    fn test_html_mode() {
        assert!(!HtmlMode::Standard.is_revealjs());
        assert!(HtmlMode::RevealJs.is_revealjs());
        assert_eq!(HtmlMode::Standard.as_str(), "html");
        assert_eq!(HtmlMode::RevealJs.as_str(), "revealjs");
    }

    #[test]
    fn test_render_html() {
        let options = WriterOptions::default();
        let (arena, root) = create_test_document();

        let output = render_html(&arena, root, &options).unwrap();
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("<h1>Title</h1>"));
        assert!(output.contains("<p>Hello world</p>"));
        assert!(!output.contains("reveal.js"));
    }

    #[test]
    fn test_render_revealjs() {
        let options = WriterOptions::default();
        let (arena, root) = create_test_document();

        let output = render_revealjs(&arena, root, &options).unwrap();
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("reveal.js"));
        assert!(output.contains("<section>"));
        assert!(output.contains("Reveal.initialize"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
    }
}
