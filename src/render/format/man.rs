//! Man page renderer

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::{ListDelimType, ListType, NodeHeading, NodeList, NodeValue};

/// Render a node tree as a Man page (groff format)
pub fn render(arena: &NodeArena, root: NodeId, options: u32) -> String {
    let mut renderer = ManRenderer::new(arena, options);
    renderer.render(root)
}

/// Man page renderer state
struct ManRenderer<'a> {
    arena: &'a NodeArena,
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

impl<'a> ManRenderer<'a> {
    fn new(arena: &'a NodeArena, _options: u32) -> Self {
        ManRenderer {
            arena,
            output: String::new(),
            beginning_of_line: true,
            indent_stack: Vec::new(),
            need_blank_line: false,
            in_verbatim: false,
            font_stack: Vec::new(),
        }
    }

    fn render(&mut self, root: NodeId) -> String {
        // Add man page header
        self.writeln(".TH \"MANUAL\" \"1\" \"\" \"\" \"\"");
        self.writeln("");

        self.render_node(root, true);

        // Remove trailing whitespace and newlines
        while self.output.ends_with('\n') || self.output.ends_with(' ') {
            self.output.pop();
        }

        // Ensure single trailing newline
        self.output.push('\n');

        self.output.clone()
    }

    fn render_node(&mut self, node_id: NodeId, entering: bool) {
        if entering {
            self.enter_node(node_id);
            let node = self.arena.get(node_id);
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                self.render_node(child_id, true);
                child_opt = self.arena.get(child_id).next;
            }
            self.exit_node(node_id);
        }
    }

    fn enter_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        // Add blank line before block elements if needed
        if self.need_blank_line
            && node.value.is_block()
            && !matches!(
                node.value,
                NodeValue::Document | NodeValue::List(..) | NodeValue::Item(..)
            )
        {
            self.writeln("");
            self.beginning_of_line = true;
            self.need_blank_line = false;
        }

        match &node.value {
            NodeValue::Document => {}
            NodeValue::BlockQuote => {
                self.writeln(".RS");
                self.indent_stack.push(1);
            }
            NodeValue::List(..) => {
                self.indent_stack.push(1);
            }
            NodeValue::Item(..) => {
                self.write(".IP \"");
                // Get the parent list to determine the marker
                if let Some(parent_id) = node.parent {
                    let parent = self.arena.get(parent_id);
                    if let NodeValue::List(NodeList {
                        list_type,
                        delimiter,
                        bullet_char,
                        ..
                    }) = &parent.value
                    {
                        match list_type {
                            ListType::Bullet => {
                                self.write(&format!("{}  \"", *bullet_char as char));
                            }
                            ListType::Ordered => {
                                let marker = match delimiter {
                                    ListDelimType::Period => "1.",
                                    ListDelimType::Paren => "1)",
                                };
                                self.write(&format!("{}  \"", marker));
                            }
                        }
                    } else {
                        self.write("-  \"");
                    }
                } else {
                    self.write("-  \"");
                }
                self.writeln("4");
            }
            NodeValue::CodeBlock(..) => {
                self.render_code_block(node_id);
                self.need_blank_line = true;
            }
            NodeValue::HtmlBlock(..) => {
                // HTML blocks are ignored in man page output
            }
            NodeValue::Paragraph => {
                if !self.in_verbatim {
                    self.writeln(".PP");
                }
            }
            NodeValue::Heading(..) => {
                self.render_heading(node_id);
                self.need_blank_line = true;
            }
            NodeValue::ThematicBreak(..) => {
                self.writeln(".PP");
                self.writeln("   *   *   *");
                self.need_blank_line = true;
            }
            NodeValue::Text(literal) => {
                if self.in_verbatim {
                    self.write(literal);
                } else {
                    self.write(&escape_man(literal));
                }
            }
            NodeValue::SoftBreak => {
                if self.in_verbatim {
                    self.writeln("");
                } else {
                    self.write(" ");
                }
            }
            NodeValue::HardBreak => {
                if self.in_verbatim {
                    self.writeln("");
                } else {
                    self.writeln(".br");
                }
            }
            NodeValue::Code(code) => {
                self.write("\\fC");
                self.write(&escape_man(&code.literal));
                self.write("\\fR");
            }
            NodeValue::HtmlInline(..) => {
                // HTML inline is ignored in man page output
            }
            NodeValue::Emph => {
                self.write("\\fI");
                self.font_stack.push(2);
            }
            NodeValue::Strong => {
                self.write("\\fB");
                self.font_stack.push(1);
            }
            NodeValue::Link(..) => {
                // In man pages, links are just shown as text
                // The URL could be shown in footnotes, but for simplicity
                // we just render the link text
            }
            NodeValue::Image(..) => {
                // Images are replaced with their alt text in man pages
                self.write("[IMAGE: ");
            }
            _ => {}
        }
    }

    fn exit_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        match &node.value {
            NodeValue::Document => {}
            NodeValue::BlockQuote => {
                self.writeln(".RE");
                self.indent_stack.pop();
                self.need_blank_line = true;
            }
            NodeValue::List(..) => {
                self.indent_stack.pop();
                self.need_blank_line = true;
            }
            NodeValue::Item(..) => {
                self.writeln("");
            }
            NodeValue::Paragraph => {
                self.writeln("");
            }
            NodeValue::Emph => {
                self.reset_font();
            }
            NodeValue::Strong => {
                self.reset_font();
            }
            NodeValue::Link(..) => {
                // Nothing to do for link exit
            }
            NodeValue::Image(..) => {
                self.write("]");
            }
            _ => {}
        }
    }

    fn render_code_block(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        if let NodeValue::CodeBlock(code_block) = &node.value {
            self.in_verbatim = true;

            if !code_block.info.is_empty() {
                let lang = code_block.info.split_whitespace().next().unwrap_or("");
                self.write(".PP");
                self.write(&escape_man(lang));
                self.writeln(":");
            }

            self.writeln(".EX");

            // Write code content
            for line in code_block.literal.lines() {
                self.writeln(line);
            }

            self.writeln(".EE");
            self.in_verbatim = false;
        }
    }

    fn render_heading(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        if let NodeValue::Heading(NodeHeading { level, .. }) = &node.value {
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
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{NodeCode, NodeCodeBlock};

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let man = render(&arena, root, 0);
        assert!(man.contains(".PP"));
        assert!(man.contains("Hello world"));
    }

    #[test]
    fn test_render_emph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("emphasized")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, emph, text);

        let man = render(&arena, root, 0);
        assert!(man.contains("\\fIemphasized\\fR"));
    }

    #[test]
    fn test_render_strong() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("strong")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, strong, text);

        let man = render(&arena, root, 0);
        assert!(man.contains("\\fBstrong\\fR"));
    }

    #[test]
    fn test_render_code() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = arena.alloc(Node::with_value(NodeValue::code(NodeCode {
            num_backticks: 1,
            literal: "code".to_string(),
        })));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let man = render(&arena, root, 0);
        assert!(man.contains("\\fCcode\\fR"));
    }

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));

        TreeOps::append_child(&mut arena, root, heading);

        let man = render(&arena, root, 0);
        assert!(man.contains(".SH"));
    }

    #[test]
    fn test_render_heading2() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));

        TreeOps::append_child(&mut arena, root, heading);

        let man = render(&arena, root, 0);
        assert!(man.contains(".SS"));
    }

    #[test]
    fn test_render_blockquote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let blockquote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Quote")));

        TreeOps::append_child(&mut arena, root, blockquote);
        TreeOps::append_child(&mut arena, blockquote, para);
        TreeOps::append_child(&mut arena, para, text);

        let man = render(&arena, root, 0);
        assert!(man.contains(".RS"));
        assert!(man.contains(".RE"));
    }

    #[test]
    fn test_render_code_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let code_block =
            arena.alloc(Node::with_value(NodeValue::code_block(NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            })));

        TreeOps::append_child(&mut arena, root, code_block);

        let man = render(&arena, root, 0);
        assert!(man.contains(".EX"));
        assert!(man.contains("fn main() {}"));
        assert!(man.contains(".EE"));
    }

    #[test]
    fn test_render_bullet_list() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            delimiter: ListDelimType::Period,
            start: 0,
            tight: true,
            bullet_char: b'-',
            marker_offset: 0,
            padding: 0,
            is_task_list: false,
        })));
        let item = arena.alloc(Node::with_value(NodeValue::Item(NodeList::default())));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Item")));

        TreeOps::append_child(&mut arena, root, list);
        TreeOps::append_child(&mut arena, list, item);
        TreeOps::append_child(&mut arena, item, para);
        TreeOps::append_child(&mut arena, para, text);

        let man = render(&arena, root, 0);
        assert!(man.contains(".IP"));
    }

    #[test]
    fn test_escape_man() {
        assert_eq!(escape_man("foo-bar"), "foo\\-bar");
        assert_eq!(escape_man("foo\\bar"), "foo\\ebar");
    }

    #[test]
    fn test_escape_dot_at_start() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text(".dot at start")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let man = render(&arena, root, 0);
        assert!(man.contains("\\&.dot"));
    }
}
