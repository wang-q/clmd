//! Configuration for the parser and renderer. Extensions affect both.
//!
//! This module provides a comrak-style Options API for configuring
//! Markdown parsing and rendering behavior.
//!
//! # Example
//!
//! ```ignore
//! use clmd::Options;
//!
//! let mut options = Options::default();
//! options.extension.table = true;
//! options.extension.strikethrough = true;
//! options.render.hardbreaks = true;
//! ```

use crate::core::adapters::{
    CodefenceRendererAdapter, HeadingAdapter, SyntaxHighlighterAdapter,
};
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use arbitrary::Arbitrary;
use bon::Builder;

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

/// Options for document readers.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReaderOptions {
    /// The input format.
    pub format: InputFormat,
    /// Whether to parse smart punctuation.
    pub smart: bool,
    /// Whether to enable extensions.
    pub extensions: crate::ext::flags::ExtensionFlags,
}

impl ReaderOptions {
    /// Convert to parser Options.
    pub fn to_parser_options(&self) -> Options<'_> {
        Options::default()
    }
}

/// Options for document writers.
#[derive(Debug, Clone, Copy, Default)]
pub struct WriterOptions {
    /// The output format.
    pub format: OutputFormat,
    /// Whether to wrap text.
    pub wrap: WrapOption,
    /// The wrap width.
    pub width: usize,
    /// Whether to enable extensions.
    pub extensions: crate::ext::flags::ExtensionFlags,
    /// Whether to output source positions.
    pub output_sourcepos: bool,
}

impl WriterOptions {
    /// Convert to render Options.
    pub fn to_render_options(&self) -> Options<'_> {
        Options::default()
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

/// Umbrella options struct for the Markdown parser and renderer.
///
/// This struct provides a convenient way to configure all aspects of
/// Markdown parsing and rendering.
///
/// The lifetime parameter `'c` allows options to hold references to
/// external data such as URL rewriters and broken link callbacks.
///
/// # Example
///
/// ```ignore
/// use clmd::Options;
///
/// let mut options = Options::default();
/// options.extension.table = true;
/// options.extension.strikethrough = true;
///
/// let html = clmd::markdown_to_html("Hello **world**!", &options);
/// assert!(html.contains("<strong>world</strong>"));
/// ```ignore
#[derive(Debug, Clone, Builder, Arbitrary, Default)]
pub struct Options<'c> {
    /// Enable CommonMark extensions.
    pub extension: Extension<'c>,

    /// Configure parse-time options.
    pub parse: Parse<'c>,

    /// Configure render-time options.
    pub render: Render,
}

impl<'c> Options<'c> {
    /// Create a new options struct with default values.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Options to select extensions.
///
/// Extensions affect both parsing and rendering.
///
/// The lifetime parameter `'c` allows extensions to hold references to
/// external data such as URL rewriters.
#[derive(Clone, Builder, Arbitrary, Default)]
pub struct Extension<'c> {
    /// Enables the strikethrough extension from the GFM spec.
    ///
    /// ```ignore
    /// use clmd::{markdown_to_html, Options};
    ///
    /// let mut options = Options::default();
    /// options.extension.strikethrough = true;
    /// let html = markdown_to_html("Hello ~~world~~ there.\n", &options);
    /// assert!(html.contains("<del>world</del>"));
    /// ```
    pub strikethrough: bool,

    /// Enables the tagfilter extension from the GFM spec.
    pub tagfilter: bool,

    /// Enables the table extension from the GFM spec.
    ///
    /// ```ignore
    /// use clmd::{markdown_to_html, Options};
    ///
    /// let mut options = Options::default();
    /// options.extension.table = true;
    /// let html = markdown_to_html("| a | b |\n|---|---|\n| c | d |\n", &options);
    /// assert!(html.contains("<table>"));
    /// ```
    pub table: bool,

    /// Enables the autolink extension from the GFM spec.
    pub autolink: bool,

    /// Enables the task list items extension from the GFM spec.
    pub tasklist: bool,

    /// Enables superscript text using `^` delimiters.
    pub superscript: bool,

