//! Node repository formatter for reference links and footnotes
//!
//! This module provides functionality for formatting reference-style links
//! and footnotes, inspired by flexmark-java's NodeRepositoryFormatter.
//! It handles the collection, sorting, and placement of references.

use super::context::NodeFormatterContext;
use super::phase::FormattingPhase;
use super::writer::MarkdownWriter;
use crate::core::arena::NodeId;
use crate::options::format::{ElementPlacement, ElementPlacementSort};
use std::collections::HashMap;

/// A reference entry in the repository
#[derive(Debug, Clone)]
pub struct ReferenceEntry {
    /// The reference label (e.g., "ref1" for "[ref1]: url")
    pub label: String,
    /// The reference URL
    pub url: String,
    /// The reference title (optional)
    pub title: Option<String>,
    /// Whether this reference is used in the document
    pub is_used: bool,
    /// The node ID of the reference definition
    pub node_id: Option<NodeId>,
}

/// Repository for collecting and managing references
#[derive(Debug, Clone, Default)]
pub struct ReferenceRepository {
    /// Map from label to reference entry
    references: HashMap<String, ReferenceEntry>,
    /// Order of references as they appear in the document
    order: Vec<String>,
}

impl ReferenceRepository {
    /// Create a new empty repository
    pub fn new() -> Self {
        Self {
            references: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Add a reference to the repository
    pub fn add(&mut self, entry: ReferenceEntry) {
        let label = entry.label.clone();
        if !self.references.contains_key(&label) {
            self.order.push(label.clone());
        }
        self.references.insert(label, entry);
    }

    /// Get a reference by label
    pub fn get(&self, label: &str) -> Option<&ReferenceEntry> {
        self.references.get(label)
    }

    /// Get a mutable reference by label
    pub fn get_mut(&mut self, label: &str) -> Option<&mut ReferenceEntry> {
        self.references.get_mut(label)
    }

    /// Check if a reference exists
    pub fn contains(&self, label: &str) -> bool {
        self.references.contains_key(label)
    }

    /// Get all references in order
    pub fn get_all(&self) -> Vec<&ReferenceEntry> {
        self.order
            .iter()
            .filter_map(|label| self.references.get(label))
            .collect()
    }

    /// Get all used references
    pub fn get_used(&self) -> Vec<&ReferenceEntry> {
        self.get_all().into_iter().filter(|r| r.is_used).collect()
    }

    /// Get all unused references
    pub fn get_unused(&self) -> Vec<&ReferenceEntry> {
        self.get_all().into_iter().filter(|r| !r.is_used).collect()
    }

    /// Mark a reference as used
    pub fn mark_used(&mut self, label: &str) {
        if let Some(entry) = self.references.get_mut(label) {
            entry.is_used = true;
        }
    }

    /// Get the number of references
    pub fn len(&self) -> usize {
        self.references.len()
    }

    /// Check if the repository is empty
    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }

    /// Clear the repository
    pub fn clear(&mut self) {
        self.references.clear();
        self.order.clear();
    }
}

/// Trait for formatters that handle reference-style links/footnotes
///
/// This trait provides the interface for collecting, sorting, and rendering
/// reference definitions (like link references or footnote definitions).
pub trait NodeRepositoryFormatter {
    /// Get the formatting phases this formatter participates in
    fn get_formatting_phases(&self) -> Vec<FormattingPhase> {
        vec![
            FormattingPhase::Collect,
            FormattingPhase::DocumentTop,
            FormattingPhase::DocumentBottom,
        ]
    }

    /// Get the reference repository
    fn get_repository(&self) -> &ReferenceRepository;

    /// Get a mutable reference to the repository
    fn get_repository_mut(&mut self) -> &mut ReferenceRepository;

    /// Get the placement strategy for references
    fn get_reference_placement(&self) -> ElementPlacement;

    /// Get the sorting strategy for references
    fn get_reference_sort(&self) -> ElementPlacementSort;

    /// Render a single reference entry
    fn render_reference(
        &self,
        entry: &ReferenceEntry,
        ctx: &dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
    );

    /// Check if references should be made unique across documents
    fn make_references_unique(&self) -> bool {
        true
    }

    /// Render all references according to placement and sort options
    fn render_references(
        &self,
        ctx: &dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
    ) {
        let references = self.get_sorted_references();

        if references.is_empty() {
            return;
        }

        writer.blank_line();
        for entry in references {
            self.render_reference(entry, ctx, writer);
        }
        writer.blank_line();
    }

