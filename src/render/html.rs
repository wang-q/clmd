//! HTML renderer

use crate::arena::{NodeArena, NodeId};
use crate::html_utils::{escape_html, is_safe_url};
use crate::nodes::{
    ListType, NodeCode, NodeCodeBlock, NodeFootnoteDefinition, NodeFootnoteReference,
    NodeHeading, NodeHtmlBlock, NodeLink, NodeList, NodeTable, NodeTaskItem, NodeValue,
    TableAlignment,
};

/// Render an arena_tree node to HTML
///
/// This is a convenience function for rendering the new arena_tree-based AST.
pub fn render_from_node<'a>(root: crate::nodes::Node<'a>, _options: u32) -> String {
    let mut renderer = ArenaTreeHtmlRenderer::new();
    renderer.render(root)
}

/// HTML renderer for arena_tree nodes
struct ArenaTreeHtmlRenderer {
    output: String,
    tag_stack: Vec<&'static str>,
    tight_list_stack: Vec<bool>,
    in_code_block: bool,
    last_out: char,
}

impl ArenaTreeHtmlRenderer {
    fn new() -> Self {
        ArenaTreeHtmlRenderer {
            output: String::new(),
            tag_stack: Vec::new(),
            tight_list_stack: Vec::new(),
            in_code_block: false,
            last_out: '\n',
        }
    }

    fn cr(&mut self) {
        if self.last_out != '\n' {
            self.output.push('\n');
            self.last_out = '\n';
        }
    }

    fn lit(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.output.push_str(s);
        self.last_out = s.chars().last().unwrap_or('\n');
    }

    fn in_tight_list(&self) -> bool {
        self.tight_list_stack.last().copied().unwrap_or(false)
    }

    fn render(&mut self, root: crate::nodes::Node<'_>) -> String {
        self.render_node(root);
        while self.output.ends_with('\n') {
            self.output.pop();
        }
        self.output.clone()
    }

    fn render_node(&mut self, node: crate::nodes::Node<'_>) {
        self.enter_node(node);

        // Render children
        let is_image = matches!(node.data.borrow().value, NodeValue::Image(..));
        if !is_image {
            if let Some(child) = node.first_child() {
                self.render_node_recursive(child);
            }
        }

        self.exit_node(node);
    }

    fn render_node_recursive(&mut self, node: crate::nodes::Node<'_>) {
        self.render_node(node);
        if let Some(sibling) = node.next_sibling() {
            self.render_node_recursive(sibling);
        }
    }

