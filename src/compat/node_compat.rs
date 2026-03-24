//! Node compatibility layer
//!
//! Provides bridges between old and new AST node systems.

use crate::ast::node::SourcePos as NewSourcePos;
use crate::node::{Node as OldNode, NodeType as OldNodeType, SourcePos as OldSourcePos};

/// Convert old SourcePos to new SourcePos
pub fn convert_source_pos(old: OldSourcePos) -> NewSourcePos {
    NewSourcePos {
        start_line: old.start_line,
        start_column: old.start_column,
        end_line: old.end_line,
        end_column: old.end_column,
    }
}

/// Convert new SourcePos to old SourcePos
pub fn convert_source_pos_back(new: NewSourcePos) -> OldSourcePos {
    OldSourcePos {
        start_line: new.start_line,
        start_column: new.start_column,
        end_line: new.end_line,
        end_column: new.end_column,
    }
}

/// Convert old NodeType to new node type name
pub fn convert_node_type(old: OldNodeType) -> &'static str {
    match old {
        OldNodeType::Document => "Document",
        OldNodeType::BlockQuote => "BlockQuote",
        OldNodeType::List => "List",
        OldNodeType::Item => "Item",
        OldNodeType::CodeBlock => "CodeBlock",
        OldNodeType::HtmlBlock => "HtmlBlock",
        OldNodeType::Paragraph => "Paragraph",
        OldNodeType::Heading => "Heading",
        OldNodeType::ThematicBreak => "ThematicBreak",
        OldNodeType::Text => "Text",
        OldNodeType::SoftBreak => "SoftBreak",
        OldNodeType::LineBreak => "LineBreak",
        OldNodeType::Code => "Code",
        OldNodeType::HtmlInline => "HtmlInline",
        OldNodeType::Emph => "Emph",
        OldNodeType::Strong => "Strong",
        OldNodeType::Link => "Link",
        OldNodeType::Image => "Image",
        OldNodeType::CustomBlock => "CustomBlock",
        OldNodeType::CustomInline => "CustomInline",
        OldNodeType::Table => "Table",
        OldNodeType::TableHead => "TableHead",
        OldNodeType::TableRow => "TableRow",
        OldNodeType::TableCell => "TableCell",
        OldNodeType::Strikethrough => "Strikethrough",
        OldNodeType::TaskItem => "TaskItem",
        OldNodeType::FootnoteRef => "FootnoteRef",
        OldNodeType::FootnoteDef => "FootnoteDef",
        OldNodeType::None => "None",
    }
}

/// Extension trait for old nodes to provide new-style operations
pub trait NodeCompatExt {
    /// Get node type name
    fn node_type_name(&self) -> &'static str;

    /// Check if block
    fn is_block(&self) -> bool;

    /// Check if inline
    fn is_inline(&self) -> bool;

    /// Check if leaf
    fn is_leaf(&self) -> bool;
}

impl NodeCompatExt for OldNode {
    fn node_type_name(&self) -> &'static str {
        convert_node_type(self.node_type)
    }

    fn is_block(&self) -> bool {
        matches!(
            self.node_type,
            OldNodeType::Document
                | OldNodeType::BlockQuote
                | OldNodeType::List
                | OldNodeType::Item
                | OldNodeType::CodeBlock
                | OldNodeType::HtmlBlock
                | OldNodeType::Paragraph
                | OldNodeType::Heading
                | OldNodeType::ThematicBreak
        )
    }

    fn is_inline(&self) -> bool {
        !self.is_block()
    }

    fn is_leaf(&self) -> bool {
        matches!(
            self.node_type,
            OldNodeType::Text
                | OldNodeType::SoftBreak
                | OldNodeType::LineBreak
                | OldNodeType::Code
                | OldNodeType::HtmlInline
                | OldNodeType::ThematicBreak
                | OldNodeType::CodeBlock
                | OldNodeType::HtmlBlock
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_pos_conversion() {
        let old = OldSourcePos {
            start_line: 1,
            start_column: 2,
            end_line: 3,
            end_column: 4,
        };
        let new = convert_source_pos(old);
        assert_eq!(new.start_line, 1);
        assert_eq!(new.start_column, 2);
        assert_eq!(new.end_line, 3);
        assert_eq!(new.end_column, 4);

        let back = convert_source_pos_back(new);
        assert_eq!(back.start_line, 1);
        assert_eq!(back.start_column, 2);
        assert_eq!(back.end_line, 3);
        assert_eq!(back.end_column, 4);
    }

    #[test]
    fn test_convert_node_type() {
        assert_eq!(convert_node_type(OldNodeType::Document), "Document");
        assert_eq!(convert_node_type(OldNodeType::Paragraph), "Paragraph");
        assert_eq!(convert_node_type(OldNodeType::Text), "Text");
        assert_eq!(convert_node_type(OldNodeType::Link), "Link");
    }

    #[test]
    fn test_node_compat_ext() {
        let doc = OldNode::new(OldNodeType::Document);
        assert!(doc.is_block());
        assert!(!doc.is_inline());
        assert!(!doc.is_leaf());
        assert_eq!(doc.node_type_name(), "Document");

        let text = OldNode::new(OldNodeType::Text);
        assert!(!text.is_block());
        assert!(text.is_inline());
        assert!(text.is_leaf());
        assert_eq!(text.node_type_name(), "Text");
    }
}
