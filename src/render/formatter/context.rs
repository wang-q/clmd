//! Node formatter context trait definitions
//!
//! This module defines the context trait for node formatters,
//! inspired by flexmark-java's NodeFormatterContext interface.

use crate::arena::{NodeArena, NodeId};
use crate::nodes::NodeValue;
use crate::render::formatter::node::NodeValueType;
use crate::render::formatter::options::FormatterOptions;
use crate::render::formatter::phase::FormattingPhase;
use crate::render::formatter::purpose::RenderPurpose;
use crate::render::formatter::writer::MarkdownWriter;

/// Context for node formatting operations
///
/// This trait provides the interface that node formatters use to
/// interact with the formatting system, including rendering nodes,
/// accessing configuration, and managing output.
pub trait NodeFormatterContext {
    /// Get the Markdown writer for output
    fn get_markdown_writer(&mut self) -> &mut MarkdownWriter;

    /// Render a specific node
    ///
    /// This should be used to render child nodes. Be careful not to
    /// pass the node that is currently being rendered, as that would
    /// result in infinite recursion.
    fn render(&mut self, node_id: NodeId);

    /// Render the children of a node
    ///
    /// Renders all child nodes of the given parent node.
    fn render_children(&mut self, node_id: NodeId);

    /// Get the current formatting phase
    fn get_formatting_phase(&self) -> FormattingPhase;

    /// Delegate rendering to the next handler
    ///
    /// This allows a formatter to pass rendering to another handler
    /// registered for the same node type.
    fn delegate_render(&mut self);

    /// Get the formatter options
    fn get_formatter_options(&self) -> &FormatterOptions;

    /// Get the current render purpose
    fn get_render_purpose(&self) -> RenderPurpose;

    /// Check if text transformation is active
    ///
    /// Returns true when rendering for translation purposes.
    fn is_transforming_text(&self) -> bool {
        self.get_render_purpose().is_transforming_text()
    }

    /// Get the node arena
    fn get_arena(&self) -> &NodeArena;

    /// Get the current node being rendered
    fn get_current_node(&self) -> Option<NodeId>;

    /// Get the parent of the current node
    fn get_current_node_parent(&self) -> Option<NodeId> {
        self.get_current_node()
            .and_then(|id| self.get_arena().get(id).parent)
    }

    /// Get the current node's value
    fn get_current_node_value(&self) -> Option<&NodeValue> {
        self.get_current_node()
            .map(|id| &self.get_arena().get(id).value)
    }

    /// Get nodes of a specific type
    ///
    /// Returns an iterator over all nodes of the given type in the document,
    /// in depth-first order.
    fn get_nodes_of_type(&self, node_type: NodeValueType) -> Vec<NodeId>;

    // Table data collection methods

    /// Start collecting table data
    ///
    /// Called when entering a table node to begin collecting row and cell data.
    fn start_table_collection(&mut self, alignments: Vec<crate::nodes::TableAlignment>);

    /// Add a table row
    ///
    /// Called when entering a table row.
    fn add_table_row(&mut self);

    /// Add a table cell
    ///
    /// Called when rendering a table cell with its content.
    fn add_table_cell(&mut self, content: String);

    /// Get collected table data and clear it
    ///
    /// Called when exiting a table node to get all collected data for formatting.
    fn take_table_data(&mut self) -> Option<(Vec<Vec<String>>, Vec<crate::nodes::TableAlignment>)>;

    /// Check if we're currently collecting table data
    fn is_collecting_table(&self) -> bool;

    /// Set whether to skip rendering children (for table cells)
    fn set_skip_children(&mut self, skip: bool);

    /// Render children to a string and return the content
    ///
    /// This is used to capture the rendered output of child nodes
    /// without writing to the main output.
    fn render_children_to_string(&mut self, node_id: NodeId) -> String;

    /// Get nodes of multiple types
    fn get_nodes_of_types(&self, node_types: &[NodeValueType]) -> Vec<NodeId>;

    /// Get the block quote-like prefix predicate
    ///
    /// Returns a function that checks if a character is a block quote-like prefix.
    fn get_block_quote_like_prefix_predicate(&self) -> Box<dyn Fn(char) -> bool>;

    /// Get the block quote-like prefix characters
    fn get_block_quote_like_prefix_chars(&self) -> &str;

    /// Transform non-translating text
    ///
    /// Used for text that should not be translated (e.g., URLs, code).
    fn transform_non_translating(&self, text: &str) -> String;

    /// Transform translating text
    ///
    /// Used for text that should be translated.
    fn transform_translating(&self, text: &str) -> String;

