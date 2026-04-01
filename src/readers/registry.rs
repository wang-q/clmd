//! Reader registry for managing document format readers.
//!
//! This module provides a centralized registry for document readers,
//! allowing formats to be looked up by name or file extension.
//!
//! # Example
//!
//! ```ignore
//! use clmd::readers::{ReaderRegistry, ReaderOptions};
//!
//! let registry = ReaderRegistry::new();
//!
//! // Get a reader by name
//! if let Some(reader) = registry.get("markdown") {
//!     println!("Found reader: {}", reader.name());
//! }
//!
//! // Get a reader by file extension
//! if let Some(name) = registry.get_by_extension("md") {
//!     println!("Format for .md files: {}", name);
//! }
//! ```

use std::collections::HashMap;

use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdError;
use crate::readers::{BoxedReader, Reader, ReaderOptions};

/// A registry of document readers.
///
/// The registry maintains a mapping from format names to reader implementations,
/// and provides methods for looking up readers by name or file extension.
#[derive(Default)]
pub struct ReaderRegistry {
    /// Map from format name to reader.
    readers: HashMap<String, BoxedReader>,
    /// Map from file extension to format name.
    extension_map: HashMap<String, String>,
}

impl std::fmt::Debug for ReaderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReaderRegistry")
            .field("readers", &self.readers.keys().collect::<Vec<_>>())
            .field("extension_map", &self.extension_map)
            .finish()
    }
}

impl ReaderRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            readers: HashMap::new(),
            extension_map: HashMap::new(),
        }
    }

    /// Create a new registry with all built-in readers registered.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_builtin_readers();
        registry
    }

    /// Register a reader.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::readers::{ReaderRegistry, MarkdownReader};
    ///
    /// let mut registry = ReaderRegistry::new();
    /// registry.register(MarkdownReader::new());
    /// ```
    pub fn register(&mut self, reader: impl Reader + 'static) {
        let name = reader.name().to_lowercase();
        let extensions: Vec<_> =
            reader.extensions().iter().map(|e| e.to_string()).collect();

        self.readers.insert(name.clone(), Box::new(reader));

        // Register extensions
        for ext in extensions {
            self.extension_map.insert(ext.to_lowercase(), name.clone());
        }
    }

    /// Get a reader by name.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::readers::ReaderRegistry;
    ///
    /// let registry = ReaderRegistry::with_defaults();
    /// if let Some(reader) = registry.get("markdown") {
    ///     println!("Found reader: {}", reader.name());
    /// }
    /// ```
    pub fn get(&self, name: &str) -> Option<&dyn Reader> {
        self.readers
            .get(name.to_lowercase().as_str())
            .map(|r| r.as_ref())
    }

    /// Get a reader by file extension.
    ///
    /// Returns the format name for the given extension, which can then be
    /// used with [`get`](Self::get) to retrieve the actual reader.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::readers::ReaderRegistry;
    ///
    /// let registry = ReaderRegistry::with_defaults();
    /// if let Some(format) = registry.get_by_extension("md") {
    ///     println!("Format for .md: {}", format);
    /// }
    /// ```
    pub fn get_by_extension(&self, ext: &str) -> Option<&str> {
        self.extension_map
            .get(ext.to_lowercase().as_str())
            .map(|s| s.as_str())
    }

    /// Get a reader by file path.
    ///
    /// This extracts the extension from the path and looks up the corresponding
    /// reader.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::readers::ReaderRegistry;
    /// use std::path::Path;
    ///
    /// let registry = ReaderRegistry::with_defaults();
    /// if let Some(reader) = registry.get_by_path(Path::new("document.md")) {
    ///     println!("Found reader for document.md");
    /// }
    /// ```
    pub fn get_by_path(&self, path: &std::path::Path) -> Option<&dyn Reader> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.get_by_extension(ext))
            .and_then(|name| self.get(name))
    }

    /// Check if a reader is registered for the given format name.
    pub fn contains(&self, name: &str) -> bool {
        self.readers.contains_key(name.to_lowercase().as_str())
    }

    /// Get a list of all registered format names.
    pub fn list(&self) -> Vec<&str> {
        self.readers.keys().map(|s| s.as_str()).collect()
    }

    /// Get a list of all registered file extensions.
    pub fn extensions(&self) -> Vec<&str> {
        self.extension_map.keys().map(|s| s.as_str()).collect()
    }

    /// Get the number of registered readers.
    pub fn len(&self) -> usize {
        self.readers.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.readers.is_empty()
    }

    /// Remove a reader from the registry.
    pub fn remove(&mut self, name: &str) -> Option<BoxedReader> {
        let name = name.to_lowercase();

        // Remove from extension map
        self.extension_map.retain(|_, v| v != &name);

        // Remove reader
        self.readers.remove(&name)
    }

    /// Clear all readers from the registry.
    pub fn clear(&mut self) {
        self.readers.clear();
        self.extension_map.clear();
    }

    /// Register all built-in readers.
    fn register_builtin_readers(&mut self) {
        self.register(MarkdownReader::new());
        self.register(HtmlReader::new());
        self.register(CommonMarkReader::new());
    }
}

