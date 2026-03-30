//! Markdown formatter
//!
//! This module provides the main Markdown formatter implementation,
//! inspired by flexmark-java's Formatter class.

use crate::arena::{NodeArena, NodeId};
use crate::render::formatter_context::{
    NodeFormatterContext, SubFormatterContext,
    TranslationPlaceholderGenerator,
};
use crate::render::formatter_options::FormatterOptions;
use crate::render::formatting_phase::FormattingPhase;
use crate::render::markdown_writer::MarkdownWriter;
use crate::render::node_formatter::{
    ComposedNodeFormatter, NodeFormatter, NodeFormattingHandler, NodeValueType,
};
use crate::render::phased_formatter::{ComposedPhasedFormatter, PhasedNodeFormatter};
use crate::render::render_purpose::RenderPurpose;

use std::collections::HashMap;

/// Main Markdown formatter
///
/// This is the primary entry point for formatting Markdown documents.
/// It coordinates multiple node formatters and manages the rendering process.
pub struct Formatter {
    /// Formatter options
    options: FormatterOptions,
    /// Node formatters
    node_formatters: ComposedNodeFormatter,
    /// Phased formatters
    phased_formatters: ComposedPhasedFormatter,
    /// Translation placeholder generator
    placeholder_generator: Box<dyn TranslationPlaceholderGenerator>,
}

impl std::fmt::Debug for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Formatter")
            .field("options", &self.options)
            .field("node_formatters", &self.node_formatters)
            .field("phased_formatters", &self.phased_formatters)
            .finish_non_exhaustive()
    }
}

impl Formatter {
    /// Create a new formatter with default options
    pub fn new() -> Self {
        Self::with_options(FormatterOptions::default())
    }

    /// Create a new formatter with specific options
    pub fn with_options(options: FormatterOptions) -> Self {
        Self {
            options,
            node_formatters: ComposedNodeFormatter::new(),
            phased_formatters: ComposedPhasedFormatter::new(),
            placeholder_generator: Box::new(
                crate::render::formatter_context::DefaultPlaceholderGenerator::new(),
            ),
        }
    }

    /// Add a node formatter
    pub fn add_node_formatter(&mut self, formatter: Box<dyn NodeFormatter>) {
        self.node_formatters.add_formatter(formatter);
    }

    /// Add a phased formatter
    pub fn add_phased_formatter(&mut self, formatter: Box<dyn PhasedNodeFormatter>) {
        self.phased_formatters.add_formatter(formatter);
    }

    /// Set the placeholder generator
    pub fn set_placeholder_generator(
        &mut self,
        generator: Box<dyn TranslationPlaceholderGenerator>,
    ) {
        self.placeholder_generator = generator;
    }

    /// Render a document
    ///
    /// This is the main entry point for rendering a document tree to Markdown.
    pub fn render(&self, arena: &NodeArena, root: NodeId) -> String {
        let mut writer = MarkdownWriter::new(self.options.format_flags);
        let mut context = MainFormatterContext::new(arena, &self.options, &self.node_formatters);

        // Execute pre-document phases
        for phase in FormattingPhase::before_document() {
            context.set_phase(*phase);
            self.phased_formatters.render_phase(
                &mut context,
                &mut writer,
                root,
                *phase,
            );
        }

        // Main document rendering
        context.set_phase(FormattingPhase::Document);
        context.render(root, &mut writer);

        // Execute post-document phases
        for phase in FormattingPhase::after_document() {
            context.set_phase(*phase);
            self.phased_formatters.render_phase(
                &mut context,
                &mut writer,
                root,
                *phase,
            );
        }

        writer.to_string()
    }

