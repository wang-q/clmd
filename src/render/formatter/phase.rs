//! Formatting phase definitions
//!
//! This module defines the different phases of the formatting process,
//! inspired by flexmark-java's FormattingPhase enum.

/// Formatting phase for multi-phase rendering
///
/// The formatter can process documents in multiple phases, allowing
/// for collection of information before the main rendering pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FormattingPhase {
    /// Collection phase - gather information about the document
    ///
    /// This phase is used to collect information about the document structure,
    /// such as identifying unused references, footnotes, etc.
    Collect,

    /// Document first phase - first pass over the document
    ///
    /// This phase runs before the main document rendering.
    DocumentFirst,

    /// Document top phase - render elements at the top of the document
    ///
    /// This phase is used to render elements that should appear at the
    /// top of the document, such as collected references.
    DocumentTop,

    /// Document phase - main document rendering
    ///
    /// This is the main rendering phase where most content is rendered.
    #[default]
    Document,

    /// Document bottom phase - render elements at the bottom of the document
    ///
    /// This phase is used to render elements that should appear at the
    /// bottom of the document, such as footnotes or references.
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
}