impl Clone for ReaderRegistry {
    fn clone(&self) -> Self {
        // Note: This creates a new registry with the same readers.
        // Since readers are trait objects, we can't directly clone them.
        // In practice, you'd typically create a new registry with_defaults().
        Self::with_defaults()
    }
}

/// Built-in Markdown reader.
#[derive(Debug, Clone, Copy)]
pub struct MarkdownReader;

impl MarkdownReader {
    /// Create a new Markdown reader.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for MarkdownReader {
    fn name(&self) -> &'static str {
        "markdown"
    }

    fn extensions(&self) -> &[&'static str] {
        &["md", "markdown", "mdown", "mkd", "mkdn"]
    }

    fn read_text<'c>(
        &self,
        input: &str,
        options: &ReaderOptions<'c>,
    ) -> Result<(NodeArena, NodeId), ClmdError> {
        crate::readers::read_markdown(input, options)
    }
}

/// Built-in HTML reader.
#[derive(Debug, Clone, Copy)]
pub struct HtmlReader;

impl HtmlReader {
    /// Create a new HTML reader.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for HtmlReader {
    fn name(&self) -> &'static str {
        "html"
    }

    fn extensions(&self) -> &[&'static str] {
        &["html", "htm", "xhtml"]
    }

    fn read_text<'c>(
        &self,
        input: &str,
        options: &ReaderOptions<'c>,
    ) -> Result<(NodeArena, NodeId), ClmdError> {
        crate::readers::read_html(input, options)
    }
}

/// Built-in CommonMark reader.
#[derive(Debug, Clone, Copy)]
pub struct CommonMarkReader;

impl CommonMarkReader {
    /// Create a new CommonMark reader.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CommonMarkReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for CommonMarkReader {
    fn name(&self) -> &'static str {
        "commonmark"
    }

    fn extensions(&self) -> &[&'static str] {
        &["commonmark", "cm"]
    }

    fn read_text<'c>(
        &self,
        input: &str,
        options: &ReaderOptions<'c>,
    ) -> Result<(NodeArena, NodeId), ClmdError> {
        crate::readers::read_commonmark(input, options)
    }
}

/// Get the default reader registry.
///
/// This is a lazily initialized global registry containing all built-in readers.
///
/// # Example
///
/// ```ignore
/// use clmd::readers::default_registry;
///
/// let registry = default_registry();
/// if let Some(reader) = registry.get("markdown") {
///     println!("Markdown reader is available");
/// }
/// ```ignore
pub fn default_registry() -> &'static ReaderRegistry {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<ReaderRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ReaderRegistry::with_defaults)
}

/// Get a reader by name from the default registry.
///
/// This is a convenience function that looks up a reader in the default registry.
///
/// # Example
///
/// ```ignore
/// use clmd::readers::get_reader;
///
/// if let Some(reader) = get_reader("markdown") {
///     println!("Found markdown reader");
/// }
/// ```ignore
pub fn get_reader(name: &str) -> Option<&'static dyn Reader> {
    default_registry().get(name)
}

/// Get a reader by file extension from the default registry.
///
/// This is a convenience function that looks up a reader by extension
/// in the default registry.
///
/// # Example
///
/// ```ignore
/// use clmd::readers::get_reader_by_extension;
///
/// if let Some(reader) = get_reader_by_extension("md") {
///     println!("Found reader for .md files");
/// }
/// ```ignore
pub fn get_reader_by_extension(ext: &str) -> Option<&'static dyn Reader> {
    default_registry()
        .get_by_extension(ext)
        .and_then(|name| default_registry().get(name))
}

