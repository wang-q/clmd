//! XML renderer
//!
//! This module provides XML output generation for documents parsed using the Arena-based parser.
//! Useful for debugging and AST inspection.

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::{ListDelimType, ListType, NodeValue};
use crate::parse::options::{Options, Plugins};
use std::fmt;

/// Render an AST as XML.
///
/// This is a convenience function that doesn't use plugins.
pub fn render(arena: &NodeArena, root: NodeId, _options: u32) -> String {
    let mut renderer = XmlRenderer::new(arena);
    renderer.render(root)
}

/// Format an AST as XML.
///
/// This is a convenience function that doesn't use plugins.
pub fn format_document(
    arena: &NodeArena,
    root: NodeId,
    options: &Options,
    output: &mut dyn fmt::Write,
) -> fmt::Result {
    format_document_with_plugins(arena, root, options, output, &Plugins::default())
}

/// Format an AST as XML with plugins (comrak-style API).
///
/// This implementation uses the new NodeArena-based AST.
pub fn format_document_with_plugins(
    arena: &NodeArena,
    root: NodeId,
    options: &Options,
    output: &mut dyn fmt::Write,
    _plugins: &Plugins<'_>,
) -> fmt::Result {
    output.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")?;
    output.write_str("<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n")?;
    format_node_xml(arena, root, options, output)
}

fn format_node_xml(
    arena: &NodeArena,
    node_id: NodeId,
    options: &Options,
    output: &mut dyn fmt::Write,
) -> fmt::Result {
    let node = arena.get(node_id);
    let tag_name = node.value.xml_node_name();

    output.write_str("<")?;
    output.write_str(tag_name)?;

    // Add source position if enabled
    if options.render.sourcepos && node.source_pos.start.line > 0 {
        write!(
            output,
            " sourcepos=\"{}:{}-{}:{}\"",
            node.source_pos.start.line,
            node.source_pos.start.column,
            node.source_pos.end.line,
            node.source_pos.end.column
        )?;
    }

    // Add type-specific attributes
    match &node.value {
        NodeValue::List(list) => {
            match list.list_type {
                ListType::Bullet => {
                    output.write_str(" type=\"bullet\"")?;
                }
                ListType::Ordered => {
                    output.write_str(" type=\"ordered\"")?;
                    if list.start != 1 {
                        write!(output, " start=\"{}\"", list.start)?;
                    }
                    match list.delimiter {
                        ListDelimType::Period => {
                            output.write_str(" delim=\"period\"")?;
                        }
                        ListDelimType::Paren => {
                            output.write_str(" delim=\"paren\"")?;
                        }
                    }
                }
            }
            if list.tight {
                output.write_str(" tight=\"true\"")?;
            }
        }
        NodeValue::Heading(heading) => {
            write!(output, " level=\"{}\"", heading.level)?;
        }
        NodeValue::CodeBlock(code) => {
            if !code.info.is_empty() {
                write!(output, " info=\"{}\"", escape_xml(&code.info))?;
            }
        }
        NodeValue::Link(link) | NodeValue::Image(link) => {
            write!(output, " destination=\"{}\"", escape_xml(&link.url))?;
            if !link.title.is_empty() {
                write!(output, " title=\"{}\"", escape_xml(&link.title))?;
            }
        }
        NodeValue::ShortCode(shortcode) => {
            write!(output, " code=\"{}\"", escape_xml(&shortcode.code))?;
        }
        _ => {}
    }

    // Handle leaf nodes with literal content
    if node.value.is_leaf() {
        match &node.value {
            NodeValue::Text(text) => {
                if !text.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(text))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::HtmlInline(html) | NodeValue::Raw(html) => {
                if !html.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(html))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::CodeBlock(code) => {
                if !code.literal.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(&code.literal))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::HtmlBlock(html) => {
                if !html.literal.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(&html.literal))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::Code(code) => {
                if !code.literal.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(&code.literal))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::ShortCode(shortcode) => {
                if !shortcode.emoji.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(&shortcode.emoji))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            _ => {
                output.write_str(" />")?;
            }
        }
    } else {
        output.write_str(">\n")?;

        // Render children
        let mut child_opt = node.first_child;
        while let Some(child_id) = child_opt {
            format_node_xml(arena, child_id, options, output)?;
            child_opt = arena.get(child_id).next;
        }

        writeln!(output, "</{tag_name}>")?;
    }

    Ok(())
}

/// XML renderer state
struct XmlRenderer<'a> {
    arena: &'a NodeArena,
    output: String,
}

impl<'a> XmlRenderer<'a> {
    fn new(arena: &'a NodeArena) -> Self {
        XmlRenderer {
            arena,
            output: String::new(),
        }
    }

    fn render(&mut self, root: NodeId) -> String {
        self.output
            .push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        self.output
            .push_str("<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n");
        self.render_node(root);
        self.output.clone()
    }

    fn render_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        let tag_name = node.value.xml_node_name();

        self.output.push('<');
        self.output.push_str(tag_name);

        // Add type-specific attributes
        match &node.value {
            NodeValue::List(list) => match list.list_type {
                ListType::Bullet => {
                    self.output.push_str(" type=\"bullet\"");
                }
                ListType::Ordered => {
                    self.output.push_str(" type=\"ordered\"");
                    if list.start != 1 {
                        self.output.push_str(&format!(" start=\"{}\"", list.start));
                    }
                }
            },
            NodeValue::Heading(heading) => {
                self.output
                    .push_str(&format!(" level=\"{}\"", heading.level));
            }
            NodeValue::ShortCode(shortcode) => {
                self.output
                    .push_str(&format!(" code=\"{}\"", escape_xml(&shortcode.code)));
            }
            _ => {}
        }

        // Handle leaf nodes
        if node.value.is_leaf() {
            match &node.value {
                NodeValue::Text(text) => {
                    if !text.is_empty() {
                        self.output.push('>');
                        self.output.push_str(&escape_xml(text));
                        self.output.push_str(&format!("</{tag_name}>"));
                    } else {
                        self.output.push_str(" />");
                    }
                }
                NodeValue::ShortCode(shortcode) => {
                    if !shortcode.emoji.is_empty() {
                        self.output.push('>');
                        self.output.push_str(&escape_xml(&shortcode.emoji));
                        self.output.push_str(&format!("</{tag_name}>"));
                    } else {
                        self.output.push_str(" />");
                    }
                }
                _ => {
                    self.output.push_str(" />");
                }
            }
        } else {
            self.output.push_str(">\n");

            // Render children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                self.render_node(child_id);
                child_opt = self.arena.get(child_id).next;
            }

            self.output.push_str(&format!("</{tag_name}>\n"));
        }
    }
}

/// Escape XML special characters
fn escape_xml(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<div>"), "&lt;div&gt;");
        assert_eq!(escape_xml("&"), "&amp;");
        assert_eq!(escape_xml("'test'"), "&apos;test&apos;");
    }
}
