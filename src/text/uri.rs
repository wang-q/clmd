//! URI handling utilities for clmd.
//!
//! This module provides URI parsing, encoding, and manipulation utilities,
//! inspired by Pandoc's URI module.
//!
//! # Example
//!
//! ```ignore
//! use clmd::text::uri::{url_encode, escape_uri, is_uri};
//!
//! // URL encode a string
//! let encoded = url_encode("hello world");
//! assert_eq!(encoded, "hello%20world");
//!
//! // Check if a string is a URI
//! assert!(is_uri("https://example.com"));
//! assert!(!is_uri("not a uri"));
//! ```

use std::borrow::Cow;

/// URL-encode a string.
///
/// This function encodes special characters in a string for use in a URL.
/// Alphanumeric characters and `-`, `_`, `.`, and `~` are left as-is.
/// All other characters are encoded as `%XX` where `XX` is the hexadecimal
/// representation of the byte.
///
/// # Arguments
///
/// * `s` - The string to encode
///
/// # Returns
///
/// The URL-encoded string
///
/// # Example
///
/// ```ignore
/// use clmd::text::uri::url_encode;
///
/// assert_eq!(url_encode("hello world"), "hello%20world");
/// assert_eq!(url_encode("foo/bar"), "foo%2Fbar");
/// assert_eq!(url_encode("100%"), "100%25");
/// ```ignore
pub fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", byte));
            }
        }
    }
    result
}

/// URL-decode a string.
///
/// This function decodes `%XX` sequences in a URL-encoded string.
/// Invalid sequences are left as-is.
///
/// # Arguments
///
/// * `s` - The string to decode
///
/// # Returns
///
/// The URL-decoded string
///
/// # Example
///
/// ```ignore
/// use clmd::text::uri::url_decode;
///
/// assert_eq!(url_decode("hello%20world"), "hello world");
/// assert_eq!(url_decode("foo%2Fbar"), "foo/bar");
/// assert_eq!(url_decode("100%25"), "100%");
/// ```ignore
pub fn url_decode(s: &str) -> Cow<'_, str> {
    // Check if decoding is needed
    if !s.contains('%') {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // Try to decode a hex sequence
            let hex1 = chars.next();
            let hex2 = chars.next();

            if let (Some(h1), Some(h2)) = (hex1, hex2) {
                if let (Some(d1), Some(d2)) = (hex_digit(h1), hex_digit(h2)) {
                    result.push((d1 * 16 + d2) as char);
                    continue;
                }
            }

            // Invalid sequence, keep the %
            result.push('%');
            if let Some(h1) = hex1 {
                result.push(h1);
            }
            if let Some(h2) = hex2 {
                result.push(h2);
            }
        } else {
            result.push(c);
        }
    }

    Cow::Owned(result)
}

/// Convert a hex digit character to its numeric value.
fn hex_digit(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'A'..='F' => Some(c as u8 - b'A' + 10),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        _ => None,
    }
}

/// Escape a URI string.
///
/// This function escapes whitespace and certain punctuation characters
/// in a URI string. It's useful for preparing URIs for display or
/// embedding in other formats.
///
/// # Arguments
///
/// * `s` - The URI string to escape
///
/// # Returns
///
/// The escaped URI string
///
/// # Example
///
/// ```ignore
/// use clmd::text::uri::escape_uri;
///
/// assert_eq!(escape_uri("hello world"), "hello%20world");
/// assert_eq!(escape_uri("<tag>"), "%3Ctag%3E");
/// ```ignore
pub fn escape_uri(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ' ' => result.push_str("%20"),
            '\t' => result.push_str("%09"),
            '\n' => result.push_str("%0A"),
            '\r' => result.push_str("%0D"),
            '<' => result.push_str("%3C"),
            '>' => result.push_str("%3E"),
            '"' => result.push_str("%22"),
            '{' => result.push_str("%7B"),
            '}' => result.push_str("%7D"),
            '|' => result.push_str("%7C"),
            '\\' => result.push_str("%5C"),
            '^' => result.push_str("%5E"),
            '[' => result.push_str("%5B"),
            ']' => result.push_str("%5D"),
            '`' => result.push_str("%60"),
            _ => result.push(c),
        }
    }
    result
}