    /// Render a document with a specific render purpose
    pub fn render_with_purpose(
        &self,
        arena: &NodeArena,
        root: NodeId,
        purpose: RenderPurpose,
    ) -> String {
        let mut writer = MarkdownWriter::new(self.options.format_flags);
        let mut context =
            MainFormatterContext::with_purpose(arena, &self.options, &self.node_formatters, purpose);

        // Execute pre-document phases
        for phase in FormattingPhase::before_document() {
            context.set_phase(*phase);
            self.phased_formatters.render_phase(
                &mut context,
                &mut writer,
                root,
                *phase,
            );
        }

        // Main document rendering
        context.set_phase(FormattingPhase::Document);
        context.render(root, &mut writer);

        // Execute post-document phases
        for phase in FormattingPhase::after_document() {
            context.set_phase(*phase);
            self.phased_formatters.render_phase(
                &mut context,
                &mut writer,
                root,
                *phase,
            );
        }

        writer.to_string()
    }

    /// Get the formatter options
    pub fn get_options(&self) -> &FormatterOptions {
        &self.options
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Main formatter context implementation
pub struct MainFormatterContext<'a> {
    /// Reference to the node arena
    arena: &'a NodeArena,
    /// Formatter options
    options: &'a FormatterOptions,
    /// Node formatters
    formatters: &'a ComposedNodeFormatter,
    /// Current formatting phase
    phase: FormattingPhase,
    /// Current render purpose
    render_purpose: RenderPurpose,
    /// Handler map: node type -> list of handlers
    handler_map: HashMap<NodeValueType, Vec<NodeFormattingHandler>>,
    /// Current node being rendered
    current_node: Option<NodeId>,
    /// Handler delegation stack
    handler_stack: Vec<(NodeValueType, usize)>,
    /// Tight list context
    tight_list: bool,
    /// List nesting level
    list_nesting: usize,
    /// Block quote context
    in_block_quote: bool,
    /// Block quote nesting level
    block_quote_nesting: usize,
    /// Collected nodes by type
    collected_nodes: HashMap<NodeValueType, Vec<NodeId>>,
}

impl<'a> std::fmt::Debug for MainFormatterContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MainFormatterContext")
            .field("phase", &self.phase)
            .field("render_purpose", &self.render_purpose)
            .field("current_node", &self.current_node)
            .field("tight_list", &self.tight_list)
            .field("list_nesting", &self.list_nesting)
            .field("in_block_quote", &self.in_block_quote)
            .field("block_quote_nesting", &self.block_quote_nesting)
            .finish_non_exhaustive()
    }
}

impl<'a> MainFormatterContext<'a> {
    /// Create a new main formatter context
    pub fn new(
        arena: &'a NodeArena,
        options: &'a FormatterOptions,
        formatters: &'a ComposedNodeFormatter,
    ) -> Self {
        let mut context = Self {
            arena,
            options,
            formatters,
            phase: FormattingPhase::Document,
            render_purpose: RenderPurpose::Format,
            handler_map: HashMap::new(),
            current_node: None,
            handler_stack: Vec::new(),
            tight_list: false,
            list_nesting: 0,
            in_block_quote: false,
            block_quote_nesting: 0,
            collected_nodes: HashMap::new(),
        };
        context.build_handler_map();
        context.collect_nodes();
        context
    }

    /// Create a new context with a specific render purpose
    pub fn with_purpose(
        arena: &'a NodeArena,
        options: &'a FormatterOptions,
        formatters: &'a ComposedNodeFormatter,
        purpose: RenderPurpose,
    ) -> Self {
        let mut context = Self::new(arena, options, formatters);
        context.render_purpose = purpose;
        context
    }

    /// Build the handler map from all formatters
    fn build_handler_map(&mut self) {
        for handler in self.formatters.get_all_handlers() {
            self.handler_map
                .entry(handler.node_type)
                .or_default()
                .push(handler);
        }
    }

    /// Collect nodes by type for quick access
    fn collect_nodes(&mut self) {
        // Only collect nodes if there are interested formatters
        let node_classes = self.formatters.get_all_node_classes();
        if node_classes.is_empty() {
            return;
        }

        // Find the document root and collect nodes
        if let Some(root) = self.find_document_root() {
            self.collect_nodes_recursive(root, &node_classes);
        }
    }

