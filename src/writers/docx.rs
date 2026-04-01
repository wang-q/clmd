//! DOCX document writer.
//!
//! This module provides a writer for Microsoft Word DOCX format.
//!
//! # Example
//!
//! ```ignore
//! use clmd::writers::DocxWriter;
//! use clmd::options::WriterOptions;
//! use clmd::context::PureContext;
//!
//! let writer = DocxWriter;
//! let ctx = PureContext::new();
//! let output = writer.write(&arena, root, &ctx, &WriterOptions::default()).unwrap();
//! ```

use crate::context::ClmdContext;
use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::{ClmdError, ClmdResult};
use crate::core::nodes::NodeValue;
use crate::options::{OutputFormat, WriterOptions};
use crate::writers::Writer;
use std::io::Write;

/// DOCX document writer.
#[derive(Debug, Clone, Copy)]
pub struct DocxWriter;

impl Writer for DocxWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        _options: &WriterOptions,
    ) -> ClmdResult<String> {
        // DOCX is a binary format, so we return base64-encoded content
        let docx_bytes = write_docx_binary(arena, root)?;
        Ok(base64::encode(docx_bytes))
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Docx
    }

    fn extensions(&self) -> &[&'static str] {
        &["docx"]
    }

    fn mime_type(&self) -> &'static str {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    }

    /// Write the AST to a file.
    fn write_to_file(
        &self,
        arena: &NodeArena,
        root: NodeId,
        path: &std::path::Path,
        ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<()> {
        let docx_bytes = write_docx_binary(arena, root)?;
        ctx.write_file(path, &docx_bytes)?;
        Ok(())
    }
}

/// Write DOCX as binary bytes.
fn write_docx_binary(arena: &NodeArena, root: NodeId) -> ClmdResult<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // [Content_Types].xml
        zip.start_file("[Content_Types].xml", options)
            .map_err(|e| ClmdError::io_error(format!("Failed to create zip entry: {}", e)))?;
        zip.write_all(CONTENT_TYPES.as_bytes())
            .map_err(|e| ClmdError::io_error(format!("Failed to write: {}", e)))?;

        // _rels/.rels
        zip.start_file("_rels/.rels", options)
            .map_err(|e| ClmdError::io_error(format!("Failed to create zip entry: {}", e)))?;
        zip.write_all(RELS.as_bytes())
            .map_err(|e| ClmdError::io_error(format!("Failed to write: {}", e)))?;

        // word/_rels/document.xml.rels
        zip.start_file("word/_rels/document.xml.rels", options)
            .map_err(|e| ClmdError::io_error(format!("Failed to create zip entry: {}", e)))?;
        zip.write_all(DOCUMENT_RELS.as_bytes())
            .map_err(|e| ClmdError::io_error(format!("Failed to write: {}", e)))?;

        // word/document.xml
        let document_xml = generate_document_xml(arena, root)?;
        zip.start_file("word/document.xml", options)
            .map_err(|e| ClmdError::io_error(format!("Failed to create zip entry: {}", e)))?;
        zip.write_all(document_xml.as_bytes())
            .map_err(|e| ClmdError::io_error(format!("Failed to write: {}", e)))?;

        // word/styles.xml
        zip.start_file("word/styles.xml", options)
            .map_err(|e| ClmdError::io_error(format!("Failed to create zip entry: {}", e)))?;
        zip.write_all(STYLES.as_bytes())
            .map_err(|e| ClmdError::io_error(format!("Failed to write: {}", e)))?;

        zip.finish()
            .map_err(|e| ClmdError::io_error(format!("Failed to finish zip: {}", e)))?;
    }

    Ok(buf)
}

/// Generate document.xml content.
fn generate_document_xml(arena: &NodeArena, root: NodeId) -> ClmdResult<String> {
    let mut body = String::new();
    render_node(arena, root, &mut body)?;

    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
    <w:body>
        {}
        <w:sectPr>
            <w:pgSz w:w="12240" w:h="15840"/>
            <w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/>
        </w:sectPr>
    </w:body>
</w:document>"#,
        body
    ))
}

