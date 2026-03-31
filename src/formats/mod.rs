//! Format detection and management for clmd.
//!
//! This module provides automatic format detection based on file extensions
//! and content analysis, inspired by Pandoc's format system.
//!
//! # Example
//!
//! ```
//! use clmd::formats::FormatDetector;
//!
//! let detector = FormatDetector::new();
//!
//! // Detect from file path
//! let format = detector.from_path("document.md").unwrap();
//! assert_eq!(format, "markdown");
//!
//! // Detect from content
//! let format = detector.from_content("<?xml version=\"1.0\"?>").unwrap();
//! assert_eq!(format, "xml");
//! ```

use std::collections::HashMap;
use std::path::Path;

/// Format detector for automatic format detection.
#[derive(Debug, Clone)]
pub struct FormatDetector {
    /// Mapping of file extensions to format names.
    extension_map: HashMap<String, String>,
    /// Content signatures for format detection.
    content_signatures: Vec<(String, Vec<String>)>,
}

impl FormatDetector {
    /// Create a new format detector with default mappings.
    pub fn new() -> Self {
        let mut detector = Self {
            extension_map: HashMap::new(),
            content_signatures: Vec::new(),
        };
        detector.register_defaults();
        detector
    }

    /// Register default format mappings.
    fn register_defaults(&mut self) {
        // Markdown variants
        self.register_extension("md", "markdown");
        self.register_extension("markdown", "markdown");
        self.register_extension("mkd", "markdown");
        self.register_extension("mkdn", "markdown");
        self.register_extension("mdown", "markdown");
        self.register_extension("mdwn", "markdown");

        // HTML variants
        self.register_extension("html", "html");
        self.register_extension("htm", "html");
        self.register_extension("xhtml", "html");

        // XML
        self.register_extension("xml", "xml");

        // LaTeX
        self.register_extension("tex", "latex");
        self.register_extension("latex", "latex");
        self.register_extension("ltx", "latex");

        // Typst
        self.register_extension("typ", "typst");

        // JSON
        self.register_extension("json", "json");

        // Text
        self.register_extension("txt", "plain");
        self.register_extension("text", "plain");

        // Content signatures (order matters - more specific signatures first)
        self.register_content_signature(
            "html",
            vec!["<!DOCTYPE html".to_string(), "<html".to_string()],
        );
        self.register_content_signature("xml", vec!["<?xml".to_string()]);
        self.register_content_signature("json", vec!["{".to_string(), "[".to_string()]);
    }

    /// Register a file extension mapping.
    pub fn register_extension(
        &mut self,
        ext: impl Into<String>,
        format: impl Into<String>,
    ) {
        self.extension_map
            .insert(ext.into().to_lowercase(), format.into());
    }

    /// Register a content signature for format detection.
    pub fn register_content_signature(
        &mut self,
        format: impl Into<String>,
        signatures: Vec<String>,
    ) {
        self.content_signatures.push((format.into(), signatures));
    }

    /// Detect format from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to analyze
    ///
    /// # Returns
    ///
    /// Some(format) if the format can be detected, None otherwise.
    pub fn from_path<P: AsRef<Path>>(&self, path: P) -> Option<String> {
        let path = path.as_ref();

        // Try to get extension
        let ext = path.extension()?.to_str()?;
        self.extension_map.get(&ext.to_lowercase()).cloned()
    }

    /// Detect format from file content.
    ///
    /// # Arguments
    ///
    /// * `content` - The file content to analyze
    ///
    /// # Returns
    ///
    /// Some(format) if the format can be detected, None otherwise.
    pub fn from_content(&self, content: &str) -> Option<String> {
        let trimmed = content.trim_start();

        for (format, signatures) in &self.content_signatures {
            for signature in signatures {
                if trimmed.starts_with(signature) {
                    return Some(format.clone());
                }
            }
        }

        None
    }

    /// Detect format from both path and content.
    ///
    /// First tries to detect from the path, then falls back to content detection.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path
    /// * `content` - The file content
    ///
    /// # Returns
    ///
    /// Some(format) if the format can be detected, None otherwise.
    pub fn detect<P: AsRef<Path>>(&self, path: P, content: &str) -> Option<String> {
        self.from_path(path).or_else(|| self.from_content(content))
    }

    /// Get all registered formats.
    pub fn formats(&self) -> Vec<&String> {
        let mut formats: Vec<&String> = self.extension_map.values().collect();
        formats.sort();
        formats.dedup();
        formats
    }

    /// Get extensions for a format.
    pub fn extensions_for(&self, format: &str) -> Vec<&String> {
        self.extension_map
            .iter()
            .filter(|(_, f)| f.as_str() == format)
            .map(|(ext, _)| ext)
            .collect()
    }
}

impl Default for FormatDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to detect format from a path.
pub fn detect_from_path<P: AsRef<Path>>(path: P) -> Option<String> {
    FormatDetector::new().from_path(path)
}

/// Convenience function to detect format from content.
pub fn detect_from_content(content: &str) -> Option<String> {
    FormatDetector::new().from_content(content)
}

/// Format information for display and documentation.
#[derive(Debug, Clone)]
pub struct FormatInfo {
    /// Format name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Common file extensions.
    pub extensions: Vec<String>,
    /// Whether this format can be read.
    pub readable: bool,
    /// Whether this format can be written.
    pub writable: bool,
    /// Default extensions for this format.
    pub default_extensions: Vec<String>,
}

/// Registry of format information.
#[derive(Debug, Clone)]
pub struct FormatInfoRegistry {
    formats: HashMap<String, FormatInfo>,
}

