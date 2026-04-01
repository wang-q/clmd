//! Markdown extensions management using bitflags.
//!
//! This module provides a unified way to manage Markdown extensions,
//! inspired by Pandoc's extension system.
//!
//! # Example
//!
//! ```ignore
//! use clmd::ext::flags::ExtensionFlags;
//!
//! let mut extensions = ExtensionFlags::empty();
//! extensions |= ExtensionFlags::TABLES;
//! extensions |= ExtensionFlags::STRIKETHROUGH;
//!
//! assert!(extensions.contains(ExtensionFlags::TABLES));
//! ```

use bitflags::bitflags;

bitflags! {
    /// Set of enabled Markdown extensions.
    ///
    /// This type uses bitflags for efficient storage and manipulation
    /// of extension states.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ExtensionFlags: u64 {
        /// Tables (GFM)
        const TABLES = 1 << 0;
        /// Task lists (GFM)
        const TASKLISTS = 1 << 1;
        /// Strikethrough (GFM)
        const STRIKETHROUGH = 1 << 2;
        /// Autolinks (GFM)
        const AUTOLINKS = 1 << 3;
        /// Tag filter (GFM)
        const TAGFILTER = 1 << 4;
        /// Footnotes
        const FOOTNOTES = 1 << 5;
        /// Description lists
        const DESCRIPTION_LISTS = 1 << 6;
        /// Smart punctuation
        const SMART = 1 << 7;
        /// Math (dollar syntax)
        const MATH_DOLLARS = 1 << 8;
        /// Math (tex syntax)
        const MATH_TEX = 1 << 9;
        /// Superscript
        const SUPERSCRIPT = 1 << 10;
        /// Subscript
        const SUBSCRIPT = 1 << 11;
        /// Wiki links (title after pipe)
        const WIKILINKS_TITLE_AFTER_PIPE = 1 << 12;
        /// Wiki links (title before pipe)
        const WIKILINKS_TITLE_BEFORE_PIPE = 1 << 13;
        /// Shortcodes
        const SHORTCODES = 1 << 14;
        /// Attributes
        const ATTRIBUTES = 1 << 15;
        /// YAML front matter
        const YAML_FRONT_MATTER = 1 << 16;
        /// Abbreviations
        const ABBREVIATIONS = 1 << 17;
        /// Underline
        const UNDERLINE = 1 << 18;
        /// Highlight
        const HIGHLIGHT = 1 << 19;
        /// Insert
        const INSERT = 1 << 20;
        /// Spoiler
        const SPOILER = 1 << 21;
        /// Greentext
        const GREENTEXT = 1 << 22;
        /// Alerts
        const ALERTS = 1 << 23;
        /// Multiline block quotes
        const MULTILINE_BLOCK_QUOTES = 1 << 24;
        /// Table of contents
        const TOC = 1 << 25;
        /// Emoji
        const EMOJI = 1 << 26;
    }
}

impl Default for ExtensionFlags {
    fn default() -> Self {
        Self::empty()
    }
}

impl ExtensionFlags {
    /// Create an empty set of extensions.
    pub fn new() -> Self {
        Self::empty()
    }

    /// Enable all GFM extensions.
    pub fn gfm() -> Self {
        Self::TABLES
            | Self::TASKLISTS
            | Self::STRIKETHROUGH
            | Self::AUTOLINKS
            | Self::TAGFILTER
    }
}

/// Individual Markdown extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtensionKind {
    /// Tables (GFM)
    Table,
    /// Task lists (GFM)
    Tasklist,
    /// Strikethrough (GFM)
    Strikethrough,
    /// Autolinks (GFM)
    Autolink,
    /// Tag filter (GFM)
    Tagfilter,
    /// Footnotes
    Footnotes,
    /// Description lists
    DescriptionLists,
    /// Smart punctuation
    Smart,
    /// Math (dollar syntax)
    MathDollars,
    /// Math (tex syntax)
    MathTex,
    /// Superscript
    Superscript,
    /// Subscript
    Subscript,
    /// Wiki links
    WikiLinks,
    /// Shortcodes
    Shortcodes,
    /// Attributes
    Attributes,
    /// YAML front matter
    YamlFrontMatter,
    /// Abbreviations
    Abbreviations,
    /// Definition lists
    DefinitionLists,
}

