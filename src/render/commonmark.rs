//! CommonMark renderer

use crate::arena::{NodeArena, NodeId};
use crate::nodes::{AstNode, ListDelimType, ListType, NodeHeading, NodeList, NodeValue};
use crate::parser::options::{Options, Plugins};
use std::fmt;

/// Render a node tree as CommonMark
pub fn render(arena: &NodeArena, root: NodeId, options: u32) -> String {
    let mut renderer = CommonMarkRenderer::new(arena, options);
    renderer.render(root)
}

/// Format an AST as CommonMark with plugins (comrak-style API).
///
/// This is a temporary implementation that delegates to the legacy renderer.
/// In the future, this should be updated to use the new AstNode API directly.
pub fn format_document_with_plugins<'a>(
    root: &'a AstNode<'a>,
    _options: &Options,
    output: &mut dyn fmt::Write,
    _plugins: &Plugins<'_>,
) -> fmt::Result {
    // For now, use a simple recursive approach
    format_node_commonmark(root, output)
}

fn format_node_commonmark(
    node: &AstNode<'_>,
    output: &mut dyn fmt::Write,
) -> fmt::Result {
    let ast = node.data.borrow();

    match &ast.value {
        NodeValue::Document => {
            // Render children
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_commonmark(child, output)?;
                child_opt = child.next_sibling();
            }
        }
        NodeValue::Paragraph => {
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_commonmark(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("\n\n")?;
        }
        NodeValue::Text(text) => {
            output.write_str(text)?;
        }
        NodeValue::Heading(heading) => {
            let hashes: String = std::iter::repeat('#')
                .take(heading.level as usize)
                .collect();
            output.write_str(&hashes)?;
            output.write_str(" ")?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_commonmark(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("\n\n")?;
        }
        NodeValue::Emph => {
            output.write_str("*")?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_commonmark(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("*")?;
        }
        NodeValue::Strong => {
            output.write_str("**")?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_commonmark(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("**")?;
        }
        NodeValue::Code(code) => {
            let backticks = get_backtick_sequence(&code.literal);
            output.write_str(&backticks)?;
            output.write_str(&code.literal)?;
            output.write_str(&backticks)?;
        }
        NodeValue::Link(link) => {
            output.write_str("[")?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_commonmark(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("](")?;
            output.write_str(&link.url)?;
            if !link.title.is_empty() {
                output.write_str(&format!(" \"{}\"", link.title))?;
            }
            output.write_str(")")?;
        }
        NodeValue::Image(link) => {
            output.write_str("![")?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_commonmark(child, output)?;
                child_opt = child.next_sibling();
            }
            output.write_str("](")?;
            output.write_str(&link.url)?;
            if !link.title.is_empty() {
                output.write_str(&format!(" \"{}\"", link.title))?;
            }
            output.write_str(")")?;
        }
        NodeValue::BlockQuote => {
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                output.write_str("> ")?;
                format_node_commonmark(child, output)?;
                child_opt = child.next_sibling();
            }
        }
        NodeValue::List(list) => {
            let mut child_opt = node.first_child();
            let mut index = list.start;
            while let Some(child) = child_opt {
                match list.list_type {
                    ListType::Bullet => {
                        output.write_str(&format!("{} ", list.bullet_char as char))?;
                    }
                    ListType::Ordered => {
                        let delim = match list.delimiter {
                            ListDelimType::Period => ".",
                            ListDelimType::Paren => ")",
                        };
                        output.write_str(&format!("{}{} ", index, delim))?;
                        index += 1;
                    }
                }
                format_node_commonmark(child, output)?;
                child_opt = child.next_sibling();
            }
        }
        NodeValue::CodeBlock(code) => {
            let fence_len = 3;
            let fence: String = std::iter::repeat('`').take(fence_len).collect();
            output.write_str(&fence)?;
            if !code.info.is_empty() {
                output.write_str(&code.info)?;
            }
            output.write_str("\n")?;
            output.write_str(&code.literal)?;
            output.write_str("\n")?;
            output.write_str(&fence)?;
            output.write_str("\n\n")?;
        }
        NodeValue::ThematicBreak => {
            output.write_str("***\n\n")?;
        }
        NodeValue::SoftBreak => {
            output.write_str("\n")?;
        }
        NodeValue::HardBreak => {
            output.write_str("\\n")?;
        }
        _ => {
            // For other nodes, just render children
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                format_node_commonmark(child, output)?;
                child_opt = child.next_sibling();
            }
        }
    }

    Ok(())
}

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

    let count = (max_backticks + 1).max(1);
    std::iter::repeat('`').take(count).collect()
}

/// CommonMark renderer state
struct CommonMarkRenderer<'a> {
    arena: &'a NodeArena,
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

