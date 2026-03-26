//! HTML renderer
//!
//! This module provides HTML rendering for documents parsed using the Arena-based parser.
//!
//! # Example
//!
//! ```
//! use clmd::{parse_document, render_html, options};
//!
//! let (arena, doc) = parse_document("# Hello", options::DEFAULT);
//! let html = render_html(&arena, doc, options::DEFAULT);
//! assert_eq!(html, "<h1>Hello</h1>");
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::html_utils::entities::decode_entities;
use crate::html_utils::{escape_html, is_safe_url};
use crate::node::{NodeData, NodeType, SourcePos};

/// Render an Arena-based AST to HTML
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Rendering options
///
/// # Returns
///
/// The HTML output as a String
pub fn render(arena: &NodeArena, root: NodeId, options: u32) -> String {
    let mut renderer = HtmlRenderer::new(arena, options);
    renderer.render_node(root, true);
    renderer.finish()
}

/// HTML renderer for Arena-based AST
struct HtmlRenderer<'a> {
    arena: &'a NodeArena,
    options: u32,
    output: String,
    /// Stack tracking whether we're inside a tight list
    tight_list_stack: Vec<bool>,
    /// Track the last output character for cr() logic
    last_out: char,
    /// Counter to disable tag rendering (for image alt text)
    disable_tags: i32,
    /// Track if we're at the first child of a list item (for tight lists)
    item_child_count: Vec<usize>,
}

impl<'a> HtmlRenderer<'a> {
    fn new(arena: &'a NodeArena, options: u32) -> Self {
        HtmlRenderer {
            arena,
            options,
            output: String::new(),
            tight_list_stack: Vec::new(),
            last_out: '\n',
            disable_tags: 0,
            item_child_count: Vec::new(),
        }
    }

