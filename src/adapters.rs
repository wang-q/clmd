//! Adapter traits for plugins
//!
//! This module provides adapter traits for customizing various aspects of
//! Markdown rendering, such as syntax highlighting, heading rendering,
//! and code block handling.
//!
//! # Example
//!
//! ```
//! use clmd::adapters::{SyntaxHighlighterAdapter, HeadingAdapter, HeadingMeta};
//! use std::collections::HashMap;
//! use std::borrow::Cow;
//! use std::fmt::{self, Write};
//!
//! // Define a custom syntax highlighter
//! struct MyHighlighter;
//!
//! impl SyntaxHighlighterAdapter for MyHighlighter {
//!     fn write_pre_tag<'s>(
//!         &self,
//!         output: &mut dyn Write,
//!         attributes: HashMap<&str, Cow<'s, str>>,
//!     ) -> fmt::Result {
//!         output.write_str("<pre>")
//!     }
//!
//!     fn write_code_tag<'s>(
//!         &self,
//!         output: &mut dyn Write,
//!         attributes: HashMap<&str, Cow<'s, str>>,
//!     ) -> fmt::Result {
//!         output.write_str("<code>")
//!     }
//!
//!     fn write_highlighted(
//!         &self,
//!         output: &mut dyn Write,
//!         lang: Option<&str>,
//!         code: &str,
//!     ) -> fmt::Result {
//!         output.write_str(code)
//!     }
//! }
//! ```

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Write};

use crate::nodes::SourcePos;

/// Adapter for syntax highlighting in code blocks.
///
/// This trait allows you to customize how code blocks are rendered in HTML output.
/// You can implement this trait to integrate with syntax highlighting libraries
/// like syntect, tree-sitter, or highlight.js.
///
/// # Example
///
/// ```
/// use clmd::adapters::SyntaxHighlighterAdapter;
/// use std::collections::HashMap;
/// use std::borrow::Cow;
/// use std::fmt::{self, Write};
///
/// struct PlainHighlighter;
///
/// impl SyntaxHighlighterAdapter for PlainHighlighter {
///     fn write_pre_tag<'s>(
///         &self,
///         output: &mut dyn Write,
///         _attributes: HashMap<&str, Cow<'s, str>>,
///     ) -> fmt::Result {
///         output.write_str("<pre>")
///     }
///
///     fn write_code_tag<'s>(
///         &self,
///         output: &mut dyn Write,
///         _attributes: HashMap<&str, Cow<'s, str>>,
///     ) -> fmt::Result {
///         output.write_str("<code>")
///     }
///
///     fn write_highlighted(
///         &self,
///         output: &mut dyn Write,
///         _lang: Option<&str>,
///         code: &str,
///     ) -> fmt::Result {
///         output.write_str(code)
///     }
/// }
/// ```
pub trait SyntaxHighlighterAdapter {
    /// Write the opening `<pre>` tag with attributes.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to write to
    /// * `attributes` - A map of attribute names to values
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn write_pre_tag<'s>(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<&str, Cow<'s, str>>,
    ) -> fmt::Result;

    /// Write the opening `<code>` tag with attributes.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to write to
    /// * `attributes` - A map of attribute names to values
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn write_code_tag<'s>(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<&str, Cow<'s, str>>,
    ) -> fmt::Result;

    /// Write the highlighted code.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to write to
    /// * `lang` - The language identifier (e.g., "rust", "python")
    /// * `code` - The code to highlight
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> fmt::Result;
}

/// Metadata for a heading element.
///
/// This struct contains information about a heading that is passed to
/// the [`HeadingAdapter`] during rendering.
#[derive(Debug, Clone)]
pub struct HeadingMeta {
    /// The heading level (1-6)
    pub level: u8,
    /// The text content of the heading (without HTML tags)
    pub content: String,
}

