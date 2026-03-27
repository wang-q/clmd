//! Abbreviation extension for Markdown
//!
//! This module implements abbreviation/definition list parsing.
//! Allows defining abbreviations that will be wrapped in <abbr> tags.
//!
//! Syntax:
//! ```markdown
//! *[HTML]: Hyper Text Markup Language
//!
//! This is a paragraph with HTML abbreviation.
//! ```

use std::collections::HashMap;

/// An abbreviation definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Abbreviation {
    /// The abbreviation term
    pub term: String,
    /// The expanded definition
    pub definition: String,
}

/// Check if a line is an abbreviation definition
/// Format: *[TERM]: Definition
pub fn is_abbreviation_def(line: &str) -> bool {
    let trimmed = line.trim_start();

    if trimmed.starts_with("*[") {
        if let Some(close_bracket) = trimmed.find("]:") {
            let term = &trimmed[2..close_bracket];
            return !term.is_empty();
        }
    }

    false
}

/// Parse an abbreviation definition line
/// Returns (term, definition) if successful
pub fn parse_abbreviation_def(line: &str) -> Option<(String, String)> {
    if !is_abbreviation_def(line) {
        return None;
    }

    let trimmed = line.trim_start();
    let start = trimmed.find("*[")?;
    let close_bracket = trimmed.find("]:")?;

    let term = trimmed[start + 2..close_bracket].trim().to_string();
    let definition = trimmed[close_bracket + 2..].trim().to_string();

    if term.is_empty() {
        return None;
    }

    Some((term, definition))
}

/// Find all occurrences of abbreviations in text and return positions
/// Returns vector of (start, end, abbreviation) for each match
pub fn find_abbreviations(
    text: &str,
    abbreviations: &HashMap<String, Abbreviation>,
) -> Vec<(usize, usize, String)> {
    let mut matches = Vec::new();

    for (term, abbr) in abbreviations {
        let mut start = 0;
        while let Some(pos) = text[start..].find(term) {
            let match_start = start + pos;
            let match_end = match_start + term.len();

            // Check word boundaries
            let before = if match_start == 0 {
                true
            } else {
                let before_char = text.chars().nth(match_start - 1).unwrap_or(' ');
                !before_char.is_alphanumeric()
            };

            let after = if match_end >= text.len() {
                true
            } else {
                let after_char = text.chars().nth(match_end).unwrap_or(' ');
                !after_char.is_alphanumeric()
            };

            if before && after {
                matches.push((match_start, match_end, abbr.definition.clone()));
            }

            start = match_end;
        }
    }

    // Sort by position
    matches.sort_by_key(|(start, _, _)| *start);
    matches
}

/// Replace abbreviations in text with HTML <abbr> tags
pub fn replace_abbreviations(
    text: &str,
    abbreviations: &HashMap<String, Abbreviation>,
) -> String {
    let matches = find_abbreviations(text, abbreviations);
    if matches.is_empty() {
        return text.to_string();
    }

    let mut result = String::new();
    let mut last_end = 0;

    for (start, end, definition) in matches {
        // Add text before this match
        result.push_str(&text[last_end..start]);

        // Add the abbreviation with <abbr> tag
        let term = &text[start..end];
        result.push_str(&format!(
            "<abbr title=\"{}\">{}</abbr>",
            crate::html_utils::escape_html(&definition),
            term
        ));

        last_end = end;
    }

    // Add remaining text
    result.push_str(&text[last_end..]);

    result
}

/// Abbreviation registry for managing abbreviations
#[derive(Debug, Default)]
pub struct AbbreviationRegistry {
    /// Map from term to abbreviation
    pub abbreviations: HashMap<String, Abbreviation>,
}

impl AbbreviationRegistry {
    /// Create a new abbreviation registry
    pub fn new() -> Self {
        Self {
            abbreviations: HashMap::new(),
        }
    }

    /// Register an abbreviation
    pub fn register(&mut self, term: &str, definition: &str) {
        self.abbreviations.insert(
            term.to_string(),
            Abbreviation {
                term: term.to_string(),
                definition: definition.to_string(),
            },
        );
    }

    /// Get an abbreviation by term
    pub fn get(&self, term: &str) -> Option<&Abbreviation> {
        self.abbreviations.get(term)
    }

    /// Check if a term is registered
    pub fn contains(&self, term: &str) -> bool {
        self.abbreviations.contains_key(term)
    }

    /// Get all abbreviations
    pub fn all(&self) -> &HashMap<String, Abbreviation> {
        &self.abbreviations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_abbreviation_def() {
        assert!(is_abbreviation_def("*[HTML]: Hyper Text Markup Language"));
        assert!(is_abbreviation_def(
            "  *[API]: Application Programming Interface"
        ));
        assert!(!is_abbreviation_def("Not an abbreviation"));
        assert!(!is_abbreviation_def("*[]: Empty term"));
        assert!(!is_abbreviation_def("*[NO_COLON] Missing colon"));
    }

    #[test]
    fn test_parse_abbreviation_def() {
        let result = parse_abbreviation_def("*[HTML]: Hyper Text Markup Language");
        assert_eq!(
            result,
            Some(("HTML".to_string(), "Hyper Text Markup Language".to_string()))
        );

        let result = parse_abbreviation_def("*[API]: Application Programming Interface");
        assert_eq!(
            result,
            Some((
                "API".to_string(),
                "Application Programming Interface".to_string()
            ))
        );
    }

    #[test]
    fn test_find_abbreviations() {
        let mut registry = AbbreviationRegistry::new();
        registry.register("HTML", "Hyper Text Markup Language");
        registry.register("API", "Application Programming Interface");

        let text = "This is HTML and API in a sentence.";
        let matches = find_abbreviations(text, registry.all());

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].0, 8); // HTML starts at position 8
        assert_eq!(matches[1].0, 17); // API starts at position 17
    }

    #[test]
    fn test_replace_abbreviations() {
        let mut registry = AbbreviationRegistry::new();
        registry.register("HTML", "Hyper Text Markup Language");

        let text = "This is HTML in a sentence.";
        let result = replace_abbreviations(text, registry.all());

        assert!(result.contains("<abbr"));
        assert!(result.contains("title=\"Hyper Text Markup Language\""));
        assert!(result.contains(">HTML<"));
    }

    #[test]
    fn test_abbreviation_registry() {
        let mut registry = AbbreviationRegistry::new();

        registry.register("HTML", "Hyper Text Markup Language");
        assert!(registry.contains("HTML"));

        let abbr = registry.get("HTML").unwrap();
        assert_eq!(abbr.term, "HTML");
        assert_eq!(abbr.definition, "Hyper Text Markup Language");
    }

    #[test]
    fn test_no_partial_matches() {
        let mut registry = AbbreviationRegistry::new();
        registry.register("HTML", "Hyper Text Markup Language");

        // "XHTML" should not match "HTML"
        let text = "This is XHTML, not just HTML.";
        let matches = find_abbreviations(text, registry.all());

        assert_eq!(matches.len(), 1); // Only "HTML" at the end should match
        assert_eq!(matches[0].0, 24);
    }
}
