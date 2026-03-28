//! Plugin system for extending Markdown rendering
//!
//! This module provides a plugin architecture that allows users to customize
//! various aspects of Markdown rendering, such as syntax highlighting,
//! heading rendering, and code block handling.
//!
//! # Example
//!
//! ```ignore
//! use clmd::plugins::{Plugins, SyntaxHighlighterAdapter};
//! use clmd::parser::options::Options;
//!
//! // Create a custom syntax highlighter
//! struct MyHighlighter;
//!
//! impl SyntaxHighlighterAdapter for MyHighlighter {
//!     fn highlight(&self, code: &str, language: Option<&str>) -> String {
//!         format!("<pre><code class=\"lang-{lang}\">{code}</code></pre>",
//!             lang = language.unwrap_or("text"),
//!             code = code)
//!     }
//! }
//!
//! // Configure plugins
//! let mut plugins = Plugins::new();
//! plugins.render.set_syntax_highlighter(&MyHighlighter);
//! ```

use std::collections::HashMap;
use std::fmt;

// Re-export adapter traits for convenience
pub use crate::adapters::{
    CodefenceRendererAdapter, HeadingAdapter, SyntaxHighlighterAdapter, UrlRewriter,
};

/// A collection of plugins for customizing rendering behavior (comrak-style).
///
/// This struct holds references to various adapter implementations
/// that can customize how different elements are rendered.
///
/// # Example
///
/// ```ignore
/// use clmd::plugins::Plugins;
///
/// let mut plugins = Plugins::new();
/// // Configure render plugins
/// plugins.render.set_syntax_highlighter(&my_highlighter);
/// ```
#[derive(Default, Debug)]
pub struct Plugins<'p> {
    /// Render-time plugins.
    pub render: RenderPlugins<'p>,
}

impl<'p> Plugins<'p> {
    /// Create a new empty plugins collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any plugins are registered.
    pub fn is_empty(&self) -> bool {
        self.render.is_empty()
    }
}

/// A collection of plugins for customizing rendering behavior (owned version).
///
/// This struct holds owned adapter implementations for use cases where
/// lifetime management is not desired.
#[derive(Default)]
pub struct OwnedPlugins {
    /// Syntax highlighter for code blocks
    syntax_highlighter: Option<Box<dyn SyntaxHighlighterAdapter>>,
    /// Custom heading renderer
    heading_adapter: Option<Box<dyn HeadingAdapter>>,
    /// Custom code fence renderers by language
    codefence_renderers: HashMap<String, Box<dyn CodefenceRendererAdapter>>,
    /// Custom link URL rewriter
    link_url_rewriter: Option<Box<dyn UrlRewriter>>,
    /// Custom image URL rewriter
    image_url_rewriter: Option<Box<dyn UrlRewriter>>,
}

impl OwnedPlugins {
    /// Create a new empty plugins collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the syntax highlighter
    pub fn set_syntax_highlighter(
        &mut self,
        adapter: Box<dyn SyntaxHighlighterAdapter>,
    ) {
        self.syntax_highlighter = Some(adapter);
    }

    /// Get the syntax highlighter if set
    pub fn syntax_highlighter(&self) -> Option<&dyn SyntaxHighlighterAdapter> {
        self.syntax_highlighter.as_ref().map(|b| b.as_ref())
    }

    /// Set the heading adapter
    pub fn set_heading_adapter(&mut self, adapter: Box<dyn HeadingAdapter>) {
        self.heading_adapter = Some(adapter);
    }

    /// Get the heading adapter if set
    pub fn heading_adapter(&self) -> Option<&dyn HeadingAdapter> {
        self.heading_adapter.as_ref().map(|b| b.as_ref())
    }

    /// Register a code fence renderer for a specific language
    pub fn register_codefence_renderer(
        &mut self,
        language: impl Into<String>,
        renderer: Box<dyn CodefenceRendererAdapter>,
    ) {
        self.codefence_renderers.insert(language.into(), renderer);
    }

    /// Get a code fence renderer for a specific language
    pub fn codefence_renderer(
        &self,
        language: &str,
    ) -> Option<&dyn CodefenceRendererAdapter> {
        self.codefence_renderers.get(language).map(|b| b.as_ref())
    }

    /// Set the link URL rewriter
    pub fn set_link_url_rewriter(&mut self, rewriter: Box<dyn UrlRewriter>) {
        self.link_url_rewriter = Some(rewriter);
    }

    /// Get the link URL rewriter if set
    pub fn link_url_rewriter(&self) -> Option<&dyn UrlRewriter> {
        self.link_url_rewriter.as_ref().map(|b| b.as_ref())
    }

    /// Set the image URL rewriter
    pub fn set_image_url_rewriter(&mut self, rewriter: Box<dyn UrlRewriter>) {
        self.image_url_rewriter = Some(rewriter);
    }

    /// Get the image URL rewriter if set
    pub fn image_url_rewriter(&self) -> Option<&dyn UrlRewriter> {
        self.image_url_rewriter.as_ref().map(|b| b.as_ref())
    }

    /// Check if any plugins are registered
    pub fn is_empty(&self) -> bool {
        self.syntax_highlighter.is_none()
            && self.heading_adapter.is_none()
            && self.codefence_renderers.is_empty()
            && self.link_url_rewriter.is_none()
            && self.image_url_rewriter.is_none()
    }

