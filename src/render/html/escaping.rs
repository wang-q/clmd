//! HTML escaping utilities for HTML renderer

use crate::text::html_utils::is_safe_url;

/// Escape URL for use in href attribute
///
/// This function performs two important security checks:
/// 1. Validates the URL scheme to prevent javascript: and other unsafe protocols
/// 2. Escapes special HTML characters to prevent XSS attacks
///
/// # Arguments
///
/// * `url` - The URL to escape
///
/// # Returns
///
/// The escaped URL string, or "#" if the URL is considered unsafe
pub fn escape_href(url: &str) -> String {
    // First check if the URL is safe (prevents javascript: and other unsafe protocols)
    // Check the original URL before any modification to prevent bypass attempts
    if !is_safe_url(url) {
        return "#".to_string();
    }

    // Empty URL is allowed by CommonMark spec (generates href="")
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return "".to_string();
    }

    // Escape special HTML characters for attribute context
    // This is more comprehensive than basic HTML escaping
    let mut result = String::with_capacity(url.len() * 2);
    for c in url.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '\'' => result.push_str("&#x27;"),
            '`' => result.push_str("&#x60;"), // Backtick can be used in IE attribute injection
            _ => result.push(c),
        }
    }
    result
}
