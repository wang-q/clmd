//! HTML Utilities - HTML escaping and tag generation
//!
//! This module provides utilities for safe HTML generation,
//! including HTML entity escaping and tag building.
//!
//! Inspired by flexmark-java's flexmark-util-html.

/// Escape special HTML characters
///
/// Converts the following characters to their HTML entities:
/// - `&` → `&amp;`
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `"` → `&quot;`
/// - `'` → `&#x27;`
pub fn escape_html(input: &str) -> String {
    // Fast path: check if any escaping is needed
    let bytes = input.as_bytes();
    let mut needs_escape = false;
    for &b in bytes {
        if matches!(b, b'&' | b'<' | b'>' | b'"' | b'\'') {
            needs_escape = true;
            break;
        }
    }

    if !needs_escape {
        return input.to_string();
    }

    // Slow path: escape needed
    let mut result = String::with_capacity(input.len() * 2);
    for &b in bytes {
        match b {
            b'&' => result.push_str("&amp;"),
            b'<' => result.push_str("&lt;"),
            b'>' => result.push_str("&gt;"),
            b'"' => result.push_str("&quot;"),
            b'\'' => result.push_str("&#x27;"),
            _ => result.push(b as char),
        }
    }
    result
}

/// Escape HTML content for attribute values
///
/// Similar to `escape_html` but also escapes whitespace characters
/// that are significant in attributes.
pub fn escape_html_attribute(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#x27;"),
            '\n' => result.push_str("&#10;"),
            '\r' => result.push_str("&#13;"),
            '\t' => result.push_str("&#9;"),
            _ => result.push(c),
        }
    }
    result
}

/// Check if a character needs HTML escaping
pub fn needs_escaping(c: char) -> bool {
    matches!(c, '&' | '<' | '>' | '"' | '\'')
}

/// HTML tag builder for constructing HTML elements
pub struct HtmlBuilder {
    output: String,
}

impl HtmlBuilder {
    /// Create a new HTML builder
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    /// Create a new HTML builder with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            output: String::with_capacity(capacity),
        }
    }

    /// Start an HTML tag
    pub fn start_tag(&mut self, tag: &str) -> &mut Self {
        self.output.push('<');
        self.output.push_str(tag);
        self.output.push('>');
        self
    }

    /// Start an HTML tag with attributes
    pub fn start_tag_with_attrs(
        &mut self,
        tag: &str,
        attrs: &[(&str, &str)],
    ) -> &mut Self {
        self.output.push('<');
        self.output.push_str(tag);
        for (name, value) in attrs {
            self.output.push(' ');
            self.output.push_str(name);
            self.output.push_str("=\"");
            self.output.push_str(&escape_html_attribute(value));
            self.output.push('"');
        }
        self.output.push('>');
        self
    }

    /// End an HTML tag
    pub fn end_tag(&mut self, tag: &str) -> &mut Self {
        self.output.push('<');
        self.output.push('/');
        self.output.push_str(tag);
        self.output.push('>');
        self
    }

    /// Add a self-closing tag
    pub fn self_closing_tag(&mut self, tag: &str) -> &mut Self {
        self.output.push('<');
        self.output.push_str(tag);
        self.output.push_str(" />");
        self
    }

    /// Add a self-closing tag with attributes
    pub fn self_closing_tag_with_attrs(
        &mut self,
        tag: &str,
        attrs: &[(&str, &str)],
    ) -> &mut Self {
        self.output.push('<');
        self.output.push_str(tag);
        for (name, value) in attrs {
            self.output.push(' ');
            self.output.push_str(name);
            self.output.push_str("=\"");
            self.output.push_str(&escape_html_attribute(value));
            self.output.push('"');
        }
        self.output.push_str(" />");
        self
    }

    /// Add raw text content (without escaping)
    pub fn raw(&mut self, text: &str) -> &mut Self {
        self.output.push_str(text);
        self
    }

    /// Add text content (with HTML escaping)
    pub fn text(&mut self, text: &str) -> &mut Self {
        self.output.push_str(&escape_html(text));
        self
    }

    /// Add a complete element with text content
    pub fn element(&mut self, tag: &str, text: &str) -> &mut Self {
        self.start_tag(tag);
        self.text(text);
        self.end_tag(tag);
        self
    }

    /// Add a line break
    pub fn newline(&mut self) -> &mut Self {
        self.output.push('\n');
        self
    }

    /// Get the built HTML string
    pub fn build(self) -> String {
        self.output
    }

    /// Get a reference to the current output
    pub fn as_str(&self) -> &str {
        &self.output
    }

    /// Check if the output is empty
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }

    /// Get the length of the output
    pub fn len(&self) -> usize {
        self.output.len()
    }
}

impl Default for HtmlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// HTML5 void elements (self-closing tags)
pub const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
    "source", "track", "wbr",
];

/// Check if a tag is a void element
pub fn is_void_element(tag: &str) -> bool {
    VOID_ELEMENTS.contains(&tag.to_ascii_lowercase().as_str())
}

