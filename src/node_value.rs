//! Unified AST node value types for CommonMark documents
//!
//! This module provides a unified `NodeValue` enum that combines node type and data,
//! inspired by comrak's design. This approach provides better type safety and
//! more ergonomic API compared to the separate `NodeType` and `NodeData` approach.
//!
//! # Example
//!
//! ```
//! use clmd::node_value::{NodeValue, NodeHeading, NodeList, ListType, ListDelimType};
//!
//! // Create a heading node value
//! let heading = NodeValue::Heading(NodeHeading {
//!     level: 1,
//!     setext: false,
//!     closed: false,
//! });
//!
//! // Create a list node value with metadata
//! let list = NodeValue::List(NodeList {
//!     list_type: ListType::Bullet,
//!     marker_offset: 0,
//!     padding: 2,
//!     start: 1,
//!     delimiter: ListDelimType::Period,
//!     bullet_char: b'-',
//!     tight: true,
//!     is_task_list: false,
//! });
//! ```

/// The core AST node value enum.
///
/// This enum combines the node type and its associated data into a single type,
/// providing better type safety and ergonomics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeValue {
    /// The root of every CommonMark document. Contains blocks.
    Document,

    /// A block quote. Contains other blocks.
    ///
    /// ```markdown
    /// > A block quote.
    /// ```
    BlockQuote,

    /// A list. Contains list items.
    ///
    /// ```markdown
    /// * An unordered list
    /// * Another item
    ///
    /// 1. An ordered list
    /// 2. Another item
    /// ```
    List(NodeList),

    /// A list item. Contains other blocks.
    Item(NodeList),

    /// A description list (definition list).
    ///
    /// ```markdown
    /// Term
    /// : Definition
    /// ```
    DescriptionList,

    /// An item in a description list.
    DescriptionItem(NodeDescriptionItem),

    /// The term in a description list item.
    DescriptionTerm,

    /// The definition/details in a description list item.
    DescriptionDetails,

    /// A code block (fenced or indented).
    ///
    /// ```markdown
    /// ```rust
    /// fn main() {}
    /// ```
    /// ```
    CodeBlock(NodeCodeBlock),

    /// An HTML block.
    ///
    /// ```markdown
    /// <div>
    /// Some HTML content
    /// </div>
    /// ```
    HtmlBlock(NodeHtmlBlock),

    /// A paragraph. Contains inlines.
    ///
    /// ```markdown
    /// This is a paragraph.
    /// ```
    Paragraph,

    /// A heading (ATX or setext).
    ///
    /// ```markdown
    /// # ATX Heading
    ///
    /// Setext Heading
    /// ==============
    /// ```
    Heading(NodeHeading),

    /// A thematic break (horizontal rule).
    ///
    /// ```markdown
    /// ---
    /// ```
    ThematicBreak,

    /// A footnote definition.
    ///
    /// ```markdown
    /// [^1]: This is a footnote.
    /// ```
    FootnoteDefinition(NodeFootnoteDefinition),

    /// A table (GFM extension).
    ///
    /// ```markdown
    /// | Header | Header |
    /// |--------|--------|
    /// | Cell   | Cell   |
    /// ```
    Table(NodeTable),

    /// A table row.
    TableRow(bool), // bool indicates if this is the header row

    /// A table cell.
    TableCell,

    /// Textual content.
    Text(String),

    /// A task list item (GFM extension).
    ///
    /// ```markdown
    /// - [x] Completed task
    /// - [ ] Incomplete task
    /// ```
    TaskItem(NodeTaskItem),

    /// A soft line break.
    SoftBreak,

    /// A hard line break.
    ///
    /// ```markdown
    /// Hard break··
    /// line\
    /// breaks
    /// ```
    /// *`·` is a space*
    HardBreak,

    /// An inline code span.
    ///
    /// ```markdown
    /// Use `code` here.
    /// ```
    Code(NodeCode),

    /// Raw HTML inline.
    ///
    /// ```markdown
    /// Some <em>HTML</em> content.
    /// ```
    HtmlInline(String),

    /// Emphasized text.
    ///
    /// ```markdown
    /// *emphasized* or _emphasized_
    /// ```
    Emph,

    /// Strongly emphasized text.
    ///
    /// ```markdown
    /// **strong** or __strong__
    /// ```
    Strong,

    /// Strikethrough text (GFM extension).
    ///
    /// ```markdown
    /// ~~deleted~~
    /// ```
    Strikethrough,

    /// Highlighted/marked text.
    ///
    /// ```markdown
    /// ==highlighted==
    /// ```
    Highlight,

    /// Inserted text.
    ///
    /// ```markdown
    /// ++inserted++
    /// ```
    Insert,

    /// Superscript text.
    ///
    /// ```markdown
    /// x^2^
    /// ```
    Superscript,

    /// Subscript text.
    ///
    /// ```markdown
    /// H~2~O
    /// ```
    Subscript,

    /// A link.
    ///
    /// ```markdown
    /// [link text](https://example.com "title")
    /// ```
    Link(NodeLink),

    /// An image.
    ///
    /// ```markdown
    /// ![alt text](image.png "title")
    /// ```
    Image(NodeLink),

    /// A footnote reference.
    ///
    /// ```markdown
    /// This needs a footnote[^1].
    /// ```
    FootnoteReference(NodeFootnoteReference),

    /// A math span.
    ///
    /// ```markdown
    /// Inline math: $1 + 2$
    /// Display math: $$x + y$$
    /// ```
    Math(NodeMath),

    /// A wikilink.
    ///
    /// ```markdown
    /// [[Page Title]] or [[Page Title|Display Text]]
    /// ```
    WikiLink(NodeWikiLink),

    /// Underlined text.
    ///
    /// ```markdown
    /// __underlined__
    /// ```
    Underline,

    /// Spoiler text.
    ///
    /// ```markdown
    /// ||spoiler||
    /// ```
    Spoiler,

    /// An escaped character.
    Escaped(char),

    /// A multiline block quote.
    ///
    /// ```markdown
    /// >>>
    /// Content
    /// >>>
    /// ```
    MultilineBlockQuote(NodeMultilineBlockQuote),

    /// An alert (GFM extension).
    ///
    /// ```markdown
    /// > [!NOTE]
    /// > This is a note.
    /// ```
    Alert(NodeAlert),

    /// Subtext (block-scoped subscript).
    Subtext,

    /// Front matter.
    ///
    /// ```markdown
    /// ---
    /// title: My Document
    /// ---
    /// ```
    FrontMatter(String),

    /// Raw output (not parsed, inserted verbatim).
    Raw(String),
}

