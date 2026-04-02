//! Format abstraction layer for clmd.
//!
//! This module provides a unified interface for document formats, inspired by
//! Pandoc's format system. It supports both text and binary formats, and provides
//! format detection, MIME type mapping, and format-specific configuration.
//!
//! # Example
//!
//! ```ignore
//! use clmd::io::format::{Format, FormatRegistry};
//!
//! let registry = FormatRegistry::new();
//!
//! // Detect format from file extension
//! if let Some(format) = registry.detect("document.md") {
//!     println!("Detected format: {:?}", format);
//! }
//! ```ignore

// HTML conversion from reader module
pub use crate::io::reader::html;

use crate::core::error::ClmdError;
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
    /// PDF
    Pdf,
    /// DOCX
    Docx,
    /// EPUB
    Epub,
    /// Text
    Text,
    /// RTF
    Rtf,
    /// ODT
    Odt,
    /// Typst
    Typst,
    /// Man page
    Man,
    /// Reveal.js
    Revealjs,
    /// Beamer
    Beamer,
    /// BibTeX
    Bibtex,
    /// CSV
    Csv,
    /// JSON
    Json,
    /// YAML
    Yaml,
    /// TOML
    Toml,
    /// Unknown format
    Unknown,
}

impl Format {
    /// Check if this format is a binary format.
    pub fn is_binary(&self) -> bool {
        matches!(
            self,
            Format::Pdf | Format::Docx | Format::Epub | Format::Odt
        )
    }

    /// Check if this format is a text format.
    pub fn is_text(&self) -> bool {
        !self.is_binary()
    }

    /// Check if this format can be used as input.
    pub fn is_input(&self) -> bool {
        matches!(
            self,
            Format::Markdown
                | Format::Gfm
                | Format::Html
                | Format::Xhtml
                | Format::Xml
                | Format::Latex
                | Format::Text
                | Format::Bibtex
                | Format::Csv
                | Format::Json
                | Format::Yaml
                | Format::Toml
        )
    }

    /// Check if this format can be used as output.
    pub fn is_output(&self) -> bool {
        !matches!(self, Format::Unknown)
    }

    /// Get the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Format::Markdown => "md",
            Format::Gfm => "md",
            Format::Html => "html",
            Format::Xhtml => "xhtml",
            Format::Xml => "xml",
            Format::Latex => "tex",
            Format::Pdf => "pdf",
            Format::Docx => "docx",
            Format::Epub => "epub",
            Format::Text => "txt",
            Format::Rtf => "rtf",
            Format::Odt => "odt",
            Format::Typst => "typ",
            Format::Man => "man",
            Format::Revealjs => "html",
            Format::Beamer => "tex",
            Format::Bibtex => "bib",
            Format::Csv => "csv",
            Format::Json => "json",
            Format::Yaml => "yaml",
            Format::Toml => "toml",
            Format::Unknown => "",
        }
    }

    /// Get the MIME type for this format.
    pub fn mime_type(&self) -> &'static str {
        match self {
            Format::Markdown => "text/markdown",
            Format::Gfm => "text/markdown",
            Format::Html => "text/html",
            Format::Xhtml => "application/xhtml+xml",
            Format::Xml => "application/xml",
            Format::Latex => "application/x-latex",
            Format::Pdf => "application/pdf",
            Format::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            Format::Epub => "application/epub+zip",
            Format::Text => "text/plain",
            Format::Rtf => "application/rtf",
            Format::Odt => "application/vnd.oasis.opendocument.text",
            Format::Typst => "text/typst",
            Format::Man => "application/x-troff-man",
            Format::Revealjs => "text/html",
            Format::Beamer => "application/x-latex",
            Format::Bibtex => "application/x-bibtex",
            Format::Csv => "text/csv",
            Format::Json => "application/json",
            Format::Yaml => "application/yaml",
            Format::Toml => "application/toml",
            Format::Unknown => "application/octet-stream",
        }
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Format::Markdown => "markdown",
            Format::Gfm => "gfm",
            Format::Html => "html",
            Format::Xhtml => "xhtml",
            Format::Xml => "xml",
            Format::Latex => "latex",
            Format::Pdf => "pdf",
            Format::Docx => "docx",
            Format::Epub => "epub",
            Format::Text => "text",
            Format::Rtf => "rtf",
            Format::Odt => "odt",
            Format::Typst => "typst",
            Format::Man => "man",
            Format::Revealjs => "revealjs",
            Format::Beamer => "beamer",
            Format::Bibtex => "bibtex",
            Format::Csv => "csv",
            Format::Json => "json",
            Format::Yaml => "yaml",
            Format::Toml => "toml",
            Format::Unknown => "unknown",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for Format {
    type Err = ClmdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" | "commonmark" => Ok(Format::Markdown),
            "gfm" | "github" => Ok(Format::Gfm),
            "html" | "htm" => Ok(Format::Html),
            "xhtml" => Ok(Format::Xhtml),
            "xml" => Ok(Format::Xml),
            "latex" | "tex" => Ok(Format::Latex),
            "pdf" => Ok(Format::Pdf),
            "docx" => Ok(Format::Docx),
            "epub" => Ok(Format::Epub),
            "text" | "txt" | "plain" => Ok(Format::Text),
            "rtf" => Ok(Format::Rtf),
            "odt" => Ok(Format::Odt),
            "typst" | "typ" => Ok(Format::Typst),
            "man" => Ok(Format::Man),
            "revealjs" | "reveal" => Ok(Format::Revealjs),
            "beamer" => Ok(Format::Beamer),
            "bibtex" | "bib" => Ok(Format::Bibtex),
            "csv" => Ok(Format::Csv),
            "json" => Ok(Format::Json),
            "yaml" | "yml" => Ok(Format::Yaml),
            "toml" => Ok(Format::Toml),
            _ => Ok(Format::Unknown),
        }
    }
}

