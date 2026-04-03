//! Basic types for options configuration.
//!
//! This module defines fundamental types used across the options system,
//! including input/output formats, wrapping options, and style enumerations.

use arbitrary::Arbitrary;
use std::fmt;

/// Input format for reading documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum InputFormat {
    /// Markdown (CommonMark)
    #[default]
    Markdown,
    /// GitHub Flavored Markdown
    Gfm,
    /// HTML
    Html,
    /// BibTeX
    Bibtex,
    /// LaTeX
    Latex,
    /// AsciiDoc
    AsciiDoc,
    /// Org mode
    Org,
    /// Textile
    Textile,
    /// MediaWiki
    MediaWiki,
    /// DokuWiki
    DokuWiki,
}

impl InputFormat {
    /// Get the format name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            InputFormat::Markdown => "markdown",
            InputFormat::Gfm => "gfm",
            InputFormat::Html => "html",
            InputFormat::Bibtex => "bibtex",
            InputFormat::Latex => "latex",
            InputFormat::AsciiDoc => "asciidoc",
            InputFormat::Org => "org",
            InputFormat::Textile => "textile",
            InputFormat::MediaWiki => "mediawiki",
            InputFormat::DokuWiki => "dokuwiki",
        }
    }
}

impl fmt::Display for InputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Output format for writing documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
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
    /// Man page
    Man,
    /// Plain text
    Plain,
    /// PDF
    Pdf,
    /// Docx
    Docx,
    /// ODT
    Odt,
    /// RTF
    Rtf,
    /// EPUB
    Epub,
    /// Typst
    Typst,
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
            OutputFormat::Man => "man",
            OutputFormat::Plain => "plain",
            OutputFormat::Pdf => "pdf",
            OutputFormat::Docx => "docx",
            OutputFormat::Odt => "odt",
            OutputFormat::Rtf => "rtf",
            OutputFormat::Epub => "epub",
            OutputFormat::Typst => "typst",
            OutputFormat::Beamer => "beamer",
            OutputFormat::RevealJs => "revealjs",
            OutputFormat::Bibtex => "bibtex",
        }
    }

    /// Check if this format requires a binary output.
    pub fn is_binary(&self) -> bool {
        matches!(
            self,
            OutputFormat::Pdf
                | OutputFormat::Docx
                | OutputFormat::Odt
                | OutputFormat::Epub
                | OutputFormat::Rtf
        )
    }

    /// Check if this is a slide format.
    pub fn is_slides(&self) -> bool {
        matches!(self, OutputFormat::Beamer | OutputFormat::RevealJs)
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
            "man" => Ok(OutputFormat::Man),
            "plain" | "text" => Ok(OutputFormat::Plain),
            "pdf" => Ok(OutputFormat::Pdf),
            "docx" => Ok(OutputFormat::Docx),
            "odt" => Ok(OutputFormat::Odt),
            "rtf" => Ok(OutputFormat::Rtf),
            "epub" => Ok(OutputFormat::Epub),
            "typst" => Ok(OutputFormat::Typst),
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

impl WrapOption {
    /// Check if wrapping is enabled.
    pub fn is_wrapping(&self) -> bool {
        matches!(self, WrapOption::Auto)
    }
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

/// Selects between wikilinks with the title first or the URL first.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Arbitrary)]
pub enum WikiLinksMode {
    /// Indicates that the URL precedes the title.
    /// For example: `[[http://example.com|link title]]`.
    UrlFirst,
    /// Indicates that the title precedes the URL.
    /// For example: `[[link title|http://example.com]]`.
    TitleFirst,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_format_as_str() {
        assert_eq!(InputFormat::Markdown.as_str(), "markdown");
        assert_eq!(InputFormat::Gfm.as_str(), "gfm");
        assert_eq!(InputFormat::Html.as_str(), "html");
        assert_eq!(InputFormat::Bibtex.as_str(), "bibtex");
        assert_eq!(InputFormat::Latex.as_str(), "latex");
    }

    #[test]
    fn test_input_format_default() {
        let format: InputFormat = Default::default();
        assert_eq!(format, InputFormat::Markdown);
    }

    #[test]
    fn test_output_format_as_str() {
        assert_eq!(OutputFormat::Markdown.as_str(), "markdown");
        assert_eq!(OutputFormat::Gfm.as_str(), "gfm");
        assert_eq!(OutputFormat::Html.as_str(), "html");
        assert_eq!(OutputFormat::Xhtml.as_str(), "xhtml");
        assert_eq!(OutputFormat::Xml.as_str(), "xml");
        assert_eq!(OutputFormat::Latex.as_str(), "latex");
        assert_eq!(OutputFormat::Man.as_str(), "man");
        assert_eq!(OutputFormat::Plain.as_str(), "plain");
        assert_eq!(OutputFormat::Pdf.as_str(), "pdf");
        assert_eq!(OutputFormat::Docx.as_str(), "docx");
        assert_eq!(OutputFormat::Typst.as_str(), "typst");
        assert_eq!(OutputFormat::Beamer.as_str(), "beamer");
        assert_eq!(OutputFormat::RevealJs.as_str(), "revealjs");
        assert_eq!(OutputFormat::Bibtex.as_str(), "bibtex");
    }

    #[test]
    fn test_output_format_default() {
        let format: OutputFormat = Default::default();
        assert_eq!(format, OutputFormat::Markdown);
    }

    #[test]
    fn test_output_format_from_str() {
        use std::str::FromStr;

        assert_eq!(
            OutputFormat::from_str("markdown").unwrap(),
            OutputFormat::Markdown
        );
        assert_eq!(OutputFormat::from_str("md").unwrap(), OutputFormat::Markdown);
        assert_eq!(
            OutputFormat::from_str("commonmark").unwrap(),
            OutputFormat::Markdown
        );
        assert_eq!(OutputFormat::from_str("gfm").unwrap(), OutputFormat::Gfm);
        assert_eq!(OutputFormat::from_str("html").unwrap(), OutputFormat::Html);
        assert_eq!(OutputFormat::from_str("HTML").unwrap(), OutputFormat::Html);
        assert!(OutputFormat::from_str("unknown").is_err());
    }

    #[test]
    fn test_output_format_is_binary() {
        assert!(OutputFormat::Pdf.is_binary());
        assert!(OutputFormat::Docx.is_binary());
        assert!(OutputFormat::Epub.is_binary());
        assert!(!OutputFormat::Html.is_binary());
        assert!(!OutputFormat::Markdown.is_binary());
    }

    #[test]
    fn test_output_format_is_slides() {
        assert!(OutputFormat::Beamer.is_slides());
        assert!(OutputFormat::RevealJs.is_slides());
        assert!(!OutputFormat::Html.is_slides());
        assert!(!OutputFormat::Pdf.is_slides());
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

    #[test]
    fn test_list_style_type_marker() {
        assert_eq!(ListStyleType::Dash.marker(), '-');
        assert_eq!(ListStyleType::Plus.marker(), '+');
        assert_eq!(ListStyleType::Star.marker(), '*');
    }
}