    /// Enables subscript text using `~` delimiters.
    ///
    /// Note: If strikethrough is also enabled, this overrides the single
    /// tilde case to output subscript text.
    pub subscript: bool,

    /// Enables header IDs.
    ///
    /// When set to Some(prefix), adds IDs to headers based on their content.
    /// The prefix is prepended to the generated ID.
    pub header_ids: Option<String>,

    /// Enables the footnotes extension.
    pub footnotes: bool,

    /// Enables inline footnotes.
    ///
    /// Allows inline footnote syntax `^[content]` where the content can include
    /// inline markup. Inline footnotes are automatically converted to regular
    /// footnotes with auto-generated names.
    ///
    /// Requires `footnotes` to be enabled as well.
    pub inline_footnotes: bool,

    /// Enables the description lists extension.
    pub description_lists: bool,

    /// Enables the front matter extension.
    ///
    /// When set to Some(delimiter), allows YAML front matter at the
    /// beginning of the document.
    pub front_matter_delimiter: Option<String>,

    /// Enables the multiline block quote extension.
    pub multiline_block_quotes: bool,

    /// Enables GitHub style alerts.
    pub alerts: bool,

    /// Enables math using dollar syntax.
    pub math_dollars: bool,

    /// Enables math using code syntax.
    pub math_code: bool,

    /// Enables wikilinks using title after pipe syntax.
    pub wikilinks_title_after_pipe: bool,

    /// Enables wikilinks using title before pipe syntax.
    pub wikilinks_title_before_pipe: bool,

    /// Enables underlines using double underscores.
    pub underline: bool,

    /// Enables spoilers using double vertical bars.
    pub spoiler: bool,

    /// Requires a space after `>` for blockquotes.
    pub greentext: bool,

    /// Enables highlighting (mark) using `==`.
    pub highlight: bool,

    /// Enables inserted text using `++`.
    pub insert: bool,

    /// Recognizes many emphasis that appear in CJK contexts.
    ///
    /// This enables emphasis patterns that are common in CJK text but
    /// not recognized by plain CommonMark.
    pub cjk_friendly_emphasis: bool,

    /// Enables block scoped subscript that acts similar to a header.
    ///
    /// ```markdown
    /// -# subtext
    /// ```
    pub subtext: bool,

    /// Enables shortcodes for emoji (e.g., `:thumbsup:` -> 👍).
    pub shortcodes: bool,

    /// Wraps embedded image URLs using a function or custom trait object.
    #[arbitrary(default)]
    pub image_url_rewriter: Option<Arc<dyn URLRewriter + 'c>>,

    /// Wraps link URLs using a function or custom trait object.
    #[arbitrary(default)]
    pub link_url_rewriter: Option<Arc<dyn URLRewriter + 'c>>,
}

impl<'c> Debug for Extension<'c> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Extension")
            .field("strikethrough", &self.strikethrough)
            .field("tagfilter", &self.tagfilter)
            .field("table", &self.table)
            .field("autolink", &self.autolink)
            .field("tasklist", &self.tasklist)
            .field("superscript", &self.superscript)
            .field("subscript", &self.subscript)
            .field("header_ids", &self.header_ids)
            .field("footnotes", &self.footnotes)
            .field("inline_footnotes", &self.inline_footnotes)
            .field("description_lists", &self.description_lists)
            .field("front_matter_delimiter", &self.front_matter_delimiter)
            .field("multiline_block_quotes", &self.multiline_block_quotes)
            .field("alerts", &self.alerts)
            .field("math_dollars", &self.math_dollars)
            .field("math_code", &self.math_code)
            .field(
                "wikilinks_title_after_pipe",
                &self.wikilinks_title_after_pipe,
            )
            .field(
                "wikilinks_title_before_pipe",
                &self.wikilinks_title_before_pipe,
            )
            .field("underline", &self.underline)
            .field("spoiler", &self.spoiler)
            .field("greentext", &self.greentext)
            .field("highlight", &self.highlight)
            .field("insert", &self.insert)
            .field("cjk_friendly_emphasis", &self.cjk_friendly_emphasis)
            .field("subtext", &self.subtext)
            .field("image_url_rewriter", &"<dyn URLRewriter>")
            .field("link_url_rewriter", &"<dyn URLRewriter>")
            .finish()
    }
}

