//! Unified options system for clmd, inspired by Pandoc's ReaderOptions and WriterOptions.
//!
//! This module provides a comprehensive options structure that combines
//! parsing and rendering configuration in a single place, similar to
//! Pandoc's approach with ReaderOptions and WriterOptions.
//!
//! # Example
//!
//! ```
//! use clmd::clmd_options::{ClmdOptions, InputFormat, OutputFormat};
//!
//! let options = ClmdOptions::default()
//!     .with_input_format(InputFormat::CommonMark)
//!     .with_output_format(OutputFormat::Html);
//! ```

use std::collections::HashMap;
use std::path::PathBuf;

use crate::extensions::Extensions;
use crate::options::{Extension as LegacyExtension, ListStyleType, Parse, Render};

/// Input format for parsing documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputFormat {
    /// CommonMark format (default).
    #[default]
    CommonMark,
    /// GitHub Flavored Markdown.
    Gfm,
    /// Markdown with all extensions.
    Markdown,
    /// HTML input.
    Html,
    /// LaTeX input.
    Latex,
    /// Typst input.
    Typst,
}

impl InputFormat {
    /// Get the format name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            InputFormat::CommonMark => "commonmark",
            InputFormat::Gfm => "gfm",
            InputFormat::Markdown => "markdown",
            InputFormat::Html => "html",
            InputFormat::Latex => "latex",
            InputFormat::Typst => "typst",
        }
    }

    /// Detect format from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" | "mkd" => Some(InputFormat::Markdown),
            "html" | "htm" => Some(InputFormat::Html),
            "tex" | "latex" => Some(InputFormat::Latex),
            "typ" | "typst" => Some(InputFormat::Typst),
            _ => None,
        }
    }

    /// Get default extensions for this input format.
    pub fn default_extensions(&self) -> Extensions {
        match self {
            InputFormat::CommonMark => Extensions::empty(),
            InputFormat::Gfm => Extensions::gfm(),
            InputFormat::Markdown => Extensions::all_extensions(),
            InputFormat::Html => Extensions::empty(),
            InputFormat::Latex => Extensions::empty(),
            InputFormat::Typst => Extensions::empty(),
        }
    }
}

impl std::str::FromStr for InputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "commonmark" => Ok(InputFormat::CommonMark),
            "gfm" | "github-flavored-markdown" => Ok(InputFormat::Gfm),
            "markdown" | "md" => Ok(InputFormat::Markdown),
            "html" | "htm" => Ok(InputFormat::Html),
            "latex" | "tex" => Ok(InputFormat::Latex),
            "typst" | "typ" => Ok(InputFormat::Typst),
            _ => Err(format!("Unknown input format: {}", s)),
        }
    }
}

/// Output format for rendering documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// HTML format (default).
    #[default]
    Html,
    /// XHTML format.
    Xhtml,
    /// CommonMark output.
    CommonMark,
    /// GitHub Flavored Markdown output.
    Gfm,
    /// XML AST representation.
    Xml,
    /// LaTeX output.
    Latex,
    /// Typst output.
    Typst,
    /// PDF output.
    Pdf,
    /// Man page output.
    Man,
    /// Plain text output.
    Plain,
}

impl OutputFormat {
    /// Get the format name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Html => "html",
            OutputFormat::Xhtml => "xhtml",
            OutputFormat::CommonMark => "commonmark",
            OutputFormat::Gfm => "gfm",
            OutputFormat::Xml => "xml",
            OutputFormat::Latex => "latex",
            OutputFormat::Typst => "typst",
            OutputFormat::Pdf => "pdf",
            OutputFormat::Man => "man",
            OutputFormat::Plain => "plain",
        }
    }

    /// Detect format from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "html" | "htm" => Some(OutputFormat::Html),
            "xhtml" => Some(OutputFormat::Xhtml),
            "md" | "markdown" | "mkd" => Some(OutputFormat::CommonMark),
            "tex" | "latex" => Some(OutputFormat::Latex),
            "typ" | "typst" => Some(OutputFormat::Typst),
            "pdf" => Some(OutputFormat::Pdf),
            "man" => Some(OutputFormat::Man),
            "txt" => Some(OutputFormat::Plain),
            _ => None,
        }
    }

    /// Check if this format is a binary format.
    pub fn is_binary(&self) -> bool {
        matches!(self, OutputFormat::Pdf)
    }

    /// Check if this format requires a standalone document.
    pub fn requires_standalone(&self) -> bool {
        matches!(
            self,
            OutputFormat::Pdf | OutputFormat::Latex | OutputFormat::Typst
        )
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "html" => Ok(OutputFormat::Html),
            "xhtml" => Ok(OutputFormat::Xhtml),
            "commonmark" | "markdown" | "md" => Ok(OutputFormat::CommonMark),
            "gfm" => Ok(OutputFormat::Gfm),
            "xml" => Ok(OutputFormat::Xml),
            "latex" | "tex" => Ok(OutputFormat::Latex),
            "typst" | "typ" => Ok(OutputFormat::Typst),
            "pdf" => Ok(OutputFormat::Pdf),
            "man" => Ok(OutputFormat::Man),
            "plain" | "txt" | "text" => Ok(OutputFormat::Plain),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

/// Options for text wrapping in output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WrapOption {
    /// Automatically wrap text to the specified column width.
    #[default]
    Auto,
    /// Do not wrap text (no non-semantic newlines).
    None,
    /// Preserve the wrapping from the input source.
    Preserve,
}