    fn enter_node(&mut self, node: crate::nodes::Node<'_>) {
        let ast = node.data.borrow();
        match &ast.value {
            NodeValue::Document => {}
            NodeValue::BlockQuote => {
                self.cr();
                self.lit("<blockquote>");
                self.lit("\n");
                self.tag_stack.push("blockquote");
                self.tight_list_stack.push(false);
            }
            NodeValue::List(NodeList {
                tight,
                list_type,
                start,
                ..
            }) => {
                self.tight_list_stack.push(*tight);
                self.cr();
                match list_type {
                    ListType::Bullet => {
                        self.lit("<ul>");
                        self.tag_stack.push("ul");
                    }
                    ListType::Ordered => {
                        if *start != 1 {
                            self.lit(&format!("<ol start=\"{}\">", start));
                        } else {
                            self.lit("<ol>");
                        }
                        self.tag_stack.push("ol");
                    }
                }
                self.lit("\n");
            }
            NodeValue::Item(..) => {
                self.lit("<li>");
                self.tag_stack.push("li");
                if !self.in_tight_list() {
                    self.lit("\n");
                }
            }
            NodeValue::CodeBlock(code_block) => {
                self.cr();
                self.in_code_block = true;
                self.lit("<pre><code");
                if !code_block.info.is_empty() {
                    let lang = code_block.info.split_whitespace().next().unwrap_or("");
                    if !lang.is_empty() {
                        self.lit(&format!(" class=\"language-{}\"", escape_html(lang)));
                    }
                }
                self.lit(">");
                self.lit(&escape_html(&code_block.literal));
                self.lit("</code></pre>");
                self.lit("\n");
                self.in_code_block = false;
            }
            NodeValue::HtmlBlock(html_block) => {
                self.cr();
                self.lit(&html_block.literal);
                self.lit("\n");
            }
            NodeValue::Paragraph => {
                if !self.in_tight_list() {
                    self.lit("<p>");
                    self.tag_stack.push("p");
                }
            }
            NodeValue::Heading(NodeHeading { level, .. }) => {
                self.lit(&format!("<h{}>", level));
                self.tag_stack.push("h");
            }
            NodeValue::ThematicBreak => {
                self.cr();
                self.lit("<hr />");
                self.lit("\n");
            }
            NodeValue::Text(literal) => {
                if self.in_code_block {
                    self.lit(literal);
                } else {
                    self.lit(&escape_html(literal));
                }
            }
            NodeValue::SoftBreak => {
                if self.in_code_block {
                    self.lit("\n");
                } else if self.in_tight_list() {
                    self.lit(" ");
                } else {
                    self.lit("\n");
                }
            }
            NodeValue::HardBreak => {
                self.lit("<br />");
                self.lit("\n");
            }
            NodeValue::Code(code) => {
                self.lit("<code>");
                self.lit(&escape_html(&code.literal));
                self.lit("</code>");
            }
            NodeValue::HtmlInline(literal) => {
                self.lit(literal.as_ref());
            }
            NodeValue::Emph => {
                self.lit("<em>");
            }
            NodeValue::Strong => {
                self.lit("<strong>");
            }
            NodeValue::Link(link) => {
                self.lit("<a href=\"");
                self.lit(&escape_href(&link.url));
                self.lit("\"");
                if !link.title.is_empty() {
                    self.lit(" title=\"");
                    self.lit(&escape_html(&link.title));
                    self.lit("\"");
                }
                self.lit(">");
            }
            NodeValue::Image(link) => {
                self.lit("<img src=\"");
                self.lit(&escape_href(&link.url));
                self.lit("\" alt=\"");
                // Collect alt text from children
                let alt_text = self.collect_alt_text(node);
                self.lit(&escape_html(&alt_text));
                if !link.title.is_empty() {
                    self.lit("\" title=\"");
                    self.lit(&escape_html(&link.title));
                }
                self.lit("\" />");
            }
            NodeValue::Strikethrough => {
                self.lit("<del>");
            }
            _ => {}
        }
    }

    fn exit_node(&mut self, node: crate::nodes::Node<'_>) {
        let ast = node.data.borrow();
        match &ast.value {
            NodeValue::Document => {}
            NodeValue::BlockQuote => {
                self.lit("</blockquote>");
                self.lit("\n");
                self.tag_stack.pop();
                self.tight_list_stack.pop();
            }
            NodeValue::List(..) => {
                if let Some(tag) = self.tag_stack.pop() {
                    self.lit(&format!("</{}>", tag));
                    self.lit("\n");
                }
                self.tight_list_stack.pop();
            }
            NodeValue::Item(..) => {
                if let Some(tag) = self.tag_stack.pop() {
                    self.lit(&format!("</{}>", tag));
                    self.lit("\n");
                }
            }
            NodeValue::Paragraph => {
                if !self.in_tight_list() {
                    if let Some(tag) = self.tag_stack.pop() {
                        self.lit(&format!("</{}>", tag));
                        self.lit("\n");
                    }
                }
            }
            NodeValue::Heading(NodeHeading { level, .. }) => {
                self.lit(&format!("</h{}>", level));
                self.lit("\n");
                self.tag_stack.pop();
            }
            NodeValue::Emph => {
                self.lit("</em>");
            }
            NodeValue::Strong => {
                self.lit("</strong>");
            }
            NodeValue::Link(..) => {
                self.lit("</a>");
            }
            NodeValue::Strikethrough => {
                self.lit("</del>");
            }
            _ => {}
        }
    }

