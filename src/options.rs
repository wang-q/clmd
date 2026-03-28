//! Options for the Markdown parser and renderer
//!
//! This module provides a comrak-style Options API that wraps the underlying
//! DataKey-based configuration system. It offers better ergonomics and
//! compile-time type safety.
//!
//! # Example
//!
//! ```
//! use clmd::options::{Options, Extension, Parse, Render};
//!
//! let mut options = Options::default();
//! options.extension.table = true;
//! options.extension.strikethrough = true;
//! options.render.hardbreaks = true;
//!
//! let html = clmd::markdown_to_html("| a | b |\n|---|---|\n| c | d |", &options);
//! ```

use std::sync::Arc;

// Re-export DataKey-based options for backward compatibility
pub use crate::config::options as data_keys;

/// Umbrella options struct for the Markdown parser and renderer.
///
/// This struct provides a convenient way to configure all aspects of
/// Markdown parsing and rendering. It wraps the underlying DataKey-based
/// configuration system.
///
/// The lifetime parameter `'c` allows options to hold references to
/// external data such as URL rewriters and broken link callbacks.
///
/// # Example
///
/// ```
/// use clmd::options::Options;
///
/// let mut options = Options::default();
/// options.extension.table = true;
/// options.extension.strikethrough = true;
///
/// // Note: Extension processing requires inline parsing
/// // which will be integrated in a future update
/// let html = clmd::markdown_to_html("Hello **world**!", &options);
/// assert!(html.contains("<strong>"));
/// ```
#[derive(Debug, Clone, Default)]
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

    /// Convert to the underlying DataKey-based Options.
    ///
    /// This is useful when you need to use the lower-level API.
    pub fn to_data_key_options(&self) -> crate::config::Options {
        let mut opts = crate::config::Options::new();

        // Extension options
        opts.set(&data_keys::ENABLE_TABLES, self.extension.table);
        opts.set(&data_keys::ENABLE_STRIKETHROUGH, self.extension.strikethrough);
        opts.set(&data_keys::ENABLE_FOOTNOTES, self.extension.footnotes);
        opts.set(&data_keys::ENABLE_TASKLISTS, self.extension.tasklist);
        opts.set(&data_keys::ENABLE_AUTOLINKS, self.extension.autolink);
        opts.set(&data_keys::ENABLE_TAGFILTER, self.extension.tagfilter);
        opts.set(&data_keys::ENABLE_SUPERSCRIPT, self.extension.superscript);
        opts.set(&data_keys::ENABLE_SUBSCRIPT, self.extension.subscript);
        opts.set(&data_keys::ENABLE_DESCRIPTION_LISTS, self.extension.description_lists);
        opts.set(&data_keys::ENABLE_MULTILINE_BLOCK_QUOTES, self.extension.multiline_block_quotes);
        opts.set(&data_keys::ENABLE_ALERTS, self.extension.alerts);
        opts.set(&data_keys::ENABLE_MATH_DOLLARS, self.extension.math_dollars);
        opts.set(&data_keys::ENABLE_MATH_CODE, self.extension.math_code);
        opts.set(&data_keys::ENABLE_WIKILINKS_TITLE_AFTER_PIPE, self.extension.wikilinks_title_after_pipe);
        opts.set(&data_keys::ENABLE_WIKILINKS_TITLE_BEFORE_PIPE, self.extension.wikilinks_title_before_pipe);
        opts.set(&data_keys::ENABLE_UNDERLINE, self.extension.underline);
        opts.set(&data_keys::ENABLE_SPOILER, self.extension.spoiler);
        opts.set(&data_keys::ENABLE_GREENTEXT, self.extension.greentext);
        opts.set(&data_keys::ENABLE_HIGHLIGHT, self.extension.highlight);
        opts.set(&data_keys::ENABLE_INSERT, self.extension.insert);
        opts.set(&data_keys::ENABLE_CJK_FRIENDLY_EMPHASIS, self.extension.cjk_friendly_emphasis);
        opts.set(&data_keys::ENABLE_SUBTEXT, self.extension.subtext);

        if let Some(ref prefix) = self.extension.header_ids {
            opts.set(&data_keys::HEADER_IDS, Some(prefix.clone()));
        }

        if let Some(ref delimiter) = self.extension.front_matter_delimiter {
            opts.set(&data_keys::FRONT_MATTER_DELIMITER, Some(delimiter.clone()));
        }

        // Parse options
        opts.set(&data_keys::SOURCEPOS, self.parse.sourcepos);
        opts.set(&data_keys::SMART, self.parse.smart);
        opts.set(&data_keys::VALIDATE_UTF8, self.parse.validate_utf8);
        opts.set(&data_keys::RELAXED_TASKLIST_MATCHING, self.parse.relaxed_tasklist_matching);
        opts.set(&data_keys::IGNORE_SETEXT, self.parse.ignore_setext);
        opts.set(&data_keys::LEAVE_FOOTNOTE_DEFINITIONS, self.parse.leave_footnote_definitions);
        opts.set(&data_keys::TASKLIST_IN_TABLE, self.parse.tasklist_in_table);
        opts.set(&data_keys::RELAXED_AUTOLINKS, self.parse.relaxed_autolinks);
        opts.set(&data_keys::ESCAPED_CHAR_SPANS, self.parse.escaped_char_spans);

        if let Some(ref info) = self.parse.default_info_string {
            opts.set(&data_keys::DEFAULT_INFO_STRING, Some(info.clone()));
        }

        // Render options
        opts.set(&data_keys::HARDBREAKS, self.render.hardbreaks);
        opts.set(&data_keys::NOBREAKS, self.render.nobreaks);
        opts.set(&data_keys::UNSAFE, self.render.r#unsafe);
        opts.set(&data_keys::ESCAPE, self.render.escape);
        opts.set(&data_keys::GITHUB_PRE_LANG, self.render.github_pre_lang);
        opts.set(&data_keys::FULL_INFO_STRING, self.render.full_info_string);
        opts.set(&data_keys::WRAP_WIDTH, self.render.width);
        opts.set(&data_keys::LIST_STYLE_TYPE, self.render.list_style.into());
        opts.set(&data_keys::PREFER_FENCED, self.render.prefer_fenced);
        opts.set(&data_keys::IGNORE_EMPTY_LINKS, self.render.ignore_empty_links);
        opts.set(&data_keys::TASKLIST_CLASSES, self.render.tasklist_classes);
        opts.set(&data_keys::COMPACT_HTML, self.render.compact_html);
        opts.set(&data_keys::SOURCEPOS_RENDER, self.render.sourcepos);
        opts.set(&data_keys::GFM_QUIRKS, self.render.gfm_quirks);
        opts.set(&data_keys::FIGURE_WITH_CAPTION, self.render.figure_with_caption);
        opts.set(&data_keys::OL_WIDTH, self.render.ol_width);
        opts.set(&data_keys::ESCAPED_CHAR_SPANS_RENDER, self.render.escaped_char_spans);

        opts
    }
}

