//! HTML document reader.
//!
//! Reads HTML and converts it to Markdown AST.
//!
//! # Example
//!
//! ```ignore
//! use clmd::io::reader::{HtmlReader, Reader};
//! use clmd::options::ReaderOptions;
//!
//! let reader = HtmlReader;
//! let options = ReaderOptions::default();
//! let (arena, root) = reader.read("<h1>Hello</h1>", &options).unwrap();
//! ```ignore

use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::io::reader::Reader;
use crate::options::{InputFormat, ReaderOptions};
use crate::parse;

/// HTML element conversion rules
#[derive(Debug, Clone, Copy, Default)]
pub struct ConversionRules {
    /// Whether to wrap text at certain column
    pub wrap_column: Option<usize>,
    /// Whether to convert <br> to two spaces + newline
    pub hard_breaks: bool,
}

/// Convert HTML to Markdown
pub fn html_to_markdown(html: &str) -> String {
    let rules = ConversionRules::default();
    html_to_markdown_with_rules(html, &rules)
}

/// Convert HTML to Markdown with custom rules
pub fn html_to_markdown_with_rules(html: &str, rules: &ConversionRules) -> String {
    let mut converter = HtmlToMarkdown::new(*rules);
    converter.convert(html)
}

/// HTML document reader.
///
/// Reads HTML and converts it to Markdown AST.
#[derive(Debug, Clone, Copy)]
pub struct HtmlReader;

impl HtmlReader {
    /// Create a new HTML reader.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for HtmlReader {
    fn read(
        &self,
        input: &str,
        _options: &ReaderOptions,
    ) -> ClmdResult<(NodeArena, NodeId)> {
        // Convert HTML to Markdown, then parse
        let markdown = html_to_markdown(input);
        let parser_options = crate::options::Options::default();
        Ok(parse::parse_document(&markdown, &parser_options))
    }

    fn format(&self) -> &'static str {
        "html"
    }

    fn extensions(&self) -> &[&'static str] {
        &["html", "htm"]
    }

    fn input_format(&self) -> InputFormat {
        InputFormat::Html
    }
}

/// HTML to Markdown converter state
struct HtmlToMarkdown {
    rules: ConversionRules,
    output: String,
    list_stack: Vec<ListType>,
    list_counters: Vec<u32>,
    in_code_block: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ListType {
    Ordered,
    Unordered,
}

impl HtmlToMarkdown {
    fn new(rules: ConversionRules) -> Self {
        Self {
            rules,
            output: String::new(),
            list_stack: Vec::new(),
            list_counters: Vec::new(),
            in_code_block: false,
        }
    }

    fn convert(&mut self, html: &str) -> String {
        // Simple state machine parser for HTML
        let chars = html.chars().peekable();
        let mut in_tag = false;
        let mut tag_name = String::new();
        let mut text_content = String::new();

        for ch in chars {
            match ch {
                '<' => {
                    // Process accumulated text
                    if !text_content.is_empty() {
                        self.process_text(&text_content);
                        text_content.clear();
                    }
                    in_tag = true;
                    tag_name.clear();
                }
                '>' if in_tag => {
                    in_tag = false;
                    self.process_tag(&tag_name);
                }
                _ if in_tag => {
                    tag_name.push(ch);
                }
                _ => {
                    text_content.push(ch);
                }
            }
        }

        // Process remaining text
        if !text_content.is_empty() {
            self.process_text(&text_content);
        }

        self.output.trim().to_string()
    }

    fn process_tag(&mut self, tag: &str) {
        let tag = tag.trim();
        let is_closing = tag.starts_with('/');
        let tag_name = if is_closing {
            &tag[1..]
        } else {
            tag.split_whitespace().next().unwrap_or(tag)
        };

        let tag_lower = tag_name.to_lowercase();

        match tag_lower.as_str() {
            // Headings
            "h1" => self.handle_heading(1, is_closing),
            "h2" => self.handle_heading(2, is_closing),
            "h3" => self.handle_heading(3, is_closing),
            "h4" => self.handle_heading(4, is_closing),
            "h5" => self.handle_heading(5, is_closing),
            "h6" => self.handle_heading(6, is_closing),

            // Paragraph
            "p" => self.handle_paragraph(is_closing),

            // Line break
            "br" => self.handle_br(),
            "hr" => self.handle_hr(),

            // Emphasis
            "em" | "i" => self.handle_emphasis("*", is_closing),
            "strong" | "b" => self.handle_emphasis("**", is_closing),

            // Code
            "code" => self.handle_code_inline(is_closing),
            "pre" => self.handle_code_block(is_closing),

            // Links
            "a" => self.handle_link(tag, is_closing),

            // Images
            "img" => self.handle_img(tag),

            // Lists
            "ul" => self.handle_list(ListType::Unordered, is_closing),
            "ol" => self.handle_list(ListType::Ordered, is_closing),
            "li" => self.handle_list_item(is_closing),

            // Blockquote
            "blockquote" => self.handle_blockquote(is_closing),

            // Strikethrough (GFM)
            "del" | "s" => self.handle_emphasis("~~", is_closing),

            // Tables (simplified)
            "table" => self.handle_table(is_closing),
            "tr" => self.handle_table_row(is_closing),
            "td" | "th" => self.handle_table_cell(is_closing),

            // Div and span (mostly ignored)
            "div" => self.handle_div(is_closing),
            "span" => {} // Inline, usually styling - ignore

            _ => {}
        }
    }