/// Get a reader by file path from the default registry.
///
/// This is a convenience function that looks up a reader by file path
/// in the default registry.
///
/// # Example
///
/// ```ignore
/// use clmd::readers::get_reader_by_path;
/// use std::path::Path;
///
/// if let Some(reader) = get_reader_by_path(Path::new("document.md")) {
///     println!("Found reader for document.md");
/// }
/// ```ignore
pub fn get_reader_by_path(path: &std::path::Path) -> Option<&'static dyn Reader> {
    default_registry().get_by_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_registry_new() {
        let registry = ReaderRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_with_defaults() {
        let registry = ReaderRegistry::with_defaults();
        assert!(!registry.is_empty());
        assert!(registry.contains("markdown"));
        assert!(registry.contains("html"));
        assert!(registry.contains("commonmark"));
    }

    #[test]
    fn test_registry_get() {
        let registry = ReaderRegistry::with_defaults();

        let reader = registry.get("markdown").unwrap();
        assert_eq!(reader.name(), "markdown");

        let reader = registry.get("MARKDOWN").unwrap(); // case insensitive
        assert_eq!(reader.name(), "markdown");

        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_registry_get_by_extension() {
        let registry = ReaderRegistry::with_defaults();

        assert_eq!(registry.get_by_extension("md"), Some("markdown"));
        assert_eq!(registry.get_by_extension("MD"), Some("markdown")); // case insensitive
        assert_eq!(registry.get_by_extension("html"), Some("html"));
        assert_eq!(registry.get_by_extension("unknown"), None);
    }

    #[test]
    fn test_registry_get_by_path() {
        let registry = ReaderRegistry::with_defaults();

        let reader = registry.get_by_path(Path::new("document.md")).unwrap();
        assert_eq!(reader.name(), "markdown");

        let reader = registry
            .get_by_path(Path::new("/path/to/file.html"))
            .unwrap();
        assert_eq!(reader.name(), "html");

        assert!(registry.get_by_path(Path::new("no_extension")).is_none());
    }

    #[test]
    fn test_registry_list() {
        let registry = ReaderRegistry::with_defaults();
        let formats = registry.list();

        assert!(formats.contains(&"markdown"));
        assert!(formats.contains(&"html"));
        assert!(formats.contains(&"commonmark"));
    }

    #[test]
    fn test_registry_extensions() {
        let registry = ReaderRegistry::with_defaults();
        let extensions = registry.extensions();

        assert!(extensions.contains(&"md"));
        assert!(extensions.contains(&"html"));
        assert!(extensions.contains(&"htm"));
    }

    #[test]
    fn test_markdown_reader() {
        let reader = MarkdownReader::new();
        assert_eq!(reader.name(), "markdown");
        assert!(reader.supports_extension("md"));
        assert!(reader.supports_extension("markdown"));
        assert!(!reader.supports_extension("html"));
    }

    #[test]
    fn test_html_reader() {
        let reader = HtmlReader::new();
        assert_eq!(reader.name(), "html");
        assert!(reader.supports_extension("html"));
        assert!(reader.supports_extension("htm"));
        assert!(!reader.supports_extension("md"));
    }

    #[test]
    fn test_commonmark_reader() {
        let reader = CommonMarkReader::new();
        assert_eq!(reader.name(), "commonmark");
        assert!(reader.supports_extension("commonmark"));
        assert!(reader.supports_extension("cm"));
    }

    #[test]
    fn test_default_registry() {
        let registry = default_registry();
        assert!(registry.contains("markdown"));
        assert!(registry.contains("html"));
    }

    #[test]
    fn test_get_reader() {
        let reader = get_reader("markdown").unwrap();
        assert_eq!(reader.name(), "markdown");

        assert!(get_reader("unknown").is_none());
    }

    #[test]
    fn test_get_reader_by_extension() {
        let reader = get_reader_by_extension("md").unwrap();
        assert_eq!(reader.name(), "markdown");

        assert!(get_reader_by_extension("unknown").is_none());
    }

    #[test]
    fn test_get_reader_by_path() {
        let reader = get_reader_by_path(Path::new("test.md")).unwrap();
        assert_eq!(reader.name(), "markdown");

        assert!(get_reader_by_path(Path::new("no_extension")).is_none());
    }
}
