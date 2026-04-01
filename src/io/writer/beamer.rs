//! Beamer document writer.
//!
//! This module provides a writer for Beamer (LaTeX slides) format.
//!
//! # Example
//!
//! ```ignore
//! use clmd::writers::BeamerWriter;
//! use clmd::options::WriterOptions;
//! use clmd::context::PureContext;
//!
//! let writer = BeamerWriter;
//! let ctx = PureContext::new();
//! let output = writer.write(&arena, root, &ctx, &WriterOptions::default()).unwrap();
//! ```

use crate::context::ClmdContext;
use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::core::nodes::NodeValue;
use crate::io::writer::Writer;
use crate::options::{OutputFormat, WriterOptions};

/// Beamer document writer.
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

        // Beamer preamble
        output.push_str(BEAMER_PREAMBLE);

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

/// Extract title from the first level 1 heading.
fn extract_title(arena: &NodeArena, root: NodeId) -> Option<String> {
    let root_node = arena.get(root);
    let mut child_opt = root_node.first_child;

    while let Some(child_id) = child_opt {
        let child = arena.get(child_id);
        if let NodeValue::Heading(heading) = &child.value {
            if heading.level == 1 {
                // Collect text from children
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
            // Code blocks become their own frames
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

        NodeValue::List(_) => {
            output.push_str("\\begin{itemize}\n");

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
                    output.push_str("\n");
                }
                child_opt = child.next;
            }

            output.push_str("\\end{itemize}\n\n");
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

/// Escape LaTeX special characters.
fn escape_latex(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);

    for c in text.chars() {
        match c {
            '\\' => result.push_str("\\textbackslash{}"),
            '{' => result.push_str("\\{"),
            '}' => result.push_str("\\}"),
            '$' => result.push_str("\\$"),
            '&' => result.push_str("\\&"),
            '#' => result.push_str("\\#"),
            '^' => result.push_str("\\^{}"),
            '_' => result.push_str("\\_"),
            '%' => result.push_str("\\%"),
            '~' => result.push_str("\\textasciitilde{}"),
            '<' => result.push_str("\\textless{}"),
            '>' => result.push_str("\\textgreater{}"),
            '|' => result.push_str("\\textbar{}"),
            '"' => result.push_str("\\textquotedbl{}"),
            '\'' => result.push_str("\\textquotesingle{}"),
            '`' => result.push_str("\\textasciigrave{}"),
            _ => result.push(c),
        }
    }

    result
}

/// Beamer document preamble.
const BEAMER_PREAMBLE: &str = r#"\documentclass[aspectratio=169]{beamer}

\usetheme{Madrid}
\usecolortheme{default}

\usepackage[utf8]{inputenc}
\usepackage[T1]{fontenc}
\usepackage{hyperref}
\usepackage{ulem}

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
    }

    #[test]
    fn test_beamer_title_extraction() {
        let (arena, root) = create_test_presentation();
        let title = extract_title(&arena, root);
        assert_eq!(title, Some("My Presentation".to_string()));
    }

    #[test]
    fn test_latex_escape() {
        assert_eq!(escape_latex("hello"), "hello");
        assert_eq!(escape_latex("$100"), "\\$100");
        assert_eq!(escape_latex("10%"), "10\\%");
        assert_eq!(escape_latex("a_b"), "a\\_b");
    }
}
