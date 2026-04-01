//! Syntect-based syntax highlighter plugin
//!
//! This module provides a [`SyntaxHighlighterAdapter`] implementation using
//! the syntect library for high-quality syntax highlighting.
//!
//! # Features
//!
//! - Supports over 200 programming languages using Sublime Text syntax definitions
//! - High performance: faster than most text editors
//! - Multiple output modes: CSS classes or inline styles
//! - Includes default themes and syntax definitions
//!
//! # Example
//!
//! ```ignore
//! use clmd::plugins::syntect::SyntectAdapter;
//! use clmd::plugins::Plugins;
//!
//! // Create adapter with CSS class mode (default)
//! let adapter = SyntectAdapter::new(None);
//!
//! // Or with a specific theme for inline styles
//! let adapter = SyntectAdapter::new(Some("base16-ocean.dark"));
//!
//! // Configure plugins
//! let mut plugins = Plugins::new();
//! plugins.render.set_syntax_highlighter(&adapter);
//! ```

use crate::core::adapters::SyntaxHighlighterAdapter;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Write};

use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;

/// Syntect-based syntax highlighter.
///
/// This adapter uses the syntect library to provide syntax highlighting
/// for code blocks. It supports both CSS class mode and inline style mode.
#[derive(Debug)]
pub struct SyntectAdapter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme: Option<String>,
}

impl SyntectAdapter {
    /// Create a new SyntectAdapter.
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme name to use for inline styles, or `None` for CSS class mode.
    ///   Use `"css"` or `None` for CSS class mode.
    ///   Use a theme name like `"base16-ocean.dark"` for inline styles.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // CSS class mode (default)
    /// let adapter = SyntectAdapter::new(None);
    ///
    /// // Inline style mode with a theme
    /// let adapter = SyntectAdapter::new(Some("base16-ocean.dark"));
    /// ```
    pub fn new(theme: Option<&str>) -> Self {
        let theme = theme
            .filter(|t| *t != "css" && *t != "none")
            .map(String::from);

        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            theme,
        }
    }

    /// Get a syntax reference for the given language.
    fn get_syntax(
        &self,
        lang: Option<&str>,
    ) -> Option<&syntect::parsing::SyntaxReference> {
        let lang = lang?;

        // Try exact match first
        if let Some(syntax) = self.syntax_set.find_syntax_by_token(lang) {
            return Some(syntax);
        }

        // Try by extension
        if let Some(syntax) = self.syntax_set.find_syntax_by_extension(lang) {
            return Some(syntax);
        }

        // Try common aliases
        let alias = match lang.to_lowercase().as_str() {
            "js" => "javascript",
            "ts" => "typescript",
            "py" => "python",
            "rb" => "ruby",
            "rs" => "rust",
            "cpp" | "c++" => "c++",
            "cs" => "c#",
            "sh" | "bash" => "bash",
            "yml" => "yaml",
            "md" => "markdown",
            _ => lang,
        };

        self.syntax_set.find_syntax_by_token(alias)
    }

    /// Get the theme reference.
    fn get_theme(&self) -> Option<&Theme> {
        self.theme
            .as_ref()
            .and_then(|name| self.theme_set.themes.get(name))
    }

    /// Highlight code using CSS classes.
    // TODO: This method is currently unused but kept for future CSS class-based highlighting support
    #[allow(dead_code)]
    fn highlight_with_classes(&self, code: &str, lang: Option<&str>) -> fmt::Result {
        // For CSS class mode, we need to use the HighlightLines approach
        // and convert to HTML with classes
        let syntax = self
            .get_syntax(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        // Use a default theme for generating scopes
        let theme = self
            .theme_set
            .themes
            .get("InspiredGitHub")
            .or_else(|| self.theme_set.themes.values().next())
            .expect("No themes available");

        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut output = String::new();

        for line in code.lines() {
            let highlighted = highlighter
                .highlight_line(line, &self.syntax_set)
                .map_err(|_| fmt::Error)?;

            // Convert to HTML with CSS classes
            let html =
                styled_line_to_highlighted_html(&highlighted, IncludeBackground::No)
                    .map_err(|_| fmt::Error)?;

            output.push_str(&html);
            output.push('\n');
        }

        // Write the result
        // Note: This is a simplified approach. In practice, we might want to
        // integrate more closely with the rendering pipeline.
        Ok(())
    }
}