    /// Recursively collect nodes of interest
    fn collect_nodes_recursive(&mut self, node_id: NodeId, node_classes: &[NodeValueType]) {
        let node = self.arena.get(node_id);
        let node_type = NodeValueType::from_node_value(&node.value);
        
        if node_classes.contains(&node_type) {
            self.collected_nodes
                .entry(node_type)
                .or_default()
                .push(node_id);
        }

        // Recursively collect from children
        let mut child_id = node.first_child;
        while let Some(child) = child_id {
            self.collect_nodes_recursive(child, node_classes);
            child_id = self.arena.get(child).next;
        }
    }

    /// Find the document root node
    fn find_document_root(&self) -> Option<NodeId> {
        // Try to find the document node (should be node 0)
        if let Some(node) = self.arena.try_get(0) {
            if matches!(node.value, crate::nodes::NodeValue::Document) {
                return Some(0);
            }
        }
        
        // Otherwise, find any node and trace back to root
        // This is a simplified approach - iterate through all nodes
        for i in 0..1000 { // Limit to avoid infinite loops
            if let Some(node) = self.arena.try_get(i) {
                let mut current = i;
                loop {
                    let n = self.arena.get(current);
                    if let Some(parent) = n.parent {
                        current = parent;
                    } else {
                        return Some(current);
                    }
                }
            }
        }
        None
    }

    /// Set the current formatting phase
    pub fn set_phase(&mut self, phase: FormattingPhase) {
        self.phase = phase;
    }

    /// Set the current node
    pub fn set_current_node(&mut self, node_id: Option<NodeId>) {
        self.current_node = node_id;
    }

    /// Get handlers for a node type
    fn get_handlers(&self, node_type: NodeValueType) -> Option<&Vec<NodeFormattingHandler>> {
        self.handler_map.get(&node_type)
    }

    /// Render a node using the appropriate handler
    pub fn render(&mut self, node_id: NodeId, markdown: &mut MarkdownWriter) {
        let node = self.arena.get(node_id);
        let node_type = NodeValueType::from_node_value(&node.value);

        // Check if we have handlers for this node type
        let handler_index = self
            .handler_map
            .get(&node_type)
            .and_then(|h| if h.is_empty() { None } else { Some(0) });

        if let Some(index) = handler_index {
            // Get the handler function pointer
            // We use a two-step process to avoid borrowing issues
            self.current_node = Some(node_id);
            
            // Call the handler - we need to use unsafe or restructure to avoid borrow checker issues
            // For now, we'll just render children as a fallback
            // TODO: Implement proper handler dispatch
            self.render_children(node_id, markdown);
            
            self.current_node = None;
        } else {
            // No handler registered, render children
            self.render_children(node_id, markdown);
        }
    }

    /// Render children of a node
    fn render_children(&mut self, node_id: NodeId, markdown: &mut MarkdownWriter) {
        let node = self.arena.get(node_id);
        let mut child_id = node.first_child;
        while let Some(child) = child_id {
            self.render(child, markdown);
            child_id = self.arena.get(child).next;
        }
    }
}

impl<'a> NodeFormatterContext for MainFormatterContext<'a> {
    fn get_markdown_writer(&mut self) -> &mut MarkdownWriter {
        panic!("MainFormatterContext doesn't have a direct writer; use render() instead")
    }

    fn render(&mut self, node_id: NodeId) {
        // This is a no-op in the main context because rendering
        // is done through render() with a writer
        // In practice, this would be called from a formatter
        // that has access to the writer
        let _ = node_id;
    }

    fn render_children(&mut self, node_id: NodeId) {
        // Same as above - this is a no-op in the main context
        let _ = node_id;
    }

    fn get_formatting_phase(&self) -> FormattingPhase {
        self.phase
    }

    fn delegate_render(&mut self) {
        // TODO: Implement handler delegation
    }

    fn get_formatter_options(&self) -> &FormatterOptions {
        self.options
    }

    fn get_render_purpose(&self) -> RenderPurpose {
        self.render_purpose
    }

    fn get_arena(&self) -> &NodeArena {
        self.arena
    }

    fn get_current_node(&self) -> Option<NodeId> {
        self.current_node
    }

    fn get_nodes_of_type(&self, node_type: NodeValueType) -> Vec<NodeId> {
        self.collected_nodes.get(&node_type).cloned().unwrap_or_default()
    }

