//! CommonMark rendering and formatting
//!
//! This module provides CommonMark output generation and the main Markdown formatter
//! implementation, inspired by flexmark-java's Formatter class.
//!
//! # Submodules
//!
//! - `options`: Formatter configuration options
//! - `context`: Formatter context traits and implementations
//! - `phase`: Formatting phase definitions
//! - `purpose`: Render purpose for translation workflows
//! - `node`: Node formatter traits
//! - `phased`: Phased formatter support
//! - `writer`: Markdown output writer
//! - `utils`: Utility functions for formatting
//! - `handler_utils`: Handler factory functions and context helpers
//! - `table`: Table formatter for GFM tables
//! - `commonmark_formatter`: CommonMark output formatter

pub mod commonmark_formatter;
pub mod context;
pub mod escaping;
pub mod format_control;
pub mod handler_utils;
pub mod handlers;
pub mod line_breaking;
pub mod node;
pub mod phase;
pub mod phased;
pub mod purpose;
pub mod repository_formatter;
pub mod table;
pub mod translation;
pub mod utils;
pub mod writer;

// Re-export commonly used types
pub use crate::options::format::{
    Alignment, BlockQuoteMarker, BulletMarker, CodeFenceMarker, DiscretionaryText,
    ElementPlacement, ElementPlacementSort, FormatFlags, FormatOptions, HeadingStyle,
    ListSpacing, NumberedMarker, TrailingMarker,
};
pub use commonmark_formatter::CommonMarkNodeFormatter;
pub use context::{
    DefaultPlaceholderGenerator, ExplicitAttributeIdProvider, NodeFormatterContext,
    SubFormatterContext, TranslatingSpanRenderer, TranslationPlaceholderGenerator,
};
// Re-export line breaking types
pub use line_breaking::{AtomicKind, ParagraphLineBreaker, UnitHandle, UnitKind, Word};
pub use node::{
    ComposedNodeFormatter, NodeFormatter, NodeFormatterFactory, NodeFormatterFn,
    NodeFormattingHandler, NodeValueType,
};
pub use phase::FormattingPhase;
pub use phased::{ComposedPhasedFormatter, PhasedNodeFormatter, SimplePhasedFormatter};
pub use purpose::{RenderPurpose, TranslationSpan, TranslationSpanCollection};
pub use repository_formatter::{
    LinkReferenceFormatter, NodeRepositoryFormatter, ReferenceEntry, ReferenceRepository,
};
pub use translation::{TranslationHandler, TranslationHandlerImpl};
pub use writer::MarkdownWriter;

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::NodeValue;
use std::collections::HashMap;