    /// Render a node and its children
    fn render_node(&mut self, node_id: NodeId, entering: bool) {
        let node = self.arena.get(node_id);

        if entering {
            self.enter_node(node_id);

            // Render children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                self.render_node(child_id, true);
                child_opt = self.arena.get(child_id).next;
            }

            self.exit_node(node_id);
        }
    }

    /// Enter a node - output opening tags
    fn enter_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        match node.node_type {
            NodeType::Document => {}
            NodeType::BlockQuote => {
                if self.track_item_child() {
                    self.lit("\n");
                } else {
                    self.cr();
                }
                self.lit("<blockquote");
                self.add_sourcepos(&node.source_pos);
                self.lit(">\n");
                self.tight_list_stack.push(false);
            }
            NodeType::List => {
                if let NodeData::List {
                    list_type, tight, ..
                } = &node.data
                {
                    self.tight_list_stack.push(*tight);
                    self.cr();
                    match list_type {
                        crate::node::ListType::Bullet => {
                            self.lit("<ul");
                            self.add_sourcepos(&node.source_pos);
                            self.lit(">\n");
                        }
                        crate::node::ListType::Ordered => {
                            self.lit("<ol");
                            self.add_sourcepos(&node.source_pos);
                            if let NodeData::List { start, .. } = &node.data {
                                if *start != 1 {
                                    self.lit(&format!(" start=\"{}\"", start));
                                }
                            }
                            self.lit(">\n");
                        }
                        _ => {}
                    }
                }
            }
            NodeType::Item => {
                self.lit("<li");
                self.add_sourcepos(&node.source_pos);
                self.lit(">");
                let has_children = node.first_child.is_some();
                if !self.in_tight_list() && has_children {
                    self.lit("\n");
                }
                self.item_child_count.push(0);
            }
            NodeType::CodeBlock => {
                if self.track_item_child() {
                    self.lit("\n");
                } else {
                    self.cr();
                }
                self.lit("<pre");
                self.add_sourcepos(&node.source_pos);
                self.lit("><code");
                if let NodeData::CodeBlock { info, .. } = &node.data {
                    if !info.is_empty() {
                        let decoded_info = decode_entities(info);
                        let lang = decoded_info.split_whitespace().next().unwrap_or("");
                        if !lang.is_empty() {
                            if lang.starts_with("language-") {
                                self.lit(" class=\"");
                                self.lit(&escape_html(lang));
                                self.lit("\"");
                            } else {
                                self.lit(" class=\"language-");
                                self.lit(&escape_html(lang));
                                self.lit("\"");
                            }
                        }
                    }
                }
                self.lit(">");
                if let NodeData::CodeBlock { literal, .. } = &node.data {
                    self.lit(&escape_html(literal));
                }
                self.lit("</code></pre>\n");
            }
            NodeType::HtmlBlock => {
                if self.track_item_child() {
                    self.lit("\n");
                } else {
                    self.cr();
                }
                if let NodeData::HtmlBlock { literal } = &node.data {
                    self.lit(literal);
                }
                self.lit("\n");
            }
            NodeType::Paragraph => {
                if !self.in_tight_list() {
                    self.track_item_child();
                    self.lit("<p");
                    self.add_sourcepos(&node.source_pos);
                    self.lit(">");
                }
            }
            NodeType::Heading => {
                self.cr();
                if let NodeData::Heading { level, .. } = &node.data {
                    self.lit("<h");
                    self.lit(&level.to_string());
                    self.add_sourcepos(&node.source_pos);
                    self.lit(">");
                }
            }
            NodeType::ThematicBreak => {
                if self.track_item_child() {
                    self.lit("\n");
                } else {
                    self.cr();
                }
                self.lit("<hr");
                self.add_sourcepos(&node.source_pos);
                self.lit(" />\n");
            }
            NodeType::Text => {
                if self.in_tight_list() && !self.item_child_count.is_empty() {
                    self.track_item_child();
                }
                if let NodeData::Text { literal } = &node.data {
                    self.lit(&escape_html(literal));
                }
            }
            NodeType::SoftBreak => {
                if self.options & crate::options::HARDBREAKS != 0 {
                    self.lit("<br />\n");
                } else if self.options & crate::options::NOBREAKS != 0 {
                    self.lit(" ");
                } else {
                    self.lit("\n");
                }
            }
            NodeType::LineBreak => {
                self.lit("<br />\n");
            }
            NodeType::Code => {
                self.lit("<code>");
                if let NodeData::Code { literal } = &node.data {
                    self.lit(&escape_html(literal));
                }
                self.lit("</code>");
            }
            NodeType::HtmlInline => {
                if let NodeData::HtmlInline { literal } = &node.data {
                    self.lit(literal);
                }
            }
            NodeType::Emph => {
                self.lit("<em>");
            }
            NodeType::Strong => {
                self.lit("<strong>");
            }
            NodeType::Link => {
                if self.disable_tags > 0 {
                    // Inside image alt text, just render children
                } else if let NodeData::Link { url, title } = &node.data {
                    if self.options & crate::options::UNSAFE != 0 || is_safe_url(url) {
                        self.lit("<a href=\"");
                        self.lit(&escape_html(url));
                        self.lit("\"");
                        if !title.is_empty() {
                            self.lit(" title=\"");
                            self.lit(&escape_html(title));
                            self.lit("\"");
                        }
                        self.lit(">");
                    } else {
                        self.lit("<a href=\"\">");
                    }
                }
            }
            NodeType::Image => {
                if self.disable_tags > 0 {
                    self.disable_tags += 1;
                } else if let NodeData::Image { url, .. } = &node.data {
                    if self.options & crate::options::UNSAFE != 0 || is_safe_url(url) {
                        self.lit("<img src=\"");
                        self.lit(&escape_html(url));
                        self.lit("\" alt=\"");
                    } else {
                        self.lit("<img src=\"\" alt=\"");
                    }
                    self.disable_tags += 1;
                }
            }
            NodeType::Table
            | NodeType::TableHead
            | NodeType::TableRow
            | NodeType::TableCell => {
                // Table rendering not yet implemented
            }
            NodeType::Strikethrough => {
                self.lit("<del>");
            }
            NodeType::TaskItem => {
                // Task item rendering
            }
            NodeType::FootnoteRef | NodeType::FootnoteDef => {
                // Footnote rendering not yet implemented
            }
            NodeType::CustomBlock | NodeType::CustomInline => {
                // Custom nodes not yet implemented
            }
            NodeType::None => {}
        }
    }

    /// Exit a node - output closing tags
    fn exit_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        match node.node_type {
            NodeType::Document => {}
            NodeType::BlockQuote => {
                self.lit("</blockquote>\n");
                self.tight_list_stack.pop();
            }
            NodeType::List => {
                if let NodeData::List { list_type, .. } = &node.data {
                    match list_type {
                        crate::node::ListType::Bullet => {
                            self.lit("</ul>\n");
                        }
                        crate::node::ListType::Ordered => {
                            self.lit("</ol>\n");
                        }
                        _ => {}
                    }
                }
                self.tight_list_stack.pop();
            }
            NodeType::Item => {
                self.lit("</li>\n");
                self.item_child_count.pop();
            }
            NodeType::CodeBlock => {}
            NodeType::HtmlBlock => {}
            NodeType::Paragraph => {
                if !self.in_tight_list() {
                    self.lit("</p>\n");
                }
            }
            NodeType::Heading => {
                if let NodeData::Heading { level, .. } = &node.data {
                    self.lit("</h");
                    self.lit(&level.to_string());
                    self.lit(">\n");
                }
            }
            NodeType::ThematicBreak => {}
            NodeType::Text => {}
            NodeType::SoftBreak => {}
            NodeType::LineBreak => {}
            NodeType::Code => {}
            NodeType::HtmlInline => {}
            NodeType::Emph => {
                self.lit("</em>");
            }
            NodeType::Strong => {
                self.lit("</strong>");
            }
            NodeType::Link => {
                if self.disable_tags == 0 {
                    self.lit("</a>");
                }
            }
            NodeType::Image => {
                self.disable_tags -= 1;
                if self.disable_tags == 0 {
                    if let NodeData::Image { title, .. } = &node.data {
                        if !title.is_empty() {
                            self.lit("\" title=\"");
                            self.lit(&escape_html(title));
                            self.lit("\" />");
                        } else {
                            self.lit("\" />");
                        }
                    } else {
                        self.lit("\" />");
                    }
                }
            }
            NodeType::Table
            | NodeType::TableHead
            | NodeType::TableRow
            | NodeType::TableCell => {}
            NodeType::Strikethrough => {
                self.lit("</del>");
            }
            NodeType::TaskItem => {}
            NodeType::FootnoteRef | NodeType::FootnoteDef => {}
            NodeType::CustomBlock | NodeType::CustomInline => {}
            NodeType::None => {}
        }
    }

    /// Output a newline if the last output wasn't already a newline
    fn cr(&mut self) {
        if self.last_out != '\n' {
            self.output.push('\n');
            self.last_out = '\n';
        }
    }

    /// Output a literal string and track last character
    fn lit(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }

        let output_str = if self.disable_tags > 0 {
            strip_html_tags(s)
        } else {
            s.to_string()
        };

        if !output_str.is_empty() {
            self.output.push_str(&output_str);
            self.last_out = output_str.chars().last().unwrap_or('\n');
        }
    }

    /// Check if we're currently inside a tight list
    fn in_tight_list(&self) -> bool {
        self.tight_list_stack.last().copied().unwrap_or(false)
    }

    /// Check if we're inside a list item and track block-level children
    fn track_item_child(&mut self) -> bool {
        let in_tight_list = self.in_tight_list();
        if let Some(count) = self.item_child_count.last_mut() {
            *count += 1;
            if in_tight_list && *count > 1 {
                return true;
            }
        }
        false
    }

    /// Add source position attribute if enabled
    fn add_sourcepos(&mut self, source_pos: &SourcePos) {
        if self.options & crate::options::SOURCEPOS != 0 {
            self.lit(&format!(
                " data-sourcepos=\"{}:{}-{}:{}\"",
                source_pos.start_line,
                source_pos.start_column,
                source_pos.end_line,
                source_pos.end_column
            ));
        }
    }

    /// Finish rendering and return output
    fn finish(mut self) -> String {
        // Remove trailing newline to match CommonMark spec test format
        while self.output.ends_with('\n') {
            self.output.pop();
        }
        self.output
    }
}

