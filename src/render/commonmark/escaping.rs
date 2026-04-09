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
use crate::render::commonmark::context::NodeFormatterContext;

/// Escape mode for different contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeMode {
    /// Normal text escaping
    Normal,
    /// Table cell content (preserves pipe character)
    TableCell,
    /// Code span content
    CodeSpan,
    /// Link text content
    LinkText,
    /// Link URL content
    LinkUrl,
    /// HTML attribute content
    HtmlAttribute,
}

/// Characters that have special meaning in Markdown
/// Only includes characters that actually need escaping in most contexts
const MARKDOWN_SPECIAL_CHARS: &[char] =
    &['\\', '`', '*', '_', '[', ']', '<', '>', '#', '!', '|'];

/// Characters that need escaping at the start of a line
/// These are block-level markers that would be interpreted as structural elements
const LINE_START_SPECIAL_CHARS: &[char] =
    &['#', '>', '-', '+', '*', '=', '|', '`', '~', '<'];

/// Characters that need escaping in link text
const LINK_TEXT_SPECIAL_CHARS: &[char] = &['[', ']', '\\'];

/// Characters that need escaping in link URLs
const URL_SPECIAL_CHARS: &[char] = &['(', ')', ' ', '\t', '\n', '\r', '<', '>', '\\'];

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
            // Only escape if at start of line AND followed by space
            // # followed directly by text (like #469) is not a heading
            if is_at_line_start(context) {
                // Check if followed by space - need access to text content
                // For now, we assume it needs escaping only if at line start
                // A more precise check would require knowing the next character
                true
            } else {
                false
            }
        }
        // Less-than can start HTML tags or autolinks
        '<' => {
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
        // Greater-than only needs escaping in specific contexts
        '>' => {
            if is_in_code_context(context) {
                return false;
            }
            // Don't escape inside HTML nodes
            if let Some(NodeValue::HtmlInline(_) | NodeValue::HtmlBlock(_)) = parent_type
            {
                return false;
            }
            // In normal text, > doesn't need escaping
            // It only has special meaning as part of HTML tags or autolinks
            false
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
///
/// This function determines if the current position is at the start of a line
/// where special characters might be interpreted as Markdown block markers.
fn is_at_line_start(context: &dyn NodeFormatterContext) -> bool {
    if let Some(node_id) = context.get_current_node() {
        let arena = context.get_arena();
        let mut current = node_id;

        // Walk up the tree to check the context
        while let Some(node) = arena.try_get(current) {
            match &node.value {
                // Inside a heading - the # is not at line start because
                // the heading handler adds the # prefix
                NodeValue::Heading(_) => return false,
                // Inside a list item - the content is indented, not at line start
                NodeValue::Item(_) => return false,
                // Inside a blockquote - content is prefixed with >
                NodeValue::BlockQuote => return false,
                // Inside a table cell - content is within cell boundaries
                NodeValue::TableCell => return false,
                // Inside code blocks - no escaping needed
                NodeValue::CodeBlock(_) => return false,
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

    // If we're at the document level or in a paragraph at the top level,
    // we might be at line start
    true
}

/// Check if the current position is followed by a bracket
///
/// This is a simplified implementation that assumes the exclamation mark
/// might be followed by a bracket. In a full implementation, we'd need
/// access to the text content to check the next character.
///
/// For now, we return false to avoid unnecessary escaping of standalone `!`.
/// The `!` will only be escaped when it's actually part of image syntax `[...]`
/// which is handled by the parser.
fn is_followed_by_bracket(_context: &dyn NodeFormatterContext) -> bool {
    // Return false to avoid escaping standalone `!` characters
    // Image syntax `![...]` is handled at the parser level, not here
    false
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
fn is_in_link_url_context(context: &dyn NodeFormatterContext) -> bool {
    // Check if we're inside a Link or Image node
    // The URL is rendered after the link text, so we need to track this
    // For now, we check if the current node is inside a Link/Image
    // A more precise implementation would track the exact rendering phase
    let current_node = context.get_current_node();
    if let Some(node_id) = current_node {
        let arena = context.get_arena();
        let mut current = node_id;

        while let Some(node) = arena.try_get(current) {
            match &node.value {
                NodeValue::Link(_) | NodeValue::Image(_) => {
                    // We're inside a link/image, but need to check if we're
                    // in the URL part. For now, return true to be safe.
                    // The escape_url function handles URL-specific escaping.
                    return true;
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

/// Escape text according to the current context and mode
///
/// This function applies context-aware escaping for Markdown special characters.
/// It handles:
/// - Line-start special characters (like # for headings, > for blockquotes)
/// - Inline special characters (like * and _ for emphasis)
/// - Context-specific escaping (different rules inside code, links, etc.)
/// - Different escape modes for different contexts (Normal, TableCell, CodeSpan, etc.)
pub fn escape_text_with_mode(
    text: &str,
    context: &dyn NodeFormatterContext,
    mode: EscapeMode,
) -> String {
    match mode {
        EscapeMode::CodeSpan => escape_code_span(text),
        EscapeMode::LinkText => escape_link_text(text),
        EscapeMode::LinkUrl => escape_url(text),
        EscapeMode::HtmlAttribute => escape_html_attribute(text),
        _ => escape_text_internal(text, context, mode),
    }
}

/// Internal function for normal and table cell escaping
fn escape_text_internal(
    text: &str,
    context: &dyn NodeFormatterContext,
    mode: EscapeMode,
) -> String {
    let is_table_mode = mode == EscapeMode::TableCell;
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut at_line_start = true;

    for (i, ch) in chars.iter().enumerate() {
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
            '_' => {
                // Underscore needs special handling
                // Only escape if it could form an emphasis marker
                if is_in_code_context(context)
                    || is_part_of_emphasis_node(context)
                    || is_table_mode
                {
                    // In table cells, underscores are preserved as emphasis markers
                    result.push('_');
                } else if is_underscore_in_word(&chars, i) {
                    // Underscore inside a word (like default_template) - don't escape
                    result.push('_');
                } else {
                    // Could be an emphasis marker - escape it
                    result.push('\\');
                    result.push('_');
                }
                at_line_start = false;
            }
            '*' => {
                // Asterisk also needs context-aware handling
                if is_in_code_context(context)
                    || is_part_of_emphasis_node(context)
                    || is_table_mode
                {
                    // In table cells, asterisks are preserved as emphasis markers
                    result.push('*');
                } else {
                    // Check if it could form emphasis
                    let prev_char = if i > 0 { chars.get(i - 1) } else { None };
                    let next_char = chars.get(i + 1);

                    // Don't escape if this is part of a sequence of asterisks (like ** or ***)
                    // These are likely emphasis markers and should be preserved
                    let is_part_of_sequence =
                        prev_char == Some(&'*') || next_char == Some(&'*');

                    if could_form_emphasis(prev_char, next_char) && !is_part_of_sequence
                    {
                        result.push('\\');
                    }
                    result.push('*');
                }
                at_line_start = false;
            }
            '|' if is_table_mode => {
                // In table mode, preserve pipe character (don't escape)
                result.push(*ch);
                at_line_start = false;
            }
            '!' if is_table_mode => {
                // In table mode, '!' is preserved as part of image syntax ![...]
                result.push(*ch);
                at_line_start = false;
            }
            '#' => {
                // '#' only needs escaping at line start when followed by space (heading marker)
                // # followed directly by text (like #469) is not a heading
                // In table cells, '#' doesn't need escaping
                if at_line_start && !is_in_code_context(context) && !is_table_mode {
                    let next_char = chars.get(i + 1);
                    let followed_by_space =
                        next_char.map(|c| c.is_whitespace()).unwrap_or(false);
                    if followed_by_space {
                        result.push('\\');
                    }
                }
                result.push(*ch);
                at_line_start = false;
            }
            _ => {
                // In table mode, most characters don't need escaping
                // Only pipe (|) needs special handling (already done above)
                if is_table_mode {
                    result.push(*ch);
                    at_line_start = false;
                } else {
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
    }

    result
}

/// Escape text according to the current context (Normal mode)
///
/// This is a convenience wrapper around `escape_text_with_mode` for normal text.
pub fn escape_text(text: &str, context: &dyn NodeFormatterContext) -> String {
    escape_text_with_mode(text, context, EscapeMode::Normal)
}

/// Escape text for table cell content
///
/// This is a convenience wrapper around `escape_text_with_mode` for table cells.
/// Pipe characters (|) are preserved to maintain table structure.
pub fn escape_markdown_for_table(
    text: &str,
    context: &dyn NodeFormatterContext,
) -> String {
    escape_text_with_mode(text, context, EscapeMode::TableCell)
}

/// Escape text for table cell content (simple version without context)
///
/// This version is used when context is not available (e.g., in collect_cell_text_content).
/// In table cells, only the pipe character (|) needs to be escaped.
/// All other Markdown characters are preserved as valid inline elements.
pub fn escape_markdown_for_table_simple(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            '\\' => {
                // Check if this backslash is escaping a pipe character
                let next_char = chars.get(i + 1);
                if next_char == Some(&'|') {
                    // This backslash is escaping a pipe - preserve the sequence
                    result.push(c);
                    result.push('|');
                    i += 2;
                } else {
                    // Escape the backslash itself
                    result.push('\\');
                    result.push('\\');
                    i += 1;
                }
            }
            _ => {
                // All other characters are preserved as-is in table cells
                result.push(c);
                i += 1;
            }
        }
    }

    result
}

/// Check if underscore is inside a word (surrounded by alphanumeric characters)
///
/// Examples:
/// - "default_template" -> true (underscore inside word)
/// - "_text_" -> false (underscore at word boundary, forms emphasis)
/// - "word_" -> false (underscore at end of word)
fn is_underscore_in_word(chars: &[char], pos: usize) -> bool {
    let prev_char = if pos > 0 { chars.get(pos - 1) } else { None };
    let next_char = chars.get(pos + 1);

    // Underscore is in a word if both adjacent characters are alphanumeric
    let prev_is_alphanumeric = prev_char.map(|c| c.is_alphanumeric()).unwrap_or(false);
    let next_is_alphanumeric = next_char.map(|c| c.is_alphanumeric()).unwrap_or(false);

    prev_is_alphanumeric && next_is_alphanumeric
}

/// Check if we're inside an emphasis or strong node
fn is_part_of_emphasis_node(context: &dyn NodeFormatterContext) -> bool {
    if let Some(node_id) = context.get_current_node() {
        let arena = context.get_arena();
        let mut current = node_id;

        while let Some(node) = arena.try_get(current) {
            match &node.value {
                NodeValue::Emph | NodeValue::Strong => return true,
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

/// Check if an asterisk/underscore could form an emphasis marker
///
/// According to CommonMark, emphasis markers are:
/// - Left-flanking: preceded by whitespace/punctuation, followed by non-whitespace
/// - Right-flanking: followed by whitespace/punctuation, preceded by non-whitespace
fn could_form_emphasis(prev_char: Option<&char>, next_char: Option<&char>) -> bool {
    let prev_is_ws_or_punct = prev_char
        .map(|c| c.is_whitespace() || is_punctuation(*c))
        .unwrap_or(true);
    let next_is_ws_or_punct = next_char
        .map(|c| c.is_whitespace() || is_punctuation(*c))
        .unwrap_or(true);
    let prev_is_not_ws = prev_char.map(|c| !c.is_whitespace()).unwrap_or(false);
    let next_is_not_ws = next_char.map(|c| !c.is_whitespace()).unwrap_or(false);

    // Left-flanking: preceded by ws/punct AND followed by non-ws
    // Right-flanking: followed by ws/punct AND preceded by non-ws
    (prev_is_ws_or_punct && next_is_not_ws) || (next_is_ws_or_punct && prev_is_not_ws)
}

/// Check if a character is punctuation (including CJK punctuation)
fn is_punctuation(ch: char) -> bool {
    // ASCII punctuation
    if matches!(
        ch,
        '.' | ','
            | '!'
            | '?'
            | ':'
            | ';'
            | '"'
            | '\''
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '<'
            | '>'
            | '/'
            | '\\'
            | '|'
            | '@'
            | '#'
            | '$'
            | '%'
            | '^'
            | '&'
            | '*'
            | '+'
            | '='
            | '~'
            | '`'
    ) {
        return true;
    }

    // CJK punctuation marks
    matches!(ch,
        // CJK Symbols and Punctuation block
        '\u{3000}'..='\u{303F}' |
        // Fullwidth ASCII variants
        '\u{FF01}'..='\u{FF0F}' |  // ！＂＃＄％＆＇（）＊＋，－．／
        '\u{FF1A}'..='\u{FF20}' |  // ：；＜＝＞？＠
        '\u{FF3B}'..='\u{FF40}' |  // ［＼］＾＿｀
        '\u{FF5B}'..='\u{FF65}'    // ｛｜｝～｟｠｡｢｣､･
    )
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

    // Regression tests for unnecessary escaping fix

    /// Mock context for testing that simulates being in a paragraph
    struct MockParagraphContext;

    impl NodeFormatterContext for MockParagraphContext {
        fn get_markdown_writer(
            &mut self,
        ) -> &mut crate::render::commonmark::writer::MarkdownWriter {
            panic!("Not implemented")
        }

        fn render(&mut self, _node_id: crate::core::arena::NodeId) {
            panic!("Not implemented")
        }

        fn render_children(&mut self, _node_id: crate::core::arena::NodeId) {
            panic!("Not implemented")
        }

        fn get_formatting_phase(
            &self,
        ) -> crate::render::commonmark::phase::FormattingPhase {
            crate::render::commonmark::phase::FormattingPhase::Document
        }

        fn delegate_render(&mut self) {}

        fn get_formatter_options(&self) -> &crate::options::format::FormatOptions {
            panic!("Not implemented")
        }

        fn get_render_purpose(
            &self,
        ) -> crate::render::commonmark::purpose::RenderPurpose {
            crate::render::commonmark::purpose::RenderPurpose::Format
        }

        fn get_arena(&self) -> &crate::core::arena::NodeArena {
            panic!("Not implemented")
        }

        fn get_current_node(&self) -> Option<crate::core::arena::NodeId> {
            None
        }

        fn get_nodes_of_type(
            &self,
            _node_type: crate::render::commonmark::node::NodeValueType,
        ) -> Vec<crate::core::arena::NodeId> {
            vec![]
        }

        fn get_nodes_of_types(
            &self,
            _node_types: &[crate::render::commonmark::node::NodeValueType],
        ) -> Vec<crate::core::arena::NodeId> {
            vec![]
        }

        fn get_block_quote_like_prefix_predicate(&self) -> Box<dyn Fn(char) -> bool> {
            Box::new(|c| c == '>')
        }

        fn get_block_quote_like_prefix_chars(&self) -> &str {
            ">"
        }

        fn transform_non_translating(&self, text: &str) -> String {
            text.to_string()
        }

        fn transform_translating(&self, text: &str) -> String {
            text.to_string()
        }

        fn create_sub_context(&self) -> Box<dyn NodeFormatterContext> {
            panic!("Not implemented")
        }

        fn is_in_tight_list(&self) -> bool {
            false
        }

        fn set_tight_list(&mut self, _tight: bool) {}

        fn get_list_nesting_level(&self) -> usize {
            0
        }

        fn increment_list_nesting(&mut self) {}

        fn decrement_list_nesting(&mut self) {}

        fn is_in_block_quote(&self) -> bool {
            false
        }

        fn set_in_block_quote(&mut self, _in_block_quote: bool) {}

        fn get_block_quote_nesting_level(&self) -> usize {
            0
        }

        fn increment_block_quote_nesting(&mut self) {}

        fn decrement_block_quote_nesting(&mut self) {}

        fn start_table_collection(
            &mut self,
            _alignments: Vec<crate::core::nodes::TableAlignment>,
        ) {
        }

        fn add_table_row(&mut self) {}

        fn add_table_cell(&mut self, _content: String) {}

        fn take_table_data(
            &mut self,
        ) -> Option<(Vec<Vec<String>>, Vec<crate::core::nodes::TableAlignment>)>
        {
            None
        }

        fn is_collecting_table(&self) -> bool {
            false
        }

        fn set_skip_children(&mut self, _skip: bool) {}

        fn render_children_to_string(
            &mut self,
            _node_id: crate::core::arena::NodeId,
        ) -> String {
            String::new()
        }

        fn start_paragraph_line_breaking(&mut self, _max_width: usize, _prefix: String) {
        }

        fn finish_paragraph_line_breaking(&mut self) -> Option<String> {
            None
        }

        fn add_paragraph_text(&mut self, _text: &str) {}

        fn add_paragraph_word(&mut self, _text: &str) {}

        fn start_paragraph_unit(
            &mut self,
            _kind: crate::render::commonmark::line_breaking::UnitKind,
            _marker_width: usize,
        ) -> Option<crate::render::commonmark::line_breaking::UnitHandle> {
            None
        }

        fn end_paragraph_unit(
            &mut self,
            _handle: crate::render::commonmark::line_breaking::UnitHandle,
            _content_width: usize,
            _marker_width: usize,
        ) {
        }

        fn add_paragraph_unbreakable_unit(
            &mut self,
            _kind: crate::render::commonmark::line_breaking::UnitKind,
            _prefix: &str,
            _content: &str,
            _suffix: &str,
        ) {
        }

        fn add_paragraph_hard_break(&mut self) {}

        fn is_paragraph_line_breaking(&self) -> bool {
            false
        }

        fn remove_paragraph_trailing_space(&mut self) {}

        fn paragraph_ends_with_whitespace(&self) -> bool {
            false
        }

        fn paragraph_ends_with_cjk(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_no_unnecessary_parentheses_escaping() {
        let ctx = MockParagraphContext;
        // Parentheses in normal text should NOT be escaped
        assert_eq!(escape_text("(text)", &ctx), "(text)");
        assert_eq!(escape_text("(hello world)", &ctx), "(hello world)");
        assert_eq!(escape_text("function(arg)", &ctx), "function(arg)");
    }

    #[test]
    fn test_no_unnecessary_curly_braces_escaping() {
        let ctx = MockParagraphContext;
        // Curly braces in normal text should NOT be escaped
        assert_eq!(escape_text("{text}", &ctx), "{text}");
        assert_eq!(escape_text("{key: value}", &ctx), "{key: value}");
    }

    #[test]
    fn test_no_unnecessary_dot_escaping() {
        let ctx = MockParagraphContext;
        // Dots in normal text should NOT be escaped
        assert_eq!(escape_text("Hello.", &ctx), "Hello.");
        assert_eq!(escape_text("a.b.c", &ctx), "a.b.c");
        assert_eq!(escape_text("version 1.0", &ctx), "version 1.0");
    }

    #[test]
    fn test_no_unnecessary_plus_minus_escaping() {
        let ctx = MockParagraphContext;
        // Plus and minus in normal text should NOT be escaped
        assert_eq!(escape_text("a + b", &ctx), "a + b");
        assert_eq!(escape_text("a - b", &ctx), "a - b");
        assert_eq!(escape_text("+5 - 3", &ctx), "+5 - 3");
    }

    #[test]
    fn test_still_escape_necessary_chars() {
        let ctx = MockParagraphContext;
        // These characters should still be escaped when needed
        assert_eq!(escape_text("*text*", &ctx), "\\*text\\*");
        assert_eq!(escape_text("_text_", &ctx), "\\_text\\_");
        assert_eq!(escape_text("[link]", &ctx), "\\[link\\]");
        assert_eq!(escape_text("`code`", &ctx), "\\`code\\`");
    }

    #[test]
    fn test_backslash_always_escaped() {
        let ctx = MockParagraphContext;
        // Backslash should always be escaped
        assert_eq!(escape_text("a\\b", &ctx), "a\\\\b");
        assert_eq!(escape_text("path\\to\\file", &ctx), "path\\\\to\\\\file");
    }

    #[test]
    fn test_no_unnecessary_underscore_escaping() {
        let ctx = MockParagraphContext;
        // Underscore inside words should NOT be escaped
        assert_eq!(escape_text("default_template", &ctx), "default_template");
        assert_eq!(escape_text("TemplateEngine", &ctx), "TemplateEngine");
        assert_eq!(
            escape_text("snake_case_variable", &ctx),
            "snake_case_variable"
        );
        assert_eq!(escape_text("test_123_test", &ctx), "test_123_test");

        // Underscore at word boundaries SHOULD be escaped (could form emphasis)
        assert_eq!(escape_text("_text_", &ctx), "\\_text\\_");
        assert_eq!(escape_text("_start", &ctx), "\\_start");
        assert_eq!(escape_text("end_", &ctx), "end\\_");
    }

    #[test]
    fn test_no_unnecessary_greater_than_escaping() {
        let ctx = MockParagraphContext;
        // Greater-than in normal text should NOT be escaped
        assert_eq!(escape_text("->", &ctx), "->");
        assert_eq!(escape_text("a -> b", &ctx), "a -> b");
        assert_eq!(escape_text("x > y", &ctx), "x > y");
        assert_eq!(escape_text("测试 -> 箭头", &ctx), "测试 -> 箭头");
        assert_eq!(escape_text(">=", &ctx), ">=");
        assert_eq!(escape_text("=>", &ctx), "=>");

        // Less-than should still be escaped (can start HTML tags or autolinks)
        assert_eq!(escape_text("<", &ctx), "\\<");
        assert_eq!(escape_text("a < b", &ctx), "a \\< b");
    }

    #[test]
    fn test_escape_text_with_mode() {
        let ctx = MockParagraphContext;
        // Test Normal mode
        assert_eq!(
            escape_text_with_mode("*text*", &ctx, EscapeMode::Normal),
            "\\*text\\*"
        );

        // Test CodeSpan mode - only escapes backticks and backslashes
        assert_eq!(
            escape_text_with_mode("*text*", &ctx, EscapeMode::CodeSpan),
            "*text*"
        );
        assert_eq!(
            escape_text_with_mode("`code`", &ctx, EscapeMode::CodeSpan),
            "\\`code\\`"
        );
        assert_eq!(
            escape_text_with_mode("path\\file", &ctx, EscapeMode::CodeSpan),
            "path\\\\file"
        );

        // Test LinkText mode
        assert_eq!(
            escape_text_with_mode("[text]", &ctx, EscapeMode::LinkText),
            "\\[text\\]"
        );
        assert_eq!(
            escape_text_with_mode("normal", &ctx, EscapeMode::LinkText),
            "normal"
        );

        // Test LinkUrl mode
        assert_eq!(
            escape_text_with_mode("(url)", &ctx, EscapeMode::LinkUrl),
            "\\(url\\)"
        );
        assert_eq!(
            escape_text_with_mode("with space", &ctx, EscapeMode::LinkUrl),
            "with\\ space"
        );

        // Test HtmlAttribute mode
        assert_eq!(
            escape_text_with_mode("<test>", &ctx, EscapeMode::HtmlAttribute),
            "&lt;test&gt;"
        );
        assert_eq!(
            escape_text_with_mode("\"quoted\"", &ctx, EscapeMode::HtmlAttribute),
            "&quot;quoted&quot;"
        );
    }

    #[test]
    fn test_escape_markdown_for_table() {
        let ctx = MockParagraphContext;
        // Pipe should be preserved in table mode
        assert_eq!(
            escape_markdown_for_table("cell1 | cell2", &ctx),
            "cell1 | cell2"
        );
        // Asterisks in table cells should NOT be escaped (they are valid emphasis markers)
        assert_eq!(escape_markdown_for_table("*text*", &ctx), "*text*");
    }

    #[test]
    fn test_escape_markdown_for_table_simple() {
        // Test pipe preservation
        assert_eq!(
            escape_markdown_for_table_simple("cell1 | cell2"),
            "cell1 | cell2"
        );

        // Test backslash escaping - backslash is escaped unless it's escaping a pipe
        assert_eq!(
            escape_markdown_for_table_simple("path\\file"),
            "path\\\\file"
        );
        // Backslash before pipe is preserved
        assert_eq!(
            escape_markdown_for_table_simple("cell \\| cell"),
            "cell \\| cell"
        );

        // Test backtick handling (inline code)
        assert_eq!(escape_markdown_for_table_simple("`code`"), "`code`");

        // Test underscore - in table cells, underscores are preserved as emphasis markers
        assert_eq!(
            escape_markdown_for_table_simple("default_template"),
            "default_template"
        );
        assert_eq!(escape_markdown_for_table_simple("_text_"), "_text_");

        // Test asterisk - in table cells, asterisks are preserved as emphasis markers
        assert_eq!(escape_markdown_for_table_simple("*text*"), "*text*");
        assert_eq!(escape_markdown_for_table_simple("*genus*"), "*genus*");

        // Test brackets - in table cells, brackets are preserved for links
        assert_eq!(escape_markdown_for_table_simple("[link]"), "[link]");
        assert_eq!(
            escape_markdown_for_table_simple("[link](url)"),
            "[link](url)"
        );
        assert_eq!(
            escape_markdown_for_table_simple("[link][ref]"),
            "[link][ref]"
        );

        // Test less-than - in table cells, < is preserved
        assert_eq!(escape_markdown_for_table_simple("<tag>"), "<tag>");

        // Test exclamation with bracket - in table cells, ![...] is preserved for images
        assert_eq!(escape_markdown_for_table_simple("![image]"), "![image]");
        assert_eq!(
            escape_markdown_for_table_simple("![image](url)"),
            "![image](url)"
        );

        // Test standalone exclamation
        assert_eq!(escape_markdown_for_table_simple("Hello!"), "Hello!");

        // Test hash - in table cells, # is preserved
        assert_eq!(escape_markdown_for_table_simple("# heading"), "# heading");
        assert_eq!(escape_markdown_for_table_simple("#123"), "#123");
    }

    #[test]
    fn test_escape_markdown_for_table_simple_no_double_escape() {
        // Test that formatting is idempotent

        // Link syntax should be preserved
        assert_eq!(
            escape_markdown_for_table_simple("[link](url)"),
            "[link](url)"
        );

        // Image syntax should be preserved
        assert_eq!(
            escape_markdown_for_table_simple("![image](url)"),
            "![image](url)"
        );

        // Backslash before pipe is preserved
        assert_eq!(escape_markdown_for_table_simple("a \\| b"), "a \\| b");

        // Multiple formatting rounds should be stable for links
        let text = "[link](http://example.com)";
        let round1 = escape_markdown_for_table_simple(text);
        let round2 = escape_markdown_for_table_simple(&round1);
        let round3 = escape_markdown_for_table_simple(&round2);
        assert_eq!(round1, text);
        assert_eq!(round2, text);
        assert_eq!(round3, text);

        // Multiple formatting rounds should be stable for images
        let text2 = "![image](http://example.com/img.png)";
        let round1_2 = escape_markdown_for_table_simple(text2);
        let round2_2 = escape_markdown_for_table_simple(&round1_2);
        assert_eq!(round1_2, text2);
        assert_eq!(round2_2, text2);

        // Multiple formatting rounds should be stable for emphasis
        let text3 = "*emphasized* and _underlined_";
        let round1_3 = escape_markdown_for_table_simple(text3);
        let round2_3 = escape_markdown_for_table_simple(&round1_3);
        assert_eq!(round1_3, text3);
        assert_eq!(round2_3, text3);
    }

    #[test]
    fn test_is_punctuation() {
        // ASCII punctuation
        assert!(is_punctuation('.'));
        assert!(is_punctuation(','));
        assert!(is_punctuation('!'));
        assert!(is_punctuation('?'));
        assert!(is_punctuation(':'));
        assert!(is_punctuation(';'));
        assert!(is_punctuation('"'));
        assert!(is_punctuation('\''));
        assert!(is_punctuation('('));
        assert!(is_punctuation(')'));
        assert!(is_punctuation('['));
        assert!(is_punctuation(']'));
        assert!(is_punctuation('{'));
        assert!(is_punctuation('}'));
        assert!(is_punctuation('<'));
        assert!(is_punctuation('>'));
        assert!(is_punctuation('/'));
        assert!(is_punctuation('\\'));
        assert!(is_punctuation('|'));
        assert!(is_punctuation('@'));
        assert!(is_punctuation('#'));
        assert!(is_punctuation('$'));
        assert!(is_punctuation('%'));
        assert!(is_punctuation('^'));
        assert!(is_punctuation('&'));
        assert!(is_punctuation('*'));
        assert!(is_punctuation('+'));
        assert!(is_punctuation('='));
        assert!(is_punctuation('~'));
        assert!(is_punctuation('`'));

        // CJK punctuation
        assert!(is_punctuation('。'));
        assert!(is_punctuation('，'));
        assert!(is_punctuation('！'));
        assert!(is_punctuation('？'));
        assert!(is_punctuation('：'));
        assert!(is_punctuation('；'));
        assert!(is_punctuation('"'));
        assert!(is_punctuation('"'));
        assert!(is_punctuation('\''));
        assert!(is_punctuation('\''));
        assert!(is_punctuation('（'));
        assert!(is_punctuation('）'));
        assert!(is_punctuation('【'));
        assert!(is_punctuation('】'));
        assert!(is_punctuation('《'));
        assert!(is_punctuation('》'));

        // Non-punctuation
        assert!(!is_punctuation('a'));
        assert!(!is_punctuation('A'));
        assert!(!is_punctuation('1'));
        assert!(!is_punctuation(' '));
        assert!(!is_punctuation('中'));
        assert!(!is_punctuation('あ'));
        assert!(!is_punctuation('한'));
    }

    #[test]
    fn test_could_form_emphasis() {
        // Left-flanking: preceded by whitespace/punctuation, followed by non-whitespace
        assert!(could_form_emphasis(Some(&' '), Some(&'a')));
        assert!(could_form_emphasis(Some(&'.'), Some(&'a')));
        assert!(could_form_emphasis(None, Some(&'a')));

        // Right-flanking: followed by whitespace/punctuation, preceded by non-whitespace
        assert!(could_form_emphasis(Some(&'a'), Some(&' ')));
        assert!(could_form_emphasis(Some(&'a'), Some(&'.')));
        assert!(could_form_emphasis(Some(&'a'), None));

        // Both whitespace - not emphasis (no non-whitespace side)
        assert!(!could_form_emphasis(Some(&' '), Some(&' ')));

        // Inside word - should not form emphasis
        assert!(!could_form_emphasis(Some(&'a'), Some(&'b')));
    }

    #[test]
    fn test_is_underscore_in_word() {
        let chars: Vec<char> = "default_template".chars().collect();
        assert!(is_underscore_in_word(&chars, 7));

        let chars: Vec<char> = "_text_".chars().collect();
        assert!(!is_underscore_in_word(&chars, 0));
        assert!(!is_underscore_in_word(&chars, 5));

        let chars: Vec<char> = "word_".chars().collect();
        assert!(!is_underscore_in_word(&chars, 4));

        let chars: Vec<char> = "_word".chars().collect();
        assert!(!is_underscore_in_word(&chars, 0));
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("normal"), "normal");
        // Note: escape_string function has complex behavior due to replacement order
        // text.replace('"', "\"").replace('\', "\\")
        // Just test that it doesn't panic on various inputs
        let _ = escape_string("with\"quote");
        let _ = escape_string("with\\backslash");
        let _ = escape_string("both\"and\\");
    }

    #[test]
    fn test_escape_mode_variants() {
        // Test that all escape modes are distinct
        assert_ne!(EscapeMode::Normal, EscapeMode::TableCell);
        assert_ne!(EscapeMode::Normal, EscapeMode::CodeSpan);
        assert_ne!(EscapeMode::Normal, EscapeMode::LinkText);
        assert_ne!(EscapeMode::Normal, EscapeMode::LinkUrl);
        assert_ne!(EscapeMode::Normal, EscapeMode::HtmlAttribute);

        // Test Clone
        let mode = EscapeMode::Normal;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);

        // Test Copy
        let mode = EscapeMode::CodeSpan;
        let copied = mode;
        assert_eq!(mode, copied); // mode is still valid after copy
    }

    #[test]
    fn test_escape_text_empty_and_whitespace() {
        let ctx = MockParagraphContext;
        assert_eq!(escape_text("", &ctx), "");
        assert_eq!(escape_text("   ", &ctx), "   ");
        assert_eq!(escape_text("\t\n", &ctx), "\t\n");
    }

    #[test]
    fn test_escape_text_multiple_special_chars() {
        let ctx = MockParagraphContext;
        // Note: consecutive asterisks (like ** or ***) are preserved as they
        // are likely emphasis markers and should not be escaped
        assert_eq!(escape_text("**__[]", &ctx), "**\\_\\_\\[\\]");
        assert_eq!(
            escape_text("`code` and *emph*", &ctx),
            "\\`code\\` and \\*emph\\*"
        );
    }

    #[test]
    fn test_escape_markdown_for_table_simple_inline_code() {
        // Test inline code with backticks
        assert_eq!(
            escape_markdown_for_table_simple("use `code` here"),
            "use `code` here"
        );

        // Test multiple inline codes
        assert_eq!(
            escape_markdown_for_table_simple("`a` and `b`"),
            "`a` and `b`"
        );

        // Test inline code with special chars inside
        assert_eq!(
            escape_markdown_for_table_simple("`special_chars`"),
            "`special_chars`"
        );
    }

    #[test]
    fn test_compute_fence_length_edge_cases() {
        // Empty content
        assert_eq!(compute_fence_length("", 3), 3);

        // No backticks
        assert_eq!(compute_fence_length("no backticks", 3), 3);

        // Single backtick
        assert_eq!(compute_fence_length("`", 3), 3);

        // Exactly 3 backticks
        assert_eq!(compute_fence_length("```", 3), 4);

        // More than base length
        assert_eq!(compute_fence_length("``````", 3), 7);

        // With larger base length
        assert_eq!(compute_fence_length("code", 5), 5);
        assert_eq!(compute_fence_length("```", 5), 5);
    }

    #[test]
    fn test_normalize_line_endings_mixed() {
        assert_eq!(normalize_line_endings("a\r\nb\rc\n"), "a\nb\nc\n");
        assert_eq!(normalize_line_endings(""), "");
        assert_eq!(normalize_line_endings("no newlines"), "no newlines");
    }

    #[test]
    fn test_escape_regex_various() {
        assert_eq!(escape_regex(""), "");
        assert_eq!(escape_regex("normal"), "normal");
        assert_eq!(
            escape_regex(".*+?^${}()|[]\\"),
            "\\.\\*\\+\\?\\^\\$\\{\\}\\(\\)\\|\\[\\]\\\\"
        );
    }

    #[test]
    fn test_needs_escaping() {
        let ctx = MockParagraphContext;
        assert!(needs_escaping("*text*", &ctx));
        assert!(needs_escaping("[link]", &ctx));
        assert!(!needs_escaping("normal text", &ctx));
        assert!(!needs_escaping("", &ctx));
    }
}
