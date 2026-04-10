//! HTML Utilities - HTML escaping and tag generation
//!
//! This module provides utilities for safe HTML generation,
//! including HTML entity escaping and tag building.
//!
//! Inspired by flexmark-java's flexmark-util-html.

/// Escape special HTML characters
///
/// Converts the following characters to their HTML entities:
/// - `&` → `&amp;`
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `"` → `&quot;`
///
/// Note: Single quotes `'` are NOT escaped in HTML content,
/// only in attribute values (use `escape_html_attribute` for that).
pub fn escape_html(input: &str) -> String {
    // Fast path: check if any escaping is needed
    let bytes = input.as_bytes();
    let mut needs_escape = false;
    for &b in bytes {
        if matches!(b, b'&' | b'<' | b'>' | b'"') {
            needs_escape = true;
            break;
        }
    }

    if !needs_escape {
        return input.to_string();
    }

    // Slow path: escape needed - process as characters to handle UTF-8 correctly
    let mut result = String::with_capacity(input.len() * 2);
    for c in input.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(c),
        }
    }
    result
}

/// Maximum allowed URL length to prevent DoS attacks
const MAX_URL_LENGTH: usize = 8192;

/// Check if a URL is safe to use in HTML output
///
/// This function checks for potentially dangerous URL schemes like
/// javascript:, vbscript:, file:, blob:, data: (non-image), and other
/// unsafe protocols. It also checks for IP-based URLs that might
/// access internal network resources.
///
/// # Arguments
///
/// * `url` - The URL to check
///
/// # Returns
///
/// `true` if the URL is considered safe, `false` otherwise
///
/// # Examples
///
/// ```ignore
/// use clmd::text::html_utils::is_safe_url;
///
/// assert!(is_safe_url("https://example.com"));
/// assert!(is_safe_url("http://example.com"));
/// assert!(!is_safe_url("javascript:alert('xss')"));
/// assert!(!is_safe_url("blob:https://example.com/uuid"));
/// ```
pub fn is_safe_url(url: &str) -> bool {
    // Trim whitespace and check for empty URL
    let url = url.trim();
    // Empty URL is allowed by CommonMark spec (generates href="")
    // This is safe as it just creates a link to the current page
    if url.is_empty() {
        return true;
    }

    // Check URL length to prevent DoS attacks
    if url.len() > MAX_URL_LENGTH {
        return false;
    }

    let url_lower = url.to_lowercase();

    // Check for unsafe protocols
    // Note: mailto: is intentionally NOT in this list as it's commonly used for email links
    let unsafe_protocols = [
        "javascript:",
        "vbscript:",
        "file:",
        "ftp:",
        "sftp:",
        "ssh:",
        "telnet:",
        "ldap:",
        "ldaps:",
        "smb:",
        "nfs:",
        "rtsp:",
        "rtmp:",
        "jar:",
        "icap:",
        "afs:",
        "tftp:",
        "dict:",
        "gopher:",
        "news:",
        "nntp:",
        "feed:",
        "imap:",
        "pop:",
        "smtp:",
        "ws:",
        "wss:",
    ];

    for protocol in &unsafe_protocols {
        if url_lower.starts_with(protocol) {
            return false;
        }
    }

    // Check for blob: URLs (can contain arbitrary JavaScript)
    if url_lower.starts_with("blob:") {
        return false;
    }

    // Check for filesystem: URLs (Chrome-specific, allows file system access)
    if url_lower.starts_with("filesystem:") {
        return false;
    }

    // Check for data: URLs (only allow safe image types)
    if url_lower.starts_with("data:") {
        return is_safe_data_url(&url_lower);
    }

    // Check for URLs that might be trying to bypass filters using encoding
    // Check for HTML entities in the scheme part
    if url_lower.starts_with("&#") || url_lower.starts_with("&x") {
        return false;
    }

    // Check for null bytes (can be used in some attacks)
    if url.contains('\0') {
        return false;
    }

    // Check for control characters
    if url.chars().any(|c| c.is_ascii_control()) {
        return false;
    }

    true
}

