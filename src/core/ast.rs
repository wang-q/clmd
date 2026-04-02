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

impl MetaData {
    /// Create empty metadata.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Get a metadata value by key.
    pub fn get(&self, key: &str) -> Option<&MetaValue> {
        self.inner.get(key)
    }

    /// Insert a metadata value.
    pub fn insert(&mut self, key: impl Into<String>, value: MetaValue) {
        self.inner.insert(key.into(), value);
    }

    /// Check if metadata is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
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

impl Attr {
    /// Create empty attributes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create attributes with an identifier.
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }

    /// Create attributes with classes.
    pub fn with_classes(classes: Vec<String>) -> Self {
        Self {
            classes,
            ..Default::default()
        }
    }

    /// Add a class.
    pub fn add_class(&mut self, class: impl Into<String>) {
        let class = class.into();
        if !self.classes.contains(&class) {
            self.classes.push(class);
        }
    }

    /// Add an attribute.
    pub fn add_attr(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attrs.push((key.into(), value.into()));
    }
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

impl Format {
    /// Get format name as string.
    pub fn as_str(&self) -> &str {
        match self {
            Format::Html => "html",
            Format::Latex => "latex",
            Format::Markdown => "markdown",
            Format::Other(s) => s.as_str(),
        }
    }
}

impl From<&str> for Format {
    fn from(s: &str) -> Self {
        // Use eq_ignore_ascii_case to avoid allocation
        if s.eq_ignore_ascii_case("html")
            || s.eq_ignore_ascii_case("html4")
            || s.eq_ignore_ascii_case("html5")
        {
            Format::Html
        } else if s.eq_ignore_ascii_case("latex") || s.eq_ignore_ascii_case("tex") {
            Format::Latex
        } else if s.eq_ignore_ascii_case("markdown") || s.eq_ignore_ascii_case("md") {
            Format::Markdown
        } else {
            Format::Other(s.to_string())
        }
    }
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

/// Trait for walking the AST.
pub trait Walkable {
    /// Walk over blocks in the element.
    fn walk_blocks<F>(&self, f: &mut F)
    where
        F: FnMut(&Block);

    /// Walk over inlines in the element.
    fn walk_inlines<F>(&self, f: &mut F)
    where
        F: FnMut(&Inline);

    /// Transform blocks in the element.
    fn transform_blocks<F>(self, f: &mut F) -> Self
    where
        F: FnMut(Block) -> Block;

    /// Transform inlines in the element.
    fn transform_inlines<F>(self, f: &mut F) -> Self
    where
        F: FnMut(Inline) -> Inline;
}

impl Walkable for Block {
    fn walk_blocks<F>(&self, f: &mut F)
    where
        F: FnMut(&Block),
    {
        f(self);
        match self {
            Block::BlockQuote(blocks) => {
                for block in blocks {
                    block.walk_blocks(f);
                }
            }
            Block::OrderedList(_, items) => {
                for item in items {
                    for block in item {
                        block.walk_blocks(f);
                    }
                }
            }
            Block::BulletList(items) => {
                for item in items {
                    for block in item {
                        block.walk_blocks(f);
                    }
                }
            }
            Block::DefinitionList(items) => {
                for (_, defs) in items {
                    for def in defs {
                        for block in def {
                            block.walk_blocks(f);
                        }
                    }
                }
            }
            Block::Div(_, blocks) => {
                for block in blocks {
                    block.walk_blocks(f);
                }
            }
            _ => {}
        }
    }