impl NodeValue {
    /// Check if this node value represents a block element.
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            NodeValue::Document
                | NodeValue::BlockQuote
                | NodeValue::FootnoteDefinition(_)
                | NodeValue::List(..)
                | NodeValue::DescriptionList
                | NodeValue::DescriptionItem(_)
                | NodeValue::DescriptionTerm
                | NodeValue::DescriptionDetails
                | NodeValue::Item(..)
                | NodeValue::CodeBlock(..)
                | NodeValue::HtmlBlock(..)
                | NodeValue::Paragraph
                | NodeValue::Heading(..)
                | NodeValue::ThematicBreak
                | NodeValue::Table(..)
                | NodeValue::TableRow(..)
                | NodeValue::TableCell
                | NodeValue::TaskItem(..)
                | NodeValue::MultilineBlockQuote(_)
                | NodeValue::Alert(_)
                | NodeValue::Subtext
                | NodeValue::FrontMatter(_)
        )
    }

    /// Check if this node value represents an inline element.
    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            NodeValue::Text(..)
                | NodeValue::SoftBreak
                | NodeValue::HardBreak
                | NodeValue::Code(..)
                | NodeValue::HtmlInline(..)
                | NodeValue::Emph
                | NodeValue::Strong
                | NodeValue::Strikethrough
                | NodeValue::Highlight
                | NodeValue::Insert
                | NodeValue::Superscript
                | NodeValue::Subscript
                | NodeValue::Link(..)
                | NodeValue::Image(..)
                | NodeValue::FootnoteReference(..)
                | NodeValue::Math(..)
                | NodeValue::WikiLink(..)
                | NodeValue::Underline
                | NodeValue::Spoiler
                | NodeValue::Escaped(..)
        )
    }

    /// Check if this node value can contain inline elements.
    pub fn contains_inlines(&self) -> bool {
        matches!(
            self,
            NodeValue::Paragraph
                | NodeValue::Heading(..)
                | NodeValue::TableCell
                | NodeValue::Subtext
        )
    }

    /// Check if this is a leaf node (cannot have children).
    pub fn is_leaf(&self) -> bool {
        matches!(
            self,
            NodeValue::Text(..)
                | NodeValue::SoftBreak
                | NodeValue::HardBreak
                | NodeValue::Code(..)
                | NodeValue::HtmlInline(..)
                | NodeValue::CodeBlock(..)
                | NodeValue::HtmlBlock(..)
                | NodeValue::ThematicBreak
                | NodeValue::Escaped(..)
                | NodeValue::Raw(..)
                | NodeValue::FrontMatter(..)
        )
    }

    /// Get the text content if this is a text node.
    pub fn text(&self) -> Option<&str> {
        match self {
            NodeValue::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Get the mutable text content if this is a text node.
    pub fn text_mut(&mut self) -> Option<&mut String> {
        match self {
            NodeValue::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Returns the XML node name for this value.
    ///
    /// This follows the CommonMark DTD for standard nodes.
    pub fn xml_node_name(&self) -> &'static str {
        match self {
            NodeValue::Document => "document",
            NodeValue::BlockQuote => "block_quote",
            NodeValue::FootnoteDefinition(_) => "footnote_definition",
            NodeValue::List(..) => "list",
            NodeValue::DescriptionList => "description_list",
            NodeValue::DescriptionItem(_) => "description_item",
            NodeValue::DescriptionTerm => "description_term",
            NodeValue::DescriptionDetails => "description_details",
            NodeValue::Item(..) => "item",
            NodeValue::CodeBlock(..) => "code_block",
            NodeValue::HtmlBlock(..) => "html_block",
            NodeValue::Paragraph => "paragraph",
            NodeValue::Heading(..) => "heading",
            NodeValue::ThematicBreak => "thematic_break",
            NodeValue::Table(..) => "table",
            NodeValue::TableRow(..) => "table_row",
            NodeValue::TableCell => "table_cell",
            NodeValue::Text(..) => "text",
            NodeValue::SoftBreak => "softbreak",
            NodeValue::HardBreak => "linebreak",
            NodeValue::Image(..) => "image",
            NodeValue::Link(..) => "link",
            NodeValue::Emph => "emph",
            NodeValue::Strong => "strong",
            NodeValue::Code(..) => "code",
            NodeValue::HtmlInline(..) => "html_inline",
            NodeValue::Raw(..) => "raw",
            NodeValue::Strikethrough => "strikethrough",
            NodeValue::Highlight => "highlight",
            NodeValue::Insert => "insert",
            NodeValue::FrontMatter(_) => "frontmatter",
            NodeValue::TaskItem { .. } => "taskitem",
            NodeValue::Superscript => "superscript",
            NodeValue::FootnoteReference(..) => "footnote_reference",
            NodeValue::MultilineBlockQuote(_) => "multiline_block_quote",
            NodeValue::Escaped(..) => "escaped",
            NodeValue::Math(..) => "math",
            NodeValue::WikiLink(..) => "wikilink",
            NodeValue::Underline => "underline",
            NodeValue::Subscript => "subscript",
            NodeValue::Spoiler => "spoiler",
            NodeValue::Alert(_) => "alert",
            NodeValue::Subtext => "subtext",
        }
    }

    /// Check if this node type can accept lines of text.
    #[allow(dead_code)]
    pub(crate) fn accepts_lines(&self) -> bool {
        matches!(
            self,
            NodeValue::Paragraph
                | NodeValue::Heading(..)
                | NodeValue::CodeBlock(..)
                | NodeValue::Subtext
        )
    }
}

// =============================================================================
// Compatibility conversions from node.rs types
// =============================================================================

use crate::node::{self, NodeData, NodeType};

impl From<NodeType> for NodeValue {
    fn from(node_type: NodeType) -> Self {
        match node_type {
            NodeType::Document => NodeValue::Document,
            NodeType::BlockQuote => NodeValue::BlockQuote,
            NodeType::List => NodeValue::List(NodeList::default()),
            NodeType::Item => NodeValue::Item(NodeList::default()),
            NodeType::CodeBlock => NodeValue::CodeBlock(NodeCodeBlock {
                fenced: false,
                fence_char: 0,
                fence_length: 0,
                fence_offset: 0,
                info: String::new(),
                literal: String::new(),
                closed: false,
            }),
            NodeType::HtmlBlock => NodeValue::HtmlBlock(NodeHtmlBlock {
                block_type: 0,
                literal: String::new(),
            }),
            NodeType::Paragraph => NodeValue::Paragraph,
            NodeType::Heading => NodeValue::Heading(NodeHeading {
                level: 1,
                setext: false,
                closed: false,
            }),
            NodeType::ThematicBreak => NodeValue::ThematicBreak,
            NodeType::Table => NodeValue::Table(NodeTable::default()),
            NodeType::TableHead => NodeValue::TableRow(true),
            NodeType::TableRow => NodeValue::TableRow(false),
            NodeType::TableCell => NodeValue::TableCell,
            NodeType::Text => NodeValue::Text(String::new()),
            NodeType::SoftBreak => NodeValue::SoftBreak,
            NodeType::LineBreak => NodeValue::HardBreak,
            NodeType::Code => NodeValue::Code(NodeCode::default()),
            NodeType::HtmlInline => NodeValue::HtmlInline(String::new()),
            NodeType::Emph => NodeValue::Emph,
            NodeType::Strong => NodeValue::Strong,
            NodeType::Link => NodeValue::Link(NodeLink {
                url: String::new(),
                title: String::new(),
            }),
            NodeType::Image => NodeValue::Image(NodeLink {
                url: String::new(),
                title: String::new(),
            }),
            NodeType::Strikethrough => NodeValue::Strikethrough,
            NodeType::TaskItem => NodeValue::TaskItem(NodeTaskItem::default()),
            NodeType::FootnoteRef => {
                NodeValue::FootnoteReference(NodeFootnoteReference::default())
            }
            NodeType::FootnoteDef => {
                NodeValue::FootnoteDefinition(NodeFootnoteDefinition::default())
            }
            NodeType::CustomBlock | NodeType::CustomInline | NodeType::None => {
                NodeValue::Raw(String::new())
            }
        }
    }
}

impl From<&NodeData> for NodeValue {
    fn from(data: &NodeData) -> Self {
        match data {
            NodeData::Document => NodeValue::Document,
            NodeData::BlockQuote => NodeValue::BlockQuote,
            NodeData::List {
                list_type,
                delim,
                start,
                tight,
                bullet_char,
            } => NodeValue::List(NodeList {
                list_type: (*list_type).into(),
                marker_offset: 0,
                padding: 0,
                start: *start as usize,
                delimiter: (*delim).into(),
                bullet_char: *bullet_char as u8,
                tight: *tight,
                is_task_list: false,
            }),
            NodeData::Item => NodeValue::Item(NodeList::default()),
            NodeData::CodeBlock { info, literal } => {
                NodeValue::CodeBlock(NodeCodeBlock {
                    fenced: !info.is_empty(),
                    fence_char: b'`',
                    fence_length: 3,
                    fence_offset: 0,
                    info: info.clone(),
                    literal: literal.clone(),
                    closed: true,
                })
            }
            NodeData::HtmlBlock { literal } => NodeValue::HtmlBlock(NodeHtmlBlock {
                block_type: 0,
                literal: literal.clone(),
            }),
            NodeData::CustomBlock { on_enter, on_exit } => {
                NodeValue::Raw(format!("{}{}", on_enter, on_exit))
            }
            NodeData::Paragraph => NodeValue::Paragraph,
            NodeData::Heading { level, content } => NodeValue::Heading(NodeHeading {
                level: *level as u8,
                setext: false,
                closed: true,
            }),
            NodeData::ThematicBreak => NodeValue::ThematicBreak,
            NodeData::Table {
                num_columns,
                alignments,
            } => NodeValue::Table(NodeTable {
                alignments: alignments.iter().map(|a| (*a).into()).collect(),
                num_columns: *num_columns,
                num_rows: 0,
                num_nonempty_cells: 0,
            }),
            NodeData::TableHead => NodeValue::TableRow(true),
            NodeData::TableRow => NodeValue::TableRow(false),
            NodeData::TableCell {
                column_index: _,
                alignment,
                is_header,
            } => {
                // Note: TableCell doesn't have metadata in NodeValue, so we just return the type
                NodeValue::TableCell
            }
            NodeData::Text { literal } => NodeValue::Text(literal.clone()),
            NodeData::SoftBreak => NodeValue::SoftBreak,
            NodeData::LineBreak => NodeValue::HardBreak,
            NodeData::Code { literal } => NodeValue::Code(NodeCode {
                num_backticks: 1,
                literal: literal.clone(),
            }),
            NodeData::HtmlInline { literal } => NodeValue::HtmlInline(literal.clone()),
            NodeData::CustomInline { on_enter, on_exit } => {
                NodeValue::Raw(format!("{}{}", on_enter, on_exit))
            }
            NodeData::Emph => NodeValue::Emph,
            NodeData::Strong => NodeValue::Strong,
            NodeData::Strikethrough => NodeValue::Strikethrough,
            NodeData::Link { url, title } => NodeValue::Link(NodeLink {
                url: url.clone(),
                title: title.clone(),
            }),
            NodeData::Image { url, title } => NodeValue::Image(NodeLink {
                url: url.clone(),
                title: title.clone(),
            }),
            NodeData::TaskItem { checked } => NodeValue::TaskItem(NodeTaskItem {
                symbol: if *checked { Some('x') } else { None },
            }),
            NodeData::FootnoteRef { label, ordinal } => {
                NodeValue::FootnoteReference(NodeFootnoteReference {
                    name: label.clone(),
                    ref_num: *ordinal as u32,
                    ix: *ordinal as u32,
                })
            }
            NodeData::FootnoteDef {
                label,
                ordinal,
                ref_count,
            } => NodeValue::FootnoteDefinition(NodeFootnoteDefinition {
                name: label.clone(),
                total_references: *ref_count as u32,
            }),
            NodeData::None => NodeValue::Raw(String::new()),
        }
    }
}

// Helper type conversions
impl From<node::ListType> for ListType {
    fn from(list_type: node::ListType) -> Self {
        match list_type {
            node::ListType::Bullet => ListType::Bullet,
            node::ListType::Ordered => ListType::Ordered,
            node::ListType::None => ListType::Bullet, // Default to bullet
        }
    }
}

impl From<node::DelimType> for ListDelimType {
    fn from(delim: node::DelimType) -> Self {
        match delim {
            node::DelimType::Period => ListDelimType::Period,
            node::DelimType::Paren => ListDelimType::Paren,
            node::DelimType::None => ListDelimType::Period, // Default to period
        }
    }
}

impl From<node::TableAlignment> for TableAlignment {
    fn from(alignment: node::TableAlignment) -> Self {
        match alignment {
            node::TableAlignment::None => TableAlignment::None,
            node::TableAlignment::Left => TableAlignment::Left,
            node::TableAlignment::Center => TableAlignment::Center,
            node::TableAlignment::Right => TableAlignment::Right,
        }
    }
}

impl From<node::SourcePos> for SourcePos {
    fn from(pos: node::SourcePos) -> Self {
        SourcePos {
            start: LineColumn {
                line: pos.start_line as usize,
                column: pos.start_column as usize,
            },
            end: LineColumn {
                line: pos.end_line as usize,
                column: pos.end_column as usize,
            },
        }
    }
}

// Reverse conversions (from node_value to node types)
impl From<ListType> for node::ListType {
    fn from(list_type: ListType) -> Self {
        match list_type {
            ListType::Bullet => node::ListType::Bullet,
            ListType::Ordered => node::ListType::Ordered,
        }
    }
}

impl From<ListDelimType> for node::DelimType {
    fn from(delim: ListDelimType) -> Self {
        match delim {
            ListDelimType::Period => node::DelimType::Period,
            ListDelimType::Paren => node::DelimType::Paren,
        }
    }
}

impl From<TableAlignment> for node::TableAlignment {
    fn from(alignment: TableAlignment) -> Self {
        match alignment {
            TableAlignment::None => node::TableAlignment::None,
            TableAlignment::Left => node::TableAlignment::Left,
            TableAlignment::Center => node::TableAlignment::Center,
            TableAlignment::Right => node::TableAlignment::Right,
        }
    }
}

impl From<SourcePos> for node::SourcePos {
    fn from(pos: SourcePos) -> Self {
        node::SourcePos {
            start_line: pos.start.line as u32,
            start_column: pos.start.column as u32,
            end_line: pos.end.line as u32,
            end_column: pos.end.column as u32,
        }
    }
}

// Conversions from NodeValue to NodeType and NodeData (for backward compatibility)
impl From<&NodeValue> for NodeType {
    fn from(value: &NodeValue) -> Self {
        match value {
            NodeValue::Document => NodeType::Document,
            NodeValue::BlockQuote => NodeType::BlockQuote,
            NodeValue::List(..) => NodeType::List,
            NodeValue::Item(..) => NodeType::Item,
            NodeValue::CodeBlock(..) => NodeType::CodeBlock,
            NodeValue::HtmlBlock(..) => NodeType::HtmlBlock,
            NodeValue::Paragraph => NodeType::Paragraph,
            NodeValue::Heading(..) => NodeType::Heading,
            NodeValue::ThematicBreak => NodeType::ThematicBreak,
            NodeValue::Table(..) => NodeType::Table,
            NodeValue::TableRow(..) => NodeType::TableRow,
            NodeValue::TableCell => NodeType::TableCell,
            NodeValue::Text(..) => NodeType::Text,
            NodeValue::SoftBreak => NodeType::SoftBreak,
            NodeValue::HardBreak => NodeType::LineBreak,
            NodeValue::Code(..) => NodeType::Code,
            NodeValue::HtmlInline(..) => NodeType::HtmlInline,
            NodeValue::Emph => NodeType::Emph,
            NodeValue::Strong => NodeType::Strong,
            NodeValue::Link(..) => NodeType::Link,
            NodeValue::Image(..) => NodeType::Image,
            NodeValue::Strikethrough => NodeType::Strikethrough,
            NodeValue::TaskItem(..) => NodeType::TaskItem,
            NodeValue::FootnoteReference(..) => NodeType::FootnoteRef,
            NodeValue::FootnoteDefinition(..) => NodeType::FootnoteDef,
            _ => NodeType::None,
        }
    }
}

impl From<&NodeValue> for NodeData {
    fn from(value: &NodeValue) -> Self {
        match value {
            NodeValue::Document => NodeData::Document,
            NodeValue::BlockQuote => NodeData::BlockQuote,
            NodeValue::List(list) => NodeData::List {
                list_type: list.list_type.into(),
                delim: list.delimiter.into(),
                start: list.start as u32,
                tight: list.tight,
                bullet_char: list.bullet_char as char,
            },
            NodeValue::Item(..) => NodeData::Item,
            NodeValue::CodeBlock(code) => NodeData::CodeBlock {
                info: code.info.clone(),
                literal: code.literal.clone(),
            },
            NodeValue::HtmlBlock(html) => NodeData::HtmlBlock {
                literal: html.literal.clone(),
            },
            NodeValue::Paragraph => NodeData::Paragraph,
            NodeValue::Heading(heading) => NodeData::Heading {
                level: heading.level as u32,
                content: String::new(),
            },
            NodeValue::ThematicBreak => NodeData::ThematicBreak,
            NodeValue::Table(table) => NodeData::Table {
                num_columns: table.num_columns,
                alignments: table.alignments.iter().map(|a| (*a).into()).collect(),
            },
            NodeValue::TableRow(..) => NodeData::TableRow,
            NodeValue::TableCell => NodeData::TableCell {
                column_index: 0,
                alignment: node::TableAlignment::None,
                is_header: false,
            },
            NodeValue::Text(text) => NodeData::Text {
                literal: text.clone(),
            },
            NodeValue::SoftBreak => NodeData::SoftBreak,
            NodeValue::HardBreak => NodeData::LineBreak,
            NodeValue::Code(code) => NodeData::Code {
                literal: code.literal.clone(),
            },
            NodeValue::HtmlInline(html) => NodeData::HtmlInline {
                literal: html.clone(),
            },
            NodeValue::Emph => NodeData::Emph,
            NodeValue::Strong => NodeData::Strong,
            NodeValue::Link(link) => NodeData::Link {
                url: link.url.clone(),
                title: link.title.clone(),
            },
            NodeValue::Image(link) => NodeData::Image {
                url: link.url.clone(),
                title: link.title.clone(),
            },
            NodeValue::Strikethrough => NodeData::Strikethrough,
            NodeValue::TaskItem(task) => NodeData::TaskItem {
                checked: task.symbol.is_some(),
            },
            NodeValue::FootnoteReference(footnote) => NodeData::FootnoteRef {
                label: footnote.name.clone(),
                ordinal: footnote.ref_num as usize,
            },
            NodeValue::FootnoteDefinition(footnote) => NodeData::FootnoteDef {
                label: footnote.name.clone(),
                ordinal: 0,
                ref_count: footnote.total_references as usize,
            },
            _ => NodeData::None,
        }
    }
}

/// Metadata for a list.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NodeList {
    /// The kind of list (bullet or ordered).
    pub list_type: ListType,

    /// Number of spaces before the list marker.
    pub marker_offset: usize,

    /// Number of characters between the start of the list marker
    /// and the item text (including the list marker(s)).
    pub padding: usize,

    /// For ordered lists, the ordinal the list starts at.
    pub start: usize,

    /// For ordered lists, the delimiter after each number.
    pub delimiter: ListDelimType,

    /// For bullet lists, the character used for each bullet.
    pub bullet_char: u8,

    /// Whether the list is tight (no blank lines between items).
    pub tight: bool,

    /// Whether the list contains tasks (checkbox items).
    pub is_task_list: bool,
}

/// The type of list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListType {
    /// A bullet list (unordered).
    #[default]
    Bullet,

    /// An ordered list.
    Ordered,
}

