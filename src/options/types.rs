//! Basic types for options configuration.
//!
//! This module defines fundamental types used across the options system,
//! including output formats, wrapping options, and style enumerations.

use arbitrary::Arbitrary;
use std::fmt;

/// Output format for writing documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Arbitrary)]
pub enum OutputFormat {
    /// Markdown (CommonMark)
    #[default]
    Markdown,
    /// GitHub Flavored Markdown
    Gfm,
    /// HTML
    Html,
    /// XHTML
    Xhtml,
    /// XML (CommonMark AST)
    Xml,
    /// LaTeX
    Latex,
    /// Plain text
    Plain,
    /// Docx
    Docx,
    /// ODT
    Odt,
    /// EPUB
    Epub,
    /// Beamer (LaTeX slides)
    Beamer,
    /// RevealJS (HTML slides)
    RevealJs,
    /// BibTeX
    Bibtex,
}

impl OutputFormat {
    /// Get the format name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Markdown => "markdown",
            OutputFormat::Gfm => "gfm",
            OutputFormat::Html => "html",
            OutputFormat::Xhtml => "xhtml",
            OutputFormat::Xml => "xml",
            OutputFormat::Latex => "latex",
            OutputFormat::Plain => "plain",
            OutputFormat::Docx => "docx",
            OutputFormat::Odt => "odt",
            OutputFormat::Epub => "epub",
            OutputFormat::Beamer => "beamer",
            OutputFormat::RevealJs => "revealjs",
            OutputFormat::Bibtex => "bibtex",
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" | "commonmark" => Ok(OutputFormat::Markdown),
            "gfm" => Ok(OutputFormat::Gfm),
            "html" => Ok(OutputFormat::Html),
            "xhtml" => Ok(OutputFormat::Xhtml),
            "xml" => Ok(OutputFormat::Xml),
            "latex" | "tex" => Ok(OutputFormat::Latex),
            "plain" | "text" => Ok(OutputFormat::Plain),
            "docx" => Ok(OutputFormat::Docx),
            "odt" => Ok(OutputFormat::Odt),
            "epub" => Ok(OutputFormat::Epub),
            "beamer" => Ok(OutputFormat::Beamer),
            "revealjs" => Ok(OutputFormat::RevealJs),
            "bibtex" | "bib" => Ok(OutputFormat::Bibtex),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

/// Text wrapping option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum WrapOption {
    /// Auto-wrap text.
    #[default]
    Auto,
    /// No wrapping.
    None,
    /// Preserve line breaks.
    Preserve,
}

/// Style type for bullet lists.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum ListStyleType {
    /// Use `-` for bullet lists.
    #[default]
    Dash,
    /// Use `+` for bullet lists.
    Plus,
    /// Use `*` for bullet lists.
    Star,
}

impl ListStyleType {
    /// Get the marker character.
    pub fn marker(&self) -> char {
        match self {
            ListStyleType::Dash => '-',
            ListStyleType::Plus => '+',
            ListStyleType::Star => '*',
        }
    }
}

impl fmt::Display for ListStyleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.marker())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_default() {
        let format: OutputFormat = Default::default();
        assert_eq!(format, OutputFormat::Markdown);
    }

    #[test]
    fn test_wrap_option_default() {
        let wrap: WrapOption = Default::default();
        assert_eq!(wrap, WrapOption::Auto);
    }

    #[test]
    fn test_list_style_type_default() {
        let style: ListStyleType = Default::default();
        assert_eq!(style, ListStyleType::Dash);
    }
}
