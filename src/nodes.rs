//! The CommonMark AST.
//!
//! This module provides the core AST node types for CommonMark documents.
//! It is inspired by comrak's design, combining node values with metadata
//! into a unified structure.

use std::borrow::Cow;
use std::cell::RefCell;

/// Shorthand for checking if a node's value matches the given expression.
///
/// Note this will call `node.data()`, which will fail if the node is already
/// mutably borrowed.
#[macro_export]
macro_rules! node_matches {
    ($node:expr, $( $pat:pat_param )|+) => {{
        matches!($node.data().value, $( $pat )|+)
    }};
}

/// The core AST node value enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeValue {
    /// The root of every CommonMark document. Contains blocks.
    Document,

    /// Non-Markdown front matter. Treated as an opaque blob.
    FrontMatter(Box<str>),

    /// A block quote. Contains other blocks.
    BlockQuote,

    /// A list. Contains list items.
    List(NodeList),

    /// A list item. Contains other blocks.
    Item(NodeList),

    /// A description list (definition list).
    DescriptionList,

    /// An item in a description list.
    DescriptionItem(NodeDescriptionItem),

    /// The term in a description list item.
    DescriptionTerm,

    /// The definition/details in a description list item.
    DescriptionDetails,

    /// A code block (fenced or indented).
    CodeBlock(Box<NodeCodeBlock>),

    /// An HTML block.
    HtmlBlock(Box<NodeHtmlBlock>),

    /// A paragraph. Contains inlines.
    Paragraph,

    /// A heading (ATX or setext).
    Heading(NodeHeading),

    /// A thematic break (horizontal rule).
    ThematicBreak,

    /// A footnote definition.
    FootnoteDefinition(Box<NodeFootnoteDefinition>),

    /// A table (GFM extension).
    Table(Box<NodeTable>),

    /// A table row.
    TableRow(bool), // bool indicates if this is the header row

    /// A table cell.
    TableCell,

    /// Textual content.
    Text(Box<str>),

    /// A task list item (GFM extension).
    TaskItem(NodeTaskItem),

    /// A soft line break.
    SoftBreak,

    /// A hard line break.
    HardBreak,

    /// An inline code span.
    Code(Box<NodeCode>),

    /// Raw HTML inline.
    HtmlInline(Box<str>),

    /// Emphasized text.
    Emph,

    /// Strongly emphasized text.
    Strong,

    /// Strikethrough text (GFM extension).
    Strikethrough,

    /// Highlighted/marked text.
    Highlight,

    /// Inserted text.
    Insert,

    /// Superscript text.
    Superscript,

    /// Subscript text.
    Subscript,

    /// A link.
    Link(Box<NodeLink>),

    /// An image.
    Image(Box<NodeLink>),

    /// A footnote reference.
    FootnoteReference(Box<NodeFootnoteReference>),

    /// A math span.
    Math(Box<NodeMath>),

    /// A wikilink.
    WikiLink(Box<NodeWikiLink>),

    /// Underlined text.
    Underline,

    /// Spoiler text (GFM Discord-style spoiler).
    SpoileredText,

    /// An escaped character.
    Escaped,

    /// A multiline block quote.
    MultilineBlockQuote(Box<NodeMultilineBlockQuote>),

    /// An alert (GFM extension).
    Alert(Box<NodeAlert>),

    /// Subtext (block-scoped subscript).
    Subtext,

    /// Raw output (not parsed, inserted verbatim).
    Raw(Box<str>),

    /// An escaped tag (used during parsing).
    EscapedTag(&'static str),
}

/// A single node in the CommonMark AST.
#[derive(Clone, PartialEq, Eq)]
pub struct Ast {
    /// The node value itself.
    pub value: NodeValue,

    /// The positions in the source document this node comes from.
    pub sourcepos: SourcePos,

    /// Content buffer for nodes that accumulate text.
    pub(crate) content: String,

    /// Whether the node is still open for adding content.
    pub(crate) open: bool,

    /// Whether the last line was blank.
    pub(crate) last_line_blank: bool,

    /// Whether this table node has been visited during processing.
    pub(crate) table_visited: bool,

    /// Line offsets for source position tracking.
    pub(crate) line_offsets: Vec<usize>,
}

