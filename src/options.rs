//! Configuration for the parser and renderer. Extensions affect both.
//!
//! This module provides a Pandoc-style Options API with separate
//! ReaderOptions and WriterOptions for configuring Markdown parsing
//! and rendering behavior.
//!
//! # Example
//!
//! ```ignore
//! use clmd::options::{Options, ReaderOptions, WriterOptions, Extensions};
//!
//! let reader_opts = ReaderOptions {
//!     extensions: Extensions::gfm(),
//!     ..Default::default()
//! };
//!
//! let writer_opts = WriterOptions {
//!     hardbreaks: true,
//!     ..Default::default()
//! };
//!
//! let options = Options::new(reader_opts, writer_opts);
//! ```

pub use crate::parser::options::{
    BrokenLinkCallback, BrokenLinkReference, Extension, ListStyleType, Parse, Plugins,
    Render, RenderPlugins, ResolvedReference, URLRewriter, WikiLinksMode,
};

use crate::extensions::Extensions;
use std::collections::HashMap;
use std::path::PathBuf;

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

/// Options for parsing documents (similar to Pandoc's ReaderOptions).
#[derive(Debug, Clone)]
pub struct ReaderOptions {
    /// Input format.
    pub input_format: InputFormat,

    /// Syntax extensions to enable.
    pub extensions: Extensions,

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

    /// Resource paths for looking up files.
    pub resource_path: Vec<PathBuf>,

    /// User data directory.
    pub user_data_dir: Option<PathBuf>,

    /// Extract media to this directory.
    pub extract_media: Option<PathBuf>,

    /// List style type for bullet lists.
    pub list_style: ListStyleType,
}

impl Default for ReaderOptions {
    fn default() -> Self {
        Self {
            input_format: InputFormat::default(),
            extensions: Extensions::empty(),
            smart: false,
            sourcepos: false,
            tab_stop: 4,
            validate_utf8: true,
            default_info_string: None,
            resource_path: Vec::new(),
            user_data_dir: None,
            extract_media: None,
            list_style: ListStyleType::default(),
        }
    }
}

impl ReaderOptions {
    /// Create new options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create options for GFM input.
    pub fn gfm() -> Self {
        Self {
            input_format: InputFormat::Gfm,
            extensions: Extensions::gfm(),
            ..Default::default()
        }
    }

    /// Create options for CommonMark strict mode.
    pub fn commonmark_strict() -> Self {
        Self {
            input_format: InputFormat::CommonMark,
            extensions: Extensions::empty(),
            ..Default::default()
        }
    }

    /// Set the input format.
    pub fn with_input_format(mut self, format: InputFormat) -> Self {
        self.input_format = format;
        // Auto-enable default extensions for the format
        if self.extensions.is_empty() {
            self.extensions = format.default_extensions();
        }
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

    /// Add a resource path.
    pub fn add_resource_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.resource_path.push(path.into());
        self
    }

    /// Convert to parser Options for internal use.
    pub fn to_parser_options(&self) -> crate::parser::options::Options<'_> {
        use crate::parser::options::{Extension, Parse, Render};

        crate::parser::options::Options {
            extension: Extension {
                strikethrough: self.extensions.contains(Extensions::STRIKETHROUGH),
                tagfilter: self.extensions.contains(Extensions::TAGFILTER),
                table: self.extensions.contains(Extensions::TABLES),
                autolink: self.extensions.contains(Extensions::AUTOLINKS),
                tasklist: self.extensions.contains(Extensions::TASKLISTS),
                superscript: self.extensions.contains(Extensions::SUPERSCRIPT),
                subscript: self.extensions.contains(Extensions::SUBSCRIPT),
                header_ids: None,
                footnotes: self.extensions.contains(Extensions::FOOTNOTES),
                inline_footnotes: self.extensions.contains(Extensions::INLINE_FOOTNOTES),
                description_lists: self
                    .extensions
                    .contains(Extensions::DESCRIPTION_LISTS),
                front_matter_delimiter: None,
                multiline_block_quotes: self
                    .extensions
                    .contains(Extensions::MULTILINE_BLOCK_QUOTES),
                alerts: self.extensions.contains(Extensions::ALERTS),
                math_dollars: self.extensions.contains(Extensions::MATH_DOLLARS),
                math_code: self.extensions.contains(Extensions::MATH_CODE),
                wikilinks_title_after_pipe: false,
                wikilinks_title_before_pipe: false,
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
            },
            parse: Parse {
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
            },
            render: Render {
                hardbreaks: false,
                nobreaks: false,
                r#unsafe: false,
                escape: false,
                github_pre_lang: false,
                full_info_string: false,
                width: 0,
                list_style: self.list_style,
                prefer_fenced: false,
                ignore_empty_links: false,
                tasklist_classes: false,
                compact_html: false,
                sourcepos: self.sourcepos,
                gfm_quirks: false,
                figure_with_caption: false,
                ol_width: 3,
                escaped_char_spans: false,
            },
        }
    }
}

/// Options for rendering documents (similar to Pandoc's WriterOptions).
#[derive(Debug, Clone)]
pub struct WriterOptions {
    /// Output format.
    pub output_format: OutputFormat,

    /// Standalone document (include header/footer).
    pub standalone: bool,

