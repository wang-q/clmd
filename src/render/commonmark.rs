use crate::iterator::{NodeWalker, WalkerEvent};
use crate::node::{DelimType, ListType, Node, NodeData, NodeType};
use std::cell::RefCell;
use std::rc::Rc;

/// Render a node tree as CommonMark
pub fn render(root: &Rc<RefCell<Node>>, options: u32) -> String {
    let mut renderer = CommonMarkRenderer::new(options);
    renderer.render(root)
}

/// CommonMark renderer state
struct CommonMarkRenderer {
    #[allow(dead_code)]
    options: u32,
    output: String,
    /// Current column position for width wrapping
    column: usize,
    /// Whether we're at the beginning of a line
    beginning_of_line: bool,
    /// Stack tracking list item prefixes for indentation
    list_prefixes: Vec<String>,
    /// Stack tracking whether we're in a tight list
    tight_list_stack: Vec<bool>,
    /// Track if we need to add a blank line before next block
    need_blank_line: bool,
}

impl CommonMarkRenderer {
    fn new(options: u32) -> Self {
        CommonMarkRenderer {
            options,
            output: String::new(),
            column: 0,
            beginning_of_line: true,
            list_prefixes: Vec::new(),
            tight_list_stack: Vec::new(),
            need_blank_line: false,
        }
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

        // Remove trailing whitespace and newlines
        while self.output.ends_with('\n') || self.output.ends_with(' ') {
            self.output.pop();
        }

        // Ensure single trailing newline
        self.output.push('\n');

        self.output.clone()
    }

    fn enter_node(&mut self, event: &WalkerEvent) {
        let node = event.node.borrow();

        // Add blank line before block elements if needed
        if self.need_blank_line
            && node.node_type.is_block()
            && !matches!(
                node.node_type,
                NodeType::Document | NodeType::List | NodeType::Item
            )
        {
            self.output.push('\n');
            self.column = 0;
            self.beginning_of_line = true;
            self.need_blank_line = false;
        }

        match node.node_type {
            NodeType::Document => {}
            NodeType::BlockQuote => {
                self.list_prefixes.push("> ".to_string());
                self.beginning_of_line = true;
            }
            NodeType::List => {
                if let NodeData::List { tight, .. } = &node.data {
                    self.tight_list_stack.push(*tight);
                }
                // Don't add blank line before first list
                if !self.output.is_empty() && !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
            }
            NodeType::Item => {
                let prefix = self.format_list_item_prefix(&node);
                self.list_prefixes.push(prefix);
                self.beginning_of_line = true;
            }
            NodeType::CodeBlock => {
                self.render_code_block(&node);
                self.need_blank_line = true;
            }
            NodeType::HtmlBlock => {
                self.render_html_block(&node);
                self.need_blank_line = true;
            }
            NodeType::Paragraph => {
                // In tight lists, don't add blank line before paragraph
                if !self.in_tight_list() && !self.output.is_empty() {
                    self.need_blank_line = true;
                }
            }
            NodeType::Heading => {
                self.render_heading(&node);
                self.need_blank_line = true;
            }
            NodeType::ThematicBreak => {
                self.write_line("***");
                self.need_blank_line = true;
            }
            NodeType::Text => {
                if let NodeData::Text { literal } = &node.data {
                    self.write_inline(&escape_markdown(literal));
                }
            }
            NodeType::SoftBreak => {
                if self.in_tight_list() {
                    self.write_inline(" ");
                } else {
                    self.write_line("");
                }
            }
            NodeType::LineBreak => {
                self.write_inline("  ");
                self.write_line("");
            }
            NodeType::Code => {
                if let NodeData::Code { literal } = &node.data {
                    let backticks = get_backtick_sequence(literal);
                    self.write_inline(&backticks);
                    self.write_inline(literal);
                    self.write_inline(&backticks);
                }
            }
            NodeType::HtmlInline => {
                if let NodeData::HtmlInline { literal } = &node.data {
                    self.write_inline(literal);
                }
            }
            NodeType::Emph => {
                self.write_inline("*");
            }
            NodeType::Strong => {
                self.write_inline("**");
            }
            NodeType::Link => {
                self.write_inline("[");
            }
            NodeType::Image => {
                self.write_inline("![");
            }
            _ => {}
        }
    }