/// Options for HTML math rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HTMLMathMethod {
    /// Plain text math (no special rendering).
    Plain,
    /// MathJax with optional URL.
    MathJax(Option<String>),
    /// KaTeX with optional URL.
    KaTeX(Option<String>),
    /// WebTeX with optional URL.
    WebTeX(Option<String>),
    /// MathML.
    MathML,
}

impl Default for HTMLMathMethod {
    fn default() -> Self {
        HTMLMathMethod::Plain
    }
}

/// Unified options structure for clmd operations.
///
/// This struct combines reader (parsing) and writer (rendering) options
/// in a single place, inspired by Pandoc's ReaderOptions and WriterOptions.
///
/// # Example
///
/// ```ignore
/// use clmd::clmd_options::{ClmdOptions, InputFormat, OutputFormat, WrapOption};
///
/// let options = ClmdOptions::default()
///     .with_input_format(InputFormat::Gfm)
///     .with_output_format(OutputFormat::Html)
///     .with_wrap(WrapOption::Auto)
///     .with_columns(80);
/// ```ignore
#[derive(Debug, Clone)]
pub struct ClmdOptions {
    // =========================================================================
    // Input/Output Format
    // =========================================================================
    /// Input format for parsing.
    pub input_format: InputFormat,

    /// Output format for rendering.
    pub output_format: OutputFormat,

    /// Standalone document (include header/footer).
    pub standalone: bool,

    // =========================================================================
    // Extensions
    // =========================================================================
    /// Syntax extensions to enable.
    pub extensions: Extensions,

    // =========================================================================
    // Parsing Options
    // =========================================================================
    /// Convert smart quotes, dashes, and ellipses.
    pub smart: bool,

    /// Include source position attributes.
    pub sourcepos: bool,

    /// Tab stop width (number of spaces).
    pub tab_stop: usize,

    /// Validate UTF-8 in input.
    pub validate_utf8: bool,

    /// Default info string for indented code blocks.
    pub default_info_string: Option<String>,

    // =========================================================================
    // Rendering Options
    // =========================================================================
    /// Convert soft line breaks to hard line breaks.
    pub hardbreaks: bool,

    /// Allow raw HTML and dangerous URLs (security risk!).
    pub unsafe_html: bool,

    /// Escape raw HTML instead of removing it.
    pub escape_html: bool,

    /// GitHub-style `<pre lang="xyz">` for code blocks.
    pub github_pre_lang: bool,

    /// Include full info strings for code blocks.
    pub full_info_string: bool,

    /// Wrap column for text output (0 = no wrapping).
    pub columns: usize,

    /// Text wrapping option.
    pub wrap: WrapOption,

    /// List style type for bullet lists.
    pub list_style: ListStyleType,

    /// Prefer fenced code blocks in output.
    pub prefer_fenced: bool,

    /// Width for ordered list markers.
    pub ol_width: usize,

    /// Include source position attributes in output.
    pub output_sourcepos: bool,

    /// GitHub Flavored Markdown quirks mode.
    pub gfm_quirks: bool,

    /// Render images as figures with captions.
    pub figure_with_caption: bool,

    /// HTML math rendering method.
    pub html_math_method: HTMLMathMethod,

    /// Number sections in output.
    pub number_sections: bool,

    /// Table of contents depth (0 = no TOC).
    pub toc_depth: usize,

    // =========================================================================
    // Resource Options
    // =========================================================================
    /// Resource paths for looking up files.
    pub resource_path: Vec<PathBuf>,

