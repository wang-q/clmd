//! Inline element handlers for CommonMark formatting
//!
//! This module contains handlers for inline elements like text, code,
//! emphasis, strong, links, images, and breaks.

use crate::render::commonmark::core::NodeFormatterContext;
use crate::render::commonmark::escaping::{escape_string, escape_url};
use crate::render::commonmark::writer::MarkdownWriter;

/// Check if a URL is a reference-style link label
///
/// Reference-style links use a label instead of a direct URL.
/// The label is case-insensitive and can contain letters, numbers, spaces, and punctuation.
pub fn is_reference_label(url: &str) -> bool {
    if url.contains("://") {
        return false;
    }

    if url.starts_with('/') || url.starts_with("./") || url.starts_with("../") {
        return false;
    }

    if url.starts_with('#') || url.starts_with('?') {
        return false;
    }

    if url.is_empty() {
        return false;
    }

    let schemes = [
        "http:",
        "https:",
        "ftp:",
        "mailto:",
        "file:",
        "data:",
        "javascript:",
        "vbscript:",
    ];
    for scheme in &schemes {
        if url.starts_with(scheme) {
            return false;
        }
    }

    if url.contains('.')
        && !url.contains(' ')
        && !url.contains('[')
        && !url.contains(']')
    {
        let domain_part: String = url
            .chars()
            .take_while(|&c| c != '/' && c != '?' && c != '#')
            .collect();
        if domain_part.contains('.')
            && domain_part
                .chars()
                .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            return false;
        }
    }

    true
}

/// Render a link URL, handling reference-style links
///
/// This function determines whether a link is inline (with direct URL)
/// or reference-style (with label), and renders accordingly.
pub fn render_link_url(
    url: &str,
    title: &str,
    _ctx: &dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    if is_reference_label(url) {
        writer.append("[");
        writer.append(url);
        writer.append("]");
    } else {
        writer.append("(");
        writer.append(escape_url(url));
        if !title.is_empty() {
            writer.append(format!(" \"{}\"", escape_string(title)));
        }
        writer.append(")");
    }
}

/// Render an image URL, handling reference-style images
///
/// Similar to render_link_url but for images.
pub fn render_image_url(
    url: &str,
    title: &str,
    _ctx: &dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    if is_reference_label(url) {
        writer.append("[");
        writer.append(url);
        writer.append("]");
    } else {
        writer.append("(");
        writer.append(escape_url(url));
        if !title.is_empty() {
            writer.append(format!(" \"{}\"", escape_string(title)));
        }
        writer.append(")");
    }
}
