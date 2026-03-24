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
pub fn escape_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
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