/// Strip HTML tags from a string
/// Used when disable_tags is active (e.g., for image alt text)
fn strip_html_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;

    for c in s.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' && in_tag {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};
    use crate::node::{NodeData, NodeType};

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello world".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render(&arena, root, 0);
        assert_eq!(html, "<p>Hello world</p>");
    }

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let heading = arena.alloc(Node::with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 1,
                content: "Title".to_string(),
            },
        ));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Title".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let html = render(&arena, root, 0);
        assert_eq!(html, "<h1>Title</h1>");
    }

    #[test]
    fn test_render_emphasis() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let emph = arena.alloc(Node::new(NodeType::Emph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "emphasized".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, emph, text);

        let html = render(&arena, root, 0);
        assert_eq!(html, "<p><em>emphasized</em></p>");
    }

    #[test]
    fn test_render_code() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let code = arena.alloc(Node::with_data(
            NodeType::Code,
            NodeData::Code {
                literal: "code".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let html = render(&arena, root, 0);
        assert_eq!(html, "<p><code>code</code></p>");
    }

    #[test]
    fn test_render_code_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let code_block = arena.alloc(Node::with_data(
            NodeType::CodeBlock,
            NodeData::CodeBlock {
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, code_block);

        let html = render(&arena, root, 0);
        assert_eq!(
            html,
            "<pre><code class=\"language-rust\">fn main() {}</code></pre>"
        );
    }

    #[test]
    fn test_render_link() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let link = arena.alloc(Node::with_data(
            NodeType::Link,
            NodeData::Link {
                url: "https://example.com".to_string(),
                title: "".to_string(),
            },
        ));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "link".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let html = render(&arena, root, 0);
        assert_eq!(html, "<p><a href=\"https://example.com\">link</a></p>");
    }
}