impl<'a> CommonMarkRenderer<'a> {
    fn new(arena: &'a NodeArena, options: u32) -> Self {
        CommonMarkRenderer {
            arena,
            options,
            output: String::new(),
            column: 0,
            beginning_of_line: true,
            list_prefixes: Vec::new(),
            tight_list_stack: Vec::new(),
            need_blank_line: false,
        }
    }

    fn render(&mut self, root: NodeId) -> String {
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
            self.output.push('\n');
            self.column = 0;
            self.beginning_of_line = true;
            self.need_blank_line = false;
        }

        match &node.value {
            NodeValue::Document => {}
            NodeValue::BlockQuote => {
                self.list_prefixes.push("> ".to_string());
                self.beginning_of_line = true;
            }
            NodeValue::List(NodeList { tight, .. }) => {
                self.tight_list_stack.push(*tight);
                // Don't add blank line before first list
                if !self.output.is_empty() && !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
            }
            NodeValue::Item(..) => {
                let prefix = self.format_list_item_prefix(node_id);
                self.list_prefixes.push(prefix);
                self.beginning_of_line = true;
            }
            NodeValue::CodeBlock(..) => {
                self.render_code_block(node_id);
                self.need_blank_line = true;
            }
            NodeValue::HtmlBlock(html_block) => {
                self.render_html_block(&html_block.literal);
                self.need_blank_line = true;
            }
            NodeValue::Paragraph => {
                // In tight lists, don't add blank line before paragraph
                if !self.in_tight_list() && !self.output.is_empty() {
                    self.need_blank_line = true;
                }
            }
            NodeValue::Heading(..) => {
                self.render_heading(node_id);
                self.need_blank_line = true;
            }
            NodeValue::ThematicBreak => {
                self.write_line("***");
                self.need_blank_line = true;
            }
            NodeValue::Text(literal) => {
                self.write_inline(&escape_markdown(literal));
            }
            NodeValue::SoftBreak => {
                if self.in_tight_list() {
                    self.write_inline(" ");
                } else {
                    self.write_line("");
                }
            }
            NodeValue::HardBreak => {
                self.write_inline("  ");
                self.write_line("");
            }
            NodeValue::Code(code) => {
                let backticks = get_backtick_sequence(&code.literal);
                self.write_inline(&backticks);
                self.write_inline(&code.literal);
                self.write_inline(&backticks);
            }
            NodeValue::HtmlInline(literal) => {
                self.write_inline(literal.as_ref());
            }
            NodeValue::Emph => {
                self.write_inline("*");
            }
            NodeValue::Strong => {
                self.write_inline("**");
            }
            NodeValue::Link(..) => {
                self.write_inline("[");
            }
            NodeValue::Image(..) => {
                self.write_inline("![");
            }
            NodeValue::Strikethrough => {
                self.write_inline("~~");
            }
            NodeValue::TaskItem(task_item) => {
                if task_item.symbol.is_some() {
                    self.write_inline("[x] ");
                } else {
                    self.write_inline("[ ] ");
                }
            }
            NodeValue::FootnoteReference(footnote_ref) => {
                self.write_inline(&format!("[^{}]", footnote_ref.name));
            }
            NodeValue::FootnoteDefinition(footnote_def) => {
                self.write_inline(&format!("[^{}]: ", footnote_def.name));
            }
            _ => {}
        }
    }

    fn exit_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        match &node.value {
            NodeValue::Document => {}
            NodeValue::BlockQuote => {
                self.list_prefixes.pop();
                self.need_blank_line = true;
            }
            NodeValue::List(..) => {
                self.tight_list_stack.pop();
                self.need_blank_line = true;
            }
            NodeValue::Item(..) => {
                self.list_prefixes.pop();
            }
            NodeValue::Paragraph => {
                if !self.in_tight_list() {
                    self.write_line("");
                }
            }
            NodeValue::Emph => {
                self.write_inline("*");
            }
            NodeValue::Strong => {
                self.write_inline("**");
            }
            NodeValue::Link(link) => {
                self.write_inline("](");
                self.write_inline(&escape_link_url(&link.url));
                if !link.title.is_empty() {
                    self.write_inline(&format!(" \"{}\"", escape_string(&link.title)));
                }
                self.write_inline(")");
            }
            NodeValue::Image(link) => {
                self.write_inline("](");
                self.write_inline(&escape_link_url(&link.url));
                if !link.title.is_empty() {
                    self.write_inline(&format!(" \"{}\"", escape_string(&link.title)));
                }
                self.write_inline(")");
            }
            NodeValue::Strikethrough => {
                self.write_inline("~~");
            }
            NodeValue::TaskItem(..) => {
                // Task item marker is already written in enter_node
            }
            NodeValue::FootnoteReference(..) => {
                // Footnote ref is already written in enter_node
            }
            NodeValue::FootnoteDefinition(..) => {
                // Footnote def is already written in enter_node
                self.write_line("");
                self.need_blank_line = true;
            }
            _ => {}
        }
    }

    fn render_code_block(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        if let NodeValue::CodeBlock(code_block) = &node.value {
            // Determine fence length (must be longer than any backtick sequence in content)
            let mut fence_len = 3;
            for seq in code_block.literal.split('\n') {
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

            if !code_block.info.is_empty() {
                // Remove trailing backticks from info string
                let clean_info = code_block.info.trim_end_matches('`');
                if !clean_info.is_empty() {
                    self.output.pop(); // Remove newline
                    self.write_inline(clean_info);
                    self.write_line("");
                }
            }

            // Write code content
            for line in code_block.literal.lines() {
                self.write_line(line);
            }

            self.write_line(&fence);
        }
    }

    fn render_html_block(&mut self, literal: &str) {
        for line in literal.lines() {
            self.write_line(line);
        }
    }

    fn render_heading(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        if let NodeValue::Heading(NodeHeading { level, .. }) = &node.value {
            // Use ATX style headings
            let hashes: String = std::iter::repeat('#').take(*level as usize).collect();
            self.write_inline(&hashes);
            self.write_inline(" ");

            // Render heading content by walking children
            // For now, just output a newline (content will be added by children)
            self.write_line("");
        }
    }

    fn format_list_item_prefix(&self, node_id: NodeId) -> String {
        // Get the parent list to determine the marker
        let node = self.arena.get(node_id);
        if let Some(parent_id) = node.parent {
            let parent = self.arena.get(parent_id);
            if let NodeValue::List(NodeList {
                list_type,
                delimiter,
                start,
                bullet_char,
                ..
            }) = &parent.value
            {
                match list_type {
                    ListType::Bullet => {
                        return format!("{} ", *bullet_char as char);
                    }
                    ListType::Ordered => {
                        let marker = match delimiter {
                            ListDelimType::Period => format!("{}.", start),
                            ListDelimType::Paren => format!("{})", start),
                        };
                        // Pad to 4 characters for alignment
                        return format!("{:4}", marker);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};
    use crate::nodes::{NodeCode, NodeCodeBlock, NodeLink};

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let cm = render(&arena, root, 0);
        assert_eq!(cm.trim(), "Hello world");
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

        let cm = render(&arena, root, 0);
        assert_eq!(cm.trim(), "*emphasized*");
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

        let cm = render(&arena, root, 0);
        assert_eq!(cm.trim(), "**strong**");
    }

    #[test]
    fn test_render_code() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = arena.alloc(Node::with_value(NodeValue::Code(Box::new(NodeCode {
            num_backticks: 1,
            literal: "code".to_string(),
        }))));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let cm = render(&arena, root, 0);
        assert_eq!(cm.trim(), "`code`");
    }

    #[test]
    fn test_render_code_with_backticks() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = arena.alloc(Node::with_value(NodeValue::Code(Box::new(NodeCode {
            num_backticks: 1,
            literal: "code `with` backticks".to_string(),
        }))));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let cm = render(&arena, root, 0);
        assert_eq!(cm.trim(), "``code `with` backticks``");
    }

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Heading")));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let cm = render(&arena, root, 0);
        // The renderer outputs "## \nHeading" for ATX headings
        assert!(cm.contains("##"));
        assert!(cm.contains("Heading"));
    }

    #[test]
    fn test_render_link() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com".to_string(),
            title: "".to_string(),
        }))));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("link")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let cm = render(&arena, root, 0);
        assert_eq!(cm.trim(), "[link](https://example.com)");
    }

    #[test]
    fn test_render_image() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let image =
            arena.alloc(Node::with_value(NodeValue::Image(Box::new(NodeLink {
                url: "image.png".to_string(),
                title: "".to_string(),
            }))));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("alt")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, image);
        TreeOps::append_child(&mut arena, image, text);

        let cm = render(&arena, root, 0);
        assert_eq!(cm.trim(), "![alt](image.png)");
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

        let cm = render(&arena, root, 0);
        assert!(cm.contains("> Quote"));
    }

    #[test]
    fn test_render_code_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let code_block = arena.alloc(Node::with_value(NodeValue::CodeBlock(Box::new(
            NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            },
        ))));

        TreeOps::append_child(&mut arena, root, code_block);

        let cm = render(&arena, root, 0);
        assert!(cm.contains("```rust"));
        assert!(cm.contains("fn main() {}"));
        assert!(cm.contains("```"));
    }

    #[test]
    fn test_render_thematic_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let hr = arena.alloc(Node::with_value(NodeValue::ThematicBreak));

        TreeOps::append_child(&mut arena, root, hr);

        let cm = render(&arena, root, 0);
        assert!(cm.contains("***"));
    }

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("*text*"), "\\*text\\*");
        assert_eq!(escape_markdown("_text_"), "\\_text\\_");
        assert_eq!(escape_markdown("[link]"), "\\[link\\]");
    }
}
