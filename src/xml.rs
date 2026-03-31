//! XML utilities for clmd.
//!
//! This module provides XML parsing and generation utilities, inspired by
//! Pandoc's XML module. It includes functions for escaping XML content,
//! generating XML elements, and parsing XML documents.
//!
//! # Example
//!
//! ```
//! use clmd::xml::{escape_xml, XmlBuilder};
//!
//! // Escape XML special characters
//! let escaped = escape_xml("<tag>");
//! assert_eq!(escaped, "&lt;tag&gt;");
//!
//! // Build XML documents
//! let mut builder = XmlBuilder::new();
//! builder.start_element("document");
//! builder.text_element("title", "Hello World");
//! builder.end_element();
//! let xml = builder.build();
//! ```

use std::fmt;

/// Escape special XML characters.
///
/// Converts the following characters to their XML entities:
/// - `&` → `&amp;`
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `"` → `&quot;`
/// - `'` → `&apos;`
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// The XML-escaped string
///
/// # Example
///
/// ```ignore
/// use clmd::xml::escape_xml;
///
/// assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
/// assert_eq!(escape_xml("&"), "&amp;");
/// assert_eq!(escape_xml("'test'"), "&apos;test&apos;");
/// ```ignore
pub fn escape_xml(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }
    result
}

/// Unescape XML entities.
///
/// Converts XML entities back to their original characters:
/// - `&amp;` → `&`
/// - `&lt;` → `<`
/// - `&gt;` → `>`
/// - `&quot;` → `"`
/// - `&apos;` → `'`
///
/// # Arguments
///
/// * `s` - The string to unescape
///
/// # Returns
///
/// The unescaped string
///
/// # Example
///
/// ```ignore
/// use clmd::xml::unescape_xml;
///
/// assert_eq!(unescape_xml("&lt;tag&gt;"), "<tag>");
/// assert_eq!(unescape_xml("&amp;"), "&");
/// ```ignore
pub fn unescape_xml(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Check if a string needs XML escaping.
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// `true` if the string contains characters that need escaping
///
/// # Example
///
/// ```ignore
/// use clmd::xml::needs_escape;
///
/// assert!(needs_escape("<tag>"));
/// assert!(!needs_escape("hello"));
/// ```ignore
pub fn needs_escape(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '&' | '<' | '>' | '"' | '\''))
}

/// XML attribute value escape.
///
/// Similar to `escape_xml` but also escapes newline and tab characters
/// which are significant in attribute values.
///
/// # Arguments
///
/// * `s` - The attribute value to escape
///
/// # Returns
///
/// The escaped attribute value
///
/// # Example
///
/// ```ignore
/// use clmd::xml::escape_xml_attr;
///
/// assert_eq!(escape_xml_attr("line1\nline2"), "line1&#10;line2");
/// assert_eq!(escape_xml_attr("tab\there"), "tab&#9;here");
/// ```ignore
pub fn escape_xml_attr(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            '\n' => result.push_str("&#10;"),
            '\r' => result.push_str("&#13;"),
            '\t' => result.push_str("&#9;"),
            _ => result.push(c),
        }
    }
    result
}

/// XML element builder for constructing XML documents.
///
/// This builder provides a convenient way to construct XML documents
/// programmatically.
///
/// # Example
///
/// ```ignore
/// use clmd::xml::XmlBuilder;
///
/// let mut builder = XmlBuilder::new();
/// builder.declaration("1.0", "UTF-8");
/// builder.start_element_with_attrs("document", &[("version", "1.0")]);
/// builder.text_element("title", "Hello World");
/// builder.start_element("body");
/// builder.text("This is the content.");
/// builder.end_element();
/// builder.end_element();
///
/// let xml = builder.build();
/// assert!(xml.contains("<?xml"));
/// assert!(xml.contains("<document"));
/// assert!(xml.contains("<title>Hello World</title>"));
/// ```ignore
#[derive(Debug)]
pub struct XmlBuilder {
    output: String,
    indent_level: usize,
    indent_size: usize,
    pretty_print: bool,
    element_stack: Vec<String>,
}

