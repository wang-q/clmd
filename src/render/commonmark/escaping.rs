//! Markdown escaping utilities
//!
//! This module provides context-aware escaping for Markdown content,
//! inspired by flexmark-java's escaping logic in CoreNodeFormatter.
//!
//! Different contexts require different escaping rules:
//! - Plain text: escape special Markdown characters
//! - Code spans: only escape backticks
//! - Code blocks: no escaping needed
//! - Link text: escape brackets
//! - Link URLs: escape parentheses and spaces
//! - HTML: no escaping needed

use crate::core::nodes::NodeValue;
use crate::formatter::context::NodeFormatterContext;

/// Characters that have special meaning in Markdown
const MARKDOWN_SPECIAL_CHARS: &[char] = &[
    '\\', '`', '*', '_', '{', '}', '[', ']', '<', '>', '(', ')', '#', '+', '-', '.',
    '!', '|',
];

/// Characters that need escaping at the start of a line
/// These are block-level markers that would be interpreted as structural elements
const LINE_START_SPECIAL_CHARS: &[char] =
    &['#', '>', '-', '+', '*', '=', '|', '`', '~', '<'];

/// Characters that need escaping in link text
const LINK_TEXT_SPECIAL_CHARS: &[char] = &['[', ']', '\\'];

/// Characters that need escaping in link URLs
const URL_SPECIAL_CHARS: &[char] = &['(', ')', ' ', '\t', '\n', '\r', '<', '>', '\\'];

/// Characters that need escaping in code spans
#[allow(dead_code)]
const CODE_SPAN_SPECIAL_CHARS: &[char] = &['`', '\\'];

/// Check if a character needs escaping in the given context
pub fn need_to_escape(ch: char, context: &dyn NodeFormatterContext) -> bool {
    if !is_markdown_special_char(ch) {
        return false;
    }

    // Get current node info
    let current_node = context.get_current_node();
    let parent_type = current_node.and_then(|id| {
        let arena = context.get_arena();
        let node = arena.get(id);
        node.parent.map(|pid| arena.get(pid).value.clone())
    });

    match ch {
        // Backslash always needs escaping (it's the escape character)
        '\\' => true,
        // Backtick needs escaping outside of code
        '`' => !is_in_code_context(context),
        // Asterisk and underscore are emphasis markers
        '*' | '_' => {
            // Don't escape inside code blocks or code spans
            if is_in_code_context(context) {
                return false;
            }
            // Check if this is part of an emphasis node
            if let Some(NodeValue::Emph | NodeValue::Strong) = parent_type {
                return false;
            }
            true
        }
        // Square brackets are link markers
        '[' | ']' => {
            if is_in_code_context(context) {
                return false;
            }
            // Don't escape inside link text
            if let Some(NodeValue::Link(_) | NodeValue::Image(_)) = parent_type {
                return false;
            }
            true
        }
        // Hash is heading marker at line start
        '#' => {
            if is_in_code_context(context) {
                return false;
            }
            // Check if at start of line
            is_at_line_start(context)
        }
        // Less-than can start HTML tags or autolinks
        '<' | '>' => {
            if is_in_code_context(context) {
                return false;
            }
            // Don't escape inside HTML nodes
            if let Some(NodeValue::HtmlInline(_) | NodeValue::HtmlBlock(_)) = parent_type
            {
                return false;
            }
            true
        }
        // Exclamation mark only needs escaping when followed by '['
        '!' => {
            if is_in_code_context(context) {
                return false;
            }
            // Check if followed by '['
            is_followed_by_bracket(context)
        }
        // Pipe character is table delimiter
        '|' => {
            if is_in_code_context(context) {
                return false;
            }
            // Check if inside a table
            is_inside_table(context)
        }
        // Parentheses only need escaping in specific contexts (like link URLs)
        '(' | ')' => {
            // In most contexts, parentheses don't need escaping
            // They only need escaping in link URLs to avoid ambiguity
            if is_in_code_context(context) {
                false
            } else {
                // Check if we're in a link URL context
                is_in_link_url_context(context)
            }
        }
        // Other special characters (curly braces, dots, etc.)
        _ => !is_in_code_context(context),
    }
}

/// Check if a character is a Markdown special character
fn is_markdown_special_char(ch: char) -> bool {
    MARKDOWN_SPECIAL_CHARS.contains(&ch)
}

