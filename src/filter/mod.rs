//! Filter system for clmd.
//!
//! This module provides a flexible filter system for transforming documents,
//! inspired by Pandoc's filter architecture. Filters can be used to modify
//! the AST between parsing and rendering.
//!
//! # Filter Types
//!
//! - **Native Filters** - Rust functions that operate on the AST
//! - **JSON Filters** - External programs that receive and return JSON
//! - **Lua Filters** - Lua scripts for document transformation
//!
//! # Example
//!
//! ```ignore
//! use clmd::filter::{Filter, FilterChain};
//! use clmd::arena::{NodeArena, NodeId};
//!
//! let mut chain = FilterChain::new();
//! chain.add(Filter::header_shift(1)); // Increase header levels by 1
//!
//! // Apply to document
//! // let (arena, root) = chain.apply(arena, root).unwrap();
//! ```

use crate::core::arena::{NodeArena, NodeId};
use std::fmt;
use std::path::PathBuf;

/// A filter that can transform a document.
#[derive(Clone)]
pub enum Filter {
    /// A native Rust filter function.
    Native(NativeFilter),

    /// An external JSON filter program.
    JSON(JSONFilter),

    /// A Lua filter script.
    Lua(LuaFilter),

    /// Built-in citeproc filter.
    Citeproc,

    /// Header level shift filter.
    HeaderShift(i32),

    /// Link transformation filter.
    LinkTransform {
        /// Base URL for relative links.
        base_url: Option<String>,
        /// Whether to make all links absolute.
        absolute_only: bool,
    },

    /// Image transformation filter.
    ImageTransform {
        /// Base URL for relative image paths.
        base_url: Option<String>,
        /// Whether to embed images as data URIs.
        embed_images: bool,
    },
}

impl fmt::Debug for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(_) => f.debug_tuple("Native").finish(),
            Self::JSON(j) => f.debug_tuple("JSON").field(&j.path).finish(),
            Self::Lua(l) => f.debug_tuple("Lua").field(&l.path).finish(),
            Self::Citeproc => f.debug_struct("Citeproc").finish(),
            Self::HeaderShift(n) => f.debug_tuple("HeaderShift").field(n).finish(),
            Self::LinkTransform {
                base_url,
                absolute_only,
            } => f
                .debug_struct("LinkTransform")
                .field("base_url", base_url)
                .field("absolute_only", absolute_only)
                .finish(),
            Self::ImageTransform {
                base_url,
                embed_images,
            } => f
                .debug_struct("ImageTransform")
                .field("base_url", base_url)
                .field("embed_images", embed_images)
                .finish(),
        }
    }
}

impl Filter {
    /// Create a header shift filter.
    ///
    /// Positive values increase header levels, negative values decrease them.
    /// Levels are clamped to the valid range (1-6).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::filter::Filter;
    ///
    /// let filter = Filter::header_shift(1); // h1 -> h2, h2 -> h3, etc.
    /// ```
    pub fn header_shift(shift: i32) -> Self {
        Self::HeaderShift(shift)
    }

    /// Create a link transformation filter.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::filter::Filter;
    ///
    /// let filter = Filter::link_transform()
    ///     .with_base_url("https://example.com/")
    ///     .absolute_only();
    /// ```
    pub fn link_transform() -> LinkTransformBuilder {
        LinkTransformBuilder::default()
    }

    /// Create an image transformation filter.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::filter::Filter;
    ///
    /// let filter = Filter::image_transform()
    ///     .with_base_url("https://example.com/images/")
    ///     .embed_images();
    /// ```
    pub fn image_transform() -> ImageTransformBuilder {
        ImageTransformBuilder::default()
    }

    /// Create a JSON filter from a file path.
    pub fn json<P: Into<PathBuf>>(path: P) -> Self {
        Self::JSON(JSONFilter {
            path: path.into(),
            args: Vec::new(),
        })
    }

    /// Create a Lua filter from a file path.
    pub fn lua<P: Into<PathBuf>>(path: P) -> Self {
        Self::Lua(LuaFilter {
            path: path.into(),
            args: Vec::new(),
        })
    }

    /// Get the name of this filter.
    pub fn name(&self) -> String {
        match self {
            Self::Native(_) => "native".to_string(),
            Self::JSON(f) => format!("json: {}", f.path.display()),
            Self::Lua(f) => format!("lua: {}", f.path.display()),
            Self::Citeproc => "citeproc".to_string(),
            Self::HeaderShift(n) => format!("header-shift({})", n),
            Self::LinkTransform { .. } => "link-transform".to_string(),
            Self::ImageTransform { .. } => "image-transform".to_string(),
        }
    }