/// A registry for format detection and management.
#[derive(Debug, Clone)]
pub struct FormatRegistry {
    /// Map of file extensions to formats
    extensions: HashMap<String, Format>,
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatRegistry {
    /// Create a new format registry with default mappings.
    pub fn new() -> Self {
        let mut extensions = HashMap::new();

        // Markdown
        extensions.insert("md".to_string(), Format::Markdown);
        extensions.insert("markdown".to_string(), Format::Markdown);
        extensions.insert("mkd".to_string(), Format::Markdown);

        // HTML
        extensions.insert("html".to_string(), Format::Html);
        extensions.insert("htm".to_string(), Format::Html);

        // LaTeX
        extensions.insert("tex".to_string(), Format::Latex);
        extensions.insert("latex".to_string(), Format::Latex);

        // PDF
        extensions.insert("pdf".to_string(), Format::Pdf);

        // DOCX
        extensions.insert("docx".to_string(), Format::Docx);

        // EPUB
        extensions.insert("epub".to_string(), Format::Epub);

        // Text
        extensions.insert("txt".to_string(), Format::Text);
        extensions.insert("text".to_string(), Format::Text);

        // RTF
        extensions.insert("rtf".to_string(), Format::Rtf);

        // ODT
        extensions.insert("odt".to_string(), Format::Odt);

        // Typst
        extensions.insert("typ".to_string(), Format::Typst);
        extensions.insert("typst".to_string(), Format::Typst);

        // Man
        extensions.insert("man".to_string(), Format::Man);

        // BibTeX
        extensions.insert("bib".to_string(), Format::Bibtex);
        extensions.insert("bibtex".to_string(), Format::Bibtex);

        // CSV
        extensions.insert("csv".to_string(), Format::Csv);

        // JSON
        extensions.insert("json".to_string(), Format::Json);

        // YAML
        extensions.insert("yaml".to_string(), Format::Yaml);
        extensions.insert("yml".to_string(), Format::Yaml);

        // TOML
        extensions.insert("toml".to_string(), Format::Toml);

        Self { extensions }
    }

    /// Detect the format from a file path.
    pub fn detect(&self, path: &Path) -> Option<Format> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.extensions.get(&ext.to_lowercase()))
            .copied()
    }

    /// Get the format for a file extension.
    pub fn get(&self, extension: &str) -> Option<Format> {
        self.extensions.get(&extension.to_lowercase()).copied()
    }

    /// Register a new format extension.
    pub fn register(&mut self, extension: &str, format: Format) {
        self.extensions.insert(extension.to_lowercase(), format);
    }

    /// Check if a format is supported.
    pub fn is_supported(&self, format: &str) -> bool {
        format
            .parse::<Format>()
            .is_ok_and(|f| !matches!(f, Format::Unknown))
    }
}

