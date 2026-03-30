//! Formatter options configuration
//!
//! This module provides comprehensive configuration options for the Markdown formatter,
//! inspired by flexmark-java's FormatterOptions.

/// Heading style options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

impl ElementPlacementSort {
    /// Check if this sort option includes sorting
    pub fn is_sort(&self) -> bool {
        matches!(
            self,
            Self::Sort | Self::SortUnusedLast | Self::SortDeleteUnused
        )
    }

    /// Check if this sort option includes deleting unused elements
    pub fn is_delete_unused(&self) -> bool {
        matches!(self, Self::SortDeleteUnused | Self::DeleteUnused)
    }

    /// Check if this sort option includes tracking unused elements
    pub fn is_unused(&self) -> bool {
        matches!(
            self,
            Self::SortUnusedLast | Self::SortDeleteUnused | Self::DeleteUnused
        )
    }
}

impl ElementPlacement {
    /// Check if this placement changes from the original
    pub fn is_change(&self) -> bool {
        !matches!(self, Self::AsIs)
    }

    /// Check if this placement is no-change
    pub fn is_no_change(&self) -> bool {
        matches!(self, Self::AsIs)
    }
}

/// Discretionary text options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone)]
pub struct FormatterOptions {
    // Heading options
    /// Heading style preference
    pub heading_style: HeadingStyle,
    /// Add space after ATX marker
    pub space_after_atx_marker: DiscretionaryText,
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
    /// Format flags
    pub format_flags: FormatFlags,

    // Format control options
    /// Enable formatter control comments
    pub formatter_tags_enabled: bool,
    /// Accept regex in formatter tags
    pub formatter_tags_accept_regexp: bool,
    /// Formatter on tag
    pub formatter_on_tag: String,
    /// Formatter off tag
    pub formatter_off_tag: String,

    // Translation options
    /// Translation ID format
    pub translation_id_format: String,
    /// Translation HTML block prefix
    pub translation_html_block_prefix: String,
    /// Translation HTML inline prefix
    pub translation_html_inline_prefix: String,
    /// Translation autolink prefix
    pub translation_autolink_prefix: String,
    /// Translation exclude pattern
    pub translation_exclude_pattern: String,
}

impl Default for FormatterOptions {
    fn default() -> Self {
        Self {
            // Heading defaults
            heading_style: HeadingStyle::default(),
            space_after_atx_marker: DiscretionaryText::Add,
            atx_heading_trailing_marker: TrailingMarker::AsIs,
            setext_heading_equalize_marker: true,
            min_setext_marker_length: 3,

            // List defaults
            list_bullet_marker: BulletMarker::default(),
            list_numbered_marker: NumberedMarker::default(),
            list_renumber_items: true,
            list_reset_first_item_number: false,
            list_remove_empty_items: false,
            list_spacing: ListSpacing::default(),
            list_align_numeric: Alignment::None,
            list_add_blank_line_before: false,
            lists_item_content_after_suffix: false,

            // Code block defaults
            fenced_code_marker_type: CodeFenceMarker::default(),
            fenced_code_marker_length: 3,
            fenced_code_match_closing_marker: true,
            fenced_code_space_before_info: false,
            indented_code_minimize_indent: true,
            fenced_code_minimize_indent: true,

            // Block quote defaults
            block_quote_blank_lines: false,
            block_quote_markers: BlockQuoteMarker::default(),

            // Line break defaults
            keep_hard_line_breaks: true,
            keep_soft_line_breaks: true,

            // Link defaults
            keep_image_links_at_start: false,
            keep_explicit_links_at_start: false,

            // Reference defaults
            reference_placement: ElementPlacement::default(),
            reference_sort: ElementPlacementSort::default(),
            append_transferred_references: false,

            // General defaults
            max_blank_lines: 2,
            max_trailing_blank_lines: 2,
            right_margin: 0,
            thematic_break: None,
            format_flags: FormatFlags::DEFAULT,

            // Format control defaults
            formatter_tags_enabled: false,
            formatter_tags_accept_regexp: false,
            formatter_on_tag: "@formatter:on".to_string(),
            formatter_off_tag: "@formatter:off".to_string(),

            // Translation defaults
            translation_id_format: "_%d_".to_string(),
            translation_html_block_prefix: "__".to_string(),
            translation_html_inline_prefix: "_".to_string(),
            translation_autolink_prefix: "___".to_string(),
            translation_exclude_pattern: "^[^\\p{IsAlphabetic}]*$".to_string(),
        }
    }
}

impl FormatterOptions {
    /// Create a new FormatterOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set heading style
    pub fn with_heading_style(mut self, style: HeadingStyle) -> Self {
        self.heading_style = style;
        self
    }

