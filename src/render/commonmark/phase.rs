//! Multi-phase rendering support
//!
//! This module defines the formatting phases and phased formatters for
//! multi-phase document rendering, inspired by flexmark-java.

use crate::core::arena::NodeId;
use crate::render::commonmark::context::NodeFormatterContext;
use crate::render::commonmark::node::{
    NodeFormatter, NodeFormattingHandler, NodeValueType,
};
use crate::render::commonmark::writer::MarkdownWriter;

// ============================================================================
// Formatting Phase Definition
// ============================================================================

/// Formatting phase for multi-phase rendering
///
/// The formatter can process documents in multiple phases, allowing
/// for collection of information before the main rendering pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FormattingPhase {
    /// Collection phase - gather information about the document
    Collect,
    /// Document first phase - first pass over the document
    DocumentFirst,
    /// Document top phase - render elements at the top of the document
    DocumentTop,
    /// Document phase - main document rendering
    #[default]
    Document,
    /// Document bottom phase - render elements at the bottom of the document
    DocumentBottom,
}

impl FormattingPhase {
    /// Get all phases in order
    pub fn all() -> &'static [FormattingPhase] {
        &[
            FormattingPhase::Collect,
            FormattingPhase::DocumentFirst,
            FormattingPhase::DocumentTop,
            FormattingPhase::Document,
            FormattingPhase::DocumentBottom,
        ]
    }

    /// Get phases that run before the main document rendering
    pub fn before_document() -> &'static [FormattingPhase] {
        &[
            FormattingPhase::Collect,
            FormattingPhase::DocumentFirst,
            FormattingPhase::DocumentTop,
        ]
    }

    /// Get phases that run after the main document rendering
    pub fn after_document() -> &'static [FormattingPhase] {
        &[FormattingPhase::DocumentBottom]
    }

    /// Check if this phase runs before the main document rendering
    pub fn is_before_document(&self) -> bool {
        matches!(
            self,
            FormattingPhase::Collect
                | FormattingPhase::DocumentFirst
                | FormattingPhase::DocumentTop
        )
    }

    /// Check if this phase runs after the main document rendering
    pub fn is_after_document(&self) -> bool {
        matches!(self, FormattingPhase::DocumentBottom)
    }

    /// Check if this is the main document phase
    pub fn is_document(&self) -> bool {
        matches!(self, FormattingPhase::Document)
    }

    /// Check if this is a collection phase
    pub fn is_collection(&self) -> bool {
        matches!(self, FormattingPhase::Collect)
    }

    /// Get the display name for this phase
    pub fn name(&self) -> &'static str {
        match self {
            FormattingPhase::Collect => "Collect",
            FormattingPhase::DocumentFirst => "DocumentFirst",
            FormattingPhase::DocumentTop => "DocumentTop",
            FormattingPhase::Document => "Document",
            FormattingPhase::DocumentBottom => "DocumentBottom",
        }
    }
}

// ============================================================================
// Phased Node Formatter
// ============================================================================

/// A node formatter that supports multi-phase rendering
///
/// Phased formatters can participate in different phases of the rendering
/// process, allowing for collection of information before the main rendering
/// pass, or for rendering elements at specific positions in the document.
pub trait PhasedNodeFormatter: NodeFormatter {
    /// Get the formatting phases this formatter participates in
    fn get_formatting_phases(&self) -> Vec<FormattingPhase>;

    /// Render the document for a specific phase
    fn render_document(
        &self,
        context: &mut dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
        root: NodeId,
        phase: FormattingPhase,
    );

    /// Check if this formatter participates in a specific phase
    fn participates_in_phase(&self, phase: FormattingPhase) -> bool {
        self.get_formatting_phases().contains(&phase)
    }

    /// Check if this formatter participates in any pre-document phases
    fn has_pre_document_phases(&self) -> bool {
        self.get_formatting_phases()
            .iter()
            .any(|p| p.is_before_document())
    }

    /// Check if this formatter participates in any post-document phases
    fn has_post_document_phases(&self) -> bool {
        self.get_formatting_phases()
            .iter()
            .any(|p| p.is_after_document())
    }

    /// Check if this formatter participates in the collection phase
    fn has_collection_phase(&self) -> bool {
        self.get_formatting_phases()
            .iter()
            .any(|p| p.is_collection())
    }
}

// ============================================================================
// Composed Phased Formatter
// ============================================================================

/// A phased formatter that delegates to a collection of formatters
pub struct ComposedPhasedFormatter {
    formatters: Vec<Box<dyn PhasedNodeFormatter>>,
}

impl std::fmt::Debug for ComposedPhasedFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComposedPhasedFormatter")
            .field("formatters", &self.formatters.len())
            .finish()
    }
}

impl ComposedPhasedFormatter {
    /// Create a new composed phased formatter
    pub fn new() -> Self {
        Self {
            formatters: Vec::new(),
        }
    }

    /// Add a phased formatter to the composition
    pub fn add_formatter(&mut self, formatter: Box<dyn PhasedNodeFormatter>) {
        self.formatters.push(formatter)
    }

