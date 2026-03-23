use crate::iterator::{NodeWalker, WalkerEvent};
use crate::node::{Node, NodeData, NodeType, SourcePos};
use crate::render::{escape_html, is_safe_url};
use std::cell::RefCell;
use std::rc::Rc;

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
}

impl HtmlRenderer {
    fn new(options: u32) -> Self {
        HtmlRenderer {
            options,
            output: String::new(),
            tight_list_stack: Vec::new(),
            last_out: '\n', // Initialize to newline like commonmark.js
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
        if !s.is_empty() {
            self.output.push_str(s);
            self.last_out = s.chars().last().unwrap_or('\n');
        }
    }

    /// Check if we're currently inside a tight list
    fn in_tight_list(&self) -> bool {
        self.tight_list_stack.last().copied().unwrap_or(false)
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
                self.lit("<blockquote");
                self.add_sourcepos(&node.source_pos);
                self.lit(">\n");
            }
            NodeType::List => {
                if let NodeData::List { list_type, tight, .. } = &node.data {
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
                // In loose lists, add newline after <li>
                if !self.in_tight_list() {
                    self.lit("\n");
                }
            }
            NodeType::CodeBlock => {
                self.cr(); // Add newline before code block if needed
                self.lit("<pre");
                self.add_sourcepos(&node.source_pos);
                self.lit("><code");
                if let NodeData::CodeBlock { info, .. } = &node.data {
                    if !info.is_empty() {
                        let lang = info.split_whitespace().next().unwrap_or("");
                        if !lang.is_empty() {
                            self.lit(" class=\"language-");
                            self.lit(&escape_html(lang));
                            self.lit("\"");
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
                self.cr();
                if self.options & crate::options::UNSAFE != 0 {
                    if let NodeData::HtmlBlock { literal } = &node.data {
                        self.lit(literal);
                    }
                } else {
                    self.lit("<!-- raw HTML omitted -->");
                }
                self.lit("\n");
            }
            NodeType::Paragraph => {
                // In tight lists, paragraphs are not wrapped in <p> tags
                if !self.in_tight_list() {
                    self.lit("<p");
                    self.add_sourcepos(&node.source_pos);
                    self.lit(">");
                }
            }
            NodeType::Heading => {
                if let NodeData::Heading { level } = &node.data {
                    self.lit("<h");
                    self.lit(&level.to_string());
                    self.add_sourcepos(&node.source_pos);
                    self.lit(">");
                }
            }
            NodeType::ThematicBreak => {
                self.lit("<hr");
                self.add_sourcepos(&node.source_pos);
                self.lit(" />\n");
            }
            NodeType::Text => {
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
                if self.options & crate::options::UNSAFE != 0 {
                    if let NodeData::HtmlInline { literal } = &node.data {
                        self.lit(literal);
                    }
                } else {
                    self.lit("<!-- raw HTML omitted -->");
                }
            }
            NodeType::Emph => {
                self.lit("<em>");
            }
            NodeType::Strong => {
                self.lit("<strong>");
            }
            NodeType::Link => {
                if let NodeData::Link { url, title } = &node.data {
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
                if let NodeData::Image { url, title } = &node.data {
                    if self.options & crate::options::UNSAFE != 0 || is_safe_url(url) {
                        self.lit("<img src=\"");
                        self.lit(&escape_html(url));
                        self.lit("\"");
                        if !title.is_empty() {
                            self.lit(" title=\"");
                            self.lit(&escape_html(title));
                            self.lit("\"");
                        }
                        self.lit(" alt=\"");
                        // alt text will be filled by children
                    } else {
                        self.lit("<img src=\"\" alt=\"");
                    }
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
            }
            NodeType::Item => {
                self.lit("</li>\n");
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
                if let NodeData::Heading { level } = &node.data {
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
                self.lit("</a>");
            }
            NodeType::Image => {
                self.lit("\" />");
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
}
