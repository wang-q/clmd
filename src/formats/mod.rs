//! Format abstraction layer for clmd.
//!
//! This module provides a unified interface for document formats, inspired by
//! Pandoc's format system. It supports both text and binary formats, and provides
//! format detection, MIME type mapping, and format-specific configuration.
//!
//! # Example
//!
//! ```ignore
//! use clmd::formats::{Format, FormatRegistry};
//!
//! let registry = FormatRegistry::new();
//!
//! // Detect format from file extension
//! if let Some(format) = registry.detect("document.md") {
//!     println!("Detected format: {:?}", format);
//! }
//! ```ignore

pub mod css;
pub mod csv;
pub mod mime;
pub mod tex;
pub mod xml;

use crate::error::{ClmdError, ClmdResult};
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::path::Path;
use std::str::FromStr;

/// A document format identifier.
///
/// Formats can be input formats, output formats, or both. This enum provides
/// a type-safe way to work with formats throughout the codebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// Markdown (CommonMark)
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
    /// JSON
    Json,
    /// YAML
    Yaml,
    /// PDF
    Pdf,
    /// Docx
    Docx,
    /// ODT (OpenDocument Text)
    Odt,
    /// RTF (Rich Text Format)
    Rtf,
    /// EPUB
    Epub,
    /// Typst
    Typst,
    /// ReStructuredText
    Rst,
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
    /// Other format with custom name
    Other(&'static str),
}

impl Format {
    /// Get the format name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Format::Markdown => "markdown",
            Format::Gfm => "gfm",
            Format::Html => "html",
            Format::Xhtml => "xhtml",
            Format::Xml => "xml",
            Format::Latex => "latex",
            Format::Man => "man",
            Format::Plain => "plain",
            Format::Json => "json",
            Format::Yaml => "yaml",
            Format::Pdf => "pdf",
            Format::Docx => "docx",
            Format::Odt => "odt",
            Format::Rtf => "rtf",
            Format::Epub => "epub",
            Format::Typst => "typst",
            Format::Rst => "rst",
            Format::AsciiDoc => "asciidoc",
            Format::Org => "org",
            Format::Textile => "textile",
            Format::MediaWiki => "mediawiki",
            Format::DokuWiki => "dokuwiki",
            Format::Other(name) => name,
        }
    }

    /// Check if this format is a binary format.
    pub fn is_binary(&self) -> bool {
        matches!(
            self,
            Format::Pdf | Format::Docx | Format::Odt | Format::Rtf | Format::Epub
        )
    }

    /// Check if this format is a text format.
    pub fn is_text(&self) -> bool {
        !self.is_binary()
    }

    /// Check if this format can be used as an input format.
    pub fn is_reader_format(&self) -> bool {
        matches!(
            self,
            Format::Markdown
                | Format::Gfm
                | Format::Html
                | Format::Rst
                | Format::AsciiDoc
                | Format::Org
                | Format::Textile
                | Format::MediaWiki
                | Format::DokuWiki
                | Format::Json
                | Format::Yaml
        )
    }

    /// Check if this format can be used as an output format.
    pub fn is_writer_format(&self) -> bool {
        matches!(
            self,
            Format::Markdown
                | Format::Gfm
                | Format::Html
                | Format::Xhtml
                | Format::Xml
                | Format::Latex
                | Format::Man
                | Format::Plain
                | Format::Json
                | Format::Pdf
                | Format::Docx
                | Format::Odt
                | Format::Rtf
                | Format::Epub
                | Format::Typst
        )
    }

    /// Get the MIME type for this format.
    pub fn mime_type(&self) -> &'static str {
        match self {
            Format::Markdown | Format::Gfm => "text/markdown",
            Format::Html => "text/html",
            Format::Xhtml => "application/xhtml+xml",
            Format::Xml => "application/xml",
            Format::Latex => "application/x-latex",
            Format::Man => "application/x-troff-man",
            Format::Plain => "text/plain",
            Format::Json => "application/json",
            Format::Yaml => "application/yaml",
            Format::Pdf => "application/pdf",
            Format::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            Format::Odt => "application/vnd.oasis.opendocument.text",
            Format::Rtf => "application/rtf",
            Format::Epub => "application/epub+zip",
            Format::Typst => "text/typst",
            Format::Rst => "text/x-rst",
            Format::AsciiDoc => "text/asciidoc",
            Format::Org => "text/org",
            Format::Textile => "text/textile",
            Format::MediaWiki => "text/mediawiki",
            Format::DokuWiki => "text/dokuwiki",
            Format::Other(_) => "application/octet-stream",
        }
    }

    /// Get the default file extensions for this format.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Format::Markdown => &["md", "markdown", "mkd", "mdown"],
            Format::Gfm => &["md", "markdown"],
            Format::Html => &["html", "htm"],
            Format::Xhtml => &["xhtml", "xht"],
            Format::Xml => &["xml"],
            Format::Latex => &["tex", "latex"],
            Format::Man => &["man", "1", "2", "3", "4", "5", "6", "7", "8"],
            Format::Plain => &["txt", "text"],
            Format::Json => &["json"],
            Format::Yaml => &["yaml", "yml"],
            Format::Pdf => &["pdf"],
            Format::Docx => &["docx"],
            Format::Odt => &["odt"],
            Format::Rtf => &["rtf"],
            Format::Epub => &["epub"],
            Format::Typst => &["typ"],
            Format::Rst => &["rst"],
            Format::AsciiDoc => &["adoc", "asciidoc"],
            Format::Org => &["org"],
            Format::Textile => &["textile"],
            Format::MediaWiki => &["mediawiki", "wiki"],
            Format::DokuWiki => &["dokuwiki"],
            Format::Other(_) => &[],
        }
    }

    /// Check if the given extension matches this format.
    pub fn matches_extension(&self, ext: &str) -> bool {
        let ext_lower = ext.to_lowercase();
        self.extensions().contains(&ext_lower.as_str())
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Format {
    type Err = ClmdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "commonmark" => Ok(Format::Markdown),
            "gfm" | "github" => Ok(Format::Gfm),
            "html" => Ok(Format::Html),
            "xhtml" => Ok(Format::Xhtml),
            "xml" => Ok(Format::Xml),
            "latex" | "tex" => Ok(Format::Latex),
            "man" | "groff" => Ok(Format::Man),
            "plain" | "text" => Ok(Format::Plain),
            "json" => Ok(Format::Json),
            "yaml" | "yml" => Ok(Format::Yaml),
            "pdf" => Ok(Format::Pdf),
            "docx" => Ok(Format::Docx),
            "odt" => Ok(Format::Odt),
            "rtf" => Ok(Format::Rtf),
            "epub" => Ok(Format::Epub),
            "typst" => Ok(Format::Typst),
            "rst" | "rest" => Ok(Format::Rst),
            "asciidoc" | "adoc" => Ok(Format::AsciiDoc),
            "org" | "orgmode" => Ok(Format::Org),
            "textile" => Ok(Format::Textile),
            "mediawiki" | "mw" => Ok(Format::MediaWiki),
            "dokuwiki" => Ok(Format::DokuWiki),
            _ => Err(ClmdError::unknown_format(s)),
        }
    }
}

