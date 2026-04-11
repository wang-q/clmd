//! Multi-phase rendering support
//!
//! This module defines the formatting phases and phased formatters for
//! multi-phase document rendering, inspired by flexmark-java.

use crate::core::arena::NodeId;
use crate::render::commonmark::context::NodeFormatterContext;
use crate::render::commonmark::node::{NodeFormatter, NodeFormattingHandler};
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

    /// Render all formatters for a specific phase
    pub fn render_phase(
        &self,
        context: &mut dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
        root: NodeId,
        phase: FormattingPhase,
    ) {
        for formatter in &self.formatters {
            if formatter.get_formatting_phases().contains(&phase) {
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
}

impl NodeFormatter for SimplePhasedFormatter {
    fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
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
    fn test_default() {
        let phase: FormattingPhase = Default::default();
        assert!(matches!(phase, FormattingPhase::Document));
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