/// The type of a node within the document.
///
/// It is bound by the lifetime `'a`, which corresponds to the `Arena` nodes are
/// allocated in. Child `Ast`s are wrapped in `RefCell` for interior mutability.
///
/// You can construct a new `AstNode` from a `NodeValue` using the `From` trait:
///
/// ```ignore
/// # use clmd::nodes::{AstNode, NodeValue};
/// let root = AstNode::from(NodeValue::Document);
/// ```
///
/// Note that no sourcepos information is given to the created node. If you wish
/// to assign sourcepos information, use the `From` trait to create an `AstNode`
/// from an `Ast`:
///
/// ```ignore
/// # use clmd::nodes::{Ast, AstNode, NodeValue};
/// let root = AstNode::from(Ast::new_with_sourcepos(
///     NodeValue::Paragraph,
///     SourcePos::new(4, 1, 4, 10),
/// ));
/// ```
///
/// For practical use, you'll probably need it allocated in an `Arena`, in which
/// case you can use `.into()` to simplify creation:
///
/// ```ignore
/// # use clmd::{nodes::{AstNode, NodeValue}, Arena};
/// # let arena = Arena::new();
/// let node_in_arena = arena.alloc(NodeValue::Document.into());
/// ```
pub type AstNode<'a> = crate::arena_tree::Node<'a, RefCell<Ast>>;

/// A reference to a node in an arena.
pub type Node<'a> = &'a AstNode<'a>;

// Size assertions to monitor type sizes (matching comrak's approach)
// These help ensure we don't accidentally bloat the node sizes
#[allow(dead_code)]
#[cfg(target_pointer_width = "64")]
const AST_SIZE_ASSERTION: [u8; 128] = [0; std::mem::size_of::<Ast>()];

#[allow(dead_code)]
#[cfg(target_pointer_width = "64")]
const AST_NODE_SIZE_ASSERTION: [u8; 176] = [0; std::mem::size_of::<AstNode<'_>>()];

impl std::fmt::Debug for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{:?} ({:?})>", self.value, self.sourcepos)
    }
}

impl Ast {
    /// Create a new AST node with the given value and starting sourcepos.
    /// The end column is set to zero; it is expected this will be set manually
    /// or later in the parse. Use [`Ast::new_with_sourcepos`] if you have full
    /// sourcepos.
    pub fn new(value: NodeValue, start: LineColumn) -> Self {
        Ast {
            value,
            content: String::new(),
            sourcepos: SourcePos {
                start,
                end: LineColumn::new(start.line, 0),
            },
            open: true,
            last_line_blank: false,
            table_visited: false,
            line_offsets: Vec::new(),
        }
    }

    /// Create a new AST node with the given value and full sourcepos.
    pub fn new_with_sourcepos(value: NodeValue, sourcepos: SourcePos) -> Self {
        Ast {
            value,
            content: String::new(),
            sourcepos,
            open: true,
            last_line_blank: false,
            table_visited: false,
            line_offsets: Vec::new(),
        }
    }
}

impl<'a> From<NodeValue> for AstNode<'a> {
    /// Create a new AST node with the given value. The sourcepos is set to (0,0)-(0,0).
    fn from(value: NodeValue) -> Self {
        crate::arena_tree::Node::new(RefCell::new(Ast::new(value, LineColumn::default())))
    }
}

impl<'a> From<Ast> for AstNode<'a> {
    /// Create a new AST node with the given Ast.
    fn from(ast: Ast) -> Self {
        crate::arena_tree::Node::new(RefCell::new(ast))
    }
}

/// Represents a position in the source document.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
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

    /// Return a new LineColumn based on this one, with the column adjusted by offset.
    /// Returns None if the result would be negative.
    pub fn column_add(&self, offset: isize) -> Option<LineColumn> {
        let new_column = (self.column as isize) + offset;
        if new_column < 0 {
            return None;
        }
        Some(LineColumn {
            line: self.line,
            column: new_column as usize,
        })
    }
}

/// Represents a source position range.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
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

impl From<(usize, usize, usize, usize)> for SourcePos {
    fn from((sl, sc, el, ec): (usize, usize, usize, usize)) -> Self {
        SourcePos {
            start: LineColumn::new(sl, sc),
            end: LineColumn::new(el, ec),
        }
    }
}

