//! HTML renderer (deprecated)
//!
//! ⚠️ **DEPRECATED**: This module is deprecated. Use `crate::render::arena_html` instead.
//!
//! This module uses the old Rc<RefCell>-based AST. It will be removed in a future version.

use crate::html_utils::{escape_html, is_safe_url};
use crate::iterator::{NodeWalker, WalkerEvent};
use crate::node::{Node, NodeData, NodeType, SourcePos};
use htmlescape::decode_html;
use std::cell::RefCell;
use std::rc::Rc;

/// Decode HTML entities in a string
fn decode_entities(input: &str) -> String {
    // Simple entity decoding for common cases
    // This is a simplified version - for full support, we'd need to use the same
    // logic as the inline parser
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '&' {
            // Try to find a semicolon to complete the entity
            let mut entity = String::new();
            entity.push(c);

            while let Some(&next_c) = chars.peek() {
                if next_c == ';' {
                    entity.push(next_c);
                    chars.next();
                    break;
                }
                if next_c.is_ascii_alphanumeric() || next_c == '#' {
                    entity.push(next_c);
                    chars.next();
                } else {
                    break;
                }
            }

            // Try to decode the entity
            if entity.ends_with(';') {
                if let Ok(decoded) = decode_html(&entity) {
                    if decoded != entity {
                        result.push_str(&decoded);
                        continue;
                    }
                }
            }

            // If decoding failed, keep the original
            result.push_str(&entity);
        } else {
            result.push(c);
        }
    }

    result
}

/// Render a node tree as HTML
pub fn render(root: &Rc<RefCell<Node>>, options: u32) -> String {
    let mut renderer = HtmlRenderer::new(options);
    renderer.render(root)
}

