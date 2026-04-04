//! LaTeX writer.
//!
//! This module provides a writer for LaTeX format, using the shared LaTeX rendering core.
//!
//! # Example
//!
//! ```ignore
//! use clmd::io::writer::LatexWriter;
//! use clmd::options::WriterOptions;
//! use clmd::context::PureContext;
//!
//! let writer = LatexWriter;
//! let ctx = PureContext::new();
//! let options = WriterOptions::default();
//! let output = writer.write(&arena, root, &ctx, &options).unwrap();
//! ```

use crate::context::ClmdContext;
use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::io::writer::latex_shared::{
    generate_preamble, render_latex, LatexRenderer, LatexState,
};
use crate::io::writer::Writer;
use crate::options::{OutputFormat, WriterOptions};

/// Render a node tree as LaTeX.
///
/// This is a convenience function for simple LaTeX rendering without
/// the full document preamble.
pub fn render(arena: &NodeArena, root: NodeId, _options: u32) -> String {
    render_latex(arena, root)
}

/// Write a document as LaTeX.
///
/// This function generates a complete LaTeX document with preamble.
pub fn write_latex(
    arena: &NodeArena,
    root: NodeId,
    _options: &WriterOptions,
) -> ClmdResult<String> {
    let mut output = String::new();

    // Generate preamble
    let state = LatexState::new();
    output.push_str(&generate_preamble(&state));

    // Begin document
    output.push_str("\\begin{document}\n\n");

    // Render content
    let renderer = LatexRenderer::new_latex(arena);
    output.push_str(&renderer.render(root));

    // End document
    output.push_str("\n\\end{document}\n");

    Ok(output)
}

/// LaTeX document writer.
///
/// Renders documents to LaTeX format with a complete document structure.
#[derive(Debug, Clone, Copy)]
pub struct LatexWriter;

impl Writer for LatexWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        write_latex(arena, root, options)
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Latex
    }

    fn extensions(&self) -> &[&'static str] {
        &["tex", "latex"]
    }

    fn mime_type(&self) -> &'static str {
        "application/x-latex"
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

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("Hello world"));
    }

    #[test]
    fn test_render_emph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("emphasized")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, emph, text);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\emph{emphasized}"));
    }

    #[test]
    fn test_render_strong() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("strong")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, strong, text);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\textbf{strong}"));
    }

    #[test]
    fn test_render_code() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = arena.alloc(Node::with_value(NodeValue::Code(Box::new(NodeCode {
            num_backticks: 1,
            literal: "code".to_string(),
        }))));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\texttt{code}"));
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

        TreeOps::append_child(&mut arena, root, heading);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\subsection*{"));
    }

    #[test]
    fn test_render_link() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com".to_string(),
            title: "".to_string(),
        }))));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("link")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\href{https://example.com}{link}"));
    }

    #[test]
    fn test_render_blockquote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let blockquote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Quote")));

        TreeOps::append_child(&mut arena, root, blockquote);
        TreeOps::append_child(&mut arena, blockquote, para);
        TreeOps::append_child(&mut arena, para, text);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\begin{quote}"));
        assert!(latex.contains("\\end{quote}"));
    }

    #[test]
    fn test_render_code_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let code_block = arena.alloc(Node::with_value(NodeValue::CodeBlock(Box::new(
            NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            },
        ))));

        TreeOps::append_child(&mut arena, root, code_block);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\begin{verbatim}"));
        assert!(latex.contains("fn main() {}"));
        assert!(latex.contains("\\end{verbatim}"));
    }

    #[test]
    fn test_render_bullet_list() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: crate::core::nodes::ListType::Bullet,
            delimiter: crate::core::nodes::ListDelimType::Period,
            start: 1,
            tight: true,
            bullet_char: b'-',
            marker_offset: 0,
            padding: 2,
            is_task_list: false,
        })));
        let item = arena.alloc(Node::with_value(NodeValue::Item(NodeList::default())));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Item")));

        TreeOps::append_child(&mut arena, root, list);
        TreeOps::append_child(&mut arena, list, item);
        TreeOps::append_child(&mut arena, item, para);
        TreeOps::append_child(&mut arena, para, text);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\begin{itemize}"));
        assert!(latex.contains("\\item"));
        assert!(latex.contains("\\end{itemize}"));
    }

    #[test]
    fn test_latex_writer_trait() {
        let writer = LatexWriter;
        assert_eq!(writer.format(), OutputFormat::Latex);
        assert!(writer.extensions().contains(&"tex"));
        assert!(writer.extensions().contains(&"latex"));
        assert_eq!(writer.mime_type(), "application/x-latex");
    }

    #[test]
    fn test_latex_writer_write() {
        let writer = LatexWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("\\documentclass{article}"));
        assert!(output.contains("\\begin{document}"));
        assert!(output.contains("Hello"));
        assert!(output.contains("\\end{document}"));
    }

    #[test]
    fn test_write_latex_full_document() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Title")));

        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let options = WriterOptions::default();
        let output = write_latex(&arena, root, &options).unwrap();

        assert!(output.contains("\\documentclass{article}"));
        assert!(output.contains("\\usepackage[utf8]{inputenc}"));
        assert!(output.contains("\\begin{document}"));
        assert!(output.contains("\\section*{"));
        assert!(output.contains("\\end{document}"));
    }
}
