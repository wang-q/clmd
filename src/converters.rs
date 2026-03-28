//! Document converters for exporting Markdown to various formats
//!
//! This module provides converters for exporting Markdown documents to
//! different formats like DOCX (Word) and PDF.

use crate::arena::{NodeArena, NodeId};
use crate::nodes::NodeValue;
use std::io::Write;

/// Export format options
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Microsoft Word format (.docx)
    Docx,
    /// PDF format (.pdf)
    Pdf,
}

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Page size (for PDF)
    pub page_size: PageSize,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            page_size: PageSize::A4,
        }
    }
}

/// Page size for PDF export
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    /// A4 page size
    A4,
    /// Letter page size
    Letter,
    /// Legal page size
    Legal,
}

/// Converter error
#[non_exhaustive]
#[derive(Debug)]
pub enum ConverterError {
    /// IO error
    Io(std::io::Error),
    /// Format error
    Format(String),
}

impl std::fmt::Display for ConverterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConverterError::Io(e) => write!(f, "IO error: {}", e),
            ConverterError::Format(s) => write!(f, "Format error: {}", s),
        }
    }
}

impl std::error::Error for ConverterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConverterError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ConverterError {
    fn from(e: std::io::Error) -> Self {
        ConverterError::Io(e)
    }
}

/// DOCX converter (placeholder implementation)
#[derive(Debug)]
pub struct DocxConverter {
    #[allow(dead_code)]
    options: ExportOptions,
}

impl DocxConverter {
    /// Create a new DOCX converter
    pub fn new(options: ExportOptions) -> Self {
        Self { options }
    }

    /// Export AST to DOCX format
    pub fn convert(
        &self,
        arena: &NodeArena,
        root: NodeId,
    ) -> Result<Vec<u8>, ConverterError> {
        let mut output = Vec::new();

        writeln!(
            output,
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        )?;
        writeln!(output, "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">")?;
        writeln!(output, "  <w:body>")?;

        self.convert_node_to_docx(arena, root, &mut output)?;

        writeln!(output, "  </w:body>")?;
        writeln!(output, "</w:document>")?;

        Ok(output)
    }

    fn convert_node_to_docx(
        &self,
        arena: &NodeArena,
        node_id: NodeId,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
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
                self.write_children_text(arena, node_id, output)?;
                writeln!(output, "</w:t></w:r>")?;
                writeln!(output, "    </w:p>")?;
            }
            NodeValue::Paragraph => {
                writeln!(output, "    <w:p>")?;
                writeln!(output, "      <w:r><w:t>")?;
                self.write_children_text(arena, node_id, output)?;
                writeln!(output, "</w:t></w:r>")?;
                writeln!(output, "    </w:p>")?;
            }
            _ => {
                self.write_children(arena, node_id, output)?;
            }
        }

        Ok(())
    }

    fn write_children(
        &self,
        arena: &NodeArena,
        node_id: NodeId,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let children = self.collect_children(arena, node_id);
        for child_id in children {
            self.convert_node_to_docx(arena, child_id, output)?;
        }
        Ok(())
    }

    fn write_children_text(
        &self,
        arena: &NodeArena,
        node_id: NodeId,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let children = self.collect_children(arena, node_id);
        for child_id in children {
            self.write_node_text(arena, child_id, output)?;
        }
        Ok(())
    }

    fn write_node_text(
        &self,
        arena: &NodeArena,
        node_id: NodeId,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let node = arena.get(node_id);
        match &node.value {
            NodeValue::Text(literal) => {
                let escaped = literal
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;");
                output.extend_from_slice(escaped.as_bytes());
            }
            _ => {
                self.write_children_text(arena, node_id, output)?;
            }
        }
        Ok(())
    }

    fn collect_children(&self, arena: &NodeArena, node_id: NodeId) -> Vec<NodeId> {
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
}

/// PDF converter (placeholder implementation)
#[derive(Debug)]
pub struct PdfConverter {
    #[allow(dead_code)]
    options: ExportOptions,
}

impl PdfConverter {
    /// Create a new PDF converter
    pub fn new(options: ExportOptions) -> Self {
        Self { options }
    }