/// HTML entity definitions
pub mod entities {
    /// HTML entities mapping
    pub const ENTITIES: &[(&str, &str)] = &[
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#x27;", "'"),
        ("&#39;", "'"),
        ("&nbsp;", " "),
        ("&copy;", "©"),
        ("&reg;", "®"),
        ("&trade;", "™"),
        ("&mdash;", "—"),
        ("&ndash;", "–"),
        ("&hellip;", "…"),
        ("&laquo;", "«"),
        ("&raquo;", "»"),
        ("&ldquo;", "\""),
        ("&rdquo;", "\""),
        ("&lsquo;", "'"),
        ("&rsquo;", "'"),
    ];

    /// Decode HTML entities in a string
    pub fn decode_entities(input: &str) -> String {
        let mut result = input.to_string();
        for (entity, char_) in ENTITIES {
            result = result.replace(*entity, *char_);
        }
        result
    }
}

/// Safe HTML wrapper that prevents double-escaping
#[derive(Clone, Debug)]
pub struct SafeHtml(String);

impl SafeHtml {
    /// Create a new SafeHtml from a string
    /// The input is assumed to already be HTML-safe
    pub fn new(html: impl Into<String>) -> Self {
        Self(html.into())
    }

    /// Get the inner HTML string
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Get a reference to the inner HTML
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for SafeHtml {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<SafeHtml> for String {
    fn from(safe: SafeHtml) -> Self {
        safe.0
    }
}

/// Check if a URL is safe to use in HTML output
///
/// This function checks for potentially dangerous URL schemes like
/// javascript:, vbscript:, file:, and unsafe data: URLs.
///
/// # Arguments
///
/// * `url` - The URL to check
///
/// # Returns
///
/// `true` if the URL is considered safe, `false` otherwise
///
/// # Examples
///
/// ```
/// use clmd::html_utils::is_safe_url;
///
/// assert!(is_safe_url("https://example.com"));
/// assert!(is_safe_url("http://example.com"));
/// assert!(!is_safe_url("javascript:alert('xss')"));
/// ```
pub fn is_safe_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();

    // Check for unsafe protocols
    let is_unsafe = url_lower.starts_with("javascript:")
        || url_lower.starts_with("vbscript:")
        || url_lower.starts_with("file:")
        || (url_lower.starts_with("data:") && !is_safe_data_url(&url_lower));

    !is_unsafe
}

/// Check if a data URL is safe (only allows image types)
///
/// Currently allows: png, gif, jpeg, webp
fn is_safe_data_url(url: &str) -> bool {
    // Allow data:image/* URLs
    url.starts_with("data:image/png")
        || url.starts_with("data:image/gif")
        || url.starts_with("data:image/jpeg")
        || url.starts_with("data:image/webp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_html("it's"), "it&#x27;s");
    }

    #[test]
    fn test_escape_html_attribute() {
        assert_eq!(escape_html_attribute("a\nb"), "a&#10;b");
        assert_eq!(escape_html_attribute("a\tb"), "a&#9;b");
    }

    #[test]
    fn test_needs_escaping() {
        assert!(needs_escaping('<'));
        assert!(needs_escaping('>'));
        assert!(needs_escaping('&'));
        assert!(!needs_escaping('a'));
    }

    #[test]
    fn test_html_builder_basic() {
        let mut builder = HtmlBuilder::new();
        builder.start_tag("p");
        builder.text("Hello & goodbye");
        builder.end_tag("p");
        let html = builder.build();
        assert_eq!(html, "<p>Hello &amp; goodbye</p>");
    }

    #[test]
    fn test_html_builder_with_attrs() {
        let mut builder = HtmlBuilder::new();
        builder.start_tag_with_attrs("a", &[("href", "https://example.com")]);
        builder.text("Link");
        builder.end_tag("a");
        let html = builder.build();
        assert_eq!(html, "<a href=\"https://example.com\">Link</a>");
    }

    #[test]
    fn test_html_builder_self_closing() {
        let mut builder = HtmlBuilder::new();
        builder.self_closing_tag("br");
        let html = builder.build();
        assert_eq!(html, "<br />");
    }

    #[test]
    fn test_html_builder_element() {
        let mut builder = HtmlBuilder::new();
        builder.element("strong", "bold text");
        let html = builder.build();
        assert_eq!(html, "<strong>bold text</strong>");
    }

    #[test]
    fn test_is_void_element() {
        assert!(is_void_element("br"));
        assert!(is_void_element("img"));
        assert!(is_void_element("hr"));
        assert!(!is_void_element("div"));
        assert!(!is_void_element("p"));
    }

    #[test]
    fn test_entities_decode() {
        assert_eq!(entities::decode_entities("&lt;div&gt;"), "<div>");
        assert_eq!(entities::decode_entities("&amp;"), "&");
    }

    #[test]
    fn test_safe_html() {
        let safe = SafeHtml::new("<b>Bold</b>");
        assert_eq!(safe.as_str(), "<b>Bold</b>");
    }
}
