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

    #[test]
    fn test_phase_variants_equality() {
        assert_eq!(FormattingPhase::Collect, FormattingPhase::Collect);
        assert_ne!(FormattingPhase::Collect, FormattingPhase::Document);
        assert_ne!(FormattingPhase::DocumentTop, FormattingPhase::DocumentBottom);
    }

    #[test]
    fn test_phase_clone() {
        let phase = FormattingPhase::DocumentTop;
        let cloned = phase.clone();
        assert_eq!(phase, cloned);
    }

    #[test]
    fn test_phase_copy() {
        let phase = FormattingPhase::DocumentBottom;
        let copied = phase;
        assert_eq!(phase, copied);
    }

    #[test]
    fn test_before_document_phases() {
        let before = FormattingPhase::before_document();
        assert_eq!(before.len(), 3);
        assert!(before.contains(&FormattingPhase::Collect));
        assert!(before.contains(&FormattingPhase::DocumentFirst));
        assert!(before.contains(&FormattingPhase::DocumentTop));
        assert!(!before.contains(&FormattingPhase::Document));
        assert!(!before.contains(&FormattingPhase::DocumentBottom));
    }

    #[test]
    fn test_after_document_phases() {
        let after = FormattingPhase::after_document();
        assert_eq!(after.len(), 1);
        assert!(after.contains(&FormattingPhase::DocumentBottom));
    }

    #[test]
    fn test_all_phases_complete() {
        let all = FormattingPhase::all();
        assert_eq!(all.len(), 5);

        // Check all variants are included
        let variants: Vec<_> = all.to_vec();
        assert!(variants.contains(&FormattingPhase::Collect));
        assert!(variants.contains(&FormattingPhase::DocumentFirst));
        assert!(variants.contains(&FormattingPhase::DocumentTop));
        assert!(variants.contains(&FormattingPhase::Document));
        assert!(variants.contains(&FormattingPhase::DocumentBottom));
    }

    #[test]
    fn test_phase_names_all() {
        assert_eq!(FormattingPhase::Collect.name(), "Collect");
        assert_eq!(FormattingPhase::DocumentFirst.name(), "DocumentFirst");
        assert_eq!(FormattingPhase::DocumentTop.name(), "DocumentTop");
        assert_eq!(FormattingPhase::Document.name(), "Document");
        assert_eq!(FormattingPhase::DocumentBottom.name(), "DocumentBottom");
    }

    #[test]
    fn test_is_before_and_after_combinations() {
        // Collect is before document, not after
        assert!(FormattingPhase::Collect.is_before_document());
        assert!(!FormattingPhase::Collect.is_after_document());
        assert!(!FormattingPhase::Collect.is_document());

        // Document is neither before nor after
        assert!(!FormattingPhase::Document.is_before_document());
        assert!(!FormattingPhase::Document.is_after_document());
        assert!(FormattingPhase::Document.is_document());

        // DocumentBottom is after, not before
        assert!(!FormattingPhase::DocumentBottom.is_before_document());
        assert!(FormattingPhase::DocumentBottom.is_after_document());
        assert!(!FormattingPhase::DocumentBottom.is_document());
    }

    #[test]
    fn test_is_collection() {
        assert!(FormattingPhase::Collect.is_collection());
        assert!(!FormattingPhase::DocumentFirst.is_collection());
        assert!(!FormattingPhase::Document.is_collection());
        assert!(!FormattingPhase::DocumentBottom.is_collection());
    }
}
