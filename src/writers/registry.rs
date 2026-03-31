//! Writer registry for managing output format writers.
//!
//! This module provides a centralized registry for document writers,
//! allowing formats to be looked up by name or file extension.
//!
//! # Example
//!
//! ```
//! use clmd::writers::{WriterRegistry, WriterOptions};
//!
//! let registry = WriterRegistry::new();
//!
//! // Get a writer by name
//! if let Some(writer) = registry.get("html") {
//!     println!("Found writer: {}", writer.name());
//! }
//!
//! // Get a writer by file extension
//! if let Some(name) = registry.get_by_extension("html") {
//!     println!("Format for .html files: {}", name);
//! }
//! ```

use std::collections::HashMap;

use crate::arena::{NodeArena, NodeId};
use crate::error::ClmdError;
use crate::writers::{BoxedWriter, Writer, WriterOptions};

/// A registry of document writers.
///
/// The registry maintains a mapping from format names to writer implementations,
/// and provides methods for looking up writers by name or file extension.
#[derive(Default)]
pub struct WriterRegistry {
    /// Map from format name to writer.
    writers: HashMap<String, BoxedWriter>,
    /// Map from file extension to format name.
    extension_map: HashMap<String, String>,
}

impl std::fmt::Debug for WriterRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriterRegistry")
            .field("writers", &self.writers.keys().collect::<Vec<_>>())
            .field("extension_map", &self.extension_map)
            .finish()
    }
}

impl WriterRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            writers: HashMap::new(),
            extension_map: HashMap::new(),
        }
    }

    /// Create a new registry with all built-in writers registered.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_builtin_writers();
        registry
    }

    /// Register a writer.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::writers::{WriterRegistry, HtmlWriter};
    ///
    /// let mut registry = WriterRegistry::new();
    /// registry.register(HtmlWriter::new());
    /// ```
    pub fn register(&mut self, writer: impl Writer + 'static) {
        let name = writer.name().to_lowercase();
        let extensions: Vec<_> =
            writer.extensions().iter().map(|e| e.to_string()).collect();

        self.writers.insert(name.clone(), Box::new(writer));

        // Register extensions
        for ext in extensions {
            self.extension_map.insert(ext.to_lowercase(), name.clone());
        }
    }

    /// Get a writer by name.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::writers::WriterRegistry;
    ///
    /// let registry = WriterRegistry::with_defaults();
    /// if let Some(writer) = registry.get("html") {
    ///     println!("Found writer: {}", writer.name());
    /// }
    /// ```
    pub fn get(&self, name: &str) -> Option<&dyn Writer> {
        self.writers
            .get(name.to_lowercase().as_str())
            .map(|w| w.as_ref())
    }

    /// Get a writer by file extension.
    ///
    /// Returns the format name for the given extension, which can then be
    /// used with [`get`](Self::get) to retrieve the actual writer.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::writers::WriterRegistry;
    ///
    /// let registry = WriterRegistry::with_defaults();
    /// if let Some(format) = registry.get_by_extension("html") {
    ///     println!("Format for .html: {}", format);
    /// }
    /// ```
    pub fn get_by_extension(&self, ext: &str) -> Option<&str> {
        self.extension_map
            .get(ext.to_lowercase().as_str())
            .map(|s| s.as_str())
    }

    /// Get a writer by file path.
    ///
    /// This extracts the extension from the path and looks up the corresponding
    /// writer.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::writers::WriterRegistry;
    /// use std::path::Path;
    ///
    /// let registry = WriterRegistry::with_defaults();
    /// if let Some(writer) = registry.get_by_path(Path::new("document.html")) {
    ///     println!("Found writer for document.html");
    /// }
    /// ```
    pub fn get_by_path(&self, path: &std::path::Path) -> Option<&dyn Writer> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.get_by_extension(ext))
            .and_then(|name| self.get(name))
    }

    /// Check if a writer is registered for the given format name.
    pub fn contains(&self, name: &str) -> bool {
        self.writers.contains_key(name.to_lowercase().as_str())
    }

    /// Get a list of all registered format names.
    pub fn list(&self) -> Vec<&str> {
        self.writers.keys().map(|s| s.as_str()).collect()
    }

    /// Get a list of all registered file extensions.
    pub fn extensions(&self) -> Vec<&str> {
        self.extension_map.keys().map(|s| s.as_str()).collect()
    }

    /// Get the number of registered writers.
    pub fn len(&self) -> usize {
        self.writers.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.writers.is_empty()
    }

    /// Remove a writer from the registry.
    pub fn remove(&mut self, name: &str) -> Option<BoxedWriter> {
        let name = name.to_lowercase();

        // Remove from extension map
        self.extension_map.retain(|_, v| v != &name);

        // Remove writer
        self.writers.remove(&name)
    }

    /// Clear all writers from the registry.
    pub fn clear(&mut self) {
        self.writers.clear();
        self.extension_map.clear();
    }

    /// Register all built-in writers.
    fn register_builtin_writers(&mut self) {
        self.register(HtmlWriter::new());
        self.register(CommonMarkWriter::new());
        self.register(XmlWriter::new());
        self.register(LatexWriter::new());
        self.register(ManWriter::new());
    }
}