    fn walk_inlines<F>(&self, f: &mut F)
    where
        F: FnMut(&Inline),
    {
        match self {
            Block::Plain(inlines)
            | Block::Para(inlines)
            | Block::Header(_, _, inlines) => {
                for inline in inlines {
                    inline.walk_inlines(f);
                }
            }
            Block::BlockQuote(blocks) | Block::Div(_, blocks) => {
                for block in blocks {
                    block.walk_inlines(f);
                }
            }
            Block::OrderedList(_, items) => {
                for item in items {
                    for block in item {
                        block.walk_inlines(f);
                    }
                }
            }
            Block::BulletList(items) => {
                for item in items {
                    for block in item {
                        block.walk_inlines(f);
                    }
                }
            }
            Block::DefinitionList(items) => {
                for (term, defs) in items {
                    for inline in term {
                        inline.walk_inlines(f);
                    }
                    for def in defs {
                        for block in def {
                            block.walk_inlines(f);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn transform_blocks<F>(self, f: &mut F) -> Self
    where
        F: FnMut(Block) -> Block,
    {
        let transformed = match self {
            Block::BlockQuote(blocks) => Block::BlockQuote(
                blocks.into_iter().map(|b| b.transform_blocks(f)).collect(),
            ),
            Block::OrderedList(attrs, items) => Block::OrderedList(
                attrs,
                items
                    .into_iter()
                    .map(|item| {
                        item.into_iter().map(|b| b.transform_blocks(f)).collect()
                    })
                    .collect(),
            ),
            Block::BulletList(items) => Block::BulletList(
                items
                    .into_iter()
                    .map(|item| {
                        item.into_iter().map(|b| b.transform_blocks(f)).collect()
                    })
                    .collect(),
            ),
            Block::DefinitionList(items) => Block::DefinitionList(
                items
                    .into_iter()
                    .map(|(term, defs)| {
                        (
                            term,
                            defs.into_iter()
                                .map(|def| {
                                    def.into_iter()
                                        .map(|b| b.transform_blocks(f))
                                        .collect()
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            ),
            Block::Div(attrs, blocks) => Block::Div(
                attrs,
                blocks.into_iter().map(|b| b.transform_blocks(f)).collect(),
            ),
            other => other,
        };
        f(transformed)
    }

    fn transform_inlines<F>(self, f: &mut F) -> Self
    where
        F: FnMut(Inline) -> Inline,
    {
        match self {
            Block::Plain(inlines) => Block::Plain(
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
            ),
            Block::Para(inlines) => Block::Para(
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
            ),
            Block::Header(level, attrs, inlines) => Block::Header(
                level,
                attrs,
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
            ),
            other => other,
        }
    }
}

impl Walkable for Inline {
    fn walk_blocks<F>(&self, _f: &mut F)
    where
        F: FnMut(&Block),
    {
        // Inlines don't contain blocks
    }

    fn walk_inlines<F>(&self, f: &mut F)
    where
        F: FnMut(&Inline),
    {
        f(self);
        match self {
            Inline::Emph(inlines)
            | Inline::Strong(inlines)
            | Inline::Strikeout(inlines)
            | Inline::Superscript(inlines)
            | Inline::Subscript(inlines)
            | Inline::SmallCaps(inlines)
            | Inline::Quoted(_, inlines)
            | Inline::Span(_, inlines) => {
                for inline in inlines {
                    inline.walk_inlines(f);
                }
            }
            Inline::Cite(_, inlines) => {
                for inline in inlines {
                    inline.walk_inlines(f);
                }
            }
            Inline::Link(_, inlines, _) | Inline::Image(_, inlines, _) => {
                for inline in inlines {
                    inline.walk_inlines(f);
                }
            }
            Inline::Note(blocks) => {
                for block in blocks {
                    block.walk_inlines(f);
                }
            }
            _ => {}
        }
    }

    fn transform_blocks<F>(self, _f: &mut F) -> Self
    where
        F: FnMut(Block) -> Block,
    {
        self
    }

    fn transform_inlines<F>(self, f: &mut F) -> Self
    where
        F: FnMut(Inline) -> Inline,
    {
        let transformed = match self {
            Inline::Emph(inlines) => Inline::Emph(
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
            ),
            Inline::Strong(inlines) => Inline::Strong(
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
            ),
            Inline::Strikeout(inlines) => Inline::Strikeout(
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
            ),
            Inline::Superscript(inlines) => Inline::Superscript(
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
            ),
            Inline::Subscript(inlines) => Inline::Subscript(
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
            ),
            Inline::SmallCaps(inlines) => Inline::SmallCaps(
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
            ),
            Inline::Quoted(qt, inlines) => Inline::Quoted(
                qt,
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
            ),
            Inline::Cite(citations, inlines) => Inline::Cite(
                citations,
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
            ),
            Inline::Link(attrs, inlines, target) => Inline::Link(
                attrs,
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
                target,
            ),
            Inline::Image(attrs, inlines, target) => Inline::Image(
                attrs,
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
                target,
            ),
            Inline::Span(attrs, inlines) => Inline::Span(
                attrs,
                inlines
                    .into_iter()
                    .map(|i| i.transform_inlines(f))
                    .collect(),
            ),
            Inline::Note(blocks) => Inline::Note(blocks),
            other => other,
        };
        f(transformed)
    }
}

impl Walkable for Document {
    fn walk_blocks<F>(&self, f: &mut F)
    where
        F: FnMut(&Block),
    {
        for block in &self.blocks {
            block.walk_blocks(f);
        }
    }

    fn walk_inlines<F>(&self, f: &mut F)
    where
        F: FnMut(&Inline),
    {
        for block in &self.blocks {
            block.walk_inlines(f);
        }
    }

    fn transform_blocks<F>(mut self, f: &mut F) -> Self
    where
        F: FnMut(Block) -> Block,
    {
        self.blocks = self
            .blocks
            .into_iter()
            .map(|b| b.transform_blocks(f))
            .collect();
        self
    }

    fn transform_inlines<F>(mut self, f: &mut F) -> Self
    where
        F: FnMut(Inline) -> Inline,
    {
        self.blocks = self
            .blocks
            .into_iter()
            .map(|b| b.transform_inlines(f))
            .collect();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document {
            meta: MetaData::new(),
            blocks: vec![
                Block::Header(
                    1,
                    Attr::with_id("intro"),
                    vec![Inline::Str("Hello".to_string())],
                ),
                Block::Para(vec![Inline::Str("World".to_string())]),
            ],
        };

        assert_eq!(doc.blocks.len(), 2);
        assert!(doc.meta.is_empty());
    }

    #[test]
    fn test_metadata() {
        let mut meta = MetaData::new();
        meta.insert("title", MetaValue::MetaString("Test".to_string()));
        meta.insert("draft", MetaValue::MetaBool(true));

        assert_eq!(
            meta.get("title"),
            Some(&MetaValue::MetaString("Test".to_string()))
        );
        assert_eq!(meta.get("draft"), Some(&MetaValue::MetaBool(true)));
        assert!(meta.get("missing").is_none());
    }

    #[test]
    fn test_attr() {
        let mut attr = Attr::new();
        attr.add_class("highlight");
        attr.add_attr("data-id", "123");

        assert!(attr.classes.contains(&"highlight".to_string()));
        assert!(attr
            .attrs
            .contains(&("data-id".to_string(), "123".to_string())));
    }

    #[test]
    fn test_walk_blocks() {
        let doc = Document {
            meta: MetaData::new(),
            blocks: vec![
                Block::Para(vec![Inline::Str("Hello".to_string())]),
                Block::BlockQuote(vec![Block::Para(vec![Inline::Str(
                    "Quote".to_string(),
                )])]),
            ],
        };

        let mut count = 0;
        doc.walk_blocks(&mut |_block| {
            count += 1;
        });

        assert_eq!(count, 3); // Para, BlockQuote, Para inside BlockQuote
    }

    #[test]
    fn test_walk_inlines() {
        let doc = Document {
            meta: MetaData::new(),
            blocks: vec![Block::Para(vec![
                Inline::Str("Hello".to_string()),
                Inline::Emph(vec![Inline::Str("world".to_string())]),
            ])],
        };

        let mut count = 0;
        doc.walk_inlines(&mut |_inline| {
            count += 1;
        });

        assert_eq!(count, 3); // Str("Hello"), Emph, Str("world") inside Emph
    }

    #[test]
    fn test_transform_inlines() {
        let doc = Document {
            meta: MetaData::new(),
            blocks: vec![Block::Para(vec![Inline::Str("hello".to_string())])],
        };

        let transformed = doc.transform_inlines(&mut |inline| match inline {
            Inline::Str(s) => Inline::Str(s.to_uppercase()),
            other => other,
        });

        match &transformed.blocks[0] {
            Block::Para(inlines) => {
                assert_eq!(inlines[0], Inline::Str("HELLO".to_string()));
            }
            _ => panic!("Expected Para block"),
        }
    }

    #[test]
    fn test_format_from_str() {
        assert_eq!(Format::from("html"), Format::Html);
        assert_eq!(Format::from("HTML"), Format::Html);
        assert_eq!(Format::from("latex"), Format::Latex);
        assert_eq!(Format::from("markdown"), Format::Markdown);
        assert_eq!(Format::from("custom"), Format::Other("custom".to_string()));
    }

    #[test]
    fn test_list_number_style() {
        let attrs: ListAttributes =
            (1, ListNumberStyle::Decimal, ListNumberDelim::Period);
        assert_eq!(attrs.0, 1);
        assert!(matches!(attrs.1, ListNumberStyle::Decimal));
        assert!(matches!(attrs.2, ListNumberDelim::Period));
    }

    #[test]
    fn test_alignment_default() {
        let align: Alignment = Default::default();
        assert!(matches!(align, Alignment::AlignLeft));
    }

    #[test]
    fn test_citation_creation() {
        let citation = Citation {
            citation_id: "smith2020".to_string(),
            citation_prefix: vec![Inline::Str("see".to_string())],
            citation_suffix: vec![Inline::Str("p. 42".to_string())],
            citation_mode: CitationMode::NormalCitation,
            citation_note_num: 1,
            citation_hash: 12345,
        };

        assert_eq!(citation.citation_id, "smith2020");
        assert!(matches!(
            citation.citation_mode,
            CitationMode::NormalCitation
        ));
    }

    #[test]
    fn test_walk_blocks_all_variants() {
        // Test CodeBlock
        let code_block =
            Block::CodeBlock(Attr::with_id("code"), "println!(\"hello\");".to_string());
        let mut count = 0;
        code_block.walk_blocks(&mut |_block| {
            count += 1;
        });
        assert_eq!(count, 1);

        // Test RawBlock
        let raw_block = Block::RawBlock(Format::Html, "<div>test</div>".to_string());
        count = 0;
        raw_block.walk_blocks(&mut |_block| {
            count += 1;
        });
        assert_eq!(count, 1);

        // Test OrderedList
        let ordered_list = Block::OrderedList(
            (1, ListNumberStyle::Decimal, ListNumberDelim::Period),
            vec![vec![Block::Para(vec![Inline::Str("item1".to_string())])]],
        );
        count = 0;
        ordered_list.walk_blocks(&mut |_block| {
            count += 1;
        });
        assert_eq!(count, 2); // OrderedList + Para

        // Test BulletList
        let bullet_list = Block::BulletList(vec![vec![Block::Para(vec![Inline::Str(
            "item".to_string(),
        )])]]);
        count = 0;
        bullet_list.walk_blocks(&mut |_block| {
            count += 1;
        });
        assert_eq!(count, 2); // BulletList + Para

        // Test DefinitionList
        let def_list = Block::DefinitionList(vec![(
            vec![Inline::Str("term".to_string())],
            vec![vec![Block::Para(vec![Inline::Str("def".to_string())])]],
        )]);
        count = 0;
        def_list.walk_blocks(&mut |_block| {
            count += 1;
        });
        assert_eq!(count, 2); // DefinitionList + Para

        // Test Div
        let div = Block::Div(
            Attr::with_id("div"),
            vec![Block::Para(vec![Inline::Str("content".to_string())])],
        );
        count = 0;
        div.walk_blocks(&mut |_block| {
            count += 1;
        });
        assert_eq!(count, 2); // Div + Para

        // Test Null
        let null = Block::Null;
        count = 0;
        null.walk_blocks(&mut |_block| {
            count += 1;
        });
        assert_eq!(count, 1);
    }

    #[test]
    fn test_walk_inlines_all_variants() {
        // Test Strikeout
        let strikeout = Inline::Strikeout(vec![Inline::Str("deleted".to_string())]);
        let mut count = 0;
        strikeout.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 2); // Strikeout + Str

        // Test Superscript
        let superscript = Inline::Superscript(vec![Inline::Str("up".to_string())]);
        count = 0;
        superscript.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 2);

        // Test Subscript
        let subscript = Inline::Subscript(vec![Inline::Str("down".to_string())]);
        count = 0;
        subscript.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 2);

        // Test SmallCaps
        let small_caps = Inline::SmallCaps(vec![Inline::Str("caps".to_string())]);
        count = 0;
        small_caps.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 2);

        // Test Quoted
        let quoted = Inline::Quoted(
            QuoteType::DoubleQuote,
            vec![Inline::Str("quote".to_string())],
        );
        count = 0;
        quoted.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 2);

        // Test Cite
        let cite = Inline::Cite(
            vec![Citation {
                citation_id: "test".to_string(),
                citation_prefix: vec![],
                citation_suffix: vec![],
                citation_mode: CitationMode::NormalCitation,
                citation_note_num: 1,
                citation_hash: 0,
            }],
            vec![Inline::Str("citation".to_string())],
        );
        count = 0;
        cite.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 2); // Cite + Str

        // Test Math
        let math = Inline::Math(MathType::InlineMath, "x^2".to_string());
        count = 0;
        math.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 1);

        // Test RawInline
        let raw_inline = Inline::RawInline(Format::Html, "<br>".to_string());
        count = 0;
        raw_inline.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 1);

        // Test Span
        let span =
            Inline::Span(Attr::with_id("span"), vec![Inline::Str("text".to_string())]);
        count = 0;
        span.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 2);
    }

    #[test]
    fn test_transform_blocks() {
        // Test transforming blocks
        let doc = Document {
            meta: MetaData::new(),
            blocks: vec![
                Block::Para(vec![Inline::Str("hello".to_string())]),
                Block::BlockQuote(vec![Block::Para(vec![Inline::Str(
                    "quote".to_string(),
                )])]),
            ],
        };

        let transformed = doc.transform_blocks(&mut |block| match block {
            Block::Para(inlines) => Block::Header(1, Attr::default(), inlines),
            other => other,
        });

        match &transformed.blocks[0] {
            Block::Header(level, _, _) => assert_eq!(*level, 1),
            _ => panic!("Expected Header block"),
        }

        match &transformed.blocks[1] {
            Block::BlockQuote(_) => (), // Should remain as BlockQuote
            _ => panic!("Expected BlockQuote block"),
        }
    }

    #[test]
    fn test_attr_full() {
        let mut attr = Attr::new();
        attr.add_class("class1");
        attr.add_class("class2");
        attr.add_attr("data-key", "data-value");

        assert!(attr.classes.contains(&"class1".to_string()));
        assert!(attr.classes.contains(&"class2".to_string()));
        assert!(attr
            .attrs
            .contains(&("data-key".to_string(), "data-value".to_string())));

        // Test with_id
        let attr_with_id = Attr::with_id("myid");
        assert_eq!(attr_with_id.id, "myid");

        // Test with_classes
        let attr_with_classes =
            Attr::with_classes(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(attr_with_classes.classes.len(), 2);
    }

    #[test]
    fn test_meta_value_all_variants() {
        // Test MetaString
        let meta = MetaValue::MetaString("test".to_string());
        assert!(matches!(meta, MetaValue::MetaString(_)));

        // Test MetaList
        let meta_list = MetaValue::MetaList(vec![
            MetaValue::MetaString("a".to_string()),
            MetaValue::MetaString("b".to_string()),
        ]);
        assert!(matches!(meta_list, MetaValue::MetaList(_)));

        // Test MetaMap
        let mut map = std::collections::HashMap::new();
        map.insert(
            "key".to_string(),
            MetaValue::MetaString("value".to_string()),
        );
        let meta_map = MetaValue::MetaMap(map);
        assert!(matches!(meta_map, MetaValue::MetaMap(_)));

        // Test MetaBool
        let meta_bool = MetaValue::MetaBool(true);
        assert!(matches!(meta_bool, MetaValue::MetaBool(true)));

        // Test MetaNull
        let meta_null = MetaValue::MetaNull;
        assert!(matches!(meta_null, MetaValue::MetaNull));
    }

    #[test]
    fn test_format_all_variants() {
        // Test Html
        let html = Format::Html;
        assert_eq!(html.as_str(), "html");

        // Test Latex
        let latex = Format::Latex;
        assert_eq!(latex.as_str(), "latex");

        // Test Markdown
        let markdown = Format::Markdown;
        assert_eq!(markdown.as_str(), "markdown");

        // Test Other
        let other = Format::Other("custom".to_string());
        assert_eq!(other.as_str(), "custom");
    }

    #[test]
    fn test_nested_structure_walk() {
        let doc = Document {
            meta: MetaData::new(),
            blocks: vec![Block::BlockQuote(vec![Block::OrderedList(
                (1, ListNumberStyle::Decimal, ListNumberDelim::Period),
                vec![vec![Block::Para(vec![Inline::Emph(vec![Inline::Str(
                    "deep".to_string(),
                )])])]],
            )])],
        };

        let mut block_count = 0;
        let mut inline_count = 0;

        doc.walk_blocks(&mut |_block| {
            block_count += 1;
        });

        doc.walk_inlines(&mut |_inline| {
            inline_count += 1;
        });

        assert_eq!(block_count, 3); // BlockQuote + OrderedList + Para
        assert_eq!(inline_count, 2); // Emph + Str
    }

    #[test]
    fn test_list_number_style_variants() {
        let styles = vec![
            ListNumberStyle::DefaultStyle,
            ListNumberStyle::Example,
            ListNumberStyle::Decimal,
            ListNumberStyle::LowerRoman,
            ListNumberStyle::UpperRoman,
            ListNumberStyle::LowerAlpha,
            ListNumberStyle::UpperAlpha,
        ];

        for style in styles {
            let attrs: ListAttributes = (1, style, ListNumberDelim::Period);
            assert_eq!(attrs.0, 1);
        }
    }

    #[test]
    fn test_list_number_delim_variants() {
        let delims = vec![
            ListNumberDelim::DefaultDelim,
            ListNumberDelim::Period,
            ListNumberDelim::OneParen,
            ListNumberDelim::TwoParens,
        ];

        for delim in delims {
            let attrs: ListAttributes = (1, ListNumberStyle::Decimal, delim);
            assert!(matches!(attrs.2, _));
        }
    }

    #[test]
    fn test_alignment_variants() {
        let alignments = vec![
            Alignment::AlignLeft,
            Alignment::AlignRight,
            Alignment::AlignCenter,
            Alignment::AlignDefault,
        ];

        for align in alignments {
            assert!(matches!(
                align,
                Alignment::AlignLeft
                    | Alignment::AlignRight
                    | Alignment::AlignCenter
                    | Alignment::AlignDefault
            ));
        }
    }

    #[test]
    fn test_quote_type_variants() {
        let double = QuoteType::DoubleQuote;
        let single = QuoteType::SingleQuote;

        assert!(matches!(double, QuoteType::DoubleQuote));
        assert!(matches!(single, QuoteType::SingleQuote));
    }

    #[test]
    fn test_math_type_variants() {
        let inline = MathType::InlineMath;
        let display = MathType::DisplayMath;

        assert!(matches!(inline, MathType::InlineMath));
        assert!(matches!(display, MathType::DisplayMath));
    }

    #[test]
    fn test_citation_mode_variants() {
        let modes = vec![
            CitationMode::AuthorInText,
            CitationMode::SuppressAuthor,
            CitationMode::NormalCitation,
        ];

        for mode in modes {
            assert!(matches!(
                mode,
                CitationMode::AuthorInText
                    | CitationMode::SuppressAuthor
                    | CitationMode::NormalCitation
            ));
        }
    }

    #[test]
    fn test_metadata_insert_and_get() {
        let mut meta = MetaData::new();

        meta.insert("title", MetaValue::MetaString("My Title".to_string()));
        meta.insert("count", MetaValue::MetaBool(true));

        assert_eq!(
            meta.get("title"),
            Some(&MetaValue::MetaString("My Title".to_string()))
        );
        assert_eq!(meta.get("count"), Some(&MetaValue::MetaBool(true)));
        assert!(meta.get("missing").is_none());
        assert!(!meta.is_empty());
    }

    #[test]
    fn test_block_table() {
        let table = Block::Table(
            Attr::default(),
            (None, vec![]),
            vec![Alignment::AlignLeft],
            vec![1.0],
            vec![vec![Block::Para(vec![])]],
            vec![vec![vec![Block::Para(vec![])]]],
        );

        let mut count = 0;
        table.walk_blocks(&mut |_block| {
            count += 1;
        });
        assert!(count > 0);
    }

    #[test]
    fn test_inline_link_and_image() {
        // Test Link
        let link = Inline::Link(
            Attr::default(),
            vec![Inline::Str("link text".to_string())],
            ("https://example.com".to_string(), "title".to_string()),
        );

        let mut count = 0;
        link.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 2); // Link + Str

        // Test Image
        let image = Inline::Image(
            Attr::default(),
            vec![Inline::Str("alt text".to_string())],
            ("image.png".to_string(), "image title".to_string()),
        );

        count = 0;
        image.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 2); // Image + Str
    }

    #[test]
    fn test_inline_note() {
        let note =
            Inline::Note(vec![Block::Para(vec![Inline::Str("footnote".to_string())])]);

        let mut count = 0;
        note.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 2); // Note doesn't call f on itself, but walks Para + Str
    }