    /// Set space after ATX marker
    pub fn with_space_after_atx_marker(mut self, value: DiscretionaryText) -> Self {
        self.space_after_atx_marker = value;
        self
    }

    /// Set ATX heading trailing marker
    pub fn with_atx_heading_trailing_marker(mut self, value: TrailingMarker) -> Self {
        self.atx_heading_trailing_marker = value;
        self
    }

    /// Set Setext heading equalize marker
    pub fn with_setext_heading_equalize_marker(mut self, value: bool) -> Self {
        self.setext_heading_equalize_marker = value;
        self
    }

    /// Set minimum Setext marker length
    pub fn with_min_setext_marker_length(mut self, value: usize) -> Self {
        self.min_setext_marker_length = value;
        self
    }

    /// Set bullet marker
    pub fn with_list_bullet_marker(mut self, value: BulletMarker) -> Self {
        self.list_bullet_marker = value;
        self
    }

    /// Set numbered marker
    pub fn with_list_numbered_marker(mut self, value: NumberedMarker) -> Self {
        self.list_numbered_marker = value;
        self
    }

    /// Set list renumber items
    pub fn with_list_renumber_items(mut self, value: bool) -> Self {
        self.list_renumber_items = value;
        self
    }

    /// Set list spacing
    pub fn with_list_spacing(mut self, value: ListSpacing) -> Self {
        self.list_spacing = value;
        self
    }

    /// Set fenced code marker type
    pub fn with_fenced_code_marker_type(mut self, value: CodeFenceMarker) -> Self {
        self.fenced_code_marker_type = value;
        self
    }

    /// Set fenced code marker length
    pub fn with_fenced_code_marker_length(mut self, value: usize) -> Self {
        self.fenced_code_marker_length = value;
        self
    }

    /// Set block quote blank lines
    pub fn with_block_quote_blank_lines(mut self, value: bool) -> Self {
        self.block_quote_blank_lines = value;
        self
    }

    /// Set block quote markers
    pub fn with_block_quote_markers(mut self, value: BlockQuoteMarker) -> Self {
        self.block_quote_markers = value;
        self
    }

    /// Set keep hard line breaks
    pub fn with_keep_hard_line_breaks(mut self, value: bool) -> Self {
        self.keep_hard_line_breaks = value;
        self
    }

    /// Set keep soft line breaks
    pub fn with_keep_soft_line_breaks(mut self, value: bool) -> Self {
        self.keep_soft_line_breaks = value;
        self
    }

    /// Set right margin
    pub fn with_right_margin(mut self, value: usize) -> Self {
        self.right_margin = value;
        self
    }

    /// Set max blank lines
    pub fn with_max_blank_lines(mut self, value: usize) -> Self {
        self.max_blank_lines = value;
        self
    }

    /// Set thematic break
    pub fn with_thematic_break(mut self, value: impl Into<String>) -> Self {
        self.thematic_break = Some(value.into());
        self
    }

    /// Set reference placement
    pub fn with_reference_placement(mut self, value: ElementPlacement) -> Self {
        self.reference_placement = value;
        self
    }

    /// Set reference sort
    pub fn with_reference_sort(mut self, value: ElementPlacementSort) -> Self {
        self.reference_sort = value;
        self
    }

    /// Set formatter tags enabled
    pub fn with_formatter_tags_enabled(mut self, value: bool) -> Self {
        self.formatter_tags_enabled = value;
        self
    }

    /// Set formatter on tag
    pub fn with_formatter_on_tag(mut self, value: impl Into<String>) -> Self {
        self.formatter_on_tag = value.into();
        self
    }

    /// Set formatter off tag
    pub fn with_formatter_off_tag(mut self, value: impl Into<String>) -> Self {
        self.formatter_off_tag = value.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = FormatterOptions::default();
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
        assert!(!opts.formatter_tags_enabled);
    }

    #[test]
    fn test_builder_pattern() {
        let opts = FormatterOptions::new()
            .with_heading_style(HeadingStyle::Atx)
            .with_list_spacing(ListSpacing::Tight)
            .with_right_margin(80);

        assert!(matches!(opts.heading_style, HeadingStyle::Atx));
        assert!(matches!(opts.list_spacing, ListSpacing::Tight));
        assert_eq!(opts.right_margin, 80);
    }