/// Check if a string is a valid URI.
///
/// This function checks if a string looks like a valid URI by checking
/// for a known scheme prefix.
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// `true` if the string appears to be a valid URI
///
/// # Example
///
/// ```ignore
/// use clmd::text::uri::is_uri;
///
/// assert!(is_uri("https://example.com"));
/// assert!(is_uri("http://example.com"));
/// assert!(is_uri("ftp://files.example.com"));
/// assert!(is_uri("mailto:test@example.com"));
/// assert!(is_uri("file:///path/to/file"));
/// assert!(!is_uri("not a uri"));
/// assert!(!is_uri("example.com")); // Missing scheme
/// ```ignore
pub fn is_uri(s: &str) -> bool {
    // Check for scheme prefix (e.g., "http://", "https://", "mailto:")
    s.contains("://") || s.starts_with("mailto:") || s.starts_with("data:")
}

/// Get the scheme from a URI.
///
/// # Arguments
///
/// * `uri` - The URI string
///
/// # Returns
///
/// The scheme (e.g., "http", "https", "mailto") if present, or `None`
///
/// # Example
///
/// ```ignore
/// use clmd::text::uri::get_scheme;
///
/// assert_eq!(get_scheme("https://example.com"), Some("https"));
/// assert_eq!(get_scheme("mailto:test@example.com"), Some("mailto"));
/// assert_eq!(get_scheme("not a uri"), None);
/// ```ignore
pub fn get_scheme(uri: &str) -> Option<&str> {
    if let Some(pos) = uri.find(':') {
        let scheme = &uri[..pos];
        // Scheme must start with a letter and contain only valid characters
        if scheme
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
        {
            return Some(scheme);
        }
    }
    None
}

/// Check if a URI is absolute (has a scheme).
///
/// # Arguments
///
/// * `uri` - The URI string
///
/// # Returns
///
/// `true` if the URI is absolute
///
/// # Example
///
/// ```ignore
/// use clmd::text::uri::is_absolute_uri;
///
/// assert!(is_absolute_uri("https://example.com"));
/// assert!(!is_absolute_uri("/path/to/file"));
/// assert!(!is_absolute_uri("relative/path"));
/// ```ignore
pub fn is_absolute_uri(uri: &str) -> bool {
    get_scheme(uri).is_some()
}

/// Check if a URI is a data URI.
///
/// Data URIs embed data directly in the URI using the `data:` scheme.
///
/// # Arguments
///
/// * `uri` - The URI string
///
/// # Returns
///
/// `true` if the URI is a data URI
///
/// # Example
///
/// ```ignore
/// use clmd::text::uri::is_data_uri;
///
/// assert!(is_data_uri("data:image/png;base64,iVBORw0KGgo="));
/// assert!(!is_data_uri("https://example.com/image.png"));
/// ```ignore
pub fn is_data_uri(uri: &str) -> bool {
    uri.starts_with("data:")
}

/// Parse a data URI and extract its components.
///
/// # Arguments
///
/// * `uri` - The data URI string
///
/// # Returns
///
/// A tuple of (mime_type, data) if parsing succeeds, or `None`
///
/// # Example
///
/// ```ignore
/// use clmd::text::uri::parse_data_uri;
///
/// let result = parse_data_uri("data:text/plain;base64,SGVsbG8gV29ybGQ=");
/// assert!(result.is_some());
///
/// let (mime, data) = result.unwrap();
/// assert_eq!(mime, "text/plain");
/// ```ignore
pub fn parse_data_uri(uri: &str) -> Option<(&str, &str)> {
    if !uri.starts_with("data:") {
        return None;
    }

    let rest = &uri[5..]; // Skip "data:"

    // Find the comma separator
    let comma_pos = rest.find(',')?;
    let metadata = &rest[..comma_pos];
    let data = &rest[comma_pos + 1..];

    // Parse metadata
    let mime_type = if metadata.is_empty() {
        "text/plain"
    } else if metadata.contains(';') {
        // Extract MIME type before semicolon
        metadata.split(';').next().unwrap_or("text/plain")
    } else {
        metadata
    };

    Some((mime_type, data))
}

