//! MIME type utilities for clmd.
//!
//! This module provides MIME type detection and management, inspired by
//! Pandoc's MIME module. It includes a comprehensive list of MIME types
//! and utilities for detecting MIME types from file extensions.
//!
//! # Example
//!
//! ```
//! use clmd::io::format::mime::{get_mime_type, get_mime_type_def, extension_from_mime_type};
//! use std::path::Path;
//!
//! // Get MIME type from file path
//! let mime = get_mime_type(Path::new("image.png"));
//! assert_eq!(mime, Some("image/png"));
//!
//! // Get MIME type with default fallback
//! let mime = get_mime_type_def(Path::new("unknown.xyz"));
//! assert_eq!(mime, "application/octet-stream");
//!
//! // Get extension from MIME type
//! let ext = extension_from_mime_type("image/jpeg");
//! assert_eq!(ext, Some("jpg"));
//! ```

use std::collections::HashMap;
use std::path::Path;

/// MIME type string.
pub type MimeType = &'static str;

/// A comprehensive list of MIME types mapped to file extensions.
///
/// This list includes common MIME types used in web and document processing.
const MIME_TYPES: &[(&str, &str)] = &[
    // Text formats
    ("txt", "text/plain"),
    ("html", "text/html"),
    ("htm", "text/html"),
    ("css", "text/css"),
    ("js", "text/javascript"),
    ("json", "application/json"),
    ("xml", "application/xml"),
    ("md", "text/markdown"),
    ("markdown", "text/markdown"),
    ("csv", "text/csv"),
    ("tsv", "text/tab-separated-values"),
    ("rtf", "application/rtf"),
    // Image formats
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("bmp", "image/bmp"),
    ("webp", "image/webp"),
    ("svg", "image/svg+xml"),
    ("ico", "image/x-icon"),
    ("tiff", "image/tiff"),
    ("tif", "image/tiff"),
    ("avif", "image/avif"),
    ("apng", "image/apng"),
    // Document formats
    ("pdf", "application/pdf"),
    ("doc", "application/msword"),
    (
        "docx",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ),
    ("odt", "application/vnd.oasis.opendocument.text"),
    ("xls", "application/vnd.ms-excel"),
    (
        "xlsx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    ),
    ("ppt", "application/vnd.ms-powerpoint"),
    (
        "pptx",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    ),
    // Archive formats
    ("zip", "application/zip"),
    ("gz", "application/gzip"),
    ("tar", "application/x-tar"),
    ("bz2", "application/x-bzip2"),
    ("7z", "application/x-7z-compressed"),
    ("rar", "application/vnd.rar"),
    // Audio formats
    ("mp3", "audio/mpeg"),
    ("wav", "audio/wav"),
    ("ogg", "audio/ogg"),
    ("oga", "audio/ogg"),
    ("flac", "audio/flac"),
    ("aac", "audio/aac"),
    ("m4a", "audio/mp4"),
    // Video formats
    ("mp4", "video/mp4"),
    ("webm", "video/webm"),
    ("ogv", "video/ogg"),
    ("avi", "video/x-msvideo"),
    ("mov", "video/quicktime"),
    ("mkv", "video/x-matroska"),
    // Font formats
    ("woff", "font/woff"),
    ("woff2", "font/woff2"),
    ("ttf", "font/ttf"),
    ("otf", "font/otf"),
    // Code formats
    ("rs", "text/rust"),
    ("py", "text/x-python"),
    ("java", "text/x-java"),
    ("c", "text/x-c"),
    ("cpp", "text/x-c++"),
    ("h", "text/x-c"),
    ("hpp", "text/x-c++"),
    ("go", "text/x-go"),
    ("rb", "text/x-ruby"),
    ("php", "text/x-php"),
    ("swift", "text/x-swift"),
    ("kt", "text/x-kotlin"),
    ("scala", "text/x-scala"),
    ("r", "text/x-r"),
    ("lua", "text/x-lua"),
    ("pl", "text/x-perl"),
    ("sh", "text/x-shellscript"),
    ("bash", "text/x-shellscript"),
    ("zsh", "text/x-shellscript"),
    ("fish", "text/x-shellscript"),
    ("ps1", "text/x-powershell"),
    // Data formats
    ("yaml", "text/yaml"),
    ("yml", "text/yaml"),
    ("toml", "text/toml"),
    ("ini", "text/plain"),
    ("sql", "text/x-sql"),
    // LaTeX formats
    ("tex", "text/x-tex"),
    ("latex", "text/x-latex"),
    ("bib", "text/x-bibtex"),
    // Other formats
    ("epub", "application/epub+zip"),
    ("mobi", "application/x-mobipocket-ebook"),
    ("wasm", "application/wasm"),
];

/// Lazy-initialized MIME type map.
fn mime_type_map() -> &'static HashMap<&'static str, &'static str> {
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    MAP.get_or_init(|| MIME_TYPES.iter().cloned().collect())
}

