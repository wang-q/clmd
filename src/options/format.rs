//! Format options for the Markdown formatter.
//!
//! This module provides comprehensive configuration options for the Markdown formatter,
//! inspired by flexmark-java's FormatterOptions.

use arbitrary::Arbitrary;

/// Heading style options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum HeadingStyle {
    /// Use ATX style headings (# Heading)
    Atx,
    /// Use Setext style headings (Heading\n===)
    Setext,
    /// Keep the original style from the source
    #[default]
    AsIs,
}

/// Bullet list marker options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum BulletMarker {
    /// Use dash (-)
    #[default]
    Dash,
    /// Use asterisk (*)
    Asterisk,
    /// Use plus (+)
    Plus,
    /// Keep any existing marker
    Any,
}

/// Numbered list marker options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum NumberedMarker {
    /// Use period (1.)
    #[default]
    Period,
    /// Use parenthesis (1))
    Paren,
    /// Keep any existing marker
    Any,
}

/// List spacing options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum ListSpacing {
    /// Tight list (no blank lines between items)
    Tight,
    /// Loose list (blank lines between items)
    Loose,
    /// Keep the original spacing
    #[default]
    AsIs,
    /// Loosen tight lists if they contain blank lines
    Loosen,
    /// Tighten loose lists
    Tighten,
}

/// Code fence marker options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum CodeFenceMarker {
    /// Use backticks (`)
    #[default]
    BackTick,
    /// Use tildes (~)
    Tilde,
    /// Keep any existing marker
    Any,
}

/// Block quote marker options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum BlockQuoteMarker {
    /// Keep as-is from source
    #[default]
    AsIs,
    /// Add compact marker (>)
    AddCompact,
    /// Add compact marker with space (> )
    AddCompactWithSpace,
    /// Add spaced marker (> )
    AddSpaced,
}

/// Element placement options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum ElementPlacement {
    /// Keep as-is from source
    #[default]
    AsIs,
    /// Place at document top
    DocumentTop,
    /// Place at document bottom
    DocumentBottom,
    /// Group with first occurrence
    GroupWithFirst,
    /// Group with last occurrence
    GroupWithLast,
}

/// Element placement sort options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum ElementPlacementSort {
    /// Keep as-is from source
    #[default]
    AsIs,
    /// Sort elements
    Sort,
    /// Sort with unused elements last
    SortUnusedLast,
    /// Sort and delete unused elements
    SortDeleteUnused,
    /// Delete unused elements
    DeleteUnused,
}

/// Discretionary text options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum DiscretionaryText {
    /// Add the element
    Add,
    /// Remove the element
    Remove,
    /// Keep as-is from source
    #[default]
    AsIs,
    /// Equalize/equalize the element
    Equalize,
}

/// Trailing marker options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum TrailingMarker {
    /// Add the marker
    Add,
    /// Remove the marker
    Remove,
    /// Keep as-is from source
    #[default]
    AsIs,
    /// Equalize the marker length
    Equalize,
}

/// Alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub enum Alignment {
    /// No alignment
    #[default]
    None,
    /// Left alignment
    Left,
    /// Right alignment
    Right,
    /// Center alignment
    Center,
}

/// Format flags for controlling output behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Arbitrary)]
pub struct FormatFlags {
    /// Trim leading whitespace
    pub trim_leading_whitespace: bool,
    /// Trim trailing whitespace
    pub trim_trailing_whitespace: bool,
    /// Convert tabs to spaces
    pub convert_tabs: bool,
    /// Collapse multiple whitespace
    pub collapse_whitespace: bool,
}

impl FormatFlags {
    /// Default format flags
    pub const DEFAULT: Self = Self {
        trim_leading_whitespace: true,
        trim_trailing_whitespace: true,
        convert_tabs: false,
        collapse_whitespace: false,
    };
}

/// Comprehensive formatter options
#[derive(Debug, Clone, Arbitrary)]
pub struct FormatOptions {
    // Heading options
    /// Heading style preference
    pub heading_style: HeadingStyle,
    /// Add space after ATX marker
    pub space_after_atx_marker: bool,
    /// ATX heading trailing marker handling
    pub atx_heading_trailing_marker: TrailingMarker,
    /// Equalize Setext heading marker length
    pub setext_heading_equalize_marker: bool,
    /// Minimum Setext heading marker length
    pub min_setext_marker_length: usize,

    // List options
    /// Bullet list marker preference
    pub list_bullet_marker: BulletMarker,
    /// Numbered list marker preference
    pub list_numbered_marker: NumberedMarker,
    /// Renumber ordered list items
    pub list_renumber_items: bool,
    /// Reset first item number to 1
    pub list_reset_first_item_number: bool,
    /// Remove empty list items
    pub list_remove_empty_items: bool,
    /// List spacing preference
    pub list_spacing: ListSpacing,
    /// Align numeric list items
    pub list_align_numeric: Alignment,
    /// Add blank line before list
    pub list_add_blank_line_before: bool,
    /// Item content after suffix
    pub lists_item_content_after_suffix: bool,
    /// Item content indent based on marker width (vs fixed indent)
    pub item_content_indent: bool,