impl<'c> Extension<'c> {
    /// Returns the wikilinks mode if either wikilinks option is enabled.
    pub fn wikilinks(&self) -> Option<WikiLinksMode> {
        match (
            self.wikilinks_title_before_pipe,
            self.wikilinks_title_after_pipe,
        ) {
            (false, false) => None,
            (true, false) => Some(WikiLinksMode::TitleFirst),
            (_, _) => Some(WikiLinksMode::UrlFirst),
        }
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

/// Options for parser functions.
///
/// The lifetime parameter `'c` allows parse options to hold references to
/// external data such as broken link callbacks.
#[derive(Clone, Builder, Arbitrary, Default)]
pub struct Parse<'c> {
    /// Punctuation (quotes, full-stops and hyphens) are converted into 'smart' punctuation.
    ///
    /// ```
    /// use clmd::{markdown_to_html, Options};
    ///
    /// let mut options = Options::default();
    /// let input = "'Hello,' \"world\" ...";
    ///
    /// let html = markdown_to_html(input, &options);
    /// // Without smart: <p>'Hello,' &quot;world&quot; ...</p>
    ///
    /// options.parse.smart = true;
    /// let html = markdown_to_html(input, &options);
    /// // With smart: <p>'Hello,' "world" …</p>
    /// ```
    pub smart: bool,

    /// Include a `data-sourcepos` attribute on all block elements.
    pub sourcepos: bool,

    /// Validate UTF-8 in the input before parsing.
    pub validate_utf8: bool,

    /// The default info string for fenced code blocks.
    pub default_info_string: Option<String>,

    /// Relax tasklist matching to allow any symbol in brackets.
    pub relaxed_tasklist_matching: bool,

    /// Ignore setext headings in input.
    pub ignore_setext: bool,

    /// Leave footnote definitions in place in the document tree.
    pub leave_footnote_definitions: bool,

    /// Whether tasklist items can be parsed in table cells.
    ///
    /// At present, the tasklist item must be the only content in the cell.
    /// Both tables and tasklists must be enabled for this to work.
    pub tasklist_in_table: bool,

    /// Relax parsing of autolinks.
    ///
    /// Allows links to be detected inside brackets and allow all URL schemes.
    /// Intended to allow specific autolink detection patterns like
    /// `[this http://and.com that]` or `{http://foo.com}`.
    pub relaxed_autolinks: bool,

    /// Leave escaped characters in an `Escaped` node in the document tree.
    pub escaped_char_spans: bool,

    /// Callback for resolving broken link references.
    ///
    /// When the parser encounters a potential link that has a broken reference
    /// (e.g `[foo]` when there is no `[foo]: url` entry), this callback is called
    /// to potentially resolve the reference.
    #[arbitrary(default)]
    pub broken_link_callback: Option<Arc<dyn BrokenLinkCallback + 'c>>,
}

impl<'c> Debug for Parse<'c> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parse")
            .field("smart", &self.smart)
            .field("sourcepos", &self.sourcepos)
            .field("validate_utf8", &self.validate_utf8)
            .field("default_info_string", &self.default_info_string)
            .field("relaxed_tasklist_matching", &self.relaxed_tasklist_matching)
            .field("ignore_setext", &self.ignore_setext)
            .field(
                "leave_footnote_definitions",
                &self.leave_footnote_definitions,
            )
            .field("tasklist_in_table", &self.tasklist_in_table)
            .field("relaxed_autolinks", &self.relaxed_autolinks)
            .field("escaped_char_spans", &self.escaped_char_spans)
            .field("broken_link_callback", &"<dyn BrokenLinkCallback>")
            .finish()
    }
}

/// Options for formatter functions.
#[derive(Debug, Clone, Copy, Builder, Arbitrary, Default)]
pub struct Render {
    /// Soft line breaks in the input translate into hard line breaks in the output.
    ///
    /// ```ignore
    /// use clmd::{markdown_to_html, Options};
    ///
    /// let mut options = Options::default();
    /// let input = "Hello.\nWorld.\n";
    ///
    /// let html = markdown_to_html(input, &options);
    /// assert!(html.contains("Hello.\nWorld."));
    ///
    /// options.render.hardbreaks = true;
    /// let html = markdown_to_html(input, &options);
    /// assert!(html.contains("<br"));
    /// ```
    pub hardbreaks: bool,

