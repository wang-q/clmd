//! Typst renderer
//!
//! This module provides Typst output generation for CommonMark documents.
//! Typst is a modern markup-based typesetting system.

use crate::nodes::{AstNode, NodeValue};
use crate::parser::options::{Options, Plugins};
use std::fmt;

/// Format an AST as Typst with plugins.
///
/// This is a basic implementation that provides Typst output.
/// Note: Full Typst support is still under development.
pub fn format_document_with_plugins<'a>(
    root: &'a AstNode<'a>,
    _options: &Options,
    output: &mut dyn fmt::Write,
    _plugins: &Plugins<'_>,
) -> fmt::Result {
    format_node_typst(root, output)
}

fn format_node_typst(node: &AstNode<'_>, output: &mut dyn fmt::Write) -> fmt::Result {
    let ast = node.data.borrow();

    match &ast.value {
        NodeValue::Document => {
            // Render children
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_typst(child, output)?;
                child_opt = child.next_sibling();
            }
        }
        NodeValue::Paragraph => {
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_typst(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("\n\n")?;
        }
        NodeValue::Text(text) => {
            // Escape special Typst characters
            let escaped = text
                .replace('\\', "\\")
                .replace('*', "\\*")
                .replace('_', "\\_")
                .replace('#', "\\#")
                .replace('@', "\\@");
            output.write_str(&escaped)?;
        }
        NodeValue::Heading(heading) => {
            let level = heading.level as usize;
            let prefix = match level {
                1 => "= ",
                2 => "== ",
                3 => "=== ",
                4 => "==== ",
                5 => "===== ",
                6 => "====== ",
                _ => "= ",
            };
            output.write_str(prefix)?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_typst(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("\n\n")?;
        }
        NodeValue::Emph => {
            output.write_str("_")?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_typst(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("_")?;
        }
        NodeValue::Strong => {
            output.write_str("*")?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_typst(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("*")?;
        }
        NodeValue::Code(code) => {
            output.write_str("`")?;
            output.write_str(&code.literal)?;
            output.write_str("`")?;
        }
        NodeValue::CodeBlock(code_block) => {
            output.write_str("```")?;
            if !code_block.info.is_empty() {
                output.write_str(&code_block.info)?;
            }
            output.write_str("\n")?;
            output.write_str(&code_block.literal)?;
            output.write_str("\n```\n\n")?;
        }
        NodeValue::List(_) => {
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_typst(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("\n")?;
        }
        NodeValue::Item(_) => {
            output.write_str("- ")?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_typst(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("\n")?;
        }
        NodeValue::BlockQuote => {
            output.write_str("#quote[\n")?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_typst(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("]\n\n")?;
        }
        NodeValue::ThematicBreak => {
            output.write_str("#line(length: 100%)\n\n")?;
        }
        NodeValue::SoftBreak => {
            output.write_str(" ")?;
        }
        NodeValue::HardBreak => {
            output.write_str("\\n")?;
        }
        NodeValue::Link(link) => {
            output.write_str("#link(\"")?;
            output.write_str(&link.url)?;
            output.write_str("\")[")?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_typst(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("]")?;
        }
        _ => {
            // For other nodes, just render children
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_typst(child, output)?;
                child_opt = child.next_sibling();
            }
        }
    }

    Ok(())
}