    fn collect_alt_text(&self, node: crate::nodes::Node<'_>) -> String {
        let mut alt_text = String::new();
        if let Some(child) = node.first_child() {
            self.collect_alt_text_recursive(child, &mut alt_text);
        }
        alt_text
    }

    fn collect_alt_text_recursive(
        &self,
        node: crate::nodes::Node<'_>,
        alt_text: &mut String,
    ) {
        let ast = node.data.borrow();
        match &ast.value {
            NodeValue::Text(literal) => {
                alt_text.push_str(literal);
            }
            NodeValue::SoftBreak | NodeValue::HardBreak => {
                alt_text.push(' ');
            }
            _ => {
                if let Some(child) = node.first_child() {
                    self.collect_alt_text_recursive(child, alt_text);
                }
            }
        }
        if let Some(sibling) = node.next_sibling() {
            self.collect_alt_text_recursive(sibling, alt_text);
        }
    }
}

/// Render a node tree as HTML
pub fn render(arena: &NodeArena, root: NodeId, options: u32) -> String {
    let mut renderer = HtmlRenderer::new(arena, options);
    renderer.render(root)
}

/// HTML renderer state
struct HtmlRenderer<'a> {
    arena: &'a NodeArena,
    output: String,
    /// Stack for tracking whether we need to close a tag
    tag_stack: Vec<&'static str>,
    /// Track if we're in a tight list
    tight_list_stack: Vec<bool>,
    /// Track footnotes for rendering at the end
    footnotes: Vec<(String, NodeId)>,
    /// Track if we're in a code block
    in_code_block: bool,
    /// Track the last output character for cr() logic
    last_out: char,
    /// Counter to disable tag rendering (for image alt text)
    disable_tags: i32,
    /// Track if we're at the first child of a list item (for tight lists)
    item_child_count: Vec<usize>,
}

impl<'a> HtmlRenderer<'a> {
    fn new(arena: &'a NodeArena, _options: u32) -> Self {
        HtmlRenderer {
            arena,
            output: String::new(),
            tag_stack: Vec::new(),
            tight_list_stack: Vec::new(),
            footnotes: Vec::new(),
            in_code_block: false,
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
    fn lit(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }

        self.output.push_str(s);
        self.last_out = s.chars().last().unwrap_or('\n');
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

    fn render(&mut self, root: NodeId) -> String {
        self.render_node(root, true);

        // Render footnotes if any
        if !self.footnotes.is_empty() {
            self.render_footnotes();
        }

        // Remove trailing newline to match CommonMark spec test format
        while self.output.ends_with('\n') {
            self.output.pop();
        }

        self.output.clone()
    }

    fn render_node(&mut self, node_id: NodeId, entering: bool) {
        if entering {
            self.enter_node(node_id);
            let node = self.arena.get(node_id);

            // For image nodes, don't render children as they are used for alt text
            let is_image = matches!(node.value, NodeValue::Image(..));

            if !is_image {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id, true);
                    child_opt = self.arena.get(child_id).next;
                }
            }

            self.exit_node(node_id);
        }
    }

