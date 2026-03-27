//! Plugin system for extending Markdown rendering
//!
//! This module provides a plugin architecture that allows users to customize
//! various aspects of Markdown rendering, such as syntax highlighting,
//! heading rendering, and code block handling.
//!
//! # Example
//!
//! ```
//! use clmd::plugins::{Plugins, SyntaxHighlighterAdapter};
//! use clmd::config::options::Options;
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
//! plugins.set_syntax_highlighter(Box::new(MyHighlighter));
//! ```

use std::collections::HashMap;

/// A collection of plugins for customizing rendering behavior.
///
/// This struct holds references to various adapter implementations
/// that can customize how different elements are rendered.
#[derive(Default)]
pub struct Plugins {
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

impl Plugins {
    /// Create a new empty plugins collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the syntax highlighter
    pub fn set_syntax_highlighter(&mut self, adapter: Box<dyn SyntaxHighlighterAdapter>) {
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
    pub fn codefence_renderer(&self, language: &str) -> Option<&dyn CodefenceRendererAdapter> {
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
}

impl std::fmt::Debug for Plugins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plugins")
            .field("has_syntax_highlighter", &self.syntax_highlighter.is_some())
            .field("has_heading_adapter", &self.heading_adapter.is_some())
            .field("codefence_renderers", &self.codefence_renderers.keys().collect::<Vec<_>>())
            .field("has_link_url_rewriter", &self.link_url_rewriter.is_some())
            .field("has_image_url_rewriter", &self.image_url_rewriter.is_some())
            .finish()
    }
}

/// Adapter for syntax highlighting code blocks.
///
/// Implement this trait to provide custom syntax highlighting
/// for fenced code blocks.
///
/// # Example
///
/// ```
/// use clmd::plugins::SyntaxHighlighterAdapter;
///
/// struct SimpleHighlighter;
///
/// impl SyntaxHighlighterAdapter for SimpleHighlighter {
///     fn highlight(&self, code: &str, language: Option<&str>) -> String {
///         let lang = language.unwrap_or("text");
///         format!(
///             r#"<pre class=\"language-{lang}\"><code class=\"language-{lang}\">{code}</code></pre>"#,
///             lang = lang,
///             code = html_escape(code)
///         )
///     }
///
///     fn language_ids(&self) -> Vec<String> {
///         vec!["rust".to_string(), "python".to_string()]
///     }
/// }
///
/// fn html_escape(s: &str) -> String {
///     s.replace('&', "&amp;")
///      .replace('<', "&lt;")
///      .replace('>', "&gt;")
/// }
/// ```
pub trait SyntaxHighlighterAdapter: Send + Sync {
    /// Highlight the given code with optional language specification.
    ///
    /// # Arguments
    ///
    /// * `code` - The code to highlight
    /// * `language` - The language identifier (e.g., "rust", "python")
    ///
    /// # Returns
    ///
    /// The highlighted HTML string
    fn highlight(&self, code: &str, language: Option<&str>) -> String;

    /// Return a list of language IDs supported by this highlighter.
    ///
    /// This is used to determine which code blocks should be handled
    /// by this highlighter.
    fn language_ids(&self) -> Vec<String> {
        Vec::new()
    }