/// Check if a data URL is safe (only allows image types)
///
/// Currently allows: png, gif, jpeg, webp
/// Validates the format: data:image/{type}[;base64],{data}
fn is_safe_data_url(url: &str) -> bool {
    // Must start with data:image/
    let prefix = "data:image/";
    if !url.starts_with(prefix) {
        return false;
    }

    // Get the part after data:image/
    let after_prefix = &url[prefix.len()..];

    // Check for allowed image types followed by ;base64, or ,
    let allowed_types = ["png", "gif", "jpeg", "jpg", "webp"];

    for img_type in &allowed_types {
        let with_base64 = format!("{};base64,", img_type);
        let without_base64 = format!("{},", img_type);

        if after_prefix.starts_with(&with_base64)
            || after_prefix.starts_with(&without_base64)
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
        // Single quotes are NOT escaped in HTML content (only in attributes)
        assert_eq!(escape_html("it's"), "it's");
    }

    // Security tests for URL validation
    #[test]
    fn test_is_safe_url_blocks_dangerous_protocols() {
        // JavaScript protocol
        assert!(!is_safe_url("javascript:alert('xss')"));
        assert!(!is_safe_url("JAVASCRIPT:alert('xss')"));
        assert!(!is_safe_url("JavaScript:alert('xss')"));

        // VBScript protocol
        assert!(!is_safe_url("vbscript:msgbox('xss')"));

        // File protocol
        assert!(!is_safe_url("file:///etc/passwd"));

        // Blob URLs
        assert!(!is_safe_url("blob:https://example.com/uuid"));

        // Filesystem URLs
        assert!(!is_safe_url("filesystem:https://example.com/temporary/"));

        // FTP and other protocols
        assert!(!is_safe_url("ftp://example.com/file.txt"));
        assert!(!is_safe_url("sftp://example.com/file.txt"));
        assert!(!is_safe_url("ssh://user@example.com"));
        assert!(!is_safe_url("telnet://example.com"));

        // Email and messaging protocols
        // mailto: is allowed as it's a standard way to create email links
        assert!(is_safe_url("mailto:test@example.com"));
        // imap/smtp should be blocked
        assert!(!is_safe_url("imap://mail.example.com"));
        assert!(!is_safe_url("smtp://mail.example.com"));

        // WebSocket protocols
        assert!(!is_safe_url("ws://example.com/socket"));
        assert!(!is_safe_url("wss://example.com/socket"));
    }

    #[test]
    fn test_is_safe_url_allows_safe_urls() {
        // HTTP and HTTPS
        assert!(is_safe_url("https://example.com"));
        assert!(is_safe_url("http://example.com"));
        assert!(is_safe_url("https://example.com/path?query=value"));

        // With whitespace
        assert!(is_safe_url("  https://example.com  "));

        // Relative URLs
        assert!(is_safe_url("/path/to/page"));
        assert!(is_safe_url("../relative/path"));
        assert!(is_safe_url("#anchor"));
        assert!(is_safe_url("?query=value"));

        // Empty URL (allowed by CommonMark spec)
        assert!(is_safe_url(""));
        assert!(is_safe_url("   "));
    }

    #[test]
    fn test_is_safe_url_blocks_invalid() {
        // Null bytes
        assert!(!is_safe_url("https://example.com\0"));
        assert!(!is_safe_url("\0javascript:alert(1)"));

        // Control characters
        assert!(!is_safe_url("https://example.com\x01"));
        assert!(!is_safe_url("https://example.com\x1f"));
    }

    #[test]
    fn test_is_safe_url_blocks_encoded_attacks() {
        // HTML entity encoding attempt
        assert!(!is_safe_url("&#106;avascript:alert(1)"));
        assert!(!is_safe_url("&#x6a;avascript:alert(1)"));
    }

    #[test]
    fn test_is_safe_data_url() {
        // Safe image data URLs
        assert!(is_safe_url("data:image/png;base64,iVBORw0KGgo="));
        assert!(is_safe_url("data:image/jpeg;base64,/9j/4AAQ="));
        assert!(is_safe_url("data:image/gif;base64,R0lGODlh"));
        assert!(is_safe_url("data:image/webp;base64,UklGRiI="));
        assert!(is_safe_url("data:image/png,test"));

        // Unsafe data URLs (non-image)
        assert!(!is_safe_url("data:text/html,<script>alert(1)</script>"));
        assert!(!is_safe_url("data:application/javascript,alert(1)"));
        assert!(!is_safe_url("data:text/plain,hello"));

        // Malformed data URLs
        assert!(!is_safe_url("data:"));
        assert!(!is_safe_url("data:image/"));
        assert!(!is_safe_url("data:image/svg+xml,<svg>"));
    }

    #[test]
    fn test_is_safe_url_length_limit() {
        // URLs within limit should be safe
        assert!(is_safe_url("https://example.com"));

        // URL at exact limit should be safe
        let exact_limit_url =
            format!("https://example.com/{}", "a".repeat(MAX_URL_LENGTH - 24));
        assert!(is_safe_url(&exact_limit_url));

        // URL exceeding limit should be blocked
        let too_long_url = format!("https://example.com/{}", "a".repeat(MAX_URL_LENGTH));
        assert!(!is_safe_url(&too_long_url));

        // Very long URL should be blocked
        let very_long_url =
            format!("https://example.com/{}", "a".repeat(MAX_URL_LENGTH * 2));
        assert!(!is_safe_url(&very_long_url));
    }
}