struct HtmlRenderer {
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

impl HtmlRenderer {
    fn new(options: u32) -> Self {
        HtmlRenderer {
            options,
            output: String::new(),
            tight_list_stack: Vec::new(),
            last_out: '\n', // Initialize to newline like commonmark.js
            disable_tags: 0,
            item_child_count: Vec::new(),
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
    /// If disable_tags > 0, HTML tags are stripped
    fn lit(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }

        let output_str = if self.disable_tags > 0 {
            // Strip HTML tags when disable_tags is active
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
    /// Returns true if we should add a newline before this block element
    fn track_item_child(&mut self) -> bool {
        let in_tight_list = self.in_tight_list();
        if let Some(count) = self.item_child_count.last_mut() {
            *count += 1;
            // In tight lists, add newline before block elements after the first one
            if in_tight_list && *count > 1 {
                return true;
            }
        }
        false
    }

    fn render(&mut self, root: &Rc<RefCell<Node>>) -> String {
        let mut walker = NodeWalker::new(root.clone());

        while let Some(event) = walker.next() {
            if event.entering {
                self.enter_node(&event);
            } else {
                self.exit_node(&event);
            }
        }

        // Remove trailing newline to match CommonMark spec test format
        while self.output.ends_with('\n') {
            self.output.pop();
        }

        self.output.clone()
    }

    fn enter_node(&mut self, event: &WalkerEvent) {
        let node = event.node.borrow();

        match node.node_type {
            NodeType::Document => {}
            NodeType::BlockQuote => {
                // In tight list items, add newline before blockquote if not first child
                if self.track_item_child() {
                    self.lit("\n");
                } else {
                    self.cr(); // Add newline before code block if needed
                }
                self.lit("<blockquote");
                self.add_sourcepos(&node.source_pos);
                self.lit(">\n");
                // Push false to tight_list_stack to disable tight mode for blockquote contents
                // Blockquotes inside tight lists should still render <p> tags for their content
                self.tight_list_stack.push(false);
            }
            NodeType::List => {
                if let NodeData::List {
                    list_type, tight, ..
                } = &node.data
                {
                    // Push tight status to stack
                    self.tight_list_stack.push(*tight);
                    self.cr(); // Add newline before list if needed (for nested lists)
                    match list_type {
                        crate::node::ListType::Bullet => {
                            self.lit("<ul");
                            self.add_sourcepos(&node.source_pos);
                            self.lit(">\n");
                        }
                        crate::node::ListType::Ordered => {
                            self.lit("<ol");
                            self.add_sourcepos(&node.source_pos);
                            // Add start attribute if not 1
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
                // In loose lists, add newline after <li>, but not for empty items
                // Empty items have no children
                let has_children = node.first_child.borrow().is_some();
                if !self.in_tight_list() && has_children {
                    self.lit("\n");
                }
                // Initialize child counter for this item
                self.item_child_count.push(0);
            }
            NodeType::CodeBlock => {
                // In tight list items, add newline before code block if not first child
                if self.track_item_child() {
                    self.lit("\n");
                } else {
                    self.cr(); // Add newline before code block if needed
                }
                self.lit("<pre");
                self.add_sourcepos(&node.source_pos);
                self.lit("><code");
                if let NodeData::CodeBlock { info, .. } = &node.data {
                    if !info.is_empty() {
                        // Decode entities in info string per CommonMark spec
                        let decoded_info = decode_entities(info);
                        let lang = decoded_info.split_whitespace().next().unwrap_or("");
                        if !lang.is_empty() {
                            // Check if lang already starts with "language-"
                            // Per commonmark.js: if (!/^language-/.exec(cls)) { cls = "language-" + cls; }
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
                // In tight list items, add newline before HTML block if not first child
                if self.track_item_child() {
                    self.lit("\n");
                } else {
                    self.cr();
                }
                // HTML blocks are always output as raw HTML
                // They are not subject to the same security restrictions as inline HTML
                if let NodeData::HtmlBlock { literal } = &node.data {
                    self.lit(literal);
                }
                self.lit("\n");
            }
            NodeType::Paragraph => {
                // In tight lists, paragraphs are not wrapped in <p> tags
                if !self.in_tight_list() {
                    // Track as item child in loose lists too
                    self.track_item_child();
                    self.lit("<p");
                    self.add_sourcepos(&node.source_pos);
                    self.lit(">");
                }
            }
            NodeType::Heading => {
                // Add newline before heading (like commonmark.js cr() function)
                // Only add newline if last output wasn't already a newline
                self.cr();
                if let NodeData::Heading { level, .. } = &node.data {
                    self.lit("<h");
                    self.lit(&level.to_string());
                    self.add_sourcepos(&node.source_pos);
                    self.lit(">");
                }
            }
            NodeType::ThematicBreak => {
                // In tight list items, add newline before thematic break if not first child
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
                // Track text nodes as item children in tight lists
                // This ensures proper newline handling for subsequent block elements
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
                // HtmlInline is output as raw HTML
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
                    // We're inside an image's alt text
                    // Links in alt text are replaced by their link text (not rendered as <a>)
                    // Just continue to render the children
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
                    // We're inside another image's alt text
                    // Images in alt text are replaced by their alt text (not rendered as <img>)
                    // Just disable tags for the nested alt text processing
                    self.disable_tags += 1;
                } else if let NodeData::Image { url, .. } = &node.data {
                    if self.options & crate::options::UNSAFE != 0 || is_safe_url(url) {
                        self.lit("<img src=\"");
                        self.lit(&escape_html(url));
                        self.lit("\" alt=\"");
                    } else {
                        self.lit("<img src=\"\" alt=\"");
                    }
                    // Disable tag rendering for alt text
                    self.disable_tags += 1;
                }
            }
            _ => {}
        }
    }

    fn exit_node(&mut self, event: &WalkerEvent) {
        let node = event.node.borrow();

        match node.node_type {
            NodeType::Document => {}
            NodeType::BlockQuote => {
                self.lit("</blockquote>\n");
                // Pop the false we pushed when entering blockquote
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
                // Pop tight status from stack
                self.tight_list_stack.pop();
            }
            NodeType::Item => {
                self.lit("</li>\n");
                // Pop child counter for this item
                self.item_child_count.pop();
            }
            NodeType::CodeBlock => {}
            NodeType::HtmlBlock => {}
            NodeType::Paragraph => {
                // In tight lists, paragraphs are not wrapped in <p> tags
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
                // Re-enable tag rendering after alt text
                self.disable_tags -= 1;
                // Only output closing tag if we're not inside another image's alt text
                if self.disable_tags == 0 {
                    // Add title attribute after alt if present
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
            _ => {}
        }
    }

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
    use crate::node::{append_child, Node, NodeData, NodeType};

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\"test\""), "&quot;test&quot;");
    }

    #[test]
    fn test_is_safe_url() {
        assert!(is_safe_url("https://example.com"));
        assert!(!is_safe_url("javascript:alert('xss')"));
        assert!(!is_safe_url("data:text/html,<script>alert('xss')</script>"));
    }

    #[test]
    fn test_render_paragraph() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello world".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, text.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<p>Hello world</p>");
    }

    #[test]
    fn test_render_emph() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let emph = Rc::new(RefCell::new(Node::new(NodeType::Emph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "emphasized".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, emph.clone());
        append_child(&emph, text.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<p><em>emphasized</em></p>");
    }

    #[test]
    fn test_render_strong() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let strong = Rc::new(RefCell::new(Node::new(NodeType::Strong)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "strong".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, strong.clone());
        append_child(&strong, text.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<p><strong>strong</strong></p>");
    }

    #[test]
    fn test_render_code() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let code = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Code,
            NodeData::Code {
                literal: "code".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, code.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<p><code>code</code></p>");
    }

    #[test]
    fn test_render_heading() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let heading = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 1,
                content: "Title".to_string(),
            },
        )));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Title".to_string(),
            },
        )));

        append_child(&root, heading.clone());
        append_child(&heading, text.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<h1>Title</h1>");
    }