impl Clone for WriterRegistry {
    fn clone(&self) -> Self {
        // Note: This creates a new registry with the same writers.
        // Since writers are trait objects, we can't directly clone them.
        // In practice, you'd typically create a new registry with_defaults().
        Self::with_defaults()
    }
}

/// Built-in HTML writer.
#[derive(Debug, Clone, Copy)]
pub struct HtmlWriter;

impl HtmlWriter {
    /// Create a new HTML writer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for HtmlWriter {
    fn name(&self) -> &'static str {
        "html"
    }

    fn extensions(&self) -> &[&'static str] {
        &["html", "htm", "xhtml"]
    }

    fn write_text<'c>(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &WriterOptions<'c>,
    ) -> Result<String, ClmdError> {
        crate::writers::write_html(arena, root, options)
    }
}

/// Built-in CommonMark writer.
#[derive(Debug, Clone, Copy)]
pub struct CommonMarkWriter;

impl CommonMarkWriter {
    /// Create a new CommonMark writer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CommonMarkWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for CommonMarkWriter {
    fn name(&self) -> &'static str {
        "commonmark"
    }

    fn extensions(&self) -> &[&'static str] {
        &["md", "markdown", "commonmark", "cm"]
    }

    fn write_text<'c>(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &WriterOptions<'c>,
    ) -> Result<String, ClmdError> {
        crate::writers::write_commonmark(arena, root, options)
    }
}

/// Built-in XML writer.
#[derive(Debug, Clone, Copy)]
pub struct XmlWriter;

impl XmlWriter {
    /// Create a new XML writer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for XmlWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for XmlWriter {
    fn name(&self) -> &'static str {
        "xml"
    }

    fn extensions(&self) -> &[&'static str] {
        &["xml"]
    }

    fn write_text<'c>(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &WriterOptions<'c>,
    ) -> Result<String, ClmdError> {
        crate::writers::write_xml(arena, root, options)
    }
}

/// Built-in LaTeX writer.
#[derive(Debug, Clone, Copy)]
pub struct LatexWriter;

impl LatexWriter {
    /// Create a new LaTeX writer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LatexWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for LatexWriter {
    fn name(&self) -> &'static str {
        "latex"
    }

    fn extensions(&self) -> &[&'static str] {
        &["tex", "latex"]
    }

    fn write_text<'c>(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &WriterOptions<'c>,
    ) -> Result<String, ClmdError> {
        crate::writers::write_latex(arena, root, options)
    }
}

/// Built-in Man page writer.
#[derive(Debug, Clone, Copy)]
pub struct ManWriter;

impl ManWriter {
    /// Create a new Man page writer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ManWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for ManWriter {
    fn name(&self) -> &'static str {
        "man"
    }

    fn extensions(&self) -> &[&'static str] {
        &["man", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
    }

    fn write_text<'c>(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &WriterOptions<'c>,
    ) -> Result<String, ClmdError> {
        crate::writers::write_man(arena, root, options)
    }
}

/// Get the default writer registry.
///
/// This is a lazily initialized global registry containing all built-in writers.
///
/// # Example
///
/// ```ignore
/// use clmd::writers::default_registry;
///
/// let registry = default_registry();
/// if let Some(writer) = registry.get("html") {
///     println!("HTML writer is available");
/// }
/// ```ignore
pub fn default_registry() -> &'static WriterRegistry {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<WriterRegistry> = OnceLock::new();
    REGISTRY.get_or_init(WriterRegistry::with_defaults)
}

/// Get a writer by name from the default registry.
///
/// This is a convenience function that looks up a writer in the default registry.
///
/// # Example
///
/// ```ignore
/// use clmd::writers::get_writer;
///
/// if let Some(writer) = get_writer("html") {
///     println!("Found HTML writer");
/// }
/// ```ignore
pub fn get_writer(name: &str) -> Option<&'static dyn Writer> {
    default_registry().get(name)
}

/// Get a writer by file extension from the default registry.
///
/// This is a convenience function that looks up a writer by extension
/// in the default registry.
///
/// # Example
///
/// ```ignore
/// use clmd::writers::get_writer_by_extension;
///
/// if let Some(writer) = get_writer_by_extension("html") {
///     println!("Found writer for .html files");
/// }
/// ```ignore
pub fn get_writer_by_extension(ext: &str) -> Option<&'static dyn Writer> {
    default_registry()
        .get_by_extension(ext)
        .and_then(|name| default_registry().get(name))
}