/// Check if we're inside a code context (code block or code span)
fn is_in_code_context(context: &dyn NodeFormatterContext) -> bool {
    let current_node = context.get_current_node();
    if let Some(node_id) = current_node {
        let arena = context.get_arena();
        let mut current = node_id;

        while let Some(node) = arena.try_get(current) {
            match &node.value {
                NodeValue::Code(_) | NodeValue::CodeBlock(_) => return true,
                _ => {
                    if let Some(parent) = node.parent {
                        current = parent;
                    } else {
                        break;
                    }
                }
            }
        }
    }
    false
}

/// Check if we're at the start of a line
fn is_at_line_start(context: &dyn NodeFormatterContext) -> bool {
    // Check if we're inside a heading - if so, we're not really at line start
    // because the heading handler adds the prefix
    if let Some(node_id) = context.get_current_node() {
        let arena = context.get_arena();
        let mut current = node_id;

        // Walk up the tree to check if we're inside a heading
        while let Some(node) = arena.try_get(current) {
            match &node.value {
                NodeValue::Heading(_) => {
                    // We're inside a heading, so the # is not at line start
                    // (the heading handler adds the # prefix)
                    return false;
                }
                _ => {
                    if let Some(parent) = node.parent {
                        current = parent;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    // For now, assume it might be at line start to be safe
    true
}

/// Check if the current position is followed by a bracket
fn is_followed_by_bracket(_context: &dyn NodeFormatterContext) -> bool {
    // Simplified - in a full implementation, we'd check the next character
    true
}

/// Check if we're inside a table
fn is_inside_table(context: &dyn NodeFormatterContext) -> bool {
    let current_node = context.get_current_node();
    if let Some(node_id) = current_node {
        let arena = context.get_arena();
        let mut current = node_id;

        while let Some(node) = arena.try_get(current) {
            match &node.value {
                NodeValue::Table(_) | NodeValue::TableRow(_) | NodeValue::TableCell => {
                    return true
                }
                _ => {
                    if let Some(parent) = node.parent {
                        current = parent;
                    } else {
                        break;
                    }
                }
            }
        }
    }
    false
}

/// Check if we're inside a link URL context
/// Parentheses need escaping in link URLs to avoid ambiguity
fn is_in_link_url_context(_context: &dyn NodeFormatterContext) -> bool {
    // For now, assume we're not in a link URL context
    // In a full implementation, we'd track whether we're rendering a link URL
    false
}

/// Escape text according to the current context
///
/// This function applies context-aware escaping for Markdown special characters.
/// It handles:
/// - Line-start special characters (like # for headings, > for blockquotes)
/// - Inline special characters (like * and _ for emphasis)
/// - Context-specific escaping (different rules inside code, links, etc.)
pub fn escape_text(text: &str, context: &dyn NodeFormatterContext) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut at_line_start = true;

    for ch in &chars {
        match *ch {
            '\n' | '\r' => {
                // Newline - reset line start flag
                result.push(*ch);
                at_line_start = true;
            }
            '\\' => {
                // Always escape backslash
                result.push('\\');
                result.push('\\');
                at_line_start = false;
            }
            _ => {
                // Check if we need to escape this character
                // First check context-aware rules, then check line-start rules
                let needs_escape = if at_line_start
                    && LINE_START_SPECIAL_CHARS.contains(ch)
                    && !is_in_code_context(context)
                {
                    // At line start with a special character - check if context says it needs escaping
                    need_to_escape(*ch, context)
                } else {
                    // Not at line start or not a line-start special char, use normal escaping rules
                    need_to_escape(*ch, context)
                };

                if needs_escape {
                    result.push('\\');
                }
                result.push(*ch);
                at_line_start = false;
            }
        }
    }

    result
}

/// Escape text for use in code spans
/// Only escapes backticks and backslashes
pub fn escape_code_span(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for ch in text.chars() {
        match ch {
            '\\' | '`' => {
                result.push('\\');
                result.push(ch);
            }
            _ => result.push(ch),
        }
    }

    result
}

/// Escape link text
pub fn escape_link_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for ch in text.chars() {
        if LINK_TEXT_SPECIAL_CHARS.contains(&ch) {
            result.push('\\');
        }
        result.push(ch);
    }

    result
}

/// Escape URL for use in links
pub fn escape_url(url: &str) -> String {
    let mut result = String::with_capacity(url.len());

    for ch in url.chars() {
        if URL_SPECIAL_CHARS.contains(&ch) {
            result.push('\\');
        }
        result.push(ch);
    }

    result
}

/// Escape string for use in link title
///
/// Escapes quotes and backslashes in link titles.
pub fn escape_string(text: &str) -> String {
    text.replace('"', "\\\"").replace('\\', "\\\\")
}

/// Escape text for use in HTML attributes
pub fn escape_html_attribute(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for ch in text.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(ch),
        }
    }

    result
}

/// Check if text contains characters that would need escaping
pub fn needs_escaping(text: &str, context: &dyn NodeFormatterContext) -> bool {
    text.chars().any(|ch| need_to_escape(ch, context))
}

/// Choose the best emphasis marker for the given text
/// Returns '*' or '_' depending on which would require less escaping
pub fn choose_emphasis_marker(text: &str) -> char {
    let asterisk_count = text.matches('*').count();
    let underscore_count = text.matches('_').count();

    // Prefer the marker that doesn't appear in the text
    if asterisk_count == 0 && underscore_count > 0 {
        '*'
    } else if underscore_count == 0 && asterisk_count > 0 {
        '_'
    } else {
        // Both appear or neither appears - prefer asterisk
        '*'
    }
}

/// Compute the required fence length for code blocks or code spans
/// based on the content to ensure the fence doesn't appear in the content
pub fn compute_fence_length(content: &str, base_length: usize) -> usize {
    let mut max_consecutive = 0;
    let mut current = 0;

    for ch in content.chars() {
        if ch == '`' {
            current += 1;
            max_consecutive = max_consecutive.max(current);
        } else {
            current = 0;
        }
    }

    // Need one more backtick than the maximum consecutive sequence
    (max_consecutive + 1).max(base_length)
}

/// Normalize line endings to LF
pub fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Escape special regex characters in a string
pub fn escape_regex(text: &str) -> String {
    let special_chars = r"\.^$|?*+()[]{}";
    let mut result = String::with_capacity(text.len());

    for ch in text.chars() {
        if special_chars.contains(ch) {
            result.push('\\');
        }
        result.push(ch);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_markdown_special_char() {
        assert!(is_markdown_special_char('\\'));
        assert!(is_markdown_special_char('*'));
        assert!(is_markdown_special_char('_'));
        assert!(is_markdown_special_char('['));
        assert!(!is_markdown_special_char('a'));
        assert!(!is_markdown_special_char(' '));
    }

    #[test]
    fn test_escape_link_text() {
        assert_eq!(escape_link_text("[text]"), "\\[text\\]");
        assert_eq!(escape_link_text("normal"), "normal");
        assert_eq!(escape_link_text("a[b]c"), "a\\[b\\]c");
    }

    #[test]
    fn test_escape_url() {
        assert_eq!(escape_url("(url)"), "\\(url\\)");
        assert_eq!(escape_url("with space"), "with\\ space");
    }

    #[test]
    fn test_escape_code_span() {
        assert_eq!(escape_code_span("`code`"), "\\`code\\`");
        assert_eq!(escape_code_span("normal"), "normal");
        assert_eq!(escape_code_span("a\\b"), "a\\\\b");
    }

    #[test]
    fn test_escape_html_attribute() {
        assert_eq!(escape_html_attribute("<test>"), "&lt;test&gt;");
        assert_eq!(escape_html_attribute("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_html_attribute("a&b"), "a&amp;b");
    }

    #[test]
    fn test_choose_emphasis_marker() {
        assert_eq!(choose_emphasis_marker("no special"), '*');
        assert_eq!(choose_emphasis_marker("has_underscore"), '*');
        assert_eq!(choose_emphasis_marker("has*asterisk"), '_');
        assert_eq!(choose_emphasis_marker("has_both*"), '*');
    }

    #[test]
    fn test_compute_fence_length() {
        assert_eq!(compute_fence_length("code", 3), 3);
        assert_eq!(compute_fence_length("``", 3), 3);
        assert_eq!(compute_fence_length("```", 3), 4);
        assert_eq!(compute_fence_length("````", 3), 5);
        assert_eq!(compute_fence_length("code `inline` more", 3), 3);
        assert_eq!(compute_fence_length("code `` more", 3), 3);
    }

    #[test]
    fn test_normalize_line_endings() {
        assert_eq!(normalize_line_endings("a\r\nb"), "a\nb");
        assert_eq!(normalize_line_endings("a\rb"), "a\nb");
        assert_eq!(normalize_line_endings("a\nb"), "a\nb");
    }

    #[test]
    fn test_escape_regex() {
        assert_eq!(escape_regex("a.b"), "a\\.b");
        assert_eq!(escape_regex("a*b"), "a\\*b");
        assert_eq!(escape_regex("[test]"), "\\[test\\]");
    }
}