    /// Create a sub-context for nested rendering
    fn create_sub_context(&self) -> Box<dyn NodeFormatterContext>;

    /// Check if we're currently in a tight list context
    fn is_in_tight_list(&self) -> bool;

    /// Set the tight list context
    fn set_tight_list(&mut self, tight: bool);

    /// Get the current list nesting level
    fn get_list_nesting_level(&self) -> usize;

    /// Increment the list nesting level
    fn increment_list_nesting(&mut self);

    /// Decrement the list nesting level
    fn decrement_list_nesting(&mut self);

    /// Check if we're in a block quote context
    fn is_in_block_quote(&self) -> bool;

    /// Set the block quote context
    fn set_in_block_quote(&mut self, in_block_quote: bool);

    /// Get the current block quote nesting level
    fn get_block_quote_nesting_level(&self) -> usize;

    /// Increment the block quote nesting level
    fn increment_block_quote_nesting(&mut self);

    /// Decrement the block quote nesting level
    fn decrement_block_quote_nesting(&mut self);
}

/// A sub-context for nested formatting operations
pub struct SubFormatterContext<'a> {
    /// Reference to the parent context
    parent: &'a mut dyn NodeFormatterContext,
    /// The Markdown writer
    markdown: MarkdownWriter,
    /// The current node being rendered
    current_node: Option<NodeId>,
    /// Whether we're in a tight list
    tight_list: bool,
    /// List nesting level
    list_nesting: usize,
    /// Whether we're in a block quote
    in_block_quote: bool,
    /// Block quote nesting level
    block_quote_nesting: usize,
}

impl<'a> std::fmt::Debug for SubFormatterContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubFormatterContext")
            .field("current_node", &self.current_node)
            .field("tight_list", &self.tight_list)
            .field("list_nesting", &self.list_nesting)
            .field("in_block_quote", &self.in_block_quote)
            .field("block_quote_nesting", &self.block_quote_nesting)
            .finish_non_exhaustive()
    }
}

impl<'a> SubFormatterContext<'a> {
    /// Create a new sub-context
    pub fn new(parent: &'a mut dyn NodeFormatterContext) -> Self {
        let markdown = MarkdownWriter::new(parent.get_formatter_options().format_flags);
        Self {
            parent,
            markdown,
            current_node: None,
            tight_list: false,
            list_nesting: 0,
            in_block_quote: false,
            block_quote_nesting: 0,
        }
    }

    /// Create a new sub-context with a specific writer
    pub fn with_writer(
        parent: &'a mut dyn NodeFormatterContext,
        markdown: MarkdownWriter,
    ) -> Self {
        Self {
            parent,
            markdown,
            current_node: None,
            tight_list: false,
            list_nesting: 0,
            in_block_quote: false,
            block_quote_nesting: 0,
        }
    }

    /// Get the Markdown writer
    pub fn get_writer(&self) -> &MarkdownWriter {
        &self.markdown
    }

    /// Get the Markdown writer mutably
    pub fn get_writer_mut(&mut self) -> &mut MarkdownWriter {
        &mut self.markdown
    }
}

impl<'a> NodeFormatterContext for SubFormatterContext<'a> {
    fn get_markdown_writer(&mut self) -> &mut MarkdownWriter {
        &mut self.markdown
    }

    fn render(&mut self, node_id: NodeId) {
        self.parent.render(node_id);
    }

    fn render_children(&mut self, node_id: NodeId) {
        self.parent.render_children(node_id);
    }

    fn get_formatting_phase(&self) -> FormattingPhase {
        self.parent.get_formatting_phase()
    }

    fn delegate_render(&mut self) {
        self.parent.delegate_render();
    }

    fn get_formatter_options(&self) -> &FormatterOptions {
        self.parent.get_formatter_options()
    }

    fn get_render_purpose(&self) -> RenderPurpose {
        self.parent.get_render_purpose()
    }

    fn get_arena(&self) -> &NodeArena {
        self.parent.get_arena()
    }

    fn get_current_node(&self) -> Option<NodeId> {
        self.current_node.or_else(|| self.parent.get_current_node())
    }

    fn get_nodes_of_type(&self, node_type: NodeValueType) -> Vec<NodeId> {
        self.parent.get_nodes_of_type(node_type)
    }

    fn get_nodes_of_types(&self, node_types: &[NodeValueType]) -> Vec<NodeId> {
        self.parent.get_nodes_of_types(node_types)
    }

    fn get_block_quote_like_prefix_predicate(&self) -> Box<dyn Fn(char) -> bool> {
        self.parent.get_block_quote_like_prefix_predicate()
    }