    /// Apply this filter to a document.
    ///
    /// # Arguments
    ///
    /// * `arena` - The node arena containing the AST
    /// * `root` - The root node ID
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or a FilterError if the filter fails.
    pub fn apply(&self, arena: &mut NodeArena, root: NodeId) -> Result<(), FilterError> {
        match self {
            Self::HeaderShift(shift) => filters::apply_header_shift(arena, root, *shift),
            Self::LinkTransform {
                base_url,
                absolute_only,
            } => filters::apply_link_transform(
                arena,
                root,
                base_url.as_deref(),
                *absolute_only,
            ),
            Self::ImageTransform {
                base_url,
                embed_images,
            } => filters::apply_image_transform(
                arena,
                root,
                base_url.as_deref(),
                *embed_images,
            ),
            Self::Native(f) => f.apply(arena, root),
            _ => Err(FilterError::NotImplemented(self.name())),
        }
    }
}

/// A native Rust filter.
#[derive(Clone)]
pub struct NativeFilter {
    /// Name of the filter.
    pub name: String,
    /// The filter function.
    pub apply: fn(&mut NodeArena, NodeId) -> Result<(), FilterError>,
}

impl fmt::Debug for NativeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeFilter")
            .field("name", &self.name)
            .finish()
    }
}

impl NativeFilter {
    /// Apply this filter.
    pub fn apply(&self, arena: &mut NodeArena, root: NodeId) -> Result<(), FilterError> {
        (self.apply)(arena, root)
    }
}

/// A JSON filter configuration.
#[derive(Debug, Clone)]
pub struct JSONFilter {
    /// Path to the filter executable or script.
    pub path: PathBuf,
    /// Additional arguments to pass to the filter.
    pub args: Vec<String>,
}

/// A Lua filter configuration.
#[derive(Debug, Clone)]
pub struct LuaFilter {
    /// Path to the Lua script.
    pub path: PathBuf,
    /// Additional arguments to pass to the filter.
    pub args: Vec<String>,
}

/// Builder for link transformation filters.
#[derive(Debug, Default)]
pub struct LinkTransformBuilder {
    base_url: Option<String>,
    absolute_only: bool,
}

impl LinkTransformBuilder {
    /// Set the base URL for relative links.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Only allow absolute URLs.
    pub fn absolute_only(mut self) -> Self {
        self.absolute_only = true;
        self
    }

    /// Build the filter.
    pub fn build(self) -> Filter {
        Filter::LinkTransform {
            base_url: self.base_url,
            absolute_only: self.absolute_only,
        }
    }
}

/// Builder for image transformation filters.
#[derive(Debug, Default)]
pub struct ImageTransformBuilder {
    base_url: Option<String>,
    embed_images: bool,
}

impl ImageTransformBuilder {
    /// Set the base URL for relative image paths.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Embed images as data URIs.
    pub fn embed_images(mut self) -> Self {
        self.embed_images = true;
        self
    }

    /// Build the filter.
    pub fn build(self) -> Filter {
        Filter::ImageTransform {
            base_url: self.base_url,
            embed_images: self.embed_images,
        }
    }
}

/// Error type for filter operations.
#[derive(Debug, Clone)]
pub enum FilterError {
    /// Filter not implemented.
    NotImplemented(String),
    /// Invalid filter configuration.
    InvalidConfig(String),
    /// Filter execution failed.
    ExecutionFailed(String),
    /// IO error.
    Io(String),
    /// JSON parsing error.
    Json(String),
    /// Lua execution error.
    Lua(String),
}

impl fmt::Display for FilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented(name) => write!(f, "Filter not implemented: {}", name),
            Self::InvalidConfig(msg) => write!(f, "Invalid filter config: {}", msg),
            Self::ExecutionFailed(msg) => write!(f, "Filter execution failed: {}", msg),
            Self::Io(msg) => write!(f, "IO error: {}", msg),
            Self::Json(msg) => write!(f, "JSON error: {}", msg),
            Self::Lua(msg) => write!(f, "Lua error: {}", msg),
        }
    }
}

impl std::error::Error for FilterError {}

impl From<std::io::Error> for FilterError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<serde_json::Error> for FilterError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e.to_string())
    }
}

/// A chain of filters to apply in sequence.
#[derive(Debug, Clone, Default)]
pub struct FilterChain {
    filters: Vec<Filter>,
}

impl FilterChain {
    /// Create a new empty filter chain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a filter to the chain.
    pub fn add(&mut self, filter: Filter) -> &mut Self {
        self.filters.push(filter);
        self
    }

    /// Add multiple filters to the chain.
    pub fn extend(&mut self, filters: impl IntoIterator<Item = Filter>) -> &mut Self {
        self.filters.extend(filters);
        self
    }

    /// Get the number of filters in the chain.
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// Clear all filters from the chain.
    pub fn clear(&mut self) {
        self.filters.clear();
    }

