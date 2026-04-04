//! Beamer document writer.
//!
//! This module provides a writer for Beamer (LaTeX slides) format,
//! using the shared LaTeX rendering core with Beamer-specific extensions.
//!
//! The Beamer writer extends the shared LaTeX core with:
//! - Slide structure conversion (frames)
//! - Title slide generation
//! - Section/subsection handling
//! - Support for fragile frames (for code blocks)
//!
//! # Example
//!
//! ```ignore
//! use clmd::io::writer::BeamerWriter;
//! use clmd::options::WriterOptions;
//! use clmd::context::PureContext;
//!
//! let writer = BeamerWriter;
//! let ctx = PureContext::new();
//! let options = WriterOptions::default();
//! let output = writer.write(&arena, root, &ctx, &options).unwrap();
//! ```

use crate::context::ClmdContext;
use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::core::nodes::NodeValue;
use crate::io::writer::latex_shared::{escape_latex, generate_preamble, LatexState};
use crate::io::writer::shared::extract_title;
use crate::io::writer::Writer;
use crate::options::{OutputFormat, WriterOptions};

/// Beamer document writer.
///
/// Renders documents to Beamer (LaTeX slides) format with a complete
/// document structure including frames and title slides.
#[derive(Debug, Clone, Copy)]
pub struct BeamerWriter;

