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
}

impl HtmlRenderer {
    fn new(options: u32) -> Self {
        HtmlRenderer {
            options,
            output: String::new(),
            tight_list_stack: Vec::new(),
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
                self.output.push_str("<blockquote");
                self.add_sourcepos(&node.source_pos);
                self.output.push_str(">\n");
            }
            NodeType::List => {
                if let NodeData::List { list_type, tight, .. } = &node.data {
                    // Push tight status to stack
                    self.tight_list_stack.push(*tight);
                    match list_type {
                        crate::node::ListType::Bullet => {
                            self.output.push_str("<ul");
                            self.add_sourcepos(&node.source_pos);
                            self.output.push_str(">\n");
                        }
                        crate::node::ListType::Ordered => {
                            self.output.push_str("<ol");
                            self.add_sourcepos(&node.source_pos);
                            self.output.push_str(">\n");
                        }
                        _ => {}
                    }
                }
            }
            NodeType::Item => {
                self.output.push_str("<li");
                self.add_sourcepos(&node.source_pos);
                self.output.push_str(">");
                // In tight lists, don't add newline after <li>
                if !self.in_tight_list() {
                    self.output.push('\n');
                }
            }
            NodeType::CodeBlock => {
                self.output.push_str("<pre");
                self.add_sourcepos(&node.source_pos);
                self.output.push_str("><code");
                if let NodeData::CodeBlock { info, .. } = &node.data {
                    if !info.is_empty() {
                        let lang = info.split_whitespace().next().unwrap_or("");
                        if !lang.is_empty() {
                            self.output.push_str(" class=\"language-");
                            self.output.push_str(&escape_html(lang));
                            self.output.push('"');
                        }
                    }
                }
                self.output.push_str(">");
                if let NodeData::CodeBlock { literal, .. } = &node.data {
                    self.output.push_str(&escape_html(literal));
                }
                self.output.push_str("</code></pre>\n");
            }
            NodeType::HtmlBlock => {
                if self.options & crate::options::UNSAFE != 0 {
                    if let NodeData::HtmlBlock { literal } = &node.data {
                        self.output.push_str(literal);
                    }
                } else {
                    self.output.push_str("<!-- raw HTML omitted -->");
                }
                self.output.push('\n');
            }
            NodeType::Paragraph => {
                // In tight lists, paragraphs are not wrapped in <p> tags
                if !self.in_tight_list() {
                    self.output.push_str("<p");
                    self.add_sourcepos(&node.source_pos);
                    self.output.push_str(">");
                }
            }
            NodeType::Heading => {
                if let NodeData::Heading { level } = &node.data {
                    self.output.push_str("<h");
                    self.output.push_str(&level.to_string());
                    self.add_sourcepos(&node.source_pos);
                    self.output.push_str(">");
                }
            }
            NodeType::ThematicBreak => {
                self.output.push_str("<hr");
                self.add_sourcepos(&node.source_pos);
                self.output.push_str(" />\n");
            }
            NodeType::Text => {
                if let NodeData::Text { literal } = &node.data {
                    self.output.push_str(&escape_html(literal));
                }
            }
            NodeType::SoftBreak => {
                if self.options & crate::options::HARDBREAKS != 0 {
                    self.output.push_str("<br />\n");
                } else if self.options & crate::options::NOBREAKS != 0 {
                    self.output.push(' ');
                } else {
                    self.output.push('\n');
                }
            }
            NodeType::LineBreak => {
                self.output.push_str("<br />\n");
            }
            NodeType::Code => {
                self.output.push_str("<code>");
                if let NodeData::Code { literal } = &node.data {
                    self.output.push_str(&escape_html(literal));
                }
                self.output.push_str("</code>");
            }
            NodeType::HtmlInline => {
                if self.options & crate::options::UNSAFE != 0 {
                    if let NodeData::HtmlInline { literal } = &node.data {
                        self.output.push_str(literal);
                    }
                } else {
                    self.output.push_str("<!-- raw HTML omitted -->");
                }
            }
            NodeType::Emph => {
                self.output.push_str("<em>");
            }
            NodeType::Strong => {
                self.output.push_str("<strong>");
            }
            NodeType::Link => {
                if let NodeData::Link { url, title } = &node.data {
                    if self.options & crate::options::UNSAFE != 0 || is_safe_url(url) {
                        self.output.push_str("<a href=\"");
                        self.output.push_str(&escape_html(url));
                        self.output.push('"');
                        if !title.is_empty() {
                            self.output.push_str(" title=\"");
                            self.output.push_str(&escape_html(title));
                            self.output.push('"');
                        }
                        self.output.push('>');
                    } else {
                        self.output.push_str("<a href=\"\">");
                    }
                }
            }
            NodeType::Image => {
                if let NodeData::Image { url, title } = &node.data {
                    if self.options & crate::options::UNSAFE != 0 || is_safe_url(url) {
                        self.output.push_str("<img src=\"");
                        self.output.push_str(&escape_html(url));
                        self.output.push('"');
                        if !title.is_empty() {
                            self.output.push_str(" title=\"");
                            self.output.push_str(&escape_html(title));
                            self.output.push('"');
                        }
                        self.output.push_str(" alt=\"");
                        // alt text will be filled by children
                    } else {
                        self.output.push_str("<img src=\"\" alt=\"");
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
                self.output.push_str("</blockquote>\n");
            }
            NodeType::List => {
                if let NodeData::List { list_type, .. } = &node.data {
                    match list_type {
                        crate::node::ListType::Bullet => {
                            self.output.push_str("</ul>\n");
                        }
                        crate::node::ListType::Ordered => {
                            self.output.push_str("</ol>\n");
                        }
                        _ => {}
                    }
                }
            }
            NodeType::Item => {
                self.output.push_str("</li>");
                // In tight lists, don't add newline before </li>
                if !self.in_tight_list() {
                    self.output.push('\n');
                }
            }
            NodeType::CodeBlock => {}
            NodeType::HtmlBlock => {}
            NodeType::Paragraph => {
                // In tight lists, paragraphs are not wrapped in <p> tags
                if !self.in_tight_list() {
                    self.output.push_str("</p>\n");
                }
            }
            NodeType::Heading => {
                if let NodeData::Heading { level } = &node.data {
                    self.output.push_str("</h");
                    self.output.push_str(&level.to_string());
                    self.output.push_str(">\n");
                }
            }
            NodeType::ThematicBreak => {}
            NodeType::Text => {}
            NodeType::SoftBreak => {}
            NodeType::LineBreak => {}
            NodeType::Code => {}
            NodeType::HtmlInline => {}
            NodeType::Emph => {
                self.output.push_str("</em>");
            }
            NodeType::Strong => {
                self.output.push_str("</strong>");
            }
            NodeType::Link => {
                self.output.push_str("</a>");
            }
            NodeType::Image => {
                self.output.push_str("\" />");
            }
            _ => {}
        }
    }

    fn add_sourcepos(&mut self, source_pos: &SourcePos) {
        if self.options & crate::options::SOURCEPOS != 0 {
            self.output.push_str(&format!(
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