/// The delimiter for ordered lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListDelimType {
    /// A period character `.`.
    #[default]
    Period,

    /// A paren character `)`.
    Paren,
}

impl ListDelimType {
    /// Returns the XML name for this delimiter type.
    pub fn xml_name(&self) -> &'static str {
        match self {
            ListDelimType::Period => "period",
            ListDelimType::Paren => "paren",
        }
    }
}

/// Metadata for a description list item.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NodeDescriptionItem {
    /// Number of spaces before the list marker.
    pub marker_offset: usize,

    /// Number of characters between the start of the list marker
    /// and the item text.
    pub padding: usize,

    /// Whether the list is tight.
    pub tight: bool,
}

/// Metadata for a code block.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeCodeBlock {
    /// Whether the code block is fenced.
    pub fenced: bool,

    /// For fenced code blocks, the fence character (` or ~).
    pub fence_char: u8,

    /// For fenced code blocks, the length of the fence.
    pub fence_length: usize,

    /// For fenced code blocks, the indentation level.
    pub fence_offset: usize,

    /// The info string after the opening fence.
    pub info: String,

    /// The literal contents of the code block.
    pub literal: String,

    /// Whether the code block was explicitly closed.
    pub closed: bool,
}

/// Metadata for an HTML block.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeHtmlBlock {
    /// The HTML block type (1-7 per CommonMark spec).
    pub block_type: u8,

    /// The literal contents of the HTML block.
    pub literal: String,
}

