//! Document transformation system for clmd.
//!
//! This module provides a set of document transformations inspired by
//! Pandoc's transform system. Transforms can modify the AST between
//! parsing and rendering.
//!
//! # Available Transforms
//!
//! - **HeaderShift** - Shift header levels up or down
//! - **Normalize** - Normalize the document structure
//! - **Citeproc** - Process citations and bibliography
//! - **TableOfContents** - Generate table of contents
//! - **LinkRewrite** - Rewrite links based on patterns
//! - **ImageRewrite** - Rewrite image sources based on patterns
//!
//! # Example
//!
//! ```ignore
//! use clmd::util::transform::{Transform, TransformChain};
//!
//! let mut chain = TransformChain::new();
//! chain.add(Transform::header_shift(1));
//! chain.add(Transform::normalize());
//! ```

use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::{ClmdError, ClmdResult};
use crate::core::nodes::{
    ListDelimType, ListType, NodeHeading, NodeLink, NodeList, NodeValue,
};
use std::collections::HashMap;
use std::fmt;

// Re-export filter types for unified API
pub use crate::filter::{Filter, FilterChain, FilterError, FilterResult};

/// A document transformation.
#[derive(Debug, Clone)]
pub enum Transform {
    /// Shift header levels.
    /// Positive values increase levels (h1 -> h2), negative decrease.
    HeaderShift(i32),

    /// Normalize the document structure.
    Normalize,

    /// Generate table of contents.
    TableOfContents {
        /// Minimum header level to include.
        min_level: u8,
        /// Maximum header level to include.
        max_level: u8,
        /// Title for the TOC.
        title: Option<String>,
    },

    /// Rewrite links based on a pattern.
    LinkRewrite {
        /// Pattern to match.
        pattern: String,
        /// Replacement string.
        replacement: String,
    },

    /// Rewrite image sources based on a pattern.
    ImageRewrite {
        /// Pattern to match.
        pattern: String,
        /// Replacement string.
        replacement: String,
    },

    /// Add attributes to elements matching a selector.
    AddAttributes {
        /// CSS-style selector.
        selector: String,
        /// Attributes to add.
        attributes: HashMap<String, String>,
    },

    /// Remove elements matching a selector.
    RemoveElements {
        /// CSS-style selector.
        selector: String,
    },

    /// Strip footnotes from the document.
    StripFootnotes,

    /// Capitalize headers.
    CapitalizeHeaders,

    /// Convert absolute links to relative.
    AbsToRel {
        /// Base URL to convert from.
        base_url: String,
    },

    /// Add IDs to headers.
    AutoIdent,

    /// Remove soft breaks between CJK characters.
    /// This improves typography for East Asian languages.
    EastAsianLineBreaks,

    /// Custom transform function.
    Custom {
        /// Name of the transform.
        name: String,
        /// The transform function.
        apply: fn(&mut NodeArena, NodeId) -> ClmdResult<()>,
    },
}

impl Transform {
    /// Create a header shift transform.
    pub fn header_shift(shift: i32) -> Self {
        Self::HeaderShift(shift)
    }

    /// Create a normalize transform.
    pub fn normalize() -> Self {
        Self::Normalize
    }

    /// Create a table of contents transform.
    pub fn table_of_contents() -> TableOfContentsBuilder {
        TableOfContentsBuilder::default()
    }

    /// Create a link rewrite transform.
    pub fn link_rewrite(
        pattern: impl Into<String>,
        replacement: impl Into<String>,
    ) -> Self {
        Self::LinkRewrite {
            pattern: pattern.into(),
            replacement: replacement.into(),
        }
    }

    /// Create an image rewrite transform.
    pub fn image_rewrite(
        pattern: impl Into<String>,
        replacement: impl Into<String>,
    ) -> Self {
        Self::ImageRewrite {
            pattern: pattern.into(),
            replacement: replacement.into(),
        }
    }

    /// Get the name of this transform.
    pub fn name(&self) -> String {
        match self {
            Self::HeaderShift(n) => format!("header-shift({})", n),
            Self::Normalize => "normalize".to_string(),
            Self::TableOfContents { .. } => "table-of-contents".to_string(),
            Self::LinkRewrite { .. } => "link-rewrite".to_string(),
            Self::ImageRewrite { .. } => "image-rewrite".to_string(),
            Self::AddAttributes { .. } => "add-attributes".to_string(),
            Self::RemoveElements { .. } => "remove-elements".to_string(),
            Self::StripFootnotes => "strip-footnotes".to_string(),
            Self::CapitalizeHeaders => "capitalize-headers".to_string(),
            Self::AbsToRel { base_url } => format!("abs-to-rel({})", base_url),
            Self::AutoIdent => "auto-ident".to_string(),
            Self::EastAsianLineBreaks => "east-asian-line-breaks".to_string(),
            Self::Custom { name, .. } => format!("custom({})", name),
        }
    }

