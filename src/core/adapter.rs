//! AST adapters for rendering customization.
//!
//! This module provides adapter traits for customizing various aspects of
//! Markdown rendering, such as syntax highlighting, code fence rendering,
//! heading rendering, and URL rewriting.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;

/// Adapter trait for syntax highlighting.
///
/// This trait allows customization of how code blocks are highlighted
/// during HTML rendering.
pub trait SyntaxHighlighterAdapter {
    /// Write highlighted code to the output.
    ///
    /// # Arguments
    ///
    /// * `output` - The output writer
    /// * `lang` - The language of the code (e.g., "rust", "python")
    /// * `code` - The code to highlight
    ///
    /// # Returns
    ///
    /// The result of the write operation
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> std::fmt::Result;

    /// Write the opening `<pre>` tag.
    ///
    /// # Arguments
    ///
    /// * `output` - The output writer
    /// * `attributes` - The attributes for the tag
    ///
    /// # Returns
    ///
    /// The result of the write operation
    fn write_pre_tag<'s>(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<&str, Cow<'s, str>>,
    ) -> std::fmt::Result;

    /// Write the opening `<code>` tag.
    ///
    /// # Arguments
    ///
    /// * `output` - The output writer
    /// * `attributes` - The attributes for the tag
    ///
    /// # Returns
    ///
    /// The result of the write operation
    fn write_code_tag<'s>(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<&str, Cow<'s, str>>,
    ) -> std::fmt::Result;
}

/// Adapter trait for code fence rendering.
///
/// This trait allows customization of how code fences (fenced code blocks)
/// are rendered.
pub trait CodefenceRendererAdapter {
    /// Check if this adapter can handle the given code fence info string.
    ///
    /// # Arguments
    ///
    /// * `info` - The info string from the code fence (e.g., "rust", "python")
    ///
    /// # Returns
    ///
    /// `true` if this adapter can handle the code fence
    fn is_codefence(&self, info: &str) -> bool;

    /// Render the code fence.
    ///
    /// # Arguments
    ///
    /// * `info` - The info string from the code fence
    /// * `content` - The content of the code fence
    ///
    /// # Returns
    ///
    /// The rendered HTML string, or `None` to use default rendering
    fn render_codefence(&self, info: &str, content: &str) -> Option<String>;
}

/// Adapter trait for heading rendering.
///
/// This trait allows customization of how headings are rendered.
pub trait HeadingAdapter {
    /// Render a heading.
    ///
    /// # Arguments
    ///
    /// * `level` - The heading level (1-6)
    /// * `content` - The heading content
    /// * `id` - An optional ID for the heading
    ///
    /// # Returns
    ///
    /// The rendered HTML string, or `None` to use default rendering
    fn render_heading(
        &self,
        level: u8,
        content: &str,
        id: Option<&str>,
    ) -> Option<String>;
}