    /// Get all formatters that participate in a specific phase
    pub fn get_formatters_for_phase(
        &self,
        phase: FormattingPhase,
    ) -> Vec<&dyn PhasedNodeFormatter> {
        self.formatters
            .iter()
            .filter(|f| f.participates_in_phase(phase))
            .map(|f| f.as_ref())
            .collect()
    }

    /// Render all formatters for a specific phase
    pub fn render_phase(
        &self,
        context: &mut dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
        root: NodeId,
        phase: FormattingPhase,
    ) {
        for formatter in &self.formatters {
            if formatter.participates_in_phase(phase) {
                formatter.render_document(context, writer, root, phase);
            }
        }
    }

    /// Get all phases used by any formatter
    pub fn get_all_phases(&self) -> Vec<FormattingPhase> {
        let mut phases = Vec::new();
        for formatter in &self.formatters {
            for phase in formatter.get_formatting_phases() {
                if !phases.contains(&phase) {
                    phases.push(phase);
                }
            }
        }
        phases.sort_by_key(|p| match p {
            FormattingPhase::Collect => 0,
            FormattingPhase::DocumentFirst => 1,
            FormattingPhase::DocumentTop => 2,
            FormattingPhase::Document => 3,
            FormattingPhase::DocumentBottom => 4,
        });
        phases
    }
}

impl Default for ComposedPhasedFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeFormatter for ComposedPhasedFormatter {
    fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
        self.formatters
            .iter()
            .flat_map(|f| f.get_node_formatting_handlers())
            .collect()
    }

    fn get_node_classes(&self) -> Vec<NodeValueType> {
        self.formatters
            .iter()
            .flat_map(|f| f.get_node_classes())
            .collect()
    }

    fn get_block_quote_like_prefix_char(&self) -> Option<char> {
        self.formatters
            .iter()
            .filter_map(|f| f.get_block_quote_like_prefix_char())
            .next()
    }
}

impl PhasedNodeFormatter for ComposedPhasedFormatter {
    fn get_formatting_phases(&self) -> Vec<FormattingPhase> {
        self.get_all_phases()
    }

    fn render_document(
        &self,
        context: &mut dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
        root: NodeId,
        phase: FormattingPhase,
    ) {
        self.render_phase(context, writer, root, phase);
    }
}

// ============================================================================
// Simple Phased Formatter
// ============================================================================

/// A simple phased formatter that only participates in specific phases
#[allow(clippy::type_complexity)]
pub struct SimplePhasedFormatter {
    phases: Vec<FormattingPhase>,
    render_fn: Box<
        dyn Fn(
                &mut dyn NodeFormatterContext,
                &mut MarkdownWriter,
                NodeId,
                FormattingPhase,
            ) + Send
            + Sync,
    >,
}

impl std::fmt::Debug for SimplePhasedFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePhasedFormatter")
            .field("phases", &self.phases)
            .finish_non_exhaustive()
    }
}

impl SimplePhasedFormatter {
    /// Create a new simple phased formatter
    pub fn new<F>(phases: Vec<FormattingPhase>, render_fn: F) -> Self
    where
        F: Fn(
                &mut dyn NodeFormatterContext,
                &mut MarkdownWriter,
                NodeId,
                FormattingPhase,
            ) + Send
            + Sync
            + 'static,
    {
        Self {
            phases,
            render_fn: Box::new(render_fn),
        }
    }

    /// Create a formatter for the collection phase only
    pub fn for_collection<F>(render_fn: F) -> Self
    where
        F: Fn(&mut dyn NodeFormatterContext, &mut MarkdownWriter, NodeId)
            + Send
            + Sync
            + 'static,
    {
        Self::new(
            vec![FormattingPhase::Collect],
            move |ctx, writer, root, phase| {
                if phase == FormattingPhase::Collect {
                    render_fn(ctx, writer, root);
                }
            },
        )
    }

    /// Create a formatter for the document top phase only
    pub fn for_document_top<F>(render_fn: F) -> Self
    where
        F: Fn(&mut dyn NodeFormatterContext, &mut MarkdownWriter, NodeId)
            + Send
            + Sync
            + 'static,
    {
        Self::new(
            vec![FormattingPhase::DocumentTop],
            move |ctx, writer, root, phase| {
                if phase == FormattingPhase::DocumentTop {
                    render_fn(ctx, writer, root);
                }
            },
        )
    }

    /// Create a formatter for the document bottom phase only
    pub fn for_document_bottom<F>(render_fn: F) -> Self
    where
        F: Fn(&mut dyn NodeFormatterContext, &mut MarkdownWriter, NodeId)
            + Send
            + Sync
            + 'static,
    {
        Self::new(
            vec![FormattingPhase::DocumentBottom],
            move |ctx, writer, root, phase| {
                if phase == FormattingPhase::DocumentBottom {
                    render_fn(ctx, writer, root);
                }
            },
        )
    }
}

impl NodeFormatter for SimplePhasedFormatter {
    fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
        Vec::new()
    }

    fn get_node_classes(&self) -> Vec<NodeValueType> {
        Vec::new()
    }
}