/// Adapter for custom heading rendering.
///
/// This trait allows you to customize how headings are rendered in HTML output.
/// You can use this to implement features like automatic ID generation for
/// anchor links, custom heading wrappers, or TOC integration.
///
/// # Example
///
/// ```
/// use clmd::adapters::{HeadingAdapter, HeadingMeta};
/// use clmd::nodes::SourcePos;
/// use std::fmt::{self, Write};
///
/// struct AnchorHeadingAdapter;
///
/// impl HeadingAdapter for AnchorHeadingAdapter {
///     fn enter(
///         &self,
///         output: &mut dyn Write,
///         heading: &HeadingMeta,
///         _sourcepos: Option<SourcePos>,
///     ) -> fmt::Result {
///         let id = heading.content.to_lowercase()
///             .replace(' ', "-")
///             .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
///         write!(output, "<h{} id=\"{}\">", heading.level, id)
///     }
///
///     fn exit(&self, output: &mut dyn Write, heading: &HeadingMeta) -> fmt::Result {
///         write!(output, "</h{}>", heading.level)
///     }
/// }
/// ```
pub trait HeadingAdapter {
    /// Called when entering a heading element.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to write to
    /// * `heading` - Metadata about the heading
    /// * `sourcepos` - The source position of the heading (if enabled)
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn enter(
        &self,
        output: &mut dyn Write,
        heading: &HeadingMeta,
        sourcepos: Option<SourcePos>,
    ) -> fmt::Result;

    /// Called when exiting a heading element.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to write to
    /// * `heading` - Metadata about the heading
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn exit(&self, output: &mut dyn Write, heading: &HeadingMeta) -> fmt::Result;
}

/// Adapter for custom link rendering.
///
/// This trait allows you to customize how links are rendered in HTML output.
/// You can use this to implement features like link rewriting, external link
/// indicators, or security checks.
pub trait LinkAdapter {
    /// Called when entering a link element.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to write to
    /// * `url` - The link URL
    /// * `title` - The link title (if any)
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn enter_link(&self, output: &mut dyn Write, url: &str, title: Option<&str>) -> fmt::Result;

