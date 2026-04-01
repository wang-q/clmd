//! Markdown extensions management using bitflags.
//!
//! This module provides a unified way to manage Markdown extensions,
//! inspired by Pandoc's extension system.
//!
//! # Example
//!
//! ```ignore
//! use clmd::extensions::Extensions;
//!
//! let mut extensions = Extensions::empty();
//! extensions |= Extensions::TABLES;
//! extensions |= Extensions::STRIKETHROUGH;
//!
//! assert!(extensions.contains(Extensions::TABLES));
//! ```

use bitflags::bitflags;

bitflags! {
    /// Set of enabled Markdown extensions.
    ///
    /// This type uses bitflags for efficient storage and manipulation
    /// of extension states.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Extensions: u64 {
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

impl Default for Extensions {
    fn default() -> Self {
        Self::empty()
    }
}

impl Extensions {
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
pub enum Extension {
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

impl Extension {
    /// Get all extensions.
    pub fn all() -> &'static [Extension] {
        &[
            Extension::Table,
            Extension::Tasklist,
            Extension::Strikethrough,
            Extension::Autolink,
            Extension::Tagfilter,
            Extension::Footnotes,
            Extension::DescriptionLists,
            Extension::Smart,
            Extension::MathDollars,
            Extension::MathTex,
            Extension::Superscript,
            Extension::Subscript,
            Extension::WikiLinks,
            Extension::Shortcodes,
            Extension::Attributes,
            Extension::YamlFrontMatter,
            Extension::Abbreviations,
            Extension::DefinitionLists,
        ]
    }

    /// Get the extension name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Extension::Table => "table",
            Extension::Tasklist => "tasklist",
            Extension::Strikethrough => "strikethrough",
            Extension::Autolink => "autolink",
            Extension::Tagfilter => "tagfilter",
            Extension::Footnotes => "footnotes",
            Extension::DescriptionLists => "description_lists",
            Extension::Smart => "smart",
            Extension::MathDollars => "math_dollars",
            Extension::MathTex => "math_tex",
            Extension::Superscript => "superscript",
            Extension::Subscript => "subscript",
            Extension::WikiLinks => "wiki_links",
            Extension::Shortcodes => "shortcodes",
            Extension::Attributes => "attributes",
            Extension::YamlFrontMatter => "yaml_front_matter",
            Extension::Abbreviations => "abbreviations",
            Extension::DefinitionLists => "definition_lists",
        }
    }

    /// Parse an extension from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "table" | "tables" => Some(Extension::Table),
            "tasklist" | "tasklists" => Some(Extension::Tasklist),
            "strikethrough" => Some(Extension::Strikethrough),
            "autolink" | "autolinks" => Some(Extension::Autolink),
            "tagfilter" => Some(Extension::Tagfilter),
            "footnote" | "footnotes" => Some(Extension::Footnotes),
            "description_list" | "description_lists" => {
                Some(Extension::DescriptionLists)
            }
            "smart" => Some(Extension::Smart),
            "math_dollars" => Some(Extension::MathDollars),
            "math_tex" => Some(Extension::MathTex),
            "superscript" => Some(Extension::Superscript),
            "subscript" => Some(Extension::Subscript),
            "wiki_link" | "wiki_links" => Some(Extension::WikiLinks),
            "shortcode" | "shortcodes" => Some(Extension::Shortcodes),
            "attribute" | "attributes" => Some(Extension::Attributes),
            "yaml_front_matter" => Some(Extension::YamlFrontMatter),
            "abbreviation" | "abbreviations" => Some(Extension::Abbreviations),
            "definition_list" | "definition_lists" => Some(Extension::DefinitionLists),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extensions_default() {
        let ext = Extensions::default();
        assert!(ext.is_empty());
    }

    #[test]
    fn test_extensions_gfm() {
        let ext = Extensions::gfm();
        assert!(ext.contains(Extensions::TABLES));
        assert!(ext.contains(Extensions::TASKLISTS));
        assert!(ext.contains(Extensions::STRIKETHROUGH));
        assert!(ext.contains(Extensions::AUTOLINKS));
        assert!(ext.contains(Extensions::TAGFILTER));
    }

    #[test]
    fn test_extension_from_str() {
        assert_eq!(Extension::from_str("table"), Some(Extension::Table));
        assert_eq!(Extension::from_str("TABLE"), Some(Extension::Table));
        assert_eq!(Extension::from_str("unknown"), None);
    }

    #[test]
    fn test_extension_as_str() {
        assert_eq!(Extension::Table.as_str(), "table");
        assert_eq!(Extension::Strikethrough.as_str(), "strikethrough");
    }

    #[test]
    fn test_extension_all() {
        let all = Extension::all();
        assert!(!all.is_empty());
        assert!(all.contains(&Extension::Table));
    }
}
