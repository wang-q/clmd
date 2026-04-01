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