    /// Check if this highlighter supports a specific language.
    fn supports_language(&self, language: &str) -> bool {
        self.language_ids()
            .iter()
            .any(|lang| lang.eq_ignore_ascii_case(language))
    }
}

/// Adapter for customizing heading rendering.
///
/// Implement this trait to customize how headings are rendered,
/// for example to add anchor links or custom styling.
///
/// # Example
///
/// ```
/// use clmd::plugins::HeadingAdapter;
///
/// struct AnchorHeadingRenderer;
///
/// impl HeadingAdapter for AnchorHeadingRenderer {
///     fn enter(&self, level: u8, content: &str, id: Option<&str>) -> String {
///         let id = id.map(|s| s.to_string())
///             .unwrap_or_else(|| content.to_lowercase().replace(' ', "-"));
///         format!(r#"<h{level} id="{id}">"#, level = level, id = id)
///     }
///
///     fn exit(&self, level: u8) -> String {
///         format!("</h{level}>", level = level)
///     }
/// }
/// ```
pub trait HeadingAdapter: Send + Sync {
    /// Generate the opening tag for a heading.
    ///
    /// # Arguments
    ///
    /// * `level` - The heading level (1-6)
    /// * `content` - The heading content (for generating IDs)
    /// * `id` - An optional explicit ID from attributes
    ///
    /// # Returns
    ///
    /// The opening HTML tag
    fn enter(&self, level: u8, content: &str, id: Option<&str>) -> String;

    /// Generate the closing tag for a heading.
    ///
    /// # Arguments
    ///
    /// * `level` - The heading level (1-6)
    ///
    /// # Returns
    ///
    /// The closing HTML tag
    fn exit(&self, level: u8) -> String;
}

/// Adapter for rendering specific code fence languages.
///
/// Implement this trait to provide custom rendering for specific
/// code fence languages (e.g., rendering Graphviz diagrams as SVG).
///
/// # Example
///
/// ```
/// use clmd::plugins::CodefenceRendererAdapter;
///
/// struct MermaidRenderer;
///
/// impl CodefenceRendererAdapter for MermaidRenderer {
///     fn render(&self, code: &str, _info: &str) -> Option<String> {
///         // In a real implementation, this would convert Mermaid to SVG
///         Some(format!(
///             r#"<div class=\"mermaid\">{}</div>"#,
///             html_escape(code)
///         ))
///     }
/// }
///
/// fn html_escape(s: &str) -> String {
///     s.replace('&', "&amp;")
///      .replace('<', "&lt;")
///      .replace('>', "&gt;")
/// }
/// ```
pub trait CodefenceRendererAdapter: Send + Sync {
    /// Render a code fence to HTML.
    ///
    /// # Arguments
    ///
    /// * `code` - The code content
    /// * `info` - The full info string from the code fence
    ///
    /// # Returns
    ///
    /// Some HTML if this renderer handled the code, None to fall back
    /// to default rendering
    fn render(&self, code: &str, info: &str) -> Option<String>;
}

/// Adapter for rewriting URLs in links and images.
///
/// Implement this trait to customize how URLs are rewritten,
/// for example to add CDN prefixes or convert relative URLs.
///
/// # Example
///
/// ```
/// use clmd::plugins::UrlRewriter;
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
pub trait UrlRewriter: Send + Sync {
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

/// A simple syntax highlighter that wraps code in a pre/code block
/// without any actual highlighting.
pub struct DefaultSyntaxHighlighter;

impl SyntaxHighlighterAdapter for DefaultSyntaxHighlighter {
    fn highlight(&self, code: &str, language: Option<&str>) -> String {
        let escaped = code
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");

        match language {
            Some(lang) => format!(
                r#"<pre><code class="language-{lang}">{escaped}</code></pre>"#,
                lang = lang,
                escaped = escaped
            ),
            None => format!("<pre><code>{}</code></pre>", escaped),
        }
    }
}

/// A heading adapter that generates anchor links for headings.
pub struct AnchorHeadingAdapter;

impl HeadingAdapter for AnchorHeadingAdapter {
    fn enter(&self, level: u8, content: &str, id: Option<&str>) -> String {
        let id = id
            .map(|s| s.to_string())
            .unwrap_or_else(|| generate_anchor_id(content));

        format!(
            "<h{level} id=\"{id}\"><a href=\"#{id}\" class=\"anchor\">#</a>",
            level = level,
            id = id
        )
    }