    fn exit_node(&mut self, event: &WalkerEvent) {
        let node = event.node.borrow();

        match node.node_type {
            NodeType::Document => {}
            NodeType::BlockQuote => {
                self.list_prefixes.pop();
                self.need_blank_line = true;
            }
            NodeType::List => {
                self.tight_list_stack.pop();
                self.need_blank_line = true;
            }
            NodeType::Item => {
                self.list_prefixes.pop();
            }
            NodeType::Paragraph => {
                if !self.in_tight_list() {
                    self.write_line("");
                }
            }
            NodeType::Emph => {
                self.write_inline("*");
            }
            NodeType::Strong => {
                self.write_inline("**");
            }
            NodeType::Link => {
                if let NodeData::Link { url, title } = &node.data {
                    self.write_inline("](");
                    self.write_inline(&escape_link_url(url));
                    if !title.is_empty() {
                        self.write_inline(&format!(" \"{}\"", escape_string(title)));
                    }
                    self.write_inline(")");
                }
            }
            NodeType::Image => {
                if let NodeData::Image { url, title } = &node.data {
                    self.write_inline("](");
                    self.write_inline(&escape_link_url(url));
                    if !title.is_empty() {
                        self.write_inline(&format!(" \"{}\"", escape_string(title)));
                    }
                    self.write_inline(")");
                }
            }
            _ => {}
        }
    }

    fn render_code_block(&mut self, node: &std::cell::Ref<Node>) {
        if let NodeData::CodeBlock { info, literal } = &node.data {
            // Determine fence length (must be longer than any backtick sequence in content)
            let mut fence_len = 3;
            for seq in literal.split('\n') {
                let mut count = 0;
                for c in seq.chars() {
                    if c == '`' {
                        count += 1;
                        fence_len = fence_len.max(count + 1);
                    } else {
                        count = 0;
                    }
                }
            }

            let fence: String = std::iter::repeat('`').take(fence_len).collect();
            self.write_line(&fence);

            if !info.is_empty() {
                // Remove trailing backticks from info string
                let clean_info = info.trim_end_matches('`');
                if !clean_info.is_empty() {
                    self.output.pop(); // Remove newline
                    self.write_inline(clean_info);
                    self.write_line("");
                }
            }

            // Write code content
            for line in literal.lines() {
                self.write_line(line);
            }

            self.write_line(&fence);
        }
    }

    fn render_html_block(&mut self, node: &std::cell::Ref<Node>) {
        if let NodeData::HtmlBlock { literal } = &node.data {
            for line in literal.lines() {
                self.write_line(line);
            }
        }
    }

    fn render_heading(&mut self, node: &std::cell::Ref<Node>) {
        if let NodeData::Heading { level, content } = &node.data {
            // Use ATX style headings
            let hashes: String = std::iter::repeat('#').take(*level as usize).collect();
            self.write_inline(&hashes);
            self.write_inline(" ");

            // Render heading content by walking children
            // For now, just output the content field if available
            if !content.is_empty() {
                self.write_line(content);
            } else {
                self.write_line("");
            }
        }
    }

    fn format_list_item_prefix(&self, node: &std::cell::Ref<Node>) -> String {
        // Get the parent list to determine the marker
        if let Some(parent_weak) = node.parent.borrow().as_ref() {
            if let Some(parent) = parent_weak.upgrade() {
                let parent_ref = parent.borrow();
                if let NodeData::List {
                    list_type,
                    delim,
                    start,
                    bullet_char,
                    ..
                } = &parent_ref.data
                {
                    match list_type {
                        ListType::Bullet => {
                            return format!("{} ", bullet_char);
                        }
                        ListType::Ordered => {
                            let marker = match delim {
                                DelimType::Period => format!("{}.", start),
                                DelimType::Paren => format!("{})", start),
                                _ => format!("{}.", start),
                            };
                            // Pad to 4 characters for alignment
                            return format!("{:4}", marker);
                        }
                        _ => {}
                    }
                }
            }
        }
        "- ".to_string()
    }

