//! Phased node formatter trait definitions
//!
//! This module defines the trait for node formatters that support
//! multi-phase rendering, inspired by flexmark-java's PhasedNodeFormatter.

use crate::core::arena::NodeId;
use crate::formatter::context::NodeFormatterContext;
use crate::formatter::node::{NodeFormatter, NodeFormattingHandler, NodeValueType};
use crate::formatter::phase::FormattingPhase;
use crate::formatter::writer::MarkdownWriter;

/// A node formatter that supports multi-phase rendering
///
/// Phased formatters can participate in different phases of the rendering
/// process, allowing for collection of information before the main rendering
/// pass, or for rendering elements at specific positions in the document.
pub trait PhasedNodeFormatter: NodeFormatter {
    /// Get the formatting phases this formatter participates in
    ///
    /// Returns a list of phases for which this formatter wants to be called.
    /// The formatter will be invoked for each phase in the list.
    fn get_formatting_phases(&self) -> Vec<FormattingPhase>;

    /// Render the document for a specific phase
    ///
    /// This method is called for each phase returned by `get_formatting_phases`.
    /// It allows the formatter to perform phase-specific rendering or collection.
    ///
    /// # Arguments
    ///
    /// * `context` - The formatter context
    /// * `writer` - The Markdown writer for output
    /// * `root` - The root node of the document being rendered
    /// * `phase` - The current formatting phase
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
        self.formatters.push(formatter);
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
        // Sort phases in order
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
        // Return the first non-None prefix char
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

/// A simple phased formatter that only participates in specific phases
/// without providing node handlers
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
        // Simple phased formatters don't provide node handlers
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

/// Standard phases used by most phased formatters
pub const STANDARD_FORMATTING_PHASES: &[FormattingPhase] = &[
    FormattingPhase::Collect,
    FormattingPhase::DocumentTop,
    FormattingPhase::DocumentBottom,
];

/// Phases for reference/footnote formatters
pub const REFERENCE_FORMATTING_PHASES: &[FormattingPhase] = &[
    FormattingPhase::Collect,
    FormattingPhase::DocumentTop,
    FormattingPhase::DocumentBottom,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_phased_formatter() {
        let formatter = SimplePhasedFormatter::for_collection(|_, _, _| {
            // Test callback
        });

        assert!(formatter.participates_in_phase(FormattingPhase::Collect));
        assert!(!formatter.participates_in_phase(FormattingPhase::Document));
    }

    #[test]
    fn test_composed_phased_formatter() {
        let mut composed = ComposedPhasedFormatter::new();

        let formatter1 = SimplePhasedFormatter::for_collection(|_, _, _| {});
        let formatter2 = SimplePhasedFormatter::for_document_top(|_, _, _| {});

        composed.add_formatter(Box::new(formatter1));
        composed.add_formatter(Box::new(formatter2));

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
    fn test_simple_phased_formatter_document_top() {
        let formatter = SimplePhasedFormatter::for_document_top(|_, _, _| {});

        assert!(formatter.participates_in_phase(FormattingPhase::DocumentTop));
        assert!(!formatter.participates_in_phase(FormattingPhase::Collect));
        assert!(!formatter.participates_in_phase(FormattingPhase::DocumentBottom));
    }

    #[test]
    fn test_simple_phased_formatter_document_bottom() {
        let formatter = SimplePhasedFormatter::for_document_bottom(|_, _, _| {});

        assert!(formatter.participates_in_phase(FormattingPhase::DocumentBottom));
        assert!(!formatter.participates_in_phase(FormattingPhase::Collect));
        assert!(!formatter.participates_in_phase(FormattingPhase::DocumentTop));
    }

    #[test]
    fn test_composed_formatter_default() {
        let composed: ComposedPhasedFormatter = Default::default();
        let phases = composed.get_all_phases();
        assert!(phases.is_empty());
    }

    #[test]
    fn test_composed_formatter_get_formatters_for_phase() {
        let mut composed = ComposedPhasedFormatter::new();

        let formatter1 = SimplePhasedFormatter::for_collection(|_, _, _| {});
        let formatter2 = SimplePhasedFormatter::for_collection(|_, _, _| {});
        let formatter3 = SimplePhasedFormatter::for_document_top(|_, _, _| {});

        composed.add_formatter(Box::new(formatter1));
        composed.add_formatter(Box::new(formatter2));
        composed.add_formatter(Box::new(formatter3));

        let collection_formatters =
            composed.get_formatters_for_phase(FormattingPhase::Collect);
        assert_eq!(collection_formatters.len(), 2);

        let top_formatters =
            composed.get_formatters_for_phase(FormattingPhase::DocumentTop);
        assert_eq!(top_formatters.len(), 1);

        let bottom_formatters =
            composed.get_formatters_for_phase(FormattingPhase::DocumentBottom);
        assert!(bottom_formatters.is_empty());
    }

    #[test]
    fn test_phased_formatter_with_multiple_phases() {
        use crate::formatter::node::NodeFormattingHandler;
        use crate::formatter::node::NodeValueType;

        struct MultiPhaseFormatter;

        impl NodeFormatter for MultiPhaseFormatter {
            fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
                Vec::new()
            }

            fn get_node_classes(&self) -> Vec<NodeValueType> {
                Vec::new()
            }
        }

        impl PhasedNodeFormatter for MultiPhaseFormatter {
            fn get_formatting_phases(&self) -> Vec<FormattingPhase> {
                vec![
                    FormattingPhase::Collect,
                    FormattingPhase::DocumentFirst,
                    FormattingPhase::Document,
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

        let formatter = MultiPhaseFormatter;
        assert!(formatter.has_pre_document_phases());
        assert!(!formatter.has_post_document_phases());
        assert!(formatter.participates_in_phase(FormattingPhase::Collect));
        assert!(formatter.participates_in_phase(FormattingPhase::DocumentFirst));
        assert!(formatter.participates_in_phase(FormattingPhase::Document));
        assert!(!formatter.participates_in_phase(FormattingPhase::DocumentTop));
        assert!(!formatter.participates_in_phase(FormattingPhase::DocumentBottom));
    }

    #[test]
    fn test_simple_phased_formatter_render() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        let formatter = SimplePhasedFormatter::for_collection(move |_, _, _| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Verify the formatter has the correct phases
        let phases = formatter.get_formatting_phases();
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0], FormattingPhase::Collect);
    }

    #[test]
    fn test_composed_formatter_node_formatter_impl() {
        let mut composed = ComposedPhasedFormatter::new();

        // Add formatters with different handlers
        let formatter1 = SimplePhasedFormatter::for_collection(|_, _, _| {});
        let formatter2 = SimplePhasedFormatter::for_document_top(|_, _, _| {});

        composed.add_formatter(Box::new(formatter1));
        composed.add_formatter(Box::new(formatter2));

        // Test NodeFormatter implementation
        let handlers = composed.get_node_formatting_handlers();
        assert!(handlers.is_empty()); // SimplePhasedFormatter returns empty handlers

        let classes = composed.get_node_classes();
        assert!(classes.is_empty()); // SimplePhasedFormatter returns empty classes

        let prefix = composed.get_block_quote_like_prefix_char();
        assert_eq!(prefix, None); // SimplePhasedFormatter returns None
    }

    #[test]
    fn test_composed_formatter_phased_impl() {
        let mut composed = ComposedPhasedFormatter::new();

        let formatter1 = SimplePhasedFormatter::for_collection(|_, _, _| {});
        let formatter2 = SimplePhasedFormatter::for_document_top(|_, _, _| {});
        let formatter3 = SimplePhasedFormatter::for_document_bottom(|_, _, _| {});

        composed.add_formatter(Box::new(formatter1));
        composed.add_formatter(Box::new(formatter2));
        composed.add_formatter(Box::new(formatter3));

        // Test PhasedNodeFormatter implementation
        let phases = composed.get_formatting_phases();
        assert_eq!(phases.len(), 3);
        assert!(phases.contains(&FormattingPhase::Collect));
        assert!(phases.contains(&FormattingPhase::DocumentTop));
        assert!(phases.contains(&FormattingPhase::DocumentBottom));
    }

    #[test]
    fn test_formatting_phase_ordering() {
        // Test that phases are sorted correctly
        let mut phases = vec![
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

    #[test]
    fn test_standard_formatting_phases() {
        assert_eq!(STANDARD_FORMATTING_PHASES.len(), 3);
        assert!(STANDARD_FORMATTING_PHASES.contains(&FormattingPhase::Collect));
        assert!(STANDARD_FORMATTING_PHASES.contains(&FormattingPhase::DocumentTop));
        assert!(STANDARD_FORMATTING_PHASES.contains(&FormattingPhase::DocumentBottom));
    }

    #[test]
    fn test_reference_formatting_phases() {
        assert_eq!(REFERENCE_FORMATTING_PHASES.len(), 3);
        assert!(REFERENCE_FORMATTING_PHASES.contains(&FormattingPhase::Collect));
        assert!(REFERENCE_FORMATTING_PHASES.contains(&FormattingPhase::DocumentTop));
        assert!(REFERENCE_FORMATTING_PHASES.contains(&FormattingPhase::DocumentBottom));
    }
}