    /// Get references sorted according to the sort option
    fn get_sorted_references(&self) -> Vec<&ReferenceEntry> {
        let sort = self.get_reference_sort();
        let mut references = self.get_repository().get_all();

        match sort {
            ElementPlacementSort::AsIs => {
                // Keep original order
            }
            ElementPlacementSort::Sort => {
                // Sort by label
                references.sort_by(|a, b| a.label.cmp(&b.label));
            }
            ElementPlacementSort::SortUnusedLast => {
                // Sort used first, then unused, each group sorted by label
                references.sort_by(|a, b| match (a.is_used, b.is_used) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.label.cmp(&b.label),
                });
            }
            ElementPlacementSort::SortDeleteUnused => {
                // Only return used references, sorted by label
                references.retain(|r| r.is_used);
                references.sort_by(|a, b| a.label.cmp(&b.label));
            }
            ElementPlacementSort::DeleteUnused => {
                // Only return used references, in original order
                references.retain(|r| r.is_used);
            }
        }

        references
    }

    /// Handle the document rendering phase
    fn handle_phase(
        &mut self,
        phase: FormattingPhase,
        ctx: &dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
    ) {
        match phase {
            FormattingPhase::DocumentTop => {
                // Render references at document top if configured
                if self.get_reference_placement() == ElementPlacement::DocumentTop {
                    self.render_references(ctx, writer);
                }
            }
            FormattingPhase::DocumentBottom => {
                // Render references at document bottom if configured
                if self.get_reference_placement() == ElementPlacement::DocumentBottom {
                    self.render_references(ctx, writer);
                }
            }
            _ => {}
        }
    }
}

/// Default implementation of a link reference formatter
#[derive(Debug)]
pub struct LinkReferenceFormatter {
    repository: ReferenceRepository,
    placement: ElementPlacement,
    sort: ElementPlacementSort,
}

impl LinkReferenceFormatter {
    /// Create a new link reference formatter
    pub fn new(placement: ElementPlacement, sort: ElementPlacementSort) -> Self {
        Self {
            repository: ReferenceRepository::new(),
            placement,
            sort,
        }
    }

    /// Create a new formatter with default settings
    pub fn with_defaults() -> Self {
        Self::new(ElementPlacement::DocumentBottom, ElementPlacementSort::AsIs)
    }
}

impl Default for LinkReferenceFormatter {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl NodeRepositoryFormatter for LinkReferenceFormatter {
    fn get_repository(&self) -> &ReferenceRepository {
        &self.repository
    }

    fn get_repository_mut(&mut self) -> &mut ReferenceRepository {
        &mut self.repository
    }

    fn get_reference_placement(&self) -> ElementPlacement {
        self.placement
    }

    fn get_reference_sort(&self) -> ElementPlacementSort {
        self.sort
    }

    fn render_reference(
        &self,
        entry: &ReferenceEntry,
        _ctx: &dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
    ) {
        // Render as: [label]: url "title"
        writer.append(format!("[{}]: {}", entry.label, entry.url));

        if let Some(title) = &entry.title {
            // Escape quotes in title
            let escaped_title = title.replace('"', "\\\"");
            writer.append(format!(" \"{}\"", escaped_title));
        }

        writer.line();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_repository() {
        let mut repo = ReferenceRepository::new();

        let entry1 = ReferenceEntry {
            label: "ref1".to_string(),
            url: "https://example.com".to_string(),
            title: Some("Example".to_string()),
            is_used: false,
            node_id: None,
        };

        let entry2 = ReferenceEntry {
            label: "ref2".to_string(),
            url: "https://test.com".to_string(),
            title: None,
            is_used: true,
            node_id: None,
        };

        repo.add(entry1);
        repo.add(entry2);

        assert_eq!(repo.len(), 2);
        assert!(repo.contains("ref1"));
        assert!(repo.contains("ref2"));

        let all = repo.get_all();
        assert_eq!(all.len(), 2);

        let used = repo.get_used();
        assert_eq!(used.len(), 1);
        assert_eq!(used[0].label, "ref2");

        let unused = repo.get_unused();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].label, "ref1");
    }

