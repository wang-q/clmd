//! Reader and Writer options for clmd.
//!
//! This module provides Pandoc-inspired `ReaderOptions` and `WriterOptions`
//! structs for configuring document reading and writing operations.
//!
//! # Example
//!
//! ```
//! use clmd::core::{ReaderOptions, WriterOptions};
//!
//! let reader_opts = ReaderOptions::default();
//! let writer_opts = WriterOptions::default();
//! ```

use std::collections::HashMap;

use crate::extensions::Extensions;

/// Options for document readers.
///
/// This struct configures how documents are parsed from various input formats.
/// It is inspired by Pandoc's `ReaderOptions`.
#[derive(Debug, Clone)]
pub struct ReaderOptions {
    /// Enabled extensions for parsing.
    pub extensions: Extensions,

    /// Whether to parse as a standalone document (with metadata).
    pub standalone: bool,

    /// Column width for wrapping (0 = no wrapping).
    pub columns: usize,

    /// Tab stop width.
    pub tab_stop: usize,

    /// Default extension for images without explicit extensions.
    pub default_image_extension: String,

    /// Whether to strip HTML comments.
    pub strip_comments: bool,

    /// Whether to accept old-style pipe tables.
    pub old_dashes: bool,

    /// Whether to parse YAML metadata at the beginning.
    pub yaml_metadata: bool,

    /// Whether to track changes in the document.
    pub track_changes: bool,

    /// Abbreviations for smart punctuation.
    pub abbreviations: Vec<String>,

    /// Default metadata values.
    pub metadata: HashMap<String, String>,

    /// Whether to preserve tabs in source.
    pub preserve_tabs: bool,

    /// Whether to allow raw HTML.
    pub allow_raw_html: bool,

    /// Whether to apply smart punctuation.
    pub smart: bool,

    /// Whether to parse footnotes.
    pub footnotes: bool,

    /// Whether to parse citation references.
    pub citations: bool,

    /// File scope mode (for multi-file documents).
    pub file_scope: bool,

    /// Whether to parse indented code blocks.
    pub indented_code_classes: Vec<String>,
}

impl Default for ReaderOptions {
    fn default() -> Self {
        Self {
            extensions: Extensions::empty(),
            standalone: false,
            columns: 80,
            tab_stop: 4,
            default_image_extension: String::new(),
            strip_comments: false,
            old_dashes: false,
            yaml_metadata: true,
            track_changes: false,
            abbreviations: Vec::new(),
            metadata: HashMap::new(),
            preserve_tabs: false,
            allow_raw_html: true,
            smart: false,
            footnotes: false,
            citations: false,
            file_scope: false,
            indented_code_classes: Vec::new(),
        }
    }
}

impl ReaderOptions {
    /// Create new reader options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create reader options for a specific format.
    ///
    /// This sets default extensions based on the format name.
    pub fn for_format(format: &str) -> Self {
        let mut opts = Self::default();
        opts.extensions = Extensions::for_format(format);

        // Format-specific defaults
        match format {
            "markdown" | "md" => {
                opts.yaml_metadata = true;
                opts.smart = true;
            }
            "gfm" | "markdown_github" => {
                opts.extensions = Extensions::gfm();
                opts.yaml_metadata = true;
            }
            "commonmark" | "cm" => {
                opts.extensions = Extensions::empty();
                opts.yaml_metadata = false;
                opts.smart = false;
            }
            _ => {}
        }

        opts
    }

    /// Set the extensions.
    pub fn with_extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = extensions;
        self
    }

    /// Enable standalone mode.
    pub fn with_standalone(mut self, standalone: bool) -> Self {
        self.standalone = standalone;
        self
    }

    /// Set the column width.
    pub fn with_columns(mut self, columns: usize) -> Self {
        self.columns = columns;
        self
    }

    /// Set the tab stop width.
    pub fn with_tab_stop(mut self, tab_stop: usize) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    /// Set the default image extension.
    pub fn with_default_image_extension(mut self, ext: impl Into<String>) -> Self {
        self.default_image_extension = ext.into();
        self
    }

    /// Enable or disable smart punctuation.
    pub fn with_smart(mut self, smart: bool) -> Self {
        self.smart = smart;
        self
    }

    /// Enable or disable footnotes.
    pub fn with_footnotes(mut self, footnotes: bool) -> Self {
        self.footnotes = footnotes;
        self
    }

    /// Check if an extension is enabled.
    pub fn has_extension(&self, ext: Extensions) -> bool {
        self.extensions.contains(ext)
    }

    /// Add a metadata value.
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Get a metadata value.
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }
}

/// Options for document writers.
///
/// This struct configures how documents are written to various output formats.
/// It is inspired by Pandoc's `WriterOptions`.
#[derive(Debug, Clone)]
pub struct WriterOptions {
    /// Enabled extensions for writing.
    pub extensions: Extensions,

