//! EPUB document writer.
//!
//! This module provides a writer for EPUB (Electronic Publication) format.
//!
//! # Example
//!
//! ```ignore
//! use clmd::writers::EpubWriter;
//! use clmd::options::WriterOptions;
//! use clmd::context::PureContext;
//!
//! let writer = EpubWriter;
//! let ctx = PureContext::new();
//! let output = writer.write(&arena, root, &ctx, &WriterOptions::default()).unwrap();
//! ```

use crate::context::ClmdContext;
use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::{ClmdError, ClmdResult};
use crate::core::nodes::NodeValue;
use crate::io::writer::Writer;
use crate::options::{OutputFormat, WriterOptions};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::io::Write;

/// EPUB document writer.
#[derive(Debug, Clone, Copy)]
pub struct EpubWriter;

impl Writer for EpubWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        _options: &WriterOptions,
    ) -> ClmdResult<String> {
        // EPUB is a binary format, so we return base64-encoded content
        let epub_bytes = write_epub_binary(arena, root)?;
        Ok(BASE64.encode(epub_bytes))
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Epub
    }

    fn extensions(&self) -> &[&'static str] {
        &["epub"]
    }

    fn mime_type(&self) -> &'static str {
        "application/epub+zip"
    }
}

/// Write EPUB as binary bytes.
fn write_epub_binary(arena: &NodeArena, root: NodeId) -> ClmdResult<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // mimetype (must be first and uncompressed)
        let mimetype_options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("mimetype", mimetype_options).map_err(|e| {
            ClmdError::io_error(format!("Failed to create zip entry: {}", e))
        })?;
        zip.write_all(b"application/epub+zip")
            .map_err(|e| ClmdError::io_error(format!("Failed to write: {}", e)))?;

        // META-INF/container.xml
        zip.start_file("META-INF/container.xml", options)
            .map_err(|e| {
                ClmdError::io_error(format!("Failed to create zip entry: {}", e))
            })?;
        zip.write_all(CONTAINER_XML.as_bytes())
            .map_err(|e| ClmdError::io_error(format!("Failed to write: {}", e)))?;

        // content.opf
        zip.start_file("OEBPS/content.opf", options).map_err(|e| {
            ClmdError::io_error(format!("Failed to create zip entry: {}", e))
        })?;
        zip.write_all(CONTENT_OPF.as_bytes())
            .map_err(|e| ClmdError::io_error(format!("Failed to write: {}", e)))?;

        // toc.ncx
        zip.start_file("OEBPS/toc.ncx", options).map_err(|e| {
            ClmdError::io_error(format!("Failed to create zip entry: {}", e))
        })?;
        zip.write_all(TOC_NCX.as_bytes())
            .map_err(|e| ClmdError::io_error(format!("Failed to write: {}", e)))?;

        // chapter1.xhtml
        let chapter = generate_chapter_xhtml(arena, root)?;
        zip.start_file("OEBPS/chapter1.xhtml", options)
            .map_err(|e| {
                ClmdError::io_error(format!("Failed to create zip entry: {}", e))
            })?;
        zip.write_all(chapter.as_bytes())
            .map_err(|e| ClmdError::io_error(format!("Failed to write: {}", e)))?;

        // stylesheet.css
        zip.start_file("OEBPS/stylesheet.css", options)
            .map_err(|e| {
                ClmdError::io_error(format!("Failed to create zip entry: {}", e))
            })?;
        zip.write_all(STYLESHEET_CSS.as_bytes())
            .map_err(|e| ClmdError::io_error(format!("Failed to write: {}", e)))?;

        zip.finish()
            .map_err(|e| ClmdError::io_error(format!("Failed to finish zip: {}", e)))?;
    }

    Ok(buf)
}

/// Generate chapter XHTML content.
fn generate_chapter_xhtml(arena: &NodeArena, root: NodeId) -> ClmdResult<String> {
    let mut body = String::new();
    render_node(arena, root, &mut body)?;

    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>Document</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
{}
</body>
</html>"#,
        body
    ))
}

