//! Man page renderer (deprecated)
//!
//! ⚠️ **DEPRECATED**: This module is deprecated. Use Arena-based rendering instead.
//!
//! This module uses the old Rc<RefCell>-based AST. It will be removed in a future version.

use crate::iterator::{NodeWalker, WalkerEvent};
use crate::node::{ListType, Node, NodeData, NodeType};
use std::cell::RefCell;
use std::rc::Rc;

/// Render a node tree as a Man page (groff format)
pub fn render(root: &Rc<RefCell<Node>>, options: u32) -> String {
    let mut renderer = ManRenderer::new(options);
    renderer.render(root)
}

/// Man page renderer state
struct ManRenderer {
    #[allow(dead_code)]
    options: u32,
    output: String,
    /// Whether we're at the beginning of a line
    beginning_of_line: bool,
    /// Stack for tracking indentation levels
    indent_stack: Vec<usize>,
    /// Track if we need to add a blank line before next block
    need_blank_line: bool,
    /// Track if we're in a code block (verbatim mode)
    in_verbatim: bool,
    /// Track font style: 0 = normal, 1 = bold, 2 = italic, 3 = monospace
    font_stack: Vec<u8>,
}

impl ManRenderer {
    fn new(options: u32) -> Self {
        ManRenderer {
            options,
            output: String::new(),
            beginning_of_line: true,
            indent_stack: Vec::new(),
            need_blank_line: false,
            in_verbatim: false,
            font_stack: Vec::new(),
        }
    }

    fn render(&mut self, root: &Rc<RefCell<Node>>) -> String {
        // Add man page header
        self.writeln(".TH \"MANUAL\" \"1\" \"\" \"\" \"\"");
        self.writeln("");

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
            self.writeln("");
            self.beginning_of_line = true;
            self.need_blank_line = false;
        }

        match node.node_type {
            NodeType::Document => {}
            NodeType::BlockQuote => {
                self.writeln(".RS");
                self.indent_stack.push(1);
            }
            NodeType::List => {
                if let NodeData::List { .. } = &node.data {
                    self.indent_stack.push(1);
                }
            }
            NodeType::Item => {
                self.write(".IP \"");
                // Get the parent list to determine the marker
                if let Some(parent_weak) = node.parent.borrow().as_ref() {
                    if let Some(parent) = parent_weak.upgrade() {
                        let parent_ref = parent.borrow();
                        if let NodeData::List {
                            list_type,
                            delim,
                            bullet_char,
                            ..
                        } = &parent_ref.data
                        {
                            match list_type {
                                ListType::Bullet => {
                                    self.write(&format!("{}  \"", bullet_char));
                                }
                                ListType::Ordered => {
                                    let marker = match delim {
                                        crate::node::DelimType::Period => "1.",
                                        crate::node::DelimType::Paren => "1)",
                                        _ => "1.",
                                    };
                                    self.write(&format!("{}  \"", marker));
                                }
                                _ => {
                                    self.write("-  \"");
                                }
                            }
                        } else {
                            self.write("-  \"");
                        }
                    } else {
                        self.write("-  \"");
                    }
                } else {
                    self.write("-  \"");
                }
                self.writeln("4");
            }
            NodeType::CodeBlock => {
                self.render_code_block(&node);
                self.need_blank_line = true;
            }
            NodeType::HtmlBlock => {
                // HTML blocks are ignored in man page output
            }
            NodeType::Paragraph => {
                if !self.in_verbatim {
                    self.writeln(".PP");
                }
            }
            NodeType::Heading => {
                self.render_heading(&node);
                self.need_blank_line = true;
            }
            NodeType::ThematicBreak => {
                self.writeln(".PP");
                self.writeln("   *   *   *");
                self.need_blank_line = true;
            }
            NodeType::Text => {
                if let NodeData::Text { literal } = &node.data {
                    if self.in_verbatim {
                        self.write(literal);
                    } else {
                        self.write(&escape_man(literal));
                    }
                }
            }
            NodeType::SoftBreak => {
                if self.in_verbatim {
                    self.writeln("");
                } else {
                    self.write(" ");
                }
            }
            NodeType::LineBreak => {
                if self.in_verbatim {
                    self.writeln("");
                } else {
                    self.writeln(".br");
                }
            }
            NodeType::Code => {
                if let NodeData::Code { literal } = &node.data {
                    self.write("\\fC");
                    self.write(&escape_man(literal));
                    self.write("\\fR");
                }
            }
            NodeType::HtmlInline => {
                // HTML inline is ignored in man page output
            }
            NodeType::Emph => {
                self.write("\\fI");
                self.font_stack.push(2);
            }
            NodeType::Strong => {
                self.write("\\fB");
                self.font_stack.push(1);
            }
            NodeType::Link => {
                // In man pages, links are just shown as text
                // The URL could be shown in footnotes, but for simplicity
                // we just render the link text
            }
            NodeType::Image => {
                // Images are replaced with their alt text in man pages
                self.write("[IMAGE: ");
            }
            _ => {}
        }
    }

    fn exit_node(&mut self, event: &WalkerEvent) {
        let node = event.node.borrow();

        match node.node_type {
            NodeType::Document => {}
            NodeType::BlockQuote => {
                self.writeln(".RE");
                self.indent_stack.pop();
                self.need_blank_line = true;
            }
            NodeType::List => {
                self.indent_stack.pop();
                self.need_blank_line = true;
            }
            NodeType::Item => {
                self.writeln("");
            }
            NodeType::Paragraph => {
                self.writeln("");
            }
            NodeType::Emph => {
                self.reset_font();
            }
            NodeType::Strong => {
                self.reset_font();
            }
            NodeType::Link => {
                // Nothing to do for link exit
            }
            NodeType::Image => {
                self.write("]");
            }
            _ => {}
        }
    }

    fn render_code_block(&mut self, node: &std::cell::Ref<Node>) {
        if let NodeData::CodeBlock { info, literal } = &node.data {
            self.in_verbatim = true;

            if !info.is_empty() {
                let lang = info.split_whitespace().next().unwrap_or("");
                self.write(".PP");
                self.write(&escape_man(lang));
                self.writeln(":");
            }

            self.writeln(".EX");

            // Write code content
            for line in literal.lines() {
                self.writeln(line);
            }

            self.writeln(".EE");
            self.in_verbatim = false;
        }
    }

    fn render_heading(&mut self, node: &std::cell::Ref<Node>) {
        if let NodeData::Heading { level, .. } = &node.data {
            match level {
                1 => self.write(".SH "),
                _ => self.write(".SS "),
            };
            // Content will be added by child text nodes
        }
    }

    fn reset_font(&mut self) {
        self.font_stack.pop();
        if let Some(&font) = self.font_stack.last() {
            match font {
                1 => self.write("\\fB"),
                2 => self.write("\\fI"),
                3 => self.write("\\fC"),
                _ => self.write("\\fR"),
            }
        } else {
            self.write("\\fR");
        }
    }

    fn write(&mut self, text: &str) {
        // Handle special characters at beginning of line
        if self.beginning_of_line && !text.is_empty() {
            let first_char = text.chars().next().unwrap();
            if first_char == '.' || first_char == '\'' {
                // Escape control characters at start of line
                self.output.push_str("\\&");
            }
        }
        self.output.push_str(text);
        self.beginning_of_line = false;
    }

    fn writeln(&mut self, text: &str) {
        // Handle special characters at beginning of line
        if self.beginning_of_line && !text.is_empty() {
            let first_char = text.chars().next().unwrap();
            if first_char == '.' || first_char == '\'' {
                // Escape control characters at start of line
                self.output.push_str("\\&");
            }
        }
        self.output.push_str(text);
        self.output.push('\n');
        self.beginning_of_line = true;
    }
}