    /// Apply all filters in the chain to a document.
    ///
    /// Filters are applied in the order they were added.
    pub fn apply(&self, arena: &mut NodeArena, root: NodeId) -> Result<(), FilterError> {
        for filter in &self.filters {
            filter.apply(arena, root)?;
        }
        Ok(())
    }

    /// Apply all filters with verbose logging.
    pub fn apply_verbose(
        &self,
        arena: &mut NodeArena,
        root: NodeId,
    ) -> Result<Vec<FilterResult>, FilterError> {
        let mut results = Vec::new();

        for filter in &self.filters {
            let start = std::time::Instant::now();
            let result = filter.apply(arena, root);
            let elapsed = start.elapsed();

            results.push(FilterResult {
                filter_name: filter.name(),
                duration: elapsed,
                success: result.is_ok(),
            });

            result?;
        }

        Ok(results)
    }

    /// Get an iterator over the filters.
    pub fn iter(&self) -> impl Iterator<Item = &Filter> {
        self.filters.iter()
    }
}

/// Result of applying a single filter.
#[derive(Debug, Clone)]
pub struct FilterResult {
    /// Name of the filter.
    pub filter_name: String,
    /// Duration of the filter execution.
    pub duration: std::time::Duration,
    /// Whether the filter succeeded.
    pub success: bool,
}

/// Built-in filter implementations.
pub mod filters {
    use super::*;
    use crate::nodes::NodeValue;

    /// Apply header level shift.
    pub fn apply_header_shift(
        arena: &mut NodeArena,
        _root: NodeId,
        shift: i32,
    ) -> Result<(), FilterError> {
        // Iterate through all nodes and shift headers
        for node_id in 0..arena.len() {
            let node_id = node_id as u32;
            let node = arena.get_mut(node_id);
            if let NodeValue::Heading(ref mut heading) = node.value {
                let new_level = (heading.level as i32 + shift).clamp(1, 6) as u8;
                heading.level = new_level;
            }
        }
        Ok(())
    }

    /// Apply link transformation.
    pub fn apply_link_transform(
        arena: &mut NodeArena,
        _root: NodeId,
        base_url: Option<&str>,
        _absolute_only: bool,
    ) -> Result<(), FilterError> {
        for node_id in 0..arena.len() {
            let node_id = node_id as u32;
            let node = arena.get_mut(node_id);
            if let NodeValue::Link(ref mut link) = node.value {
                if let Some(base) = base_url {
                    if link.url.starts_with("./") || link.url.starts_with("../") {
                        link.url = format!("{}{}", base, link.url);
                    }
                }
            }
        }
        Ok(())
    }

    /// Apply image transformation.
    pub fn apply_image_transform(
        arena: &mut NodeArena,
        _root: NodeId,
        base_url: Option<&str>,
        _embed_images: bool,
    ) -> Result<(), FilterError> {
        for node_id in 0..arena.len() {
            let node_id = node_id as u32;
            let node = arena.get_mut(node_id);
            if let NodeValue::Image(ref mut image) = node.value {
                if let Some(base) = base_url {
                    if image.url.starts_with("./") || image.url.starts_with("../") {
                        image.url = format!("{}{}", base, image.url);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};
    use crate::nodes::{NodeHeading, NodeValue};

    #[test]
    fn test_filter_chain() {
        let mut chain = FilterChain::new();
        chain.add(Filter::header_shift(1));

        assert_eq!(chain.len(), 1);
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_header_shift_filter() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        TreeOps::append_child(&mut arena, root, heading);

        let filter = Filter::header_shift(1);
        filter.apply(&mut arena, root).unwrap();

        let heading_node = arena.get(heading);
        if let NodeValue::Heading(h) = &heading_node.value {
            assert_eq!(h.level, 2);
        } else {
            panic!("Expected heading node");
        }
    }

    #[test]
    fn test_header_shift_clamping() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 6,
            setext: false,
            closed: false,
        })));
        TreeOps::append_child(&mut arena, root, heading);

        let filter = Filter::header_shift(1);
        filter.apply(&mut arena, root).unwrap();

        let heading_node = arena.get(heading);
        if let NodeValue::Heading(h) = &heading_node.value {
            assert_eq!(h.level, 6); // Should be clamped to 6
        } else {
            panic!("Expected heading node");
        }
    }

    #[test]
    fn test_filter_names() {
        assert_eq!(Filter::header_shift(1).name(), "header-shift(1)");
        assert_eq!(Filter::Citeproc.name(), "citeproc");
        assert!(Filter::link_transform()
            .build()
            .name()
            .contains("link-transform"));
    }

    #[test]
    fn test_filter_result() {
        let result = FilterResult {
            filter_name: "test".to_string(),
            duration: std::time::Duration::from_millis(10),
            success: true,
        };

        assert_eq!(result.filter_name, "test");
        assert!(result.success);
    }
}
