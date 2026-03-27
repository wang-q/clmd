//! HTML renderer

use crate::arena::{NodeArena, NodeId};
use crate::node_value::{
    ListType, NodeCode, NodeCodeBlock, NodeFootnoteDefinition, NodeFootnoteReference,
    NodeHeading, NodeHtmlBlock, NodeLink, NodeList, NodeTable, NodeTaskItem, NodeValue,
    TableAlignment,
};

/// Render a node tree as HTML
pub fn render(arena: &NodeArena, root: NodeId, options: u32) -> String {
    let mut renderer = HtmlRenderer::new(arena, options);
    renderer.render(root)
}

/// HTML renderer state
struct HtmlRenderer<'a> {
    arena: &'a NodeArena,
    #[allow(dead_code)]
    options: u32,
    output: String,
    /// Stack for tracking whether we need to close a tag
    tag_stack: Vec<&'static str>,
    /// Track if we need to add a newline before next block
    need_blank_line: bool,
    /// Track if we're at the beginning of a line
    beginning_of_line: bool,
    /// Track if we're in a tight list
    tight_list_stack: Vec<bool>,
    /// Track footnotes for rendering at the end
    footnotes: Vec<(String, NodeId)>,
    /// Track if we're in a code block
    in_code_block: bool,
}

impl<'a> HtmlRenderer<'a> {
    fn new(arena: &'a NodeArena, options: u32) -> Self {
        HtmlRenderer {
            arena,
            options,
            output: String::new(),
            tag_stack: Vec::new(),
            need_blank_line: false,
            beginning_of_line: true,
            tight_list_stack: Vec::new(),
            footnotes: Vec::new(),
            in_code_block: false,
        }
    }

    fn render(&mut self, root: NodeId) -> String {
        self.render_node(root, true);

        // Render footnotes if any
        if !self.footnotes.is_empty() {
            self.render_footnotes();
        }

        // Remove trailing whitespace and newlines
        while self.output.ends_with('\n') || self.output.ends_with(' ') {
            self.output.pop();
        }

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
            && !self.in_code_block
        {
            self.output.push('\n');
            self.beginning_of_line = true;
            self.need_blank_line = false;
        }

        match &node.value {
            NodeValue::Document => {}
            NodeValue::BlockQuote => {
                self.write("<blockquote>");
                self.tag_stack.push("blockquote");
            }
            NodeValue::List(NodeList {
                tight, list_type, ..
            }) => {
                self.tight_list_stack.push(*tight);
                match list_type {
                    ListType::Bullet => {
                        self.write_line("<ul>");
                        self.tag_stack.push("ul");
                    }
                    ListType::Ordered => {
                        self.write_line("<ol>");
                        self.tag_stack.push("ol");
                    }
                }
            }
            NodeValue::Item(..) => {
                self.write("<li");
                let parent = node.parent.map(|id| self.arena.get(id));
                let is_task_list = parent.map_or(false, |p| {
                    if let NodeValue::List(NodeList { is_task_list, .. }) = &p.value {
                        *is_task_list
                    } else {
                        false
                    }
                });

                if is_task_list {
                    self.write(" class=\"task-list-item\"");
                }
                self.write_line(">");
                self.tag_stack.push("li");
            }
            NodeValue::CodeBlock(..) => {
                self.render_code_block(node_id);
                self.need_blank_line = true;
            }
            NodeValue::HtmlBlock(NodeHtmlBlock { literal, .. }) => {
                self.write(literal);
                self.need_blank_line = true;
            }
            NodeValue::Paragraph => {
                if !self.in_tight_list() {
                    self.write("<p>");
                    self.tag_stack.push("p");
                }
            }
            NodeValue::Heading(NodeHeading { level, .. }) => {
                let tag = format!("h{}", level);
                self.write(&format!("<{}>", tag));
                self.tag_stack.push(Box::leak(tag.into_boxed_str()));
            }
            NodeValue::ThematicBreak => {
                self.write_line("<hr />");
                self.need_blank_line = true;
            }
            NodeValue::Text(literal) => {
                if self.in_code_block {
                    self.write(literal);
                } else {
                    self.write(&escape_html(literal));
                }
            }
            NodeValue::SoftBreak => {
                if self.in_code_block {
                    self.write("\n");
                } else if self.in_tight_list() {
                    self.write(" ");
                } else {
                    self.write_line("");
                }
            }
            NodeValue::HardBreak => {
                self.write_line("<br />");
            }
            NodeValue::Code(NodeCode { literal, .. }) => {
                self.write("<code>");
                self.write(&escape_html(literal));
                self.write("</code>");
            }
            NodeValue::HtmlInline(literal) => {
                self.write(literal);
            }
            NodeValue::Emph => {
                self.write("<em>");
            }
            NodeValue::Strong => {
                self.write("<strong>");
            }
            NodeValue::Link(NodeLink { url, title }) => {
                self.write(&format!("<a href=\"{}\"", escape_href(url)));
                if !title.is_empty() {
                    self.write(&format!(" title=\"{}\"", escape_html(title)));
                }
                self.write(">");
            }
            NodeValue::Image(NodeLink { url, title }) => {
                self.write(&format!("<img src=\"{}\"", escape_href(url)));
                // The alt text comes from the image's children
                if !title.is_empty() {
                    self.write(&format!(" title=\"{}\"", escape_html(title)));
                }
                self.write(" />");
            }
            NodeValue::Strikethrough => {
                self.write("<del>");
            }
            NodeValue::TaskItem(NodeTaskItem { symbol, .. }) => {
                let checked = symbol.is_some();
                self.write(&format!(
                    "<input type=\"checkbox\" disabled=\"disabled\"{} />",
                    if checked { " checked=\"checked\"" } else { "" }
                ));
            }
            NodeValue::FootnoteReference(NodeFootnoteReference { name, .. }) => {
                // Collect footnote for rendering at the end
                if let Some(def_id) = self.find_footnote_def(name) {
                    self.footnotes.push((name.clone(), def_id));
                }
                self.write(&format!(
                    "<sup class=\"footnote-ref\"><a href=\"#fn-{}\" id=\"fnref-{}\">[{}]</a></sup>",
                    escape_html(name),
                    escape_html(name),
                    escape_html(name)
                ));
            }
            NodeValue::FootnoteDefinition(NodeFootnoteDefinition { name, .. }) => {
                // Footnote definitions are rendered at the end
                self.write(&format!("<li id=\"fn-{}\">", escape_html(name)));
                self.tag_stack.push("li");
            }
            NodeValue::Table(NodeTable { alignments, .. }) => {
                self.write_line("<table>");
                if !alignments.is_empty() {
                    self.write_line("<thead>");
                    self.write_line("<tr>");
                    for alignment in alignments {
                        let align_attr = match alignment {
                            TableAlignment::Left => " align=\"left\"",
                            TableAlignment::Center => " align=\"center\"",
                            TableAlignment::Right => " align=\"right\"",
                            TableAlignment::None => "",
                        };
                        self.write_line(&format!("<th{}>", align_attr));
                        self.tag_stack.push("th");
                    }
                }
            }
            NodeValue::TableRow(is_header) => {
                if *is_header {
                    self.write_line("</tr>");
                    self.write_line("</thead>");
                    self.write_line("<tbody>");
                } else {
                    self.write_line("<tr>");
                }
            }
            NodeValue::TableCell => {
                self.write("<td>");
                self.tag_stack.push("td");
            }
            _ => {}
        }
    }