/// Adapter trait for URL rewriting.
///
/// This trait allows customization of how URLs are rewritten during rendering.
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
    fn rewrite_url(&self, url: &str) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple test implementation of SyntaxHighlighterAdapter
    struct TestSyntaxHighlighter;

    impl SyntaxHighlighterAdapter for TestSyntaxHighlighter {
        fn write_highlighted(
            &self,
            output: &mut dyn Write,
            lang: Option<&str>,
            code: &str,
        ) -> std::fmt::Result {
            if let Some(lang) = lang {
                write!(output, "<span class=\"lang-{lang}\">{code}</span>")
            } else {
                write!(output, "<span>{code}</span>")
            }
        }

        fn write_pre_tag<'s>(
            &self,
            output: &mut dyn Write,
            _attributes: HashMap<&str, Cow<'s, str>>,
        ) -> std::fmt::Result {
            write!(output, "<pre>")
        }

        fn write_code_tag<'s>(
            &self,
            output: &mut dyn Write,
            _attributes: HashMap<&str, Cow<'s, str>>,
        ) -> std::fmt::Result {
            write!(output, "<code>")
        }
    }

    /// A simple test implementation of CodefenceRendererAdapter
    struct TestCodefenceRenderer;

    impl CodefenceRendererAdapter for TestCodefenceRenderer {
        fn is_codefence(&self, info: &str) -> bool {
            info == "mermaid" || info == "graphviz"
        }

        fn render_codefence(&self, info: &str, content: &str) -> Option<String> {
            if info == "mermaid" {
                Some(format!(
                    "<div class=\"mermaid\">{}</div>",
                    html_escape(content)
                ))
            } else if info == "graphviz" {
                Some(format!(
                    "<div class=\"graphviz\">{}</div>",
                    html_escape(content)
                ))
            } else {
                None
            }
        }
    }

    /// A simple test implementation of HeadingAdapter
    struct TestHeadingAdapter;

    impl HeadingAdapter for TestHeadingAdapter {
        fn render_heading(
            &self,
            level: u8,
            content: &str,
            id: Option<&str>,
        ) -> Option<String> {
            let id_attr = id.map(|i| format!(" id=\"{}\"", i)).unwrap_or_default();
            Some(format!(
                "<h{level}{id_attr} class=\"custom-heading\">{content}</h{level}>"
            ))
        }
    }

    /// A simple test implementation of UrlRewriter
    struct TestUrlRewriter;

    impl UrlRewriter for TestUrlRewriter {
        fn rewrite_url(&self, url: &str) -> String {
            if url.starts_with("http://") {
                url.replacen("http://", "https://", 1)
            } else {
                url.to_string()
            }
        }
    }

    /// Simple HTML escape helper
    fn html_escape(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    #[test]
    fn test_syntax_highlighter_adapter() {
        let highlighter = TestSyntaxHighlighter;
        let mut output = String::new();

        // Test with language
        highlighter
            .write_highlighted(&mut output, Some("rust"), "fn main() {}")
            .unwrap();
        assert_eq!(output, "<span class=\"lang-rust\">fn main() {}</span>");

        // Test without language
        output.clear();
        highlighter
            .write_highlighted(&mut output, None, "plain text")
            .unwrap();
        assert_eq!(output, "<span>plain text</span>");
    }

    #[test]
    fn test_syntax_highlighter_pre_tag() {
        let highlighter = TestSyntaxHighlighter;
        let mut output = String::new();
        let attributes = HashMap::new();

        highlighter.write_pre_tag(&mut output, attributes).unwrap();
        assert_eq!(output, "<pre>");
    }

    #[test]
    fn test_syntax_highlighter_code_tag() {
        let highlighter = TestSyntaxHighlighter;
        let mut output = String::new();
        let attributes = HashMap::new();

        highlighter.write_code_tag(&mut output, attributes).unwrap();
        assert_eq!(output, "<code>");
    }

    #[test]
    fn test_codefence_renderer_adapter() {
        let renderer = TestCodefenceRenderer;

        // Test supported codefences
        assert!(renderer.is_codefence("mermaid"));
        assert!(renderer.is_codefence("graphviz"));
        assert!(!renderer.is_codefence("rust"));
        assert!(!renderer.is_codefence(""));

        // Test mermaid rendering
        let result = renderer.render_codefence("mermaid", "graph TD; A-->B;");
        assert!(result.is_some());
        assert!(result.unwrap().contains("mermaid"));

        // Test graphviz rendering
        let result = renderer.render_codefence("graphviz", "digraph { A -> B }");
        assert!(result.is_some());
        assert!(result.unwrap().contains("graphviz"));

        // Test unsupported codefence returns None
        let result = renderer.render_codefence("rust", "fn main() {}");
        assert!(result.is_none());
    }

    #[test]
    fn test_heading_adapter() {
        let adapter = TestHeadingAdapter;

        // Test with ID
        let result = adapter.render_heading(1, "Title", Some("title-id"));
        assert!(result.is_some());
        let html = result.unwrap();
        assert!(html.contains("<h1"));
        assert!(html.contains("id=\"title-id\""));
        assert!(html.contains("class=\"custom-heading\""));
        assert!(html.contains("Title"));

        // Test without ID
        let result = adapter.render_heading(2, "Subtitle", None);
        assert!(result.is_some());
        let html = result.unwrap();
        assert!(html.contains("<h2"));
        assert!(!html.contains("id="));
    }

    #[test]
    fn test_url_rewriter_adapter() {
        let rewriter = TestUrlRewriter;

        // Test HTTP to HTTPS conversion
        assert_eq!(
            rewriter.rewrite_url("http://example.com"),
            "https://example.com"
        );

        // Test HTTPS stays the same
        assert_eq!(
            rewriter.rewrite_url("https://example.com"),
            "https://example.com"
        );

        // Test other URLs stay the same
        assert_eq!(
            rewriter.rewrite_url("/path/to/resource"),
            "/path/to/resource"
        );
        assert_eq!(
            rewriter.rewrite_url("mailto:test@example.com"),
            "mailto:test@example.com"
        );
    }

    #[test]
    fn test_html_escape_helper() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("&"), "&amp;");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(
            html_escape("<a href=\"test\">"),
            "&lt;a href=&quot;test&quot;&gt;"
        );
    }
}
