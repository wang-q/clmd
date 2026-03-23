pub mod html;
pub mod xml;

use crate::node::Node;
use std::cell::RefCell;
use std::rc::Rc;

/// Base renderer trait
pub trait Renderer {
    fn render(&mut self, root: &Rc<RefCell<Node>>, options: u32) -> String;
}

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
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
fn is_safe_url(url: &str) -> bool {
    let url = url.to_lowercase();
    !url.starts_with("javascript:")
        && !url.starts_with("vbscript:")
        && !url.starts_with("file:")
        && !url.starts_with("data:")
}
