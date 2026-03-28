//! Adapter traits for plugins
//!
//! This module provides adapter traits for customizing various aspects of
//! Markdown rendering. Each plugin implements one or more of these traits.
//!
//! # Example
//!
//! ```ignore
//! use clmd::adapters::{SyntaxHighlighterAdapter, HeadingAdapter, HeadingMeta};
//! use std::collections::HashMap;
//! use std::borrow::Cow;
//! use std::fmt;
//!
//! struct MyHighlighter;
//!
//! impl SyntaxHighlighterAdapter for MyHighlighter {
//!     fn write_highlighted(
//!         &self,
//!         output: &mut dyn fmt::Write,
//!         lang: Option<&str>,
//!         code: &str,
//!     ) -> fmt::Result {
//!         write!(output, "<code class=\"lang-{}\">{}</code>",
//!             lang.unwrap_or("text"), code)
//!     }
//!
//!     fn write_pre_tag(
//!         &self,
//!         output: &mut dyn fmt::Write,
//!         _attributes: HashMap<&'static str, Cow<'_, str>>,
//!     ) -> fmt::Result {
//!         output.write_str("<pre>")
//!     }
//!
//!     fn write_code_tag(
//!         &self,
//!         output: &mut dyn fmt::Write,
//!         _attributes: HashMap<&'static str, Cow<'_, str>>,
//!     ) -> fmt::Result {
//!         output.write_str("<code>")
//!     }
//! }
//! ```

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

/// Implement this adapter for custom rendering of codefence blocks.
///
/// This allows you to provide custom rendering for specific languages,
/// such as rendering Mermaid diagrams as SVG or Math as formatted output.
pub trait CodefenceRendererAdapter: Send + Sync {
    /// Render a codefence block.
    ///
    /// # Arguments
    ///
    /// * `output` - The output stream to write to
    /// * `lang` - Name of the programming language (the first token of the info string)
    /// * `meta` - The remaining codefence info string after the language token, trimmed
    /// * `code` - The code content to render
    /// * `sourcepos` - Optional source position information
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn write(
        &self,
        output: &mut dyn fmt::Write,
        lang: &str,
        meta: &str,
        code: &str,
        sourcepos: Option<crate::nodes::SourcePos>,
    ) -> fmt::Result;
}

/// Implement this adapter for custom syntax highlighting of codefence blocks.
///
/// This trait provides fine-grained control over how code blocks are rendered,
/// allowing you to integrate with external syntax highlighting libraries.
pub trait SyntaxHighlighterAdapter: Send + Sync {
    /// Generates syntax highlighted HTML output.
    ///
    /// # Arguments
    ///
    /// * `output` - The output stream to write to
    /// * `lang` - The language identifier (e.g., "rust", "python")
    /// * `code` - The source code to be syntax highlighted
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn write_highlighted(
        &self,
        output: &mut dyn fmt::Write,
        lang: Option<&str>,
        code: &str,
    ) -> fmt::Result;

