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
        assert!(matches!("html".parse::<Format>().unwrap(), Format::Html));
        assert!(matches!("pdf".parse::<Format>().unwrap(), Format::Pdf));
    }

    #[test]
    fn test_format_display() {
        assert_eq!(Format::Markdown.to_string(), "markdown");
        assert_eq!(Format::Html.to_string(), "html");
        assert_eq!(Format::Pdf.to_string(), "pdf");
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
}