    /// Convert to comrak-style Plugins struct.
    ///
    /// Note: This creates a new RenderPlugins with references to the owned adapters.
    /// The returned Plugins must not outlive this OwnedPlugins.
    pub fn as_plugins(&self) -> Plugins<'_> {
        let mut render = RenderPlugins::new();
        if let Some(ref highlighter) = self.syntax_highlighter {
            render.set_syntax_highlighter(highlighter.as_ref());
        }
        if let Some(ref adapter) = self.heading_adapter {
            render.set_heading_adapter(adapter.as_ref());
        }
        for (lang, renderer) in &self.codefence_renderers {
            render.register_codefence_renderer(lang.clone(), renderer.as_ref());
        }
        Plugins { render }
    }
}

impl fmt::Debug for OwnedPlugins {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedPlugins")
            .field("has_syntax_highlighter", &self.syntax_highlighter.is_some())
            .field("has_heading_adapter", &self.heading_adapter.is_some())
            .field(
                "codefence_renderers",
                &self.codefence_renderers.keys().collect::<Vec<_>>(),
            )
            .field("has_link_url_rewriter", &self.link_url_rewriter.is_some())
            .field("has_image_url_rewriter", &self.image_url_rewriter.is_some())
            .finish()
    }
}

/// Plugins for alternative rendering (comrak-style).
///
/// This struct provides a comrak-compatible interface for render-time plugins.
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

impl fmt::Debug for RenderPlugins<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    fn write_pre_tag<'s>(
        &self,
        output: &mut dyn fmt::Write,
        _attributes: HashMap<&str, std::borrow::Cow<'s, str>>,
    ) -> fmt::Result {
        output.write_str("<pre>")
    }

    fn write_code_tag<'s>(
        &self,
        output: &mut dyn fmt::Write,
        attributes: HashMap<&str, std::borrow::Cow<'s, str>>,
    ) -> fmt::Result {
        if let Some(lang) = attributes.get("class") {
            write!(output, r#"<code class="{}">"#, lang)
        } else {
            output.write_str("<code>")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::collections::HashMap;

    #[test]
    fn test_plugins_default() {
        let plugins = Plugins::new();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_owned_plugins_default() {
        let plugins = OwnedPlugins::new();
        assert!(plugins.is_empty());
        assert!(plugins.syntax_highlighter().is_none());
        assert!(plugins.heading_adapter().is_none());
    }

    #[test]
    fn test_syntax_highlighter() {
        let mut plugins = OwnedPlugins::new();
        assert!(plugins.is_empty());

        plugins.set_syntax_highlighter(Box::new(DefaultSyntaxHighlighter));
        assert!(!plugins.is_empty());
        assert!(plugins.syntax_highlighter().is_some());
    }

    #[test]
    fn test_heading_adapter() {
        use crate::adapters::{HeadingAdapter, HeadingMeta};
        use crate::nodes::SourcePos;
        use std::fmt::Write;

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

        let mut plugins = OwnedPlugins::new();
        plugins.set_heading_adapter(Box::new(TestHeadingAdapter));
        assert!(plugins.heading_adapter().is_some());
    }

    #[test]
    fn test_default_syntax_highlighter() {
        let highlighter = DefaultSyntaxHighlighter;
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
        let highlighter = DefaultSyntaxHighlighter;
        let mut output = String::new();

        highlighter
            .write_pre_tag(&mut output, HashMap::new())
            .unwrap();
        assert_eq!(output, "<pre>");
    }

    #[test]
    fn test_default_syntax_highlighter_code_tag() {
        let highlighter = DefaultSyntaxHighlighter;
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
    fn test_plugins_debug() {
        let mut plugins = OwnedPlugins::new();
        plugins.set_syntax_highlighter(Box::new(DefaultSyntaxHighlighter));

        let debug = format!("{:?}", plugins);
        assert!(debug.contains("has_syntax_highlighter: true"));
    }

    #[test]
    fn test_render_plugins_default() {
        let plugins = RenderPlugins::new();
        assert!(plugins.is_empty());
        assert!(plugins.syntax_highlighter().is_none());
        assert!(plugins.heading_adapter().is_none());
    }

    #[test]
    fn test_render_plugins_with_highlighter() {
        struct TestHighlighter;
        impl SyntaxHighlighterAdapter for TestHighlighter {
            fn write_highlighted(
                &self,
                output: &mut dyn fmt::Write,
                _lang: Option<&str>,
                code: &str,
            ) -> fmt::Result {
                write!(output, "<code>{}</code>", code)
            }

            fn write_pre_tag(
                &self,
                output: &mut dyn fmt::Write,
                _attrs: HashMap<&'static str, Cow<'_, str>>,
            ) -> fmt::Result {
                output.write_str("<pre>")
            }

            fn write_code_tag(
                &self,
                output: &mut dyn fmt::Write,
                _attrs: HashMap<&'static str, Cow<'_, str>>,
            ) -> fmt::Result {
                output.write_str("<code>")
            }
        }

        let highlighter = TestHighlighter;
        let mut plugins = RenderPlugins::new();
        plugins.set_syntax_highlighter(&highlighter);

        assert!(!plugins.is_empty());
        assert!(plugins.syntax_highlighter().is_some());
    }
}