    #[test]
    fn test_block_plain() {
        let plain = Block::Plain(vec![Inline::Str("plain text".to_string())]);

        let mut count = 0;
        plain.walk_inlines(&mut |_inline| {
            count += 1;
        });
        assert_eq!(count, 1);
    }

    #[test]
    fn test_block_horizontal_rule() {
        let hr = Block::HorizontalRule;

        let mut count = 0;
        hr.walk_blocks(&mut |_block| {
            count += 1;
        });
        assert_eq!(count, 1);
    }

    #[test]
    fn test_document_default() {
        let doc: Document = Default::default();
        assert!(doc.meta.is_empty());
        assert!(doc.blocks.is_empty());
    }

    #[test]
    fn test_target_type() {
        let target: Target = ("url".to_string(), "title".to_string());
        assert_eq!(target.0, "url");
        assert_eq!(target.1, "title");
    }

    #[test]
    fn test_table_caption() {
        let caption: TableCaption = (
            Some(vec![Inline::Str("short".to_string())]),
            vec![Inline::Str("long caption".to_string())],
        );
        assert!(caption.0.is_some());
        assert!(!caption.1.is_empty());
    }

    #[test]
    fn test_table_cell() {
        let cell: TableCell =
            vec![Block::Para(vec![Inline::Str("cell content".to_string())])];
        assert_eq!(cell.len(), 1);
    }
}