/// Options to select extensions.
///
/// Extensions affect both parsing and rendering.
///
/// The lifetime parameter `'c` allows extensions to hold references to
/// external data such as URL rewriters.
#[derive(Debug, Clone)]
pub struct Extension<'c> {
    /// Enables the strikethrough extension from the GFM spec.
    ///
    /// Note: This extension requires inline parsing which will be
    /// integrated in a future update.
    ///
    /// ```rust,ignore
    /// use clmd::options::{Options, Extension};
    ///
    /// let mut options = Options::default();
    /// options.extension.strikethrough = true;
    /// let html = clmd::markdown_to_html("Hello ~world~ there.\n", &options);
    /// assert_eq!(html, "<p>Hello <del>world</del> there.</p>\n");
    /// ```
    pub strikethrough: bool,

    /// Enables the tagfilter extension from the GFM spec.
    pub tagfilter: bool,

    /// Enables the table extension from the GFM spec.
    ///
    /// Note: This extension requires block parsing support which will be
    /// integrated in a future update.
    ///
    /// ```rust,ignore
    /// use clmd::options::Options;
    ///
    /// let mut options = Options::default();
    /// options.extension.table = true;
    /// let html = clmd::markdown_to_html("| a | b |\n|---|---|\n| c | d |\n", &options);
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

    /// Wraps embedded image URLs using a function or custom trait object.
    pub image_url_rewriter: Option<Arc<dyn crate::adapters::UrlRewriter + 'c>>,

    /// Wraps link URLs using a function or custom trait object.
    pub link_url_rewriter: Option<Arc<dyn crate::adapters::UrlRewriter + 'c>>,
}

impl<'c> Default for Extension<'c> {
    fn default() -> Self {
        Self {
            strikethrough: false,
            tagfilter: false,
            table: false,
            autolink: false,
            tasklist: false,
            superscript: false,
            subscript: false,
            header_ids: None,
            footnotes: false,
            inline_footnotes: false,
            description_lists: false,
            front_matter_delimiter: None,
            multiline_block_quotes: false,
            alerts: false,
            math_dollars: false,
            math_code: false,
            wikilinks_title_after_pipe: false,
            wikilinks_title_before_pipe: false,
            underline: false,
            spoiler: false,
            greentext: false,
            highlight: false,
            insert: false,
            cjk_friendly_emphasis: false,
            subtext: false,
            image_url_rewriter: None,
            link_url_rewriter: None,
        }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone)]