impl PhasedNodeFormatter for SimplePhasedFormatter {
    fn get_formatting_phases(&self) -> Vec<FormattingPhase> {
        self.phases.clone()
    }

    fn render_document(
        &self,
        context: &mut dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
        root: NodeId,
        phase: FormattingPhase,
    ) {
        (self.render_fn)(context, writer, root, phase);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_order() {
        let all = FormattingPhase::all();
        assert_eq!(all.len(), 5);
        assert!(matches!(all[0], FormattingPhase::Collect));
        assert!(matches!(all[1], FormattingPhase::DocumentFirst));
        assert!(matches!(all[2], FormattingPhase::DocumentTop));
        assert!(matches!(all[3], FormattingPhase::Document));
        assert!(matches!(all[4], FormattingPhase::DocumentBottom));
    }

    #[test]
    fn test_phase_checks() {
        assert!(FormattingPhase::Collect.is_before_document());
        assert!(FormattingPhase::DocumentFirst.is_before_document());
        assert!(FormattingPhase::DocumentTop.is_before_document());
        assert!(!FormattingPhase::Document.is_before_document());
        assert!(!FormattingPhase::DocumentBottom.is_before_document());

        assert!(!FormattingPhase::Collect.is_after_document());
        assert!(!FormattingPhase::Document.is_after_document());
        assert!(FormattingPhase::DocumentBottom.is_after_document());

        assert!(FormattingPhase::Document.is_document());
        assert!(!FormattingPhase::Collect.is_document());

        assert!(FormattingPhase::Collect.is_collection());
        assert!(!FormattingPhase::Document.is_collection());
    }

    #[test]
    fn test_phase_names() {
        assert_eq!(FormattingPhase::Collect.name(), "Collect");
        assert_eq!(FormattingPhase::Document.name(), "Document");
        assert_eq!(FormattingPhase::DocumentBottom.name(), "DocumentBottom");
    }

    #[test]
    fn test_default() {
        let phase: FormattingPhase = Default::default();
        assert!(matches!(phase, FormattingPhase::Document));
    }

    #[test]
    fn test_simple_phased_formatter() {
        let formatter = SimplePhasedFormatter::for_collection(|_, _, _| {});
        assert!(formatter.participates_in_phase(FormattingPhase::Collect));
        assert!(!formatter.participates_in_phase(FormattingPhase::Document));
    }

    #[test]
    fn test_composed_phased_formatter() {
        let mut composed = ComposedPhasedFormatter::new();
        composed.add_formatter(Box::new(SimplePhasedFormatter::for_collection(
            |_, _, _| {},
        )));
        composed.add_formatter(Box::new(SimplePhasedFormatter::for_document_top(
            |_, _, _| {},
        )));

        let phases = composed.get_all_phases();
        assert_eq!(phases.len(), 2);
        assert!(phases.contains(&FormattingPhase::Collect));
        assert!(phases.contains(&FormattingPhase::DocumentTop));
    }

    #[test]
    fn test_phased_formatter_checks() {
        struct TestFormatter;
        impl NodeFormatter for TestFormatter {
            fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
                Vec::new()
            }
        }
        impl PhasedNodeFormatter for TestFormatter {
            fn get_formatting_phases(&self) -> Vec<FormattingPhase> {
                vec![
                    FormattingPhase::Collect,
                    FormattingPhase::DocumentTop,
                    FormattingPhase::DocumentBottom,
                ]
            }

            fn render_document(
                &self,
                _context: &mut dyn NodeFormatterContext,
                _writer: &mut MarkdownWriter,
                _root: NodeId,
                _phase: FormattingPhase,
            ) {
            }
        }

        let formatter = TestFormatter;
        assert!(formatter.has_pre_document_phases());
        assert!(formatter.has_post_document_phases());
        assert!(formatter.has_collection_phase());
        assert!(!formatter.participates_in_phase(FormattingPhase::Document));
    }

    #[test]
    fn test_composed_formatter_default() {
        let composed: ComposedPhasedFormatter = Default::default();
        let phases = composed.get_all_phases();
        assert!(phases.is_empty());
    }

    #[test]
    fn test_formatting_phase_ordering() {
        let mut phases = [
            FormattingPhase::Document,
            FormattingPhase::Collect,
            FormattingPhase::DocumentTop,
            FormattingPhase::DocumentBottom,
            FormattingPhase::DocumentFirst,
        ];

        phases.sort_by_key(|p| match p {
            FormattingPhase::Collect => 0,
            FormattingPhase::DocumentFirst => 1,
            FormattingPhase::DocumentTop => 2,
            FormattingPhase::Document => 3,
            FormattingPhase::DocumentBottom => 4,
        });

        assert_eq!(phases[0], FormattingPhase::Collect);
        assert_eq!(phases[1], FormattingPhase::DocumentFirst);
        assert_eq!(phases[2], FormattingPhase::DocumentTop);
        assert_eq!(phases[3], FormattingPhase::Document);
        assert_eq!(phases[4], FormattingPhase::DocumentBottom);
    }
}