    /// Template for standalone documents.
    pub template: Option<String>,

    /// Template variables.
    pub variables: HashMap<String, String>,

    /// How to wrap text.
    pub wrap_text: WrapOption,

    /// Column width for wrapping.
    pub columns: usize,

    /// Tab stop width.
    pub tab_stop: usize,

    /// Whether to produce a standalone document.
    pub standalone: bool,

    /// Whether to include a table of contents.
    pub table_of_contents: bool,

    /// Depth of table of contents.
    pub toc_depth: usize,

    /// Highlighting style for code blocks.
    pub highlight_style: Option<String>,

    /// Whether to number sections.
    pub number_sections: bool,

    /// Number offset for sections.
    pub number_offset: Vec<usize>,

    /// Section depth for numbering.
    pub section_depth: usize,

    /// Whether to use incremental lists in presentations.
    pub incremental: bool,

    /// Slide level for presentations.
    pub slide_level: Option<usize>,

    /// Email obfuscation method.
    pub email_obfuscation: EmailObfuscation,

    /// Identifier prefix (for HTML output).
    pub identifier_prefix: String,

    /// Title prefix.
    pub title_prefix: Option<String>,

    /// CSS files to include (for HTML output).
    pub css: Vec<String>,

    /// Reference location for footnotes.
    pub reference_location: ReferenceLocation,

    /// Markdown flavor for output.
    pub markdown_flavor: MarkdownFlavor,

    /// Whether to set attributes on headers.
    pub setext_headers: bool,

    /// Whether to use ATX-style headers.
    pub atx_headers: bool,

    /// Whether to use reference-style links.
    pub reference_links: bool,

    /// Whether to use reference-style images.
    pub reference_images: bool,

    /// Whether to use fenced code blocks.
    pub fenced_code_blocks: bool,

    /// Preferred fence character for code blocks.
    pub preferred_fence: char,

    /// Whether to use implicit figures.
    pub implicit_figures: bool,

    /// Line ending style.
    pub line_ending: LineEnding,
}

impl Default for WriterOptions {
    fn default() -> Self {
        Self {
            extensions: Extensions::empty(),
            template: None,
            variables: HashMap::new(),
            wrap_text: WrapOption::Auto,
            columns: 80,
            tab_stop: 4,
            standalone: false,
            table_of_contents: false,
            toc_depth: 3,
            highlight_style: None,
            number_sections: false,
            number_offset: Vec::new(),
            section_depth: 3,
            incremental: false,
            slide_level: None,
            email_obfuscation: EmailObfuscation::None,
            identifier_prefix: String::new(),
            title_prefix: None,
            css: Vec::new(),
            reference_location: ReferenceLocation::Document,
            markdown_flavor: MarkdownFlavor::CommonMark,
            setext_headers: true,
            atx_headers: true,
            reference_links: false,
            reference_images: false,
            fenced_code_blocks: true,
            preferred_fence: '`',
            implicit_figures: false,
            line_ending: LineEnding::default(),
        }
    }
}

impl WriterOptions {
    /// Create new writer options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create writer options for a specific format.
    pub fn for_format(format: &str) -> Self {
        let mut opts = Self::default();
        opts.extensions = Extensions::for_format(format);

        // Format-specific defaults
        match format {
            "html" | "html5" => {
                opts.atx_headers = true;
                opts.fenced_code_blocks = true;
            }
            "markdown" | "md" => {
                opts.setext_headers = true;
                opts.fenced_code_blocks = true;
            }
            "gfm" | "markdown_github" => {
                opts.extensions = Extensions::gfm();
                opts.atx_headers = true;
                opts.fenced_code_blocks = true;
            }
            "commonmark" | "cm" => {
                opts.extensions = Extensions::empty();
                opts.setext_headers = true;
                opts.fenced_code_blocks = true;
            }
            _ => {}
        }

        opts
    }

    /// Set the extensions.
    pub fn with_extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = extensions;
        self
    }

    /// Set the template.
    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.template = Some(template.into());
        self
    }

    /// Enable standalone mode.
    pub fn with_standalone(mut self, standalone: bool) -> Self {
        self.standalone = standalone;
        self
    }

    /// Set the column width.
    pub fn with_columns(mut self, columns: usize) -> Self {
        self.columns = columns;
        self
    }

    /// Enable table of contents.
    pub fn with_toc(mut self, toc: bool) -> Self {
        self.table_of_contents = toc;
        self
    }

    /// Set the TOC depth.
    pub fn with_toc_depth(mut self, depth: usize) -> Self {
        self.toc_depth = depth;
        self
    }

    /// Add a variable.
    pub fn add_variable(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(key.into(), value.into());
    }

    /// Get a variable.
    pub fn get_variable(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(|s| s.as_str())
    }

    /// Add a CSS file (for HTML output).
    pub fn add_css(&mut self, css: impl Into<String>) {
        self.css.push(css.into());
    }

    /// Check if an extension is enabled.
    pub fn has_extension(&self, ext: Extensions) -> bool {
        self.extensions.contains(ext)
    }
}