/// Known URI schemes.
///
/// This list includes common URI schemes from IANA and other sources.
pub const KNOWN_SCHEMES: &[&str] = &[
    // Web protocols
    "http",
    "https",
    "ftp",
    "ftps",
    "sftp",
    // File protocols
    "file",
    "nfs",
    // Email protocols
    "mailto",
    "imap",
    "pop",
    "smtp",
    // Media protocols
    "rtsp",
    "rtmp",
    "mms",
    // Other protocols
    "data",
    "javascript",
    "irc",
    "ircs",
    "xmpp",
    "ssh",
    "telnet",
    "ldap",
    "ldaps",
    "news",
    "nntp",
    "tel",
    "sms",
    "geo",
    "market",
    "steam",
    "itms",
    // Version control
    "git",
    "svn",
    "hg",
    // Document formats
    "doi",
    "isbn",
];

/// Check if a scheme is a known URI scheme.
///
/// # Arguments
///
/// * `scheme` - The scheme to check
///
/// # Returns
///
/// `true` if the scheme is known
///
/// # Example
///
/// ```ignore
/// use clmd::text::uri::is_known_scheme;
///
/// assert!(is_known_scheme("https"));
/// assert!(is_known_scheme("mailto"));
/// assert!(!is_known_scheme("unknown"));
/// ```ignore
pub fn is_known_scheme(scheme: &str) -> bool {
    KNOWN_SCHEMES.contains(&scheme.to_lowercase().as_str())
}

/// Normalize a URI path.
///
/// This function resolves `.` and `..` components in a path.
///
/// # Arguments
///
/// * `path` - The path to normalize
///
/// # Returns
///
/// The normalized path
///
/// # Example
///
/// ```ignore
/// use clmd::text::uri::normalize_path;
///
/// assert_eq!(normalize_path("/foo/bar/../baz"), "/foo/baz");
/// assert_eq!(normalize_path("/foo/./bar"), "/foo/bar");
/// assert_eq!(normalize_path("/foo/bar/../.."), "/");
/// ```ignore
pub fn normalize_path(path: &str) -> String {
    let mut components = Vec::new();

    for component in path.split('/') {
        match component {
            "" | "." => {
                // Skip empty components and current directory
            }
            ".." => {
                // Parent directory - pop if possible
                if let Some(last) = components.last() {
                    if last != &"" {
                        components.pop();
                    }
                }
            }
            _ => {
                components.push(component);
            }
        }
    }

    // Preserve leading slash
    let leading_slash = path.starts_with('/');
    let result = if leading_slash {
        "/".to_string() + &components.join("/")
    } else {
        components.join("/")
    };

    // Preserve trailing slash
    if path.ends_with('/') && !result.ends_with('/') && !result.is_empty() {
        result + "/"
    } else {
        result
    }
}