/// Metadata for a heading.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NodeHeading {
    /// The level of the heading (1-6 for ATX, 1-2 for setext).
    pub level: u8,

    /// Whether the heading is setext style.
    pub setext: bool,

    /// For ATX headings, whether it had closing hashes.
    pub closed: bool,
}

/// Metadata for a footnote definition.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeFootnoteDefinition {
    /// The name of the footnote.
    pub name: String,

    /// Total number of references to this footnote.
    pub total_references: u32,
}

/// Metadata for a table.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeTable {
    /// The alignments for each column.
    pub alignments: Vec<TableAlignment>,

    /// Number of columns.
    pub num_columns: usize,

    /// Number of rows.
    pub num_rows: usize,

    /// Number of non-empty cells.
    pub num_nonempty_cells: usize,
}

/// Alignment of a table cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableAlignment {
    /// No specific alignment.
    #[default]
    None,

    /// Left-aligned.
    Left,

    /// Center-aligned.
    Center,

    /// Right-aligned.
    Right,
}

impl TableAlignment {
    /// Returns the XML name for this alignment.
    pub fn xml_name(&self) -> Option<&'static str> {
        match self {
            TableAlignment::None => None,
            TableAlignment::Left => Some("left"),
            TableAlignment::Center => Some("center"),
            TableAlignment::Right => Some("right"),
        }
    }
}