    fn process_text(&mut self, text: &str) {
        if self.in_code_block {
            self.output.push_str(text);
        } else {
            // Escape special Markdown characters
            let escaped = text
                .replace("\\", "\\\\")
                .replace("*", "\\*")
                .replace("_", "\\_")
                .replace("[", "\\[")
                .replace("]", "\\]");
            self.output.push_str(&escaped);
        }
    }

    fn handle_heading(&mut self, level: usize, is_closing: bool) {
        if !is_closing {
            self.ensure_newline();
            self.output.push_str(&"#".repeat(level));
            self.output.push(' ');
        } else {
            self.output.push('\n');
        }
    }

    fn handle_paragraph(&mut self, is_closing: bool) {
        if is_closing {
            self.output.push_str("\n\n");
        }
    }

    fn handle_br(&mut self) {
        if self.rules.hard_breaks {
            self.output.push_str("  \n");
        } else {
            self.output.push('\n');
        }
    }

    fn handle_hr(&mut self) {
        self.ensure_newline();
        self.output.push_str("---\n\n");
    }

    fn handle_emphasis(&mut self, marker: &str, _is_closing: bool) {
        self.output.push_str(marker);
    }

    fn handle_code_inline(&mut self, _is_closing: bool) {
        self.output.push('`');
    }

    fn handle_code_block(&mut self, is_closing: bool) {
        if !is_closing {
            self.ensure_newline();
            self.output.push_str("```\n");
            self.in_code_block = true;
        } else {
            self.output.push_str("\n```\n\n");
            self.in_code_block = false;
        }
    }

    fn handle_link(&mut self, tag: &str, is_closing: bool) {
        if !is_closing {
            // Extract href from tag
            if let Some(_href) = extract_attribute(tag, "href") {
                self.output.push('[');
                // Store href for later
                // This is simplified - in a real implementation we'd need to track state
            }
        } else {
            self.output.push_str("](url)"); // Placeholder
        }
    }

    fn handle_img(&mut self, tag: &str) {
        let src = extract_attribute(tag, "src").unwrap_or_default();
        let alt = extract_attribute(tag, "alt").unwrap_or_default();
        self.output.push_str(&format!("![{}]({})", alt, src));
    }

    fn handle_list(&mut self, list_type: ListType, is_closing: bool) {
        if !is_closing {
            self.list_stack.push(list_type);
            if list_type == ListType::Ordered {
                self.list_counters.push(1);
            } else {
                self.list_counters.push(0);
            }
        } else {
            self.list_stack.pop();
            self.list_counters.pop();
            if self.list_stack.is_empty() {
                self.output.push('\n');
            }
        }
    }

    fn handle_list_item(&mut self, is_closing: bool) {
        if !is_closing {
            self.ensure_newline();
            let indent = "  ".repeat(self.list_stack.len().saturating_sub(1));
            self.output.push_str(&indent);

            if let Some(list_type) = self.list_stack.last() {
                match list_type {
                    ListType::Ordered => {
                        let counter_idx = self.list_counters.len() - 1;
                        let n = self.list_counters[counter_idx];
                        self.output.push_str(&format!("{}. ", n));
                        self.list_counters[counter_idx] = n + 1;
                    }
                    ListType::Unordered => {
                        self.output.push_str("- ");
                    }
                }
            }
        } else {
            self.output.push('\n');
        }
    }

    fn handle_blockquote(&mut self, is_closing: bool) {
        if !is_closing {
            self.ensure_newline();
            self.output.push_str("> ");
        } else {
            self.output.push('\n');
        }
    }

    fn handle_table(&mut self, is_closing: bool) {
        if is_closing {
            self.output.push('\n');
        }
    }