    // Code block options
    /// Fenced code block marker type
    pub fenced_code_marker_type: CodeFenceMarker,
    /// Fenced code block marker length
    pub fenced_code_marker_length: usize,
    /// Match closing fence marker to opening
    pub fenced_code_match_closing_marker: bool,
    /// Add space before info string
    pub fenced_code_space_before_info: bool,
    /// Minimize indent for indented code blocks
    pub indented_code_minimize_indent: bool,
    /// Minimize indent for fenced code blocks
    pub fenced_code_minimize_indent: bool,

    // Block quote options
    /// Add blank lines around block quotes
    pub block_quote_blank_lines: bool,
    /// Block quote marker style
    pub block_quote_markers: BlockQuoteMarker,

    // Line break options
    /// Preserve hard line breaks
    pub keep_hard_line_breaks: bool,
    /// Preserve soft line breaks
    pub keep_soft_line_breaks: bool,

    // Link options
    /// Keep image links at start of line
    pub keep_image_links_at_start: bool,
    /// Keep explicit links at start of line
    pub keep_explicit_links_at_start: bool,

    // Reference options
    /// Reference placement
    pub reference_placement: ElementPlacement,
    /// Reference sorting
    pub reference_sort: ElementPlacementSort,
    /// Append transferred references
    pub append_transferred_references: bool,

    // General formatting options
    /// Maximum consecutive blank lines
    pub max_blank_lines: usize,
    /// Maximum trailing blank lines
    pub max_trailing_blank_lines: usize,
    /// Right margin for wrapping (0 = no wrapping)
    pub right_margin: usize,
    /// Thematic break string
    pub thematic_break: Option<String>,
    /// Thematic break marker character
    pub thematic_break_marker: char,
    /// Format flags
    pub format_flags: FormatFlags,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            heading_style: HeadingStyle::default(),
            space_after_atx_marker: true,
            atx_heading_trailing_marker: TrailingMarker::AsIs,
            setext_heading_equalize_marker: true,
            min_setext_marker_length: 3,

            list_bullet_marker: BulletMarker::default(),
            list_numbered_marker: NumberedMarker::default(),
            list_renumber_items: true,
            list_reset_first_item_number: false,
            list_remove_empty_items: false,
            list_spacing: ListSpacing::default(),
            list_align_numeric: Alignment::None,
            list_add_blank_line_before: false,
            lists_item_content_after_suffix: false,
            item_content_indent: true,

            fenced_code_marker_type: CodeFenceMarker::default(),
            fenced_code_marker_length: 3,
            fenced_code_match_closing_marker: true,
            fenced_code_space_before_info: false,
            indented_code_minimize_indent: true,
            fenced_code_minimize_indent: true,

            block_quote_blank_lines: false,
            block_quote_markers: BlockQuoteMarker::default(),

            keep_hard_line_breaks: true,
            keep_soft_line_breaks: true,

            keep_image_links_at_start: false,
            keep_explicit_links_at_start: false,

            reference_placement: ElementPlacement::default(),
            reference_sort: ElementPlacementSort::default(),
            append_transferred_references: false,

            max_blank_lines: 2,
            max_trailing_blank_lines: 2,
            right_margin: 0,
            thematic_break: None,
            thematic_break_marker: '*',
            format_flags: FormatFlags::DEFAULT,
        }
    }
}

impl FormatOptions {
    /// Create a new FormatOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set right margin
    pub fn with_right_margin(mut self, value: usize) -> Self {
        self.right_margin = value;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = FormatOptions::default();
        assert!(matches!(opts.heading_style, HeadingStyle::AsIs));
        assert!(matches!(opts.list_bullet_marker, BulletMarker::Dash));
        assert!(matches!(opts.list_numbered_marker, NumberedMarker::Period));
        assert!(matches!(opts.list_spacing, ListSpacing::AsIs));
        assert!(matches!(
            opts.fenced_code_marker_type,
            CodeFenceMarker::BackTick
        ));
        assert!(matches!(opts.block_quote_markers, BlockQuoteMarker::AsIs));
        assert!(matches!(opts.reference_placement, ElementPlacement::AsIs));
        assert!(matches!(opts.reference_sort, ElementPlacementSort::AsIs));
        assert_eq!(opts.max_blank_lines, 2);
        assert_eq!(opts.max_trailing_blank_lines, 2);
        assert_eq!(opts.right_margin, 0);
        assert_eq!(opts.fenced_code_marker_length, 3);
        assert_eq!(opts.min_setext_marker_length, 3);
        assert!(opts.keep_hard_line_breaks);
        assert!(opts.keep_soft_line_breaks);
        assert!(opts.list_renumber_items);
        assert!(opts.setext_heading_equalize_marker);
        assert!(opts.fenced_code_match_closing_marker);
    }

    #[test]
    fn test_builder_pattern() {
        let opts = FormatOptions::new().with_right_margin(80);

        assert_eq!(opts.right_margin, 80);
    }

    #[test]
    fn test_format_flags_default() {
        let flags = FormatFlags::DEFAULT;
        assert!(flags.trim_leading_whitespace);
        assert!(flags.trim_trailing_whitespace);
        assert!(!flags.convert_tabs);
        assert!(!flags.collapse_whitespace);
    }

    #[test]
    fn test_options_clone() {
        let opts = FormatOptions::new().with_right_margin(100);

        let cloned = opts.clone();
        assert_eq!(cloned.right_margin, 100);
    }
}