    /// User data directory.
    pub user_data_dir: Option<PathBuf>,

    /// Extract media to this directory.
    pub extract_media: Option<PathBuf>,

    // =========================================================================
    // Template Options
    // =========================================================================
    /// Template file path.
    pub template: Option<PathBuf>,

    /// Template variables.
    pub variables: HashMap<String, String>,

    // =========================================================================
    // Metadata
    // =========================================================================
    /// Document title (overrides metadata).
    pub title: Option<String>,

    /// Document author (overrides metadata).
    pub author: Option<String>,

    /// Document date (overrides metadata).
    pub date: Option<String>,
}

impl Default for ClmdOptions {
    fn default() -> Self {
        Self {
            input_format: InputFormat::default(),
            output_format: OutputFormat::default(),
            standalone: false,
            extensions: Extensions::empty(),
            smart: false,
            sourcepos: false,
            tab_stop: 4,
            validate_utf8: true,
            default_info_string: None,
            hardbreaks: false,
            unsafe_html: false,
            escape_html: false,
            github_pre_lang: false,
            full_info_string: false,
            columns: 80,
            wrap: WrapOption::default(),
            list_style: ListStyleType::default(),
            prefer_fenced: false,
            ol_width: 3,
            output_sourcepos: false,
            gfm_quirks: false,
            figure_with_caption: false,
            html_math_method: HTMLMathMethod::default(),
            number_sections: false,
            toc_depth: 0,
            resource_path: Vec::new(),
            user_data_dir: None,
            extract_media: None,
            template: None,
            variables: HashMap::new(),
            title: None,
            author: None,
            date: None,
        }
    }
}

impl ClmdOptions {
    /// Create new options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create options for GFM input/output.
    pub fn gfm() -> Self {
        Self {
            input_format: InputFormat::Gfm,
            output_format: OutputFormat::Html,
            extensions: Extensions::gfm(),
            github_pre_lang: true,
            ..Default::default()
        }
    }

    /// Create options for CommonMark strict mode.
    pub fn commonmark_strict() -> Self {
        Self {
            input_format: InputFormat::CommonMark,
            output_format: OutputFormat::CommonMark,
            extensions: Extensions::empty(),
            ..Default::default()
        }
    }

    // =========================================================================
    // Builder-style methods
    // =========================================================================

    /// Set the input format.
    pub fn with_input_format(mut self, format: InputFormat) -> Self {
        self.input_format = format;
        // Auto-enable default extensions for the format
        if self.extensions.is_empty() {
            self.extensions = format.default_extensions();
        }
        self
    }

    /// Set the output format.
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Set standalone mode.
    pub fn with_standalone(mut self, standalone: bool) -> Self {
        self.standalone = standalone;
        self
    }