/// Lazy-initialized reverse MIME type map (MIME type to extension).
fn reverse_mime_type_map() -> &'static HashMap<&'static str, &'static str> {
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut map = HashMap::new();
        for (ext, mime) in MIME_TYPES {
            // Only insert if not already present (first one wins)
            map.entry(*mime).or_insert(*ext);
        }
        map
    })
}

/// Get the MIME type for a file path.
///
/// This function extracts the file extension from the path and looks up
/// the corresponding MIME type.
///
/// # Arguments
///
/// * `path` - The file path
///
/// # Returns
///
/// The MIME type as a string slice, or `None` if the extension is not recognized.
///
/// # Example
///
/// ```ignore
/// use clmd::formats::mime::get_mime_type;
/// use std::path::Path;
///
/// assert_eq!(get_mime_type(Path::new("image.png")), Some("image/png"));
/// assert_eq!(get_mime_type(Path::new("document.pdf")), Some("application/pdf"));
/// assert_eq!(get_mime_type(Path::new("no_extension")), None);
/// ```ignore
pub fn get_mime_type(path: &Path) -> Option<MimeType> {
    path.extension()
        .and_then(|e| e.to_str())
        .and_then(|ext| mime_type_map().get(ext.to_lowercase().as_str()).copied())
}

/// Get the MIME type for a file path with a default fallback.
///
/// If the file extension is not recognized, returns "application/octet-stream".
///
/// # Arguments
///
/// * `path` - The file path
///
/// # Returns
///
/// The MIME type as a string slice.
///
/// # Example
///
/// ```ignore
/// use clmd::formats::mime::get_mime_type_def;
/// use std::path::Path;
///
/// assert_eq!(get_mime_type_def(Path::new("image.png")), "image/png");
/// assert_eq!(get_mime_type_def(Path::new("unknown.xyz")), "application/octet-stream");
/// ```ignore
pub fn get_mime_type_def(path: &Path) -> MimeType {
    get_mime_type(path).unwrap_or("application/octet-stream")
}

/// Get the file extension for a MIME type.
///
/// # Arguments
///
/// * `mime_type` - The MIME type
///
/// # Returns
///
/// The file extension as a string slice, or `None` if the MIME type is not recognized.
///
/// # Example
///
/// ```ignore
/// use clmd::formats::mime::extension_from_mime_type;
///
/// assert_eq!(extension_from_mime_type("image/png"), Some("png"));
/// assert_eq!(extension_from_mime_type("text/html"), Some("html"));
/// assert_eq!(extension_from_mime_type("unknown/type"), None);
/// ```ignore
pub fn extension_from_mime_type(mime_type: &str) -> Option<&'static str> {
    // Normalize the MIME type (remove parameters)
    let normalized = mime_type.split(';').next().unwrap_or(mime_type).trim();
    reverse_mime_type_map().get(normalized).copied()
}

/// Get the general media category for a MIME type.
///
/// Returns the part before the slash in the MIME type (e.g., "image", "text", "application").
///
/// # Arguments
///
/// * `mime_type` - The MIME type
///
/// # Returns
///
/// The media category as a string slice, or `None` if the MIME type is invalid.
///
/// # Example
///
/// ```ignore
/// use clmd::formats::mime::media_category;
///
/// assert_eq!(media_category("image/png"), Some("image"));
/// assert_eq!(media_category("text/html"), Some("text"));
/// assert_eq!(media_category("application/pdf"), Some("application"));
/// ```ignore
pub fn media_category(mime_type: &str) -> Option<&str> {
    mime_type.split('/').next()
}

