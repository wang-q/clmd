//! AST Node extensions
//!
//! Provides extension traits and utility methods for AST nodes.

use crate::ast_nodes::traits::*;

/// Extension trait for block nodes
pub trait BlockNodeExt: BlockNode {
    /// Check if this block can contain inline content
    fn can_contain_inline(&self) -> bool {
        matches!(
            self.node_type_name(),
            "Paragraph" | "Heading" | "BlockQuote" | "Item"
        )
    }

    /// Check if this block can contain other blocks
    fn can_contain_blocks(&self) -> bool {
        matches!(
            self.node_type_name(),
            "Document" | "BlockQuote" | "List" | "Item"
        )
    }

    /// Get the heading level if this is a heading
    fn heading_level(&self) -> Option<u32> {
        None
    }
}

impl BlockNodeExt for Document {}
impl BlockNodeExt for BlockQuote {}
impl BlockNodeExt for List {}
impl BlockNodeExt for Item {}
impl BlockNodeExt for CodeBlock {}
impl BlockNodeExt for HtmlBlock {}
impl BlockNodeExt for Paragraph {}

impl BlockNodeExt for Heading {
    fn heading_level(&self) -> Option<u32> {
        Some(self.level)
    }
}

impl BlockNodeExt for ThematicBreak {}

/// Extension trait for inline nodes
pub trait InlineNodeExt: InlineNode {
    /// Check if this inline can contain other inlines
    fn can_contain_inlines(&self) -> bool {
        matches!(self.node_type_name(), "Emph" | "Strong" | "Link" | "Image")
    }

    /// Get the literal content if this is a text-like node
    fn literal_content(&self) -> Option<&str> {
        None
    }

    /// Get the URL if this is a link or image
    fn url(&self) -> Option<&str> {
        None
    }

    /// Get the title if this is a link or image
    fn title(&self) -> Option<&str> {
        None
    }
}

impl InlineNodeExt for Text {
    fn literal_content(&self) -> Option<&str> {
        Some(&self.literal)
    }
}

impl InlineNodeExt for SoftBreak {}
impl InlineNodeExt for LineBreak {}

impl InlineNodeExt for Code {
    fn literal_content(&self) -> Option<&str> {
        Some(&self.literal)
    }
}

impl InlineNodeExt for HtmlInline {
    fn literal_content(&self) -> Option<&str> {
        Some(&self.literal)
    }
}

impl InlineNodeExt for Emph {}
impl InlineNodeExt for Strong {}

impl InlineNodeExt for Link {
    fn url(&self) -> Option<&str> {
        Some(&self.url)
    }

    fn title(&self) -> Option<&str> {
        Some(&self.title)
    }
}

impl InlineNodeExt for Image {
    fn url(&self) -> Option<&str> {
        Some(&self.url)
    }

    fn title(&self) -> Option<&str> {
        Some(&self.title)
    }
}

/// Extension trait for list nodes
pub trait ListExt {
    /// Check if this is an ordered list
    fn is_ordered(&self) -> bool;

    /// Check if this is a bullet list
    fn is_bullet(&self) -> bool;

    /// Get the list marker character
    fn marker(&self) -> char;
}

impl ListExt for List {
    fn is_ordered(&self) -> bool {
        matches!(self.list_type, ListType::Ordered)
    }

    fn is_bullet(&self) -> bool {
        matches!(self.list_type, ListType::Bullet)
    }

    fn marker(&self) -> char {
        match self.list_type {
            ListType::Bullet => self.bullet_char,
            ListType::Ordered => match self.delim {
                DelimType::Period => '.',
                DelimType::Paren => ')',
                _ => '.',
            },
            _ => '-',
        }
    }
}

/// Factory functions for creating nodes
pub mod factory {
    use super::*;

    /// Create a document node
    pub fn document() -> Document {
        Document
    }

    /// Create a paragraph node
    pub fn paragraph() -> Paragraph {
        Paragraph
    }

    /// Create a heading node
    pub fn heading(level: u32) -> Heading {
        Heading { level }
    }

    /// Create a text node
    pub fn text(literal: impl Into<String>) -> Text {
        Text {
            literal: literal.into(),
        }
    }

    /// Create a code node
    pub fn code(literal: impl Into<String>) -> Code {
        Code {
            literal: literal.into(),
        }
    }

    /// Create a code block node
    pub fn code_block(info: impl Into<String>, literal: impl Into<String>) -> CodeBlock {
        CodeBlock {
            info: info.into(),
            literal: literal.into(),
        }
    }

    /// Create a link node
    pub fn link(url: impl Into<String>, title: impl Into<String>) -> Link {
        Link {
            url: url.into(),
            title: title.into(),
        }
    }

    /// Create an image node
    pub fn image(url: impl Into<String>, title: impl Into<String>) -> Image {
        Image {
            url: url.into(),
            title: title.into(),
        }
    }

    /// Create a bullet list
    pub fn bullet_list(tight: bool, bullet_char: char) -> List {
        List {
            list_type: ListType::Bullet,
            delim: DelimType::None,
            start: 0,
            tight,
            bullet_char,
        }
    }

    /// Create an ordered list
    pub fn ordered_list(start: u32, tight: bool, delim: DelimType) -> List {
        List {
            list_type: ListType::Ordered,
            delim,
            start,
            tight,
            bullet_char: '\0',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::factory::*;
    use super::*;

    #[test]
    fn test_block_node_ext() {
        let doc = document();
        assert!(doc.can_contain_blocks());
        assert!(!doc.can_contain_inline());
        assert_eq!(doc.heading_level(), None);

        let para = paragraph();
        assert!(!para.can_contain_blocks());
        assert!(para.can_contain_inline());

        let h2 = heading(2);
        assert_eq!(h2.heading_level(), Some(2));
        assert!(h2.can_contain_inline());
    }

    #[test]
    fn test_inline_node_ext() {
        let t = text("hello");
        assert_eq!(t.literal_content(), Some("hello"));
        assert!(!t.can_contain_inlines());

        let code_node = code("fn main()");
        assert_eq!(code_node.literal_content(), Some("fn main()"));

        let link_node = link("https://example.com", "Example");
        assert_eq!(link_node.url(), Some("https://example.com"));
        assert_eq!(link_node.title(), Some("Example"));
        assert!(link_node.can_contain_inlines());

        let emph = Emph;
        assert!(emph.can_contain_inlines());
    }

    #[test]
    fn test_list_ext() {
        let bullet = bullet_list(true, '-');
        assert!(bullet.is_bullet());
        assert!(!bullet.is_ordered());
        assert_eq!(bullet.marker(), '-');

        let ordered = ordered_list(1, false, DelimType::Period);
        assert!(!ordered.is_bullet());
        assert!(ordered.is_ordered());
        assert_eq!(ordered.marker(), '.');
    }

    #[test]
    fn test_factory_functions() {
        let doc = document();
        assert_eq!(doc.node_type_name(), "Document");

        let h1 = heading(1);
        assert_eq!(h1.level, 1);

        let t = text("Hello, World!");
        assert_eq!(t.literal, "Hello, World!");

        let cb = code_block("rust", "fn main() {}");
        assert_eq!(cb.info, "rust");
        assert_eq!(cb.literal, "fn main() {}");
    }
}