impl From<(LineColumn, LineColumn)> for SourcePos {
    fn from((start, end): (LineColumn, LineColumn)) -> Self {
        SourcePos { start, end }
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
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListType {
    /// A bullet list (unordered).
    #[default]
    Bullet,

    /// An ordered list.
    Ordered,
}

/// The delimiter for ordered lists.
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
                | NodeValue::SpoileredText
                | NodeValue::Escaped
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
                | NodeValue::Escaped
                | NodeValue::Raw(..)
                | NodeValue::FrontMatter(..)
        )
    }

    /// Return a reference to the text of a `Text` inline, if this node is one.
    ///
    /// Convenience method.
    pub fn text(&self) -> Option<&str> {
        match self {
            NodeValue::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Return a mutable reference to the text of a `Text` inline, if this node is one.
    ///
    /// Convenience method.
    pub fn text_mut(&mut self) -> Option<&mut Box<str>> {
        match self {
            NodeValue::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Returns the XML node name for this value.
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
            NodeValue::TaskItem(..) => "taskitem",
            NodeValue::Superscript => "superscript",
            NodeValue::FootnoteReference(..) => "footnote_reference",
            NodeValue::MultilineBlockQuote(_) => "multiline_block_quote",
            NodeValue::Escaped => "escaped",
            NodeValue::Math(..) => "math",
            NodeValue::WikiLink(..) => "wikilink",
            NodeValue::Underline => "underline",
            NodeValue::Subscript => "subscript",
            NodeValue::SpoileredText => "spoilered_text",
            NodeValue::Alert(_) => "alert",
            NodeValue::Subtext => "subtext",
            NodeValue::EscapedTag(_) => "escaped_tag",
        }
    }

    /// Check if this node type can accept lines of text.
    pub fn accepts_lines(&self) -> bool {
        matches!(
            self,
            NodeValue::Paragraph
                | NodeValue::Heading(..)
                | NodeValue::CodeBlock(..)
                | NodeValue::Subtext
        )
    }

    /// Create a Text node from a string.
    pub fn make_text<S: Into<Box<str>>>(s: S) -> Self {
        NodeValue::Text(s.into())
    }

    /// Create a CodeBlock node.
    pub fn code_block(code: NodeCodeBlock) -> Self {
        NodeValue::CodeBlock(Box::new(code))
    }

    /// Create an HtmlBlock node.
    pub fn html_block(block: NodeHtmlBlock) -> Self {
        NodeValue::HtmlBlock(Box::new(block))
    }

    /// Create a FootnoteDefinition node.
    pub fn footnote_definition(def: NodeFootnoteDefinition) -> Self {
        NodeValue::FootnoteDefinition(Box::new(def))
    }

    /// Create a Table node.
    pub fn table(table: NodeTable) -> Self {
        NodeValue::Table(Box::new(table))
    }

    /// Create a Code inline node.
    pub fn code(code: NodeCode) -> Self {
        NodeValue::Code(Box::new(code))
    }

    /// Create a Link node.
    pub fn link(link: NodeLink) -> Self {
        NodeValue::Link(Box::new(link))
    }

    /// Create an Image node.
    pub fn image(image: NodeLink) -> Self {
        NodeValue::Image(Box::new(image))
    }

    /// Create a FootnoteReference node.
    pub fn footnote_reference(ref_: NodeFootnoteReference) -> Self {
        NodeValue::FootnoteReference(Box::new(ref_))
    }

    /// Create a Math node.
    pub fn math(math: NodeMath) -> Self {
        NodeValue::Math(Box::new(math))
    }

    /// Create a WikiLink node.
    pub fn wiki_link(wiki: NodeWikiLink) -> Self {
        NodeValue::WikiLink(Box::new(wiki))
    }

    /// Create a MultilineBlockQuote node.
    pub fn multiline_block_quote(quote: NodeMultilineBlockQuote) -> Self {
        NodeValue::MultilineBlockQuote(Box::new(quote))
    }

    /// Create an Alert node.
    pub fn alert(alert: NodeAlert) -> Self {
        NodeValue::Alert(Box::new(alert))
    }
}

/// Validates whether a parent node can contain a child node value.
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
        | NodeValue::SpoileredText
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
        assert!(!NodeValue::make_text("").is_block());

        assert!(NodeValue::make_text("").is_inline());
        assert!(NodeValue::link(NodeLink::default()).is_inline());
        assert!(!NodeValue::Paragraph.is_inline());

        assert!(NodeValue::make_text("").is_leaf());
        assert!(NodeValue::code_block(NodeCodeBlock::default()).is_leaf());
        assert!(!NodeValue::Paragraph.is_leaf());
    }

    #[test]
    fn test_text_methods() {
        let mut value = NodeValue::make_text("hello");
        assert_eq!(value.text(), Some("hello"));
        assert!(value.text_mut().is_some());

        if let Some(text) = value.text_mut() {
            *text = format!("{} world", text).into_boxed_str();
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
        assert_eq!(NodeValue::make_text("").xml_node_name(), "text");
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
            &NodeValue::make_text("hi")
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

        let image = NodeValue::image(NodeLink {
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

    #[test]
    fn test_ast_creation() {
        let ast = Ast::new(NodeValue::Paragraph, LineColumn::new(1, 1));
        assert!(ast.open);
        assert_eq!(ast.sourcepos.start.line, 1);
        assert_eq!(ast.sourcepos.start.column, 1);
    }

    #[test]
    fn test_ast_with_sourcepos() {
        let sourcepos = SourcePos::new(1, 1, 5, 10);
        let ast = Ast::new_with_sourcepos(NodeValue::Document, sourcepos);
        assert_eq!(ast.sourcepos.start.line, 1);
        assert_eq!(ast.sourcepos.end.line, 5);
    }
}

/// Error type for AST validation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A node has an invalid parent-child relationship.
    InvalidParentChild {
        /// The parent node type.
        parent: &'static str,
        /// The child node type.
        child: &'static str,
    },
    /// A leaf node has children.
    LeafNodeWithChildren {
        /// The node type.
        node_type: &'static str,
    },
    /// A required child is missing.
    MissingRequiredChild {
        /// The parent node type.
        parent: &'static str,
        /// Description of what's missing.
        description: &'static str,
    },
    /// A node has an invalid property value.
    InvalidProperty {
        /// The node type.
        node_type: &'static str,
        /// The property name.
        property: &'static str,
        /// The invalid value.
        value: String,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidParentChild { parent, child } => {
                write!(f, "Invalid parent-child relationship: {} cannot contain {}", parent, child)
            }
            ValidationError::LeafNodeWithChildren { node_type } => {
                write!(f, "Leaf node {} cannot have children", node_type)
            }
            ValidationError::MissingRequiredChild { parent, description } => {
                write!(f, "{} is missing required child: {}", parent, description)
            }
            ValidationError::InvalidProperty { node_type, property, value } => {
                write!(f, "Invalid property {} on {}: {}", property, node_type, value)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate an AST node and its children.
///
/// This function checks that the AST structure is valid according to CommonMark
/// rules. It verifies parent-child relationships, ensures leaf nodes don't have
/// children, and checks for other structural constraints.
///
/// # Arguments
///
/// * `node` - The root node to validate
///
/// # Returns
///
/// `Ok(())` if the AST is valid, or a `ValidationError` describing the first
/// problem found.
///
/// # Example
///
/// ```
/// use clmd::{Arena, parse_document, Options, nodes::validate};
///
/// let arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&arena, "# Hello\n\nWorld", &options);
///
/// assert!(validate(root).is_ok());
/// ```
pub fn validate(node: &AstNode<'_>) -> Result<(), ValidationError> {
    validate_node(node, None)
}

/// Internal validation function that tracks the parent.
fn validate_node(
    node: &AstNode<'_>,
    parent: Option<&NodeValue>,
) -> Result<(), ValidationError> {
    let ast = node.data.borrow();
    let value = &ast.value;

    // Check parent-child relationship
    if let Some(parent_value) = parent {
        if !can_contain_type(parent_value, value) {
            return Err(ValidationError::InvalidParentChild {
                parent: parent_value.xml_node_name(),
                child: value.xml_node_name(),
            });
        }
    }

    // Check that leaf nodes don't have children
    if value.is_leaf() && node.first_child().is_some() {
        return Err(ValidationError::LeafNodeWithChildren {
            node_type: value.xml_node_name(),
        });
    }

    // Validate specific node types
    match value {
        NodeValue::Heading(heading) => {
            if heading.level == 0 || heading.level > 6 {
                return Err(ValidationError::InvalidProperty {
                    node_type: "heading",
                    property: "level",
                    value: heading.level.to_string(),
                });
            }
        }
        NodeValue::List(list) => {
            // Check that lists have at least one item
            if node.first_child().is_none() {
                return Err(ValidationError::MissingRequiredChild {
                    parent: "list",
                    description: "at least one list item",
                });
            }
        }
        NodeValue::Table(table) => {
            // Check that tables have at least one row
            if node.first_child().is_none() {
                return Err(ValidationError::MissingRequiredChild {
                    parent: "table",
                    description: "at least one table row",
                });
            }
            // Check column count consistency
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                let child_ast = child.data.borrow();
                if let NodeValue::TableRow(_) = &child_ast.value {
                    let cell_count = child.children().count();
                    if cell_count != table.num_columns {
                        return Err(ValidationError::InvalidProperty {
                            node_type: "table_row",
                            property: "cell_count",
                            value: format!("{} (expected {})", cell_count, table.num_columns),
                        });
                    }
                }
                child_opt = child.next_sibling();
            }
        }
        _ => {}
    }

    // Recursively validate children
    let mut child_opt = node.first_child();
    while let Some(child) = child_opt {
        validate_node(child, Some(value))?;
        child_opt = child.next_sibling();
    }

    Ok(())
}

/// Extension trait for AST nodes providing validation methods.
pub trait NodeValidationExt {
    /// Validate this node and its children.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the AST is valid, or a `ValidationError` describing the first
    /// problem found.
    fn validate(&self) -> Result<(), ValidationError>;

    /// Check if this node can contain another node type.
    ///
    /// # Arguments
    ///
    /// * `child` - The child node value to check
    ///
    /// # Returns
    ///
    /// `true` if this node can contain the given child type.
    fn can_contain(&self, child: &NodeValue) -> bool;
}

impl NodeValidationExt for AstNode<'_> {
    fn validate(&self) -> Result<(), ValidationError> {
        validate(self)
    }

    fn can_contain(&self, child: &NodeValue) -> bool {
        let ast = self.data.borrow();
        can_contain_type(&ast.value, child)
    }
}