impl ExtensionKind {
    /// Get all extensions.
    pub fn all() -> &'static [ExtensionKind] {
        &[
            ExtensionKind::Table,
            ExtensionKind::Tasklist,
            ExtensionKind::Strikethrough,
            ExtensionKind::Autolink,
            ExtensionKind::Tagfilter,
            ExtensionKind::Footnotes,
            ExtensionKind::DescriptionLists,
            ExtensionKind::Smart,
            ExtensionKind::MathDollars,
            ExtensionKind::MathTex,
            ExtensionKind::Superscript,
            ExtensionKind::Subscript,
            ExtensionKind::WikiLinks,
            ExtensionKind::Shortcodes,
            ExtensionKind::Attributes,
            ExtensionKind::YamlFrontMatter,
            ExtensionKind::Abbreviations,
            ExtensionKind::DefinitionLists,
        ]
    }

    /// Get the extension name.
    pub fn as_str(&self) -> &'static str {
        match self {
            ExtensionKind::Table => "table",
            ExtensionKind::Tasklist => "tasklist",
            ExtensionKind::Strikethrough => "strikethrough",
            ExtensionKind::Autolink => "autolink",
            ExtensionKind::Tagfilter => "tagfilter",
            ExtensionKind::Footnotes => "footnotes",
            ExtensionKind::DescriptionLists => "description_lists",
            ExtensionKind::Smart => "smart",
            ExtensionKind::MathDollars => "math_dollars",
            ExtensionKind::MathTex => "math_tex",
            ExtensionKind::Superscript => "superscript",
            ExtensionKind::Subscript => "subscript",
            ExtensionKind::WikiLinks => "wiki_links",
            ExtensionKind::Shortcodes => "shortcodes",
            ExtensionKind::Attributes => "attributes",
            ExtensionKind::YamlFrontMatter => "yaml_front_matter",
            ExtensionKind::Abbreviations => "abbreviations",
            ExtensionKind::DefinitionLists => "definition_lists",
        }
    }

    /// Parse an extension from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "table" | "tables" => Some(ExtensionKind::Table),
            "tasklist" | "tasklists" => Some(ExtensionKind::Tasklist),
            "strikethrough" => Some(ExtensionKind::Strikethrough),
            "autolink" | "autolinks" => Some(ExtensionKind::Autolink),
            "tagfilter" => Some(ExtensionKind::Tagfilter),
            "footnote" | "footnotes" => Some(ExtensionKind::Footnotes),
            "description_list" | "description_lists" => {
                Some(ExtensionKind::DescriptionLists)
            }
            "smart" => Some(ExtensionKind::Smart),
            "math_dollars" => Some(ExtensionKind::MathDollars),
            "math_tex" => Some(ExtensionKind::MathTex),
            "superscript" => Some(ExtensionKind::Superscript),
            "subscript" => Some(ExtensionKind::Subscript),
            "wiki_link" | "wiki_links" => Some(ExtensionKind::WikiLinks),
            "shortcode" | "shortcodes" => Some(ExtensionKind::Shortcodes),
            "attribute" | "attributes" => Some(ExtensionKind::Attributes),
            "yaml_front_matter" => Some(ExtensionKind::YamlFrontMatter),
            "abbreviation" | "abbreviations" => Some(ExtensionKind::Abbreviations),
            "definition_list" | "definition_lists" => Some(ExtensionKind::DefinitionLists),
            _ => None,
        }
    }
}

/// Deprecated alias for `ExtensionFlags`.
#[deprecated(since = "0.2.0", note = "Use `ExtensionFlags` instead")]
pub type Extensions = ExtensionFlags;

/// Deprecated alias for `ExtensionKind`.
#[deprecated(since = "0.2.0", note = "Use `ExtensionKind` instead")]
pub type Extension = ExtensionKind;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_flags_default() {
        let ext = ExtensionFlags::default();
        assert!(ext.is_empty());
    }

    #[test]
    fn test_extension_flags_gfm() {
        let ext = ExtensionFlags::gfm();
        assert!(ext.contains(ExtensionFlags::TABLES));
        assert!(ext.contains(ExtensionFlags::TASKLISTS));
        assert!(ext.contains(ExtensionFlags::STRIKETHROUGH));
        assert!(ext.contains(ExtensionFlags::AUTOLINKS));
        assert!(ext.contains(ExtensionFlags::TAGFILTER));
    }

    #[test]
    fn test_extension_kind_from_str() {
        assert_eq!(ExtensionKind::from_str("table"), Some(ExtensionKind::Table));
        assert_eq!(ExtensionKind::from_str("TABLE"), Some(ExtensionKind::Table));
        assert_eq!(ExtensionKind::from_str("unknown"), None);
    }

    #[test]
    fn test_extension_kind_as_str() {
        assert_eq!(ExtensionKind::Table.as_str(), "table");
        assert_eq!(ExtensionKind::Strikethrough.as_str(), "strikethrough");
    }

    #[test]
    fn test_extension_kind_all() {
        let all = ExtensionKind::all();
        assert!(!all.is_empty());
        assert!(all.contains(&ExtensionKind::Table));
    }
}
