//! Owned plugins for clmd
//!
//! This module provides owned versions of plugins for use cases where
//! lifetime management is not desired.

use std::collections::HashMap;
use std::fmt;

use crate::core::adapters::{
    CodefenceRendererAdapter, HeadingAdapter, SyntaxHighlighterAdapter, UrlRewriter,
};
use crate::parser::options::{Plugins, RenderPlugins};

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
        use crate::adapters::HeadingAdapter;

        struct TestHeadingAdapter;
        impl HeadingAdapter for TestHeadingAdapter {
            fn render_heading(
                &self,
                level: u8,
                content: &str,
                id: Option<&str>,
            ) -> Option<String> {
                Some(format!("<h{} id={:?}>{}</h{}>", level, id, content, level))
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
}