    /// Syntax extensions to enable.
    pub extensions: Extensions,

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

    /// Template file path.
    pub template: Option<PathBuf>,

    /// Template variables.
    pub variables: HashMap<String, String>,

    /// Document title (overrides metadata).
    pub title: Option<String>,

    /// Document author (overrides metadata).
    pub author: Option<String>,

    /// Document date (overrides metadata).
    pub date: Option<String>,
}

impl Default for WriterOptions {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::default(),
            standalone: false,
            extensions: Extensions::empty(),
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
            template: None,
            variables: HashMap::new(),
            title: None,
            author: None,
            date: None,
        }
    }
}

impl WriterOptions {
    /// Create new options with default values.
    pub fn new() -> Self {
        Self::default()
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
}

/// Unified options structure combining ReaderOptions and WriterOptions.
///
/// This is the main entry point for configuring clmd operations,
/// similar to how Pandoc combines ReaderOptions and WriterOptions.
#[derive(Debug, Clone)]
pub struct Options {
    /// Reader (parsing) options.
    pub reader: ReaderOptions,

    /// Writer (rendering) options.
    pub writer: WriterOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            reader: ReaderOptions::default(),
            writer: WriterOptions::default(),
        }
    }
}

impl Options {
    /// Create new options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create options from reader and writer options.
    pub fn from_parts(reader: ReaderOptions, writer: WriterOptions) -> Self {
        Self { reader, writer }
    }

    /// Create options for GFM input/output.
    pub fn gfm() -> Self {
        Self {
            reader: ReaderOptions::gfm(),
            writer: WriterOptions::default()
                .with_output_format(OutputFormat::Html)
                .with_extension(Extensions::gfm()),
        }
    }

    /// Create options for CommonMark strict mode.
    pub fn commonmark_strict() -> Self {
        Self {
            reader: ReaderOptions::commonmark_strict(),
            writer: WriterOptions::default()
                .with_output_format(OutputFormat::CommonMark),
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

    /// Set input format (convenience method).
    pub fn with_input_format(mut self, format: InputFormat) -> Self {
        self.reader = self.reader.with_input_format(format);
        self
    }

    /// Set output format (convenience method).
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.writer = self.writer.with_output_format(format);
        self
    }

    /// Set extensions for both reader and writer.
    pub fn with_extensions(mut self, extensions: Extensions) -> Self {
        self.reader.extensions = extensions;
        self.writer.extensions = extensions;
        self
    }

    /// Enable an extension for both reader and writer.
    pub fn with_extension(mut self, extension: Extensions) -> Self {
        self.reader.extensions |= extension;
        self.writer.extensions |= extension;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_options_default() {
        let opts = ReaderOptions::default();
        assert_eq!(opts.input_format, InputFormat::CommonMark);
        assert!(!opts.smart);
        assert_eq!(opts.tab_stop, 4);
    }

    #[test]
    fn test_writer_options_default() {
        let opts = WriterOptions::default();
        assert_eq!(opts.output_format, OutputFormat::Html);
        assert!(!opts.hardbreaks);
        assert_eq!(opts.columns, 80);
    }

    #[test]
    fn test_options_default() {
        let opts = Options::default();
        assert_eq!(opts.reader.input_format, InputFormat::CommonMark);
        assert_eq!(opts.writer.output_format, OutputFormat::Html);
    }

    #[test]
    fn test_reader_builder() {
        let opts = ReaderOptions::new()
            .with_input_format(InputFormat::Gfm)
            .with_smart(true)
            .with_tab_stop(2);

        assert_eq!(opts.input_format, InputFormat::Gfm);
        assert!(opts.smart);
        assert_eq!(opts.tab_stop, 2);
    }

    #[test]
    fn test_writer_builder() {
        let opts = WriterOptions::new()
            .with_output_format(OutputFormat::Xml)
            .with_hardbreaks(true)
            .with_columns(100);

        assert_eq!(opts.output_format, OutputFormat::Xml);
        assert!(opts.hardbreaks);
        assert_eq!(opts.columns, 100);
    }

    #[test]
    fn test_options_builder() {
        let opts = Options::new()
            .with_input_format(InputFormat::Gfm)
            .with_output_format(OutputFormat::Xml)
            .with_extension(Extensions::TABLES);

        assert_eq!(opts.reader.input_format, InputFormat::Gfm);
        assert_eq!(opts.writer.output_format, OutputFormat::Xml);
        assert!(opts.reader.extensions.contains(Extensions::TABLES));
        assert!(opts.writer.extensions.contains(Extensions::TABLES));
    }

    #[test]
    fn test_gfm_preset() {
        let opts = Options::gfm();
        assert_eq!(opts.reader.input_format, InputFormat::Gfm);
        assert_eq!(opts.writer.output_format, OutputFormat::Html);
        assert!(opts.reader.extensions.contains(Extensions::TABLES));
    }

    #[test]
    fn test_commonmark_strict_preset() {
        let opts = Options::commonmark_strict();
        assert_eq!(opts.reader.input_format, InputFormat::CommonMark);
        assert_eq!(opts.writer.output_format, OutputFormat::CommonMark);
        assert!(opts.reader.extensions.is_empty());
    }
}
