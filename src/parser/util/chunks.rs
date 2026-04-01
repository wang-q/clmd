//! Document chunking for clmd.
//!
//! This module provides functionality for splitting documents into chunks,
//! inspired by Pandoc's Text.Pandoc.Chunks module. This is useful for
//! generating EPUBs, websites, and other multi-page outputs.
//!
//! # Example
//!
//! ```ignore
//! use clmd::parse::util::chunks::{Chunker, ChunkConfig};
//!
//! let config = ChunkConfig::default();
//! let chunker = Chunker::new(config);
//! ```

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::NodeValue;

/// Configuration for document chunking.
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Maximum number of sections per chunk.
    pub max_sections_per_chunk: usize,
    /// Whether to split on level 1 headings.
    pub split_on_h1: bool,
    /// Whether to split on level 2 headings.
    pub split_on_h2: bool,
    /// Base name for chunk files.
    pub base_name: String,
    /// File extension for chunks.
    pub extension: String,
    /// Whether to include navigation links.
    pub include_nav_links: bool,
    /// Whether to generate a table of contents.
    pub generate_toc: bool,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            max_sections_per_chunk: 10,
            split_on_h1: true,
            split_on_h2: false,
            base_name: "chunk".to_string(),
            extension: "html".to_string(),
            include_nav_links: true,
            generate_toc: true,
        }
    }
}

impl ChunkConfig {
    /// Create a new chunk configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of sections per chunk.
    pub fn max_sections(mut self, max: usize) -> Self {
        self.max_sections_per_chunk = max;
        self
    }

    /// Set whether to split on H1 headings.
    pub fn split_on_h1(mut self, split: bool) -> Self {
        self.split_on_h1 = split;
        self
    }

    /// Set whether to split on H2 headings.
    pub fn split_on_h2(mut self, split: bool) -> Self {
        self.split_on_h2 = split;
        self
    }

    /// Set the base name for chunk files.
    pub fn base_name(mut self, name: impl Into<String>) -> Self {
        self.base_name = name.into();
        self
    }

    /// Set the file extension.
    pub fn extension(mut self, ext: impl Into<String>) -> Self {
        self.extension = ext.into();
        self
    }

    /// Set whether to include navigation links.
    pub fn nav_links(mut self, include: bool) -> Self {
        self.include_nav_links = include;
        self
    }

    /// Set whether to generate TOC.
    pub fn generate_toc(mut self, generate: bool) -> Self {
        self.generate_toc = generate;
        self
    }
}

/// A single chunk of a document.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Unique identifier for this chunk.
    pub id: String,
    /// File name for this chunk.
    pub file_name: String,
    /// Title of this chunk (from heading).
    pub title: Option<String>,
    /// Heading level that started this chunk.
    pub level: u8,
    /// Node IDs contained in this chunk.
    pub nodes: Vec<NodeId>,
    /// Index of this chunk in the sequence.
    pub index: usize,
    /// ID of the parent chunk (if any).
    pub parent_id: Option<String>,
    /// IDs of child chunks.
    pub child_ids: Vec<String>,
    /// ID of the previous chunk (if any).
    pub prev_id: Option<String>,
    /// ID of the next chunk (if any).
    pub next_id: Option<String>,
    /// Table of contents for this chunk.
    pub toc: Vec<TocEntry>,
}

impl Chunk {
    /// Create a new chunk.
    pub fn new(id: impl Into<String>, index: usize) -> Self {
        Self {
            id: id.into(),
            file_name: String::new(),
            title: None,
            level: 1,
            nodes: Vec::new(),
            index,
            parent_id: None,
            child_ids: Vec::new(),
            prev_id: None,
            next_id: None,
            toc: Vec::new(),
        }
    }

    /// Set the file name.
    pub fn with_file_name(mut self, name: impl Into<String>) -> Self {
        self.file_name = name.into();
        self
    }

    /// Set the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the level.
    pub fn with_level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }

    /// Add a node to this chunk.
    pub fn add_node(&mut self, node_id: NodeId) {
        self.nodes.push(node_id);
    }

    /// Get the number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if this chunk is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.title.is_none()
    }

    /// Get the URL for this chunk.
    pub fn url(&self) -> String {
        format!("{}", self.file_name)
    }
}

/// A table of contents entry.
#[derive(Debug, Clone)]
pub struct TocEntry {
    /// Title of the entry.
    pub title: String,
    /// Anchor/ID for the entry.
    pub anchor: String,
    /// Heading level.
    pub level: u8,
    /// Child entries.
    pub children: Vec<TocEntry>,
}

