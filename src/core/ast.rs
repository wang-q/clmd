//! Unified AST definition for clmd.
//!
//! This module provides a unified AST type system inspired by Pandoc's Definition module.
//! It defines the core document structure that can be used across different formats.
//!
//! The AST is designed to be:
//! - **Format-agnostic**: Can represent documents from any input format
//! - **Extensible**: Easy to add new node types
//! - **Serializable**: Supports conversion to/from various formats
//!
//! # Architecture
//!
//! The AST hierarchy follows Pandoc's design:
//! - `Document` - Root container with metadata and blocks
//! - `Block` - Block-level elements (paragraphs, headers, lists, etc.)
//! - `Inline` - Inline elements (text, emphasis, links, etc.)
//! - `MetaValue` - Metadata values
//!
//! # Example
//!
//! ```ignore
//! use clmd::ast::{Document, Block, Inline, Attr};
//!
//! let doc = Document {
//!     meta: Default::default(),
//!     blocks: vec![
//!         Block::Header(1, Attr::default(), vec![
//!             Inline::Str("Hello".to_string())
//!         ]),
//!         Block::Para(vec![
//!             Inline::Str("World".to_string())
//!         ]),
//!     ],
//! };
//! ```

use std::collections::HashMap;

/// A document with metadata and content blocks.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    /// Document metadata (title, author, date, etc.)
    pub meta: MetaData,
    /// The content blocks of the document
    pub blocks: Vec<Block>,
}

/// Metadata for a document.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetaData {
    /// Map of metadata keys to values
    pub inner: HashMap<String, MetaValue>,
}

/// A metadata value.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum MetaValue {
    /// A string value
    MetaString(String),
    /// A list of metadata values
    MetaList(Vec<MetaValue>),
    /// A map of string keys to metadata values
    MetaMap(HashMap<String, MetaValue>),
    /// A boolean value
    MetaBool(bool),
    /// No value (null)
    #[default]
    MetaNull,
}

/// Attributes for elements (identifier, classes, key-value pairs).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Attr {
    /// Element identifier
    pub id: String,
    /// Element classes
    pub classes: Vec<String>,
    /// Key-value pairs
    pub attrs: Vec<(String, String)>,
}

/// Format specifier for raw blocks and inlines.
#[derive(Debug, Clone, PartialEq)]
pub enum Format {
    /// HTML format
    Html,
    /// LaTeX format
    Latex,
    /// Markdown format
    Markdown,
    /// Other format with name
    Other(String),
}

/// A block-level element.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// Plain text (not wrapped in paragraph)
    Plain(Vec<Inline>),

    /// Paragraph
    Para(Vec<Inline>),

    /// Code block with attributes and content
    CodeBlock(Attr, String),

    /// Raw block with format and content
    RawBlock(Format, String),

    /// Block quote containing blocks
    BlockQuote(Vec<Block>),

    /// Ordered list with list attributes and items
    OrderedList(ListAttributes, Vec<Vec<Block>>),

    /// Bullet list with items
    BulletList(Vec<Vec<Block>>),

    /// Definition list with terms and definitions
    DefinitionList(Vec<(Vec<Inline>, Vec<Vec<Block>>)>),

    /// Header with level, attributes, and content
    Header(u8, Attr, Vec<Inline>),

    /// Horizontal rule
    HorizontalRule,

    /// Table with attributes, caption, column alignments,
    /// relative column widths, column headers, and rows
    Table(
        Attr,
        TableCaption,
        Vec<Alignment>,
        Vec<f64>,
        Vec<TableCell>,
        Vec<Vec<TableCell>>,
    ),

    /// Div with attributes and content
    Div(Attr, Vec<Block>),

    /// Null block (placeholder)
    Null,
}

/// An inline element.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    /// Plain text
    Str(String),

    /// Emphasized text
    Emph(Vec<Inline>),

    /// Strongly emphasized text
    Strong(Vec<Inline>),

    /// Strikeout text
    Strikeout(Vec<Inline>),

    /// Superscript
    Superscript(Vec<Inline>),

    /// Subscript
    Subscript(Vec<Inline>),

    /// Small caps
    SmallCaps(Vec<Inline>),

    /// Quoted text
    Quoted(QuoteType, Vec<Inline>),

    /// Citation
    Cite(Vec<Citation>, Vec<Inline>),

    /// Code span with attributes and content
    Code(Attr, String),

    /// Space
    Space,

    /// Soft line break
    SoftBreak,

    /// Hard line break
    LineBreak,

    /// Math with type and content
    Math(MathType, String),

    /// Raw inline with format and content
    RawInline(Format, String),

    /// Link with attributes, content, target, and title
    Link(Attr, Vec<Inline>, Target),

    /// Image with attributes, alt text, target, and title
    Image(Attr, Vec<Inline>, Target),

    /// Note (footnote)
    Note(Vec<Block>),

    /// Span with attributes and content
    Span(Attr, Vec<Inline>),
}

/// Target for links and images (URL and title).
pub type Target = (String, String);

/// List attributes (start number, style, delimiter).
pub type ListAttributes = (i32, ListNumberStyle, ListNumberDelim);

/// List number style.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ListNumberStyle {
    /// Default style (decimal)
    #[default]
    DefaultStyle,
    /// Example format
    Example,
    /// Decimal numbers
    Decimal,
    /// Lowercase Roman numerals
    LowerRoman,
    /// Uppercase Roman numerals
    UpperRoman,
    /// Lowercase letters
    LowerAlpha,
    /// Uppercase letters
    UpperAlpha,
}

/// List number delimiter.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ListNumberDelim {
    /// Default delimiter (period)
    #[default]
    DefaultDelim,
    /// Period delimiter
    Period,
    /// Parenthesis delimiter
    OneParen,
    /// Both parentheses
    TwoParens,
}

/// Alignment for table cells.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Alignment {
    /// Left aligned
    #[default]
    AlignLeft,
    /// Right aligned
    AlignRight,
    /// Center aligned
    AlignCenter,
    /// Default alignment
    AlignDefault,
}

/// Table caption (optional short caption and long caption).
pub type TableCaption = (Option<Vec<Inline>>, Vec<Inline>);

/// Table cell (list of blocks).
pub type TableCell = Vec<Block>;

/// Quote type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuoteType {
    /// Double quotes
    DoubleQuote,
    /// Single quotes
    SingleQuote,
}

/// Math type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MathType {
    /// Inline math
    InlineMath,
    /// Display math
    DisplayMath,
}

/// Citation.
#[derive(Debug, Clone, PartialEq)]
pub struct Citation {
    /// Citation identifier
    pub citation_id: String,
    /// Citation prefix
    pub citation_prefix: Vec<Inline>,
    /// Citation suffix
    pub citation_suffix: Vec<Inline>,
    /// Citation mode
    pub citation_mode: CitationMode,
    /// Citation note number
    pub citation_note_num: i32,
    /// Citation hash
    pub citation_hash: i32,
}

/// Citation mode.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CitationMode {
    /// Author in parentheses
    #[default]
    AuthorInText,
    /// Suppress author
    SuppressAuthor,
    /// Normal citation
    NormalCitation,
}
