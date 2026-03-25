//! HTML rendering for Arena-based AST
//!
//! This is the Arena-based version of HTML rendering.

use crate::arena::{NodeArena, NodeId, TreeOps};
use crate::node::{NodeData, NodeType};
use std::fmt::Write;

/// HTML renderer for Arena-based AST
pub struct HtmlRenderer;

impl HtmlRenderer {
    /// Render a node and its children to HTML
    pub fn render(arena: &NodeArena, node_id: NodeId) -> String {
        let mut output = String::new();
        Self::render_node(arena, node_id, &mut output);
        output
    }

    fn render_node(arena: &NodeArena, node_id: NodeId, output: &mut String) {
        let node = arena.get(node_id);

        match node.node_type {
            NodeType::Document => {
                Self::render_children(arena, node_id, output);
            }
            NodeType::Paragraph => {
                output.push_str("<p>");
                // Try to get content from block_info if no children
                if arena.get(node_id).first_child.is_none() {
                    if let NodeData::Text { literal } = &node.data {
                        output.push_str(&escape_html(literal));
                    }
                } else {
                    Self::render_children(arena, node_id, output);
                }
                output.push_str("</p>\n");
            }
            NodeType::Heading => {
                if let NodeData::Heading { level, content } = &node.data {
                    write!(output, "<h{}>", level).unwrap();
                    if !content.is_empty() {
                        output.push_str(&escape_html(content));
                    } else {
                        Self::render_children(arena, node_id, output);
                    }
                    write!(output, "</h{}>\n", level).unwrap();
                }
            }
            NodeType::BlockQuote => {
                output.push_str("<blockquote>\n");
                Self::render_children(arena, node_id, output);
                output.push_str("</blockquote>\n");
            }
            NodeType::List => {
                if let NodeData::List { list_type, .. } = &node.data {
                    match list_type {
                        crate::node::ListType::Bullet => {
                            output.push_str("<ul>\n");
                            Self::render_children(arena, node_id, output);
                            output.push_str("</ul>\n");
                        }
                        crate::node::ListType::Ordered => {
                            output.push_str("<ol>\n");
                            Self::render_children(arena, node_id, output);
                            output.push_str("</ol>\n");
                        }
                        _ => {
                            Self::render_children(arena, node_id, output);
                        }
                    }
                }
            }
            NodeType::Item => {
                output.push_str("<li>");
                Self::render_children(arena, node_id, output);
                output.push_str("</li>\n");
            }
            NodeType::CodeBlock => {
                if let NodeData::CodeBlock { info, literal } = &node.data {
                    if info.is_empty() {
                        output.push_str("<pre><code>");
                    } else {
                        write!(
                            output,
                            "<pre><code class=\"language-{}\">",
                            escape_html(info)
                        )
                        .unwrap();
                    }
                    output.push_str(&escape_html(literal));
                    output.push_str("</code></pre>\n");
                }
            }
            NodeType::HtmlBlock => {
                if let NodeData::HtmlBlock { literal } = &node.data {
                    output.push_str(literal);
                }
            }
            NodeType::ThematicBreak => {
                output.push_str("<hr />\n");
            }
            NodeType::Text => {
                if let NodeData::Text { literal } = &node.data {
                    output.push_str(&escape_html(literal));
                }
            }
            NodeType::Code => {
                if let NodeData::Code { literal } = &node.data {
                    output.push_str("<code>");
                    output.push_str(&escape_html(literal));
                    output.push_str("</code>");
                }
            }
            NodeType::Emph => {
                output.push_str("<em>");
                Self::render_children(arena, node_id, output);
                output.push_str("</em>");
            }
            NodeType::Strong => {
                output.push_str("<strong>");
                Self::render_children(arena, node_id, output);
                output.push_str("</strong>");
            }
            NodeType::Link => {
                if let NodeData::Link { url, title } = &node.data {
                    if title.is_empty() {
                        write!(output, "<a href=\"{}\">", escape_html(url)).unwrap();
                    } else {
                        write!(
                            output,
                            "<a href=\"{}\" title=\"{}\">",
                            escape_html(url),
                            escape_html(title)
                        )
                        .unwrap();
                    }
                    Self::render_children(arena, node_id, output);
                    output.push_str("</a>");
                }
            }
            NodeType::Image => {
                if let NodeData::Image { url, title } = &node.data {
                    let alt = Self::collect_text(arena, node_id);
                    if title.is_empty() {
                        write!(
                            output,
                            "<img src=\"{}\" alt=\"{}\" />",
                            escape_html(url),
                            escape_html(&alt)
                        )
                        .unwrap();
                    } else {
                        write!(
                            output,
                            "<img src=\"{}\" alt=\"{}\" title=\"{}\" />",
                            escape_html(url),
                            escape_html(&alt),
                            escape_html(title)
                        )
                        .unwrap();
                    }
                }
            }
            NodeType::HtmlInline => {
                if let NodeData::HtmlInline { literal } = &node.data {
                    output.push_str(literal);
                }
            }
            NodeType::SoftBreak => {
                output.push('\n');
            }
            NodeType::LineBreak => {
                output.push_str("<br />\n");
            }
            _ => {
                // Unknown node type, just render children
                Self::render_children(arena, node_id, output);
            }
        }
    }

    fn render_children(arena: &NodeArena, node_id: NodeId, output: &mut String) {
        if let Some(child_id) = arena.get(node_id).first_child {
            let mut current = Some(child_id);
            while let Some(id) = current {
                Self::render_node(arena, id, output);
                current = TreeOps::next_sibling(arena, id);
            }
        }
    }

    fn collect_text(arena: &NodeArena, node_id: NodeId) -> String {
        let mut result = String::new();
        Self::collect_text_recursive(arena, node_id, &mut result);
        result
    }

    fn collect_text_recursive(arena: &NodeArena, node_id: NodeId, result: &mut String) {
        let node = arena.get(node_id);

        if let NodeData::Text { literal } = &node.data {
            result.push_str(literal);
        }

        if let Some(child_id) = node.first_child {
            let mut current = Some(child_id);
            while let Some(id) = current {
                Self::collect_text_recursive(arena, id, result);
                current = TreeOps::next_sibling(arena, id);
            }
        }
    }
}

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks_arena::BlockParser;

    #[test]
    fn test_render_simple() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "Hello world");

        let html = HtmlRenderer::render(&arena, doc);
        assert!(html.contains("<p>"));
        assert!(html.contains("Hello world"));
        assert!(html.contains("</p>"));
    }

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "# Heading 1");

        let html = HtmlRenderer::render(&arena, doc);
        assert!(html.contains("<h1>"));
        assert!(html.contains("Heading 1"));
        assert!(html.contains("</h1>"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\"test\""), "&quot;test&quot;");
    }
}