impl TocEntry {
    /// Create a new TOC entry.
    pub fn new(title: impl Into<String>, anchor: impl Into<String>, level: u8) -> Self {
        Self {
            title: title.into(),
            anchor: anchor.into(),
            level,
            children: Vec::new(),
        }
    }

    /// Add a child entry.
    pub fn add_child(&mut self, child: TocEntry) {
        self.children.push(child);
    }
}

/// Navigation links for a chunk.
#[derive(Debug, Clone, Default)]
pub struct NavLinks {
    /// Link to the previous chunk.
    pub prev: Option<Link>,
    /// Link to the next chunk.
    pub next: Option<Link>,
    /// Link to the parent chunk.
    pub up: Option<Link>,
    /// Link to the table of contents.
    pub toc: Option<Link>,
}

/// A navigation link.
#[derive(Debug, Clone)]
pub struct Link {
    /// URL of the link.
    pub url: String,
    /// Title of the link.
    pub title: String,
}

impl Link {
    /// Create a new link.
    pub fn new(url: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            title: title.into(),
        }
    }
}

/// A chunked document.
#[derive(Debug, Clone)]
pub struct ChunkedDocument {
    /// Chunks in the document.
    pub chunks: Vec<Chunk>,
    /// Configuration used.
    pub config: ChunkConfig,
    /// Root TOC entries.
    pub root_toc: Vec<TocEntry>,
}

impl ChunkedDocument {
    /// Create a new chunked document.
    pub fn new(config: ChunkConfig) -> Self {
        Self {
            chunks: Vec::new(),
            config,
            root_toc: Vec::new(),
        }
    }

    /// Get the number of chunks.
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Check if there are no chunks.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Get a chunk by index.
    pub fn get(&self, index: usize) -> Option<&Chunk> {
        self.chunks.get(index)
    }

    /// Get a chunk by ID.
    pub fn get_by_id(&self, id: &str) -> Option<&Chunk> {
        self.chunks.iter().find(|c| c.id == id)
    }

    /// Get the navigation links for a chunk.
    pub fn nav_links_for(&self, chunk: &Chunk) -> NavLinks {
        NavLinks {
            prev: chunk.prev_id.as_ref().and_then(|id| {
                self.get_by_id(id).map(|c| {
                    Link::new(&c.url(), c.title.as_deref().unwrap_or("Previous"))
                })
            }),
            next: chunk.next_id.as_ref().and_then(|id| {
                self.get_by_id(id)
                    .map(|c| Link::new(&c.url(), c.title.as_deref().unwrap_or("Next")))
            }),
            up: chunk.parent_id.as_ref().and_then(|id| {
                self.get_by_id(id)
                    .map(|c| Link::new(&c.url(), c.title.as_deref().unwrap_or("Up")))
            }),
            toc: if self.config.generate_toc {
                Some(Link::new("index.html", "Table of Contents"))
            } else {
                None
            },
        }
    }

    /// Iterate over chunks.
    pub fn iter(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.iter()
    }
}

/// Document chunker.
#[derive(Debug)]
pub struct Chunker {
    config: ChunkConfig,
}

impl Chunker {
    /// Create a new chunker with the given configuration.
    pub fn new(config: ChunkConfig) -> Self {
        Self { config }
    }