/// Escape man page special characters
fn escape_man(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);

    for c in text.chars() {
        match c {
            '\\' => result.push_str("\\e"),
            '-' => result.push_str("\\-"),
            _ => result.push(c),
        }
    }

    result
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

        let man = render(&root, 0);
        assert!(man.contains(".PP"));
        assert!(man.contains("Hello world"));
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

        let man = render(&root, 0);
        assert!(man.contains("\\fIemphasized\\fR"));
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

        let man = render(&root, 0);
        assert!(man.contains("\\fBstrong\\fR"));
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

        let man = render(&root, 0);
        assert!(man.contains("\\fCcode\\fR"));
    }

    #[test]
    fn test_render_heading() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let heading = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 1,
                content: "Heading".to_string(),
            },
        )));

        append_child(&root, heading.clone());

        let man = render(&root, 0);
        assert!(man.contains(".SH"));
    }

    #[test]
    fn test_render_heading2() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let heading = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 2,
                content: "Subheading".to_string(),
            },
        )));

        append_child(&root, heading.clone());

        let man = render(&root, 0);
        assert!(man.contains(".SS"));
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

        let man = render(&root, 0);
        assert!(man.contains(".RS"));
        assert!(man.contains(".RE"));
    }

    #[test]
    fn test_render_code_block() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let code_block = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::CodeBlock,
            NodeData::CodeBlock {
                info: "".to_string(),
                literal: "fn main() {}".to_string(),
            },
        )));

        append_child(&root, code_block.clone());

        let man = render(&root, 0);
        assert!(man.contains(".EX"));
        assert!(man.contains("fn main() {}"));
        assert!(man.contains(".EE"));
    }

    #[test]
    fn test_render_bullet_list() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let list = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::List,
            NodeData::List {
                list_type: ListType::Bullet,
                delim: crate::node::DelimType::None,
                start: 0,
                tight: true,
                bullet_char: '-',
            },
        )));
        let item = Rc::new(RefCell::new(Node::new(NodeType::Item)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Item".to_string(),
            },
        )));

        append_child(&root, list.clone());
        append_child(&list, item.clone());
        append_child(&item, para.clone());
        append_child(&para, text.clone());

        let man = render(&root, 0);
        assert!(man.contains(".IP"));
    }

    #[test]
    fn test_escape_man() {
        assert_eq!(escape_man("foo-bar"), "foo\\-bar");
        assert_eq!(escape_man("foo\\bar"), "foo\\ebar");
    }

    #[test]
    fn test_escape_dot_at_start() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: ".dot at start".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, text.clone());

        let man = render(&root, 0);
        assert!(man.contains("\\&.dot"));
    }
}