    fn enter_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        match &node.value {
            NodeValue::Document => {}
            NodeValue::BlockQuote => {
                // In tight list items, add newline before blockquote if not first child
                if self.track_item_child() {
                    self.lit("\n");
                } else {
                    self.cr();
                }
                self.lit("<blockquote>");
                self.lit("\n");
                self.tag_stack.push("blockquote");
                // Push false to tight_list_stack to disable tight mode for blockquote contents
                self.tight_list_stack.push(false);
            }
            NodeValue::List(NodeList {
                tight,
                list_type,
                start,
                ..
            }) => {
                // Push tight status to stack
                self.tight_list_stack.push(*tight);
                self.cr(); // Add newline before list if needed (for nested lists)
                match list_type {
                    ListType::Bullet => {
                        self.lit("<ul>");
                        self.tag_stack.push("ul");
                    }
                    ListType::Ordered => {
                        if *start != 1 {
                            self.lit(&format!("<ol start=\"{}\">", start));
                        } else {
                            self.lit("<ol>");
                        }
                        self.tag_stack.push("ol");
                    }
                }
                self.lit("\n");
            }
            NodeValue::Item(..) => {
                self.lit("<li>");
                self.tag_stack.push("li");
                // In loose lists, add newline after <li>, but not for empty items
                let has_children = node.first_child.is_some();
                if !self.in_tight_list() && has_children {
                    self.lit("\n");
                }
                // Initialize child counter for this item
                self.item_child_count.push(0);
            }
            NodeValue::CodeBlock(..) => {
                // In tight list items, add newline before code block if not first child
                if self.track_item_child() {
                    self.lit("\n");
                } else {
                    self.cr();
                }
                self.render_code_block(node_id);
            }
            NodeValue::HtmlBlock(html_block) => {
                // In tight list items, add newline before HTML block if not first child
                if self.track_item_child() {
                    self.lit("\n");
                } else {
                    self.cr();
                }
                // HTML blocks are always output as raw HTML
                self.lit(&html_block.literal);
                self.lit("\n");
            }
            NodeValue::Paragraph => {
                // In tight lists, paragraphs are not wrapped in <p> tags
                if !self.in_tight_list() {
                    // Track as item child in loose lists too
                    self.track_item_child();
                    self.lit("<p>");
                    self.tag_stack.push("p");
                }
            }
            NodeValue::Heading(NodeHeading { level, .. }) => {
                // In tight list items, add newline before heading
                // The first heading in a list item should also have a newline
                if !self.item_child_count.is_empty() {
                    self.track_item_child();
                    self.lit("\n");
                }
                self.lit(&format!("<h{}>", level));
                self.tag_stack.push("h");
            }
            NodeValue::ThematicBreak => {
                // In tight list items, add newline before thematic break if not first child
                if self.track_item_child() {
                    self.lit("\n");
                } else {
                    self.cr();
                }
                self.lit("<hr />");
                self.lit("\n");
            }
            NodeValue::Text(literal) => {
                // Track text nodes as item children in tight lists
                // This ensures proper newline handling for subsequent block elements
                if self.in_tight_list() && !self.item_child_count.is_empty() {
                    self.track_item_child();
                }
                if self.in_code_block {
                    self.lit(literal);
                } else {
                    self.lit(&escape_html(literal));
                }
            }
            NodeValue::SoftBreak => {
                if self.in_code_block {
                    self.lit("\n");
                } else if self.in_tight_list() {
                    self.lit(" ");
                } else {
                    self.lit("\n");
                }
            }
            NodeValue::HardBreak => {
                self.lit("<br />");
                self.lit("\n");
            }
            NodeValue::Code(code) => {
                self.lit("<code>");
                self.lit(&escape_html(&code.literal));
                self.lit("</code>");
            }
            NodeValue::HtmlInline(literal) => {
                self.lit(literal.as_ref());
            }
            NodeValue::Emph => {
                self.lit("<em>");
            }
            NodeValue::Strong => {
                self.lit("<strong>");
            }
            NodeValue::Link(link) => {
                if self.disable_tags > 0 {
                    // We're inside an image's alt text
                    // Links in alt text are replaced by their link text
                } else {
                    self.lit("<a href=\"");
                    self.lit(&escape_href(&link.url));
                    self.lit("\"");
                    if !link.title.is_empty() {
                        self.lit(" title=\"");
                        self.lit(&escape_html(&link.title));
                        self.lit("\"");
                    }
                    self.lit(">");
                }
            }
            NodeValue::Image(link) => {
                self.lit("<img src=\"");
                self.lit(&escape_href(&link.url));
                self.lit("\" alt=\"");
                // Collect alt text from children
                let alt_text = self.collect_alt_text(node_id);
                self.lit(&escape_html(&alt_text));
                if !link.title.is_empty() {
                    self.lit("\" title=\"");
                    self.lit(&escape_html(&link.title));
                }
                self.lit("\" />");
            }
            NodeValue::Strikethrough => {
                self.lit("<del>");
            }
            NodeValue::TaskItem(task_item) => {
                let checked = task_item.symbol.is_some();
                self.lit(&format!(
                    "<input type=\"checkbox\" disabled=\"disabled\"{} />",
                    if checked { " checked=\"checked\"" } else { "" }
                ));
            }
            NodeValue::FootnoteReference(footnote_ref) => {
                // Collect footnote for rendering at the end
                if let Some(def_id) = self.find_footnote_def(&footnote_ref.name) {
                    self.footnotes.push((footnote_ref.name.clone(), def_id));
                }
                self.lit(&format!(
                    "<sup class=\"footnote-ref\"><a href=\"#fn-{}\" id=\"fnref-{}\">[{}]</a></sup>",
                    escape_html(&footnote_ref.name),
                    escape_html(&footnote_ref.name),
                    escape_html(&footnote_ref.name)
                ));
            }
            NodeValue::FootnoteDefinition(footnote_def) => {
                // Footnote definitions are rendered at the end
                self.lit(&format!(
                    "<li id=\"fn-{}\">",
                    escape_html(&footnote_def.name)
                ));
                self.tag_stack.push("li");
            }
            NodeValue::Table(table) => {
                self.lit("<table>");
                self.lit("\n");
                if !table.alignments.is_empty() {
                    self.lit("<thead>");
                    self.lit("\n");
                    self.lit("<tr>");
                    self.lit("\n");
                    for alignment in &table.alignments {
                        let align_attr = match alignment {
                            TableAlignment::Left => " align=\"left\"",
                            TableAlignment::Center => " align=\"center\"",
                            TableAlignment::Right => " align=\"right\"",
                            TableAlignment::None => "",
                        };
                        self.lit(&format!("<th{}>", align_attr));
                        self.lit("\n");
                        self.tag_stack.push("th");
                    }
                }
            }
            NodeValue::TableRow(is_header) => {
                if *is_header {
                    self.lit("</tr>");
                    self.lit("\n");
                    self.lit("</thead>");
                    self.lit("\n");
                    self.lit("<tbody>");
                    self.lit("\n");
                } else {
                    self.lit("<tr>");
                    self.lit("\n");
                }
            }
            NodeValue::TableCell => {
                self.lit("<td>");
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
                self.lit("</blockquote>");
                self.lit("\n");
                self.tag_stack.pop();
                // Pop the false we pushed when entering blockquote
                self.tight_list_stack.pop();
            }
            NodeValue::List(..) => {
                if let Some(tag) = self.tag_stack.pop() {
                    self.lit(&format!("</{}>", tag));
                    self.lit("\n");
                }
                // Pop tight status from stack
                self.tight_list_stack.pop();
            }
            NodeValue::Item(..) => {
                if let Some(tag) = self.tag_stack.pop() {
                    self.lit(&format!("</{}>", tag));
                    self.lit("\n");
                }
                // Pop child counter for this item
                self.item_child_count.pop();
            }
            NodeValue::CodeBlock(..) => {}
            NodeValue::HtmlBlock(..) => {}
            NodeValue::Paragraph => {
                // In tight lists, paragraphs are not wrapped in <p> tags
                if !self.in_tight_list() {
                    if let Some(tag) = self.tag_stack.pop() {
                        self.lit(&format!("</{}>", tag));
                        self.lit("\n");
                    }
                }
            }
            NodeValue::Heading(NodeHeading { level, .. }) => {
                self.lit(&format!("</h{}>", level));
                self.lit("\n");
                self.tag_stack.pop();
            }
            NodeValue::ThematicBreak => {}
            NodeValue::Text(..) => {}
            NodeValue::SoftBreak => {}
            NodeValue::HardBreak => {}
            NodeValue::Code(..) => {}
            NodeValue::HtmlInline(..) => {}
            NodeValue::Emph => {
                self.lit("</em>");
            }
            NodeValue::Strong => {
                self.lit("</strong>");
            }
            NodeValue::Link(..) => {
                if self.disable_tags == 0 {
                    self.lit("</a>");
                }
            }
            NodeValue::Image(..) => {
                // Image tag is already output in enter_node
                // No need to do anything here
            }
            NodeValue::Strikethrough => {
                self.lit("</del>");
            }
            NodeValue::FootnoteDefinition(..) => {
                if let Some(tag) = self.tag_stack.pop() {
                    self.lit(&format!("</{}>", tag));
                    self.lit("\n");
                }
                self.lit("</li>");
                self.lit("\n");
            }
            NodeValue::Table(..) => {
                self.lit("</tbody>");
                self.lit("\n");
                self.lit("</table>");
                self.lit("\n");
            }
            NodeValue::TableRow(..) => {
                self.lit("</tr>");
                self.lit("\n");
            }
            NodeValue::TableCell => {
                if let Some(tag) = self.tag_stack.pop() {
                    self.lit(&format!("</{}>", tag));
                    self.lit("\n");
                }
            }
            _ => {}
        }
    }

    fn render_code_block(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        if let NodeValue::CodeBlock(code_block) = &node.value {
            self.in_code_block = true;

            self.lit("<pre><code");
            if !code_block.info.is_empty() {
                let lang = code_block.info.split_whitespace().next().unwrap_or("");
                if !lang.is_empty() {
                    self.lit(&format!(" class=\"language-{}\"", escape_html(lang)));
                }
            }
            self.lit(">");

            // Write code content
            self.lit(&escape_html(&code_block.literal));

            self.lit("</code></pre>");
            self.lit("\n");
            self.in_code_block = false;
        }
    }

    fn render_footnotes(&mut self) {
        self.lit("<section class=\"footnotes\">");
        self.lit("\n");
        self.lit("<ol>");
        self.lit("\n");

        // Collect footnotes to avoid borrow issues
        let footnotes: Vec<(String, NodeId)> = self.footnotes.clone();

        for (name, def_id) in footnotes {
            self.lit(&format!("<li id=\"fn-{}\">", escape_html(&name)));
            // Render footnote content
            self.render_node(def_id, true);
            self.lit(&format!(
                " <a href=\"#fnref-{}\" class=\"footnote-backref\">↩</a>",
                escape_html(&name)
            ));
            self.lit("</li>");
            self.lit("\n");
        }

        self.lit("</ol>");
        self.lit("\n");
        self.lit("</section>");
        self.lit("\n");
    }

    fn find_footnote_def(&self, name: &str) -> Option<NodeId> {
        // Search the arena for the footnote definition with matching name
        for (id, node) in self.arena.iter() {
            if let NodeValue::FootnoteDefinition(def) = &node.value {
                if def.name == name {
                    return Some(id);
                }
            }
        }
        None
    }

    /// Collect alt text from image node's children
    /// Alt text is the plain text content of the image's children, without HTML tags
    fn collect_alt_text(&self, node_id: NodeId) -> String {
        let mut alt_text = String::new();
        self.collect_alt_text_recursive(node_id, &mut alt_text);
        alt_text
    }

    fn collect_alt_text_recursive(&self, node_id: NodeId, alt_text: &mut String) {
        let node = self.arena.get(node_id);
        match &node.value {
            NodeValue::Text(literal) => {
                alt_text.push_str(literal);
            }
            NodeValue::SoftBreak => {
                alt_text.push(' ');
            }
            NodeValue::HardBreak => {
                alt_text.push(' ');
            }
            _ => {
                // For other node types, recursively collect from children
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.collect_alt_text_recursive(child_id, alt_text);
                    child_opt = self.arena.get(child_id).next;
                }
            }
        }
    }
}