/// Metadata for a task list item.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NodeTaskItem {
    /// The symbol used to mark the task as checked, or None if unchecked.
    pub symbol: Option<char>,
}

/// Metadata for inline code.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeCode {
    /// The number of backticks used.
    pub num_backticks: usize,

    /// The content of the code span.
    pub literal: String,
}

/// Metadata for a link or image.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeLink {
    /// The URL.
    pub url: String,

    /// The title.
    pub title: String,
}

/// Metadata for a footnote reference.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeFootnoteReference {
    /// The name of the footnote.
    pub name: String,

    /// The reference number.
    pub ref_num: u32,

    /// The index of the footnote in the document.
    pub ix: u32,
}

/// Metadata for a math span.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeMath {
    /// Whether this is dollar math (true) or code math (false).
    pub dollar_math: bool,

    /// Whether this is display math.
    pub display_math: bool,

    /// The literal contents.
    pub literal: String,
}

/// Metadata for a wikilink.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeWikiLink {
    /// The URL or page title.
    pub url: String,
}

/// Metadata for a multiline block quote.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NodeMultilineBlockQuote {
    /// The length of the fence.
    pub fence_length: usize,

    /// The indentation level of the fence marker.
    pub fence_offset: usize,
}

/// The type of alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertType {
    /// Useful information.
    #[default]
    Note,

    /// Helpful advice.
    Tip,

    /// Key information.
    Important,

    /// Urgent info.
    Warning,

    /// Risk warning.
    Caution,
}