    fn exit_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        match &node.value {
            NodeValue::Document => {}
            NodeValue::BlockQuote => {
                self.write_line("</blockquote>");
                self.tag_stack.pop();
                self.need_blank_line = true;
            }
            NodeValue::List(..) => {
                self.tight_list_stack.pop();
                if let Some(tag) = self.tag_stack.pop() {
                    self.write_line(&format!("</{}>", tag));
                }
                self.need_blank_line = true;
            }
            NodeValue::Item(..) => {
                if let Some(tag) = self.tag_stack.pop() {
                    self.write_line(&format!("</{}>", tag));
                }
            }
            NodeValue::Paragraph => {
                if !self.in_tight_list() {
                    if let Some(tag) = self.tag_stack.pop() {
                        self.write_line(&format!("</{}>", tag));
                    }
                }
            }
            NodeValue::Heading(NodeHeading { level, .. }) => {
                let tag = format!("h{}", level);
                self.write_line(&format!("</{}>", tag));
                self.tag_stack.pop();
                self.need_blank_line = true;
            }
            NodeValue::Emph => {
                self.write("</em>");
            }
            NodeValue::Strong => {
                self.write("</strong>");
            }
            NodeValue::Link(..) => {
                self.write("</a>");
            }
            NodeValue::Strikethrough => {
                self.write("</del>");
            }
            NodeValue::FootnoteDefinition(..) => {
                if let Some(tag) = self.tag_stack.pop() {
                    self.write_line(&format!("</{}>", tag));
                }
                self.write_line("</li>");
            }
            NodeValue::Table(..) => {
                self.write_line("</tbody>");
                self.write_line("</table>");
                self.need_blank_line = true;
            }
            NodeValue::TableRow(..) => {
                self.write_line("</tr>");
            }
            NodeValue::TableCell => {
                if let Some(tag) = self.tag_stack.pop() {
                    self.write_line(&format!("</{}>", tag));
                }
            }
            _ => {}
        }
    }

    fn render_code_block(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        if let NodeValue::CodeBlock(NodeCodeBlock { info, literal, .. }) = &node.value {
            self.in_code_block = true;

            self.write("<pre><code");
            if !info.is_empty() {
                let lang = info.split_whitespace().next().unwrap_or("");
                if !lang.is_empty() {
                    self.write(&format!(" class=\"language-{}\"", escape_html(lang)));
                }
            }
            self.write_line(">");

            // Write code content
            self.write(&escape_html(literal));

            self.write_line("</code></pre>");
            self.in_code_block = false;
        }
    }

    fn render_footnotes(&mut self) {
        self.write_line("<section class=\"footnotes\">");
        self.write_line("<ol>");

        // Collect footnotes to avoid borrow issues
        let footnotes: Vec<(String, NodeId)> = self.footnotes.clone();

        for (name, def_id) in footnotes {
            self.write(&format!("<li id=\"fn-{}\">", escape_html(&name)));
            // Render footnote content
            self.render_node(def_id, true);
            self.write(&format!(
                " <a href=\"#fnref-{}\" class=\"footnote-backref\">↩</a>",
                escape_html(&name)
            ));
            self.write_line("</li>");
        }

        self.write_line("</ol>");
        self.write_line("</section>");
    }

    fn find_footnote_def(&self, _name: &str) -> Option<NodeId> {
        // This is a simplified implementation
        // In a real implementation, we'd search the arena for the footnote definition
        None
    }

    fn write(&mut self, text: &str) {
        self.output.push_str(text);
        self.beginning_of_line = false;
    }

    fn write_line(&mut self, text: &str) {
        self.output.push_str(text);
        self.output.push('\n');
        self.beginning_of_line = true;
    }

    fn in_tight_list(&self) -> bool {
        self.tight_list_stack.last().copied().unwrap_or(false)
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

/// Escape URL for use in href attribute
fn escape_href(url: &str) -> String {
    let mut result = String::with_capacity(url.len());
    for c in url.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\"test\""), "&quot;test&quot;");
    }

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text =
            arena.alloc(Node::with_value(NodeValue::Text("Hello world".to_string())));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render(&arena, root, 0);
        println!("HTML output: {:?}", html);
        assert!(
            html.contains("<p>Hello world</p>"),
            "Expected <p>Hello world</p> in {}",
            html
        );
    }

    #[test]
    fn test_render_emph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text =
            arena.alloc(Node::with_value(NodeValue::Text("emphasized".to_string())));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, emph, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<em>emphasized</em>"));
    }

    #[test]
    fn test_render_strong() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::Text("strong".to_string())));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, strong, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<strong>strong</strong>"));
    }

    #[test]
    fn test_render_code() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = arena.alloc(Node::with_value(NodeValue::Code(NodeCode {
            num_backticks: 1,
            literal: "code".to_string(),
        })));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let html = render(&arena, root, 0);
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Title".to_string())));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<h2>Title</h2>"));
    }

    #[test]
    fn test_render_link() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(NodeValue::Link(NodeLink {
            url: "https://example.com".to_string(),
            title: "".to_string(),
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("link".to_string())));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<a href=\"https://example.com\">link</a>"));
    }

    #[test]
    fn test_render_blockquote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let blockquote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Quote".to_string())));

        TreeOps::append_child(&mut arena, root, blockquote);
        TreeOps::append_child(&mut arena, blockquote, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("<p>Quote</p>"));
        assert!(html.contains("</blockquote>"));
    }

    #[test]
    fn test_render_code_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let code_block =
            arena.alloc(Node::with_value(NodeValue::CodeBlock(NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            })));

        TreeOps::append_child(&mut arena, root, code_block);

        let html = render(&arena, root, 0);
        assert!(html.contains("<pre><code class=\"language-rust\">"));
        assert!(html.contains("fn main() {}"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn test_render_bullet_list() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            delimiter: crate::node_value::ListDelimType::Period,
            start: 1,
            tight: true,
            bullet_char: b'-',
            marker_offset: 0,
            padding: 2,
            is_task_list: false,
        })));
        let item = arena.alloc(Node::with_value(NodeValue::Item(NodeList::default())));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Item".to_string())));

        TreeOps::append_child(&mut arena, root, list);
        TreeOps::append_child(&mut arena, list, item);
        TreeOps::append_child(&mut arena, item, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>"));
        assert!(html.contains("Item"));
        assert!(html.contains("</li>"));
        assert!(html.contains("</ul>"));
    }

    #[test]
    fn test_render_thematic_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let hr = arena.alloc(Node::with_value(NodeValue::ThematicBreak));

        TreeOps::append_child(&mut arena, root, hr);

        let html = render(&arena, root, 0);
        assert!(html.contains("<hr />"));
    }
}