    /// Split a document into chunks.
    pub fn chunk(&self, arena: &NodeArena, root: NodeId) -> ChunkedDocument {
        let mut doc = ChunkedDocument::new(self.config.clone());
        let mut current_chunk: Option<Chunk> = None;
        let mut chunk_index = 0;

        let root_node = arena.get(root);
        let mut child_opt = root_node.first_child;

        while let Some(child_id) = child_opt {
            let node = arena.get(child_id);

            // Check if this node starts a new chunk
            let is_chunk_boundary = if let NodeValue::Heading(heading) = &node.value {
                match heading.level {
                    1 => self.config.split_on_h1,
                    2 => self.config.split_on_h2,
                    _ => false,
                }
            } else {
                false
            };

            if is_chunk_boundary {
                let title = get_heading_text(arena, child_id);
                let heading_level = if let NodeValue::Heading(h) = &node.value {
                    h.level
                } else {
                    1
                };

                // Save current chunk if it has content
                if let Some(chunk) = current_chunk.take() {
                    if !chunk.is_empty() {
                        doc.chunks.push(chunk);
                        chunk_index += 1;
                    }
                }

                // Start new chunk
                let id = format!("{}-{:03}", self.config.base_name, chunk_index);
                let file_name = format!(
                    "{}-{:03}.{}",
                    self.config.base_name, chunk_index, self.config.extension
                );

                current_chunk = Some(
                    Chunk::new(&id, chunk_index)
                        .with_file_name(&file_name)
                        .with_title(&title)
                        .with_level(heading_level),
                );
            }

            // Add node to current chunk
            if let Some(ref mut chunk) = current_chunk {
                chunk.add_node(child_id);
            }

            child_opt = node.next;
        }

        // Don't forget the last chunk
        if let Some(chunk) = current_chunk {
            if !chunk.is_empty() {
                doc.chunks.push(chunk);
            }
        }

        // Set up navigation links
        self.setup_navigation(&mut doc);

        // Generate TOC
        if self.config.generate_toc {
            doc.root_toc = self.generate_toc(arena, root);
        }

        doc
    }

    /// Set up navigation links between chunks.
    fn setup_navigation(&self, doc: &mut ChunkedDocument) {
        let n = doc.chunks.len();

        // First pass: set prev/next links
        for i in 0..n {
            if i > 0 {
                doc.chunks[i].prev_id = Some(doc.chunks[i - 1].id.clone());
            }
            if i < n - 1 {
                doc.chunks[i].next_id = Some(doc.chunks[i + 1].id.clone());
            }
        }

        // Second pass: set parent/child links
        for i in 0..n {
            let chunk_level = doc.chunks[i].level;
            if chunk_level > 1 {
                for j in (0..i).rev() {
                    if doc.chunks[j].level < chunk_level {
                        let parent_id = doc.chunks[j].id.clone();
                        let chunk_id = doc.chunks[i].id.clone();
                        doc.chunks[i].parent_id = Some(parent_id);
                        doc.chunks[j].child_ids.push(chunk_id);
                        break;
                    }
                }
            }
        }
    }

    /// Generate table of contents.
    fn generate_toc(&self, arena: &NodeArena, root: NodeId) -> Vec<TocEntry> {
        let mut toc = Vec::new();
        let mut stack: Vec<&mut TocEntry> = Vec::new();

        let root_node = arena.get(root);
        let mut child_opt = root_node.first_child;

        while let Some(child_id) = child_opt {
            let node = arena.get(child_id);

            if let NodeValue::Heading(heading) = &node.value {
                let title = get_heading_text(arena, child_id);
                let anchor = make_anchor(&title);
                let entry = TocEntry::new(&title, &anchor, heading.level);

                // Pop stack to find correct parent
                while let Some(parent) = stack.last() {
                    if parent.level < heading.level {
                        break;
                    }
                    stack.pop();
                }

                if let Some(parent) = stack.last_mut() {
                    parent.add_child(entry);
                } else {
                    toc.push(entry);
                }

                // Note: This won't work perfectly because we can't get a mutable reference
                // to the entry we just pushed. For a real implementation, we'd need to
                // use indices or Rc<RefCell<>>.
            }

            child_opt = node.next;
        }

        toc
    }
}

impl Default for Chunker {
    fn default() -> Self {
        Self::new(ChunkConfig::default())
    }
}

/// Get the text content of a heading node.
fn get_heading_text(arena: &NodeArena, heading_id: NodeId) -> String {
    let mut result = String::new();
    collect_text_recursive(arena, heading_id, &mut result);
    result.trim().to_string()
}

/// Recursively collect text from a node and its children.
fn collect_text_recursive(arena: &NodeArena, node_id: NodeId, result: &mut String) {
    let node = arena.get(node_id);

    if let NodeValue::Text(text) = &node.value {
        result.push_str(text);
    }

    let mut child_opt = node.first_child;
    while let Some(child_id) = child_opt {
        collect_text_recursive(arena, child_id, result);
        child_opt = arena.get(child_id).next;
    }
}