    #[test]
    fn test_reference_repository_mark_used() {
        let mut repo = ReferenceRepository::new();

        repo.add(ReferenceEntry {
            label: "ref1".to_string(),
            url: "https://example.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        assert!(!repo.get("ref1").unwrap().is_used);
        repo.mark_used("ref1");
        assert!(repo.get("ref1").unwrap().is_used);
    }

    #[test]
    fn test_link_reference_formatter_creation() {
        let formatter = LinkReferenceFormatter::with_defaults();
        assert_eq!(
            formatter.get_reference_placement(),
            ElementPlacement::DocumentBottom
        );
        assert_eq!(formatter.get_reference_sort(), ElementPlacementSort::AsIs);
        assert!(formatter.get_repository().is_empty());
    }

    #[test]
    fn test_link_reference_formatter_custom() {
        let formatter = LinkReferenceFormatter::new(
            ElementPlacement::DocumentTop,
            ElementPlacementSort::Sort,
        );
        assert_eq!(
            formatter.get_reference_placement(),
            ElementPlacement::DocumentTop
        );
        assert_eq!(formatter.get_reference_sort(), ElementPlacementSort::Sort);
    }

    #[test]
    fn test_sorted_references() {
        let mut formatter = LinkReferenceFormatter::new(
            ElementPlacement::DocumentBottom,
            ElementPlacementSort::Sort,
        );

        formatter.repository.add(ReferenceEntry {
            label: "zebra".to_string(),
            url: "https://zebra.com".to_string(),
            title: None,
            is_used: true,
            node_id: None,
        });

        formatter.repository.add(ReferenceEntry {
            label: "alpha".to_string(),
            url: "https://alpha.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        formatter.repository.add(ReferenceEntry {
            label: "beta".to_string(),
            url: "https://beta.com".to_string(),
            title: None,
            is_used: true,
            node_id: None,
        });

        let sorted = formatter.get_sorted_references();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].label, "alpha");
        assert_eq!(sorted[1].label, "beta");
        assert_eq!(sorted[2].label, "zebra");
    }

    #[test]
    fn test_sorted_references_unused_last() {
        let mut formatter = LinkReferenceFormatter::new(
            ElementPlacement::DocumentBottom,
            ElementPlacementSort::SortUnusedLast,
        );

        formatter.repository.add(ReferenceEntry {
            label: "zebra".to_string(),
            url: "https://zebra.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        formatter.repository.add(ReferenceEntry {
            label: "alpha".to_string(),
            url: "https://alpha.com".to_string(),
            title: None,
            is_used: true,
            node_id: None,
        });

        formatter.repository.add(ReferenceEntry {
            label: "beta".to_string(),
            url: "https://beta.com".to_string(),
            title: None,
            is_used: true,
            node_id: None,
        });

        let sorted = formatter.get_sorted_references();
        assert_eq!(sorted.len(), 3);
        // Used references first, sorted alphabetically
        assert_eq!(sorted[0].label, "alpha");
        assert_eq!(sorted[1].label, "beta");
        // Unused references last
        assert_eq!(sorted[2].label, "zebra");
    }

