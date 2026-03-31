//! Tagfilter extension (GFM)
//!
//! This module implements the GFM tagfilter extension, which filters out
//! certain HTML tags that are considered potentially dangerous.
//!
//! The following tags are filtered:
//! - `<title>`, `<textarea>`, `<style>`, `<xmp>`
//! - `<iframe>`, `<noembed>`, `<noframes>`
//! - `<script>`, `<plaintext>`
//!
//! When tagfilter is enabled, these tags are escaped rather than rendered
//! as raw HTML, preventing potential XSS attacks.

use crate::html_utils::escape_html;
use once_cell::sync::Lazy;
use std::collections::HashSet;

/// Set of HTML tag names that are disallowed by the GFM tagfilter extension.
static DISALLOWED_TAGS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("title");
    set.insert("textarea");
    set.insert("style");
    set.insert("xmp");
    set.insert("iframe");
    set.insert("noembed");
    set.insert("noframes");
    set.insert("script");
    set.insert("plaintext");
    set
});

/// Check if a tag name is in the disallowed list.
///
/// The comparison is case-insensitive.
pub fn is_disallowed_tag(tag: &str) -> bool {
    DISALLOWED_TAGS.contains(tag.to_lowercase().as_str())
}

/// Extract tag name from an HTML tag string.
///
/// Handles both opening tags (`<tag>`) and closing tags (`</tag>`).
/// Returns the lowercase tag name if found, or None if not a valid tag.
fn extract_tag_name(tag: &str) -> Option<String> {
    let mut chars = tag.chars();

    // Must start with '<'
    if chars.next() != Some('<') {
        return None;
    }

    // Skip optional '/'
    let is_end_tag = chars.clone().next() == Some('/');
    if is_end_tag {
        chars.next();
    }

    // Collect tag name
    let mut name = String::new();
    for ch in chars {
        if ch.is_ascii_whitespace() || ch == '>' || ch == '/' {
            break;
        }
        name.push(ch);
    }

    if name.is_empty() {
        None
    } else {
        Some(name.to_lowercase())
    }
}

/// Filter HTML content, escaping disallowed tags.
///
/// This function scans through HTML content and escapes any tags that are
/// in the disallowed list. Allowed tags are passed through unchanged.
///
/// # Arguments
///
/// * `html` - The HTML content to filter
///
/// # Returns
///
/// The filtered HTML with disallowed tags escaped.
pub fn filter_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut i = 0;
    let bytes = html.as_bytes();

    while i < html.len() {
        if bytes[i] == b'<' {
            // Check if this is a comment
            if html[i..].starts_with("<!--") {
                // Find end of comment
                if let Some(end) = html[i..].find("-->") {
                    let comment_end = i + end + 3;
                    result.push_str(&html[i..comment_end]);
                    i = comment_end;
                    continue;
                }
            }

            // Check if this is a doctype
            if html[i..].to_uppercase().starts_with("<!DOCTYPE") {
                // Find end of doctype
                if let Some(end) = html[i..].find('>') {
                    let doctype_end = i + end + 1;
                    result.push_str(&html[i..doctype_end]);
                    i = doctype_end;
                    continue;
                }
            }

            // Find the end of this tag
            let mut in_quote: Option<char> = None;
            let mut j = i + 1;

            while j < html.len() {
                let ch = bytes[j] as char;

                match ch {
                    '"' | '\'' => {
                        if in_quote == Some(ch) {
                            in_quote = None;
                        } else if in_quote.is_none() {
                            in_quote = Some(ch);
                        }
                    }
                    '>' if in_quote.is_none() => {
                        j += 1;
                        break;
                    }
                    _ => {}
                }
                j += 1;
            }

            // Extract the tag
            let tag = &html[i..j];

            // Check if this is a disallowed tag
            if let Some(tag_name) = extract_tag_name(tag) {
                if is_disallowed_tag(&tag_name) {
                    // Escape the entire tag
                    result.push_str(&escape_html(tag));
                } else {
                    // Keep allowed tags as-is
                    result.push_str(tag);
                }
            } else {
                // Not a valid tag, just copy
                result.push_str(tag);
            }

            i = j;
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_disallowed_tag() {
        assert!(is_disallowed_tag("script"));
        assert!(is_disallowed_tag("SCRIPT"));
        assert!(is_disallowed_tag("Script"));
        assert!(is_disallowed_tag("iframe"));
        assert!(is_disallowed_tag("style"));
        assert!(!is_disallowed_tag("div"));
        assert!(!is_disallowed_tag("p"));
    }

    #[test]
    fn test_extract_tag_name() {
        assert_eq!(extract_tag_name("<script>"), Some("script".to_string()));
        assert_eq!(extract_tag_name("</script>"), Some("script".to_string()));
        assert_eq!(extract_tag_name("<SCRIPT>"), Some("script".to_string()));
        assert_eq!(
            extract_tag_name("<div class=\"test\">"),
            Some("div".to_string())
        );
        assert_eq!(
            extract_tag_name("<iframe src=\"test\">"),
            Some("iframe".to_string())
        );
    }

    #[test]
    fn test_filter_html_script() {
        let input = "<script>alert('xss')</script>";
        let expected = "&lt;script&gt;alert('xss')&lt;/script&gt;";
        assert_eq!(filter_html(input), expected);
    }

    #[test]
    fn test_filter_html_iframe() {
        let input = r#"<iframe src="http://evil.com"></iframe>"#;
        let expected =
            r#"&lt;iframe src=&quot;http://evil.com&quot;&gt;&lt;/iframe&gt;"#;
        assert_eq!(filter_html(input), expected);
    }

    #[test]
    fn test_filter_html_allowed_tags() {
        let input = "<div><p>Hello</p></div>";
        assert_eq!(filter_html(input), input);
    }

    #[test]
    fn test_filter_html_mixed() {
        let input = "<div><script>evil()</script><p>safe</p></div>";
        let expected = "<div>&lt;script&gt;evil()&lt;/script&gt;<p>safe</p></div>";
        assert_eq!(filter_html(input), expected);
    }

    #[test]
    fn test_filter_html_with_attributes() {
        let input = r#"<script type="text/javascript" src="evil.js">alert(1)</script>"#;
        let result = filter_html(input);
        assert!(result.starts_with("&lt;script"));
        assert!(result.contains("&gt;"));
        assert!(!result.contains("<script "));
    }

    #[test]
    fn test_filter_html_style() {
        let input = "<style>body { background: red; }</style>";
        let expected = "&lt;style&gt;body { background: red; }&lt;/style&gt;";
        assert_eq!(filter_html(input), expected);
    }

    #[test]
    fn test_filter_html_textarea() {
        let input = "<textarea>content</textarea>";
        let expected = "&lt;textarea&gt;content&lt;/textarea&gt;";
        assert_eq!(filter_html(input), expected);
    }

    #[test]
    fn test_filter_html_comment_preserved() {
        let input = "<!-- comment --><script>evil()</script>";
        let result = filter_html(input);
        assert!(result.contains("<!-- comment -->"));
        assert!(result.contains("&lt;script&gt;"));
    }
}