    /// Set extensions.
    pub fn with_extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = extensions;
        self
    }

    /// Enable an extension.
    pub fn with_extension(mut self, extension: Extensions) -> Self {
        self.extensions |= extension;
        self
    }

    /// Set smart punctuation.
    pub fn with_smart(mut self, smart: bool) -> Self {
        self.smart = smart;
        self
    }

    /// Set source position tracking.
    pub fn with_sourcepos(mut self, sourcepos: bool) -> Self {
        self.sourcepos = sourcepos;
        self
    }

    /// Set tab stop width.
    pub fn with_tab_stop(mut self, tab_stop: usize) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    /// Set hard line breaks.
    pub fn with_hardbreaks(mut self, hardbreaks: bool) -> Self {
        self.hardbreaks = hardbreaks;
        self
    }

    /// Set unsafe HTML mode.
    pub fn with_unsafe(mut self, unsafe_html: bool) -> Self {
        self.unsafe_html = unsafe_html;
        self
    }

    /// Set escape HTML mode.
    pub fn with_escape_html(mut self, escape_html: bool) -> Self {
        self.escape_html = escape_html;
        self
    }

    /// Set wrap column.
    pub fn with_columns(mut self, columns: usize) -> Self {
        self.columns = columns;
        self
    }

    /// Set wrap option.
    pub fn with_wrap(mut self, wrap: WrapOption) -> Self {
        self.wrap = wrap;
        self
    }

    /// Add a resource path.
    pub fn add_resource_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.resource_path.push(path.into());
        self
    }

    /// Set template.
    pub fn with_template(mut self, template: impl Into<PathBuf>) -> Self {
        self.template = Some(template.into());
        self
    }

    /// Add a template variable.
    pub fn with_variable(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Set title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set date.
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }

    /// Set HTML math method.
    pub fn with_html_math(mut self, method: HTMLMathMethod) -> Self {
        self.html_math_method = method;
        self
    }

    /// Set TOC depth.
    pub fn with_toc_depth(mut self, depth: usize) -> Self {
        self.toc_depth = depth;
        self
    }

    // =========================================================================
    // Conversion methods
    // =========================================================================

    /// Convert to parser Options for compatibility.
    pub fn to_parser_options(&self) -> crate::parser::options::Options<'_> {
        crate::parser::options::Options {
            extension: self.to_extension(),
            parse: self.to_parse(),
            render: self.to_render(),
        }
    }

    /// Convert extensions to legacy Extension struct.
    fn to_extension(&self) -> LegacyExtension<'_> {
        LegacyExtension {
            strikethrough: self.extensions.contains(Extensions::STRIKETHROUGH),
            tagfilter: self.extensions.contains(Extensions::TAGFILTER),
            table: self.extensions.contains(Extensions::TABLES),
            autolink: self.extensions.contains(Extensions::AUTOLINKS),
            tasklist: self.extensions.contains(Extensions::TASKLISTS),
            superscript: self.extensions.contains(Extensions::SUPERSCRIPT),
            subscript: self.extensions.contains(Extensions::SUBSCRIPT),
            header_ids: None, // TODO: Add support
            footnotes: self.extensions.contains(Extensions::FOOTNOTES),
            inline_footnotes: self.extensions.contains(Extensions::INLINE_FOOTNOTES),
            description_lists: self.extensions.contains(Extensions::DESCRIPTION_LISTS),
            front_matter_delimiter: None, // TODO: Add support
            multiline_block_quotes: self
                .extensions
                .contains(Extensions::MULTILINE_BLOCK_QUOTES),
            alerts: self.extensions.contains(Extensions::ALERTS),
            math_dollars: self.extensions.contains(Extensions::MATH_DOLLARS),
            math_code: self.extensions.contains(Extensions::MATH_CODE),
            wikilinks_title_after_pipe: false, // TODO: Add support
            wikilinks_title_before_pipe: false, // TODO: Add support
            underline: self.extensions.contains(Extensions::UNDERLINE),
            spoiler: self.extensions.contains(Extensions::SPOILER),
            greentext: self.extensions.contains(Extensions::GREENTEXT),
            highlight: self.extensions.contains(Extensions::HIGHLIGHT),
            insert: self.extensions.contains(Extensions::INSERT),
            cjk_friendly_emphasis: self
                .extensions
                .contains(Extensions::CJK_FRIENDLY_EMPHASIS),
            subtext: self.extensions.contains(Extensions::SUBTEXT),
            shortcodes: self.extensions.contains(Extensions::SHORTCODES),
            image_url_rewriter: None,
            link_url_rewriter: None,
        }
    }

    /// Convert to legacy Parse struct.
    fn to_parse(&self) -> Parse<'_> {
        Parse {
            smart: self.smart,
            sourcepos: self.sourcepos,
            validate_utf8: self.validate_utf8,
            default_info_string: self.default_info_string.clone(),
            relaxed_tasklist_matching: false,
            ignore_setext: false,
            leave_footnote_definitions: false,
            tasklist_in_table: false,
            relaxed_autolinks: false,
            escaped_char_spans: false,
            broken_link_callback: None,
        }
    }

    /// Convert to legacy Render struct.
    fn to_render(&self) -> Render {
        Render {
            hardbreaks: self.hardbreaks,
            nobreaks: false,
            r#unsafe: self.unsafe_html,
            escape: self.escape_html,
            github_pre_lang: self.github_pre_lang,
            full_info_string: self.full_info_string,
            width: if self.wrap == WrapOption::Auto {
                self.columns
            } else {
                0
            },
            list_style: self.list_style,
            prefer_fenced: self.prefer_fenced,
            ignore_empty_links: false,
            tasklist_classes: false,
            compact_html: false,
            sourcepos: self.output_sourcepos,
            gfm_quirks: self.gfm_quirks,
            figure_with_caption: self.figure_with_caption,
            ol_width: self.ol_width,
            escaped_char_spans: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = ClmdOptions::default();
        assert_eq!(opts.input_format, InputFormat::CommonMark);
        assert_eq!(opts.output_format, OutputFormat::Html);
        assert!(!opts.standalone);
        assert!(!opts.smart);
        assert_eq!(opts.tab_stop, 4);
        assert_eq!(opts.columns, 80);
    }

    #[test]
    fn test_builder_methods() {
        let opts = ClmdOptions::new()
            .with_input_format(InputFormat::Gfm)
            .with_output_format(OutputFormat::Xml)
            .with_standalone(true)
            .with_smart(true)
            .with_columns(100);

        assert_eq!(opts.input_format, InputFormat::Gfm);
        assert_eq!(opts.output_format, OutputFormat::Xml);
        assert!(opts.standalone);
        assert!(opts.smart);
        assert_eq!(opts.columns, 100);
    }

    #[test]
    fn test_gfm_preset() {
        let opts = ClmdOptions::gfm();
        assert_eq!(opts.input_format, InputFormat::Gfm);
        assert_eq!(opts.output_format, OutputFormat::Html);
        assert!(opts.github_pre_lang);
    }

    #[test]
    fn test_commonmark_strict_preset() {
        let opts = ClmdOptions::commonmark_strict();
        assert_eq!(opts.input_format, InputFormat::CommonMark);
        assert_eq!(opts.output_format, OutputFormat::CommonMark);
    }

    #[test]
    fn test_input_format_from_str() {
        assert_eq!(
            "commonmark".parse::<InputFormat>().unwrap(),
            InputFormat::CommonMark
        );
        assert_eq!("gfm".parse::<InputFormat>().unwrap(), InputFormat::Gfm);
        assert_eq!("html".parse::<InputFormat>().unwrap(), InputFormat::Html);
        assert!("unknown".parse::<InputFormat>().is_err());
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("html".parse::<OutputFormat>().unwrap(), OutputFormat::Html);
        assert_eq!(
            "commonmark".parse::<OutputFormat>().unwrap(),
            OutputFormat::CommonMark
        );
        assert_eq!("pdf".parse::<OutputFormat>().unwrap(), OutputFormat::Pdf);
        assert!("unknown".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_format_from_extension() {
        assert_eq!(
            InputFormat::from_extension("md"),
            Some(InputFormat::Markdown)
        );
        assert_eq!(InputFormat::from_extension("html"), Some(InputFormat::Html));
        assert_eq!(InputFormat::from_extension("unknown"), None);

        assert_eq!(
            OutputFormat::from_extension("html"),
            Some(OutputFormat::Html)
        );
        assert_eq!(
            OutputFormat::from_extension("tex"),
            Some(OutputFormat::Latex)
        );
        assert_eq!(OutputFormat::from_extension("pdf"), Some(OutputFormat::Pdf));
    }

    #[test]
    fn test_resource_path_builder() {
        let opts = ClmdOptions::new()
            .add_resource_path("/path1")
            .add_resource_path("/path2");

        assert_eq!(opts.resource_path.len(), 2);
        assert_eq!(opts.resource_path[0], PathBuf::from("/path1"));
        assert_eq!(opts.resource_path[1], PathBuf::from("/path2"));
    }

    #[test]
    fn test_template_variables() {
        let opts = ClmdOptions::new()
            .with_variable("title", "My Title")
            .with_variable("author", "John Doe");

        assert_eq!(opts.variables.get("title"), Some(&"My Title".to_string()));
        assert_eq!(opts.variables.get("author"), Some(&"John Doe".to_string()));
    }

    #[test]
    fn test_metadata_builder() {
        let opts = ClmdOptions::new()
            .with_title("Test Title")
            .with_author("Test Author")
            .with_date("2024-01-01");

        assert_eq!(opts.title, Some("Test Title".to_string()));
        assert_eq!(opts.author, Some("Test Author".to_string()));
        assert_eq!(opts.date, Some("2024-01-01".to_string()));
    }

    #[test]
    fn test_html_math_method() {
        let opts = ClmdOptions::new().with_html_math(HTMLMathMethod::MathJax(None));
        assert_eq!(opts.html_math_method, HTMLMathMethod::MathJax(None));
    }

    #[test]
    fn test_output_format_is_binary() {
        assert!(OutputFormat::Pdf.is_binary());
        assert!(!OutputFormat::Html.is_binary());
        assert!(!OutputFormat::CommonMark.is_binary());
    }

    #[test]
    fn test_output_format_requires_standalone() {
        assert!(OutputFormat::Pdf.requires_standalone());
        assert!(OutputFormat::Latex.requires_standalone());
        assert!(!OutputFormat::Html.requires_standalone());
    }
}