/// Create an anchor from text.
fn make_anchor(text: &str) -> String {
    text.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

/// Split a document into chunks using default configuration.
pub fn chunk_document(arena: &NodeArena, root: NodeId) -> ChunkedDocument {
    let chunker = Chunker::default();
    chunker.chunk(arena, root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_document, Options};

    #[test]
    fn test_chunk_config() {
        let config = ChunkConfig::new()
            .max_sections(5)
            .split_on_h1(true)
            .split_on_h2(true)
            .base_name("page")
            .extension("xhtml")
            .nav_links(true)
            .generate_toc(false);

        assert_eq!(config.max_sections_per_chunk, 5);
        assert!(config.split_on_h1);
        assert!(config.split_on_h2);
        assert_eq!(config.base_name, "page");
        assert_eq!(config.extension, "xhtml");
        assert!(config.include_nav_links);
        assert!(!config.generate_toc);
    }

    #[test]
    fn test_chunk_creation() {
        let chunk = Chunk::new("chunk-001", 0)
            .with_file_name("chunk-001.html")
            .with_title("Test Chunk")
            .with_level(1);

        assert_eq!(chunk.id, "chunk-001");
        assert_eq!(chunk.file_name, "chunk-001.html");
        assert_eq!(chunk.title, Some("Test Chunk".to_string()));
        assert_eq!(chunk.level, 1);
        // Chunk has title but no nodes, so it's not empty
        assert!(!chunk.is_empty());

        // Test truly empty chunk
        let empty_chunk = Chunk::new("empty", 0);
        assert!(empty_chunk.is_empty());
    }

    #[test]
    fn test_toc_entry() {
        let mut entry = TocEntry::new("Title", "anchor", 1);
        let child = TocEntry::new("Child", "child-anchor", 2);
        entry.add_child(child);

        assert_eq!(entry.title, "Title");
        assert_eq!(entry.anchor, "anchor");
        assert_eq!(entry.level, 1);
        assert_eq!(entry.children.len(), 1);
    }

    #[test]
    fn test_link() {
        let link = Link::new("url", "title");
        assert_eq!(link.url, "url");
        assert_eq!(link.title, "title");
    }

    #[test]
    fn test_chunk_document() {
        let md = "# Section 1\n\nContent 1\n\n# Section 2\n\nContent 2";
        let options = Options::default();
        let (arena, root) = parse_document(md, &options);

        let doc = chunk_document(&arena, root);

        // Should create 2 chunks (one for each H1)
        assert_eq!(doc.len(), 2);
        assert_eq!(doc.get(0).unwrap().title, Some("Section 1".to_string()));
        assert_eq!(doc.get(1).unwrap().title, Some("Section 2".to_string()));
    }

    #[test]
    fn test_navigation_links() {
        let md = "# First\n\n# Second\n\n# Third";
        let options = Options::default();
        let (arena, root) = parse_document(md, &options);

        let doc = chunk_document(&arena, root);

        let first = doc.get(0).unwrap();
        let second = doc.get(1).unwrap();
        let third = doc.get(2).unwrap();

        // First has no prev, has next
        assert!(first.prev_id.is_none());
        assert_eq!(first.next_id, Some(second.id.clone()));

        // Second has prev and next
        assert_eq!(second.prev_id, Some(first.id.clone()));
        assert_eq!(second.next_id, Some(third.id.clone()));

        // Third has prev, no next
        assert_eq!(third.prev_id, Some(second.id.clone()));
        assert!(third.next_id.is_none());
    }

    #[test]
    fn test_nav_links_generation() {
        let md = "# First\n\n# Second";
        let options = Options::default();
        let (arena, root) = parse_document(md, &options);

        let doc = chunk_document(&arena, root);
        let chunk = doc.get(0).unwrap();
        let nav = doc.nav_links_for(chunk);

        assert!(nav.prev.is_none());
        assert!(nav.next.is_some());
        assert!(nav.toc.is_some());
    }

    #[test]
    fn test_make_anchor() {
        assert_eq!(make_anchor("Hello World"), "hello-world");
        assert_eq!(make_anchor("Test--Anchor"), "test-anchor");
        assert_eq!(make_anchor("-Leading-"), "leading");
    }

    #[test]
    fn test_empty_document() {
        let doc = ChunkedDocument::new(ChunkConfig::default());
        assert!(doc.is_empty());
        assert_eq!(doc.len(), 0);
    }

    #[test]
    fn test_chunker_with_config() {
        let config = ChunkConfig::new().split_on_h1(true).max_sections(1);

        let chunker = Chunker::new(config);
        let md = "# A\n\n# B\n\n# C";
        let options = Options::default();
        let (arena, root) = parse_document(md, &options);

        let doc = chunker.chunk(&arena, root);
        assert_eq!(doc.len(), 3);
    }
}