/// Main Markdown formatter
///
/// This is the primary entry point for formatting Markdown documents.
/// It coordinates multiple node formatters and manages the rendering process.
pub struct Formatter {
    /// Formatter options
    options: FormatOptions,
    /// Node formatters
    node_formatters: node::ComposedNodeFormatter,
    /// Phased formatters
    phased_formatters: phased::ComposedPhasedFormatter,
    /// Translation placeholder generator
    placeholder_generator: Box<dyn context::TranslationPlaceholderGenerator>,
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
        Self::with_options(FormatOptions::default())
    }

    /// Create a new formatter with specific options
    pub fn with_options(options: FormatOptions) -> Self {
        Self {
            options,
            node_formatters: node::ComposedNodeFormatter::new(),
            phased_formatters: phased::ComposedPhasedFormatter::new(),
            placeholder_generator: Box::new(context::DefaultPlaceholderGenerator::new()),
        }
    }

    /// Add a node formatter
    pub fn add_node_formatter(&mut self, formatter: Box<dyn node::NodeFormatter>) {
        self.node_formatters.add_formatter(formatter);
    }

    /// Add a phased formatter
    pub fn add_phased_formatter(
        &mut self,
        formatter: Box<dyn phased::PhasedNodeFormatter>,
    ) {
        self.phased_formatters.add_formatter(formatter);
    }

    /// Set the placeholder generator
    pub fn set_placeholder_generator(
        &mut self,
        generator: Box<dyn context::TranslationPlaceholderGenerator>,
    ) {
        self.placeholder_generator = generator;
    }

    /// Render a document
    ///
    /// This is the main entry point for rendering a document tree to Markdown.
    pub fn render(&self, arena: &NodeArena, root: NodeId) -> String {
        let mut writer = writer::MarkdownWriter::new(self.options.format_flags);
        writer.set_max_trailing_blank_lines(self.options.max_trailing_blank_lines);
        writer.set_right_margin(self.options.right_margin);
        let mut context =
            MainFormatterContext::new(arena, &self.options, &self.node_formatters);

        // Execute pre-document phases
        for phase in phase::FormattingPhase::before_document() {
            context.set_phase(*phase);
            self.phased_formatters
                .render_phase(&mut context, &mut writer, root, *phase);
        }

        // Main document rendering
        context.set_phase(phase::FormattingPhase::Document);
        context.render(root, &mut writer);

        // Execute post-document phases
        for phase in phase::FormattingPhase::after_document() {
            context.set_phase(*phase);
            self.phased_formatters
                .render_phase(&mut context, &mut writer, root, *phase);
        }

        writer.to_string()
    }

    /// Render a document with a specific render purpose
    pub fn render_with_purpose(
        &self,
        arena: &NodeArena,
        root: NodeId,
        purpose: purpose::RenderPurpose,
    ) -> String {
        let mut writer = writer::MarkdownWriter::new(self.options.format_flags);
        writer.set_max_trailing_blank_lines(self.options.max_trailing_blank_lines);
        writer.set_right_margin(self.options.right_margin);
        let mut context = MainFormatterContext::with_purpose(
            arena,
            &self.options,
            &self.node_formatters,
            purpose,
        );

        // Execute pre-document phases
        for phase in phase::FormattingPhase::before_document() {
            context.set_phase(*phase);
            self.phased_formatters
                .render_phase(&mut context, &mut writer, root, *phase);
        }

        // Main document rendering
        context.set_phase(phase::FormattingPhase::Document);
        context.render(root, &mut writer);

        // Execute post-document phases
        for phase in phase::FormattingPhase::after_document() {
            context.set_phase(*phase);
            self.phased_formatters
                .render_phase(&mut context, &mut writer, root, *phase);
        }

        writer.to_string()
    }

    /// Get the formatter options
    pub fn get_options(&self) -> &FormatOptions {
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
    options: &'a FormatOptions,
    /// Node formatters
    formatters: &'a node::ComposedNodeFormatter,
    /// Current formatting phase
    phase: phase::FormattingPhase,
    /// Current render purpose
    render_purpose: purpose::RenderPurpose,
    /// Handler map: node type -> list of handlers
    handler_map: HashMap<node::NodeValueType, Vec<node::NodeFormattingHandler>>,
    /// Current node being rendered
    current_node: Option<NodeId>,
    /// Handler delegation stack
    handler_stack: Vec<(node::NodeValueType, usize)>,
    /// Tight list context
    tight_list: bool,
    /// List nesting level
    list_nesting: usize,
    /// Block quote context
    in_block_quote: bool,
    /// Block quote nesting level
    block_quote_nesting: usize,
    /// Collected nodes by type
    collected_nodes: HashMap<node::NodeValueType, Vec<NodeId>>,
    /// Table data collection
    table_rows: Vec<Vec<String>>,
    /// Table alignments
    table_alignments: Vec<crate::core::nodes::TableAlignment>,
    /// Whether we're collecting table data
    collecting_table: bool,
    /// Whether to skip rendering children (for table cells)
    skip_children: bool,
    /// Format control processor for formatter:on/off comments
    format_control: format_control::FormatControlProcessor,
    /// Content buffer for format-off regions
    format_off_buffer: Option<String>,
    /// Whether delegation was requested by the current handler
    delegation_requested: bool,
    /// Paragraph line breaker for AST-based line breaking
    paragraph_line_breaker: Option<line_breaking::ParagraphLineBreaker>,
    /// Text collection buffer for render_children_to_string
    text_collection_buffer: Option<String>,
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
        options: &'a FormatOptions,
        formatters: &'a node::ComposedNodeFormatter,
    ) -> Self {
        let format_control = format_control::FormatControlProcessor::new(options);
        let mut context = Self {
            arena,
            options,
            formatters,
            phase: phase::FormattingPhase::Document,
            render_purpose: purpose::RenderPurpose::Format,
            handler_map: HashMap::new(),
            current_node: None,
            handler_stack: Vec::new(),
            tight_list: false,
            list_nesting: 0,
            in_block_quote: false,
            block_quote_nesting: 0,
            collected_nodes: HashMap::new(),
            table_rows: Vec::new(),
            table_alignments: Vec::new(),
            collecting_table: false,
            skip_children: false,
            format_control,
            format_off_buffer: None,
            delegation_requested: false,
            paragraph_line_breaker: None,
            text_collection_buffer: None,
        };
        context.build_handler_map();
        context.collect_nodes();
        context
    }

    /// Create a new context with a specific render purpose
    pub fn with_purpose(
        arena: &'a NodeArena,
        options: &'a FormatOptions,
        formatters: &'a node::ComposedNodeFormatter,
        purpose: purpose::RenderPurpose,
    ) -> Self {
        let mut context = Self::new(arena, options, formatters);
        context.render_purpose = purpose;
        context
    }

    /// Check if formatting is currently off
    pub fn is_formatting_off(&self) -> bool {
        self.format_control.is_formatting_off()
    }

    /// Process an HTML comment for format control
    /// Returns true if the comment was a format control comment
    pub fn process_format_control_comment(&mut self, comment_text: &str) -> bool {
        self.format_control.process_comment(comment_text)
    }

    /// Start buffering content for format-off region
    pub fn start_format_off_buffer(&mut self) {
        self.format_off_buffer = Some(String::new());
    }

    /// End buffering and return the buffered content
    pub fn end_format_off_buffer(&mut self) -> Option<String> {
        self.format_off_buffer.take()
    }

    /// Append content to format-off buffer if active
    pub fn append_to_format_off_buffer(&mut self, content: &str) {
        if let Some(ref mut buffer) = self.format_off_buffer {
            buffer.push_str(content);
        }
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
    fn collect_nodes_recursive(
        &mut self,
        node_id: NodeId,
        node_classes: &[node::NodeValueType],
    ) {
        let node = self.arena.get(node_id);
        let node_type = node::NodeValueType::from_node_value(&node.value);

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
        if let Some(_node) = self.arena.try_get(0) {
            if matches!(_node.value, crate::core::nodes::NodeValue::Document) {
                return Some(0);
            }
        }

        // Otherwise, find any node and trace back to root
        // Use arena's actual length instead of hardcoded limit
        let node_count = self.arena.len();
        for i in 0..node_count {
            let node_id = i as NodeId;
            if let Some(_node) = self.arena.try_get(node_id) {
                let mut current = node_id;
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
    pub fn set_phase(&mut self, phase: phase::FormattingPhase) {
        self.phase = phase;
    }

    /// Set the current node
    pub fn set_current_node(&mut self, node_id: Option<NodeId>) {
        self.current_node = node_id;
    }

    /// Render a node using the appropriate handler
    pub fn render(&mut self, node_id: NodeId, markdown: &mut writer::MarkdownWriter) {
        self.render_with_handler_index(node_id, markdown, 0);
    }

    /// Render a node starting from a specific handler index
    fn render_with_handler_index(
        &mut self,
        node_id: NodeId,
        markdown: &mut writer::MarkdownWriter,
        handler_index: usize,
    ) {
        let node = self.arena.get(node_id);
        let node_type = node::NodeValueType::from_node_value(&node.value);

        // Check for HTML comment that might be a format control comment
        if let NodeValue::HtmlBlock(html) = &node.value {
            let literal = &html.literal;
            // Check if this is an HTML comment
            if literal.trim().starts_with("<!--") && literal.trim().ends_with("-->") {
                let was_format_control = self.process_format_control_comment(literal);
                if was_format_control {
                    // Output the comment as-is
                    for line in literal.lines() {
                        markdown.append(line);
                        markdown.line();
                    }
                    return;
                }
            }
        }

        // Check for inline HTML comment
        if let NodeValue::HtmlInline(html) = &node.value {
            let literal = html.as_ref();
            if literal.trim().starts_with("<!--") && literal.trim().ends_with("-->") {
                let was_format_control = self.process_format_control_comment(literal);
                if was_format_control {
                    // Output the comment as-is
                    markdown.append(literal);
                    return;
                }
            }
        }

        // If formatting is off, render content without formatting
        if self.is_formatting_off() {
            self.render_unformatted(node_id, markdown);
            return;
        }

        // Check if we have handlers for this node type
        let handlers = self.handler_map.get(&node_type);

        if let Some(handler_list) = handlers {
            if handler_index < handler_list.len() {
                let handler = handler_list[handler_index].clone();

                self.current_node = Some(node_id);
                self.handler_stack.push((node_type, handler_index));
                let node_value = &self.arena.get(node_id).value;

                // Call the opening formatter
                handler.format_open(node_value, self, markdown);

                // Render children unless skip_children is set
                // (used for table cells to avoid double rendering)
                if !self.skip_children {
                    self.render_children(node_id, markdown);
                }
                self.skip_children = false;

                // Re-set current_node before calling format_close
                // because render_children may have set it to None
                self.current_node = Some(node_id);

                // Call the closing formatter if present
                handler.format_close(node_value, self, markdown);

                self.handler_stack.pop();
                self.current_node = None;

                // Check if delegation was requested
                if self.delegation_requested {
                    self.delegation_requested = false;
                    // Call the next handler for this node type
                    self.render_with_handler_index(node_id, markdown, handler_index + 1);
                }
            } else {
                // No more handlers, render children
                self.render_children(node_id, markdown);
            }
        } else {
            // No handler registered, render children
            self.render_children(node_id, markdown);
        }
    }

    /// Render a node without any formatting (for format-off regions)
    fn render_unformatted(
        &mut self,
        node_id: NodeId,
        markdown: &mut writer::MarkdownWriter,
    ) {
        let node = self.arena.get(node_id);

        // For text nodes, output the literal text
        match &node.value {
            NodeValue::Text(text) => {
                markdown.append_raw(text.as_ref());
            }
            NodeValue::HtmlBlock(html) => {
                markdown.append_raw(&html.literal);
                markdown.line();
            }
            NodeValue::HtmlInline(html) => {
                markdown.append_raw(html);
            }
            NodeValue::SoftBreak => {
                markdown.line();
            }
            NodeValue::HardBreak => {
                markdown.append_raw("  ");
                markdown.line();
            }
            _ => {
                // For other nodes, recursively render children
                let mut child_id = node.first_child;
                while let Some(child) = child_id {
                    self.render_unformatted(child, markdown);
                    child_id = self.arena.get(child).next;
                }
            }
        }
    }

    /// Render children of a node
    fn render_children(
        &mut self,
        node_id: NodeId,
        markdown: &mut writer::MarkdownWriter,
    ) {
        let node = self.arena.get(node_id);
        let mut child_id = node.first_child;
        while let Some(child) = child_id {
            self.render(child, markdown);
            child_id = self.arena.get(child).next;
        }
    }
}

impl<'a> context::NodeFormatterContext for MainFormatterContext<'a> {
    fn get_markdown_writer(&mut self) -> &mut writer::MarkdownWriter {
        panic!("MainFormatterContext doesn't have a direct writer; use render() instead")
    }

    fn render(&mut self, node_id: NodeId) {
        // This is a no-op in the main context because rendering
        // is done through render() with a writer
        let _ = node_id;
    }

    fn render_children(&mut self, node_id: NodeId) {
        // Same as above - this is a no-op in the main context
        let _ = node_id;
    }

    fn get_formatting_phase(&self) -> phase::FormattingPhase {
        self.phase
    }

    fn delegate_render(&mut self) {
        // Handler delegation is used when a formatter wants to pass rendering
        // to the next handler registered for the same node type.
        //
        // This implementation uses the handler_stack to track the current
        // handler index and calls the next handler if available.
        if self.handler_stack.last().copied().is_some() {
            // The actual delegation happens in render_with_handler_index
            // by calling the next handler index
            self.delegation_requested = true;
        }
    }

    fn get_formatter_options(&self) -> &FormatOptions {
        self.options
    }

    fn get_render_purpose(&self) -> purpose::RenderPurpose {
        self.render_purpose
    }

    fn get_arena(&self) -> &NodeArena {
        self.arena
    }

    fn get_current_node(&self) -> Option<NodeId> {
        self.current_node
    }

    fn get_nodes_of_type(&self, node_type: node::NodeValueType) -> Vec<NodeId> {
        self.collected_nodes
            .get(&node_type)
            .cloned()
            .unwrap_or_default()
    }

    fn get_nodes_of_types(&self, node_types: &[node::NodeValueType]) -> Vec<NodeId> {
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
            format!("_{}_", text.len())
        } else {
            text.to_string()
        }
    }

    fn transform_translating(&self, text: &str) -> String {
        // In translation mode, return a placeholder
        // In normal mode, return the text as-is
        if self.render_purpose.is_transforming_text() {
            format!("_{}_", text.len())
        } else {
            text.to_string()
        }
    }

    fn create_sub_context(&self) -> Box<dyn context::NodeFormatterContext> {
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

    // Table data collection methods

    fn start_table_collection(
        &mut self,
        alignments: Vec<crate::core::nodes::TableAlignment>,
    ) {
        self.table_rows = Vec::new();
        self.table_alignments = alignments;
        self.collecting_table = true;
    }

    fn add_table_row(&mut self) {
        if self.collecting_table {
            self.table_rows.push(Vec::new());
        }
    }

    fn add_table_cell(&mut self, content: String) {
        if self.collecting_table {
            if let Some(last_row) = self.table_rows.last_mut() {
                last_row.push(content);
            }
        }
    }

    fn take_table_data(
        &mut self,
    ) -> Option<(Vec<Vec<String>>, Vec<crate::core::nodes::TableAlignment>)> {
        if self.collecting_table {
            self.collecting_table = false;
            let rows = std::mem::take(&mut self.table_rows);
            let alignments = std::mem::take(&mut self.table_alignments);
            if !rows.is_empty() {
                Some((rows, alignments))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn is_collecting_table(&self) -> bool {
        self.collecting_table
    }

    fn set_skip_children(&mut self, skip: bool) {
        self.skip_children = skip;
    }

    fn render_children_to_string(&mut self, node_id: NodeId) -> String {
        // Temporarily reset skip_children to allow rendering
        let old_skip_children = self.skip_children;
        self.skip_children = false;

        // Temporarily disable paragraph line breaking to avoid double-adding text
        let old_line_breaker = self.paragraph_line_breaker.take();

        // Set up text collection buffer
        self.text_collection_buffer = Some(String::new());

        // Create a temporary writer to capture output
        let mut temp_writer = writer::MarkdownWriter::new(self.options.format_flags);

        // Render children to the temporary writer
        self.render_children(node_id, &mut temp_writer);

        // Collect the text and clear the buffer
        let result = self.text_collection_buffer.take().unwrap_or_default();

        // Restore paragraph line breaker
        self.paragraph_line_breaker = old_line_breaker;

        // Restore skip_children
        self.skip_children = old_skip_children;

        // Return the captured content
        result
    }

    // Line breaking methods (legacy - deprecated, use paragraph line breaking methods instead)

    // ParagraphLineBreaker methods

    fn start_paragraph_line_breaking(&mut self, max_width: usize, prefix: String) {
        self.paragraph_line_breaker =
            Some(line_breaking::ParagraphLineBreaker::new(max_width, prefix));
    }

    fn finish_paragraph_line_breaking(&mut self) -> Option<String> {
        self.paragraph_line_breaker
            .take()
            .map(|breaker| breaker.format())
    }

    fn add_paragraph_text(&mut self, text: &str) {
        if let Some(ref mut breaker) = self.paragraph_line_breaker {
            breaker.add_text(text);
        }
        // Also add to text collection buffer if active
        if let Some(ref mut buffer) = self.text_collection_buffer {
            buffer.push_str(text);
        }
    }

    fn add_paragraph_word(&mut self, text: &str) {
        if let Some(ref mut breaker) = self.paragraph_line_breaker {
            // For markdown markers like * and **, add as regular word
            // The emphasis/strong content will be handled as a unit by start_unit/end_unit
            breaker.add_word(text);
        }
        // Also add to text collection buffer if active
        if let Some(ref mut buffer) = self.text_collection_buffer {
            buffer.push_str(text);
        }
    }

    fn start_paragraph_unit(
        &mut self,
        kind: line_breaking::UnitKind,
        marker_width: usize,
    ) -> Option<line_breaking::UnitHandle> {
        self.paragraph_line_breaker
            .as_mut()
            .map(|breaker| breaker.start_unit(kind, marker_width))
    }

    fn end_paragraph_unit(
        &mut self,
        handle: line_breaking::UnitHandle,
        content_width: usize,
        marker_width: usize,
    ) {
        if let Some(ref mut breaker) = self.paragraph_line_breaker {
            breaker.end_unit(handle, content_width, marker_width);
        }
    }

    fn add_paragraph_unbreakable_unit(
        &mut self,
        kind: line_breaking::UnitKind,
        prefix: &str,
        content: &str,
        suffix: &str,
    ) {
        if let Some(ref mut breaker) = self.paragraph_line_breaker {
            breaker.add_unbreakable_unit(kind, prefix, content, suffix);
        }
        // Also add to text collection buffer if active
        if let Some(ref mut buffer) = self.text_collection_buffer {
            buffer.push_str(prefix);
            buffer.push_str(content);
            buffer.push_str(suffix);
        }
    }

    fn add_paragraph_atomic(&mut self, content: &str, kind: AtomicKind) {
        if let Some(ref mut breaker) = self.paragraph_line_breaker {
            breaker.add_atomic(content, kind);
        }
        // Also add to text collection buffer if active
        if let Some(ref mut buffer) = self.text_collection_buffer {
            buffer.push_str(content);
        }
    }

    fn add_paragraph_hard_break(&mut self) {
        if let Some(ref mut breaker) = self.paragraph_line_breaker {
            breaker.add_hard_break();
        }
    }

    fn is_paragraph_line_breaking(&self) -> bool {
        // Return true if either paragraph line breaker is active or we're collecting text
        self.paragraph_line_breaker.is_some() || self.text_collection_buffer.is_some()
    }

    fn remove_paragraph_trailing_space(&mut self) {
        if let Some(ref mut breaker) = self.paragraph_line_breaker {
            breaker.remove_trailing_space();
        }
    }

    fn paragraph_ends_with_whitespace(&self) -> bool {
        self.paragraph_line_breaker
            .as_ref()
            .map(|breaker| breaker.ends_with_whitespace())
            .unwrap_or(false)
    }

    fn paragraph_ends_with_cjk(&self) -> bool {
        self.paragraph_line_breaker
            .as_ref()
            .map(|breaker| breaker.ends_with_cjk())
            .unwrap_or(false)
    }
}

/// Formatter builder for convenient configuration
pub struct FormatterBuilder {
    options: FormatOptions,
    node_formatters: Vec<Box<dyn node::NodeFormatter>>,
    phased_formatters: Vec<Box<dyn phased::PhasedNodeFormatter>>,
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
            options: FormatOptions::default(),
            node_formatters: Vec::new(),
            phased_formatters: Vec::new(),
        }
    }

    /// Set the formatter options
    pub fn options(mut self, options: FormatOptions) -> Self {
        self.options = options;
        self
    }

    /// Add a node formatter
    pub fn add_node_formatter(
        mut self,
        formatter: Box<dyn node::NodeFormatter>,
    ) -> Self {
        self.node_formatters.push(formatter);
        self
    }

    /// Add a phased formatter
    pub fn add_phased_formatter(
        mut self,
        formatter: Box<dyn phased::PhasedNodeFormatter>,
    ) -> Self {
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
    options: FormatOptions,
) -> String {
    let formatter = Formatter::with_options(options);
    formatter.render(arena, root)
}

/// Render a node tree as CommonMark
///
/// This function uses the CommonMarkNodeFormatter via the Formatter framework,
/// which provides a flexible, node-based approach to rendering CommonMark output.
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
/// * `wrap_width` - Maximum line width for wrapping (0 = no wrapping)
///
/// # Returns
///
/// The CommonMark output as a String
pub fn render(arena: &NodeArena, root: NodeId, wrap_width: usize) -> String {
    let opts = FormatOptions::new().with_right_margin(wrap_width);
    let mut formatter = Formatter::with_options(opts);
    formatter.add_node_formatter(Box::new(
        commonmark_formatter::CommonMarkNodeFormatter::new(),
    ));
    formatter.render(arena, root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::NodeValue;

    #[test]
    fn test_formatter_creation() {
        let formatter = Formatter::new();
        assert!(matches!(
            formatter.get_options().heading_style,
            HeadingStyle::AsIs
        ));
    }

    #[test]
    fn test_formatter_builder() {
        let formatter = FormatterBuilder::new()
            .options(FormatOptions::new().with_right_margin(80))
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

    #[test]
    fn test_format_document_with_commonmark_formatter() {
        use crate::render::commonmark::CommonMarkNodeFormatter;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(
            crate::core::nodes::NodeHeading {
                level: 1,
                setext: false,
                closed: false,
            },
        )));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let mut formatter = Formatter::new();
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(result.contains("#"), "Should contain heading marker");
        assert!(result.contains("Hello"), "Should contain text content");
        assert!(
            result.contains("# Hello"),
            "Should have proper heading format with space"
        );
    }

    #[test]
    fn test_format_document_paragraphs() {
        use crate::render::commonmark::CommonMarkNodeFormatter;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // First paragraph with "Hello World"
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Hello World")));
        TreeOps::append_child(&mut arena, para1, text1);
        TreeOps::append_child(&mut arena, root, para1);

        // Second paragraph with "Second paragraph"
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text2 =
            arena.alloc(Node::with_value(NodeValue::make_text("Second paragraph")));
        TreeOps::append_child(&mut arena, para2, text2);
        TreeOps::append_child(&mut arena, root, para2);

        let mut formatter = Formatter::new();
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("Hello World"),
            "Should contain first paragraph"
        );
        assert!(
            result.contains("Second paragraph"),
            "Should contain second paragraph"
        );
        // Paragraphs should be separated by blank line
        assert!(
            result.contains("Hello World\n\nSecond")
                || result.contains("Hello World\r\n\r\nSecond"),
            "Paragraphs should be separated by blank line. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_emphasis() {
        use crate::render::commonmark::CommonMarkNodeFormatter;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        // "This is "
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("This is ")));
        TreeOps::append_child(&mut arena, para, text1);

        // **bold**
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let bold_text = arena.alloc(Node::with_value(NodeValue::make_text("bold")));
        TreeOps::append_child(&mut arena, strong, bold_text);
        TreeOps::append_child(&mut arena, para, strong);

        // " text"
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text(" text")));
        TreeOps::append_child(&mut arena, para, text2);

        TreeOps::append_child(&mut arena, root, para);

        let mut formatter = Formatter::new();
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("This is"),
            "Should contain 'This is'. Result: {:?}",
            result
        );
        assert!(
            result.contains("**bold**"),
            "Should contain '**bold**'. Result: {:?}",
            result
        );
        assert!(
            result.contains(" text"),
            "Should contain ' text'. Result: {:?}",
            result
        );
        // Check the full format with proper spacing
        assert!(
            result.contains("This is **bold** text"),
            "Should have proper spacing around emphasis. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_emphasis_and_width() {
        use crate::options::format::FormatOptions;
        use crate::render::commonmark::CommonMarkNodeFormatter;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        // "This is "
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("This is ")));
        TreeOps::append_child(&mut arena, para, text1);

        // *italic*
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let emph_text = arena.alloc(Node::with_value(NodeValue::make_text("italic")));
        TreeOps::append_child(&mut arena, emph, emph_text);
        TreeOps::append_child(&mut arena, para, emph);

        // " text"
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text(" text")));
        TreeOps::append_child(&mut arena, para, text2);

        TreeOps::append_child(&mut arena, root, para);

        // Test with right_margin = 80 (like CLI)
        let opts = FormatOptions::new().with_right_margin(80);
        let mut formatter = Formatter::with_options(opts);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        println!("Result with width 80: {:?}", result);
        println!("Expected: {:?}", "This is *italic* text\n\n");

        // Check the full format with proper spacing
        assert!(
            result == "This is *italic* text\n\n",
            "Should have proper spacing around emphasis. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_table() {
        use crate::core::nodes::{NodeTable, TableAlignment};
        use crate::render::commonmark::CommonMarkNodeFormatter;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create table
        let table =
            arena.alloc(Node::with_value(NodeValue::Table(Box::new(NodeTable {
                alignments: vec![TableAlignment::None, TableAlignment::None],
                num_columns: 2,
                num_rows: 2,
                num_nonempty_cells: 4,
            }))));

        // Header row
        let header_row = arena.alloc(Node::with_value(NodeValue::TableRow(true)));
        let cell1 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell1_text = arena.alloc(Node::with_value(NodeValue::make_text("Name")));
        TreeOps::append_child(&mut arena, cell1, cell1_text);
        TreeOps::append_child(&mut arena, header_row, cell1);

        let cell2 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell2_text = arena.alloc(Node::with_value(NodeValue::make_text("Age")));
        TreeOps::append_child(&mut arena, cell2, cell2_text);
        TreeOps::append_child(&mut arena, header_row, cell2);

        TreeOps::append_child(&mut arena, table, header_row);

        // Data row
        let data_row = arena.alloc(Node::with_value(NodeValue::TableRow(false)));
        let cell3 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell3_text = arena.alloc(Node::with_value(NodeValue::make_text("Alice")));
        TreeOps::append_child(&mut arena, cell3, cell3_text);
        TreeOps::append_child(&mut arena, data_row, cell3);

        let cell4 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell4_text = arena.alloc(Node::with_value(NodeValue::make_text("30")));
        TreeOps::append_child(&mut arena, cell4, cell4_text);
        TreeOps::append_child(&mut arena, data_row, cell4);

        TreeOps::append_child(&mut arena, table, data_row);
        TreeOps::append_child(&mut arena, root, table);

        let mut formatter = Formatter::new();
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        println!("Table result: {:?}", result);

        // Check that pipe characters are NOT escaped (allow for variable spacing)
        assert!(
            result.contains("Name") && result.contains("Age"),
            "Table header should contain Name and Age. Result: {:?}",
            result
        );
        // Check for data row
        assert!(
            result.contains("Alice") && result.contains("30"),
            "Table data should contain Alice and 30. Result: {:?}",
            result
        );
        // Check for delimiter row
        assert!(
            result.contains("---"),
            "Table should have delimiter row. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_table_and_code_block() {
        use crate::core::nodes::{NodeCodeBlock, NodeTable, TableAlignment};
        use crate::render::commonmark::CommonMarkNodeFormatter;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create table
        let table =
            arena.alloc(Node::with_value(NodeValue::Table(Box::new(NodeTable {
                alignments: vec![TableAlignment::None, TableAlignment::None],
                num_columns: 2,
                num_rows: 2,
                num_nonempty_cells: 4,
            }))));

        // Header row
        let header_row = arena.alloc(Node::with_value(NodeValue::TableRow(true)));
        let cell1 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell1_text = arena.alloc(Node::with_value(NodeValue::make_text("Name")));
        TreeOps::append_child(&mut arena, cell1, cell1_text);
        TreeOps::append_child(&mut arena, header_row, cell1);

        let cell2 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell2_text = arena.alloc(Node::with_value(NodeValue::make_text("Age")));
        TreeOps::append_child(&mut arena, cell2, cell2_text);
        TreeOps::append_child(&mut arena, header_row, cell2);

        TreeOps::append_child(&mut arena, table, header_row);

        // Data row
        let data_row = arena.alloc(Node::with_value(NodeValue::TableRow(false)));
        let cell3 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell3_text = arena.alloc(Node::with_value(NodeValue::make_text("Alice")));
        TreeOps::append_child(&mut arena, cell3, cell3_text);
        TreeOps::append_child(&mut arena, data_row, cell3);

        let cell4 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell4_text = arena.alloc(Node::with_value(NodeValue::make_text("30")));
        TreeOps::append_child(&mut arena, cell4, cell4_text);
        TreeOps::append_child(&mut arena, data_row, cell4);

        TreeOps::append_child(&mut arena, table, data_row);
        TreeOps::append_child(&mut arena, root, table);

        // Create code block
        let code_block = arena.alloc(Node::with_value(NodeValue::CodeBlock(Box::new(
            NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            },
        ))));
        TreeOps::append_child(&mut arena, root, code_block);

        let mut formatter = Formatter::new();
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        println!("Table + code block result: {:?}", result);

        // Check that table is present
        assert!(
            result.contains("Name") && result.contains("Age"),
            "Table header should contain Name and Age. Result: {:?}",
            result
        );

        // Check that code block is present
        assert!(
            result.contains("```rust"),
            "Should contain code block start. Result: {:?}",
            result
        );

        // Check that there's a blank line between table and code block
        // The table should end with a blank line before the code block
        assert!(
            result.contains("30  |\n\n```rust"),
            "Should have blank line between table and code block. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_heading() {
        use crate::core::nodes::NodeHeading;
        use crate::render::commonmark::CommonMarkNodeFormatter;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create heading
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Title")));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let mut formatter = Formatter::new();
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("# Title"),
            "Should contain heading. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_list() {
        use crate::core::nodes::{ListDelimType, ListType, NodeList};
        use crate::render::commonmark::CommonMarkNodeFormatter;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create bullet list
        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        })));

        let item1 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        })));
        let item1_text = arena.alloc(Node::with_value(NodeValue::make_text("Item 1")));
        TreeOps::append_child(&mut arena, item1, item1_text);
        TreeOps::append_child(&mut arena, list, item1);

        let item2 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        })));
        let item2_text = arena.alloc(Node::with_value(NodeValue::make_text("Item 2")));
        TreeOps::append_child(&mut arena, item2, item2_text);
        TreeOps::append_child(&mut arena, list, item2);

        TreeOps::append_child(&mut arena, root, list);

        let mut formatter = Formatter::new();
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("- Item 1"),
            "Should contain list item 1. Result: {:?}",
            result
        );
        assert!(
            result.contains("- Item 2"),
            "Should contain list item 2. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_code_block() {
        use crate::core::nodes::NodeCodeBlock;
        use crate::render::commonmark::CommonMarkNodeFormatter;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create code block
        let code_block = arena.alloc(Node::with_value(NodeValue::CodeBlock(Box::new(
            NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            },
        ))));
        TreeOps::append_child(&mut arena, root, code_block);

        let mut formatter = Formatter::new();
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("```rust"),
            "Should contain code block start. Result: {:?}",
            result
        );
        assert!(
            result.contains("fn main() {}"),
            "Should contain code. Result: {:?}",
            result
        );
        assert!(
            result.contains("```"),
            "Should contain code block end. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_link() {
        use crate::core::nodes::NodeLink;
        use crate::render::commonmark::CommonMarkNodeFormatter;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create paragraph with link
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
        }))));
        let link_text = arena.alloc(Node::with_value(NodeValue::make_text("Example")));
        TreeOps::append_child(&mut arena, link, link_text);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, root, para);

        let mut formatter = Formatter::new();
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("[Example]"),
            "Should contain link text. Result: {:?}",
            result
        );
        assert!(
            result.contains("https://example.com"),
            "Should contain link URL. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_blockquote() {
        use crate::render::commonmark::CommonMarkNodeFormatter;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create blockquote
        let quote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Quote text")));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, quote, para);
        TreeOps::append_child(&mut arena, root, quote);

        let mut formatter = Formatter::new();
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("> Quote text"),
            "Should contain blockquote. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_empty_document() {
        use crate::render::commonmark::CommonMarkNodeFormatter;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let mut formatter = Formatter::new();
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(result.is_empty() || result.trim().is_empty(), "Empty document should produce empty or whitespace-only output. Result: {:?}", result);
    }
}
