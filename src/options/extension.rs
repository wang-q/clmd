//! Extension options for the Markdown parser.
//!
//! This module provides configuration options for Markdown extensions,
//! including GFM extensions and other syntax extensions.

use super::traits::URLRewriter;
use arbitrary::Arbitrary;
use bon::Builder;
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

/// Options to select extensions.
///
/// Extensions affect both parsing and rendering.
///
/// The lifetime parameter `'c` allows extensions to hold references to
/// external data such as URL rewriters.
#[derive(Clone, Builder, Arbitrary, Default)]
pub struct Extension<'c> {
    // =========================================================================
    // GFM Extensions
    // =========================================================================
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

    // =========================================================================
    // Syntax Extensions
    // =========================================================================
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

    // =========================================================================
    // Callbacks (require lifetime)
    // =========================================================================
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
            .field("shortcodes", &self.shortcodes)
            .field("image_url_rewriter", &"<dyn URLRewriter>")
            .field("link_url_rewriter", &"<dyn URLRewriter>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_default() {
        let ext = Extension::default();
        assert!(!ext.strikethrough);
        assert!(!ext.tagfilter);
        assert!(!ext.table);
        assert!(!ext.autolink);
        assert!(!ext.tasklist);
        assert!(!ext.superscript);
        assert!(!ext.subscript);
        assert!(ext.header_ids.is_none());
        assert!(!ext.footnotes);
        assert!(!ext.inline_footnotes);
        assert!(!ext.description_lists);
        assert!(ext.front_matter_delimiter.is_none());
        assert!(!ext.multiline_block_quotes);
        assert!(!ext.alerts);
        assert!(!ext.math_dollars);
        assert!(!ext.math_code);
        assert!(!ext.wikilinks_title_after_pipe);
        assert!(!ext.wikilinks_title_before_pipe);
        assert!(!ext.underline);
        assert!(!ext.spoiler);
        assert!(!ext.greentext);
        assert!(!ext.highlight);
        assert!(!ext.insert);
        assert!(!ext.cjk_friendly_emphasis);
        assert!(!ext.subtext);
        assert!(!ext.shortcodes);
        assert!(ext.image_url_rewriter.is_none());
        assert!(ext.link_url_rewriter.is_none());
    }

    #[test]
    fn test_extension_debug() {
        let ext = Extension::default();
        let debug_str = format!("{:?}", ext);
        assert!(debug_str.contains("Extension"));
        assert!(debug_str.contains("strikethrough"));
    }
}