/// A flavored format with extensions.
///
/// This represents a format with specific extensions enabled, similar to
/// Pandoc's FlavoredFormat. For example, "markdown+smart-tasklists" or
/// "gfm-hard_line_breaks".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlavoredFormat {
    /// The base format.
    pub format: Format,
    /// Extensions to enable (positive).
    pub extensions: Vec<String>,
    /// Extensions to disable (negative).
    pub disabled_extensions: Vec<String>,
}

impl FlavoredFormat {
    /// Create a new flavored format.
    pub fn new(format: Format) -> Self {
        Self {
            format,
            extensions: Vec::new(),
            disabled_extensions: Vec::new(),
        }
    }

    /// Add an extension.
    pub fn with_extension(mut self, ext: impl Into<String>) -> Self {
        self.extensions.push(ext.into());
        self
    }

    /// Disable an extension.
    pub fn without_extension(mut self, ext: impl Into<String>) -> Self {
        self.disabled_extensions.push(ext.into());
        self
    }

    /// Parse a format string with extensions.
    ///
    /// Format: `format+ext1-ext2+ext3`
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::formats::FlavoredFormat;
    ///
    /// let flavored = FlavoredFormat::parse("markdown+smart-tasklists").unwrap();
    /// assert_eq!(flavored.format.as_str(), "markdown");
    /// assert!(flavored.extensions.contains(&"smart".to_string()));
    /// assert!(flavored.disabled_extensions.contains(&"tasklists".to_string()));
    /// ```
    pub fn parse(s: &str) -> ClmdResult<Self> {
        let parts: Vec<&str> = s.split(|c| c == '+' || c == '-').collect();

        if parts.is_empty() {
            return Err(ClmdError::unknown_format(s));
        }

        let format = parts[0].parse::<Format>()?;
        let mut flavored = Self::new(format);

        // Parse extensions
        let mut chars = s.chars().peekable();
        let mut current_ext = String::new();
        let mut sign = '+';

        // Skip the format name
        while let Some(c) = chars.next() {
            if c == '+' || c == '-' {
                sign = c;
                break;
            }
        }

        for c in chars {
            if c == '+' || c == '-' {
                if !current_ext.is_empty() {
                    if sign == '+' {
                        flavored.extensions.push(current_ext.clone());
                    } else {
                        flavored.disabled_extensions.push(current_ext.clone());
                    }
                    current_ext.clear();
                }
                sign = c;
            } else {
                current_ext.push(c);
            }
        }

        // Don't forget the last extension
        if !current_ext.is_empty() {
            if sign == '+' {
                flavored.extensions.push(current_ext);
            } else {
                flavored.disabled_extensions.push(current_ext);
            }
        }

        Ok(flavored)
    }