/// Join two URI paths.
///
/// This function joins a base path with a relative path, similar to
/// how web browsers resolve relative URLs.
///
/// # Arguments
///
/// * `base` - The base path
/// * `relative` - The relative path to join
///
/// # Returns
///
/// The joined path
///
/// # Example
///
/// ```ignore
/// use clmd::text::uri::join_paths;
///
/// assert_eq!(join_paths("/foo/bar", "baz"), "/foo/baz");
/// assert_eq!(join_paths("/foo/bar/", "baz"), "/foo/bar/baz");
/// assert_eq!(join_paths("/foo/bar", "../baz"), "/baz");
/// ```ignore
pub fn join_paths(base: &str, relative: &str) -> String {
    if relative.starts_with('/') {
        // Absolute path - return as-is
        return relative.to_string();
    }

    // Get the directory part of the base
    let base_dir = if base.ends_with('/') {
        base.to_string()
    } else if let Some(pos) = base.rfind('/') {
        base[..pos + 1].to_string()
    } else {
        String::new()
    };

    normalize_path(&(base_dir + relative))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello%20world");
        assert_eq!(url_encode("foo/bar"), "foo%2Fbar");
        assert_eq!(url_encode("100%"), "100%25");
        assert_eq!(url_encode("test&foo"), "test%26foo");
        assert_eq!(url_encode("hello-world_test.txt"), "hello-world_test.txt");
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("foo%2Fbar"), "foo/bar");
        assert_eq!(url_decode("100%25"), "100%");
        assert_eq!(url_decode("hello-world_test.txt"), "hello-world_test.txt");

        // Invalid sequences should be preserved
        assert_eq!(url_decode("hello%ZZ"), "hello%ZZ");
        assert_eq!(url_decode("hello%"), "hello%");
    }

    #[test]
    fn test_escape_uri() {
        assert_eq!(escape_uri("hello world"), "hello%20world");
        assert_eq!(escape_uri("<tag>"), "%3Ctag%3E");
        assert_eq!(escape_uri("test|pipe"), "test%7Cpipe");
        assert_eq!(escape_uri("path\\to\\file"), "path%5Cto%5Cfile");
    }

    #[test]
    fn test_is_uri() {
        assert!(is_uri("https://example.com"));
        assert!(is_uri("http://example.com"));
        assert!(is_uri("ftp://files.example.com"));
        assert!(is_uri("mailto:test@example.com"));
        assert!(is_uri("file:///path/to/file"));
        assert!(is_uri("data:image/png;base64,abc"));

        assert!(!is_uri("not a uri"));
        assert!(!is_uri("example.com"));
        assert!(!is_uri("/path/to/file"));
    }

    #[test]
    fn test_get_scheme() {
        assert_eq!(get_scheme("https://example.com"), Some("https"));
        assert_eq!(get_scheme("mailto:test@example.com"), Some("mailto"));
        assert_eq!(get_scheme("data:text/plain,hello"), Some("data"));
        assert_eq!(get_scheme("not a uri"), None);
        assert_eq!(get_scheme("123://invalid"), None);
    }

    #[test]
    fn test_is_absolute_uri() {
        assert!(is_absolute_uri("https://example.com"));
        assert!(is_absolute_uri("mailto:test@example.com"));
        assert!(!is_absolute_uri("/path/to/file"));
        assert!(!is_absolute_uri("relative/path"));
    }

    #[test]
    fn test_is_data_uri() {
        assert!(is_data_uri("data:image/png;base64,abc"));
        assert!(is_data_uri("data:text/plain,hello"));
        assert!(!is_data_uri("https://example.com"));
        assert!(!is_data_uri("not a data uri"));
    }

    #[test]
    fn test_parse_data_uri() {
        let result = parse_data_uri("data:text/plain;base64,SGVsbG8=");
        assert_eq!(result, Some(("text/plain", "SGVsbG8=")));

        let result = parse_data_uri("data:image/png,abc123");
        assert_eq!(result, Some(("image/png", "abc123")));

        let result = parse_data_uri("data:,hello");
        assert_eq!(result, Some(("text/plain", "hello")));

        assert_eq!(parse_data_uri("not a data uri"), None);
        assert_eq!(parse_data_uri("data:"), None);
    }

    #[test]
    fn test_is_known_scheme() {
        assert!(is_known_scheme("https"));
        assert!(is_known_scheme("mailto"));
        assert!(is_known_scheme("file"));
        assert!(is_known_scheme("HTTPS")); // case insensitive
        assert!(!is_known_scheme("unknown"));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/foo/bar/../baz"), "/foo/baz");
        assert_eq!(normalize_path("/foo/./bar"), "/foo/bar");
        assert_eq!(normalize_path("/foo/bar/../.."), "/");
        assert_eq!(normalize_path("foo/bar/../baz"), "foo/baz");
        assert_eq!(normalize_path("/foo/bar/"), "/foo/bar/");
    }

    #[test]
    fn test_join_paths() {
        assert_eq!(join_paths("/foo/bar", "baz"), "/foo/baz");
        assert_eq!(join_paths("/foo/bar/", "baz"), "/foo/bar/baz");
        assert_eq!(join_paths("/foo/bar", "../baz"), "/baz");
        assert_eq!(join_paths("/foo/bar", "/absolute"), "/absolute");
        assert_eq!(join_paths("foo", "bar"), "bar");
    }
}