    fn get_block_quote_like_prefix_chars(&self) -> &str {
        self.parent.get_block_quote_like_prefix_chars()
    }

    fn transform_non_translating(&self, text: &str) -> String {
        self.parent.transform_non_translating(text)
    }

    fn transform_translating(&self, text: &str) -> String {
        self.parent.transform_translating(text)
    }

    fn create_sub_context(&self) -> Box<dyn NodeFormatterContext> {
        // Cannot create a sub-context from a sub-context
        panic!("Cannot create nested sub-contexts");
    }

    fn is_in_tight_list(&self) -> bool {
        self.tight_list
    }

    fn set_tight_list(&mut self, tight: bool) {
        self.tight_list = tight;
    }

    fn get_list_nesting_level(&self) -> usize {
        self.list_nesting
    }

    fn increment_list_nesting(&mut self) {
        self.list_nesting += 1;
    }

    fn decrement_list_nesting(&mut self) {
        if self.list_nesting > 0 {
            self.list_nesting -= 1;
        }
    }

    fn is_in_block_quote(&self) -> bool {
        self.in_block_quote
    }

    fn set_in_block_quote(&mut self, in_block_quote: bool) {
        self.in_block_quote = in_block_quote;
    }

    fn get_block_quote_nesting_level(&self) -> usize {
        self.block_quote_nesting
    }

    fn increment_block_quote_nesting(&mut self) {
        self.block_quote_nesting += 1;
    }

    fn decrement_block_quote_nesting(&mut self) {
        if self.block_quote_nesting > 0 {
            self.block_quote_nesting -= 1;
        }
    }

    // Table data collection methods - delegate to parent

    fn start_table_collection(&mut self, alignments: Vec<crate::nodes::TableAlignment>) {
        self.parent.start_table_collection(alignments);
    }

    fn add_table_row(&mut self) {
        self.parent.add_table_row();
    }

    fn add_table_cell(&mut self, content: String) {
        self.parent.add_table_cell(content);
    }

    fn take_table_data(&mut self) -> Option<(Vec<Vec<String>>, Vec<crate::nodes::TableAlignment>)> {
        self.parent.take_table_data()
    }

    fn is_collecting_table(&self) -> bool {
        self.parent.is_collecting_table()
    }

    fn set_skip_children(&mut self, skip: bool) {
        self.parent.set_skip_children(skip);
    }

    fn render_children_to_string(&mut self, node_id: NodeId) -> String {
        self.parent.render_children_to_string(node_id)
    }
}

/// Trait for explicit attribute ID providers
///
/// This trait is used by extensions to insert explicit IDs for headings during formatting.
pub trait ExplicitAttributeIdProvider {
    /// Add an explicit ID to a node
    ///
    /// This is called when a node has an explicit ID attribute.
    fn add_explicit_id(
        &self,
        node_id: NodeId,
        id: Option<&str>,
        context: &mut dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
    );
}

/// Trait for translation span renderers
///
/// This trait is used for rendering content that should be translated.
pub trait TranslatingSpanRenderer {
    /// Render the span
    fn render(
        &self,
        context: &mut dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
    );
}

/// Trait for translation placeholder generators
///
/// This trait is used to generate placeholders for translation spans.
pub trait TranslationPlaceholderGenerator {
    /// Get a placeholder for the given index
    ///
    /// The index is 1-based and should be unique within the document.
    fn get_placeholder(&self, index: usize) -> String;
}

/// Default translation placeholder generator
#[derive(Debug)]
pub struct DefaultPlaceholderGenerator {
    format: String,
}

impl DefaultPlaceholderGenerator {
    /// Create a new default placeholder generator
    pub fn new() -> Self {
        Self {
            format: "_{}_".to_string(),
        }
    }

    /// Create a new generator with a custom format
    pub fn with_format(format: impl Into<String>) -> Self {
        Self {
            format: format.into(),
        }
    }
}

impl Default for DefaultPlaceholderGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationPlaceholderGenerator for DefaultPlaceholderGenerator {
    fn get_placeholder(&self, index: usize) -> String {
        self.format.replace("{}", &index.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require a mock implementation of NodeFormatterContext
    // For now, we just test the placeholder generator

    #[test]
    fn test_default_placeholder_generator() {
        let generator = DefaultPlaceholderGenerator::new();
        assert_eq!(generator.get_placeholder(1), "_1_");
        assert_eq!(generator.get_placeholder(42), "_42_");
    }

    #[test]
    fn test_custom_placeholder_generator() {
        let generator = DefaultPlaceholderGenerator::with_format("[{}]");
        assert_eq!(generator.get_placeholder(1), "[1]");
    }
}
