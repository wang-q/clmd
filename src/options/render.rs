//! Render options for the Markdown renderer.
//!
//! This module provides configuration options for rendering behavior.

use super::types::ListStyleType;
use arbitrary::Arbitrary;
use bon::Builder;

/// Options for formatter functions.
#[derive(Debug, Clone, Copy, Builder, Arbitrary, Default)]
pub struct RenderOptions {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_options_default() {
        let render = RenderOptions::default();
        assert!(!render.hardbreaks);
        assert!(!render.nobreaks);
        assert!(!render.r#unsafe);
        assert!(!render.escape);
        assert!(!render.github_pre_lang);
        assert!(!render.full_info_string);
        assert_eq!(render.width, 0);
        assert_eq!(render.list_style, ListStyleType::Dash);
        assert!(!render.prefer_fenced);
        assert!(!render.ignore_empty_links);
        assert!(!render.tasklist_classes);
        assert!(!render.compact_html);
        assert!(!render.sourcepos);
        assert!(!render.gfm_quirks);
        assert!(!render.figure_with_caption);
        assert_eq!(render.ol_width, 0);
        assert!(!render.escaped_char_spans);
    }

    #[test]
    fn test_render_options_debug() {
        let render = RenderOptions::default();
        let debug_str = format!("{:?}", render);
        assert!(debug_str.contains("RenderOptions"));
        assert!(debug_str.contains("hardbreaks"));
    }
}