impl XmlBuilder {
    /// Create a new XML builder.
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_size: 2,
            pretty_print: true,
            element_stack: Vec::new(),
        }
    }

    /// Create a new XML builder with custom settings.
    pub fn with_options(pretty_print: bool, indent_size: usize) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_size,
            pretty_print,
            element_stack: Vec::new(),
        }
    }

    /// Add XML declaration.
    pub fn declaration(&mut self, version: &str, encoding: &str) -> &mut Self {
        self.output.push_str(&format!(
            "<?xml version=\"{}\" encoding=\"{}\"?>\n",
            version, encoding
        ));
        self
    }

    /// Add DOCTYPE declaration.
    pub fn doctype(&mut self, root_element: &str, system_id: &str) -> &mut Self {
        self.output.push_str(&format!(
            "<!DOCTYPE {} SYSTEM \"{}\">\n",
            root_element, system_id
        ));
        self
    }

    /// Start an XML element.
    pub fn start_element(&mut self, name: &str) -> &mut Self {
        self.write_indent();
        self.output.push('<');
        self.output.push_str(name);
        self.output.push('>');
        if self.pretty_print {
            self.output.push('\n');
        }
        self.element_stack.push(name.to_string());
        self.indent_level += 1;
        self
    }

    /// Start an XML element with attributes.
    pub fn start_element_with_attrs(
        &mut self,
        name: &str,
        attrs: &[(&str, &str)],
    ) -> &mut Self {
        self.write_indent();
        self.output.push('<');
        self.output.push_str(name);
        for (key, value) in attrs {
            self.output.push(' ');
            self.output.push_str(key);
            self.output.push_str("=\"");
            self.output.push_str(&escape_xml_attr(value));
            self.output.push('"');
        }
        self.output.push('>');
        if self.pretty_print {
            self.output.push('\n');
        }
        self.element_stack.push(name.to_string());
        self.indent_level += 1;
        self
    }

    /// End the current element.
    pub fn end_element(&mut self) -> &mut Self {
        if let Some(name) = self.element_stack.pop() {
            self.indent_level -= 1;
            self.write_indent();
            self.output.push_str("</");
            self.output.push_str(&name);
            self.output.push('>');
            if self.pretty_print {
                self.output.push('\n');
            }
        }
        self
    }

    /// Add a text element (element with text content).
    pub fn text_element(&mut self, name: &str, text: &str) -> &mut Self {
        self.write_indent();
        self.output.push('<');
        self.output.push_str(name);
        self.output.push('>');
        self.output.push_str(&escape_xml(text));
        self.output.push_str("</");
        self.output.push_str(name);
        self.output.push('>');
        if self.pretty_print {
            self.output.push('\n');
        }
        self
    }

    /// Add a text element with attributes.
    pub fn text_element_with_attrs(
        &mut self,
        name: &str,
        attrs: &[(&str, &str)],
        text: &str,
    ) -> &mut Self {
        self.write_indent();
        self.output.push('<');
        self.output.push_str(name);
        for (key, value) in attrs {
            self.output.push(' ');
            self.output.push_str(key);
            self.output.push_str("=\"");
            self.output.push_str(&escape_xml_attr(value));
            self.output.push('"');
        }
        self.output.push('>');
        self.output.push_str(&escape_xml(text));
        self.output.push_str("</");
        self.output.push_str(name);
        self.output.push('>');
        if self.pretty_print {
            self.output.push('\n');
        }
        self
    }

    /// Add a self-closing element.
    pub fn empty_element(&mut self, name: &str) -> &mut Self {
        self.write_indent();
        self.output.push('<');
        self.output.push_str(name);
        self.output_str(" />");
        if self.pretty_print {
            self.output.push('\n');
        }
        self
    }

    /// Add a self-closing element with attributes.
    pub fn empty_element_with_attrs(
        &mut self,
        name: &str,
        attrs: &[(&str, &str)],
    ) -> &mut Self {
        self.write_indent();
        self.output.push('<');
        self.output.push_str(name);
        for (key, value) in attrs {
            self.output.push(' ');
            self.output.push_str(key);
            self.output.push_str("=\"");
            self.output.push_str(&escape_xml_attr(value));
            self.output.push('"');
        }
        self.output_str(" />");
        if self.pretty_print {
            self.output.push('\n');
        }
        self
    }

    /// Add raw text content.
    pub fn text(&mut self, text: &str) -> &mut Self {
        self.output.push_str(&escape_xml(text));
        self
    }

    /// Add raw XML content (without escaping).
    pub fn raw(&mut self, xml: &str) -> &mut Self {
        self.output.push_str(xml);
        self
    }

    /// Add a comment.
    pub fn comment(&mut self, text: &str) -> &mut Self {
        self.write_indent();
        self.output.push_str("<!-- ");
        self.output.push_str(text);
        self.output_str(" -->");
        if self.pretty_print {
            self.output.push('\n');
        }
        self
    }

    /// Add a CDATA section.
    pub fn cdata(&mut self, text: &str) -> &mut Self {
        self.write_indent();
        self.output.push_str("<![CDATA[");
        self.output.push_str(text);
        self.output_str("]]>");
        if self.pretty_print {
            self.output.push('\n');
        }
        self
    }

    /// Get the built XML string.
    pub fn build(self) -> String {
        self.output
    }

    /// Get a reference to the current output.
    pub fn as_str(&self) -> &str {
        &self.output
    }

    /// Check if the builder is empty.
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }

    /// Get the length of the output.
    pub fn len(&self) -> usize {
        self.output.len()
    }

    fn write_indent(&mut self) {
        if self.pretty_print {
            for _ in 0..self.indent_level * self.indent_size {
                self.output.push(' ');
            }
        }
    }

    fn output_str(&mut self, s: &str) {
        self.output.push_str(s);
    }
}

