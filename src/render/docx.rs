//! DOCX renderer (placeholder implementation)
//!
//! This module provides DOCX (Microsoft Word) output generation.
//! Note: This is a placeholder implementation. Full DOCX support would require
//! the docx-rs crate for proper ZIP packaging and XML relationships.

use crate::arena::{NodeArena, NodeId};
use crate::nodes::NodeValue;
use std::io::Write;

/// DOCX export options
#[derive(Debug, Clone, Default)]
pub struct DocxOptions {
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
}

/// Render an AST as DOCX
///
/// This is a convenience function that generates a simple Word XML structure.
/// Note: This is a placeholder implementation.
pub fn render(arena: &NodeArena, root: NodeId, _options: u32) -> String {
    let mut renderer = DocxRenderer::new(arena);
    renderer.render(root)
}

/// Format an AST as DOCX with options
///
/// This is a placeholder implementation that generates Word XML.
pub fn format_document(
    arena: &NodeArena,
    root: NodeId,
    options: &DocxOptions,
    output: &mut dyn Write,
) -> std::io::Result<()> {
    writeln!(
        output,
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
    )?;
    writeln!(output, "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">")?;
    writeln!(output, "  <w:body>")?;

    format_node_docx(arena, root, options, output)?;

    writeln!(output, "  </w:body>")?;
    writeln!(output, "</w:document>")?;

    Ok(())
}

fn format_node_docx(
    arena: &NodeArena,
    node_id: NodeId,
    options: &DocxOptions,
    output: &mut dyn Write,
) -> std::io::Result<()> {
    let node = arena.get(node_id);
    let value = &node.value;

    match value {
        NodeValue::Heading(heading) => {
            writeln!(output, "    <w:p>")?;
            writeln!(
                output,
                "      <w:pPr><w:pStyle w:val=\"Heading{}\"/></w:pPr>",
                heading.level
            )?;
            writeln!(output, "      <w:r><w:t>")?;
            write_children_text(arena, node_id, output)?;
            writeln!(output, "</w:t></w:r>")?;
            writeln!(output, "    </w:p>")?;
        }
        NodeValue::Paragraph => {
            writeln!(output, "    <w:p>")?;
            writeln!(output, "      <w:r><w:t>")?;
            write_children_text(arena, node_id, output)?;
            writeln!(output, "</w:t></w:r>")?;
            writeln!(output, "    </w:p>")?;
        }
        _ => {
            write_children(arena, node_id, options, output)?;
        }
    }

    Ok(())
}

fn write_children(
    arena: &NodeArena,
    node_id: NodeId,
    options: &DocxOptions,
    output: &mut dyn Write,
) -> std::io::Result<()> {
    let children = collect_children(arena, node_id);
    for child_id in children {
        format_node_docx(arena, child_id, options, output)?;
    }
    Ok(())
}

fn write_children_text(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut dyn Write,
) -> std::io::Result<()> {
    let children = collect_children(arena, node_id);
    for child_id in children {
        write_node_text(arena, child_id, output)?;
    }
    Ok(())
}

fn write_node_text(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut dyn Write,
) -> std::io::Result<()> {
    let node = arena.get(node_id);
    match &node.value {
        NodeValue::Text(literal) => {
            let escaped = literal
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;");
            output.write_all(escaped.as_bytes())?;
        }
        _ => {
            write_children_text(arena, node_id, output)?;
        }
    }
    Ok(())
}

fn collect_children(arena: &NodeArena, node_id: NodeId) -> Vec<NodeId> {
    let mut children = Vec::new();

    let first_opt = arena.get(node_id).first_child;
    if let Some(first) = first_opt {
        children.push(first);
        let mut current = first;
        loop {
            let next_opt = arena.get(current).next;
            if let Some(next) = next_opt {
                children.push(next);
                current = next;
            } else {
                break;
            }
        }
    }

    children
}

/// DOCX renderer state
struct DocxRenderer<'a> {
    arena: &'a NodeArena,
    output: String,
}

impl<'a> DocxRenderer<'a> {
    fn new(arena: &'a NodeArena) -> Self {
        DocxRenderer {
            arena,
            output: String::new(),
        }
    }

    fn render(&mut self, root: NodeId) -> String {
        self.output
            .push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        self.output.push_str(
            "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\n",
        );
        self.output.push_str("  <w:body>\n");

        self.render_node(root);

        self.output.push_str("  </w:body>\n");
        self.output.push_str("</w:document>\n");

        self.output.clone()
    }

    fn render_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        let value = &node.value;

        match value {
            NodeValue::Heading(heading) => {
                self.output.push_str("    <w:p>\n");
                self.output.push_str(&format!(
                    "      <w:pPr><w:pStyle w:val=\"Heading{}\"/></w:pPr>\n",
                    heading.level
                ));
                self.output.push_str("      <w:r><w:t>");
                self.write_children_text(node_id);
                self.output.push_str("</w:t></w:r>\n");
                self.output.push_str("    </w:p>\n");
            }
            NodeValue::Paragraph => {
                self.output.push_str("    <w:p>\n");
                self.output.push_str("      <w:r><w:t>");
                self.write_children_text(node_id);
                self.output.push_str("</w:t></w:r>\n");
                self.output.push_str("    </w:p>\n");
            }
            _ => {
                self.write_children(node_id);
            }
        }
    }

    fn write_children(&mut self, node_id: NodeId) {
        let children = self.collect_children(node_id);
        for child_id in children {
            self.render_node(child_id);
        }
    }

    fn write_children_text(&mut self, node_id: NodeId) {
        let children = self.collect_children(node_id);
        for child_id in children {
            self.write_node_text(child_id);
        }
    }

    fn write_node_text(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        match &node.value {
            NodeValue::Text(literal) => {
                let escaped = literal
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;");
                self.output.push_str(&escaped);
            }
            _ => {
                self.write_children_text(node_id);
            }
        }
    }

    fn collect_children(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut children = Vec::new();

        let first_opt = self.arena.get(node_id).first_child;
        if let Some(first) = first_opt {
            children.push(first);
            let mut current = first;
            loop {
                let next_opt = self.arena.get(current).next;
                if let Some(next) = next_opt {
                    children.push(next);
                    current = next;
                } else {
                    break;
                }
            }
        }

        children
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};

    fn create_test_document() -> (NodeArena, NodeId) {
        let mut arena = NodeArena::new();
        let doc = arena.alloc(Node::with_value(NodeValue::Document));

        let heading = arena.alloc(Node::with_value(NodeValue::Heading(
            crate::nodes::NodeHeading {
                level: 1,
                setext: false,
                closed: false,
            },
        )));

        let text = arena.alloc(Node::with_value(NodeValue::make_text("Test Document")));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, doc, heading);

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        let para_text = arena.alloc(Node::with_value(NodeValue::make_text(
            "This is a test paragraph.",
        )));
        TreeOps::append_child(&mut arena, para, para_text);
        TreeOps::append_child(&mut arena, doc, para);

        (arena, doc)
    }

    #[test]
    fn test_docx_render() {
        let (arena, doc) = create_test_document();
        let output = render(&arena, doc, 0);

        assert!(output.contains("<?xml version=\"1.0\""));
        assert!(output.contains("<w:document"));
        assert!(output.contains("Test Document"));
    }

    #[test]
    fn test_docx_format_document() {
        let (arena, doc) = create_test_document();
        let options = DocxOptions::default();
        let mut output = Vec::new();

        format_document(&arena, doc, &options, &mut output).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("<?xml version=\"1.0\""));
        assert!(output_str.contains("<w:document"));
        assert!(output_str.contains("Test Document"));
    }
}