    #[test]
    fn test_all_builder_methods() {
        let opts = FormatterOptions::new()
            .with_heading_style(HeadingStyle::Setext)
            .with_space_after_atx_marker(DiscretionaryText::Remove)
            .with_atx_heading_trailing_marker(TrailingMarker::Remove)
            .with_setext_heading_equalize_marker(false)
            .with_min_setext_marker_length(5)
            .with_list_bullet_marker(BulletMarker::Asterisk)
            .with_list_numbered_marker(NumberedMarker::Paren)
            .with_list_renumber_items(false)
            .with_list_spacing(ListSpacing::Loose)
            .with_fenced_code_marker_type(CodeFenceMarker::Tilde)
            .with_fenced_code_marker_length(4)
            .with_block_quote_blank_lines(true)
            .with_block_quote_markers(BlockQuoteMarker::AddCompact)
            .with_keep_hard_line_breaks(false)
            .with_keep_soft_line_breaks(false)
            .with_right_margin(120)
            .with_max_blank_lines(3)
            .with_thematic_break("---")
            .with_reference_placement(ElementPlacement::DocumentBottom)
            .with_reference_sort(ElementPlacementSort::Sort)
            .with_formatter_tags_enabled(true)
            .with_formatter_on_tag("on")
            .with_formatter_off_tag("off");

        assert!(matches!(opts.heading_style, HeadingStyle::Setext));
        assert!(matches!(
            opts.space_after_atx_marker,
            DiscretionaryText::Remove
        ));
        assert!(matches!(
            opts.atx_heading_trailing_marker,
            TrailingMarker::Remove
        ));
        assert!(!opts.setext_heading_equalize_marker);
        assert_eq!(opts.min_setext_marker_length, 5);
        assert!(matches!(opts.list_bullet_marker, BulletMarker::Asterisk));
        assert!(matches!(opts.list_numbered_marker, NumberedMarker::Paren));
        assert!(!opts.list_renumber_items);
        assert!(matches!(opts.list_spacing, ListSpacing::Loose));
        assert!(matches!(
            opts.fenced_code_marker_type,
            CodeFenceMarker::Tilde
        ));
        assert_eq!(opts.fenced_code_marker_length, 4);
        assert!(opts.block_quote_blank_lines);
        assert!(matches!(
            opts.block_quote_markers,
            BlockQuoteMarker::AddCompact
        ));
        assert!(!opts.keep_hard_line_breaks);
        assert!(!opts.keep_soft_line_breaks);
        assert_eq!(opts.right_margin, 120);
        assert_eq!(opts.max_blank_lines, 3);
        assert_eq!(opts.thematic_break, Some("---".to_string()));
        assert!(matches!(
            opts.reference_placement,
            ElementPlacement::DocumentBottom
        ));
        assert!(matches!(opts.reference_sort, ElementPlacementSort::Sort));
        assert!(opts.formatter_tags_enabled);
        assert_eq!(opts.formatter_on_tag, "on");
        assert_eq!(opts.formatter_off_tag, "off");
    }

    #[test]
    fn test_element_placement_sort() {
        assert!(ElementPlacementSort::Sort.is_sort());
        assert!(ElementPlacementSort::SortUnusedLast.is_sort());
        assert!(ElementPlacementSort::SortDeleteUnused.is_sort());
        assert!(!ElementPlacementSort::AsIs.is_sort());
        assert!(!ElementPlacementSort::DeleteUnused.is_sort());

        assert!(ElementPlacementSort::SortDeleteUnused.is_delete_unused());
        assert!(ElementPlacementSort::DeleteUnused.is_delete_unused());
        assert!(!ElementPlacementSort::Sort.is_delete_unused());
        assert!(!ElementPlacementSort::AsIs.is_delete_unused());

        assert!(ElementPlacementSort::SortUnusedLast.is_unused());
        assert!(ElementPlacementSort::SortDeleteUnused.is_unused());
        assert!(ElementPlacementSort::DeleteUnused.is_unused());
        assert!(!ElementPlacementSort::Sort.is_unused());
        assert!(!ElementPlacementSort::AsIs.is_unused());
    }