    fn exit(&self, level: u8) -> String {
        format!("</h{level}>", level = level)
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
    fn test_plugins_default() {
        let plugins = Plugins::new();
        assert!(plugins.is_empty());
        assert!(plugins.syntax_highlighter().is_none());
        assert!(plugins.heading_adapter().is_none());
    }

    #[test]
    fn test_syntax_highlighter() {
        let mut plugins = Plugins::new();
        assert!(plugins.is_empty());

        plugins.set_syntax_highlighter(Box::new(DefaultSyntaxHighlighter));
        assert!(!plugins.is_empty());
        assert!(plugins.syntax_highlighter().is_some());
    }

    #[test]
    fn test_heading_adapter() {
        let mut plugins = Plugins::new();
        plugins.set_heading_adapter(Box::new(AnchorHeadingAdapter));
        assert!(plugins.heading_adapter().is_some());
    }

    #[test]
    fn test_codefence_renderer() {
        struct TestRenderer;
        impl CodefenceRendererAdapter for TestRenderer {
            fn render(&self, code: &str, _info: &str) -> Option<String> {
                Some(format!("<test>{}</test>", code))
            }
        }

        let mut plugins = Plugins::new();
        plugins.register_codefence_renderer("test", Box::new(TestRenderer));

        assert!(plugins.codefence_renderer("test").is_some());
        assert!(plugins.codefence_renderer("other").is_none());

        let renderer = plugins.codefence_renderer("test").unwrap();
        assert_eq!(renderer.render("hello", "test"), Some("<test>hello</test>".to_string()));
    }

    #[test]
    fn test_url_rewriter() {
        struct TestRewriter;
        impl UrlRewriter for TestRewriter {
            fn rewrite(&self, url: &str) -> String {
                format!("https://example.com/{}", url)
            }
        }

        let mut plugins = Plugins::new();
        plugins.set_link_url_rewriter(Box::new(TestRewriter));

        let rewriter = plugins.link_url_rewriter().unwrap();
        assert_eq!(rewriter.rewrite("page"), "https://example.com/page");
    }

    #[test]
    fn test_default_syntax_highlighter() {
        let highlighter = DefaultSyntaxHighlighter;

        // Test without language
        let result = highlighter.highlight("fn main() {}", None);
        assert!(result.contains("<pre><code>"));
        assert!(result.contains("fn main() {}"));

        // Test with language
        let result = highlighter.highlight("fn main() {}", Some("rust"));
        assert!(result.contains("class=\"language-rust\""));

        // Test HTML escaping
        let result = highlighter.highlight("<script>", None);
        assert!(result.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_anchor_heading_adapter() {
        let adapter = AnchorHeadingAdapter;

        let open = adapter.enter(1, "Hello World", None);
        assert!(open.contains("<h1"));
        assert!(open.contains("id=\"hello-world\""));
        assert!(open.contains("<a href=\"#hello-world\""));

        let close = adapter.exit(1);
        assert_eq!(close, "</h1>");
    }

    #[test]
    fn test_anchor_heading_with_explicit_id() {
        let adapter = AnchorHeadingAdapter;

        let open = adapter.enter(2, "Hello World", Some("custom-id"));
        assert!(open.contains("id=\"custom-id\""));
        assert!(open.contains("<a href=\"#custom-id\""));
    }

    #[test]
    fn test_generate_anchor_id() {
        assert_eq!(generate_anchor_id("Hello World"), "hello-world");
        assert_eq!(generate_anchor_id("Test 123"), "test-123");
        assert_eq!(generate_anchor_id("Special!@#Chars"), "specialchars");
        assert_eq!(generate_anchor_id("Multiple   Spaces"), "multiple---spaces");
    }

    #[test]
    fn test_plugins_debug() {
        let mut plugins = Plugins::new();
        plugins.set_syntax_highlighter(Box::new(DefaultSyntaxHighlighter));

        let debug = format!("{:?}", plugins);
        assert!(debug.contains("has_syntax_highlighter: true"));
    }
}