impl Default for XmlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// XML element representation for parsing.
#[derive(Debug, Clone, PartialEq)]
pub struct XmlElement {
    /// Element name.
    pub name: String,
    /// Element attributes.
    pub attrs: Vec<(String, String)>,
    /// Text content.
    pub text: Option<String>,
    /// Child elements.
    pub children: Vec<XmlElement>,
}

impl XmlElement {
    /// Create a new XML element.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            attrs: Vec::new(),
            text: None,
            children: Vec::new(),
        }
    }

    /// Add an attribute.
    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.push((name.into(), value.into()));
        self
    }

    /// Set text content.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Add a child element.
    pub fn child(mut self, child: XmlElement) -> Self {
        self.children.push(child);
        self
    }

    /// Get an attribute value by name.
    pub fn get_attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// Find a child element by name.
    pub fn find_child(&self, name: &str) -> Option<&XmlElement> {
        self.children.iter().find(|c| c.name == name)
    }

    /// Find all child elements by name.
    pub fn find_children(&self, name: &str) -> Vec<&XmlElement> {
        self.children.iter().filter(|c| c.name == name).collect()
    }
}

impl fmt::Display for XmlElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = XmlBuilder::new();
        self.write_to_builder(&mut builder);
        write!(f, "{}", builder.build())
    }
}

impl XmlElement {
    fn write_to_builder(&self, builder: &mut XmlBuilder) {
        let attrs: Vec<(&str, &str)> = self
            .attrs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        if self.children.is_empty() && self.text.is_none() {
            builder.empty_element_with_attrs(&self.name, &attrs);
        } else if self.children.is_empty() {
            let text = self.text.as_deref().unwrap_or("");
            builder.text_element_with_attrs(&self.name, &attrs, text);
        } else {
            builder.start_element_with_attrs(&self.name, &attrs);
            if let Some(text) = &self.text {
                builder.text(text);
            }
            for child in &self.children {
                child.write_to_builder(builder);
            }
            builder.end_element();
        }
    }
}