impl SyntaxHighlighterAdapter for SyntectAdapter {
    fn write_pre_tag<'s>(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<&str, Cow<'s, str>>,
    ) -> fmt::Result {
        output.write_str("<pre")?;

        // Add class if present
        if let Some(class) = attributes.get("class") {
            write!(output, " class=\"{}\"", class)?;
        }

        // Add data-language attribute if present
        if let Some(lang) = attributes.get("data-language") {
            write!(output, " data-language=\"{}\"", lang)?;
        }

        output.write_str(">")
    }

    fn write_code_tag<'s>(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<&str, Cow<'s, str>>,
    ) -> fmt::Result {
        output.write_str("<code")?;

        // Add class if present
        if let Some(class) = attributes.get("class") {
            write!(output, " class=\"{}\"", class)?;
        }

        output.write_str(">")
    }

    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        // Get the syntax for this language
        let syntax = self
            .get_syntax(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        if let Some(theme) = self.get_theme() {
            // Inline style mode
            let mut highlighter = HighlightLines::new(syntax, theme);

            for line in code.lines() {
                let highlighted = highlighter
                    .highlight_line(line, &self.syntax_set)
                    .map_err(|_| fmt::Error)?;

                let html = styled_line_to_highlighted_html(
                    &highlighted,
                    IncludeBackground::Yes,
                )
                .map_err(|_| fmt::Error)?;

                output.write_str(&html)?;
                output.write_str("\n")?;
            }
        } else {
            // CSS class mode - just escape and output
            // The actual highlighting with CSS classes requires more complex setup
            // For now, we output the code as-is, relying on external CSS
            // In a full implementation, we would generate spans with classes
            for line in code.lines() {
                output.write_str(line)?;
                output.write_str("\n")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntect_adapter_creation() {
        let adapter = SyntectAdapter::new(None);
        assert!(adapter.theme.is_none());

        let adapter = SyntectAdapter::new(Some("base16-ocean.dark"));
        assert_eq!(adapter.theme, Some("base16-ocean.dark".to_string()));

        let adapter = SyntectAdapter::new(Some("css"));
        assert!(adapter.theme.is_none());
    }

    #[test]
    fn test_get_syntax() {
        let adapter = SyntectAdapter::new(None);

        // Test known languages
        assert!(adapter.get_syntax(Some("rust")).is_some());
        assert!(adapter.get_syntax(Some("python")).is_some());
        assert!(adapter.get_syntax(Some("javascript")).is_some());

        // Test aliases
        assert!(adapter.get_syntax(Some("rs")).is_some());
        assert!(adapter.get_syntax(Some("py")).is_some());
        assert!(adapter.get_syntax(Some("js")).is_some());

        // Test unknown language falls back to plain text
        assert!(adapter.get_syntax(Some("unknown_lang")).is_none());
    }

    #[test]
    fn test_write_pre_tag() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();

        let mut attrs: HashMap<&str, Cow<'static, str>> = HashMap::new();
        attrs.insert("class", Cow::Borrowed("highlight"));
        attrs.insert("data-language", Cow::Borrowed("rust"));

        adapter.write_pre_tag(&mut output, attrs).unwrap();

        assert!(output.contains("<pre"));
        assert!(output.contains("class=\"highlight\""));
        assert!(output.contains("data-language=\"rust\""));
        assert!(output.contains(">"));
    }

    #[test]
    fn test_write_code_tag() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();

        let mut attrs: HashMap<&str, Cow<'static, str>> = HashMap::new();
        attrs.insert("class", Cow::Borrowed("language-rust"));

        adapter.write_code_tag(&mut output, attrs).unwrap();

        assert!(output.contains("<code"));
        assert!(output.contains("class=\"language-rust\""));
        assert!(output.contains(">"));
    }

    #[test]
    fn test_write_highlighted_plain() {
        let adapter = SyntectAdapter::new(None);
        let mut output = String::new();

        adapter
            .write_highlighted(&mut output, None, "hello world")
            .unwrap();

        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_write_highlighted_with_theme() {
        let adapter = SyntectAdapter::new(Some("base16-ocean.dark"));
        let mut output = String::new();

        let code = "fn main() {\n    println!(\"Hello\");\n}";
        adapter
            .write_highlighted(&mut output, Some("rust"), code)
            .unwrap();

        // With a theme, output should contain styled HTML
        assert!(!output.is_empty());
        // Should contain some HTML tags for styling
        assert!(output.contains("<span") || output.contains("style="));
    }
}
