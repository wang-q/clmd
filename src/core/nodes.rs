//! The CommonMark AST.
//!
//! This module provides the core AST node types for CommonMark documents.
//! It is inspired by comrak's design, combining node values with metadata
//! into a unified structure.

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
    ThematicBreak(NodeThematicBreak),

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

    /// A shortcode emoji (e.g., `:thumbsup:` -> 👍).
    ShortCode(Box<NodeShortCode>),
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

// Size assertions to monitor type sizes (matching comrak's approach)
// These help ensure we don't accidentally bloat the node sizes
#[cfg(target_pointer_width = "64")]
const _AST_SIZE_ASSERTION: [u8; 128] = [0; std::mem::size_of::<Ast>()];

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

/// Metadata for a thematic break.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NodeThematicBreak {
    /// The character used for the thematic break (*, -, or _).
    pub marker: char,
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

/// Metadata for a shortcode emoji.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeShortCode {
    /// The shortcode name (e.g., "thumbsup").
    pub code: String,
    /// The emoji character (e.g., "👍").
    pub emoji: String,
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
                | NodeValue::ThematicBreak(..)
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
                | NodeValue::ThematicBreak(..)
                | NodeValue::Escaped
                | NodeValue::Raw(..)
                | NodeValue::FrontMatter(..)
                | NodeValue::ShortCode(..)
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
            NodeValue::ThematicBreak(..) => "thematic_break",
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
            NodeValue::ShortCode(_) => "shortcode",
        }
    }

    /// Create a Text node from a string.
    #[inline]
    pub fn make_text<S: Into<Box<str>>>(s: S) -> Self {
        NodeValue::Text(s.into())
    }

    /// Create a CodeBlock node.
    #[inline]
    pub fn code_block(code: NodeCodeBlock) -> Self {
        NodeValue::CodeBlock(Box::new(code))
    }

    /// Create an HtmlBlock node.
    #[inline]
    pub fn html_block(block: NodeHtmlBlock) -> Self {
        NodeValue::HtmlBlock(Box::new(block))
    }

    /// Create a FootnoteDefinition node.
    #[inline]
    pub fn footnote_definition(def: NodeFootnoteDefinition) -> Self {
        NodeValue::FootnoteDefinition(Box::new(def))
    }

    /// Create a Table node.
    #[inline]
    pub fn table(table: NodeTable) -> Self {
        NodeValue::Table(Box::new(table))
    }

    /// Create a Code inline node.
    #[inline]
    pub fn code(code: NodeCode) -> Self {
        NodeValue::Code(Box::new(code))
    }

    /// Create a Link node.
    #[inline]
    pub fn link(link: NodeLink) -> Self {
        NodeValue::Link(Box::new(link))
    }

    /// Create an Image node.
    #[inline]
    pub fn image(image: NodeLink) -> Self {
        NodeValue::Image(Box::new(image))
    }

    /// Create a FootnoteReference node.
    #[inline]
    pub fn footnote_reference(ref_: NodeFootnoteReference) -> Self {
        NodeValue::FootnoteReference(Box::new(ref_))
    }

    /// Create a Heading node.
    #[inline]
    pub fn heading(heading: NodeHeading) -> Self {
        NodeValue::Heading(heading)
    }

    /// Create a Math node.
    #[inline]
    pub fn math(math: NodeMath) -> Self {
        NodeValue::Math(Box::new(math))
    }

    /// Create a WikiLink node.
    #[inline]
    pub fn wiki_link(wiki: NodeWikiLink) -> Self {
        NodeValue::WikiLink(Box::new(wiki))
    }

    /// Create a MultilineBlockQuote node.
    #[inline]
    pub fn multiline_block_quote(quote: NodeMultilineBlockQuote) -> Self {
        NodeValue::MultilineBlockQuote(Box::new(quote))
    }

    /// Create an Alert node.
    #[inline]
    pub fn alert(alert: NodeAlert) -> Self {
        NodeValue::Alert(Box::new(alert))
    }
}