impl FormatInfoRegistry {
    /// Create a new format info registry with default information.
    pub fn new() -> Self {
        let mut registry = Self {
            formats: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    /// Register default format information.
    fn register_defaults(&mut self) {
        self.register(FormatInfo {
            name: "markdown".to_string(),
            description: "Markdown format".to_string(),
            extensions: vec![
                "md".to_string(),
                "markdown".to_string(),
                "mkd".to_string(),
            ],
            readable: true,
            writable: true,
            default_extensions: vec!["table".to_string(), "strikethrough".to_string()],
        });

        self.register(FormatInfo {
            name: "html".to_string(),
            description: "HTML format".to_string(),
            extensions: vec!["html".to_string(), "htm".to_string(), "xhtml".to_string()],
            readable: true,
            writable: true,
            default_extensions: vec![],
        });

        self.register(FormatInfo {
            name: "xml".to_string(),
            description: "XML format".to_string(),
            extensions: vec!["xml".to_string()],
            readable: false,
            writable: true,
            default_extensions: vec![],
        });

        self.register(FormatInfo {
            name: "latex".to_string(),
            description: "LaTeX format".to_string(),
            extensions: vec!["tex".to_string(), "latex".to_string(), "ltx".to_string()],
            readable: false,
            writable: true,
            default_extensions: vec![],
        });

        self.register(FormatInfo {
            name: "typst".to_string(),
            description: "Typst format".to_string(),
            extensions: vec!["typ".to_string()],
            readable: false,
            writable: true,
            default_extensions: vec![],
        });

        self.register(FormatInfo {
            name: "plain".to_string(),
            description: "Plain text".to_string(),
            extensions: vec!["txt".to_string(), "text".to_string()],
            readable: true,
            writable: true,
            default_extensions: vec![],
        });

        self.register(FormatInfo {
            name: "json".to_string(),
            description: "JSON format".to_string(),
            extensions: vec!["json".to_string()],
            readable: false,
            writable: true,
            default_extensions: vec![],
        });
    }

    /// Register format information.
    pub fn register(&mut self, info: FormatInfo) {
        self.formats.insert(info.name.clone(), info);
    }

    /// Get format information.
    pub fn get(&self, name: &str) -> Option<&FormatInfo> {
        self.formats.get(name)
    }

    /// Get all readable formats.
    pub fn readable_formats(&self) -> Vec<&FormatInfo> {
        self.formats.values().filter(|f| f.readable).collect()
    }

    /// Get all writable formats.
    pub fn writable_formats(&self) -> Vec<&FormatInfo> {
        self.formats.values().filter(|f| f.writable).collect()
    }

    /// List all formats.
    pub fn list_formats(&self) -> Vec<&FormatInfo> {
        self.formats.values().collect()
    }
}

impl Default for FormatInfoRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_from_path_markdown() {
        let detector = FormatDetector::new();
        assert_eq!(detector.from_path("test.md"), Some("markdown".to_string()));
        assert_eq!(
            detector.from_path("test.markdown"),
            Some("markdown".to_string())
        );
        assert_eq!(detector.from_path("test.mkd"), Some("markdown".to_string()));
    }

    #[test]
    fn test_detect_from_path_html() {
        let detector = FormatDetector::new();
        assert_eq!(detector.from_path("test.html"), Some("html".to_string()));
        assert_eq!(detector.from_path("test.htm"), Some("html".to_string()));
    }

    #[test]
    fn test_detect_from_path_unknown() {
        let detector = FormatDetector::new();
        assert_eq!(detector.from_path("test.unknown"), None);
    }

    #[test]
    fn test_detect_from_content_xml() {
        let detector = FormatDetector::new();
        assert_eq!(
            detector.from_content("<?xml version=\"1.0\"?>"),
            Some("xml".to_string())
        );
    }

    #[test]
    fn test_detect_from_content_html() {
        let detector = FormatDetector::new();
        assert_eq!(
            detector.from_content("<!DOCTYPE html>"),
            Some("html".to_string())
        );
        assert_eq!(detector.from_content("<html>"), Some("html".to_string()));
    }

    #[test]
    fn test_detect_from_content_unknown() {
        let detector = FormatDetector::new();
        assert_eq!(detector.from_content("Hello world"), None);
    }

    #[test]
    fn test_detect_combined() {
        let detector = FormatDetector::new();
        // Path takes precedence
        assert_eq!(
            detector.detect("test.md", "<?xml version=\"1.0\"?>"),
            Some("markdown".to_string())
        );
        // Falls back to content
        assert_eq!(
            detector.detect("test.unknown", "<?xml version=\"1.0\"?>"),
            Some("xml".to_string())
        );
    }

    #[test]
    fn test_convenience_functions() {
        assert_eq!(detect_from_path("test.md"), Some("markdown".to_string()));
        assert_eq!(
            detect_from_content("<?xml version=\"1.0\"?>"),
            Some("xml".to_string())
        );
    }

    #[test]
    fn test_format_info_registry() {
        let registry = FormatInfoRegistry::new();

        let markdown = registry.get("markdown").unwrap();
        assert!(markdown.readable);
        assert!(markdown.writable);
        assert!(markdown.extensions.contains(&"md".to_string()));

        let readable = registry.readable_formats();
        assert!(!readable.is_empty());

        let writable = registry.writable_formats();
        assert!(!writable.is_empty());
    }

    #[test]
    fn test_custom_extension() {
        let mut detector = FormatDetector::new();
        detector.register_extension("custom", "myformat");
        assert_eq!(
            detector.from_path("test.custom"),
            Some("myformat".to_string())
        );
    }

    #[test]
    fn test_formats_list() {
        let detector = FormatDetector::new();
        let formats = detector.formats();
        assert!(formats.contains(&&"markdown".to_string()));
        assert!(formats.contains(&&"html".to_string()));
    }
}
