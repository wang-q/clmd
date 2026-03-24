//! YAML Front Matter extension for Markdown
//!
//! This module implements YAML front matter parsing for Markdown documents.
//! Front matter is metadata at the beginning of a document, delimited by `---`.
//!
//! Syntax:
//! ```markdown
//! ---
//! title: My Document
//! author: John Doe
//! date: 2024-01-01
//! tags:
//!   - markdown
//!   - rust
//! ---
//!
//! # Document Content
//!
//! The rest of the document...
//! ```

use std::collections::HashMap;

/// YAML Front Matter data
#[derive(Debug, Clone, Default, PartialEq)]
pub struct YamlFrontMatter {
    /// Raw YAML content (without delimiters)
    pub raw: String,
    /// Parsed key-value pairs (flattened)
    pub data: HashMap<String, String>,
}

impl YamlFrontMatter {
    /// Create empty front matter
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if front matter is empty
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty() && self.data.is_empty()
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Parse YAML front matter from raw content
    pub fn parse(raw: &str) -> Self {
        let mut data = HashMap::new();

        // Simple key-value parsing (supports only basic format)
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for list items (skip for now)
            if trimmed.starts_with('-') {
                continue;
            }

            // Parse key: value
            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim().to_string();
                let value = trimmed[colon_pos + 1..].trim().to_string();

                // Remove quotes if present
                let value = if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    value[1..value.len() - 1].to_string()
                } else {
                    value
                };

                if !key.is_empty() {
                    data.insert(key, value);
                }
            }
        }

        Self {
            raw: raw.to_string(),
            data,
        }
    }
}

/// Check if content starts with YAML front matter
pub fn has_front_matter(content: &str) -> bool {
    let trimmed = content.trim_start();
    trimmed.starts_with("---\n") || trimmed.starts_with("---\r\n")
}

/// Extract YAML front matter and body from content
/// Returns (front_matter, body)
pub fn extract_front_matter(content: &str) -> (Option<YamlFrontMatter>, &str) {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return (None, content);
    }

    // Find the end of front matter
    let after_first_delimiter = &trimmed[3..];
    let start_of_content = if let Some(pos) = after_first_delimiter.find("\n---") {
        pos + 4 // Include the newline before ---
    } else if let Some(pos) = after_first_delimiter.find("\r\n---") {
        pos + 5 // Include the \r\n before ---
    } else {
        return (None, content);
    };

    let front_matter_raw = &after_first_delimiter[..start_of_content - 3].trim();
    let body_start = content.len() - trimmed.len() + 3 + start_of_content;

    // Skip the ending --- and following newlines
    let body = &content[body_start..];
    let body = body.trim_start_matches("---").trim_start_matches('\n').trim_start_matches("\r\n");

    let front_matter = YamlFrontMatter::parse(front_matter_raw);

    (Some(front_matter), body)
}

/// Strip front matter from content and return just the body
pub fn strip_front_matter(content: &str) -> &str {
    let (_, body) = extract_front_matter(content);
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_front_matter() {
        assert!(has_front_matter("---\ntitle: Test\n---\nContent"));
        assert!(has_front_matter("  ---\ntitle: Test\n---\nContent"));
        assert!(!has_front_matter("No front matter here"));
        assert!(!has_front_matter("--- not front matter"));
    }

    #[test]
    fn test_extract_front_matter() {
        let content = "---\ntitle: My Document\nauthor: John\n---\n# Heading\n\nBody text";
        let (front_matter, body) = extract_front_matter(content);

        assert!(front_matter.is_some());
        let fm = front_matter.unwrap();
        assert_eq!(fm.get("title"), Some(&"My Document".to_string()));
        assert_eq!(fm.get("author"), Some(&"John".to_string()));
        assert_eq!(body, "# Heading\n\nBody text");
    }

    #[test]
    fn test_extract_front_matter_crlf() {
        let content = "---\r\ntitle: Test\r\n---\r\nContent";
        let (front_matter, body) = extract_front_matter(content);

        assert!(front_matter.is_some());
        assert_eq!(body, "Content");
    }

    #[test]
    fn test_no_front_matter() {
        let content = "# Just a heading\n\nSome text";
        let (front_matter, body) = extract_front_matter(content);

        assert!(front_matter.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_strip_front_matter() {
        let content = "---\ntitle: Test\n---\n# Heading";
        let body = strip_front_matter(content);
        assert_eq!(body, "# Heading");
    }

    #[test]
    fn test_yaml_parse() {
        let yaml = "title: My Doc\nauthor: John Doe\ncount: 42";
        let fm = YamlFrontMatter::parse(yaml);

        assert_eq!(fm.get("title"), Some(&"My Doc".to_string()));
        assert_eq!(fm.get("author"), Some(&"John Doe".to_string()));
        assert_eq!(fm.get("count"), Some(&"42".to_string()));
    }

    #[test]
    fn test_yaml_parse_quoted_values() {
        let yaml = "title: \"Quoted Title\"\nname: 'Single Quoted'";
        let fm = YamlFrontMatter::parse(yaml);

        assert_eq!(fm.get("title"), Some(&"Quoted Title".to_string()));
        assert_eq!(fm.get("name"), Some(&"Single Quoted".to_string()));
    }

    #[test]
    fn test_yaml_empty() {
        let fm = YamlFrontMatter::new();
        assert!(fm.is_empty());
    }
}
