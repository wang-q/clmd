//! CSS parsing utilities for clmd.
//!
//! This module provides basic CSS parsing capabilities, inspired by Pandoc's
//! CSS handling. It supports parsing inline styles and extracting CSS properties.
//!
//! # Example
//!
//! ```ignore
//! use clmd::formats::css::{StyleDeclaration, parse_inline_style};
//!
//! let style = parse_inline_style("color: red; font-size: 14px").unwrap();
//! assert_eq!(style.get("color"), Some("red"));
//! assert_eq!(style.get("font-size"), Some("14px"));
//! ```

use std::collections::HashMap;

/// A CSS style declaration (set of properties).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleDeclaration {
    properties: HashMap<String, String>,
}

impl StyleDeclaration {
    /// Create a new empty style declaration.
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    /// Create a style declaration from a CSS string.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::formats::css::StyleDeclaration;
    ///
    /// let style = StyleDeclaration::from_css("color: red; font-size: 14px");
    /// assert_eq!(style.get("color"), Some("red"));
    /// ```
    pub fn from_css(css: &str) -> Self {
        let mut declaration = Self::new();
        declaration.parse_and_add(css);
        declaration
    }

    /// Parse CSS and add properties to this declaration.
    fn parse_and_add(&mut self, css: &str) {
        for declaration in css.split(';') {
            let declaration = declaration.trim();
            if declaration.is_empty() {
                continue;
            }

            if let Some(colon_pos) = declaration.find(':') {
                let property = declaration[..colon_pos].trim().to_string();
                let value = declaration[colon_pos + 1..].trim().to_string();

                if !property.is_empty() {
                    self.properties.insert(property, value);
                }
            }
        }
    }

    /// Get a property value.
    pub fn get(&self, property: &str) -> Option<&str> {
        self.properties.get(property).map(|s| s.as_str())
    }

    /// Set a property value.
    pub fn set(&mut self, property: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(property.into(), value.into());
    }

    /// Remove a property.
    pub fn remove(&mut self, property: &str) -> Option<String> {
        self.properties.remove(property)
    }

    /// Check if a property exists.
    pub fn has(&self, property: &str) -> bool {
        self.properties.contains_key(property)
    }

    /// Get all properties.
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    /// Convert to a CSS string.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::formats::css::StyleDeclaration;
    ///
    /// let mut style = StyleDeclaration::new();
    /// style.set("color", "red");
    /// style.set("font-size", "14px");
    ///
    /// let css = style.to_css();
    /// assert!(css.contains("color: red"));
    /// assert!(css.contains("font-size: 14px"));
    /// ```
    pub fn to_css(&self) -> String {
        let mut parts: Vec<String> = self
            .properties
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect();
        parts.sort(); // For deterministic output
        parts.join("; ")
    }

    /// Merge another style declaration into this one.
    ///
    /// Properties from `other` will overwrite existing properties.
    pub fn merge(&mut self, other: &StyleDeclaration) {
        self.properties.extend(other.properties.clone());
    }

    /// Get the number of properties.
    pub fn len(&self) -> usize {
        self.properties.len()
    }

    /// Check if there are no properties.
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    /// Clear all properties.
    pub fn clear(&mut self) {
        self.properties.clear();
    }
}

impl std::fmt::Display for StyleDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_css())
    }
}

/// Parse an inline CSS style string.
///
/// This is a convenience function equivalent to `StyleDeclaration::from_css`.
///
/// # Example
///
/// ```ignore
/// use clmd::formats::css::parse_inline_style;
///
/// let style = parse_inline_style("color: red; font-size: 14px").unwrap();
/// assert_eq!(style.get("color"), Some("red"));
/// ```ignore
pub fn parse_inline_style(css: &str) -> Option<StyleDeclaration> {
    let trimmed = css.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(StyleDeclaration::from_css(trimmed))
}

/// CSS color utilities.
pub mod colors {
    /// Check if a string is a valid CSS color value.
    ///
    /// This is a basic check that handles:
    /// - Named colors (red, blue, etc.)
    /// - Hex colors (#fff, #ffffff)
    /// - RGB/RGBA colors
    /// - HSL/HSLA colors
    pub fn is_valid_color(value: &str) -> bool {
        let value = value.trim().to_lowercase();

        // Named colors (common subset)
        let named_colors = [
            "black",
            "white",
            "red",
            "green",
            "blue",
            "yellow",
            "cyan",
            "magenta",
            "gray",
            "grey",
            "orange",
            "purple",
            "pink",
            "brown",
            "silver",
            "gold",
            "transparent",
            "inherit",
            "initial",
            "unset",
        ];
        if named_colors.contains(&value.as_str()) {
            return true;
        }

        // Hex colors
        if let Some(hex) = value.strip_prefix('#') {
            return hex.len() == 3 || hex.len() == 4 || hex.len() == 6 || hex.len() == 8;
        }

        // RGB/RGBA
        if value.starts_with("rgb(") || value.starts_with("rgba(") {
            return value.ends_with(')');
        }

        // HSL/HSLA
        if value.starts_with("hsl(") || value.starts_with("hsla(") {
            return value.ends_with(')');
        }

        false
    }