/// Get the media category for a file path.
///
/// This is a convenience function that combines `get_mime_type` and `media_category`.
///
/// # Arguments
///
/// * `path` - The file path
///
/// # Returns
///
/// The media category as a string slice, or `None` if the file extension is not recognized.
///
/// # Example
///
/// ```ignore
/// use clmd::formats::mime::media_category_from_path;
/// use std::path::Path;
///
/// assert_eq!(media_category_from_path(Path::new("image.png")), Some("image"));
/// assert_eq!(media_category_from_path(Path::new("document.txt")), Some("text"));
/// ```ignore
pub fn media_category_from_path(path: &Path) -> Option<&str> {
    get_mime_type(path).and_then(|mime| media_category(mime))
}

/// Check if a MIME type is an image.
///
/// # Arguments
///
/// * `mime_type` - The MIME type
///
/// # Returns
///
/// `true` if the MIME type is an image type.
///
/// # Example
///
/// ```ignore
/// use clmd::formats::mime::is_image;
///
/// assert!(is_image("image/png"));
/// assert!(is_image("image/jpeg"));
/// assert!(!is_image("text/html"));
/// ```ignore
pub fn is_image(mime_type: &str) -> bool {
    media_category(mime_type) == Some("image")
}

/// Check if a MIME type is text.
///
/// # Arguments
///
/// * `mime_type` - The MIME type
///
/// # Returns
///
/// `true` if the MIME type is a text type.
///
/// # Example
///
/// ```ignore
/// use clmd::formats::mime::is_text;
///
/// assert!(is_text("text/html"));
/// assert!(is_text("text/plain"));
/// assert!(!is_text("image/png"));
/// ```ignore
pub fn is_text(mime_type: &str) -> bool {
    media_category(mime_type) == Some("text")
}

/// Check if a MIME type is a font.
///
/// # Arguments
///
/// * `mime_type` - The MIME type
///
/// # Returns
///
/// `true` if the MIME type is a font type.
///
/// # Example
///
/// ```ignore
/// use clmd::formats::mime::is_font;
///
/// assert!(is_font("font/woff2"));
/// assert!(is_font("font/ttf"));
/// assert!(!is_font("text/css"));
/// ```ignore
pub fn is_font(mime_type: &str) -> bool {
    media_category(mime_type) == Some("font")
}

/// Get the charset from a MIME type if specified.
///
/// # Arguments
///
/// * `mime_type` - The MIME type
///
/// # Returns
///
/// The charset as a string slice, or `None` if not specified.
///
/// # Example
///
/// ```ignore
/// use clmd::formats::mime::get_charset;
///
/// assert_eq!(get_charset("text/html; charset=utf-8"), Some("utf-8"));
/// assert_eq!(get_charset("text/plain"), None);
/// ```ignore
pub fn get_charset(mime_type: &str) -> Option<&str> {
    mime_type
        .split(';')
        .skip(1)
        .find_map(|part| {
            let part = part.trim();
            part.strip_prefix("charset=")
                .or_else(|| part.strip_prefix("charset = "))
        })
        .map(|s| s.trim())
}

/// Normalize a MIME type by removing extra whitespace and parameters.
///
/// # Arguments
///
/// * `mime_type` - The MIME type
///
/// # Returns
///
/// The normalized MIME type.
///
/// # Example
///
/// ```ignore
/// use clmd::formats::mime::normalize_mime_type;
///
/// assert_eq!(normalize_mime_type("  text/html  "), "text/html");
/// assert_eq!(normalize_mime_type("text/html; charset=utf-8"), "text/html");
/// ```ignore
pub fn normalize_mime_type(mime_type: &str) -> &str {
    mime_type.split(';').next().unwrap_or(mime_type).trim()
}

/// Get all registered MIME types.
///
/// # Returns
///
/// A slice of all registered MIME type mappings.
pub fn all_mime_types() -> &'static [(&'static str, &'static str)] {
    MIME_TYPES
}