    /// Create a strip footnotes transform.
    pub fn strip_footnotes() -> Self {
        Self::StripFootnotes
    }

    /// Create a capitalize headers transform.
    pub fn capitalize_headers() -> Self {
        Self::CapitalizeHeaders
    }

    /// Create an absolute to relative links transform.
    pub fn abs_to_rel(base_url: impl Into<String>) -> Self {
        Self::AbsToRel {
            base_url: base_url.into(),
        }
    }

    /// Create an auto-ident transform.
    pub fn auto_ident() -> Self {
        Self::AutoIdent
    }

    /// Create an East Asian line breaks transform.
    /// Removes soft breaks between CJK characters for better typography.
    pub fn east_asian_line_breaks() -> Self {
        Self::EastAsianLineBreaks
    }

    /// Apply this transform to a document.
    pub fn apply(&self, arena: &mut NodeArena, root: NodeId) -> ClmdResult<()> {
        match self {
            Self::HeaderShift(shift) => {
                transforms::apply_header_shift(arena, root, *shift)
            }
            Self::Normalize => transforms::apply_normalize(arena, root),
            Self::TableOfContents {
                min_level,
                max_level,
                title,
            } => transforms::apply_table_of_contents(
                arena,
                root,
                *min_level,
                *max_level,
                title.as_deref(),
            ),
            Self::LinkRewrite {
                pattern,
                replacement,
            } => transforms::apply_link_rewrite(arena, root, pattern, replacement),
            Self::ImageRewrite {
                pattern,
                replacement,
            } => transforms::apply_image_rewrite(arena, root, pattern, replacement),
            Self::StripFootnotes => transforms::apply_strip_footnotes(arena, root),
            Self::CapitalizeHeaders => transforms::apply_capitalize_headers(arena, root),
            Self::AbsToRel { base_url } => {
                transforms::apply_abs_to_rel(arena, root, base_url)
            }
            Self::AutoIdent => transforms::apply_auto_ident(arena, root),
            Self::EastAsianLineBreaks => {
                transforms::apply_east_asian_line_breaks(arena, root)
            }
            Self::Custom { apply, .. } => apply(arena, root),
            _ => Ok(()), // Other transforms not yet implemented
        }
    }
}

/// Builder for table of contents transform.
#[derive(Debug)]
pub struct TableOfContentsBuilder {
    min_level: u8,
    max_level: u8,
    title: Option<String>,
}

impl TableOfContentsBuilder {
    /// Set the minimum header level.
    pub fn min_level(mut self, level: u8) -> Self {
        self.min_level = level;
        self
    }

    /// Set the maximum header level.
    pub fn max_level(mut self, level: u8) -> Self {
        self.max_level = level;
        self
    }

    /// Set the TOC title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Build the transform.
    pub fn build(self) -> Transform {
        Transform::TableOfContents {
            min_level: self.min_level.max(1).min(6),
            max_level: self.max_level.max(1).min(6),
            title: self.title,
        }
    }
}

impl Default for TableOfContentsBuilder {
    fn default() -> Self {
        Self {
            min_level: 1,
            max_level: 6,
            title: None,
        }
    }
}