    /// Format as a string.
    pub fn to_string(&self) -> String {
        let mut result = self.format.as_str().to_string();

        for ext in &self.extensions {
            result.push('+');
            result.push_str(ext);
        }

        for ext in &self.disabled_extensions {
            result.push('-');
            result.push_str(ext);
        }

        result
    }
}

/// Format registry for format detection and lookup.
///
/// The registry provides methods to detect formats from file paths and
/// look up format information.
#[derive(Debug, Default)]
pub struct FormatRegistry {
    formats: Vec<Format>,
    extension_map: HashMap<&'static str, Format>,
}

impl FormatRegistry {
    /// Create a new registry with all supported formats.
    pub fn new() -> Self {
        let mut registry = Self::empty();
        registry.register_default_formats();
        registry
    }

    /// Create an empty registry.
    pub fn empty() -> Self {
        Self {
            formats: Vec::new(),
            extension_map: HashMap::new(),
        }
    }

    /// Register a format.
    pub fn register(&mut self, format: Format) {
        // Register extensions
        for ext in format.extensions() {
            self.extension_map.insert(ext, format);
        }

        // Register the format
        if !self.formats.contains(&format) {
            self.formats.push(format);
        }
    }

    /// Detect format from a file path.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::formats::FormatRegistry;
    ///
    /// let registry = FormatRegistry::new();
    ///
    /// let format = registry.detect("document.html");
    /// assert!(format.is_some());
    /// assert_eq!(format.unwrap().as_str(), "html");
    /// ```
    pub fn detect<P: AsRef<Path>>(&self, path: P) -> Option<Format> {
        path.as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.extension_map.get(ext).copied())
    }

    /// Get a format by name.
    pub fn get(&self, name: &str) -> Option<Format> {
        name.parse::<Format>().ok()
    }

    /// Get all registered formats.
    pub fn formats(&self) -> &[Format] {
        &self.formats
    }

    /// Get all registered extensions.
    pub fn extensions(&self) -> Vec<&'static str> {
        self.extension_map.keys().copied().collect()
    }

    /// Check if a format is supported.
    pub fn supports_format(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Check if an extension is supported.
    pub fn supports_extension(&self, ext: &str) -> bool {
        let ext_lower = ext.to_lowercase();
        self.extension_map.contains_key(ext_lower.as_str())
    }

    /// Get formats that support reading.
    pub fn reader_formats(&self) -> Vec<Format> {
        self.formats
            .iter()
            .copied()
            .filter(|f| f.is_reader_format())
            .collect()
    }

    /// Get formats that support writing.
    pub fn writer_formats(&self) -> Vec<Format> {
        self.formats
            .iter()
            .copied()
            .filter(|f| f.is_writer_format())
            .collect()
    }

    /// Register default formats.
    fn register_default_formats(&mut self) {
        // Input formats
        self.register(Format::Markdown);
        self.register(Format::Gfm);
        self.register(Format::Html);
        self.register(Format::Rst);
        self.register(Format::AsciiDoc);
        self.register(Format::Org);
        self.register(Format::Textile);
        self.register(Format::MediaWiki);
        self.register(Format::DokuWiki);
        self.register(Format::Json);
        self.register(Format::Yaml);

        // Output formats
        self.register(Format::Xhtml);
        self.register(Format::Xml);
        self.register(Format::Latex);
        self.register(Format::Man);
        self.register(Format::Plain);
        self.register(Format::Pdf);
        self.register(Format::Docx);
        self.register(Format::Odt);
        self.register(Format::Rtf);
        self.register(Format::Epub);
        self.register(Format::Typst);
    }
}

impl Clone for FormatRegistry {
    fn clone(&self) -> Self {
        Self::new()
    }
}

/// Format category for organizing formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatCategory {
    /// Markup formats (Markdown, HTML, etc.)
    Markup,
    /// Word processing formats (Docx, ODT, etc.)
    WordProcessing,
    /// Page layout formats (PDF, etc.)
    PageLayout,
    /// Presentation formats
    Presentation,
    /// E-book formats
    Ebook,
    /// Documentation formats (Man, etc.)
    Documentation,
    /// Data interchange formats (JSON, YAML, XML)
    Data,
    /// Other formats
    Other,
}

