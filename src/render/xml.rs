//! XML renderer
//!
//! This module provides XML output generation for documents parsed using the Arena-based parser.
//! Useful for debugging and AST inspection.

use crate::nodes::{AstNode, ListDelimType, ListType, NodeValue};
use crate::parser::options::{Options, Plugins};
use std::fmt;

/// Format an AST as XML.
///
/// This is a convenience function that doesn't use plugins.
pub fn format_document<'a>(
    root: &'a AstNode<'a>,
    options: &Options,
    output: &mut dyn fmt::Write,
) -> fmt::Result {
    format_document_with_plugins(root, options, output, &Plugins::default())
}

/// Format an AST as XML with plugins (comrak-style API).
///
/// This is a temporary implementation that provides basic XML output.
pub fn format_document_with_plugins<'a>(
    root: &'a AstNode<'a>,
    options: &Options,
    output: &mut dyn fmt::Write,
    _plugins: &Plugins<'_>,
) -> fmt::Result {
    output.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")?;
    output.write_str("<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n")?;
    format_node_xml(root, options, output)
}

fn format_node_xml(
    node: &AstNode<'_>,
    options: &Options,
    output: &mut dyn fmt::Write,
) -> fmt::Result {
    let ast = node.data.borrow();
    let tag_name = ast.value.xml_node_name();

    output.write_str("<")?;
    output.write_str(tag_name)?;

    // Add source position if enabled
    if options.render.sourcepos && ast.sourcepos.start.line > 0 {
        write!(
            output,
            " sourcepos=\"{}:{}-{}:{}\"",
            ast.sourcepos.start.line,
            ast.sourcepos.start.column,
            ast.sourcepos.end.line,
            ast.sourcepos.end.column
        )?;
    }

    // Add type-specific attributes
    match &ast.value {
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
        _ => {}
    }

    // Handle leaf nodes with literal content
    if ast.value.is_leaf() {
        match &ast.value {
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
            _ => {
                output.write_str(" />")?;
            }
        }
    } else {
        output.write_str(">\n")?;

        // Render children
        let mut child_opt = node.first_child();
        while let Some(child) = child_opt {
            format_node_xml(child, options, output)?;
            child_opt = child.next_sibling();
        }

        write!(output, "</{tag_name}>\n")?;
    }

    Ok(())
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

    // Note: Tests requiring tree manipulation are disabled due to lifetime issues
    // with the arena-based tree structure. The XML renderer itself is tested
    // through integration tests.
}