    /// Soft line breaks in the input translate into spaces.
    pub nobreaks: bool,

    /// Allow rendering of raw HTML and potentially dangerous links.
    ///
    /// # Security Warning
    ///
    /// Only enable this option if you trust the input completely.
    /// Rendering untrusted user input with this option enabled can
    /// lead to XSS (Cross-Site Scripting) attacks.
    pub r#unsafe: bool,

    /// Escape raw HTML instead of removing it.
    pub escape: bool,

    /// GitHub-style `<pre lang="xyz">` for fenced code blocks.
    pub github_pre_lang: bool,

    /// Enable full info strings for code blocks.
    pub full_info_string: bool,

    /// The wrap column when outputting CommonMark.
    /// A value of 0 disables wrapping.
    pub width: usize,

    /// List style type for bullet lists.
    pub list_style: ListStyleType,

    /// Prefer fenced code blocks when outputting CommonMark.
    pub prefer_fenced: bool,

    /// Ignore empty links in input.
    pub ignore_empty_links: bool,

    /// Add classes to tasklist output.
    pub tasklist_classes: bool,

    /// Compact HTML output (no newlines between block elements).
    pub compact_html: bool,

    /// Include source position attributes in HTML and XML output.
    ///
    /// Sourcepos information is reliable for core block items excluding
    /// lists and list items, all inlines, and most extensions.
    pub sourcepos: bool,

    /// Enables GFM quirks in HTML output which break CommonMark compatibility.
    ///
    /// This changes how nested emphasis is rendered to match GitHub's behavior.
    pub gfm_quirks: bool,

    /// Render the image as a figure element with the title as its caption.
    pub figure_with_caption: bool,

    /// Render ordered list with a minimum marker width.
    /// Having a width lower than 3 doesn't do anything.
    pub ol_width: usize,

    /// Wrap escaped characters in a `<span>` to allow any
    /// post-processing to recognize them.
    ///
    /// Note that enabling this option will cause the `escaped_char_spans`
    /// parse option to be enabled.
    pub escaped_char_spans: bool,
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

/// Trait for link and image URL rewrite extensions.
pub trait URLRewriter: Send + Sync {
    /// Converts the given URL from Markdown to its representation when output as HTML.
    fn rewrite(&self, url: &str) -> String;
}

impl Debug for dyn URLRewriter + '_ {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        formatter.write_str("<dyn URLRewriter>")
    }
}

impl<F> URLRewriter for F
where
    F: Fn(&str) -> String + Send + Sync,
{
    fn rewrite(&self, url: &str) -> String {
        self(url)
    }
}

/// The type of the callback used when a reference link is encountered with no
/// matching reference.
///
/// The details of the broken reference are passed in the
/// [`BrokenLinkReference`] argument. If a [`ResolvedReference`] is returned, it
/// is used as the link; otherwise, no link is made and the reference text is
/// preserved in its entirety.
pub trait BrokenLinkCallback: Send + Sync {
    /// Potentially resolve a single broken link reference.
    fn resolve(
        &self,
        broken_link_reference: BrokenLinkReference,
    ) -> Option<ResolvedReference>;
}

impl Debug for dyn BrokenLinkCallback + '_ {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        formatter.write_str("<dyn BrokenLinkCallback>")
    }
}

impl<F> BrokenLinkCallback for F
where
    F: Fn(BrokenLinkReference) -> Option<ResolvedReference> + Send + Sync,
{
    fn resolve(
        &self,
        broken_link_reference: BrokenLinkReference,
    ) -> Option<ResolvedReference> {
        self(broken_link_reference)
    }
}

/// Struct to the broken link callback, containing details on the link reference
/// which failed to find a match.
#[derive(Debug)]
pub struct BrokenLinkReference<'l> {
    /// The normalized reference link label. Unicode case folding is applied.
    pub normalized: &'l str,

    /// The original text in the link label.
    pub original: &'l str,
}

