//! Attributes extension for Markdown
//!
//! This module implements attribute syntax for adding IDs, classes, and custom attributes
//! to Markdown elements.
//!
//! Syntax:
//! ```markdown
//! # Heading {#id .class key=value}
//!
//! [Link](url){.class #id}
//!
//! *emphasis*{.special}
//! ```

use std::collections::HashMap;

/// Element attributes (ID, classes, and custom attributes)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attributes {
    /// Element ID
    pub id: Option<String>,
    /// CSS classes
    pub classes: Vec<String>,
    /// Custom key-value attributes
    pub attrs: HashMap<String, String>,
}

impl Attributes {
    /// Create empty attributes
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if attributes are empty
    pub fn is_empty(&self) -> bool {
        self.id.is_none() && self.classes.is_empty() && self.attrs.is_empty()
    }

    /// Add an ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Add a class
    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.classes.push(class.into());
        self
    }

    /// Add a custom attribute
    pub fn with_attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.insert(key.into(), value.into());
        self
    }

    /// Render attributes to HTML string
    pub fn render_html(&self) -> String {
        let mut parts = Vec::new();

        if let Some(id) = &self.id {
            parts.push(format!("id=\"{}\"", escape_attr(id)));
        }

        if !self.classes.is_empty() {
            let class_str = self.classes.join(" ");
            parts.push(format!("class=\"{}\"", escape_attr(&class_str)));
        }

        for (key, value) in &self.attrs {
            parts.push(format!("{}=\"{}\"", key, escape_attr(value)));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!(" {}", parts.join(" "))
        }
    }

    /// Parse attributes from a string
    /// Format: {#id .class key=value key="value with spaces"}
    pub fn parse(input: &str) -> Option<(Self, usize)> {
        let trimmed = input.trim_start();
        if !trimmed.starts_with('{') {
            return None;
        }

        let mut attrs = Attributes::new();
        let mut pos = 1; // Skip opening brace
        let chars: Vec<char> = trimmed.chars().collect();

        while pos < chars.len() {
            // Skip whitespace
            while pos < chars.len() && chars[pos].is_whitespace() {
                pos += 1;
            }

            if pos >= chars.len() {
                break;
            }

            // Check for closing brace
            if chars[pos] == '}' {
                pos += 1;
                return Some((attrs, pos));
            }

            // Parse attribute
            match chars[pos] {
                '#' => {
                    // ID
                    pos += 1;
                    let start = pos;
                    while pos < chars.len() && !chars[pos].is_whitespace() && chars[pos] != '}' {
                        pos += 1;
                    }
                    if pos > start {
                        attrs.id = Some(trimmed[start..pos].to_string());
                    }
                }
                '.' => {
                    // Class
                    pos += 1;
                    let start = pos;
                    while pos < chars.len() && !chars[pos].is_whitespace() && chars[pos] != '}' {
                        pos += 1;
                    }
                    if pos > start {
                        attrs.classes.push(trimmed[start..pos].to_string());
                    }
                }
                _ => {
                    // Key=value attribute
                    let start = pos;
                    while pos < chars.len() && chars[pos] != '=' && chars[pos] != '}' && !chars[pos].is_whitespace() {
                        pos += 1;
                    }

                    if pos >= chars.len() || chars[pos] == '}' {
                        // Just a key without value
                        let key = trimmed[start..pos].trim().to_string();
                        if !key.is_empty() {
                            attrs.attrs.insert(key, String::new());
                        }
                        continue;
                    }

                    let key = trimmed[start..pos].trim().to_string();

                    // Skip '='
                    if pos < chars.len() && chars[pos] == '=' {
                        pos += 1;
                    }

                    // Parse value
                    let value = if pos < chars.len() && chars[pos] == '"' {
                        // Quoted value
                        pos += 1;
                        let value_start = pos;
                        while pos < chars.len() && chars[pos] != '"' {
                            pos += 1;
                        }
                        let value = trimmed[value_start..pos].to_string();
                        if pos < chars.len() && chars[pos] == '"' {
                            pos += 1;
                        }
                        value
                    } else {
                        // Unquoted value
                        let value_start = pos;
                        while pos < chars.len() && !chars[pos].is_whitespace() && chars[pos] != '}' {
                            pos += 1;
                        }
                        trimmed[value_start..pos].to_string()
                    };

                    if !key.is_empty() {
                        attrs.attrs.insert(key, value);
                    }
                }
            }
        }

        None
    }
}

/// Escape attribute value for HTML
fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Check if text ends with attributes
pub fn has_trailing_attributes(text: &str) -> bool {
    text.trim_end().ends_with('}')
}

/// Extract attributes from the end of text
/// Returns (text_without_attrs, attributes)
pub fn extract_attributes(text: &str) -> (String, Option<Attributes>) {
    let trimmed = text.trim_end();

    // Find the last opening brace
    if let Some(open_pos) = trimmed.rfind('{') {
        let potential_attrs = &trimmed[open_pos..];
        if let Some((attrs, consumed)) = Attributes::parse(potential_attrs) {
            if consumed == potential_attrs.len() {
                let text_without = trimmed[..open_pos].trim_end().to_string();
                return (text_without, Some(attrs));
            }
        }
    }

    (text.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attributes_parse_id() {
        let (attrs, _) = Attributes::parse("{#my-id}").unwrap();
        assert_eq!(attrs.id, Some("my-id".to_string()));
    }

    #[test]
    fn test_attributes_parse_class() {
        let (attrs, _) = Attributes::parse("{.class1 .class2}").unwrap();
        assert_eq!(attrs.classes, vec!["class1", "class2"]);
    }

    #[test]
    fn test_attributes_parse_custom() {
        let (attrs, _) = Attributes::parse("{key=value}").unwrap();
        assert_eq!(attrs.attrs.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_attributes_parse_quoted() {
        let (attrs, _) = Attributes::parse("{title=\"hello world\"}").unwrap();
        assert_eq!(attrs.attrs.get("title"), Some(&"hello world".to_string()));
    }

    #[test]
    fn test_attributes_parse_combined() {
        let (attrs, _) = Attributes::parse("{#id .class key=value}").unwrap();
        assert_eq!(attrs.id, Some("id".to_string()));
        assert!(attrs.classes.contains(&"class".to_string()));
        assert_eq!(attrs.attrs.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_attributes_render_html() {
        let attrs = Attributes::new()
            .with_id("my-id")
            .with_class("class1")
            .with_class("class2")
            .with_attr("data-key", "value");

        let html = attrs.render_html();
        assert!(html.contains("id=\"my-id\""));
        assert!(html.contains("class=\"class1 class2\""));
        assert!(html.contains("data-key=\"value\""));
    }

    #[test]
    fn test_attributes_empty() {
        let attrs = Attributes::new();
        assert!(attrs.is_empty());
        assert_eq!(attrs.render_html(), "");
    }

    #[test]
    fn test_extract_attributes() {
        let (text, attrs) = extract_attributes("Heading {#id .class}");
        assert_eq!(text, "Heading");
        assert!(attrs.is_some());
        let attrs = attrs.unwrap();
        assert_eq!(attrs.id, Some("id".to_string()));
    }

    #[test]
    fn test_extract_no_attributes() {
        let (text, attrs) = extract_attributes("Just text");
        assert_eq!(text, "Just text");
        assert!(attrs.is_none());
    }

    #[test]
    fn test_has_trailing_attributes() {
        assert!(has_trailing_attributes("text {attr}"));
        assert!(has_trailing_attributes("text{attr}"));
        assert!(!has_trailing_attributes("text"));
    }
}