impl Format {
    /// Get the category of this format.
    pub fn category(&self) -> FormatCategory {
        match self {
            Format::Markdown
            | Format::Gfm
            | Format::Html
            | Format::Xhtml
            | Format::Rst
            | Format::AsciiDoc
            | Format::Org
            | Format::Textile
            | Format::MediaWiki
            | Format::DokuWiki => FormatCategory::Markup,
            Format::Docx | Format::Odt | Format::Rtf => FormatCategory::WordProcessing,
            Format::Pdf | Format::Typst => FormatCategory::PageLayout,
            Format::Epub => FormatCategory::Ebook,
            Format::Man | Format::Latex => FormatCategory::Documentation,
            Format::Json | Format::Yaml | Format::Xml => FormatCategory::Data,
            Format::Plain | Format::Other(_) => FormatCategory::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_as_str() {
        assert_eq!(Format::Markdown.as_str(), "markdown");
        assert_eq!(Format::Html.as_str(), "html");
        assert_eq!(Format::Pdf.as_str(), "pdf");
    }

    #[test]
    fn test_format_is_binary() {
        assert!(!Format::Markdown.is_binary());
        assert!(!Format::Html.is_binary());
        assert!(Format::Pdf.is_binary());
        assert!(Format::Docx.is_binary());
    }

    #[test]
    fn test_format_is_text() {
        assert!(Format::Markdown.is_text());
        assert!(Format::Html.is_text());
        assert!(!Format::Pdf.is_text());
        assert!(!Format::Docx.is_text());
    }

    #[test]
    fn test_format_from_str() {
        assert_eq!("markdown".parse::<Format>().unwrap(), Format::Markdown);
        assert_eq!("html".parse::<Format>().unwrap(), Format::Html);
        assert_eq!("gfm".parse::<Format>().unwrap(), Format::Gfm);
        assert!("unknown".parse::<Format>().is_err());
    }

    #[test]
    fn test_format_matches_extension() {
        assert!(Format::Markdown.matches_extension("md"));
        assert!(Format::Markdown.matches_extension("markdown"));
        assert!(Format::Html.matches_extension("html"));
        assert!(!Format::Html.matches_extension("md"));
    }

    #[test]
    fn test_flavored_format_parse() {
        let flavored = FlavoredFormat::parse("markdown+smart-tasklists").unwrap();
        assert_eq!(flavored.format, Format::Markdown);
        assert!(flavored.extensions.contains(&"smart".to_string()));
        assert!(flavored
            .disabled_extensions
            .contains(&"tasklists".to_string()));
    }

    #[test]
    fn test_flavored_format_to_string() {
        let flavored = FlavoredFormat::new(Format::Markdown)
            .with_extension("smart")
            .without_extension("tasklists");

        assert_eq!(flavored.to_string(), "markdown+smart-tasklists");
    }

    #[test]
    fn test_format_registry_detect() {
        let registry = FormatRegistry::new();

        // Note: .md maps to Gfm because Gfm is registered after Markdown
        // and both have "md" in their extensions
        let detected = registry.detect("document.md");
        assert!(detected.is_some());
        assert!(matches!(detected.unwrap(), Format::Markdown | Format::Gfm));

        assert_eq!(registry.detect("document.html"), Some(Format::Html));
        assert_eq!(registry.detect("document.pdf"), Some(Format::Pdf));
        assert_eq!(registry.detect("document"), None);
    }

    #[test]
    fn test_format_registry_get() {
        let registry = FormatRegistry::new();

        assert_eq!(registry.get("markdown"), Some(Format::Markdown));
        assert_eq!(registry.get("html"), Some(Format::Html));
        assert_eq!(registry.get("unknown"), None);
    }

    #[test]
    fn test_format_registry_supports() {
        let registry = FormatRegistry::new();

        assert!(registry.supports_format("markdown"));
        assert!(registry.supports_format("html"));
        assert!(!registry.supports_format("unknown"));

        assert!(registry.supports_extension("md"));
        assert!(registry.supports_extension("html"));
        assert!(!registry.supports_extension("xyz"));
    }

    #[test]
    fn test_format_category() {
        assert_eq!(Format::Markdown.category(), FormatCategory::Markup);
        assert_eq!(Format::Html.category(), FormatCategory::Markup);
        assert_eq!(Format::Pdf.category(), FormatCategory::PageLayout);
        assert_eq!(Format::Json.category(), FormatCategory::Data);
    }
}