pub struct Parse<'c> {
    /// Punctuation (quotes, full-stops and hyphens) are converted into 'smart' punctuation.
    ///
    /// ```rust
    /// use clmd::options::Options;
    ///
    /// let mut options = Options::default();
    /// let input = "'Hello,' \"world\" ...";
    ///
    /// let html = clmd::markdown_to_html(input, &options);
    /// // Without smart: <p>'Hello,' &quot;world&quot; ...</p>
    ///
    /// options.parse.smart = true;
    /// let html = clmd::markdown_to_html(input, &options);
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
    pub broken_link_callback: Option<Arc<dyn crate::adapters::BrokenLinkCallback + 'c>>,
}

impl<'c> Default for Parse<'c> {
    fn default() -> Self {
        Self {
            smart: false,
            sourcepos: false,
            validate_utf8: false,
            default_info_string: None,
            relaxed_tasklist_matching: false,
            ignore_setext: false,
            leave_footnote_definitions: false,
            tasklist_in_table: false,
            relaxed_autolinks: false,
            escaped_char_spans: false,
            broken_link_callback: None,
        }
    }
}

/// Options for formatter functions.
#[derive(Debug, Clone)]
pub struct Render {
    /// Soft line breaks in the input translate into hard line breaks in the output.
    ///
    /// Note: This option requires inline parsing which will be
    /// integrated in a future update.
    ///
    /// ```rust,ignore
    /// use clmd::options::Options;
    ///
    /// let mut options = Options::default();
    /// let input = "Hello.\nWorld.\n";
    ///
    /// let html = clmd::markdown_to_html(input, &options);
    /// assert_eq!(html, "<p>Hello.\nWorld.</p>\n");
    ///
    /// options.render.hardbreaks = true;
    /// let html = clmd::markdown_to_html(input, &options);
    /// assert_eq!(html, "<p>Hello.<br />\nWorld.</p>\n");
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

impl Default for Render {
    fn default() -> Self {
        Self {
            hardbreaks: false,
            nobreaks: false,
            r#unsafe: false,
            escape: false,
            github_pre_lang: false,
            full_info_string: false,
            width: 0,
            list_style: ListStyleType::default(),
            prefer_fenced: false,
            ignore_empty_links: false,
            tasklist_classes: false,
            compact_html: false,
            sourcepos: false,
            gfm_quirks: false,
            figure_with_caption: false,
            ol_width: 0,
            escaped_char_spans: false,
        }
    }
}

/// Style type for bullet lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListStyleType {
    /// Use `-` for bullet lists.
    #[default]
    Dash,
    /// Use `+` for bullet lists.
    Plus,
    /// Use `*` for bullet lists.
    Star,
}

impl From<ListStyleType> for crate::config::options::ListStyleType {
    fn from(style: ListStyleType) -> Self {
        match style {
            ListStyleType::Dash => crate::config::options::ListStyleType::Dash,
            ListStyleType::Plus => crate::config::options::ListStyleType::Plus,
            ListStyleType::Star => crate::config::options::ListStyleType::Star,
        }
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
    fn test_to_data_key_options() {
        let mut options = Options::default();
        options.extension.table = true;
        options.extension.strikethrough = true;
        options.parse.smart = true;
        options.render.hardbreaks = true;

        let data_opts = options.to_data_key_options();

        assert!(data_opts.get(&data_keys::ENABLE_TABLES));
        assert!(data_opts.get(&data_keys::ENABLE_STRIKETHROUGH));
        assert!(data_opts.get(&data_keys::SMART));
        assert!(data_opts.get(&data_keys::HARDBREAKS));
    }

    #[test]
    fn test_list_style_type_default() {
        let style: ListStyleType = Default::default();
        assert_eq!(style, ListStyleType::Dash);
    }

    #[test]
    fn test_list_style_type_conversion() {
        assert!(matches!(
            crate::config::options::ListStyleType::from(ListStyleType::Dash),
            crate::config::options::ListStyleType::Dash
        ));
        assert!(matches!(
            crate::config::options::ListStyleType::from(ListStyleType::Plus),
            crate::config::options::ListStyleType::Plus
        ));
        assert!(matches!(
            crate::config::options::ListStyleType::from(ListStyleType::Star),
            crate::config::options::ListStyleType::Star
        ));
    }

    #[test]
    fn test_new_render_options() {
        let render = Render::default();
        assert!(!render.sourcepos);
        assert!(!render.gfm_quirks);
        assert!(!render.figure_with_caption);
        assert_eq!(render.ol_width, 0);
        assert!(!render.escaped_char_spans);
    }

    #[test]
    fn test_new_parse_options() {
        let parse = Parse::default();
        assert!(!parse.tasklist_in_table);
        assert!(!parse.relaxed_autolinks);
        assert!(!parse.escaped_char_spans);
        assert!(parse.broken_link_callback.is_none());
    }

    #[test]
    fn test_new_extension_options() {
        let ext = Extension::default();
        assert!(!ext.inline_footnotes);
        assert!(!ext.cjk_friendly_emphasis);
        assert!(!ext.subtext);
        assert!(ext.image_url_rewriter.is_none());
        assert!(ext.link_url_rewriter.is_none());
    }
}