impl Writer for BeamerWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        _options: &WriterOptions,
    ) -> ClmdResult<String> {
        let mut output = String::new();

        // Generate Beamer preamble
        let state = LatexState::beamer();
        output.push_str(&generate_preamble(&state));

        // Document content
        output.push_str("\\begin{document}\n\n");

        // Generate title slide if there's a level 1 heading
        if let Some(title) = extract_title(arena, root) {
            output.push_str("\\title{");
            output.push_str(&escape_latex(&title));
            output.push_str("}\n");
            output.push_str("\\author{}\n");
            output.push_str("\\date{\\today}\n");
            output.push_str("\\maketitle\n\n");
        }

        // Render slides
        render_slides(arena, root, &mut output)?;

        output.push_str("\\end{document}\n");

        Ok(output)
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Beamer
    }

    fn extensions(&self) -> &[&'static str] {
        &["tex", "beamer"]
    }

    fn mime_type(&self) -> &'static str {
        "text/x-tex"
    }
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
            // Level 1 heading starts a new section
            if heading.level == 1 {
                let mut title = String::new();
                let mut text_opt = node.first_child;
                while let Some(text_id) = text_opt {
                    let text_node = arena.get(text_id);
                    if let NodeValue::Text(t) = &text_node.value {
                        title.push_str(t);
                    }
                    text_opt = text_node.next;
                }

                output.push_str("\\section{");
                output.push_str(&escape_latex(&title));
                output.push_str("}\n\n");
            } else {
                // Level 2+ headings become frame titles
                output.push_str("\\begin{frame}\n");

                let mut title = String::new();
                let mut text_opt = node.first_child;
                while let Some(text_id) = text_opt {
                    let text_node = arena.get(text_id);
                    if let NodeValue::Text(t) = &text_node.value {
                        title.push_str(t);
                    }
                    text_opt = text_node.next;
                }

                output.push_str("\\frametitle{");
                output.push_str(&escape_latex(&title));
                output.push_str("}\n\n");

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

                output.push_str("\\end{frame}\n\n");
            }
        }

        NodeValue::Paragraph => {
            // Paragraphs outside frames become their own frames
            output.push_str("\\begin{frame}\n");
            render_slide_content(arena, node_id, output)?;
            output.push_str("\\end{frame}\n\n");
        }

        NodeValue::List(_) => {
            // Lists outside frames become their own frames
            output.push_str("\\begin{frame}\n");
            render_slide_content(arena, node_id, output)?;
            output.push_str("\\end{frame}\n\n");
        }

        NodeValue::BlockQuote => {
            // Blockquotes outside frames become their own frames
            output.push_str("\\begin{frame}\n");
            render_slide_content(arena, node_id, output)?;
            output.push_str("\\end{frame}\n\n");
        }

        NodeValue::CodeBlock(code) => {
            // Code blocks become their own frames with fragile option
            output.push_str("\\begin{frame}[fragile]\n");
            output.push_str("\\begin{verbatim}\n");
            output.push_str(&code.literal);
            output.push_str("\n\\end{verbatim}\n");
            output.push_str("\\end{frame}\n\n");
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

/// Render content inside a slide frame.
fn render_slide_content(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut String,
) -> ClmdResult<()> {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Paragraph => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("\n\n");
        }

        NodeValue::List(list) => {
            match list.list_type {
                crate::core::nodes::ListType::Bullet => {
                    output.push_str("\\begin{itemize}\n");
                }
                crate::core::nodes::ListType::Ordered => {
                    output.push_str("\\begin{enumerate}\n");
                }
            }

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                let child = arena.get(child_id);
                if matches!(child.value, NodeValue::Item(_)) {
                    output.push_str("\\item ");
                    let mut item_child_opt = child.first_child;
                    while let Some(item_child_id) = item_child_opt {
                        render_inline(arena, item_child_id, output)?;
                        let item_child = arena.get(item_child_id);
                        item_child_opt = item_child.next;
                    }
                    output.push('\n');
                }
                child_opt = child.next;
            }

            match list.list_type {
                crate::core::nodes::ListType::Bullet => {
                    output.push_str("\\end{itemize}\n\n");
                }
                crate::core::nodes::ListType::Ordered => {
                    output.push_str("\\end{enumerate}\n\n");
                }
            }
        }

        NodeValue::BlockQuote => {
            output.push_str("\\begin{quote}\n");

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_slide_content(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("\\end{quote}\n\n");
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
            output.push_str(&escape_latex(text));
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
            output.push_str(&escape_latex(&code.literal));
            output.push('}');
        }

        NodeValue::Link(link) => {
            output.push_str("\\href{");
            output.push_str(&escape_latex(&link.url));
            output.push_str("}{");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push('}');
        }

        NodeValue::Strikethrough => {
            output.push_str("\\sout{");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push('}');
        }

        NodeValue::Underline => {
            output.push_str("\\underline{");
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push('}');
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::PureContext;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{
        NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeValue,
    };

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
    fn test_beamer_writer_basic() {
        let writer = BeamerWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let (arena, root) = create_test_presentation();

        let output = writer.write(&arena, root, &ctx, &options).unwrap();

        // Check for Beamer structure
        assert!(output.contains("\\documentclass"));
        assert!(output.contains("\\begin{document}"));
        assert!(output.contains("\\end{document}"));
        assert!(output.contains("\\begin{frame}"));
        assert!(output.contains("\\end{frame}"));
    }

    #[test]
    fn test_beamer_writer_format() {
        let writer = BeamerWriter;
        assert_eq!(writer.format(), OutputFormat::Beamer);
        assert!(writer.extensions().contains(&"tex"));
        assert!(writer.extensions().contains(&"beamer"));
        assert_eq!(writer.mime_type(), "text/x-tex");
    }

    #[test]
    fn test_beamer_title_extraction() {
        let (arena, root) = create_test_presentation();
        let title = extract_title(&arena, root);
        assert_eq!(title, Some("My Presentation".to_string()));
    }

    #[test]
    fn test_beamer_empty_document() {
        let writer = BeamerWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("\\documentclass"));
        assert!(output.contains("\\begin{document}"));
        assert!(output.contains("\\end{document}"));
    }

    #[test]
    fn test_beamer_title_slide() {
        let writer = BeamerWriter;
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
        assert!(output.contains("\\title{Title Slide}"));
        assert!(output.contains("\\maketitle"));
    }

    #[test]
    fn test_beamer_section() {
        let writer = BeamerWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Section Name".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("\\section{Section Name}"));
    }

    #[test]
    fn test_beamer_frame_with_frametitle() {
        let writer = BeamerWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Frame Title".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("\\begin{frame}"));
        assert!(output.contains("\\frametitle{Frame Title}"));
        assert!(output.contains("\\end{frame}"));
    }

    #[test]
    fn test_beamer_paragraph_frame() {
        let writer = BeamerWriter;
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
        assert!(output.contains("\\begin{frame}"));
        assert!(output.contains("Paragraph content"));
        assert!(output.contains("\\end{frame}"));
    }

    #[test]
    fn test_beamer_code_block_frame() {
        let writer = BeamerWriter;
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
        assert!(output.contains("\\begin{frame}[fragile]"));
        assert!(output.contains("\\begin{verbatim}"));
        assert!(output.contains("fn main() {}"));
        assert!(output.contains("\\end{verbatim}"));
        assert!(output.contains("\\end{frame}"));
    }

    #[test]
    fn test_beamer_emphasis() {
        let writer = BeamerWriter;
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
        assert!(output.contains("\\emph{italic}"));
    }

    #[test]
    fn test_beamer_strong() {
        let writer = BeamerWriter;
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
        assert!(output.contains("\\textbf{bold}"));
    }

    #[test]
    fn test_beamer_code_inline() {
        let writer = BeamerWriter;
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
        assert!(output.contains("\\texttt{code}"));
    }

    #[test]
    fn test_beamer_link() {
        let writer = BeamerWriter;
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
        assert!(output.contains("\\href{https://example.com}{click}"));
    }

    #[test]
    fn test_beamer_strikethrough() {
        let writer = BeamerWriter;
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
        assert!(output.contains("\\sout{deleted}"));
    }

    #[test]
    fn test_beamer_underline() {
        let writer = BeamerWriter;
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
        assert!(output.contains("\\underline{underlined}"));
    }

    #[test]
    fn test_beamer_blockquote() {
        let writer = BeamerWriter;
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
        assert!(output.contains("\\begin{frame}"));
        assert!(output.contains("\\begin{quote}"));
        assert!(output.contains("Quoted text"));
        assert!(output.contains("\\end{quote}"));
        assert!(output.contains("\\end{frame}"));
    }

    #[test]
    fn test_beamer_list() {
        let writer = BeamerWriter;
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
        assert!(output.contains("\\begin{frame}"));
        assert!(output.contains("\\begin{itemize}"));
        assert!(output.contains("\\item Item 1"));
        assert!(output.contains("\\end{itemize}"));
        assert!(output.contains("\\end{frame}"));
    }

    #[test]
    fn test_beamer_soft_break() {
        let writer = BeamerWriter;
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
    fn test_beamer_hard_break() {
        let writer = BeamerWriter;
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
        assert!(output.contains("Line1 Line2"));
    }

    #[test]
    fn test_beamer_nested_inline() {
        let writer = BeamerWriter;
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
        assert!(output.contains("\\textbf{\\emph{bold italic}}"));
    }
}
