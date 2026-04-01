//! BibTeX document writer.
//!
//! This module provides a writer for BibTeX format.
//!
//! # Example
//!
//! ```ignore
//! use clmd::writers::BibTeXWriter;
//! use clmd::options::WriterOptions;
//! use clmd::context::PureContext;
//!
//! let writer = BibTeXWriter;
//! let ctx = PureContext::new();
//! let output = writer.write(&arena, root, &ctx, &WriterOptions::default()).unwrap();
//! ```

use crate::context::ClmdContext;
use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::core::nodes::NodeValue;
use crate::options::{OutputFormat, WriterOptions};
use crate::writers::Writer;

/// BibTeX document writer.
#[derive(Debug, Clone, Copy)]
pub struct BibTeXWriter;

impl Writer for BibTeXWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        _options: &WriterOptions,
    ) -> ClmdResult<String> {
        let mut output = String::new();
        render_node(arena, root, &mut output)?;
        Ok(output)
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Bibtex
    }

    fn extensions(&self) -> &[&'static str] {
        &["bib", "bibtex"]
    }

    fn mime_type(&self) -> &'static str {
        "text/x-bibtex"
    }
}

/// Render a node and its children to BibTeX.
fn render_node(arena: &NodeArena, node_id: NodeId, output: &mut String) -> ClmdResult<()> {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Document => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }

        NodeValue::Heading(_heading) => {
            // Get the heading text from children
            let mut title = String::new();
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                collect_text(arena, child_id, &mut title)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            if !title.is_empty() {
                // Create a comment for the heading
                output.push_str("% ");
                output.push_str(&title);
                output.push('\n');
            }
        }

        NodeValue::Paragraph => {
            let mut text = String::new();
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, &mut text)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            if !text.is_empty() {
                output.push_str("% ");
                output.push_str(&text);
                output.push('\n');
            }
        }

        NodeValue::CodeBlock(code) => {
            // Try to parse as BibTeX entry
            output.push_str(&code.literal);
            output.push('\n');
            output.push('\n');
        }

        NodeValue::List(_) => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }

        NodeValue::Item(_) => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }

        NodeValue::Text(text) => {
            output.push_str(text);
        }

        _ => {
            // For other node types, render children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }
    }

    Ok(())
}

/// Render inline content.
fn render_inline(arena: &NodeArena, node_id: NodeId, output: &mut String) -> ClmdResult<()> {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Text(text) => {
            output.push_str(text);
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
            output.push_str(&code.literal);
            output.push('}');
        }

        NodeValue::Link(link) => {
            output.push_str("\\url{");
            output.push_str(&link.url);
            output.push('}');
        }

        _ => {
            // For other inline types, collect text from children
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

/// Collect text content from a node and its children.
fn collect_text(arena: &NodeArena, node_id: NodeId, output: &mut String) -> ClmdResult<()> {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Text(text) => {
            output.push_str(text);
        }
        NodeValue::SoftBreak | NodeValue::HardBreak => {
            output.push(' ');
        }
        _ => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                collect_text(arena, child_id, output)?;
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
    use crate::core::arena::{Node, NodeArena};
    use crate::core::nodes::NodeValue;
    use crate::options::WriterOptions;

    #[test]
    fn test_bibtex_writer_basic() {
        let writer = BibTeXWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.is_empty() || output == "\n");
    }

    #[test]
    fn test_bibtex_writer_format() {
        let writer = BibTeXWriter;
        assert_eq!(writer.format(), OutputFormat::Bibtex);
        assert!(writer.extensions().contains(&"bib"));
        assert!(writer.extensions().contains(&"bibtex"));
    }
}
