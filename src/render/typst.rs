//! Typst renderer
//!
//! This module provides Typst output generation for CommonMark documents.
//! Typst is a modern markup-based typesetting system.

use crate::arena::{NodeArena, NodeId};
use crate::nodes::NodeValue;
use crate::parser::options::{Options, Plugins};
use std::fmt;

/// Render an AST as Typst.
///
/// This is a convenience function that doesn't use plugins.
pub fn render(arena: &NodeArena, root: NodeId, _options: u32) -> String {
    let mut renderer = TypstRenderer::new(arena);
    renderer.render(root)
}

/// Format an AST as Typst with plugins.
///
/// This is a basic implementation that provides Typst output.
/// Note: Full Typst support is still under development.
pub fn format_document_with_plugins(
    arena: &NodeArena,
    root: NodeId,
    _options: &Options,
    output: &mut dyn fmt::Write,
    _plugins: &Plugins<'_>,
) -> fmt::Result {
    format_node_typst(arena, root, output)
}

fn format_node_typst(
    arena: &NodeArena,
    node_id: NodeId,
    output: &mut dyn fmt::Write,
) -> fmt::Result {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Document => {
            // Render children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output)?;
                child_opt = arena.get(child_id).next;
            }
        }
        NodeValue::Paragraph => {
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output)?;
                child_opt = arena.get(child_id).next;
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
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("\n\n")?;
        }
        NodeValue::Emph => {
            output.write_str("_")?;
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("_")?;
        }
        NodeValue::Strong => {
            output.write_str("*")?;
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output)?;
                child_opt = arena.get(child_id).next;
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
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("\n")?;
        }
        NodeValue::Item(_) => {
            output.write_str("- ")?;
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("\n")?;
        }
        NodeValue::BlockQuote => {
            output.write_str("#quote[\n")?;
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output)?;
                child_opt = arena.get(child_id).next;
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
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output)?;
                child_opt = arena.get(child_id).next;
            }
            output.write_str("]")?;
        }
        _ => {
            // For other nodes, just render children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                format_node_typst(arena, child_id, output)?;
                child_opt = arena.get(child_id).next;
            }
        }
    }

    Ok(())
}

/// Typst renderer state
struct TypstRenderer<'a> {
    arena: &'a NodeArena,
    output: String,
}

impl<'a> TypstRenderer<'a> {
    fn new(arena: &'a NodeArena) -> Self {
        TypstRenderer {
            arena,
            output: String::new(),
        }
    }

    fn render(&mut self, root: NodeId) -> String {
        self.render_node(root);
        self.output.clone()
    }

    fn render_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        match &node.value {
            NodeValue::Document => {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
            }
            NodeValue::Paragraph => {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push_str("\n\n");
            }
            NodeValue::Text(text) => {
                let escaped = text
                    .replace('\\', "\\")
                    .replace('*', "\\*")
                    .replace('_', "\\_")
                    .replace('#', "\\#")
                    .replace('@', "\\@");
                self.output.push_str(&escaped);
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
                self.output.push_str(prefix);
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push_str("\n\n");
            }
            NodeValue::Emph => {
                self.output.push('_');
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push('_');
            }
            NodeValue::Strong => {
                self.output.push('*');
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push('*');
            }
            NodeValue::Code(code) => {
                self.output.push('`');
                self.output.push_str(&code.literal);
                self.output.push('`');
            }
            _ => {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
            }
        }
    }
}