/// Text wrapping options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapOption {
    /// Automatic wrapping based on column width.
    Auto,
    /// Preserve line breaks from source.
    Preserve,
    /// No wrapping, output as single line where possible.
    None,
}

impl Default for WrapOption {
    fn default() -> Self {
        Self::Auto
    }
}

/// Email obfuscation methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailObfuscation {
    /// No obfuscation.
    None,
    /// JavaScript obfuscation.
    Javascript,
    /// Reference-style obfuscation.
    References,
}

impl Default for EmailObfuscation {
    fn default() -> Self {
        Self::None
    }
}

/// Reference location for footnotes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceLocation {
    /// At the end of the document.
    Document,
    /// At the end of each block.
    Block,
    /// At the end of each section.
    Section,
}

impl Default for ReferenceLocation {
    fn default() -> Self {
        Self::Document
    }
}

/// Markdown flavor for output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownFlavor {
    /// CommonMark standard.
    CommonMark,
    /// GitHub Flavored Markdown.
    Gfm,
    /// Original Markdown.
    Markdown,
    /// MultiMarkdown.
    MultiMarkdown,
    /// PHP Markdown Extra.
    MarkdownExtra,
    /// Strict mode.
    Strict,
}

impl Default for MarkdownFlavor {
    fn default() -> Self {
        Self::CommonMark
    }
}

/// Line ending style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    /// Unix-style (LF).
    Unix,
    /// Windows-style (CRLF).
    Windows,
    /// Native to the platform.
    Native,
}

impl Default for LineEnding {
    fn default() -> Self {
        #[cfg(windows)]
        return Self::Windows;
        #[cfg(not(windows))]
        return Self::Unix;
    }
}

/// Extension configuration for formats.
///
/// This struct manages which extensions are enabled by default
/// for specific formats, and which are supported.
#[derive(Debug, Clone, Copy)]
pub struct ExtensionConfig {
    /// Default extensions for this format.
    pub defaults: Extensions,
    /// All supported extensions for this format.
    pub supported: Extensions,
}

impl ExtensionConfig {
    /// Create a new extension configuration.
    pub fn new(defaults: Extensions, supported: Extensions) -> Self {
        Self {
            defaults,
            supported,
        }
    }

    /// Get the extension config for a specific format.
    pub fn for_format(format: &str) -> Self {
        match format {
            "commonmark" | "cm" => Self::new(Extensions::empty(), Extensions::empty()),
            "gfm" | "markdown_github" => {
                let gfm = Extensions::gfm();
                Self::new(gfm, Extensions::all_extensions())
            }
            "markdown" | "md" => {
                Self::new(Extensions::empty(), Extensions::all_extensions())
            }
            "html" | "html5" => Self::new(Extensions::empty(), Extensions::empty()),
            "latex" | "tex" => Self::new(Extensions::empty(), Extensions::empty()),
            _ => Self::new(Extensions::empty(), Extensions::all_extensions()),
        }
    }

    /// Check if an extension is supported.
    pub fn is_supported(&self, ext: Extensions) -> bool {
        self.supported.contains(ext)
    }

    /// Check if an extension is enabled by default.
    pub fn is_default(&self, ext: Extensions) -> bool {
        self.defaults.contains(ext)
    }
}

/// Unified options struct combining reader and writer options.
///
/// This provides a convenient way to pass both reader and writer
/// options together.
#[derive(Debug, Clone)]
pub struct UnifiedOptions {
    /// Reader options.
    pub reader: ReaderOptions,
    /// Writer options.
    pub writer: WriterOptions,
}

impl Default for UnifiedOptions {
    fn default() -> Self {
        Self {
            reader: ReaderOptions::default(),
            writer: WriterOptions::default(),
        }
    }
}

impl UnifiedOptions {
    /// Create new unified options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create unified options for a specific format pair.
    pub fn for_formats(from: &str, to: &str) -> Self {
        Self {
            reader: ReaderOptions::for_format(from),
            writer: WriterOptions::for_format(to),
        }
    }

    /// Set reader options.
    pub fn with_reader(mut self, reader: ReaderOptions) -> Self {
        self.reader = reader;
        self
    }

    /// Set writer options.
    pub fn with_writer(mut self, writer: WriterOptions) -> Self {
        self.writer = writer;
        self
    }

