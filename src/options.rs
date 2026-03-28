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

use crate::config::DataHolder;

// Re-export DataKey-based options for backward compatibility
pub use crate::config::options as data_keys;

/// Umbrella options struct for the Markdown parser and renderer.
///
/// This struct provides a convenient way to configure all aspects of
/// Markdown parsing and rendering. It wraps the underlying DataKey-based
/// configuration system.
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
pub struct Options {
    /// Enable CommonMark extensions.
    pub extension: Extension,

    /// Configure parse-time options.
    pub parse: Parse,

    /// Configure render-time options.
    pub render: Render,
}

impl Options {
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

        opts
    }
}

/// Options to select extensions.
///
/// Extensions affect both parsing and rendering.
#[derive(Debug, Clone)]
pub struct Extension {
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
}

impl Default for Extension {
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
        }
    }
}

impl Extension {
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
#[derive(Debug, Clone)]
pub struct Parse {
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
}

impl Default for Parse {
    fn default() -> Self {
        Self {
            smart: false,
            sourcepos: false,
            validate_utf8: false,
            default_info_string: None,
            relaxed_tasklist_matching: false,
            ignore_setext: false,
            leave_footnote_definitions: false,
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
}
