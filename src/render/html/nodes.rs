//! Node handling for HTML renderer
//!
//! This module implements the enter_node and exit_node methods for HtmlRenderer.

use crate::core::arena::NodeId;
use crate::core::nodes::{ListType, NodeHeading, NodeList, NodeValue};
use crate::ext::gfm::tagfilter::filter_html;
use crate::text::html_utils::escape_html;
use std::fmt::Write;

use super::escaping::escape_href;
use crate::render::html::renderer::HtmlRenderer;

impl<'a> HtmlRenderer<'a> {
    pub(crate) fn enter_node(&mut self, node_id: NodeId) {
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
                self.lit("<blockquote");
                self.render_sourcepos(node_id);
                self.lit(">");
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
                        self.lit("<ul");
                        self.render_sourcepos(node_id);
                        self.lit(">");
                        self.tag_stack.push("ul");
                    }
                    ListType::Ordered => {
                        self.lit("<ol");
                        self.render_sourcepos(node_id);
                        if *start != 1 {
                            write!(self.output, " start=\"{}\">", start)
                                .expect("write to String cannot fail");
                        } else {
                            self.lit(">");
                        }
                        self.tag_stack.push("ol");
                    }
                }
                self.lit("\n");
            }
            NodeValue::Item(..) => {
                self.lit("<li");
                self.render_sourcepos(node_id);
                self.lit(">");
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
                // HTML blocks are output as raw HTML, but filtered if tagfilter is enabled
                if self.options.extension.tagfilter {
                    self.lit(&filter_html(&html_block.literal));
                } else {
                    self.lit(&html_block.literal);
                }
                self.lit("\n");
            }
            NodeValue::Paragraph => {
                // In tight lists, paragraphs are not wrapped in <p> tags
                if !self.in_tight_list() {
                    // Track as item child in loose lists too
                    self.track_item_child();
                    self.lit("<p");
                    self.render_sourcepos(node_id);
                    self.lit(">");
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
                // Optimization: use write! instead of format!
                write!(self.output, "<h{}", level).expect("write to String cannot fail");
                self.render_sourcepos(node_id);
                write!(self.output, ">").expect("write to String cannot fail");
                self.last_out = '>';
                self.tag_stack.push("h");
            }
            NodeValue::ThematicBreak(..) => {
                // In tight list items, add newline before thematic break if not first child
                if self.track_item_child() {
                    self.lit("\n");
                } else {
                    self.cr();
                }
                self.lit("<hr");
                self.render_sourcepos(node_id);
                self.lit(" />");
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
                if self.options.extension.tagfilter {
                    self.lit(&filter_html(literal.as_ref()));
                } else {
                    self.lit(literal.as_ref());
                }
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
                    self.lit(&escape_href(link.url.as_ref()));
                    self.lit("\"");
                    if !link.title.is_empty() {
                        self.lit(" title=\"");
                        self.lit(&escape_html(link.title.as_ref()));
                        self.lit("\"");
                    }
                    self.lit(">");
                }
            }
            NodeValue::Image(link) => {
                self.lit("<img src=\"");
                self.lit(&escape_href(link.url.as_ref()));
                self.lit("\" alt=\"");
                // Collect alt text from children
                let alt_text = self.collect_alt_text(node_id);
                self.lit(&escape_html(&alt_text));
                if !link.title.is_empty() {
                    self.lit("\" title=\"");
                    self.lit(&escape_html(link.title.as_ref()));
                }
                self.lit("\" />");
            }
            NodeValue::Strikethrough => {
                self.lit("<del>");
            }
            NodeValue::TaskItem(task_item) => {
                let checked = task_item.symbol.is_some();
                // Optimization: avoid format! for simple string concatenation
                self.lit("<input type=\"checkbox\" disabled=\"disabled\"");
                if checked {
                    self.lit(" checked=\"checked\"");
                }
                self.lit(" />");
            }
            NodeValue::FootnoteReference(footnote_ref) => {
                // Collect footnote for rendering at the end
                if let Some(def_id) = self.find_footnote_def(&footnote_ref.name) {
                    self.footnotes.push((footnote_ref.name.clone(), def_id));
                }
                // Optimization: use write! instead of format!
                let name_escaped = escape_html(&footnote_ref.name);
                write!(
                    self.output,
                    "<sup class=\"footnote-ref\"><a href=\"#fn-{}\" id=\"fnref-{}\">[{}]</a></sup>",
                    name_escaped, name_escaped, name_escaped
                )
                .expect("write to String cannot fail");
                self.last_out = '>';
            }
            NodeValue::FootnoteDefinition(footnote_def) => {
                // Footnote definitions are rendered at the end
                write!(
                    self.output,
                    "<li id=\"fn-{}\">",
                    escape_html(&footnote_def.name)
                )
                .expect("write to String cannot fail");
                self.last_out = '>';
                self.tag_stack.push("li");
            }
            NodeValue::Table(_table) => {
                self.lit("<table>");
                self.lit("\n");
                self.table_row_index = 0;
            }
            NodeValue::TableRow(is_header) => {
                if *is_header {
                    // Header end marker: close thead and start tbody
                    // Note: the </tr> for the header row was already output by the header row's exit_node
                    self.lit("</thead>");
                    self.lit("\n");
                    self.lit("<tbody>");
                    self.lit("\n");
                    self.table_row_index = 2; // Next row will be body row
                } else {
                    // Regular row
                    if self.table_row_index == 0 {
                        // First row is header
                        self.lit("<thead>");
                        self.lit("\n");
                    }
                    self.lit("<tr>");
                    self.lit("\n");
                }
            }
            NodeValue::TableCell => {
                if self.table_row_index == 0 {
                    // Header row
                    self.lit("<th>");
                    self.tag_stack.push("th");
                } else {
                    // Body row
                    self.lit("<td>");
                    self.tag_stack.push("td");
                }
            }
            NodeValue::EscapedTag(data) => {
                self.lit(data);
            }
            NodeValue::ShortCode(shortcode) => {
                self.lit(&shortcode.emoji);
            }
            _ => {}
        }
    }

    pub(crate) fn exit_node(&mut self, node_id: NodeId) {
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
                    write!(self.output, "</{}>", tag)
                        .expect("write to String cannot fail");
                    self.lit("\n");
                }
                // Pop tight status from stack
                self.tight_list_stack.pop();
            }
            NodeValue::Item(..) => {
                if let Some(tag) = self.tag_stack.pop() {
                    write!(self.output, "</{}>", tag)
                        .expect("write to String cannot fail");
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
                        write!(self.output, "</{}>", tag)
                            .expect("write to String cannot fail");
                        self.lit("\n");
                    }
                }
            }
            NodeValue::Heading(NodeHeading { level, .. }) => {
                writeln!(self.output, "</h{}>", level)
                    .expect("write to String cannot fail");
                self.last_out = '\n';
                self.tag_stack.pop();
            }
            NodeValue::ThematicBreak(..) => {}
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
                self.lit("</table>");
                self.lit("\n");
            }
            NodeValue::TableRow(is_header) => {
                if !*is_header {
                    // Only output </tr> for non-header-marker rows
                    self.lit("</tr>");
                    self.lit("\n");
                }
                // Increment row index after processing each row
                self.table_row_index += 1;
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
}
