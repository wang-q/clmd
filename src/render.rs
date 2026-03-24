pub mod commonmark;
pub mod html;
pub mod latex;
pub mod man;
pub mod xml;

use crate::node::Node;
use std::cell::RefCell;
use std::rc::Rc;

/// Base renderer trait
pub trait Renderer {
    fn render(&mut self, root: &Rc<RefCell<Node>>, options: u32) -> String;
}

/// Escape HTML special characters
/// Optimized with byte-level scanning and pre-allocated capacity
pub fn escape_html(text: &str) -> String {
    // Fast path: check if any escaping is needed
    let bytes = text.as_bytes();
    let mut needs_escape = false;
    for &b in bytes {
        if matches!(b, b'&' | b'<' | b'>' | b'"') {
            needs_escape = true;
            break;
        }
    }

    if !needs_escape {
        return text.to_string();
    }

    // Slow path: escape needed - pre-allocate with extra capacity for entities
    let mut result = String::with_capacity(text.len() * 2);
    for &b in bytes {
        match b {
            b'&' => result.push_str("&amp;"),
            b'<' => result.push_str("&lt;"),
            b'>' => result.push_str("&gt;"),
            b'"' => result.push_str("&quot;"),
            _ => result.push(b as char),
        }
    }
    result
}

/// Check if a URL is safe
/// Based on commonmark.js reUnsafeProtocol and reSafeDataProtocol
pub fn is_safe_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();

    // Check for unsafe protocols
    let is_unsafe = url_lower.starts_with("javascript:")
        || url_lower.starts_with("vbscript:")
        || url_lower.starts_with("file:")
        || (url_lower.starts_with("data:") && !is_safe_data_url(&url_lower));

    !is_unsafe
}

/// Check if a data URL is safe (only allows image types)
fn is_safe_data_url(url: &str) -> bool {
    // Allow data:image/* URLs
    url.starts_with("data:image/png")
        || url.starts_with("data:image/gif")
        || url.starts_with("data:image/jpeg")
        || url.starts_with("data:image/webp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\"test\""), "&quot;test&quot;");
        assert_eq!(escape_html("'test'"), "'test'"); // Single quote is not escaped
        assert_eq!(escape_html("hello"), "hello"); // No special chars
    }

    #[test]
    fn test_is_safe_url_http() {
        assert!(is_safe_url("https://example.com"));
        assert!(is_safe_url("http://example.com"));
    }

    #[test]
    fn test_is_safe_url_javascript() {
        assert!(!is_safe_url("javascript:alert('xss')"));
        assert!(!is_safe_url("JAVASCRIPT:alert('xss')")); // Case insensitive
    }

    #[test]
    fn test_is_safe_url_vbscript() {
        assert!(!is_safe_url("vbscript:msgbox('xss')"));
    }

    #[test]
    fn test_is_safe_url_file() {
        assert!(!is_safe_url("file:///etc/passwd"));
    }

    #[test]
    fn test_is_safe_url_data_image() {
        assert!(is_safe_url("data:image/png;base64,abc123"));
        assert!(is_safe_url("data:image/gif;base64,abc123"));
        assert!(is_safe_url("data:image/jpeg;base64,abc123"));
        assert!(is_safe_url("data:image/webp;base64,abc123"));
    }

    #[test]
    fn test_is_safe_url_data_unsafe() {
        assert!(!is_safe_url("data:text/html,<script>alert('xss')</script>"));
        assert!(!is_safe_url("data:application/javascript,alert(1)"));
    }
}