    #[test]
    fn test_sorted_references_delete_unused() {
        let mut formatter = LinkReferenceFormatter::new(
            ElementPlacement::DocumentBottom,
            ElementPlacementSort::DeleteUnused,
        );

        formatter.repository.add(ReferenceEntry {
            label: "unused".to_string(),
            url: "https://unused.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        formatter.repository.add(ReferenceEntry {
            label: "used".to_string(),
            url: "https://used.com".to_string(),
            title: None,
            is_used: true,
            node_id: None,
        });

        let sorted = formatter.get_sorted_references();
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0].label, "used");
    }

    #[test]
    fn test_reference_repository_default() {
        let repo: ReferenceRepository = Default::default();
        assert!(repo.is_empty());
        assert_eq!(repo.len(), 0);
    }

    #[test]
    fn test_reference_repository_new() {
        let repo = ReferenceRepository::new();
        assert!(repo.is_empty());
        assert_eq!(repo.len(), 0);
    }

    #[test]
    fn test_reference_repository_get_mut() {
        let mut repo = ReferenceRepository::new();

        repo.add(ReferenceEntry {
            label: "ref1".to_string(),
            url: "https://example.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        // Get mutable reference and modify it
        if let Some(entry) = repo.get_mut("ref1") {
            entry.is_used = true;
        }

        assert!(repo.get("ref1").unwrap().is_used);
    }

    #[test]
    fn test_reference_repository_update_existing() {
        let mut repo = ReferenceRepository::new();

        repo.add(ReferenceEntry {
            label: "ref1".to_string(),
            url: "https://old.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        // Add with same label - should update but not change order
        repo.add(ReferenceEntry {
            label: "ref1".to_string(),
            url: "https://new.com".to_string(),
            title: Some("Updated".to_string()),
            is_used: true,
            node_id: None,
        });

        assert_eq!(repo.len(), 1);
        let entry = repo.get("ref1").unwrap();
        assert_eq!(entry.url, "https://new.com");
        assert_eq!(entry.title, Some("Updated".to_string()));
        assert!(entry.is_used);
    }

    #[test]
    fn test_reference_repository_clear() {
        let mut repo = ReferenceRepository::new();

        repo.add(ReferenceEntry {
            label: "ref1".to_string(),
            url: "https://example.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        assert!(!repo.is_empty());

        repo.clear();

        assert!(repo.is_empty());
        assert_eq!(repo.len(), 0);
        assert!(!repo.contains("ref1"));
    }

    #[test]
    fn test_reference_repository_get_nonexistent() {
        let repo = ReferenceRepository::new();
        assert!(repo.get("nonexistent").is_none());
        assert!(!repo.contains("nonexistent"));
    }

    #[test]
    fn test_reference_repository_mark_used_nonexistent() {
        let mut repo = ReferenceRepository::new();
        // Should not panic
        repo.mark_used("nonexistent");
    }

    #[test]
    fn test_reference_repository_order_preserved() {
        let mut repo = ReferenceRepository::new();

        repo.add(ReferenceEntry {
            label: "first".to_string(),
            url: "https://first.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        repo.add(ReferenceEntry {
            label: "second".to_string(),
            url: "https://second.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        repo.add(ReferenceEntry {
            label: "third".to_string(),
            url: "https://third.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        let all = repo.get_all();
        assert_eq!(all[0].label, "first");
        assert_eq!(all[1].label, "second");
        assert_eq!(all[2].label, "third");
    }

    #[test]
    fn test_reference_entry_clone() {
        let entry = ReferenceEntry {
            label: "ref1".to_string(),
            url: "https://example.com".to_string(),
            title: Some("Title".to_string()),
            is_used: true,
            node_id: None,
        };

        let cloned = entry.clone();
        assert_eq!(entry.label, cloned.label);
        assert_eq!(entry.url, cloned.url);
        assert_eq!(entry.title, cloned.title);
        assert_eq!(entry.is_used, cloned.is_used);
    }

    #[test]
    fn test_link_reference_formatter_default() {
        let formatter: LinkReferenceFormatter = Default::default();
        assert_eq!(
            formatter.get_reference_placement(),
            ElementPlacement::DocumentBottom
        );
        assert_eq!(formatter.get_reference_sort(), ElementPlacementSort::AsIs);
        assert!(formatter.get_repository().is_empty());
    }

    #[test]
    fn test_link_reference_formatter_make_references_unique() {
        let formatter = LinkReferenceFormatter::with_defaults();
        assert!(formatter.make_references_unique());
    }

    #[test]
    fn test_link_reference_formatter_get_formatting_phases() {
        let formatter = LinkReferenceFormatter::with_defaults();
        let phases = formatter.get_formatting_phases();

        assert!(phases.contains(&FormattingPhase::Collect));
        assert!(phases.contains(&FormattingPhase::DocumentTop));
        assert!(phases.contains(&FormattingPhase::DocumentBottom));
    }

    #[test]
    fn test_sorted_references_sort_delete_unused() {
        let mut formatter = LinkReferenceFormatter::new(
            ElementPlacement::DocumentBottom,
            ElementPlacementSort::SortDeleteUnused,
        );

        formatter.repository.add(ReferenceEntry {
            label: "zebra".to_string(),
            url: "https://zebra.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        formatter.repository.add(ReferenceEntry {
            label: "alpha".to_string(),
            url: "https://alpha.com".to_string(),
            title: None,
            is_used: true,
            node_id: None,
        });

        formatter.repository.add(ReferenceEntry {
            label: "beta".to_string(),
            url: "https://beta.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        let sorted = formatter.get_sorted_references();
        // Only used references, sorted alphabetically
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0].label, "alpha");
    }

    #[test]
    fn test_sorted_references_empty() {
        let formatter = LinkReferenceFormatter::with_defaults();
        let sorted = formatter.get_sorted_references();
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_sorted_references_all_unused() {
        let mut formatter = LinkReferenceFormatter::new(
            ElementPlacement::DocumentBottom,
            ElementPlacementSort::SortUnusedLast,
        );

        formatter.repository.add(ReferenceEntry {
            label: "zebra".to_string(),
            url: "https://zebra.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        formatter.repository.add(ReferenceEntry {
            label: "alpha".to_string(),
            url: "https://alpha.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        let sorted = formatter.get_sorted_references();
        // All unused, just sorted alphabetically
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].label, "alpha");
        assert_eq!(sorted[1].label, "zebra");
    }

    #[test]
    fn test_sorted_references_as_is() {
        let mut formatter = LinkReferenceFormatter::new(
            ElementPlacement::DocumentBottom,
            ElementPlacementSort::AsIs,
        );

        formatter.repository.add(ReferenceEntry {
            label: "zebra".to_string(),
            url: "https://zebra.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        formatter.repository.add(ReferenceEntry {
            label: "alpha".to_string(),
            url: "https://alpha.com".to_string(),
            title: None,
            is_used: false,
            node_id: None,
        });

        let sorted = formatter.get_sorted_references();
        // Original order preserved
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].label, "zebra");
        assert_eq!(sorted[1].label, "alpha");
    }
}