    #[test]
    fn test_render_heading_level_3() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let heading = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 3,
                content: "Section".to_string(),
            },
        )));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Section".to_string(),
            },
        )));

        append_child(&root, heading.clone());
        append_child(&heading, text.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<h3>Section</h3>");
    }

    #[test]
    fn test_render_blockquote() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let blockquote = Rc::new(RefCell::new(Node::new(NodeType::BlockQuote)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Quote".to_string(),
            },
        )));

        append_child(&root, blockquote.clone());
        append_child(&blockquote, para.clone());
        append_child(&para, text.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<blockquote>\n<p>Quote</p>\n</blockquote>");
    }

    #[test]
    fn test_render_code_block() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let code_block = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::CodeBlock,
            NodeData::CodeBlock {
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
            },
        )));

        append_child(&root, code_block.clone());

        let html = render(&root, 0);
        assert_eq!(
            html,
            "<pre><code class=\"language-rust\">fn main() {}</code></pre>"
        );
    }

    #[test]
    fn test_render_code_block_no_lang() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let code_block = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::CodeBlock,
            NodeData::CodeBlock {
                info: "".to_string(),
                literal: "code".to_string(),
            },
        )));

        append_child(&root, code_block.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<pre><code>code</code></pre>");
    }

    #[test]
    fn test_render_thematic_break() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let hr = Rc::new(RefCell::new(Node::new(NodeType::ThematicBreak)));

        append_child(&root, hr.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<hr />");
    }

    #[test]
    fn test_render_link() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let link = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Link,
            NodeData::Link {
                url: "https://example.com".to_string(),
                title: "".to_string(),
            },
        )));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "link".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, link.clone());
        append_child(&link, text.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<p><a href=\"https://example.com\">link</a></p>");
    }

    #[test]
    fn test_render_link_with_title() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let link = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Link,
            NodeData::Link {
                url: "https://example.com".to_string(),
                title: "Title".to_string(),
            },
        )));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "link".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, link.clone());
        append_child(&link, text.clone());

        let html = render(&root, 0);
        assert_eq!(
            html,
            "<p><a href=\"https://example.com\" title=\"Title\">link</a></p>"
        );
    }

    #[test]
    fn test_render_image() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let image = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Image,
            NodeData::Image {
                url: "image.png".to_string(),
                title: "".to_string(),
            },
        )));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "alt".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, image.clone());
        append_child(&image, text.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<p><img src=\"image.png\" alt=\"alt\" /></p>");
    }

    #[test]
    fn test_render_soft_break() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text1 = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello".to_string(),
            },
        )));
        let soft_break = Rc::new(RefCell::new(Node::new(NodeType::SoftBreak)));
        let text2 = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "world".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, text1.clone());
        append_child(&para, soft_break.clone());
        append_child(&para, text2.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<p>Hello\nworld</p>");
    }

    #[test]
    fn test_render_line_break() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text1 = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello".to_string(),
            },
        )));
        let line_break = Rc::new(RefCell::new(Node::new(NodeType::LineBreak)));
        let text2 = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "world".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, text1.clone());
        append_child(&para, line_break.clone());
        append_child(&para, text2.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<p>Hello<br />\nworld</p>");
    }

    #[test]
    fn test_render_html_block() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let html_block = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::HtmlBlock,
            NodeData::HtmlBlock {
                literal: "<div>content</div>".to_string(),
            },
        )));

        append_child(&root, html_block.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<div>content</div>");
    }

    #[test]
    fn test_render_html_inline() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let html_inline = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::HtmlInline,
            NodeData::HtmlInline {
                literal: "<br />".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, html_inline.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<p><br /></p>");
    }

    #[test]
    fn test_render_with_sourcepos() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        {
            let mut para_mut = para.borrow_mut();
            para_mut.source_pos.start_line = 1;
            para_mut.source_pos.start_column = 1;
            para_mut.source_pos.end_line = 1;
            para_mut.source_pos.end_column = 10;
        }
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, text.clone());

        let html = render(&root, crate::options::SOURCEPOS);
        assert!(html.contains("data-sourcepos"));
        assert!(html.contains("1:1-1:10"));
    }

    #[test]
    fn test_render_unsafe_url_blocked() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let link = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Link,
            NodeData::Link {
                url: "javascript:alert('xss')".to_string(),
                title: "".to_string(),
            },
        )));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "click".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, link.clone());
        append_child(&link, text.clone());

        let html = render(&root, 0);
        assert_eq!(html, "<p><a href=\"\">click</a></p>");
    }

    #[test]
    fn test_render_unsafe_option_allows_url() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let link = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Link,
            NodeData::Link {
                url: "javascript:alert('xss')".to_string(),
                title: "".to_string(),
            },
        )));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "click".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, link.clone());
        append_child(&link, text.clone());

        let html = render(&root, crate::options::UNSAFE);
        assert_eq!(html, "<p><a href=\"javascript:alert('xss')\">click</a></p>");
    }
}