/// Format category for grouping related formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatCategory {
    /// Markup formats (HTML, XML, etc.)
    Markup,
    /// Document formats (PDF, DOCX, etc.)
    Document,
    /// Text formats
    Text,
    /// Data formats (JSON, YAML, etc.)
    Data,
    /// Unknown category
    Unknown,
}

impl FormatCategory {
    /// Get the category for a format.
    pub fn from_format(format: Format) -> Self {
        match format {
            Format::Html | Format::Xhtml | Format::Xml => FormatCategory::Markup,
            Format::Pdf | Format::Docx | Format::Epub | Format::Odt => {
                FormatCategory::Document
            }
            Format::Text
            | Format::Markdown
            | Format::Gfm
            | Format::Latex
            | Format::Typst
            | Format::Man => FormatCategory::Text,
            Format::Json
            | Format::Yaml
            | Format::Toml
            | Format::Csv
            | Format::Bibtex => FormatCategory::Data,
            _ => FormatCategory::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_str() {
        assert!(matches!(
            "markdown".parse::<Format>().unwrap(),
            Format::Markdown
        ));
        assert!(matches!("md".parse::<Format>().unwrap(), Format::Markdown));
        assert!(matches!("gfm".parse::<Format>().unwrap(), Format::Gfm));
        assert!(matches!("github".parse::<Format>().unwrap(), Format::Gfm));
        assert!(matches!("html".parse::<Format>().unwrap(), Format::Html));
        assert!(matches!("htm".parse::<Format>().unwrap(), Format::Html));
        assert!(matches!("xhtml".parse::<Format>().unwrap(), Format::Xhtml));
        assert!(matches!("xml".parse::<Format>().unwrap(), Format::Xml));
        assert!(matches!("latex".parse::<Format>().unwrap(), Format::Latex));
        assert!(matches!("tex".parse::<Format>().unwrap(), Format::Latex));
        assert!(matches!("pdf".parse::<Format>().unwrap(), Format::Pdf));
        assert!(matches!("docx".parse::<Format>().unwrap(), Format::Docx));
        assert!(matches!("epub".parse::<Format>().unwrap(), Format::Epub));
        assert!(matches!("text".parse::<Format>().unwrap(), Format::Text));
        assert!(matches!("txt".parse::<Format>().unwrap(), Format::Text));
        assert!(matches!("plain".parse::<Format>().unwrap(), Format::Text));
        assert!(matches!("rtf".parse::<Format>().unwrap(), Format::Rtf));
        assert!(matches!("odt".parse::<Format>().unwrap(), Format::Odt));
        assert!(matches!("typst".parse::<Format>().unwrap(), Format::Typst));
        assert!(matches!("typ".parse::<Format>().unwrap(), Format::Typst));
        assert!(matches!("man".parse::<Format>().unwrap(), Format::Man));
        assert!(matches!(
            "revealjs".parse::<Format>().unwrap(),
            Format::Revealjs
        ));
        assert!(matches!(
            "reveal".parse::<Format>().unwrap(),
            Format::Revealjs
        ));
        assert!(matches!(
            "beamer".parse::<Format>().unwrap(),
            Format::Beamer
        ));
        assert!(matches!(
            "bibtex".parse::<Format>().unwrap(),
            Format::Bibtex
        ));
        assert!(matches!("bib".parse::<Format>().unwrap(), Format::Bibtex));
        assert!(matches!("csv".parse::<Format>().unwrap(), Format::Csv));
        assert!(matches!("json".parse::<Format>().unwrap(), Format::Json));
        assert!(matches!("yaml".parse::<Format>().unwrap(), Format::Yaml));
        assert!(matches!("yml".parse::<Format>().unwrap(), Format::Yaml));
        assert!(matches!("toml".parse::<Format>().unwrap(), Format::Toml));
    }

    #[test]
    fn test_format_from_str_unknown() {
        assert!(matches!(
            "unknown".parse::<Format>().unwrap(),
            Format::Unknown
        ));
        assert!(matches!("xyz".parse::<Format>().unwrap(), Format::Unknown));
    }

    #[test]
    fn test_format_display() {
        assert_eq!(Format::Markdown.to_string(), "markdown");
        assert_eq!(Format::Gfm.to_string(), "gfm");
        assert_eq!(Format::Html.to_string(), "html");
        assert_eq!(Format::Xhtml.to_string(), "xhtml");
        assert_eq!(Format::Xml.to_string(), "xml");
        assert_eq!(Format::Latex.to_string(), "latex");
        assert_eq!(Format::Pdf.to_string(), "pdf");
        assert_eq!(Format::Docx.to_string(), "docx");
        assert_eq!(Format::Epub.to_string(), "epub");
        assert_eq!(Format::Text.to_string(), "text");
        assert_eq!(Format::Rtf.to_string(), "rtf");
        assert_eq!(Format::Odt.to_string(), "odt");
        assert_eq!(Format::Typst.to_string(), "typst");
        assert_eq!(Format::Man.to_string(), "man");
        assert_eq!(Format::Revealjs.to_string(), "revealjs");
        assert_eq!(Format::Beamer.to_string(), "beamer");
        assert_eq!(Format::Bibtex.to_string(), "bibtex");
        assert_eq!(Format::Csv.to_string(), "csv");
        assert_eq!(Format::Json.to_string(), "json");
        assert_eq!(Format::Yaml.to_string(), "yaml");
        assert_eq!(Format::Toml.to_string(), "toml");
        assert_eq!(Format::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_format_is_binary() {
        assert!(Format::Pdf.is_binary());
        assert!(Format::Docx.is_binary());
        assert!(Format::Epub.is_binary());
        assert!(Format::Odt.is_binary());
        assert!(!Format::Markdown.is_binary());
        assert!(!Format::Html.is_binary());
        assert!(!Format::Text.is_binary());
    }

    #[test]
    fn test_format_is_text() {
        assert!(Format::Markdown.is_text());
        assert!(Format::Html.is_text());
        assert!(Format::Text.is_text());
        assert!(!Format::Pdf.is_text());
        assert!(!Format::Docx.is_text());
    }

    #[test]
    fn test_format_is_input() {
        assert!(Format::Markdown.is_input());
        assert!(Format::Gfm.is_input());
        assert!(Format::Html.is_input());
        assert!(Format::Xhtml.is_input());
        assert!(Format::Xml.is_input());
        assert!(Format::Latex.is_input());
        assert!(Format::Text.is_input());
        assert!(Format::Bibtex.is_input());
        assert!(Format::Csv.is_input());
        assert!(Format::Json.is_input());
        assert!(Format::Yaml.is_input());
        assert!(Format::Toml.is_input());
        assert!(!Format::Pdf.is_input());
        assert!(!Format::Docx.is_input());
        assert!(!Format::Epub.is_input());
    }

    #[test]
    fn test_format_is_output() {
        assert!(Format::Markdown.is_output());
        assert!(Format::Html.is_output());
        assert!(Format::Pdf.is_output());
        assert!(!Format::Unknown.is_output());
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(Format::Markdown.extension(), "md");
        assert_eq!(Format::Gfm.extension(), "md");
        assert_eq!(Format::Html.extension(), "html");
        assert_eq!(Format::Xhtml.extension(), "xhtml");
        assert_eq!(Format::Xml.extension(), "xml");
        assert_eq!(Format::Latex.extension(), "tex");
        assert_eq!(Format::Pdf.extension(), "pdf");
        assert_eq!(Format::Docx.extension(), "docx");
        assert_eq!(Format::Epub.extension(), "epub");
        assert_eq!(Format::Text.extension(), "txt");
        assert_eq!(Format::Rtf.extension(), "rtf");
        assert_eq!(Format::Odt.extension(), "odt");
        assert_eq!(Format::Typst.extension(), "typ");
        assert_eq!(Format::Man.extension(), "man");
        assert_eq!(Format::Revealjs.extension(), "html");
        assert_eq!(Format::Beamer.extension(), "tex");
        assert_eq!(Format::Bibtex.extension(), "bib");
        assert_eq!(Format::Csv.extension(), "csv");
        assert_eq!(Format::Json.extension(), "json");
        assert_eq!(Format::Yaml.extension(), "yaml");
        assert_eq!(Format::Toml.extension(), "toml");
        assert_eq!(Format::Unknown.extension(), "");
    }

    #[test]
    fn test_format_mime_type() {
        assert_eq!(Format::Markdown.mime_type(), "text/markdown");
        assert_eq!(Format::Html.mime_type(), "text/html");
        assert_eq!(Format::Pdf.mime_type(), "application/pdf");
        assert_eq!(Format::Json.mime_type(), "application/json");
        assert_eq!(Format::Unknown.mime_type(), "application/octet-stream");
    }

    #[test]
    fn test_format_registry() {
        let registry = FormatRegistry::new();

        let md_path = std::path::Path::new("test.md");
        assert!(matches!(
            registry.detect(md_path).unwrap(),
            Format::Markdown
        ));

        let html_path = std::path::Path::new("test.html");
        assert!(matches!(registry.detect(html_path).unwrap(), Format::Html));
    }

    #[test]
    fn test_format_registry_detect() {
        let registry = FormatRegistry::new();

        // Test various extensions
        assert!(matches!(
            registry.detect(std::path::Path::new("doc.md")).unwrap(),
            Format::Markdown
        ));
        assert!(matches!(
            registry
                .detect(std::path::Path::new("doc.markdown"))
                .unwrap(),
            Format::Markdown
        ));
        assert!(matches!(
            registry.detect(std::path::Path::new("doc.mkd")).unwrap(),
            Format::Markdown
        ));
        assert!(matches!(
            registry.detect(std::path::Path::new("page.html")).unwrap(),
            Format::Html
        ));
        assert!(matches!(
            registry.detect(std::path::Path::new("doc.pdf")).unwrap(),
            Format::Pdf
        ));
        assert!(matches!(
            registry.detect(std::path::Path::new("doc.tex")).unwrap(),
            Format::Latex
        ));

        // Test no extension
        assert!(registry.detect(std::path::Path::new("README")).is_none());

        // Test unknown extension
        assert!(registry.detect(std::path::Path::new("file.xyz")).is_none());
    }

    #[test]
    fn test_format_registry_get() {
        let registry = FormatRegistry::new();

        assert!(matches!(registry.get("md").unwrap(), Format::Markdown));
        assert!(matches!(registry.get("html").unwrap(), Format::Html));
        assert!(matches!(registry.get("MD").unwrap(), Format::Markdown));
        assert!(registry.get("xyz").is_none());
    }

    #[test]
    fn test_format_registry_register() {
        let mut registry = FormatRegistry::new();
        registry.register("custom", Format::Markdown);
        assert!(matches!(registry.get("custom").unwrap(), Format::Markdown));
    }

    #[test]
    fn test_format_registry_is_supported() {
        let registry = FormatRegistry::new();
        assert!(registry.is_supported("markdown"));
        assert!(registry.is_supported("html"));
        assert!(registry.is_supported("pdf"));
        assert!(!registry.is_supported("unknown"));
        assert!(!registry.is_supported("xyz"));
    }

    #[test]
    fn test_format_category() {
        assert_eq!(
            FormatCategory::from_format(Format::Html),
            FormatCategory::Markup
        );
        assert_eq!(
            FormatCategory::from_format(Format::Xhtml),
            FormatCategory::Markup
        );
        assert_eq!(
            FormatCategory::from_format(Format::Xml),
            FormatCategory::Markup
        );
        assert_eq!(
            FormatCategory::from_format(Format::Pdf),
            FormatCategory::Document
        );
        assert_eq!(
            FormatCategory::from_format(Format::Docx),
            FormatCategory::Document
        );
        assert_eq!(
            FormatCategory::from_format(Format::Epub),
            FormatCategory::Document
        );
        assert_eq!(
            FormatCategory::from_format(Format::Odt),
            FormatCategory::Document
        );
        assert_eq!(
            FormatCategory::from_format(Format::Text),
            FormatCategory::Text
        );
        assert_eq!(
            FormatCategory::from_format(Format::Markdown),
            FormatCategory::Text
        );
        assert_eq!(
            FormatCategory::from_format(Format::Json),
            FormatCategory::Data
        );
        assert_eq!(
            FormatCategory::from_format(Format::Yaml),
            FormatCategory::Data
        );
        assert_eq!(
            FormatCategory::from_format(Format::Unknown),
            FormatCategory::Unknown
        );
    }

    #[test]
    fn test_format_registry_default() {
        let registry: FormatRegistry = Default::default();
        assert!(registry.get("md").is_some());
    }
}
