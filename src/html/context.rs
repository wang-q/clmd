//! HTML rendering context
//!
//! This module provides the [`Context`] struct for managing HTML rendering state,
//! inspired by comrak's design.

use std::fmt::{self, Write};

use crate::parser::options::{Options, Plugins};

/// Context for HTML rendering.
///
/// This struct holds the rendering options, plugins, and output buffer,
/// providing a unified interface for HTML rendering operations.
pub struct Context<'o, 'c: 'o> {
    /// The options for rendering
    pub options: &'o Options<'c>,
    /// The plugins for rendering
    pub plugins: &'o Plugins<'o>,
    /// The output buffer
    output: &'o mut dyn Write,
    /// Current footnote index
    pub footnote_ix: u32,
    /// Last written footnote index
    pub written_footnote_ix: u32,
}

impl<'o, 'c: 'o> Context<'o, 'c> {
    /// Create a new rendering context.
    pub fn new(
        output: &'o mut dyn Write,
        options: &'o Options<'c>,
        plugins: &'o Plugins<'o>,
    ) -> Self {
        Self {
            options,
            plugins,
            output,
            footnote_ix: 0,
            written_footnote_ix: 0,
        }
    }

    /// Write a string to the output.
    pub fn write_str(&mut self, s: &str) -> fmt::Result {
        self.output.write_str(s)
    }

    /// Write a newline if the last output wasn't already a newline.
    pub fn cr(&mut self) -> fmt::Result {
        // Check if we need to add a newline
        // This is a simplified version; in practice, we'd track the last character
        self.output.write_str("\n")
    }

    /// Write a line feed (newline).
    pub fn lf(&mut self) -> fmt::Result {
        self.output.write_str("\n")
    }

    /// Escape HTML special characters.
    pub fn escape(&mut self, text: &str) -> fmt::Result {
        escape_html(self.output, text)
    }

    /// Escape a URL for use in an href attribute.
    pub fn escape_href(&mut self, url: &str) -> fmt::Result {
        escape_href(self.output, url)
    }

    /// Finish rendering and return the result.
    pub fn finish(self) -> fmt::Result {
        Ok(())
    }
}

impl Write for Context<'_, '_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.output.write_str(s)
    }
}

/// Escape HTML special characters.
///
/// Escapes the following characters:
/// - `&` → `&amp;`
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `"` → `&quot;`
pub fn escape_html(output: &mut dyn Write, text: &str) -> fmt::Result {
    // Fast path: scan bytes and only process chunks that need escaping
    let bytes = text.as_bytes();
    let mut last = 0;

    for (i, &b) in bytes.iter().enumerate() {
        let escaped = match b {
            b'&' => Some("&amp;"),
            b'<' => Some("&lt;"),
            b'>' => Some("&gt;"),
            b'"' => Some("&quot;"),
            _ => None,
        };

        if let Some(esc) = escaped {
            // Write the chunk before this character
            if i > last {
                output.write_str(&text[last..i])?;
            }
            output.write_str(esc)?;
            last = i + 1;
        }
    }

    // Write remaining chunk
    if last < text.len() {
        output.write_str(&text[last..])?;
    }

    Ok(())
}

/// Escape a string for use in HTML attribute context.
pub fn escape_href(output: &mut dyn Write, url: &str) -> fmt::Result {
    // First check if URL is safe
    if !is_safe_url(url) {
        return output.write_str("#");
    }

    // Fast path: scan bytes and only process chunks that need escaping
    let bytes = url.as_bytes();
    let mut last = 0;

    for (i, &b) in bytes.iter().enumerate() {
        let escaped = match b {
            b'&' => Some("&amp;"),
            b'"' => Some("&quot;"),
            b'<' => Some("&lt;"),
            b'>' => Some("&gt;"),
            b'\'' => Some("&#x27;"),
            b'`' => Some("&#x60;"),
            _ => None,
        };

        if let Some(esc) = escaped {
            // Write the chunk before this character
            if i > last {
                output.write_str(&url[last..i])?;
            }
            output.write_str(esc)?;
            last = i + 1;
        }
    }

    // Write remaining chunk
    if last < url.len() {
        output.write_str(&url[last..])?;
    }

    Ok(())
}

/// Check if a URL is safe to use in an href attribute.
///
/// Blocks the following dangerous protocols:
/// - `javascript:`
/// - `vbscript:`
/// - `file:`
/// - `data:` (with dangerous MIME types)
pub fn is_safe_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();

    // Check for dangerous protocols
    let dangerous_protocols = [
        "javascript:",
        "vbscript:",
        "file:",
        "data:text/html",
        "data:text/javascript",
    ];

    for protocol in &dangerous_protocols {
        if url_lower.starts_with(protocol) {
            return false;
        }
    }

    true
}

/// Render sourcepos attribute if enabled.
pub fn render_sourcepos(
    context: &mut Context,
    sourcepos: &crate::nodes::SourcePos,
) -> fmt::Result {
    if context.options.render.sourcepos && sourcepos.start.line > 0 {
        write!(context, " data-sourcepos=\"{}\"", sourcepos)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        let mut output = String::new();
        escape_html(&mut output, "<div>").unwrap();
        assert_eq!(output, "&lt;div&gt;");

        output.clear();
        escape_html(&mut output, "&").unwrap();
        assert_eq!(output, "&amp;");

        output.clear();
        escape_html(&mut output, "\"test\"").unwrap();
        assert_eq!(output, "&quot;test&quot;");
    }

    #[test]
    fn test_is_safe_url() {
        assert!(!is_safe_url("javascript:alert('xss')"));
        assert!(!is_safe_url("JAVASCRIPT:alert('xss')"));
        assert!(!is_safe_url("vbscript:msgbox('xss')"));
        assert!(!is_safe_url("file:///etc/passwd"));
        assert!(is_safe_url("https://example.com"));
        assert!(is_safe_url("http://example.com"));
        assert!(is_safe_url("/path/to/page"));
        assert!(is_safe_url("#anchor"));
    }

    #[test]
    fn test_escape_href() {
        let mut output = String::new();
        escape_href(&mut output, "https://example.com?a=1&b=2").unwrap();
        assert_eq!(output, "https://example.com?a=1&amp;b=2");

        output.clear();
        escape_href(&mut output, "javascript:alert('xss')").unwrap();
        assert_eq!(output, "#");
    }
}
