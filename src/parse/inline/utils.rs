//! Utility functions for inline parsing

/// Re-export reference normalization from scanners module
pub use crate::parse::util::scanners::normalize_reference;
/// Re-export character type utilities from scanners module
pub use crate::parse::util::scanners::{is_escapable, is_punctuation};
/// Re-export special character check functions from scanners module
pub use crate::parse::util::scanners::{is_special_byte, is_special_char};
/// Re-export URI normalization from text::uri module
pub use crate::text::uri::normalize_uri;

/// HTML5 named entities lookup table
/// This includes all 2125 HTML5 entities for full CommonMark compliance
pub fn get_html5_entity(name: &str) -> Option<&'static str> {
    use super::entities_table::lookup_entity;
    lookup_entity(name)
}

/// Result of scanning delimiters
pub struct DelimScanResult {
    pub num_delims: usize,
    pub can_open: bool,
    pub can_close: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_html5_entity() {
        assert_eq!(get_html5_entity("amp"), Some("&"));
        assert_eq!(get_html5_entity("lt"), Some("<"));
        assert_eq!(get_html5_entity("gt"), Some(">"));
        assert_eq!(get_html5_entity("quot"), Some("\""));
        assert_eq!(get_html5_entity("nonexistent"), None);
    }

    #[test]
    fn test_delim_scan_result() {
        let result = DelimScanResult {
            num_delims: 2,
            can_open: true,
            can_close: false,
        };
        assert_eq!(result.num_delims, 2);
        assert!(result.can_open);
        assert!(!result.can_close);
    }

    #[test]
    fn test_reexported_functions() {
        // Test that re-exported functions work correctly
        assert!(is_punctuation('!'));
        assert!(is_special_char('*', false));
        assert!(is_special_byte(b'*', false));
        assert!(is_escapable('!'));
        assert_eq!(normalize_reference("hello"), "HELLO");
        assert_eq!(normalize_uri("hello world"), "hello%20world");
    }
}