/// Render a node and its children to DOCX XML.
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

        NodeValue::Heading(heading) => {
            let style = match heading.level {
                1 => "Heading1",
                2 => "Heading2",
                3 => "Heading3",
                4 => "Heading4",
                5 => "Heading5",
                _ => "Heading6",
            };

            output.push_str(&format!(
                r#"<w:p><w:pPr><w:pStyle w:val="{}"/></w:pPr>"#,
                style
            ));

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("</w:p>");
        }

        NodeValue::Paragraph => {
            output.push_str("<w:p>");

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("</w:p>");
        }

        NodeValue::BlockQuote => {
            output.push_str(r#"<w:p><w:pPr><w:ind w:left="720"/></w:pPr>"#);

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("</w:p>");
        }

        NodeValue::CodeBlock(code) => {
            output.push_str(
                r#"<w:p><w:pPr><w:spacing w:before="120" w:after="120"/></w:pPr>"#,
            );

            for line in code.literal.lines() {
                output.push_str(r#"<w:r><w:rPr><w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/><w:sz w:val="20"/></w:rPr><w:t xml:space="preserve">"#);
                escape_xml_text(line, output);
                output.push_str("</w:t></w:r><w:br/>");
            }

            output.push_str("</w:p>");
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
            output.push_str(
                r#"<w:p><w:pPr><w:pStyle w:val="ListParagraph"/><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr>"#,
            );

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("</w:p>");
        }

        NodeValue::ThematicBreak => {
            output.push_str(
                r#"<w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="6" w:space="1" w:color="auto"/></w:pBdr></w:pPr><w:r><w:t></w:t></w:r></w:p>"#,
            );
        }

        _ => {
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
            output.push_str("<w:r><w:t xml:space=\"preserve\">");
            escape_xml_text(text, output);
            output.push_str("</w:t></w:r>");
        }

        NodeValue::SoftBreak | NodeValue::HardBreak => {
            output.push_str("<w:r><w:br/></w:r>");
        }

        NodeValue::Emph => {
            output.push_str(r#"<w:r><w:rPr><w:i/></w:rPr><w:t xml:space="preserve">"#);
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline_text(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("</w:t></w:r>");
        }

        NodeValue::Strong => {
            output.push_str(r#"<w:r><w:rPr><w:b/></w:rPr><w:t xml:space="preserve">"#);
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline_text(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("</w:t></w:r>");
        }

        NodeValue::Code(code) => {
            output.push_str(r#"<w:r><w:rPr><w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/><w:sz w:val="20"/></w:rPr><w:t xml:space="preserve">"#);
            escape_xml_text(&code.literal, output);
            output.push_str("</w:t></w:r>");
        }

        NodeValue::Link(link) => {
            // Hyperlink in DOCX
            output.push_str(&format!(
                r#"<w:hyperlink r:id="{}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
                link.url
            ));

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("</w:hyperlink>");
        }

        NodeValue::Strikethrough => {
            output.push_str(r#"<w:r><w:rPr><w:strike/></w:rPr><w:t xml:space="preserve">"#);
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline_text(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("</w:t></w:r>");
        }

        NodeValue::Underline => {
            output.push_str(r#"<w:r><w:rPr><w:u w:val="single"/></w:rPr><w:t xml:space="preserve">"#);
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline_text(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
            output.push_str("</w:t></w:r>");
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

/// Render inline text only (for nested formatting).
fn render_inline_text(arena: &NodeArena, node_id: NodeId, output: &mut String) -> ClmdResult<()> {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Text(text) => {
            escape_xml_text(text, output);
        }

        NodeValue::SoftBreak | NodeValue::HardBreak => {
            output.push(' ');
        }

        _ => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline_text(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }
        }
    }

    Ok(())
}

/// Escape XML special characters.
fn escape_xml_text(text: &str, output: &mut String) {
    for c in text.chars() {
        match c {
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '&' => output.push_str("&amp;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            _ => output.push(c),
        }
    }
}

// Static XML content for DOCX structure
const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="xml" ContentType="application/xml"/>
    <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
    <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#;

const RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

const DOCUMENT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;

const STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
        <w:name w:val="Normal"/>
        <w:pPr>
            <w:spacing w:after="160"/>
        </w:pPr>
    </w:style>
    <w:style w:type="paragraph" w:styleId="Heading1">
        <w:name w:val="heading 1"/>
        <w:basedOn w:val="Normal"/>
        <w:pPr>
            <w:keepNext/>
            <w:keepLines/>
            <w:spacing w:before="240" w:after="120"/>
            <w:outlineLvl w:val="0"/>
        </w:pPr>
        <w:rPr>
            <w:b/>
            <w:sz w:val="32"/>
        </w:rPr>
    </w:style>
    <w:style w:type="paragraph" w:styleId="Heading2">
        <w:name w:val="heading 2"/>
        <w:basedOn w:val="Normal"/>
        <w:pPr>
            <w:keepNext/>
            <w:keepLines/>
            <w:spacing w:before="200" w:after="100"/>
            <w:outlineLvl w:val="1"/>
        </w:pPr>
        <w:rPr>
            <w:b/>
            <w:sz w:val="28"/>
        </w:rPr>
    </w:style>
    <w:style w:type="paragraph" w:styleId="Heading3">
        <w:name w:val="heading 3"/>
        <w:basedOn w:val="Normal"/>
        <w:pPr>
            <w:keepNext/>
            <w:keepLines/>
            <w:spacing w:before="160" w:after="80"/>
            <w:outlineLvl w:val="2"/>
        </w:pPr>
        <w:rPr>
            <w:b/>
            <w:sz w:val="24"/>
        </w:rPr>
    </w:style>
    <w:style w:type="paragraph" w:styleId="Heading4">
        <w:name w:val="heading 4"/>
        <w:basedOn w:val="Normal"/>
        <w:pPr>
            <w:keepNext/>
            <w:keepLines/>
            <w:spacing w:before="120" w:after="60"/>
            <w:outlineLvl w:val="3"/>
        </w:pPr>
        <w:rPr>
            <w:b/>
            <w:sz w:val="22"/>
        </w:rPr>
    </w:style>
    <w:style w:type="paragraph" w:styleId="Heading5">
        <w:name w:val="heading 5"/>
        <w:basedOn w:val="Normal"/>
        <w:pPr>
            <w:keepNext/>
            <w:keepLines/>
            <w:spacing w:before="100" w:after="50"/>
            <w:outlineLvl w:val="4"/>
        </w:pPr>
        <w:rPr>
            <w:b/>
            <w:sz w:val="20"/>
        </w:rPr>
    </w:style>
    <w:style w:type="paragraph" w:styleId="Heading6">
        <w:name w:val="heading 6"/>
        <w:basedOn w:val="Normal"/>
        <w:pPr>
            <w:keepNext/>
            <w:keepLines/>
            <w:spacing w:before="80" w:after="40"/>
            <w:outlineLvl w:val="5"/>
        </w:pPr>
        <w:rPr>
            <w:b/>
            <w:sz w:val="18"/>
        </w:rPr>
    </w:style>
    <w:style w:type="paragraph" w:styleId="ListParagraph">
        <w:name w:val="List Paragraph"/>
        <w:basedOn w:val="Normal"/>
        <w:pPr>
            <w:ind w:left="720"/>
        </w:pPr>
    </w:style>
</w:styles>"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::PureContext;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{NodeHeading, NodeValue};
    use crate::options::WriterOptions;

    fn create_test_document() -> (NodeArena, NodeId) {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Add a heading
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text(
            "Hello".to_string().into_boxed_str(),
        )));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        // Add a paragraph
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para_text = arena.alloc(Node::with_value(NodeValue::Text(
            "World".to_string().into_boxed_str(),
        )));
        TreeOps::append_child(&mut arena, para, para_text);
        TreeOps::append_child(&mut arena, root, para);

        (arena, root)
    }

    #[test]
    fn test_docx_writer_basic() {
        let writer = DocxWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let (arena, root) = create_test_document();

        let output = writer.write(&arena, root, &ctx, &options).unwrap();

        // Output should be base64 encoded
        assert!(!output.is_empty());
    }

    #[test]
    fn test_docx_writer_format() {
        let writer = DocxWriter;
        assert_eq!(writer.format(), OutputFormat::Docx);
        assert!(writer.extensions().contains(&"docx"));
    }

    #[test]
    fn test_docx_binary_generation() {
        let (arena, root) = create_test_document();
        let bytes = write_docx_binary(&arena, root).unwrap();

        // Should be a valid ZIP file (starts with PK)
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[0..2], b"PK");
    }
}