    /// Called when exiting a link element.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to write to
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn exit_link(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom image rendering.
///
/// This trait allows you to customize how images are rendered in HTML output.
/// You can use this to implement features like lazy loading, responsive images,
/// or image optimization.
pub trait ImageAdapter {
    /// Called when rendering an image element.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to write to
    /// * `src` - The image source URL
    /// * `alt` - The alt text
    /// * `title` - The image title (if any)
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn render_image(
        &self,
        output: &mut dyn Write,
        src: &str,
        alt: &str,
        title: Option<&str>,
    ) -> fmt::Result;
}

/// Adapter for custom list rendering.
///
/// This trait allows you to customize how lists are rendered in HTML output.
pub trait ListAdapter {
    /// Called when entering a list element.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to write to
    /// * `ordered` - Whether the list is ordered
    /// * `start` - The starting number for ordered lists
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn enter_list(&self, output: &mut dyn Write, ordered: bool, start: u32) -> fmt::Result;

    /// Called when exiting a list element.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to write to
    /// * `ordered` - Whether the list is ordered
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn exit_list(&self, output: &mut dyn Write, ordered: bool) -> fmt::Result;
}

/// Adapter for custom table rendering.
///
/// This trait allows you to customize how tables are rendered in HTML output.
pub trait TableAdapter {
    /// Called when entering a table element.
    fn enter_table(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting a table element.
    fn exit_table(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when entering a table row.
    fn enter_row(&self, output: &mut dyn Write, is_header: bool) -> fmt::Result;

    /// Called when exiting a table row.
    fn exit_row(&self, output: &mut dyn Write, is_header: bool) -> fmt::Result;

    /// Called when entering a table cell.
    fn enter_cell(&self, output: &mut dyn Write, is_header: bool, align: Option<&str>) -> fmt::Result;

    /// Called when exiting a table cell.
    fn exit_cell(&self, output: &mut dyn Write, is_header: bool) -> fmt::Result;
}

/// Adapter for custom code block rendering.
///
/// This trait allows you to customize how code blocks are rendered in HTML output.
/// Unlike [`SyntaxHighlighterAdapter`], this gives you full control over the
/// entire code block rendering.
pub trait CodeBlockAdapter {
    /// Called when entering a code block.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to write to
    /// * `info` - The info string from the code fence (e.g., "rust" from ```rust)
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn enter_code_block(&self, output: &mut dyn Write, info: &str) -> fmt::Result;

    /// Called when exiting a code block.
    fn exit_code_block(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called to render the code content.
    fn render_code(&self, output: &mut dyn Write, code: &str) -> fmt::Result;
}

/// Adapter for custom blockquote rendering.
///
/// This trait allows you to customize how blockquotes are rendered in HTML output.
pub trait BlockQuoteAdapter {
    /// Called when entering a blockquote.
    fn enter_blockquote(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting a blockquote.
    fn exit_blockquote(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom paragraph rendering.
///
/// This trait allows you to customize how paragraphs are rendered in HTML output.
pub trait ParagraphAdapter {
    /// Called when entering a paragraph.
    fn enter_paragraph(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting a paragraph.
    fn exit_paragraph(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom emphasis rendering.
///
/// This trait allows you to customize how emphasis (italic) is rendered in HTML output.
pub trait EmphasisAdapter {
    /// Called when entering emphasized text.
    fn enter_emphasis(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting emphasized text.
    fn exit_emphasis(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom strong rendering.
///
/// This trait allows you to customize how strong (bold) text is rendered in HTML output.
pub trait StrongAdapter {
    /// Called when entering strong text.
    fn enter_strong(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting strong text.
    fn exit_strong(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom strikethrough rendering.
///
/// This trait allows you to customize how strikethrough text is rendered in HTML output.
pub trait StrikethroughAdapter {
    /// Called when entering strikethrough text.
    fn enter_strikethrough(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting strikethrough text.
    fn exit_strikethrough(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom inline code rendering.
///
/// This trait allows you to customize how inline code is rendered in HTML output.
pub trait InlineCodeAdapter {
    /// Called when entering inline code.
    fn enter_inline_code(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting inline code.
    fn exit_inline_code(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called to render the code content.
    fn render_code(&self, output: &mut dyn Write, code: &str) -> fmt::Result;
}

/// Adapter for custom footnote rendering.
///
/// This trait allows you to customize how footnotes are rendered in HTML output.
pub trait FootnoteAdapter {
    /// Called when entering a footnote definition.
    fn enter_footnote_definition(&self, output: &mut dyn Write, name: &str) -> fmt::Result;

    /// Called when exiting a footnote definition.
    fn exit_footnote_definition(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when rendering a footnote reference.
    fn render_footnote_reference(&self, output: &mut dyn Write, name: &str) -> fmt::Result;
}

/// Adapter for custom task list item rendering.
///
/// This trait allows you to customize how task list items are rendered in HTML output.
pub trait TaskItemAdapter {
    /// Called when rendering a task list item checkbox.
    fn render_checkbox(&self, output: &mut dyn Write, checked: bool) -> fmt::Result;
}

/// Adapter for custom math rendering.
///
/// This trait allows you to customize how math expressions are rendered in HTML output.
pub trait MathAdapter {
    /// Called when rendering an inline math expression.
    fn render_inline_math(&self, output: &mut dyn Write, math: &str) -> fmt::Result;

    /// Called when rendering a display math expression.
    fn render_display_math(&self, output: &mut dyn Write, math: &str) -> fmt::Result;
}

/// Adapter for custom definition list rendering.
///
/// This trait allows you to customize how definition lists are rendered in HTML output.
pub trait DefinitionListAdapter {
    /// Called when entering a definition list.
    fn enter_definition_list(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting a definition list.
    fn exit_definition_list(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when entering a definition term.
    fn enter_definition_term(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting a definition term.
    fn exit_definition_term(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when entering a definition description.
    fn enter_definition_description(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting a definition description.
    fn exit_definition_description(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom abbreviation rendering.
///
/// This trait allows you to customize how abbreviations are rendered in HTML output.
pub trait AbbreviationAdapter {
    /// Called when rendering an abbreviation.
    fn render_abbreviation(&self, output: &mut dyn Write, abbr: &str, title: &str) -> fmt::Result;
}

/// Adapter for custom subscript rendering.
///
/// This trait allows you to customize how subscript text is rendered in HTML output.
pub trait SubscriptAdapter {
    /// Called when entering subscript text.
    fn enter_subscript(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting subscript text.
    fn exit_subscript(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom superscript rendering.
///
/// This trait allows you to customize how superscript text is rendered in HTML output.
pub trait SuperscriptAdapter {
    /// Called when entering superscript text.
    fn enter_superscript(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting superscript text.
    fn exit_superscript(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom inserted text rendering.
///
/// This trait allows you to customize how inserted (underlined) text is rendered in HTML output.
pub trait InsertedAdapter {
    /// Called when entering inserted text.
    fn enter_inserted(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting inserted text.
    fn exit_inserted(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom marked text rendering.
///
/// This trait allows you to customize how marked (highlighted) text is rendered in HTML output.
pub trait MarkedAdapter {
    /// Called when entering marked text.
    fn enter_marked(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting marked text.
    fn exit_marked(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom spoiler rendering.
///
/// This trait allows you to customize how spoiler text is rendered in HTML output.
pub trait SpoilerAdapter {
    /// Called when entering spoiler text.
    fn enter_spoiler(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting spoiler text.
    fn exit_spoiler(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom wiki link rendering.
///
/// This trait allows you to customize how wiki links are rendered in HTML output.
pub trait WikiLinkAdapter {
    /// Called when rendering a wiki link.
    fn render_wiki_link(&self, output: &mut dyn Write, target: &str, title: Option<&str>) -> fmt::Result;
}

/// Adapter for custom emoji rendering.
///
/// This trait allows you to customize how emoji shortcodes are rendered in HTML output.
pub trait EmojiAdapter {
    /// Called when rendering an emoji.
    fn render_emoji(&self, output: &mut dyn Write, shortcode: &str) -> fmt::Result;
}

/// Adapter for custom autolink rendering.
///
/// This trait allows you to customize how autolinks are rendered in HTML output.
pub trait AutolinkAdapter {
    /// Called when rendering an autolink.
    fn render_autolink(&self, output: &mut dyn Write, url: &str, is_email: bool) -> fmt::Result;
}

/// Adapter for custom raw HTML rendering.
///
/// This trait allows you to customize how raw HTML is rendered in HTML output.
pub trait RawHtmlAdapter {
    /// Called when rendering raw HTML.
    fn render_raw_html(&self, output: &mut dyn Write, html: &str, is_block: bool) -> fmt::Result;
}

/// Adapter for custom line break rendering.
///
/// This trait allows you to customize how line breaks are rendered in HTML output.
pub trait LineBreakAdapter {
    /// Called when rendering a soft line break.
    fn render_soft_break(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when rendering a hard line break.
    fn render_hard_break(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom thematic break rendering.
///
/// This trait allows you to customize how thematic breaks (horizontal rules) are rendered in HTML output.
pub trait ThematicBreakAdapter {
    /// Called when rendering a thematic break.
    fn render_thematic_break(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for custom document rendering.
///
/// This trait allows you to customize how the document is rendered in HTML output.
pub trait DocumentAdapter {
    /// Called when entering the document.
    fn enter_document(&self, output: &mut dyn Write) -> fmt::Result;

    /// Called when exiting the document.
    fn exit_document(&self, output: &mut dyn Write) -> fmt::Result;
}

/// Adapter for rendering code fence blocks with custom logic.
///
/// This trait allows you to customize how code fence blocks are rendered
/// based on the language identifier.
///
/// # Example
///
/// ```
/// use clmd::adapters::CodefenceRendererAdapter;
/// use clmd::nodes::SourcePos;
/// use std::fmt::{self, Write};
///
/// struct MermaidRenderer;
///
/// impl CodefenceRendererAdapter for MermaidRenderer {
///     fn write(
///         &self,
///         output: &mut dyn Write,
///         lang: &str,
///         _meta: &str,
///         code: &str,
///         _sourcepos: Option<SourcePos>,
///     ) -> fmt::Result {
///         if lang == "mermaid" {
///             write!(output, "<div class=\"mermaid\">{}</div>", code)
///         } else {
///             write!(output, "<pre><code>{}</code></pre>", code)
///         }
///     }
/// }
/// ```
pub trait CodefenceRendererAdapter {
    /// Write the code fence block.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to write to
    /// * `lang` - The language identifier from the info string
    /// * `meta` - Additional metadata from the info string
    /// * `code` - The code content
    /// * `sourcepos` - The source position (if enabled)
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn write(
        &self,
        output: &mut dyn Write,
        lang: &str,
        meta: &str,
        code: &str,
        sourcepos: Option<SourcePos>,
    ) -> fmt::Result;
}

/// Adapter for URL rewriting.
///
/// This trait allows you to customize how URLs are rewritten during rendering.
/// You can use this to implement features like base URL prepending, CDN rewriting,
/// or link validation.
///
/// # Example
///
/// ```
/// use clmd::adapters::UrlRewriter;
///
/// struct CdnRewriter {
///     base_url: String,
/// }
///
/// impl UrlRewriter for CdnRewriter {
///     fn rewrite(&self, url: &str) -> String {
///         if url.starts_with("http") {
///             url.to_string()
///         } else {
///             format!("{}{}", self.base_url, url)
///         }
///     }
/// }
/// ```
pub trait UrlRewriter {
    /// Rewrite a URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The original URL
    ///
    /// # Returns
    ///
    /// The rewritten URL
    fn rewrite(&self, url: &str) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHighlighter;

    impl SyntaxHighlighterAdapter for TestHighlighter {
        fn write_pre_tag<'s>(
            &self,
            output: &mut dyn Write,
            _attributes: HashMap<&str, Cow<'s, str>>,
        ) -> fmt::Result {
            output.write_str("<pre class=\"highlight\">")
        }

        fn write_code_tag<'s>(
            &self,
            output: &mut dyn Write,
            _attributes: HashMap<&str, Cow<'s, str>>,
        ) -> fmt::Result {
            output.write_str("<code>")
        }

        fn write_highlighted(
            &self,
            output: &mut dyn Write,
            _lang: Option<&str>,
            code: &str,
        ) -> fmt::Result {
            output.write_str(code)
        }
    }

    struct TestHeadingAdapter;

    impl HeadingAdapter for TestHeadingAdapter {
        fn enter(
            &self,
            output: &mut dyn Write,
            heading: &HeadingMeta,
            _sourcepos: Option<SourcePos>,
        ) -> fmt::Result {
            write!(output, "<h{}>", heading.level)
        }

        fn exit(&self, output: &mut dyn Write, heading: &HeadingMeta) -> fmt::Result {
            write!(output, "</h{}>", heading.level)
        }
    }

    #[test]
    fn test_syntax_highlighter_adapter() {
        let highlighter = TestHighlighter;
        let mut output = String::new();

        let attrs: HashMap<&str, Cow<'static, str>> = HashMap::new();
        highlighter.write_pre_tag(&mut output, attrs.clone()).unwrap();
        highlighter.write_code_tag(&mut output, attrs).unwrap();
        highlighter.write_highlighted(&mut output, Some("rust"), "fn main() {}").unwrap();

        assert!(output.contains("<pre class=\"highlight\">"));
        assert!(output.contains("<code>"));
        assert!(output.contains("fn main() {}"));
    }

    #[test]
    fn test_heading_adapter() {
        let adapter = TestHeadingAdapter;
        let mut output = String::new();

        let meta = HeadingMeta {
            level: 1,
            content: "Test Heading".to_string(),
        };

        adapter.enter(&mut output, &meta, None).unwrap();
        adapter.exit(&mut output, &meta).unwrap();

        assert_eq!(output, "<h1></h1>");
    }
}
