//! Parse options for the Markdown parser.
//!
//! This module provides configuration options for parsing behavior.

use super::traits::BrokenLinkCallback;
use arbitrary::Arbitrary;
use bon::Builder;
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

/// Options for parser functions.
///
/// The lifetime parameter `'c` allows parse options to hold references to
/// external data such as broken link callbacks.
#[derive(Clone, Builder, Arbitrary, Default)]
pub struct ParseOptions<'c> {
    /// Punctuation (quotes, full-stops and hyphens) are converted into 'smart' punctuation.
    ///
    /// ```ignore
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

impl<'c> Debug for ParseOptions<'c> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseOptions")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_options_default() {
        let parse = ParseOptions::default();
        assert!(!parse.smart);
        assert!(!parse.sourcepos);
        assert!(!parse.validate_utf8);
        assert!(parse.default_info_string.is_none());
        assert!(!parse.relaxed_tasklist_matching);
        assert!(!parse.ignore_setext);
        assert!(!parse.leave_footnote_definitions);
        assert!(!parse.tasklist_in_table);
        assert!(!parse.relaxed_autolinks);
        assert!(!parse.escaped_char_spans);
        assert!(parse.broken_link_callback.is_none());
    }

    #[test]
    fn test_parse_options_debug() {
        let parse = ParseOptions::default();
        let debug_str = format!("{:?}", parse);
        assert!(debug_str.contains("ParseOptions"));
        assert!(debug_str.contains("smart"));
    }
}