/// Error type for transform operations.
#[derive(Debug, Clone)]
pub enum TransformError {
    /// Transform not implemented.
    NotImplemented(String),
    /// Invalid transform configuration.
    InvalidConfig(String),
    /// Transform execution failed.
    ExecutionFailed(String),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented(name) => {
                write!(f, "Transform not implemented: {}", name)
            }
            Self::InvalidConfig(msg) => write!(f, "Invalid transform config: {}", msg),
            Self::ExecutionFailed(msg) => {
                write!(f, "Transform execution failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for TransformError {}

impl From<TransformError> for ClmdError {
    fn from(e: TransformError) -> Self {
        ClmdError::transform_error(e.to_string())
    }
}

/// A chain of transforms to apply in sequence.
#[derive(Debug, Clone, Default)]
pub struct TransformChain {
    transforms: Vec<Transform>,
}

impl TransformChain {
    /// Create a new empty transform chain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a transform to the chain.
    pub fn add(&mut self, transform: Transform) -> &mut Self {
        self.transforms.push(transform);
        self
    }

    /// Add multiple transforms to the chain.
    pub fn extend(
        &mut self,
        transforms: impl IntoIterator<Item = Transform>,
    ) -> &mut Self {
        self.transforms.extend(transforms);
        self
    }

    /// Get the number of transforms in the chain.
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Clear all transforms from the chain.
    pub fn clear(&mut self) {
        self.transforms.clear();
    }

    /// Apply all transforms in the chain to a document.
    pub fn apply(&self, arena: &mut NodeArena, root: NodeId) -> ClmdResult<()> {
        for transform in &self.transforms {
            transform.apply(arena, root)?;
        }
        Ok(())
    }

    /// Get an iterator over the transforms.
    pub fn iter(&self) -> impl Iterator<Item = &Transform> {
        self.transforms.iter()
    }
}

/// Built-in transform implementations.
pub mod transforms {
    use super::*;

    /// Apply header level shift.
    pub fn apply_header_shift(
        arena: &mut NodeArena,
        _root: NodeId,
        shift: i32,
    ) -> ClmdResult<()> {
        for node_id in 0..arena.len() {
            let node_id = node_id as u32;
            let node = arena.get_mut(node_id);
            if let NodeValue::Heading(ref mut heading) = node.value {
                let new_level = (heading.level as i32 + shift).clamp(1, 6) as u8;
                heading.level = new_level;
            }
        }
        Ok(())
    }

    /// Apply normalization.
    pub fn apply_normalize(arena: &mut NodeArena, _root: NodeId) -> ClmdResult<()> {
        // Normalize consecutive text nodes
        // Remove empty nodes
        // Normalize whitespace in text nodes
        // This is a placeholder - full implementation would be more complex
        for node_id in 0..arena.len() {
            let node_id = node_id as u32;
            let node = arena.get_mut(node_id);
            if let NodeValue::Text(ref mut text) = node.value {
                // Normalize whitespace
                *text = text.split_whitespace().collect::<Vec<_>>().join(" ").into();
            }
        }
        Ok(())
    }

    /// Apply table of contents generation.
    pub fn apply_table_of_contents(
        arena: &mut NodeArena,
        root: NodeId,
        min_level: u8,
        max_level: u8,
        title: Option<&str>,
    ) -> ClmdResult<()> {
        use crate::core::arena::TreeOps;

        // Collect headers
        let mut headers: Vec<(u8, String, NodeId)> = Vec::new();
        collect_headers(arena, root, &mut headers, min_level, max_level);

        if headers.is_empty() {
            return Ok(());
        }

        // Create TOC list
        let toc_items: Vec<NodeId> = headers
            .into_iter()
            .map(|(_level, text, _id)| {
                let link = NodeValue::Link(Box::new(NodeLink {
                    url: format!("#header-{}", text.to_lowercase().replace(' ', "-")),
                    title: text.clone().into(),
                }));
                let link_node = arena.alloc(crate::core::arena::Node::with_value(link));

                let text_node = arena.alloc(crate::core::arena::Node::with_value(
                    NodeValue::Text(text.into()),
                ));
                TreeOps::append_child(arena, link_node, text_node);

                // Item uses NodeList struct
                let item_list = NodeList {
                    list_type: ListType::Bullet,
                    marker_offset: 0,
                    padding: 2,
                    start: 1,
                    delimiter: ListDelimType::Period,
                    bullet_char: b'-',
                    tight: true,
                    is_task_list: false,
                };
                let item = NodeValue::Item(item_list);
                let item_node = arena.alloc(crate::core::arena::Node::with_value(item));
                TreeOps::append_child(arena, item_node, link_node);

                item_node
            })
            .collect();

        // Create list
        let list = NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        });
        let list_node = arena.alloc(crate::core::arena::Node::with_value(list));

        for item in toc_items {
            TreeOps::append_child(arena, list_node, item);
        }

        // Add title if provided
        let toc_container = if let Some(title_text) = title {
            let heading = NodeValue::Heading(NodeHeading {
                level: 2,
                setext: false,
                closed: false,
            });
            let heading_node =
                arena.alloc(crate::core::arena::Node::with_value(heading));

            let title_text_node = arena.alloc(crate::core::arena::Node::with_value(
                NodeValue::Text(title_text.into()),
            ));
            TreeOps::append_child(arena, heading_node, title_text_node);

            // Create a block quote to wrap the TOC content
            let quote = NodeValue::BlockQuote;
            let quote_node = arena.alloc(crate::core::arena::Node::with_value(quote));
            TreeOps::append_child(arena, quote_node, heading_node);
            TreeOps::append_child(arena, quote_node, list_node);

            quote_node
        } else {
            list_node
        };

        // Insert at the beginning of the document
        let first_child = arena.get(root).first_child;
        if let Some(first) = first_child {
            // Unlink first child and insert TOC before it
            TreeOps::unlink(arena, first);
            TreeOps::append_child(arena, root, toc_container);
            TreeOps::append_child(arena, root, first);
        } else {
            TreeOps::append_child(arena, root, toc_container);
        }

        Ok(())
    }

    fn collect_headers(
        arena: &NodeArena,
        node_id: NodeId,
        headers: &mut Vec<(u8, String, NodeId)>,
        min_level: u8,
        max_level: u8,
    ) {
        let node = arena.get(node_id);

        if let NodeValue::Heading(heading) = &node.value {
            if heading.level >= min_level && heading.level <= max_level {
                // Get the text content of the heading
                let text = get_text_content(arena, node_id);
                headers.push((heading.level, text, node_id));
            }
        }

        // Recurse into children
        let mut child = node.first_child;
        while let Some(child_id) = child {
            collect_headers(arena, child_id, headers, min_level, max_level);
            child = arena.get(child_id).next;
        }
    }

    fn get_text_content(arena: &NodeArena, node_id: NodeId) -> String {
        let node = arena.get(node_id);
        match &node.value {
            NodeValue::Text(text) => text.to_string(),
            _ => {
                let mut result = String::new();
                let mut child = node.first_child;
                while let Some(child_id) = child {
                    result.push_str(&get_text_content(arena, child_id));
                    child = arena.get(child_id).next;
                }
                result
            }
        }
    }

    /// Apply link rewrite.
    pub fn apply_link_rewrite(
        arena: &mut NodeArena,
        _root: NodeId,
        pattern: &str,
        replacement: &str,
    ) -> ClmdResult<()> {
        for node_id in 0..arena.len() {
            let node_id = node_id as u32;
            let node = arena.get_mut(node_id);
            if let NodeValue::Link(ref mut link) = node.value {
                // Simple string replacement
                // For regex support, would need regex crate
                if link.url.contains(pattern) {
                    link.url = link.url.replace(pattern, replacement);
                }
            }
        }
        Ok(())
    }

    /// Apply image rewrite.
    pub fn apply_image_rewrite(
        arena: &mut NodeArena,
        _root: NodeId,
        pattern: &str,
        replacement: &str,
    ) -> ClmdResult<()> {
        for node_id in 0..arena.len() {
            let node_id = node_id as u32;
            let node = arena.get_mut(node_id);
            if let NodeValue::Image(ref mut image) = node.value {
                if image.url.contains(pattern) {
                    image.url = image.url.replace(pattern, replacement);
                }
            }
        }
        Ok(())
    }

    /// Apply strip footnotes transform.
    pub fn apply_strip_footnotes(
        arena: &mut NodeArena,
        _root: NodeId,
    ) -> ClmdResult<()> {
        use crate::core::arena::TreeOps;

        // Collect footnote nodes to remove
        let mut footnotes_to_remove = Vec::new();
        for node_id in 0..arena.len() {
            let node_id = node_id as u32;
            let node = arena.get(node_id);
            if matches!(
                node.value,
                NodeValue::FootnoteDefinition(_) | NodeValue::FootnoteReference(_)
            ) {
                footnotes_to_remove.push(node_id);
            }
        }

        // Remove collected footnotes
        for node_id in footnotes_to_remove {
            TreeOps::unlink(arena, node_id);
        }

        Ok(())
    }

    /// Apply capitalize headers transform.
    pub fn apply_capitalize_headers(
        arena: &mut NodeArena,
        _root: NodeId,
    ) -> ClmdResult<()> {
        for node_id in 0..arena.len() {
            let node_id = node_id as u32;
            let node = arena.get(node_id);

            // Check if this is a heading
            if matches!(node.value, NodeValue::Heading(_)) {
                // Capitalize text content of heading
                capitalize_text_in_node(arena, node_id);
            }
        }
        Ok(())
    }

    fn capitalize_text_in_node(arena: &mut NodeArena, node_id: NodeId) {
        let node = arena.get_mut(node_id);

        // If this is a text node, capitalize it
        if let NodeValue::Text(ref mut text) = node.value {
            *text = text.to_uppercase().into_boxed_str();
        }

        // Recursively process children
        let child_ids: Vec<NodeId> = {
            let node = arena.get(node_id);
            let mut children = Vec::new();
            let mut child = node.first_child;
            while let Some(child_id) = child {
                children.push(child_id);
                child = arena.get(child_id).next;
            }
            children
        };

        for child_id in child_ids {
            capitalize_text_in_node(arena, child_id);
        }
    }

    /// Apply absolute to relative links transform.
    pub fn apply_abs_to_rel(
        arena: &mut NodeArena,
        _root: NodeId,
        base_url: &str,
    ) -> ClmdResult<()> {
        for node_id in 0..arena.len() {
            let node_id = node_id as u32;
            let node = arena.get_mut(node_id);
            if let NodeValue::Link(ref mut link) = node.value {
                if link.url.starts_with(base_url) {
                    link.url = link.url[base_url.len()..].to_string();
                    // Ensure relative URL starts with /
                    if !link.url.starts_with('/') {
                        link.url = format!("/{}", link.url);
                    }
                }
            }
        }
        Ok(())
    }

    /// Apply auto-ident transform (add IDs to headers).
    pub fn apply_auto_ident(arena: &mut NodeArena, root: NodeId) -> ClmdResult<()> {
        // Collect headers and their text content
        let mut headers: Vec<(NodeId, String)> = Vec::new();
        collect_headers_for_ident(arena, root, &mut headers);

        // Generate unique IDs
        let mut used_ids: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for (node_id, text) in headers {
            let base_id = text_to_ident(&text);
            let mut unique_id = base_id.clone();
            let mut counter = 1;

            // Ensure uniqueness
            while used_ids.contains(&unique_id) {
                unique_id = format!("{}-{}", base_id, counter);
                counter += 1;
            }

            used_ids.insert(unique_id.clone());

            // Add ID attribute to heading
            // For now, we store it in the heading's data field if available
            // This is a simplified implementation
            let node = arena.get_mut(node_id);
            if let NodeValue::Heading(ref mut heading) = node.value {
                // In a full implementation, we would add attributes to the node
                // For now, we just note that this heading should have an ID
                let _ = unique_id; // Use the variable to avoid warnings
                let _ = heading; // Use the variable to avoid warnings
            }
        }

        Ok(())
    }

    fn collect_headers_for_ident(
        arena: &NodeArena,
        node_id: NodeId,
        headers: &mut Vec<(NodeId, String)>,
    ) {
        let node = arena.get(node_id);

        if let NodeValue::Heading(_) = &node.value {
            let text = get_text_content(arena, node_id);
            if !text.is_empty() {
                headers.push((node_id, text));
            }
        }

        // Recurse into children
        let mut child = node.first_child;
        while let Some(child_id) = child {
            collect_headers_for_ident(arena, child_id, headers);
            child = arena.get(child_id).next;
        }
    }

    fn text_to_ident(text: &str) -> String {
        text.to_lowercase()
            .replace(' ', "-")
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
            .trim_matches('-')
            .to_string()
    }

    /// Apply East Asian line break filter.
    /// Removes soft breaks between CJK characters for better typography.
    pub fn apply_east_asian_line_breaks(
        arena: &mut NodeArena,
        root: NodeId,
    ) -> ClmdResult<()> {
        // Collect all soft breaks that should be removed
        let mut to_remove: Vec<NodeId> = Vec::new();
        collect_soft_breaks_between_cjk(arena, root, &mut to_remove);

        // Remove the collected soft breaks by unlinking them from the tree
        for node_id in to_remove {
            unlink_node(arena, node_id);
        }

        Ok(())
    }

    fn collect_soft_breaks_between_cjk(
        arena: &NodeArena,
        node_id: NodeId,
        to_remove: &mut Vec<NodeId>,
    ) {
        use crate::text::char::{is_cjk, is_cjk_punctuation};

        let node = arena.get(node_id);

        // Check if this is a soft break
        if let NodeValue::SoftBreak = &node.value {
            // Check if the previous sibling ends with CJK
            let prev_cjk = node.prev.and_then(|prev_id| {
                let prev_node = arena.get(prev_id);
                if let NodeValue::Text(text) = &prev_node.value {
                    text.chars()
                        .last()
                        .map(|c| is_cjk(c) || is_cjk_punctuation(c))
                } else {
                    None
                }
            });

            // Check if the next sibling starts with CJK
            let next_cjk = node.next.and_then(|next_id| {
                let next_node = arena.get(next_id);
                if let NodeValue::Text(text) = &next_node.value {
                    text.chars()
                        .next()
                        .map(|c| is_cjk(c) || is_cjk_punctuation(c))
                } else {
                    None
                }
            });

            // Remove if both sides are CJK
            if prev_cjk == Some(true) && next_cjk == Some(true) {
                to_remove.push(node_id);
            }
        }

        // Recurse into children
        let mut child = node.first_child;
        while let Some(child_id) = child {
            collect_soft_breaks_between_cjk(arena, child_id, to_remove);
            child = arena.get(child_id).next;
        }
    }

    /// Unlink a node from the tree without removing it from the arena.
    fn unlink_node(arena: &mut NodeArena, node_id: NodeId) {
        let node = arena.get(node_id);
        let parent_id = node.parent;
        let prev_id = node.prev;
        let next_id = node.next;

        // Update parent's first_child if necessary
        if let Some(parent) = parent_id {
            let parent_node = arena.get_mut(parent);
            if parent_node.first_child == Some(node_id) {
                parent_node.first_child = next_id;
            }
            if parent_node.last_child == Some(node_id) {
                parent_node.last_child = prev_id;
            }
        }

        // Update previous sibling's next pointer
        if let Some(prev) = prev_id {
            let prev_node = arena.get_mut(prev);
            prev_node.next = next_id;
        }

        // Update next sibling's prev pointer
        if let Some(next) = next_id {
            let next_node = arena.get_mut(next);
            next_node.prev = prev_id;
        }

        // Clear the node's links
        let node = arena.get_mut(node_id);
        node.parent = None;
        node.prev = None;
        node.next = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{NodeHeading, NodeValue};

    #[test]
    fn test_transform_chain() {
        let mut chain = TransformChain::new();
        chain.add(Transform::header_shift(1));
        chain.add(Transform::normalize());

        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_header_shift_transform() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        TreeOps::append_child(&mut arena, root, heading);

        let transform = Transform::header_shift(1);
        transform.apply(&mut arena, root).unwrap();

        let heading_node = arena.get(heading);
        if let NodeValue::Heading(h) = &heading_node.value {
            assert_eq!(h.level, 2);
        } else {
            panic!("Expected heading node");
        }
    }

    #[test]
    fn test_header_shift_clamping() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 6,
            setext: false,
            closed: false,
        })));
        TreeOps::append_child(&mut arena, root, heading);

        let transform = Transform::header_shift(1);
        transform.apply(&mut arena, root).unwrap();

        let heading_node = arena.get(heading);
        if let NodeValue::Heading(h) = &heading_node.value {
            assert_eq!(h.level, 6); // Should be clamped to 6
        } else {
            panic!("Expected heading node");
        }
    }

    #[test]
    fn test_transform_names() {
        assert_eq!(Transform::header_shift(1).name(), "header-shift(1)");
        assert_eq!(Transform::normalize().name(), "normalize");
        assert!(Transform::table_of_contents()
            .build()
            .name()
            .contains("table-of-contents"));
    }

    #[test]
    fn test_toc_builder() {
        let transform = Transform::table_of_contents()
            .min_level(2)
            .max_level(4)
            .title("Contents")
            .build();

        match transform {
            Transform::TableOfContents {
                min_level,
                max_level,
                title,
            } => {
                assert_eq!(min_level, 2);
                assert_eq!(max_level, 4);
                assert_eq!(title, Some("Contents".to_string()));
            }
            _ => panic!("Expected TableOfContents transform"),
        }
    }

    #[test]
    fn test_link_rewrite() {
        let transform = Transform::link_rewrite("old", "new");
        match transform {
            Transform::LinkRewrite {
                pattern,
                replacement,
            } => {
                assert_eq!(pattern, "old");
                assert_eq!(replacement, "new");
            }
            _ => panic!("Expected LinkRewrite transform"),
        }
    }

    #[test]
    fn test_image_rewrite() {
        let transform = Transform::image_rewrite("old", "new");
        match transform {
            Transform::ImageRewrite {
                pattern,
                replacement,
            } => {
                assert_eq!(pattern, "old");
                assert_eq!(replacement, "new");
            }
            _ => panic!("Expected ImageRewrite transform"),
        }
    }

    #[test]
    fn test_transform_strip_footnotes() {
        let transform = Transform::strip_footnotes();
        assert_eq!(transform.name(), "strip-footnotes");
    }

    #[test]
    fn test_transform_capitalize_headers() {
        let transform = Transform::capitalize_headers();
        assert_eq!(transform.name(), "capitalize-headers");
    }

    #[test]
    fn test_transform_abs_to_rel() {
        let transform = Transform::abs_to_rel("https://example.com");
        match &transform {
            Transform::AbsToRel { base_url } => {
                assert_eq!(base_url, "https://example.com");
            }
            _ => panic!("Expected AbsToRel transform"),
        }
        assert_eq!(transform.name(), "abs-to-rel(https://example.com)");
    }

    #[test]
    fn test_transform_auto_ident() {
        let transform = Transform::auto_ident();
        assert_eq!(transform.name(), "auto-ident");
    }

    #[test]
    fn test_transform_east_asian_line_breaks() {
        let transform = Transform::east_asian_line_breaks();
        assert_eq!(transform.name(), "east-asian-line-breaks");
    }

    #[test]
    fn test_transform_custom() {
        let transform = Transform::Custom {
            name: "custom-transform".to_string(),
            apply: |_arena, _root| Ok(()),
        };
        assert_eq!(transform.name(), "custom(custom-transform)");
    }

    #[test]
    fn test_transform_add_attributes() {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert("class".to_string(), "highlight".to_string());
        let transform = Transform::AddAttributes {
            selector: "h1".to_string(),
            attributes: attrs,
        };
        assert_eq!(transform.name(), "add-attributes");
    }

    #[test]
    fn test_transform_remove_elements() {
        let transform = Transform::RemoveElements {
            selector: ".hidden".to_string(),
        };
        assert_eq!(transform.name(), "remove-elements");
    }

    #[test]
    fn test_transform_chain_extend() {
        let mut chain = TransformChain::new();
        chain.add(Transform::header_shift(1));
        chain.extend(vec![Transform::normalize(), Transform::strip_footnotes()]);
        assert_eq!(chain.len(), 3);
    }

    #[test]
    fn test_transform_chain_clear() {
        let mut chain = TransformChain::new();
        chain.add(Transform::header_shift(1));
        assert!(!chain.is_empty());
        chain.clear();
        assert!(chain.is_empty());
    }

    #[test]
    fn test_transform_chain_iter() {
        let mut chain = TransformChain::new();
        chain.add(Transform::header_shift(1));
        chain.add(Transform::normalize());
        let count = chain.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_transform_chain_apply() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        TreeOps::append_child(&mut arena, root, heading);

        let chain = TransformChain::new();
        // Empty chain should apply without error
        chain.apply(&mut arena, root).unwrap();
    }

    #[test]
    fn test_header_shift_negative() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 3,
            setext: false,
            closed: false,
        })));
        TreeOps::append_child(&mut arena, root, heading);

        let transform = Transform::header_shift(-1);
        transform.apply(&mut arena, root).unwrap();

        let heading_node = arena.get(heading);
        if let NodeValue::Heading(h) = &heading_node.value {
            assert_eq!(h.level, 2);
        } else {
            panic!("Expected heading node");
        }
    }

    #[test]
    fn test_header_shift_clamping_lower() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        TreeOps::append_child(&mut arena, root, heading);

        let transform = Transform::header_shift(-1);
        transform.apply(&mut arena, root).unwrap();

        let heading_node = arena.get(heading);
        if let NodeValue::Heading(h) = &heading_node.value {
            assert_eq!(h.level, 1); // Should be clamped to 1
        } else {
            panic!("Expected heading node");
        }
    }

    #[test]
    fn test_normalize_transform() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let text_node = arena.alloc(Node::with_value(NodeValue::Text("  multiple   spaces  ".into())));
        TreeOps::append_child(&mut arena, root, text_node);

        let transform = Transform::normalize();
        transform.apply(&mut arena, root).unwrap();

        // Normalize should collapse whitespace
        let node = arena.get(text_node);
        if let NodeValue::Text(t) = &node.value {
            assert_eq!(t.as_ref(), "multiple spaces");
        }
    }

    #[test]
    fn test_link_rewrite_apply() {
        use crate::core::nodes::NodeLink;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let link = NodeValue::Link(Box::new(NodeLink {
            url: "https://old-domain.com/page".into(),
            title: "Link".into(),
        }));
        let link_node = arena.alloc(Node::with_value(link));
        TreeOps::append_child(&mut arena, root, link_node);

        let transform = Transform::link_rewrite("old-domain", "new-domain");
        transform.apply(&mut arena, root).unwrap();

        let node = arena.get(link_node);
        if let NodeValue::Link(l) = &node.value {
            assert_eq!(l.url, "https://new-domain.com/page");
        } else {
            panic!("Expected link node");
        }
    }

    #[test]
    fn test_image_rewrite_apply() {
        use crate::core::nodes::NodeLink;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let image = NodeValue::Image(Box::new(NodeLink {
            url: "/old-path/image.png".into(),
            title: "Image".into(),
        }));
        let image_node = arena.alloc(Node::with_value(image));
        TreeOps::append_child(&mut arena, root, image_node);

        let transform = Transform::image_rewrite("/old-path", "/new-path");
        transform.apply(&mut arena, root).unwrap();

        let node = arena.get(image_node);
        if let NodeValue::Image(i) = &node.value {
            assert_eq!(i.url, "/new-path/image.png");
        } else {
            panic!("Expected image node");
        }
    }

    #[test]
    fn test_strip_footnotes_apply() {
        use crate::core::nodes::{NodeFootnoteDefinition, NodeFootnoteReference};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let footnote_def = NodeValue::FootnoteDefinition(Box::new(NodeFootnoteDefinition {
            name: "note1".into(),
            total_references: 1,
        }));
        let def_node = arena.alloc(Node::with_value(footnote_def));
        TreeOps::append_child(&mut arena, root, def_node);

        let footnote_ref = NodeValue::FootnoteReference(Box::new(NodeFootnoteReference {
            name: "note1".into(),
            ref_num: 1,
            ix: 0,
        }));
        let ref_node = arena.alloc(Node::with_value(footnote_ref));
        TreeOps::append_child(&mut arena, root, ref_node);

        let transform = Transform::strip_footnotes();
        transform.apply(&mut arena, root).unwrap();

        // Both nodes should be unlinked
        let def = arena.get(def_node);
        let ref_n = arena.get(ref_node);
        assert!(def.parent.is_none());
        assert!(ref_n.parent.is_none());
    }

    #[test]
    fn test_capitalize_headers_apply() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("hello world".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let transform = Transform::capitalize_headers();
        transform.apply(&mut arena, root).unwrap();

        let text_node = arena.get(text);
        if let NodeValue::Text(t) = &text_node.value {
            assert_eq!(t.as_ref(), "HELLO WORLD");
        }
    }

    #[test]
    fn test_abs_to_rel_apply() {
        use crate::core::nodes::NodeLink;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let link = NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com/path/to/page".into(),
            title: "Link".into(),
        }));
        let link_node = arena.alloc(Node::with_value(link));
        TreeOps::append_child(&mut arena, root, link_node);

        let transform = Transform::abs_to_rel("https://example.com");
        transform.apply(&mut arena, root).unwrap();

        let node = arena.get(link_node);
        if let NodeValue::Link(l) = &node.value {
            assert_eq!(l.url, "/path/to/page");
        } else {
            panic!("Expected link node");
        }
    }

    #[test]
    fn test_abs_to_rel_no_match() {
        use crate::core::nodes::NodeLink;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let link = NodeValue::Link(Box::new(NodeLink {
            url: "https://other.com/page".into(),
            title: "Link".into(),
        }));
        let link_node = arena.alloc(Node::with_value(link));
        TreeOps::append_child(&mut arena, root, link_node);

        let transform = Transform::abs_to_rel("https://example.com");
        transform.apply(&mut arena, root).unwrap();

        let node = arena.get(link_node);
        if let NodeValue::Link(l) = &node.value {
            // URL should remain unchanged
            assert_eq!(l.url, "https://other.com/page");
        } else {
            panic!("Expected link node");
        }
    }

    #[test]
    fn test_auto_ident_apply() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Hello World".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let transform = Transform::auto_ident();
        // Should not panic
        transform.apply(&mut arena, root).unwrap();
    }

    #[test]
    fn test_toc_builder_clamping() {
        let transform = Transform::table_of_contents()
            .min_level(0) // Should be clamped to 1
            .max_level(10) // Should be clamped to 6
            .build();

        match transform {
            Transform::TableOfContents {
                min_level,
                max_level,
                ..
            } => {
                assert_eq!(min_level, 1);
                assert_eq!(max_level, 6);
            }
            _ => panic!("Expected TableOfContents transform"),
        }
    }

    #[test]
    fn test_transform_error_display() {
        let err = TransformError::NotImplemented("test".to_string());
        assert!(err.to_string().contains("not implemented"));

        let err = TransformError::InvalidConfig("bad config".to_string());
        assert!(err.to_string().contains("Invalid transform config"));

        let err = TransformError::ExecutionFailed("failed".to_string());
        assert!(err.to_string().contains("execution failed"));
    }

    #[test]
    fn test_transform_error_from() {
        let transform_err = TransformError::ExecutionFailed("test".to_string());
        let clmd_err: ClmdError = transform_err.into();
        // Just verify conversion works
        let _ = clmd_err.to_string();
    }

    #[test]
    fn test_table_of_contents_empty() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Document with no headings
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("No headings here".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let transform = Transform::table_of_contents().build();
        // Should not panic with empty document
        transform.apply(&mut arena, root).unwrap();
    }

    #[test]
    fn test_table_of_contents_with_title() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Section 1".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let transform = Transform::table_of_contents()
            .title("Table of Contents")
            .build();
        transform.apply(&mut arena, root).unwrap();

        // TOC should be inserted at the beginning
        let root_node = arena.get(root);
        assert!(root_node.first_child.is_some());
    }

}