    /// Enable an extension for both reader and writer.
    pub fn enable_extension(&mut self, ext: Extensions) {
        self.reader.extensions |= ext;
        self.writer.extensions |= ext;
    }

    /// Disable an extension for both reader and writer.
    pub fn disable_extension(&mut self, ext: Extensions) {
        self.reader.extensions &= !ext;
        self.writer.extensions &= !ext;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_options_default() {
        let opts = ReaderOptions::default();
        assert!(!opts.standalone);
        assert_eq!(opts.columns, 80);
        assert_eq!(opts.tab_stop, 4);
        assert!(opts.yaml_metadata);
    }

    #[test]
    fn test_reader_options_builder() {
        let opts = ReaderOptions::new()
            .with_standalone(true)
            .with_columns(100)
            .with_smart(true);

        assert!(opts.standalone);
        assert_eq!(opts.columns, 100);
        assert!(opts.smart);
    }

    #[test]
    fn test_reader_options_for_format() {
        let gfm = ReaderOptions::for_format("gfm");
        assert!(gfm.extensions.has_gfm_extensions());
        assert!(gfm.yaml_metadata);

        let cm = ReaderOptions::for_format("commonmark");
        assert!(!cm.extensions.has_gfm_extensions());
        assert!(!cm.yaml_metadata);
    }

    #[test]
    fn test_reader_options_metadata() {
        let mut opts = ReaderOptions::new();
        opts.add_metadata("title", "Test Document");
        opts.add_metadata("author", "Test Author");

        assert_eq!(opts.get_metadata("title"), Some("Test Document"));
        assert_eq!(opts.get_metadata("author"), Some("Test Author"));
        assert_eq!(opts.get_metadata("missing"), None);
    }

    #[test]
    fn test_writer_options_default() {
        let opts = WriterOptions::default();
        assert!(!opts.standalone);
        assert_eq!(opts.columns, 80);
        assert_eq!(opts.toc_depth, 3);
        assert!(!opts.table_of_contents);
    }

    #[test]
    fn test_writer_options_builder() {
        let opts = WriterOptions::new()
            .with_standalone(true)
            .with_toc(true)
            .with_toc_depth(2)
            .with_columns(120);

        assert!(opts.standalone);
        assert!(opts.table_of_contents);
        assert_eq!(opts.toc_depth, 2);
        assert_eq!(opts.columns, 120);
    }

    #[test]
    fn test_writer_options_variables() {
        let mut opts = WriterOptions::new();
        opts.add_variable("title", "My Document");
        opts.add_variable("date", "2024-01-01");

        assert_eq!(opts.get_variable("title"), Some("My Document"));
        assert_eq!(opts.get_variable("date"), Some("2024-01-01"));
    }

    #[test]
    fn test_writer_options_css() {
        let mut opts = WriterOptions::new();
        opts.add_css("style.css");
        opts.add_css("theme.css");

        assert_eq!(opts.css.len(), 2);
        assert!(opts.css.contains(&"style.css".to_string()));
    }

    #[test]
    fn test_wrap_option_default() {
        let wrap: WrapOption = Default::default();
        assert_eq!(wrap, WrapOption::Auto);
    }

    #[test]
    fn test_email_obfuscation_default() {
        let obf: EmailObfuscation = Default::default();
        assert_eq!(obf, EmailObfuscation::None);
    }

    #[test]
    fn test_reference_location_default() {
        let loc: ReferenceLocation = Default::default();
        assert_eq!(loc, ReferenceLocation::Document);
    }

    #[test]
    fn test_markdown_flavor_default() {
        let flavor: MarkdownFlavor = Default::default();
        assert_eq!(flavor, MarkdownFlavor::CommonMark);
    }

    #[test]
    fn test_extension_config_for_format() {
        let cm = ExtensionConfig::for_format("commonmark");
        assert_eq!(cm.defaults, Extensions::empty());
        assert_eq!(cm.supported, Extensions::empty());

        let gfm = ExtensionConfig::for_format("gfm");
        assert!(gfm.defaults.has_gfm_extensions());
        assert!(gfm.supported.contains(Extensions::TABLES));
    }

    #[test]
    fn test_unified_options() {
        let opts = UnifiedOptions::new();
        assert!(!opts.reader.standalone);
        assert!(!opts.writer.standalone);

        let mut opts = UnifiedOptions::for_formats("markdown", "html");
        opts.enable_extension(Extensions::TABLES);

        assert!(opts.reader.has_extension(Extensions::TABLES));
        assert!(opts.writer.has_extension(Extensions::TABLES));

        opts.disable_extension(Extensions::TABLES);
        assert!(!opts.reader.has_extension(Extensions::TABLES));
        assert!(!opts.writer.has_extension(Extensions::TABLES));
    }
}