/// Get all registered file extensions.
///
/// # Returns
///
/// A vector of all registered file extensions.
pub fn all_extensions() -> Vec<&'static str> {
    mime_type_map().keys().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type(Path::new("file.png")), Some("image/png"));
        assert_eq!(get_mime_type(Path::new("file.jpg")), Some("image/jpeg"));
        assert_eq!(get_mime_type(Path::new("file.html")), Some("text/html"));
        assert_eq!(
            get_mime_type(Path::new("file.pdf")),
            Some("application/pdf")
        );
        assert_eq!(get_mime_type(Path::new("file.unknown")), None);
        assert_eq!(get_mime_type(Path::new("no_extension")), None);
    }

    #[test]
    fn test_get_mime_type_case_insensitive() {
        assert_eq!(get_mime_type(Path::new("file.PNG")), Some("image/png"));
        assert_eq!(get_mime_type(Path::new("file.HTML")), Some("text/html"));
    }

    #[test]
    fn test_get_mime_type_def() {
        assert_eq!(get_mime_type_def(Path::new("file.png")), "image/png");
        assert_eq!(
            get_mime_type_def(Path::new("file.unknown")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_extension_from_mime_type() {
        assert_eq!(extension_from_mime_type("image/png"), Some("png"));
        assert_eq!(extension_from_mime_type("text/html"), Some("html"));
        assert_eq!(extension_from_mime_type("application/pdf"), Some("pdf"));
        assert_eq!(extension_from_mime_type("unknown/type"), None);
    }

    #[test]
    fn test_extension_from_mime_type_with_params() {
        assert_eq!(
            extension_from_mime_type("text/html; charset=utf-8"),
            Some("html")
        );
    }

    #[test]
    fn test_media_category() {
        assert_eq!(media_category("image/png"), Some("image"));
        assert_eq!(media_category("text/html"), Some("text"));
        assert_eq!(media_category("application/pdf"), Some("application"));
        assert_eq!(media_category("font/woff2"), Some("font"));
        assert_eq!(media_category("invalid"), Some("invalid"));
    }

    #[test]
    fn test_media_category_from_path() {
        assert_eq!(
            media_category_from_path(Path::new("image.png")),
            Some("image")
        );
        assert_eq!(media_category_from_path(Path::new("doc.txt")), Some("text"));
        assert_eq!(media_category_from_path(Path::new("unknown.xyz")), None);
    }

    #[test]
    fn test_is_image() {
        assert!(is_image("image/png"));
        assert!(is_image("image/jpeg"));
        assert!(!is_image("text/html"));
        assert!(!is_image("application/pdf"));
    }

    #[test]
    fn test_is_text() {
        assert!(is_text("text/html"));
        assert!(is_text("text/plain"));
        assert!(!is_text("image/png"));
        assert!(!is_text("application/pdf"));
    }

    #[test]
    fn test_is_font() {
        assert!(is_font("font/woff2"));
        assert!(is_font("font/ttf"));
        assert!(!is_font("text/css"));
        assert!(!is_font("image/png"));
    }

    #[test]
    fn test_get_charset() {
        assert_eq!(get_charset("text/html; charset=utf-8"), Some("utf-8"));
        assert_eq!(
            get_charset("text/plain; charset=ISO-8859-1"),
            Some("ISO-8859-1")
        );
        assert_eq!(get_charset("text/html"), None);
    }

    #[test]
    fn test_normalize_mime_type() {
        assert_eq!(normalize_mime_type("  text/html  "), "text/html");
        assert_eq!(normalize_mime_type("text/html; charset=utf-8"), "text/html");
        assert_eq!(normalize_mime_type("text/html;charset=utf-8"), "text/html");
    }

    #[test]
    fn test_all_mime_types() {
        let types = all_mime_types();
        assert!(!types.is_empty());
        assert!(types.iter().any(|(ext, _)| *ext == "png"));
        assert!(types.iter().any(|(ext, _)| *ext == "html"));
    }

    #[test]
    fn test_all_extensions() {
        let exts = all_extensions();
        assert!(!exts.is_empty());
        assert!(exts.contains(&"png"));
        assert!(exts.contains(&"html"));
    }
}