impl AlertType {
    /// Returns the default title for this alert type.
    pub fn default_title(&self) -> &'static str {
        match self {
            AlertType::Note => "Note",
            AlertType::Tip => "Tip",
            AlertType::Important => "Important",
            AlertType::Warning => "Warning",
            AlertType::Caution => "Caution",
        }
    }

    /// Returns the CSS class for this alert type.
    pub fn css_class(&self) -> &'static str {
        match self {
            AlertType::Note => "markdown-alert-note",
            AlertType::Tip => "markdown-alert-tip",
            AlertType::Important => "markdown-alert-important",
            AlertType::Warning => "markdown-alert-warning",
            AlertType::Caution => "markdown-alert-caution",
        }
    }
}

/// Metadata for an alert.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeAlert {
    /// The type of alert.
    pub alert_type: AlertType,

    /// Optional custom title.
    pub title: Option<String>,

    /// Whether this originated from a multiline block quote.
    pub multiline: bool,

    /// The fence length (for multiline alerts).
    pub fence_length: usize,

    /// The fence offset (for multiline alerts).
    pub fence_offset: usize,
}

/// Represents a position in the source document.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LineColumn {
    /// The 1-based line number.
    pub line: usize,

    /// The 1-based column number (in bytes).
    pub column: usize,
}

impl LineColumn {
    /// Create a new LineColumn.
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Returns a new LineColumn with the column adjusted by the given offset.
    pub fn column_add(&self, offset: isize) -> Option<LineColumn> {
        let new_column = (self.column as isize) + offset;
        if new_column >= 1 {
            Some(LineColumn {
                line: self.line,
                column: new_column as usize,
            })
        } else {
            None
        }
    }
}

/// Represents a source position range.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourcePos {
    /// The starting position.
    pub start: LineColumn,

    /// The ending position.
    pub end: LineColumn,
}

impl SourcePos {
    /// Create a new SourcePos.
    pub fn new(
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
    ) -> Self {
        Self {
            start: LineColumn::new(start_line, start_column),
            end: LineColumn::new(end_line, end_column),
        }
    }

    /// Create a SourcePos from LineColumns.
    pub fn from_line_columns(start: LineColumn, end: LineColumn) -> Self {
        Self { start, end }
    }
}

impl std::fmt::Display for SourcePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}-{}:{}",
            self.start.line, self.start.column, self.end.line, self.end.column
        )
    }
}

