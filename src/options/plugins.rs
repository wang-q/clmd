//! Plugin system for extending Markdown rendering.
//!
//! This module provides the plugin infrastructure for customizing
//! code fence rendering, syntax highlighting, and heading adaptation.

use crate::core::adapter::{
    CodefenceRendererAdapter, HeadingAdapter, SyntaxHighlighterAdapter,
};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

/// Umbrella plugins struct.
#[derive(Default, Clone, Debug)]
pub struct Plugins<'p> {
    /// Configure render-time plugins.
    pub render: RenderPlugins<'p>,
}

impl<'p> Plugins<'p> {
    /// Create a new empty plugins collection
    pub fn new() -> Self {
        Self::default()
    }
}

/// Plugins for alternative rendering.
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

    /// Register a code fence renderer for a specific language
    pub fn register_codefence_renderer(
        &mut self,
        language: impl Into<String>,
        renderer: &'p dyn CodefenceRendererAdapter,
    ) {
        self.codefence_renderers.insert(language.into(), renderer);
    }
}

impl Debug for RenderPlugins<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugins_default() {
        let plugins = Plugins::default();
        let _ = &plugins.render;
    }

    #[test]
    fn test_plugins_new() {
        let plugins = Plugins::new();
        let _ = &plugins.render;
    }

    #[test]
    fn test_render_plugins_default() {
        let plugins = RenderPlugins::default();
        assert!(plugins.codefence_renderers.is_empty());
        assert!(plugins.codefence_syntax_highlighter.is_none());
        assert!(plugins.heading_adapter.is_none());
    }

    #[test]
    fn test_render_plugins_new() {
        let _plugins = RenderPlugins::new();
    }

    #[test]
    fn test_render_plugins_debug() {
        let plugins = RenderPlugins::default();
        let debug_str = format!("{:?}", plugins);
        assert!(debug_str.contains("RenderPlugins"));
    }
}