    /// Generates the opening `<pre>` tag.
    ///
    /// Some syntax highlighter libraries might include their own `<pre>` tag
    /// possibly with some HTML attributes pre-filled.
    ///
    /// # Arguments
    ///
    /// * `output` - The output stream to write to
    /// * `attributes` - A map of HTML attributes provided by the parser
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn write_pre_tag(
        &self,
        output: &mut dyn fmt::Write,
        attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result;

    /// Generates the opening `<code>` tag.
    ///
    /// Some syntax highlighter libraries might include their own `<code>` tag
    /// possibly with some HTML attributes pre-filled.
    ///
    /// # Arguments
    ///
    /// * `output` - The output stream to write to
    /// * `attributes` - A map of HTML attributes provided by the parser
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn write_code_tag(
        &self,
        output: &mut dyn fmt::Write,
        attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result;
}

/// Metadata for a heading, passed to the [`HeadingAdapter`].
#[derive(Clone, Debug)]
pub struct HeadingMeta {
    /// The level of the heading; from 1 to 6 for ATX headings, 1 or 2 for setext headings.
    pub level: u8,

    /// The content of the heading as a "flattened" string.
    ///
    /// Flattened in the sense that any `<strong>` or other tags are removed.
    /// In the Markdown heading `## This is **bold**`, for example, this would
    /// be the string `"This is bold"`.
    pub content: String,
}

/// Implement this adapter for custom heading rendering.
///
/// The `enter` method defines what's rendered prior to the AST content of the
/// heading while the `exit` method defines what's rendered after it. Both
/// methods provide access to a [`HeadingMeta`] struct and leave the AST content
/// of the heading unchanged.
///
/// # Example
///
/// ```ignore
/// use clmd::adapters::{HeadingAdapter, HeadingMeta};
/// use std::fmt;
///
/// struct AnchorHeadingAdapter;
///
/// impl HeadingAdapter for AnchorHeadingAdapter {
///     fn enter(
///         &self,
///         output: &mut dyn fmt::Write,
///         heading: &HeadingMeta,
///         _sourcepos: Option<clmd::nodes::SourcePos>,
///     ) -> fmt::Result {
///         let id = heading.content.to_lowercase().replace(' ', "-");
///         write!(output, r#"<h{} id="{}"><a href="#{}">"#, heading.level, id, id)
///     }
///
///     fn exit(&self, output: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result {
///         write!(output, "</a></h{}>", heading.level)
///     }
/// }
/// ```
pub trait HeadingAdapter: Send + Sync {
    /// Render the opening tag.
    ///
    /// # Arguments
    ///
    /// * `output` - The output stream to write to
    /// * `heading` - Metadata about the heading
    /// * `sourcepos` - Optional source position information
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn enter(
        &self,
        output: &mut dyn fmt::Write,
        heading: &HeadingMeta,
        sourcepos: Option<crate::nodes::SourcePos>,
    ) -> fmt::Result;

    /// Render the closing tag.
    ///
    /// # Arguments
    ///
    /// * `output` - The output stream to write to
    /// * `heading` - Metadata about the heading
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or failure
    fn exit(&self, output: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result;
}

/// Trait for link and image URL rewrite extensions.
///
/// This trait allows you to customize how URLs are rewritten during rendering,
/// for example to add CDN prefixes or convert relative URLs.
///
/// # Example
///
/// ```ignore
/// use clmd::adapters::UrlRewriter;
///
/// struct CdnRewriter {
///     base_url: String,
/// }
///
/// impl UrlRewriter for CdnRewriter {
///     fn rewrite(&self, url: &str) -> String {
///         if url.starts_with("http://") || url.starts_with("https://") {
///             url.to_string()
///         } else {
///             format!("{}/{}", self.base_url, url)
///         }
///     }
/// }
/// ```
pub trait UrlRewriter: Send + Sync + std::fmt::Debug {
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

/// Trait for resolving broken link references.
///
/// When the parser encounters a potential link that has a broken reference
/// (e.g `[foo]` when there is no `[foo]: url` entry), this callback is called
/// to potentially resolve the reference.
///
/// # Example
///
/// ```ignore
/// use clmd::adapters::{BrokenLinkCallback, BrokenLinkReference, ResolvedReference};
///
/// struct MyBrokenLinkHandler;
///
/// impl BrokenLinkCallback for MyBrokenLinkHandler {
///     fn resolve(&self, link: BrokenLinkReference) -> Option<ResolvedReference> {
///         if link.normalized == "example" {
///             Some(ResolvedReference {
///                 url: "https://example.com".to_string(),
///                 title: "Example".to_string(),
///             })
///         } else {
///             None
///         }
///     }
/// }
/// ```
pub trait BrokenLinkCallback: Send + Sync + std::fmt::Debug {
    /// Potentially resolve a single broken link reference.
    ///
    /// # Arguments
    ///
    /// * `link` - Details about the broken link reference
    ///
    /// # Returns
    ///
    /// `Some(ResolvedReference)` if the link should be resolved, `None` otherwise
    fn resolve(&self, link: BrokenLinkReference) -> Option<ResolvedReference>;
}

/// Details about a broken link reference.
#[derive(Debug, Clone)]
pub struct BrokenLinkReference<'a> {
    /// The normalized reference link label.
    ///
    /// Unicode case folding is applied; see <https://github.com/commonmark/commonmark-spec/issues/695>
    /// for a discussion on the details of what this exactly means.
    pub normalized: &'a str,

    /// The original text in the link label.
    pub original: &'a str,
}

/// A resolved reference for a broken link.
#[derive(Debug, Clone)]
pub struct ResolvedReference {
    /// The URL for the link.
    pub url: String,

    /// The title for the link.
    pub title: String,
}

/// A simple syntax highlighter that wraps code in a pre/code block
/// without any actual highlighting.
#[derive(Debug, Clone, Copy)]
pub struct DefaultSyntaxHighlighter;

impl DefaultSyntaxHighlighter {
    /// Create a new default syntax highlighter.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultSyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighterAdapter for DefaultSyntaxHighlighter {
    fn write_highlighted(
        &self,
        output: &mut dyn fmt::Write,
        _lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        // HTML escape the code
        let escaped = code
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        write!(output, "{}", escaped)
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn fmt::Write,
        _attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        output.write_str("<pre>")
    }

    fn write_code_tag(
        &self,
        output: &mut dyn fmt::Write,
        attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        if let Some(lang) = attributes.get("class") {
            write!(output, r#"<code class="{}">"#, lang)
        } else {
            output.write_str("<code>")
        }
    }
}

/// A heading adapter that generates anchor links for headings.
#[derive(Debug, Clone, Copy)]
pub struct AnchorHeadingAdapter;

impl AnchorHeadingAdapter {
    /// Create a new anchor heading adapter.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnchorHeadingAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl HeadingAdapter for AnchorHeadingAdapter {
    fn enter(
        &self,
        output: &mut dyn fmt::Write,
        heading: &HeadingMeta,
        _sourcepos: Option<crate::nodes::SourcePos>,
    ) -> fmt::Result {
        let id = generate_anchor_id(&heading.content);
        write!(
            output,
            "<h{} id=\"{}\" class=\"anchor\"><a href=\"#{}\">",
            heading.level, id, id
        )
    }

    fn exit(&self, output: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result {
        write!(output, "</a></h{}>", heading.level)
    }
}

/// Generate an anchor ID from heading content.
fn generate_anchor_id(content: &str) -> String {
    content
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_syntax_highlighter() {
        let highlighter = DefaultSyntaxHighlighter::new();
        let mut output = String::new();

        // Test write_highlighted
        highlighter
            .write_highlighted(&mut output, Some("rust"), "fn main() {}")
            .unwrap();
        assert_eq!(output, "fn main() {}");

        // Test HTML escaping
        output.clear();
        highlighter
            .write_highlighted(&mut output, None, "<script>")
            .unwrap();
        assert_eq!(output, "&lt;script&gt;");
    }

    #[test]
    fn test_default_syntax_highlighter_pre_tag() {
        let highlighter = DefaultSyntaxHighlighter::new();
        let mut output = String::new();

        highlighter
            .write_pre_tag(&mut output, HashMap::new())
            .unwrap();
        assert_eq!(output, "<pre>");
    }

    #[test]
    fn test_default_syntax_highlighter_code_tag() {
        let highlighter = DefaultSyntaxHighlighter::new();
        let mut output = String::new();

        // Without class
        highlighter
            .write_code_tag(&mut output, HashMap::new())
            .unwrap();
        assert_eq!(output, "<code>");

        // With class
        output.clear();
        let mut attrs = HashMap::new();
        attrs.insert("class", Cow::Borrowed("language-rust"));
        highlighter.write_code_tag(&mut output, attrs).unwrap();
        assert_eq!(output, r#"<code class="language-rust">"#);
    }

    #[test]
    fn test_anchor_heading_adapter() {
        let adapter = AnchorHeadingAdapter::new();
        let mut output = String::new();

        let meta = HeadingMeta {
            level: 1,
            content: "Hello World".to_string(),
        };

        adapter.enter(&mut output, &meta, None).unwrap();
        assert!(output.contains("<h1"));
        assert!(output.contains("id=\"hello-world\""));
        assert!(output.contains("<a href=\"#hello-world\""));

        output.clear();
        adapter.exit(&mut output, &meta).unwrap();
        assert_eq!(output, "</a></h1>");
    }

    #[test]
    fn test_generate_anchor_id() {
        assert_eq!(generate_anchor_id("Hello World"), "hello-world");
        assert_eq!(generate_anchor_id("Test 123"), "test-123");
        assert_eq!(generate_anchor_id("Special!@#Chars"), "specialchars");
        assert_eq!(generate_anchor_id("Multiple   Spaces"), "multiple---spaces");
    }

    #[test]
    fn test_broken_link_reference() {
        let link = BrokenLinkReference {
            normalized: "example",
            original: "Example",
        };
        assert_eq!(link.normalized, "example");
        assert_eq!(link.original, "Example");
    }

    #[test]
    fn test_resolved_reference() {
        let resolved = ResolvedReference {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
        };
        assert_eq!(resolved.url, "https://example.com");
        assert_eq!(resolved.title, "Example");
    }
}