    /// Export AST to PDF format
    pub fn convert(
        &self,
        arena: &NodeArena,
        root: NodeId,
    ) -> Result<Vec<u8>, ConverterError> {
        let mut output = Vec::new();

        writeln!(output, "%PDF-1.4")?;
        writeln!(
            output,
            "% Placeholder PDF - real implementation would use printpdf crate"
        )?;

        self.convert_node_to_pdf(arena, root, &mut output)?;

        writeln!(output, "%%EOF")?;

        Ok(output)
    }

    fn convert_node_to_pdf(
        &self,
        arena: &NodeArena,
        node_id: NodeId,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let node = arena.get(node_id);
        let value = &node.value;

        match value {
            NodeValue::Heading(heading) => {
                write!(output, "Heading {}: ", heading.level)?;
                self.write_children_text(arena, node_id, output)?;
                writeln!(output)?;
            }
            NodeValue::Paragraph => {
                self.write_children_text(arena, node_id, output)?;
                writeln!(output)?;
                writeln!(output)?;
            }
            _ => {
                self.write_children(arena, node_id, output)?;
            }
        }

        Ok(())
    }

    fn write_children(
        &self,
        arena: &NodeArena,
        node_id: NodeId,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let children = self.collect_children(arena, node_id);
        for child_id in children {
            self.convert_node_to_pdf(arena, child_id, output)?;
        }
        Ok(())
    }

    fn write_children_text(
        &self,
        arena: &NodeArena,
        node_id: NodeId,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let children = self.collect_children(arena, node_id);
        for child_id in children {
            self.write_node_text(arena, child_id, output)?;
        }
        Ok(())
    }

    fn write_node_text(
        &self,
        arena: &NodeArena,
        node_id: NodeId,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let node = arena.get(node_id);
        match &node.value {
            NodeValue::Text(literal) => {
                output.extend_from_slice(literal.as_bytes());
            }
            _ => {
                self.write_children_text(arena, node_id, output)?;
            }
        }
        Ok(())
    }

    fn collect_children(&self, arena: &NodeArena, node_id: NodeId) -> Vec<NodeId> {
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
}

/// Export a document to the specified format
pub fn export_document(
    arena: &NodeArena,
    root: NodeId,
    format: ExportFormat,
    options: ExportOptions,
) -> Result<Vec<u8>, ConverterError> {
    match format {
        ExportFormat::Docx => {
            let converter = DocxConverter::new(options);
            converter.convert(arena, root)
        }
        ExportFormat::Pdf => {
            let converter = PdfConverter::new(options);
            converter.convert(arena, root)
        }
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
    fn test_docx_converter() {
        let (arena, doc) = create_test_document();
        let options = ExportOptions::default();
        let converter = DocxConverter::new(options);

        let result = converter.convert(&arena, doc);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("<?xml version=\"1.0\""));
        assert!(output_str.contains("<w:document"));
        assert!(output_str.contains("Test Document"));
    }

    #[test]
    fn test_pdf_converter() {
        let (arena, doc) = create_test_document();
        let options = ExportOptions::default();
        let converter = PdfConverter::new(options);

        let result = converter.convert(&arena, doc);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("%PDF"));
        assert!(output_str.contains("Test Document"));
    }

    #[test]
    fn test_export_document() {
        let (arena, doc) = create_test_document();
        let options = ExportOptions::default();

        let docx_result =
            export_document(&arena, doc, ExportFormat::Docx, options.clone());
        assert!(docx_result.is_ok());

        let pdf_result = export_document(&arena, doc, ExportFormat::Pdf, options);
        assert!(pdf_result.is_ok());
    }

    #[test]
    fn test_export_options() {
        let options = ExportOptions {
            title: Some("My Doc".to_string()),
            author: Some("Author".to_string()),
            page_size: PageSize::Letter,
        };

        assert_eq!(options.title, Some("My Doc".to_string()));
        assert_eq!(options.author, Some("Author".to_string()));
        assert_eq!(options.page_size, PageSize::Letter);
    }
}