/// Escape URL for use in href attribute
///
/// This function performs two important security checks:
/// 1. Validates the URL scheme to prevent javascript: and other unsafe protocols
/// 2. Escapes special HTML characters to prevent XSS attacks
///
/// # Arguments
///
/// * `url` - The URL to escape
///
/// # Returns
///
/// The escaped URL string, or "#" if the URL is considered unsafe
fn escape_href(url: &str) -> String {
    // First check if the URL is safe (prevents javascript: and other unsafe protocols)
    if !is_safe_url(url) {
        return "#".to_string();
    }

    // Escape special HTML characters for attribute context
    // This is more comprehensive than basic HTML escaping
    let mut result = String::with_capacity(url.len() * 2);
    for c in url.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '\'' => result.push_str("&#x27;"),
            '`' => result.push_str("&#x60;"), // Backtick can be used in IE attribute injection
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
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello world")));

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
        let text = arena.alloc(Node::with_value(NodeValue::make_text("emphasized")));

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
        let text = arena.alloc(Node::with_value(NodeValue::make_text("strong")));

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
        let code = arena.alloc(Node::with_value(NodeValue::Code(Box::new(NodeCode {
            num_backticks: 1,
            literal: "code".to_string(),
        }))));

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
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Title")));

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
        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com".to_string(),
            title: "".to_string(),
        }))));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("link")));

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
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Quote")));

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
            delimiter: crate::nodes::ListDelimType::Period,
            start: 1,
            tight: true,
            bullet_char: b'-',
            marker_offset: 0,
            padding: 2,
            is_task_list: false,
        })));
        let item = arena.alloc(Node::with_value(NodeValue::Item(NodeList::default())));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Item")));

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

    // Security tests for XSS prevention
    #[test]
    fn test_escape_href_blocks_javascript() {
        // javascript: protocol should be blocked
        let result = escape_href("javascript:alert('xss')");
        assert_eq!(result, "#");

        // Case variations
        let result = escape_href("JAVASCRIPT:alert('xss')");
        assert_eq!(result, "#");

        let result = escape_href("JavaScript:alert('xss')");
        assert_eq!(result, "#");
    }

    #[test]
    fn test_escape_href_blocks_vbscript() {
        let result = escape_href("vbscript:msgbox('xss')");
        assert_eq!(result, "#");
    }

    #[test]
    fn test_escape_href_blocks_file_protocol() {
        let result = escape_href("file:///etc/passwd");
        assert_eq!(result, "#");
    }

    #[test]
    fn test_escape_href_allows_safe_urls() {
        // HTTP/HTTPS should be allowed
        let result = escape_href("https://example.com");
        assert_eq!(result, "https://example.com");

        let result = escape_href("http://example.com/path?query=value");
        assert_eq!(result, "http://example.com/path?query=value");
    }

    #[test]
    fn test_escape_href_escapes_special_chars() {
        // Special characters should be escaped
        let result = escape_href("https://example.com?a=1&b=2");
        assert_eq!(result, "https://example.com?a=1&amp;b=2");

        let result = escape_href("https://example.com/<script>");
        assert_eq!(result, "https://example.com/&lt;script&gt;");

        let result = escape_href("https://example.com/\"quoted\"");
        assert_eq!(result, "https://example.com/&quot;quoted&quot;");

        // Single quotes and backticks should be escaped for attribute context
        let result = escape_href("https://example.com/path'");
        assert_eq!(result, "https://example.com/path&#x27;");

        let result = escape_href("https://example.com/`backtick`");
        assert_eq!(result, "https://example.com/&#x60;backtick&#x60;");
    }

    #[test]
    fn test_render_link_with_unsafe_url() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: "javascript:alert('xss')".to_string(),
            title: "".to_string(),
        }))));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("click me")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let html = render(&arena, root, 0);
        // Unsafe URL should be replaced with "#"
        assert!(
            html.contains("href=\"#\""),
            "Unsafe URL should be replaced with #"
        );
        assert!(
            !html.contains("javascript:"),
            "javascript: should not appear in output"
        );
    }
}