/// Convert a boolean to XML boolean string ("true" or "false").
pub fn bool_to_xml(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

/// Parse an XML boolean string.
pub fn xml_to_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("&"), "&amp;");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_xml("'single'"), "&apos;single&apos;");
        assert_eq!(escape_xml("a > b"), "a &gt; b");
    }

    #[test]
    fn test_unescape_xml() {
        assert_eq!(unescape_xml("&lt;tag&gt;"), "<tag>");
        assert_eq!(unescape_xml("&amp;"), "&");
        assert_eq!(unescape_xml("&quot;quoted&quot;"), "\"quoted\"");
        assert_eq!(unescape_xml("&apos;single&apos;"), "'single'");
    }

    #[test]
    fn test_needs_escape() {
        assert!(needs_escape("<tag>"));
        assert!(needs_escape("&"));
        assert!(needs_escape("\""));
        assert!(needs_escape("'"));
        assert!(!needs_escape("hello"));
        assert!(!needs_escape("123"));
    }

    #[test]
    fn test_escape_xml_attr() {
        assert_eq!(escape_xml_attr("line1\nline2"), "line1&#10;line2");
        assert_eq!(escape_xml_attr("tab\there"), "tab&#9;here");
        assert_eq!(escape_xml_attr("a\rb"), "a&#13;b");
    }

    #[test]
    fn test_xml_builder_basic() {
        let mut builder = XmlBuilder::new();
        builder.declaration("1.0", "UTF-8");
        builder.start_element("document");
        builder.text_element("title", "Test");
        builder.end_element();

        let xml = builder.build();
        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<document>"));
        assert!(xml.contains("<title>Test</title>"));
        assert!(xml.contains("</document>"));
    }

    #[test]
    fn test_xml_builder_with_attrs() {
        let mut builder = XmlBuilder::new();
        builder
            .start_element_with_attrs("element", &[("id", "test"), ("class", "main")]);
        builder.end_element();

        let xml = builder.build();
        assert!(xml.contains("<element id=\"test\" class=\"main\">"));
    }

    #[test]
    fn test_xml_builder_empty_element() {
        let mut builder = XmlBuilder::new();
        builder.empty_element("br");

        let xml = builder.build();
        assert!(xml.contains("<br />"));
    }

    #[test]
    fn test_xml_builder_comment() {
        let mut builder = XmlBuilder::new();
        builder.comment("This is a comment");

        let xml = builder.build();
        assert!(xml.contains("<!-- This is a comment -->"));
    }

    #[test]
    fn test_xml_builder_cdata() {
        let mut builder = XmlBuilder::new();
        builder.cdata("<raw>content</raw>");

        let xml = builder.build();
        assert!(xml.contains("<![CDATA[<raw>content</raw>]]>"));
    }

    #[test]
    fn test_xml_element() {
        let element = XmlElement::new("document")
            .attr("version", "1.0")
            .text("Hello World");

        assert_eq!(element.name, "document");
        assert_eq!(element.get_attr("version"), Some("1.0"));
        assert_eq!(element.text, Some("Hello World".to_string()));
    }

    #[test]
    fn test_xml_element_with_children() {
        let child = XmlElement::new("child").text("Child content");
        let parent = XmlElement::new("parent").child(child);

        assert_eq!(parent.children.len(), 1);
        assert_eq!(
            parent.find_child("child").unwrap().text,
            Some("Child content".to_string())
        );
    }

    #[test]
    fn test_xml_element_display() {
        let element = XmlElement::new("test").attr("id", "1").text("Hello");
        let xml = element.to_string();
        assert!(xml.contains("<test"));
        assert!(xml.contains("id=\"1\""));
        assert!(xml.contains("Hello"));
        assert!(xml.contains("</test>"));
    }

    #[test]
    fn test_bool_to_xml() {
        assert_eq!(bool_to_xml(true), "true");
        assert_eq!(bool_to_xml(false), "false");
    }

    #[test]
    fn test_xml_to_bool() {
        assert_eq!(xml_to_bool("true"), Some(true));
        assert_eq!(xml_to_bool("TRUE"), Some(true));
        assert_eq!(xml_to_bool("1"), Some(true));
        assert_eq!(xml_to_bool("yes"), Some(true));
        assert_eq!(xml_to_bool("false"), Some(false));
        assert_eq!(xml_to_bool("FALSE"), Some(false));
        assert_eq!(xml_to_bool("0"), Some(false));
        assert_eq!(xml_to_bool("no"), Some(false));
        assert_eq!(xml_to_bool("maybe"), None);
    }

    #[test]
    fn test_xml_builder_doctype() {
        let mut builder = XmlBuilder::new();
        builder.doctype("document", "document.dtd");

        let xml = builder.build();
        assert!(xml.contains("<!DOCTYPE document SYSTEM \"document.dtd\">"));
    }

    #[test]
    fn test_xml_builder_no_pretty_print() {
        let mut builder = XmlBuilder::with_options(false, 2);
        builder.start_element("root");
        builder.text_element("item", "value");
        builder.end_element();

        let xml = builder.build();
        // Should not contain newlines when pretty_print is false
        assert!(!xml.contains("\n"));
        assert!(xml.contains("<root>"));
        assert!(xml.contains("<item>value</item>"));
        assert!(xml.contains("</root>"));
    }

    #[test]
    fn test_xml_builder_nested() {
        let mut builder = XmlBuilder::new();
        builder.start_element("root");
        builder.start_element("level1");
        builder.start_element("level2");
        builder.text("Deep content");
        builder.end_element();
        builder.end_element();
        builder.end_element();

        let xml = builder.build();
        assert!(xml.contains("<root>"));
        assert!(xml.contains("<level1>"));
        assert!(xml.contains("<level2>"));
        assert!(xml.contains("Deep content"));
        assert!(xml.contains("</level2>"));
        assert!(xml.contains("</level1>"));
        assert!(xml.contains("</root>"));
    }
}