/// Render a node and its children to XHTML.
fn render_node(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut String,
) -> ClmdResult<()> {
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
            let tag = match heading.level {
                1 => "h1",
                2 => "h2",
                3 => "h3",
                4 => "h4",
                5 => "h5",
                _ => "h6",
            };

            output.push_str(&format!("<{}>", tag));

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str(&format!("</{}>", tag));
        }

        NodeValue::Paragraph => {
            output.push_str("<p>");

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("</p>");
        }

        NodeValue::BlockQuote => {
            output.push_str("<blockquote>");

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("</blockquote>");
        }

        NodeValue::CodeBlock(code) => {
            output.push_str("<pre><code>");
            escape_html(&code.literal, output);
            output.push_str("</code></pre>");
        }

        NodeValue::List(_) => {
            output.push_str("<ul>");

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_node(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("</ul>");
        }

        NodeValue::Item(_) => {
            output.push_str("<li>");

            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                render_inline(arena, child_id, output)?;
                let child = arena.get(child_id);
                child_opt = child.next;
            }

            output.push_str("</li>");
        }

        NodeValue::ThematicBreak(..) => {
            output.push_str("<hr/>");
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
fn render_inline(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut String,
) -> ClmdResult<()> {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Text(text) => {
            escape_html(text, output);
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
            escape_html(&code.literal, output);
            output.push_str("</code>");
        }

        NodeValue::Link(link) => {
            output.push_str(&format!(r#"<a href="{}">"#, link.url));
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
fn escape_html(text: &str, output: &mut String) {
    for c in text.chars() {
        match c {
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '&' => output.push_str("&amp;"),
            '"' => output.push_str("&quot;"),
            _ => output.push(c),
        }
    }
}

// Static XML content for EPUB structure
const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
    <rootfiles>
        <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
    </rootfiles>
</container>"#;

const CONTENT_OPF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf" unique-identifier="BookId">
    <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
        <dc:title>Document</dc:title>
        <dc:language>en</dc:language>
        <dc:identifier id="BookId">urn:uuid:document</dc:identifier>
    </metadata>
    <manifest>
        <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
        <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
        <item id="stylesheet" href="stylesheet.css" media-type="text/css"/>
    </manifest>
    <spine toc="ncx">
        <itemref idref="chapter1"/>
    </spine>
</package>"#;

const TOC_NCX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE ncx PUBLIC "-//NISO//DTD ncx 2005-1//EN" "http://www.daisy.org/z3986/2005/ncx-2005-1.dtd">
<ncx version="2005-1" xmlns="http://www.daisy.org/z3986/2005/ncx/">
    <head>
        <meta name="dtb:uid" content="urn:uuid:document"/>
        <meta name="dtb:depth" content="1"/>
        <meta name="dtb:totalPageCount" content="0"/>
        <meta name="dtb:maxPageNumber" content="0"/>
    </head>
    <docTitle>
        <text>Document</text>
    </docTitle>
    <navMap>
        <navPoint id="navpoint-1" playOrder="1">
            <navLabel>
                <text>Chapter 1</text>
            </navLabel>
            <content src="chapter1.xhtml"/>
        </navPoint>
    </navMap>
</ncx>"#;

const STYLESHEET_CSS: &str = r#"body {
    font-family: serif;
    line-height: 1.5;
    margin: 1em;
}

h1, h2, h3, h4, h5, h6 {
    font-weight: bold;
    margin-top: 1em;
    margin-bottom: 0.5em;
}

h1 { font-size: 2em; }
h2 { font-size: 1.5em; }
h3 { font-size: 1.17em; }
h4 { font-size: 1em; }
h5 { font-size: 0.83em; }
h6 { font-size: 0.67em; }

p {
    margin-top: 0.5em;
    margin-bottom: 0.5em;
}

blockquote {
    margin-left: 2em;
    margin-right: 2em;
    font-style: italic;
}

pre {
    background-color: #f5f5f5;
    padding: 1em;
    overflow-x: auto;
}

code {
    font-family: monospace;
    background-color: #f5f5f5;
    padding: 0.1em 0.2em;
}

pre code {
    background-color: transparent;
    padding: 0;
}

ul, ol {
    margin-left: 2em;
}

li {
    margin-bottom: 0.25em;
}

hr {
    border: none;
    border-top: 1px solid #ccc;
    margin: 1em 0;
}

a {
    color: #0066cc;
    text-decoration: underline;
}

em, i {
    font-style: italic;
}

strong, b {
    font-weight: bold;
}

del {
    text-decoration: line-through;
}

u {
    text-decoration: underline;
}"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::PureContext;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{
        NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeValue,
    };
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
        let text = arena.alloc(Node::with_value(NodeValue::Text("Hello".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        // Add a paragraph
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para_text = arena.alloc(Node::with_value(NodeValue::Text("World".into())));
        TreeOps::append_child(&mut arena, para, para_text);
        TreeOps::append_child(&mut arena, root, para);

        (arena, root)
    }

    #[test]
    fn test_epub_writer_basic() {
        let writer = EpubWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let (arena, root) = create_test_document();

        let output = writer.write(&arena, root, &ctx, &options).unwrap();

        // Output should be base64 encoded
        assert!(!output.is_empty());
    }

    #[test]
    fn test_epub_writer_format() {
        let writer = EpubWriter;
        assert_eq!(writer.format(), OutputFormat::Epub);
        assert!(writer.extensions().contains(&"epub"));
        assert_eq!(writer.mime_type(), "application/epub+zip");
    }

    #[test]
    fn test_epub_binary_generation() {
        let (arena, root) = create_test_document();
        let bytes = write_epub_binary(&arena, root).unwrap();

        // Should be a valid ZIP file (starts with PK)
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[0..2], b"PK");

        // Should contain mimetype file
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("mimetype"));
        assert!(content.contains("META-INF/container.xml"));
        assert!(content.contains("OEBPS/content.opf"));
        assert!(content.contains("OEBPS/toc.ncx"));
        assert!(content.contains("OEBPS/chapter1.xhtml"));
        assert!(content.contains("OEBPS/stylesheet.css"));
    }

    #[test]
    fn test_epub_empty_document() {
        let writer = EpubWriter;
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(!output.is_empty());

        // Verify it's valid base64
        let decoded = BASE64.decode(&output).unwrap();
        assert_eq!(&decoded[0..2], b"PK");
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Section".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("<h2>Section</h2>"));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_emphasis() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("italic".into())));
        TreeOps::append_child(&mut arena, emph, text);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, root, para);

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("<em>italic</em>"));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_strong() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::Text("bold".into())));
        TreeOps::append_child(&mut arena, strong, text);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, root, para);

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_strikethrough() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strike = arena.alloc(Node::with_value(NodeValue::Strikethrough));
        let text = arena.alloc(Node::with_value(NodeValue::Text("deleted".into())));
        TreeOps::append_child(&mut arena, strike, text);
        TreeOps::append_child(&mut arena, para, strike);
        TreeOps::append_child(&mut arena, root, para);

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("<del>deleted</del>"));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_underline() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let underline = arena.alloc(Node::with_value(NodeValue::Underline));
        let text = arena.alloc(Node::with_value(NodeValue::Text("underlined".into())));
        TreeOps::append_child(&mut arena, underline, text);
        TreeOps::append_child(&mut arena, para, underline);
        TreeOps::append_child(&mut arena, root, para);

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("<u>underlined</u>"));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_code() {
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

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("<code>code</code>"));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_code_block() {
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

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("<pre><code>"));
        assert!(xhtml.contains("fn main() {}"));
        assert!(xhtml.contains("</code></pre>"));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_code_block_escape() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let code_block = NodeValue::CodeBlock(Box::new(NodeCodeBlock {
            literal: "<div> & \"test\"".into(),
            info: "".into(),
            fenced: true,
            fence_char: b'`',
            fence_length: 3,
            fence_offset: 0,
            closed: true,
        }));
        let code_node = arena.alloc(Node::with_value(code_block));
        TreeOps::append_child(&mut arena, root, code_node);

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("&lt;div&gt;"));
        assert!(xhtml.contains("&amp;"));
        assert!(xhtml.contains("&quot;test&quot;"));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_link() {
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

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains(r#"<a href="https://example.com">click</a>"#));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_blockquote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let quote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Quoted".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, quote, para);
        TreeOps::append_child(&mut arena, root, quote);

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("<blockquote>"));
        assert!(xhtml.contains("<p>Quoted</p>"));
        assert!(xhtml.contains("</blockquote>"));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_thematic_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let hr = arena.alloc(Node::with_value(NodeValue::ThematicBreak(
            crate::core::nodes::NodeThematicBreak::default(),
        )));
        TreeOps::append_child(&mut arena, root, hr);

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("<hr/>"));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_soft_break() {
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

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("Line1 Line2")); // Soft break becomes space
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_hard_break() {
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

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("Line1<br/>Line2"));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_text_escape() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("<>&\"".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("&lt;"));
        assert!(xhtml.contains("&gt;"));
        assert!(xhtml.contains("&amp;"));
        assert!(xhtml.contains("&quot;"));
    }

    #[test]
    fn test_epub_generate_chapter_xhtml_nested_inline() {
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

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("<strong><em>bold italic</em></strong>"));
    }

    #[test]
    fn test_epub_all_heading_levels() {
        for level in 1..=6 {
            let mut arena = NodeArena::new();
            let root = arena.alloc(Node::with_value(NodeValue::Document));
            let heading =
                arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
                    level,
                    setext: false,
                    closed: false,
                })));
            let text = arena.alloc(Node::with_value(NodeValue::Text(
                format!("Heading {}", level).into(),
            )));
            TreeOps::append_child(&mut arena, heading, text);
            TreeOps::append_child(&mut arena, root, heading);

            let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
            assert!(xhtml.contains(&format!("<h{}>Heading {}", level, level)));
            assert!(xhtml.contains(&format!("</h{}>", level)));
        }
    }

    #[test]
    fn test_escape_html() {
        let mut output = String::new();
        escape_html("<>&\"'", &mut output);
        assert_eq!(output, "&lt;&gt;&amp;&quot;'");
    }

    #[test]
    fn test_generate_chapter_xhtml() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Test".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let xhtml = generate_chapter_xhtml(&arena, root).unwrap();
        assert!(xhtml.contains("<?xml version="));
        assert!(xhtml.contains("<!DOCTYPE html"));
        assert!(xhtml.contains("<html"));
        assert!(xhtml.contains("<head>"));
        assert!(xhtml.contains("<title>Document</title>"));
        assert!(xhtml.contains("<body>"));
        assert!(xhtml.contains("<p>Test</p>"));
        assert!(xhtml.contains("</body>"));
        assert!(xhtml.contains("</html>"));
    }
}