    /// Normalize a CSS color to lowercase.
    pub fn normalize_color(value: &str) -> String {
        value.trim().to_lowercase()
    }
}

/// CSS unit utilities.
pub mod units {
    /// Common CSS length units.
    pub const LENGTH_UNITS: &[&str] = &[
        "px", "em", "rem", "%", "pt", "pc", "in", "cm", "mm", "ex", "ch", "vw", "vh",
        "vmin", "vmax",
    ];

    /// Check if a value has a CSS unit.
    pub fn has_unit(value: &str) -> bool {
        let value = value.trim().to_lowercase();
        LENGTH_UNITS.iter().any(|unit| value.ends_with(unit))
    }

    /// Extract the numeric part from a CSS length value.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::io::format::css::units::extract_number;
    ///
    /// assert_eq!(extract_number("14px"), Some(14.0));
    /// assert_eq!(extract_number("1.5em"), Some(1.5));
    /// assert_eq!(extract_number("auto"), None);
    /// ```
    pub fn extract_number(value: &str) -> Option<f64> {
        let value = value.trim();
        let numeric_part: String = value
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect();
        numeric_part.parse().ok()
    }

    /// Extract the unit from a CSS length value.
    pub fn extract_unit(value: &str) -> Option<&str> {
        let value = value.trim().to_lowercase();
        LENGTH_UNITS
            .iter()
            .find(|&&unit| value.ends_with(unit))
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inline_style() {
        let style = parse_inline_style("color: red; font-size: 14px").unwrap();
        assert_eq!(style.get("color"), Some("red"));
        assert_eq!(style.get("font-size"), Some("14px"));
    }

    #[test]
    fn test_parse_empty_style() {
        assert!(parse_inline_style("").is_none());
        assert!(parse_inline_style("   ").is_none());
    }

    #[test]
    fn test_style_declaration_set_get() {
        let mut style = StyleDeclaration::new();
        style.set("color", "blue");
        assert_eq!(style.get("color"), Some("blue"));
    }

    #[test]
    fn test_style_declaration_remove() {
        let mut style = StyleDeclaration::new();
        style.set("color", "red");
        assert!(style.has("color"));

        style.remove("color");
        assert!(!style.has("color"));
    }

    #[test]
    fn test_style_declaration_to_css() {
        let mut style = StyleDeclaration::new();
        style.set("color", "red");
        style.set("font-size", "14px");

        let css = style.to_css();
        assert!(css.contains("color: red"));
        assert!(css.contains("font-size: 14px"));
    }

    #[test]
    fn test_style_declaration_merge() {
        let mut style1 = StyleDeclaration::new();
        style1.set("color", "red");

        let mut style2 = StyleDeclaration::new();
        style2.set("font-size", "14px");
        style2.set("color", "blue"); // Will overwrite

        style1.merge(&style2);
        assert_eq!(style1.get("color"), Some("blue"));
        assert_eq!(style1.get("font-size"), Some("14px"));
    }

    #[test]
    fn test_style_declaration_display() {
        let mut style = StyleDeclaration::new();
        style.set("color", "red");

        let css = format!("{}", style);
        assert!(css.contains("color: red"));
    }

    #[test]
    fn test_colors_is_valid_color() {
        assert!(colors::is_valid_color("red"));
        assert!(colors::is_valid_color("#fff"));
        assert!(colors::is_valid_color("#ffffff"));
        assert!(colors::is_valid_color("rgb(255, 0, 0)"));
        assert!(!colors::is_valid_color("invalid"));
    }

    #[test]
    fn test_units_has_unit() {
        assert!(units::has_unit("14px"));
        assert!(units::has_unit("1.5em"));
        assert!(!units::has_unit("auto"));
    }

    #[test]
    fn test_units_extract_number() {
        assert_eq!(units::extract_number("14px"), Some(14.0));
        assert_eq!(units::extract_number("1.5em"), Some(1.5));
        assert_eq!(units::extract_number("auto"), None);
    }

    #[test]
    fn test_units_extract_unit() {
        assert_eq!(units::extract_unit("14px"), Some("px"));
        assert_eq!(units::extract_unit("1.5em"), Some("em"));
        assert_eq!(units::extract_unit("auto"), None);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let style = StyleDeclaration::from_css("  color :  red  ;  font-size : 14px ");
        assert_eq!(style.get("color"), Some("red"));
        assert_eq!(style.get("font-size"), Some("14px"));
    }

    #[test]
    fn test_parse_empty_declarations() {
        let style = StyleDeclaration::from_css("color: red;;; font-size: 14px;;");
        assert_eq!(style.get("color"), Some("red"));
        assert_eq!(style.get("font-size"), Some("14px"));
    }
}
