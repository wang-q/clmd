//! Filter system for clmd.
//!
//! This module provides a filter system for document transformations,
//! inspired by Pandoc's filter system (JSON filters, Lua filters, etc.)
//!
//! # Example
//!
//! ```
//! use clmd::filters::{Filter, FilterChain, NativeFilter};
//! use clmd::readers::Document;
//!
//! // Create a native filter
//! let filter = NativeFilter::new(|doc: &mut Document| {
//!     // Transform the document
//!     Ok(())
//! });
//!
//! // Create a filter chain
//! let chain = FilterChain::new()
//!     .add(Box::new(filter));
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::error::{ClmdError, ClmdResult};
use crate::readers::Document;
use std::fmt;

/// Trait for document filters.
///
/// Filters can transform documents in various ways.
pub trait Filter {
    /// Apply the filter to a document.
    ///
    /// # Arguments
    ///
    /// * `doc` - The document to transform
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or an error if the filter fails.
    fn apply(&self, doc: &mut Document) -> FilterResult<()>;

    /// Get the name of this filter.
    fn name(&self) -> &str;
}

/// Result type for filter operations.
pub type FilterResult<T> = Result<T, FilterError>;

/// Error type for filter operations.
#[derive(Debug, Clone)]
pub enum FilterError {
    /// Generic error message.
    Message(String),
    /// Filter execution error.
    Execution(String),
    /// Invalid filter configuration.
    Config(String),
    /// IO error.
    Io(String),
}

impl FilterError {
    /// Create a new filter error.
    pub fn new<S: Into<String>>(msg: S) -> Self {
        Self::Message(msg.into())
    }

    /// Create an execution error.
    pub fn execution<S: Into<String>>(msg: S) -> Self {
        Self::Execution(msg.into())
    }

    /// Create a config error.
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }
}

impl fmt::Display for FilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(msg) => write!(f, "{}", msg),
            Self::Execution(msg) => write!(f, "Execution error: {}", msg),
            Self::Config(msg) => write!(f, "Config error: {}", msg),
            Self::Io(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for FilterError {}

/// Native filter using a closure.
pub struct NativeFilter<F>
where
    F: Fn(&mut Document) -> FilterResult<()>,
{
    name: String,
    func: F,
}

impl<F> NativeFilter<F>
where
    F: Fn(&mut Document) -> FilterResult<()>,
{
    /// Create a new native filter.
    pub fn new<S: Into<String>>(name: S, func: F) -> Self {
        Self {
            name: name.into(),
            func,
        }
    }
}

impl<F> Filter for NativeFilter<F>
where
    F: Fn(&mut Document) -> FilterResult<()>,
{
    fn apply(&self, doc: &mut Document) -> FilterResult<()> {
        (self.func)(doc)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Filter chain for applying multiple filters.
pub struct FilterChain {
    filters: Vec<Box<dyn Filter>>,
}

impl FilterChain {
    /// Create a new empty filter chain.
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    /// Add a filter to the chain.
    pub fn add(mut self, filter: Box<dyn Filter>) -> Self {
        self.filters.push(filter);
        self
    }

    /// Apply all filters in the chain.
    pub fn apply(&self, doc: &mut Document) -> FilterResult<()> {
        for filter in &self.filters {
            filter.apply(doc)?;
        }
        Ok(())
    }

    /// Get the number of filters in the chain.
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }
}

impl Default for FilterChain {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON filter support (placeholder for future implementation).
pub struct JsonFilter {
    path: std::path::PathBuf,
}

impl JsonFilter {
    /// Create a new JSON filter.
    pub fn new<P: Into<std::path::PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }
}

impl Filter for JsonFilter {
    fn apply(&self, _doc: &mut Document) -> FilterResult<()> {
        // Placeholder: JSON filter implementation would:
        // 1. Serialize document to JSON
        // 2. Execute external filter program
        // 3. Deserialize result back to document
        Err(FilterError::new("JSON filters not yet implemented"))
    }

    fn name(&self) -> &str {
        "json"
    }
}

/// Lua filter support (placeholder for future implementation).
pub struct LuaFilter {
    script: String,
}

impl LuaFilter {
    /// Create a new Lua filter.
    pub fn new<S: Into<String>>(script: S) -> Self {
        Self { script: script.into() }
    }
}

impl Filter for LuaFilter {
    fn apply(&self, _doc: &mut Document) -> FilterResult<()> {
        // Placeholder: Lua filter implementation would:
        // 1. Use mlua or similar crate to execute Lua script
        // 2. Pass document to Lua
        // 3. Apply transformations
        Err(FilterError::new("Lua filters not yet implemented"))
    }

    fn name(&self) -> &str {
        "lua"
    }
}

/// Filter registry for managing available filters.
pub struct FilterRegistry {
    filters: std::collections::HashMap<String, Box<dyn Filter>>,
}

impl FilterRegistry {
    /// Create a new filter registry.
    pub fn new() -> Self {
        Self {
            filters: std::collections::HashMap::new(),
        }
    }

    /// Register a filter.
    pub fn register<F: Filter + 'static>(&mut self, name: impl Into<String>, filter: F) {
        self.filters.insert(name.into(), Box::new(filter));
    }

    /// Get a filter by name.
    pub fn get(&self, name: &str) -> Option<&dyn Filter> {
        self.filters.get(name).map(|f| f.as_ref())
    }

    /// Check if a filter is registered.
    pub fn has(&self, name: &str) -> bool {
        self.filters.contains_key(name)
    }

    /// List registered filter names.
    pub fn list(&self) -> Vec<&str> {
        self.filters.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for FilterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_error() {
        let err = FilterError::new("test error");
        assert_eq!(err.to_string(), "test error");

        let err = FilterError::execution("execution failed");
        assert!(err.to_string().contains("Execution error"));
    }

    #[test]
    fn test_native_filter() {
        let filter = NativeFilter::new("test", |_doc: &mut Document| Ok(()));
        assert_eq!(filter.name(), "test");
    }

    #[test]
    fn test_filter_chain() {
        let chain = FilterChain::new()
            .add(Box::new(NativeFilter::new("f1", |_doc| Ok(()))))
            .add(Box::new(NativeFilter::new("f2", |_doc| Ok(()))));

        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_filter_chain_empty() {
        let chain = FilterChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn test_filter_registry() {
        let mut registry = FilterRegistry::new();
        registry.register("test", NativeFilter::new("test", |_doc| Ok(())));

        assert!(registry.has("test"));
        assert!(!registry.has("missing"));
        assert!(registry.get("test").is_some());
    }

    #[test]
    fn test_json_filter() {
        let filter = JsonFilter::new("/path/to/filter");
        assert_eq!(filter.name(), "json");
    }

    #[test]
    fn test_lua_filter() {
        let filter = LuaFilter::new("return doc");
        assert_eq!(filter.name(), "lua");
    }
}