/// Validates whether a parent node can contain a child node value.
///
/// This function implements the CommonMark spec's containment rules.
pub fn can_contain_type(parent: &NodeValue, child: &NodeValue) -> bool {
    match parent {
        NodeValue::Raw(_) => true,

        NodeValue::Document => child.is_block() && !matches!(child, NodeValue::Document),

        NodeValue::BlockQuote
        | NodeValue::FootnoteDefinition(_)
        | NodeValue::DescriptionTerm
        | NodeValue::DescriptionDetails
        | NodeValue::Item(..)
        | NodeValue::TaskItem(..) => {
            child.is_block()
                && !matches!(child, NodeValue::Item(..) | NodeValue::TaskItem(..))
        }

        NodeValue::List(..) => {
            matches!(child, NodeValue::Item(..) | NodeValue::TaskItem(..))
        }

        NodeValue::DescriptionList => matches!(child, NodeValue::DescriptionItem(_)),

        NodeValue::DescriptionItem(_) => matches!(
            child,
            NodeValue::DescriptionTerm | NodeValue::DescriptionDetails
        ),

        NodeValue::Table(..) => matches!(child, NodeValue::TableRow(..)),

        NodeValue::TableRow(..) => matches!(child, NodeValue::TableCell),

        NodeValue::TableCell => child.is_inline(),

        NodeValue::MultilineBlockQuote(_) | NodeValue::Alert(_) => {
            child.is_block()
                && !matches!(child, NodeValue::Item(..) | NodeValue::TaskItem(..))
        }

        NodeValue::Paragraph
        | NodeValue::Heading(..)
        | NodeValue::Emph
        | NodeValue::Strong
        | NodeValue::Link(..)
        | NodeValue::Image(..)
        | NodeValue::WikiLink(..)
        | NodeValue::Strikethrough
        | NodeValue::Highlight
        | NodeValue::Insert
        | NodeValue::Superscript
        | NodeValue::Spoiler
        | NodeValue::Underline
        | NodeValue::Subscript
        | NodeValue::Subtext => child.is_inline(),

        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_value_classification() {
        assert!(NodeValue::Document.is_block());
        assert!(NodeValue::Paragraph.is_block());
        assert!(!NodeValue::Text(String::new()).is_block());

        assert!(NodeValue::Text(String::new()).is_inline());
        assert!(NodeValue::Link(NodeLink::default()).is_inline());
        assert!(!NodeValue::Paragraph.is_inline());

        assert!(NodeValue::Text(String::new()).is_leaf());
        assert!(NodeValue::CodeBlock(NodeCodeBlock::default()).is_leaf());
        assert!(!NodeValue::Paragraph.is_leaf());
    }

    #[test]
    fn test_text_methods() {
        let mut value = NodeValue::Text("hello".to_string());
        assert_eq!(value.text(), Some("hello"));
        assert_eq!(value.text_mut(), Some(&mut "hello".to_string()));

        if let Some(text) = value.text_mut() {
            text.push_str(" world");
        }
        assert_eq!(value.text(), Some("hello world"));

        let mut non_text = NodeValue::Paragraph;
        assert_eq!(non_text.text(), None);
        assert_eq!(non_text.text_mut(), None);
    }

    #[test]
    fn test_xml_node_names() {
        assert_eq!(NodeValue::Document.xml_node_name(), "document");
        assert_eq!(NodeValue::Paragraph.xml_node_name(), "paragraph");
        assert_eq!(NodeValue::Text(String::new()).xml_node_name(), "text");
        assert_eq!(NodeValue::Strong.xml_node_name(), "strong");
    }

    #[test]
    fn test_list_types() {
        assert_ne!(ListType::Bullet, ListType::Ordered);

        let list = NodeList {
            list_type: ListType::Bullet,
            bullet_char: b'-',
            ..Default::default()
        };
        assert_eq!(list.list_type, ListType::Bullet);
        assert_eq!(list.bullet_char, b'-');
    }

    #[test]
    fn test_delim_types() {
        assert_eq!(ListDelimType::Period.xml_name(), "period");
        assert_eq!(ListDelimType::Paren.xml_name(), "paren");
    }

    #[test]
    fn test_table_alignments() {
        assert_eq!(TableAlignment::None.xml_name(), None);
        assert_eq!(TableAlignment::Left.xml_name(), Some("left"));
        assert_eq!(TableAlignment::Center.xml_name(), Some("center"));
        assert_eq!(TableAlignment::Right.xml_name(), Some("right"));
    }

    #[test]
    fn test_alert_types() {
        assert_eq!(AlertType::Note.default_title(), "Note");
        assert_eq!(AlertType::Warning.css_class(), "markdown-alert-warning");
    }

    #[test]
    fn test_line_column() {
        let lc = LineColumn::new(1, 5);
        assert_eq!(lc.line, 1);
        assert_eq!(lc.column, 5);

        let adjusted = lc.column_add(3).unwrap();
        assert_eq!(adjusted.column, 8);

        let invalid = lc.column_add(-10);
        assert!(invalid.is_none());
    }

    #[test]
    fn test_source_pos() {
        let pos = SourcePos::new(1, 1, 3, 10);
        assert_eq!(pos.start.line, 1);
        assert_eq!(pos.start.column, 1);
        assert_eq!(pos.end.line, 3);
        assert_eq!(pos.end.column, 10);

        let display = format!("{}", pos);
        assert_eq!(display, "1:1-3:10");
    }

    #[test]
    fn test_can_contain_type() {
        // Document can contain blocks
        assert!(can_contain_type(
            &NodeValue::Document,
            &NodeValue::Paragraph
        ));
        assert!(!can_contain_type(
            &NodeValue::Document,
            &NodeValue::Document
        ));

        // Paragraph can contain inlines
        assert!(can_contain_type(
            &NodeValue::Paragraph,
            &NodeValue::Text("hi".to_string())
        ));
        assert!(!can_contain_type(
            &NodeValue::Paragraph,
            &NodeValue::Paragraph
        ));

        // List can contain items
        assert!(can_contain_type(
            &NodeValue::List(NodeList::default()),
            &NodeValue::Item(NodeList::default())
        ));

        // Table can contain rows
        assert!(can_contain_type(
            &NodeValue::Table(NodeTable::default()),
            &NodeValue::TableRow(false)
        ));
    }

    #[test]
    fn test_contains_inlines() {
        assert!(NodeValue::Paragraph.contains_inlines());
        assert!(NodeValue::Heading(NodeHeading::default()).contains_inlines());
        assert!(NodeValue::TableCell.contains_inlines());
        assert!(!NodeValue::BlockQuote.contains_inlines());
    }

    #[test]
    fn test_heading_metadata() {
        let heading = NodeHeading {
            level: 2,
            setext: false,
            closed: true,
        };
        assert_eq!(heading.level, 2);
        assert!(!heading.setext);
        assert!(heading.closed);
    }

    #[test]
    fn test_code_block_metadata() {
        let code = NodeCodeBlock {
            fenced: true,
            fence_char: b'`',
            fence_length: 3,
            info: "rust".to_string(),
            literal: "fn main() {}".to_string(),
            ..Default::default()
        };
        assert!(code.fenced);
        assert_eq!(code.fence_char, b'`');
        assert_eq!(code.info, "rust");
    }

    #[test]
    fn test_link_metadata() {
        let link = NodeLink {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
        };
        assert_eq!(link.url, "https://example.com");
        assert_eq!(link.title, "Example");

        let image = NodeValue::Image(NodeLink {
            url: "image.png".to_string(),
            title: "An image".to_string(),
        });
        assert!(matches!(image, NodeValue::Image(_)));
    }

    #[test]
    fn test_footnote_metadata() {
        let def = NodeFootnoteDefinition {
            name: "1".to_string(),
            total_references: 2,
        };
        assert_eq!(def.name, "1");
        assert_eq!(def.total_references, 2);

        let ref_node = NodeFootnoteReference {
            name: "1".to_string(),
            ref_num: 1,
            ix: 0,
        };
        assert_eq!(ref_node.ref_num, 1);
    }

    #[test]
    fn test_math_metadata() {
        let math = NodeMath {
            dollar_math: true,
            display_math: true,
            literal: "x + y".to_string(),
        };
        assert!(math.dollar_math);
        assert!(math.display_math);
    }

    #[test]
    fn test_task_item_metadata() {
        let checked = NodeTaskItem { symbol: Some('x') };
        assert_eq!(checked.symbol, Some('x'));

        let unchecked = NodeTaskItem { symbol: None };
        assert_eq!(unchecked.symbol, None);
    }

    // =============================================================================
    // Migration tests from node.rs to node_value.rs
    // =============================================================================

    #[test]
    fn test_node_type_to_value_conversion() {
        use crate::node::NodeType;

        // Test block types
        assert!(matches!(
            NodeValue::from(NodeType::Document),
            NodeValue::Document
        ));
        assert!(matches!(
            NodeValue::from(NodeType::BlockQuote),
            NodeValue::BlockQuote
        ));
        assert!(matches!(
            NodeValue::from(NodeType::Paragraph),
            NodeValue::Paragraph
        ));
        assert!(matches!(
            NodeValue::from(NodeType::Heading),
            NodeValue::Heading(_)
        ));

        // Test inline types
        assert!(matches!(
            NodeValue::from(NodeType::Text),
            NodeValue::Text(_)
        ));
        assert!(matches!(NodeValue::from(NodeType::Emph), NodeValue::Emph));
        assert!(matches!(
            NodeValue::from(NodeType::Strong),
            NodeValue::Strong
        ));
    }

    #[test]
    fn test_node_data_to_value_conversion() {
        use crate::node::{NodeData, NodeType};

        // Test text data
        let text_data = NodeData::Text {
            literal: "Hello".to_string(),
        };
        let value = NodeValue::from(&text_data);
        assert!(matches!(value, NodeValue::Text(ref s) if s == "Hello"));

        // Test heading data
        let heading_data = NodeData::Heading {
            level: 2,
            content: "Title".to_string(),
        };
        let value = NodeValue::from(&heading_data);
        if let NodeValue::Heading(heading) = value {
            assert_eq!(heading.level, 2);
        } else {
            panic!("Expected Heading");
        }

        // Test code block data
        let code_data = NodeData::CodeBlock {
            info: "rust".to_string(),
            literal: "fn main() {}".to_string(),
        };
        let value = NodeValue::from(&code_data);
        if let NodeValue::CodeBlock(code) = value {
            assert_eq!(code.info, "rust");
            assert_eq!(code.literal, "fn main() {}");
        } else {
            panic!("Expected CodeBlock");
        }

        // Test link data
        let link_data = NodeData::Link {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
        };
        let value = NodeValue::from(&link_data);
        if let NodeValue::Link(link) = value {
            assert_eq!(link.url, "https://example.com");
            assert_eq!(link.title, "Example");
        } else {
            panic!("Expected Link");
        }
    }

    #[test]
    fn test_list_type_conversion() {
        use crate::node;

        let bullet: ListType = node::ListType::Bullet.into();
        assert!(matches!(bullet, ListType::Bullet));

        let ordered: ListType = node::ListType::Ordered.into();
        assert!(matches!(ordered, ListType::Ordered));
    }

    #[test]
    fn test_delim_type_conversion() {
        use crate::node;

        let period: ListDelimType = node::DelimType::Period.into();
        assert!(matches!(period, ListDelimType::Period));

        let paren: ListDelimType = node::DelimType::Paren.into();
        assert!(matches!(paren, ListDelimType::Paren));
    }

    #[test]
    fn test_table_alignment_conversion() {
        use crate::node;

        let none: TableAlignment = node::TableAlignment::None.into();
        assert!(matches!(none, TableAlignment::None));

        let left: TableAlignment = node::TableAlignment::Left.into();
        assert!(matches!(left, TableAlignment::Left));

        let center: TableAlignment = node::TableAlignment::Center.into();
        assert!(matches!(center, TableAlignment::Center));

        let right: TableAlignment = node::TableAlignment::Right.into();
        assert!(matches!(right, TableAlignment::Right));
    }

    #[test]
    fn test_source_pos_conversion() {
        use crate::node;

        let old_pos = node::SourcePos {
            start_line: 1,
            start_column: 2,
            end_line: 3,
            end_column: 4,
        };
        let new_pos: SourcePos = old_pos.into();

        assert_eq!(new_pos.start.line, 1);
        assert_eq!(new_pos.start.column, 2);
        assert_eq!(new_pos.end.line, 3);
        assert_eq!(new_pos.end.column, 4);
    }

    #[test]
    fn test_reverse_list_type_conversion() {
        let bullet: crate::node::ListType = ListType::Bullet.into();
        assert!(matches!(bullet, crate::node::ListType::Bullet));

        let ordered: crate::node::ListType = ListType::Ordered.into();
        assert!(matches!(ordered, crate::node::ListType::Ordered));
    }

    #[test]
    fn test_reverse_delim_type_conversion() {
        let period: crate::node::DelimType = ListDelimType::Period.into();
        assert!(matches!(period, crate::node::DelimType::Period));

        let paren: crate::node::DelimType = ListDelimType::Paren.into();
        assert!(matches!(paren, crate::node::DelimType::Paren));
    }

    #[test]
    fn test_reverse_table_alignment_conversion() {
        let none: crate::node::TableAlignment = TableAlignment::None.into();
        assert!(matches!(none, crate::node::TableAlignment::None));

        let left: crate::node::TableAlignment = TableAlignment::Left.into();
        assert!(matches!(left, crate::node::TableAlignment::Left));
    }

    #[test]
    fn test_reverse_source_pos_conversion() {
        let new_pos = SourcePos::new(1, 2, 3, 4);
        let old_pos: crate::node::SourcePos = new_pos.into();

        assert_eq!(old_pos.start_line, 1);
        assert_eq!(old_pos.start_column, 2);
        assert_eq!(old_pos.end_line, 3);
        assert_eq!(old_pos.end_column, 4);
    }

    // =============================================================================
    // Tests migrated from node.rs
    // =============================================================================

    #[test]
    fn test_list_type_variants_migrated() {
        assert_ne!(ListType::Bullet, ListType::Ordered);
    }

    #[test]
    fn test_delim_type_variants_migrated() {
        assert_ne!(ListDelimType::Period, ListDelimType::Paren);
    }

    #[test]
    fn test_node_value_list_metadata_migrated() {
        let list = NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        };

        if let NodeValue::List(list_data) = NodeValue::List(list) {
            assert_eq!(list_data.list_type, ListType::Bullet);
            assert_eq!(list_data.bullet_char, b'-');
            assert_eq!(list_data.start, 1);
            assert!(list_data.tight);
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_node_value_heading_metadata_migrated() {
        let heading = NodeHeading {
            level: 2,
            setext: false,
            closed: true,
        };

        if let NodeValue::Heading(heading_data) = NodeValue::Heading(heading) {
            assert_eq!(heading_data.level, 2);
            assert!(!heading_data.setext);
            assert!(heading_data.closed);
        } else {
            panic!("Expected Heading");
        }
    }

    #[test]
    fn test_node_value_code_block_metadata_migrated() {
        let code = NodeCodeBlock {
            fenced: true,
            fence_char: b'`',
            fence_length: 3,
            fence_offset: 0,
            info: "rust".to_string(),
            literal: "fn main() {}".to_string(),
            closed: true,
        };

        if let NodeValue::CodeBlock(code_data) = NodeValue::CodeBlock(code) {
            assert!(code_data.fenced);
            assert_eq!(code_data.fence_char, b'`');
            assert_eq!(code_data.info, "rust");
            assert_eq!(code_data.literal, "fn main() {}");
        } else {
            panic!("Expected CodeBlock");
        }
    }
}