/// A reference link's resolved details.
#[derive(Clone, Debug)]
pub struct ResolvedReference {
    /// The destination URL of the reference link.
    pub url: String,

    /// The text of the link.
    pub title: String,
}

/// Umbrella plugins struct.
#[derive(Default, Clone, Debug)]
pub struct Plugins<'p> {
    /// Configure render-time plugins.
    pub render: RenderPlugins<'p>,
}

impl<'p> Plugins<'p> {
    /// Create a new empty plugins collection
    pub fn new() -> Self {
        Self::default()
    }
}

/// Plugins for alternative rendering.
#[derive(Default, Clone)]
pub struct RenderPlugins<'p> {
    /// Provide language-specific renderers for codefence blocks.
    ///
    /// `math` codefence blocks are handled separately by the built-in math renderer,
    /// so entries keyed by `"math"` in this map are not used.
    pub codefence_renderers: HashMap<String, &'p dyn CodefenceRendererAdapter>,

    /// Provide a syntax highlighter adapter implementation for syntax
    /// highlighting of codefence blocks.
    pub codefence_syntax_highlighter: Option<&'p dyn SyntaxHighlighterAdapter>,

    /// Optional heading adapter
    pub heading_adapter: Option<&'p dyn HeadingAdapter>,
}

impl<'p> RenderPlugins<'p> {
    /// Create a new empty render plugins collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a code fence renderer for a specific language
    pub fn register_codefence_renderer(
        &mut self,
        language: impl Into<String>,
        renderer: &'p dyn CodefenceRendererAdapter,
    ) {
        self.codefence_renderers.insert(language.into(), renderer);
    }

    /// Get a code fence renderer for a specific language
    pub fn codefence_renderer(
        &self,
        language: &str,
    ) -> Option<&dyn CodefenceRendererAdapter> {
        self.codefence_renderers.get(language).copied()
    }

    /// Set the syntax highlighter
    pub fn set_syntax_highlighter(&mut self, adapter: &'p dyn SyntaxHighlighterAdapter) {
        self.codefence_syntax_highlighter = Some(adapter);
    }

    /// Get the syntax highlighter if set
    pub fn syntax_highlighter(&self) -> Option<&dyn SyntaxHighlighterAdapter> {
        self.codefence_syntax_highlighter
    }

    /// Set the heading adapter
    pub fn set_heading_adapter(&mut self, adapter: &'p dyn HeadingAdapter) {
        self.heading_adapter = Some(adapter);
    }

    /// Get the heading adapter if set
    pub fn heading_adapter(&self) -> Option<&dyn HeadingAdapter> {
        self.heading_adapter
    }

    /// Check if any plugins are registered
    pub fn is_empty(&self) -> bool {
        self.codefence_renderers.is_empty()
            && self.codefence_syntax_highlighter.is_none()
            && self.heading_adapter.is_none()
    }
}

impl Debug for RenderPlugins<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderPlugins")
            .field(
                "codefence_renderers",
                &self.codefence_renderers.keys().collect::<Vec<_>>(),
            )
            .field(
                "has_syntax_highlighter",
                &self.codefence_syntax_highlighter.is_some(),
            )
            .field("has_heading_adapter", &self.heading_adapter.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = Options::default();
        assert!(!options.extension.strikethrough);
        assert!(!options.extension.table);
        assert!(!options.parse.smart);
        assert!(!options.render.hardbreaks);
    }

    #[test]
    fn test_options_new() {
        let options = Options::new();
        assert!(!options.extension.strikethrough);
        assert!(!options.extension.table);
    }

    #[test]
    fn test_extension_wikilinks() {
        let mut ext = Extension::default();
        assert_eq!(ext.wikilinks(), None);

        ext.wikilinks_title_before_pipe = true;
        assert_eq!(ext.wikilinks(), Some(WikiLinksMode::TitleFirst));

        ext.wikilinks_title_after_pipe = true;
        assert_eq!(ext.wikilinks(), Some(WikiLinksMode::UrlFirst));
    }

    #[test]
    fn test_list_style_type_default() {
        let style: ListStyleType = Default::default();
        assert_eq!(style, ListStyleType::Dash);
    }
}