    #[test]
    fn test_element_placement() {
        assert!(ElementPlacement::DocumentTop.is_change());
        assert!(ElementPlacement::DocumentBottom.is_change());
        assert!(ElementPlacement::GroupWithFirst.is_change());
        assert!(ElementPlacement::GroupWithLast.is_change());
        assert!(!ElementPlacement::AsIs.is_change());

        assert!(ElementPlacement::AsIs.is_no_change());
        assert!(!ElementPlacement::DocumentTop.is_no_change());
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
    fn test_format_flags_custom() {
        let flags = FormatFlags {
            trim_leading_whitespace: false,
            trim_trailing_whitespace: false,
            convert_tabs: true,
            collapse_whitespace: true,
        };
        assert!(!flags.trim_leading_whitespace);
        assert!(!flags.trim_trailing_whitespace);
        assert!(flags.convert_tabs);
        assert!(flags.collapse_whitespace);
    }

    #[test]
    fn test_heading_style_variants() {
        assert!(matches!(HeadingStyle::Atx, HeadingStyle::Atx));
        assert!(matches!(HeadingStyle::Setext, HeadingStyle::Setext));
        assert!(matches!(HeadingStyle::AsIs, HeadingStyle::AsIs));
    }

    #[test]
    fn test_bullet_marker_variants() {
        assert!(matches!(BulletMarker::Dash, BulletMarker::Dash));
        assert!(matches!(BulletMarker::Asterisk, BulletMarker::Asterisk));
        assert!(matches!(BulletMarker::Plus, BulletMarker::Plus));
        assert!(matches!(BulletMarker::Any, BulletMarker::Any));
    }

    #[test]
    fn test_numbered_marker_variants() {
        assert!(matches!(NumberedMarker::Period, NumberedMarker::Period));
        assert!(matches!(NumberedMarker::Paren, NumberedMarker::Paren));
        assert!(matches!(NumberedMarker::Any, NumberedMarker::Any));
    }

    #[test]
    fn test_list_spacing_variants() {
        assert!(matches!(ListSpacing::Tight, ListSpacing::Tight));
        assert!(matches!(ListSpacing::Loose, ListSpacing::Loose));
        assert!(matches!(ListSpacing::AsIs, ListSpacing::AsIs));
        assert!(matches!(ListSpacing::Loosen, ListSpacing::Loosen));
        assert!(matches!(ListSpacing::Tighten, ListSpacing::Tighten));
    }

    #[test]
    fn test_code_fence_marker_variants() {
        assert!(matches!(
            CodeFenceMarker::BackTick,
            CodeFenceMarker::BackTick
        ));
        assert!(matches!(CodeFenceMarker::Tilde, CodeFenceMarker::Tilde));
        assert!(matches!(CodeFenceMarker::Any, CodeFenceMarker::Any));
    }

    #[test]
    fn test_block_quote_marker_variants() {
        assert!(matches!(BlockQuoteMarker::AsIs, BlockQuoteMarker::AsIs));
        assert!(matches!(
            BlockQuoteMarker::AddCompact,
            BlockQuoteMarker::AddCompact
        ));
        assert!(matches!(
            BlockQuoteMarker::AddCompactWithSpace,
            BlockQuoteMarker::AddCompactWithSpace
        ));
        assert!(matches!(
            BlockQuoteMarker::AddSpaced,
            BlockQuoteMarker::AddSpaced
        ));
    }

    #[test]
    fn test_discretionary_text_variants() {
        assert!(matches!(DiscretionaryText::Add, DiscretionaryText::Add));
        assert!(matches!(
            DiscretionaryText::Remove,
            DiscretionaryText::Remove
        ));
        assert!(matches!(DiscretionaryText::AsIs, DiscretionaryText::AsIs));
        assert!(matches!(
            DiscretionaryText::Equalize,
            DiscretionaryText::Equalize
        ));
    }

    #[test]
    fn test_trailing_marker_variants() {
        assert!(matches!(TrailingMarker::Add, TrailingMarker::Add));
        assert!(matches!(TrailingMarker::Remove, TrailingMarker::Remove));
        assert!(matches!(TrailingMarker::AsIs, TrailingMarker::AsIs));
        assert!(matches!(TrailingMarker::Equalize, TrailingMarker::Equalize));
    }

    #[test]
    fn test_alignment_variants() {
        assert!(matches!(Alignment::None, Alignment::None));
        assert!(matches!(Alignment::Left, Alignment::Left));
        assert!(matches!(Alignment::Right, Alignment::Right));
        assert!(matches!(Alignment::Center, Alignment::Center));
    }

    #[test]
    fn test_element_placement_variants() {
        assert!(matches!(ElementPlacement::AsIs, ElementPlacement::AsIs));
        assert!(matches!(
            ElementPlacement::DocumentTop,
            ElementPlacement::DocumentTop
        ));
        assert!(matches!(
            ElementPlacement::DocumentBottom,
            ElementPlacement::DocumentBottom
        ));
        assert!(matches!(
            ElementPlacement::GroupWithFirst,
            ElementPlacement::GroupWithFirst
        ));
        assert!(matches!(
            ElementPlacement::GroupWithLast,
            ElementPlacement::GroupWithLast
        ));
    }

    #[test]
    fn test_options_clone() {
        let opts = FormatterOptions::new()
            .with_right_margin(100)
            .with_heading_style(HeadingStyle::Atx);

        let cloned = opts.clone();
        assert_eq!(cloned.right_margin, 100);
        assert!(matches!(cloned.heading_style, HeadingStyle::Atx));
    }

    #[test]
    fn test_options_debug() {
        let opts = FormatterOptions::new();
        let debug_str = format!("{:?}", opts);
        assert!(debug_str.contains("FormatterOptions"));
        assert!(debug_str.contains("heading_style"));
        assert!(debug_str.contains("right_margin"));
    }
}