    fn write_inline(&mut self, text: &str) {
        if self.beginning_of_line {
            self.write_prefixes();
        }
        self.output.push_str(text);
        self.column += text.chars().count();
        self.beginning_of_line = false;
    }

    fn write_line(&mut self, text: &str) {
        if self.beginning_of_line {
            self.write_prefixes();
        }
        self.output.push_str(text);
        self.output.push('\n');
        self.column = 0;
        self.beginning_of_line = true;
    }

    fn write_prefixes(&mut self) {
        for prefix in &self.list_prefixes {
            self.output.push_str(prefix);
            self.column += prefix.chars().count();
        }
    }

    fn in_tight_list(&self) -> bool {
        self.tight_list_stack.last().copied().unwrap_or(false)
    }
}

/// Escape Markdown special characters in text content
fn escape_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let special_chars = [
        '*', '_', '[', ']', '(', ')', '<', '>', '#', '`', '\\', '!', '|',
    ];

    for c in text.chars() {
        if special_chars.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }

    result
}

/// Escape string for use in quotes
fn escape_string(text: &str) -> String {
    text.replace('"', "\\\"").replace('\\', "\\\\")
}

/// Escape URL for use in link destination
fn escape_link_url(url: &str) -> String {
    let mut result = String::with_capacity(url.len());
    let special_chars = ['(', ')', '<', '>', '[', ']', '"', ' ', '\n'];

    for c in url.chars() {
        if special_chars.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }

    result
}

/// Get the appropriate backtick sequence for code content
fn get_backtick_sequence(content: &str) -> String {
    let mut max_backticks = 0;
    let mut current = 0;

    for c in content.chars() {
        if c == '`' {
            current += 1;
            max_backticks = max_backticks.max(current);
        } else {
            current = 0;
        }
    }

    // Use one more backtick than the maximum sequence in content
    let count = (max_backticks + 1).max(1);
    std::iter::repeat('`').take(count).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{append_child, Node, NodeData, NodeType};

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

        let cm = render(&root, 0);
        assert_eq!(cm.trim(), "Hello world");
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

        let cm = render(&root, 0);
        assert_eq!(cm.trim(), "*emphasized*");
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

        let cm = render(&root, 0);
        assert_eq!(cm.trim(), "**strong**");
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

        let cm = render(&root, 0);
        assert_eq!(cm.trim(), "`code`");
    }

    #[test]
    fn test_render_code_with_backticks() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let code = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Code,
            NodeData::Code {
                literal: "code `with` backticks".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, code.clone());

        let cm = render(&root, 0);
        assert_eq!(cm.trim(), "``code `with` backticks``");
    }

    #[test]
    fn test_render_heading() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let heading = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 2,
                content: "Heading".to_string(),
            },
        )));

        append_child(&root, heading.clone());

        let cm = render(&root, 0);
        assert_eq!(cm.trim(), "## Heading");
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

        let cm = render(&root, 0);
        assert_eq!(cm.trim(), "[link](https://example.com)");
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

        let cm = render(&root, 0);
        assert_eq!(cm.trim(), "![alt](image.png)");
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

        let cm = render(&root, 0);
        assert!(cm.contains("> Quote"));
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

        let cm = render(&root, 0);
        assert!(cm.contains("```rust"));
        assert!(cm.contains("fn main() {}"));
        assert!(cm.contains("```"));
    }

    #[test]
    fn test_render_thematic_break() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let hr = Rc::new(RefCell::new(Node::new(NodeType::ThematicBreak)));

        append_child(&root, hr.clone());

        let cm = render(&root, 0);
        assert!(cm.contains("***"));
    }

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("*text*"), "\\*text\\*");
        assert_eq!(escape_markdown("_text_"), "\\_text\\_");
        assert_eq!(escape_markdown("[link]"), "\\[link\\]");
    }
}