/// Get a writer by file path from the default registry.
///
/// This is a convenience function that looks up a writer by file path
/// in the default registry.
///
/// # Example
///
/// ```ignore
/// use clmd::writers::get_writer_by_path;
/// use std::path::Path;
///
/// if let Some(writer) = get_writer_by_path(Path::new("document.html")) {
///     println!("Found writer for document.html");
/// }
/// ```ignore
pub fn get_writer_by_path(path: &std::path::Path) -> Option<&'static dyn Writer> {
    default_registry().get_by_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_registry_new() {
        let registry = WriterRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_with_defaults() {
        let registry = WriterRegistry::with_defaults();
        assert!(!registry.is_empty());
        assert!(registry.contains("html"));
        assert!(registry.contains("commonmark"));
        assert!(registry.contains("xml"));
        assert!(registry.contains("latex"));
        assert!(registry.contains("man"));
    }

    #[test]
    fn test_registry_get() {
        let registry = WriterRegistry::with_defaults();

        let writer = registry.get("html").unwrap();
        assert_eq!(writer.name(), "html");

        let writer = registry.get("HTML").unwrap(); // case insensitive
        assert_eq!(writer.name(), "html");

        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_registry_get_by_extension() {
        let registry = WriterRegistry::with_defaults();

        assert_eq!(registry.get_by_extension("html"), Some("html"));
        assert_eq!(registry.get_by_extension("HTML"), Some("html")); // case insensitive
        assert_eq!(registry.get_by_extension("md"), Some("commonmark"));
        assert_eq!(registry.get_by_extension("tex"), Some("latex"));
        assert_eq!(registry.get_by_extension("unknown"), None);
    }

    #[test]
    fn test_registry_get_by_path() {
        let registry = WriterRegistry::with_defaults();

        let writer = registry.get_by_path(Path::new("document.html")).unwrap();
        assert_eq!(writer.name(), "html");

        let writer = registry.get_by_path(Path::new("/path/to/file.md")).unwrap();
        assert_eq!(writer.name(), "commonmark");

        assert!(registry.get_by_path(Path::new("no_extension")).is_none());
    }

    #[test]
    fn test_registry_list() {
        let registry = WriterRegistry::with_defaults();
        let formats = registry.list();

        assert!(formats.contains(&"html"));
        assert!(formats.contains(&"commonmark"));
        assert!(formats.contains(&"xml"));
    }

    #[test]
    fn test_registry_extensions() {
        let registry = WriterRegistry::with_defaults();
        let extensions = registry.extensions();

        assert!(extensions.contains(&"html"));
        assert!(extensions.contains(&"htm"));
        assert!(extensions.contains(&"md"));
        assert!(extensions.contains(&"xml"));
    }

    #[test]
    fn test_html_writer() {
        let writer = HtmlWriter::new();
        assert_eq!(writer.name(), "html");
        assert!(writer.supports_extension("html"));
        assert!(writer.supports_extension("htm"));
        assert!(!writer.supports_extension("md"));
    }

    #[test]
    fn test_commonmark_writer() {
        let writer = CommonMarkWriter::new();
        assert_eq!(writer.name(), "commonmark");
        assert!(writer.supports_extension("md"));
        assert!(writer.supports_extension("markdown"));
        assert!(!writer.supports_extension("html"));
    }

    #[test]
    fn test_xml_writer() {
        let writer = XmlWriter::new();
        assert_eq!(writer.name(), "xml");
        assert!(writer.supports_extension("xml"));
        assert!(!writer.supports_extension("html"));
    }

    #[test]
    fn test_latex_writer() {
        let writer = LatexWriter::new();
        assert_eq!(writer.name(), "latex");
        assert!(writer.supports_extension("tex"));
        assert!(writer.supports_extension("latex"));
    }

    #[test]
    fn test_man_writer() {
        let writer = ManWriter::new();
        assert_eq!(writer.name(), "man");
        assert!(writer.supports_extension("man"));
        assert!(writer.supports_extension("1"));
    }

    #[test]
    fn test_default_registry() {
        let registry = default_registry();
        assert!(registry.contains("html"));
        assert!(registry.contains("commonmark"));
    }

    #[test]
    fn test_get_writer() {
        let writer = get_writer("html").unwrap();
        assert_eq!(writer.name(), "html");

        assert!(get_writer("unknown").is_none());
    }

    #[test]
    fn test_get_writer_by_extension() {
        let writer = get_writer_by_extension("html").unwrap();
        assert_eq!(writer.name(), "html");

        assert!(get_writer_by_extension("unknown").is_none());
    }

    #[test]
    fn test_get_writer_by_path() {
        let writer = get_writer_by_path(Path::new("test.html")).unwrap();
        assert_eq!(writer.name(), "html");

        assert!(get_writer_by_path(Path::new("no_extension")).is_none());
    }
}