    fn handle_table_row(&mut self, is_closing: bool) {
        if !is_closing {
            self.ensure_newline();
        } else {
            self.output.push_str(" |\n");
        }
    }

    fn handle_table_cell(&mut self, is_closing: bool) {
        if !is_closing {
            self.output.push_str("| ");
        } else {
            self.output.push(' ');
        }
    }

    fn handle_div(&mut self, is_closing: bool) {
        if is_closing {
            self.output.push('\n');
        }
    }

    fn ensure_newline(&mut self) {
        if !self.output.ends_with('\n') && !self.output.is_empty() {
            self.output.push('\n');
        }
    }
}

/// Extract an attribute value from an HTML tag
fn extract_attribute(tag: &str, attr_name: &str) -> Option<String> {
    let attr_pattern = format!("{}=\"", attr_name);
    if let Some(start) = tag.find(&attr_pattern) {
        let value_start = start + attr_pattern.len();
        if let Some(end) = tag[value_start..].find('"') {
            return Some(tag[value_start..value_start + end].to_string());
        }
    }

    // Try single quotes
    let attr_pattern = format!("{}='", attr_name);
    if let Some(start) = tag.find(&attr_pattern) {
        let value_start = start + attr_pattern.len();
        if let Some(end) = tag[value_start..].find('\'') {
            return Some(tag[value_start..value_start + end].to_string());
        }
    }

    None
}

/// Quick check if content looks like HTML
pub fn is_html(content: &str) -> bool {
    content.trim_start().starts_with('<') && content.contains('>')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_reader_basic() {
        let reader = HtmlReader::new();
        let options = ReaderOptions::default();

        let (arena, root) = reader.read("<h1>Hello</h1>", &options).unwrap();
        let node = arena.get(root);
        assert!(matches!(
            node.value,
            crate::core::nodes::NodeValue::Document
        ));
    }

    #[test]
    fn test_html_reader_format() {
        let reader = HtmlReader::new();
        assert_eq!(reader.format(), "html");
        assert!(reader.extensions().contains(&"html"));
        assert!(reader.extensions().contains(&"htm"));
    }

    #[test]
    fn test_convert_heading() {
        let md = html_to_markdown("<h1>Title</h1>");
        assert!(md.contains("# Title"));
    }

    #[test]
    fn test_convert_paragraph() {
        let md = html_to_markdown("<p>Hello world</p>");
        assert!(md.contains("Hello world"));
    }

    #[test]
    fn test_convert_emphasis() {
        let md = html_to_markdown("<em>italic</em>");
        assert!(md.contains("*italic*"));

        let md = html_to_markdown("<strong>bold</strong>");
        assert!(md.contains("**bold**"));
    }

    #[test]
    fn test_convert_code() {
        let md = html_to_markdown("<code>inline code</code>");
        assert!(md.contains("`inline code`"));
    }

    #[test]
    fn test_convert_code_block() {
        let md = html_to_markdown("<pre><code>code block</code></pre>");
        assert!(md.contains("```"));
        assert!(md.contains("code block"));
    }

    #[test]
    fn test_convert_list() {
        let md = html_to_markdown("<ul><li>Item 1</li><li>Item 2</li></ul>");
        assert!(md.contains("- Item 1"));
        assert!(md.contains("- Item 2"));
    }

    #[test]
    fn test_convert_ordered_list() {
        let md = html_to_markdown("<ol><li>First</li><li>Second</li></ol>");
        assert!(md.contains("1. First"));
        assert!(md.contains("2. Second"));
    }

    #[test]
    fn test_convert_blockquote() {
        let md = html_to_markdown("<blockquote>Quote</blockquote>");
        assert!(md.contains("> Quote"));
    }

    #[test]
    fn test_convert_strikethrough() {
        let md = html_to_markdown("<del>deleted</del>");
        assert!(md.contains("~~deleted~~"));
    }

    #[test]
    fn test_convert_image() {
        let md = html_to_markdown(r#"<img src="image.png" alt="Description">"#);
        assert!(md.contains("![Description](image.png)"));
    }

    #[test]
    fn test_is_html() {
        assert!(is_html("<p>text</p>"));
        assert!(is_html("  <div>content</div>"));
        assert!(!is_html("Just plain text"));
    }

    #[test]
    fn test_extract_attribute() {
        assert_eq!(
            extract_attribute(r#"href="https://example.com""#, "href"),
            Some("https://example.com".to_string())
        );
        assert_eq!(
            extract_attribute("src='image.png'", "src"),
            Some("image.png".to_string())
        );
        assert_eq!(extract_attribute("<p>", "class"), None);
    }
}