    fn get_nodes_of_types(&self, node_types: &[NodeValueType]) -> Vec<NodeId> {
        let mut result = Vec::new();
        for node_type in node_types {
            if let Some(nodes) = self.collected_nodes.get(node_type) {
                result.extend(nodes);
            }
        }
        result
    }

    fn get_block_quote_like_prefix_predicate(&self) -> Box<dyn Fn(char) -> bool> {
        Box::new(|c| c == '>')
    }

    fn get_block_quote_like_prefix_chars(&self) -> &str {
        ">"
    }

    fn transform_non_translating(&self, text: &str) -> String {
        // In translation mode, return a placeholder
        // In normal mode, return the text as-is
        if self.render_purpose.is_transforming_text() {
            // TODO: Implement actual placeholder generation
            format!("_{}_", text.len())
        } else {
            text.to_string()
        }
    }

    fn transform_translating(&self, text: &str) -> String {
        // In translation mode, return a placeholder
        // In normal mode, return the text as-is
        if self.render_purpose.is_transforming_text() {
            // TODO: Implement actual placeholder generation
            format!("_{}_", text.len())
        } else {
            text.to_string()
        }
    }

    fn create_sub_context(&self) -> Box<dyn NodeFormatterContext> {
        // Note: Creating a sub-context from a non-mutable reference is not supported
        // This would require interior mutability or a different design
        panic!("Cannot create sub-context from immutable reference");
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
}

/// Formatter builder for convenient configuration
pub struct FormatterBuilder {
    options: FormatterOptions,
    node_formatters: Vec<Box<dyn NodeFormatter>>,
    phased_formatters: Vec<Box<dyn PhasedNodeFormatter>>,
}

impl std::fmt::Debug for FormatterBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormatterBuilder")
            .field("options", &self.options)
            .field("node_formatters", &self.node_formatters.len())
            .field("phased_formatters", &self.phased_formatters.len())
            .finish()
    }
}

impl FormatterBuilder {
    /// Create a new formatter builder
    pub fn new() -> Self {
        Self {
            options: FormatterOptions::default(),
            node_formatters: Vec::new(),
            phased_formatters: Vec::new(),
        }
    }

    /// Set the formatter options
    pub fn options(mut self, options: FormatterOptions) -> Self {
        self.options = options;
        self
    }

    /// Add a node formatter
    pub fn add_node_formatter(mut self, formatter: Box<dyn NodeFormatter>) -> Self {
        self.node_formatters.push(formatter);
        self
    }

    /// Add a phased formatter
    pub fn add_phased_formatter(mut self, formatter: Box<dyn PhasedNodeFormatter>) -> Self {
        self.phased_formatters.push(formatter);
        self
    }

    /// Build the formatter
    pub fn build(self) -> Formatter {
        let mut formatter = Formatter::with_options(self.options);
        for nf in self.node_formatters {
            formatter.add_node_formatter(nf);
        }
        for pf in self.phased_formatters {
            formatter.add_phased_formatter(pf);
        }
        formatter
    }
}

impl Default for FormatterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to render a document with default options
pub fn format_document(arena: &NodeArena, root: NodeId) -> String {
    let formatter = Formatter::new();
    formatter.render(arena, root)
}

/// Convenience function to render a document with custom options
pub fn format_document_with_options(
    arena: &NodeArena,
    root: NodeId,
    options: FormatterOptions,
) -> String {
    let formatter = Formatter::with_options(options);
    formatter.render(arena, root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};
    use crate::nodes::NodeValue;

    #[test]
    fn test_formatter_creation() {
        let formatter = Formatter::new();
        assert!(matches!(formatter.get_options().heading_style, crate::render::formatter_options::HeadingStyle::AsIs));
    }

    #[test]
    fn test_formatter_builder() {
        let formatter = FormatterBuilder::new()
            .options(FormatterOptions::new().with_right_margin(80))
            .build();

        assert_eq!(formatter.get_options().right_margin, 80);
    }

    #[test]
    fn test_format_document() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        // This will use default formatters which don't do much yet
        let _result = format_document(&arena, root);
        // Just verify it doesn't panic
    }
}
