//! Document converters for exporting Markdown to various formats
//!
//! This module provides converters for exporting Markdown documents to
//! different formats like DOCX (Word) and PDF.

use crate::node::{Node, NodeData, NodeType};
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

/// Export format options
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    A4,
    Letter,
    Legal,
}

/// Converter error
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
    pub fn convert(&self, root: &Rc<RefCell<Node>>) -> Result<Vec<u8>, ConverterError> {
        let mut output = Vec::new();

        writeln!(
            output,
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        )?;
        writeln!(output, "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">")?;
        writeln!(output, "  <w:body>")?;

        self.convert_node_to_docx(root, &mut output)?;

        writeln!(output, "  </w:body>")?;
        writeln!(output, "</w:document>")?;

        Ok(output)
    }

    fn convert_node_to_docx(
        &self,
        node: &Rc<RefCell<Node>>,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let node_type = node.borrow().node_type;
        let data = node.borrow().data.clone();

        match node_type {
            NodeType::Heading => {
                if let NodeData::Heading { level, .. } = data {
                    writeln!(output, "    <w:p>")?;
                    writeln!(
                        output,
                        "      <w:pPr><w:pStyle w:val=\"Heading{}\"/></w:pPr>",
                        level
                    )?;
                    writeln!(output, "      <w:r><w:t>")?;
                    self.write_children_text(node, output)?;
                    writeln!(output, "</w:t></w:r>")?;
                    writeln!(output, "    </w:p>")?;
                }
            }
            NodeType::Paragraph => {
                writeln!(output, "    <w:p>")?;
                writeln!(output, "      <w:r><w:t>")?;
                self.write_children_text(node, output)?;
                writeln!(output, "</w:t></w:r>")?;
                writeln!(output, "    </w:p>")?;
            }
            _ => {
                self.write_children(node, output)?;
            }
        }

        Ok(())
    }

    fn write_children(
        &self,
        node: &Rc<RefCell<Node>>,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let children = self.collect_children(node);
        for child in children {
            self.convert_node_to_docx(&child, output)?;
        }
        Ok(())
    }

    fn write_children_text(
        &self,
        node: &Rc<RefCell<Node>>,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let children = self.collect_children(node);
        for child in children {
            self.write_node_text(&child, output)?;
        }
        Ok(())
    }

    fn write_node_text(
        &self,
        node: &Rc<RefCell<Node>>,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let data = node.borrow().data.clone();
        match data {
            NodeData::Text { literal } => {
                let escaped = literal
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;");
                output.extend_from_slice(escaped.as_bytes());
            }
            _ => {
                self.write_children_text(node, output)?;
            }
        }
        Ok(())
    }

    fn collect_children(&self, node: &Rc<RefCell<Node>>) -> Vec<Rc<RefCell<Node>>> {
        let mut children = Vec::new();

        let first_opt = node.borrow().first_child.borrow().clone();
        if let Some(first) = first_opt {
            children.push(first.clone());
            let mut current = first;
            loop {
                let next_opt = current.borrow().next.borrow().clone();
                if let Some(next) = next_opt {
                    children.push(next.clone());
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
    pub fn convert(&self, root: &Rc<RefCell<Node>>) -> Result<Vec<u8>, ConverterError> {
        let mut output = Vec::new();

        writeln!(output, "%PDF-1.4")?;
        writeln!(
            output,
            "% Placeholder PDF - real implementation would use printpdf crate"
        )?;

        self.convert_node_to_pdf(root, &mut output)?;

        writeln!(output, "%%EOF")?;

        Ok(output)
    }

    fn convert_node_to_pdf(
        &self,
        node: &Rc<RefCell<Node>>,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let node_type = node.borrow().node_type;
        let data = node.borrow().data.clone();

        match node_type {
            NodeType::Heading => {
                if let NodeData::Heading { level, .. } = data {
                    write!(output, "Heading {}: ", level)?;
                    self.write_children_text(node, output)?;
                    writeln!(output)?;
                }
            }
            NodeType::Paragraph => {
                self.write_children_text(node, output)?;
                writeln!(output)?;
                writeln!(output)?;
            }
            _ => {
                self.write_children(node, output)?;
            }
        }

        Ok(())
    }

    fn write_children(
        &self,
        node: &Rc<RefCell<Node>>,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let children = self.collect_children(node);
        for child in children {
            self.convert_node_to_pdf(&child, output)?;
        }
        Ok(())
    }

    fn write_children_text(
        &self,
        node: &Rc<RefCell<Node>>,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let children = self.collect_children(node);
        for child in children {
            self.write_node_text(&child, output)?;
        }
        Ok(())
    }

    fn write_node_text(
        &self,
        node: &Rc<RefCell<Node>>,
        output: &mut Vec<u8>,
    ) -> Result<(), ConverterError> {
        let data = node.borrow().data.clone();
        match data {
            NodeData::Text { literal } => {
                output.extend_from_slice(literal.as_bytes());
            }
            _ => {
                self.write_children_text(node, output)?;
            }
        }
        Ok(())
    }

    fn collect_children(&self, node: &Rc<RefCell<Node>>) -> Vec<Rc<RefCell<Node>>> {
        let mut children = Vec::new();

        let first_opt = node.borrow().first_child.borrow().clone();
        if let Some(first) = first_opt {
            children.push(first.clone());
            let mut current = first;
            loop {
                let next_opt = current.borrow().next.borrow().clone();
                if let Some(next) = next_opt {
                    children.push(next.clone());
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
    root: &Rc<RefCell<Node>>,
    format: ExportFormat,
    options: ExportOptions,
) -> Result<Vec<u8>, ConverterError> {
    match format {
        ExportFormat::Docx => {
            let converter = DocxConverter::new(options);
            converter.convert(root)
        }
        ExportFormat::Pdf => {
            let converter = PdfConverter::new(options);
            converter.convert(root)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{Node, NodeData, NodeType};

    fn create_test_document() -> Rc<RefCell<Node>> {
        let doc = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        doc.borrow_mut().data = NodeData::Document;

        let heading = Rc::new(RefCell::new(Node::new(NodeType::Heading)));
        heading.borrow_mut().data = NodeData::Heading {
            level: 1,
            content: "Title".to_string(),
        };

        let text = Rc::new(RefCell::new(Node::new(NodeType::Text)));
        text.borrow_mut().data = NodeData::Text {
            literal: "Test Document".to_string(),
        };
        crate::node::append_child(&heading, text);
        crate::node::append_child(&doc, heading);

        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        para.borrow_mut().data = NodeData::Paragraph;

        let para_text = Rc::new(RefCell::new(Node::new(NodeType::Text)));
        para_text.borrow_mut().data = NodeData::Text {
            literal: "This is a test paragraph.".to_string(),
        };
        crate::node::append_child(&para, para_text);
        crate::node::append_child(&doc, para);

        doc
    }

    #[test]
    fn test_docx_converter() {
        let doc = create_test_document();
        let options = ExportOptions::default();
        let converter = DocxConverter::new(options);

        let result = converter.convert(&doc);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("<?xml version=\"1.0\""));
        assert!(output_str.contains("<w:document"));
        assert!(output_str.contains("Test Document"));
    }

    #[test]
    fn test_pdf_converter() {
        let doc = create_test_document();
        let options = ExportOptions::default();
        let converter = PdfConverter::new(options);

        let result = converter.convert(&doc);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("%PDF"));
        assert!(output_str.contains("Test Document"));
    }

    #[test]
    fn test_export_document() {
        let doc = create_test_document();
        let options = ExportOptions::default();

        let docx_result = export_document(&doc, ExportFormat::Docx, options.clone());
        assert!(docx_result.is_ok());

        let pdf_result = export_document(&doc, ExportFormat::Pdf, options);
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
