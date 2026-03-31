//! Markdown extensions management using bitflags.
//!
//! This module provides a unified way to manage Markdown extensions,
//! inspired by Pandoc's extension system. Extensions can be combined
//! using bitwise operators.
//!
//! # Example
//!
//! ```
//! use clmd::extensions::Extensions;
//!
//! // Combine extensions
//! let ext = Extensions::TABLE | Extensions::STRIKETHROUGH;
//!
//! // Check if an extension is enabled
//! assert!(ext.contains(Extensions::TABLE));
//! assert!(!ext.contains(Extensions::FOOTNOTES));
//! ```

use bitflags::bitflags;
use std::fmt;
use std::str::FromStr;

use crate::error::{ClmdError, ClmdResult};

bitflags! {
    /// Markdown extension flags.
    ///
    /// These flags represent various Markdown extensions that can be
    /// enabled or disabled when parsing or rendering.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Extensions: u64 {
        /// Tables extension (GFM)
        const TABLE = 1 << 0;

        /// Strikethrough extension (GFM)
        const STRIKETHROUGH = 1 << 1;

        /// Task list extension (GFM)
        const TASKLIST = 1 << 2;

        /// Autolink extension (GFM)
        const AUTOLINK = 1 << 3;

        /// Footnotes extension
        const FOOTNOTES = 1 << 4;

        /// Definition list extension
        const DEFINITION_LIST = 1 << 5;

        /// Tag filter extension (GFM)
        const TAG_FILTER = 1 << 6;

        /// Superscript extension
        const SUPERSCRIPT = 1 << 7;

        /// Subscript extension
        const SUBSCRIPT = 1 << 8;

        /// Underline extension
        const UNDERLINE = 1 << 9;

        /// Highlight extension
        const HIGHLIGHT = 1 << 10;

        /// Math extension
        const MATH = 1 << 11;

        /// Wikilinks extension
        const WIKILINKS = 1 << 12;

        /// Alerts extension (GFM)
        const ALERTS = 1 << 13;

        /// YAML front matter extension
        const YAML_FRONT_MATTER = 1 << 14;

        /// Abbreviation extension
        const ABBREVIATION = 1 << 15;

        /// Attributes extension
        const ATTRIBUTES = 1 << 16;

        /// Table of contents extension
        const TOC = 1 << 17;

        /// Emoji extension
        const EMOJI = 1 << 18;

        /// Shortcodes extension
        const SHORTCODES = 1 << 19;

        /// Smart punctuation
        const SMART = 1 << 20;

        /// Hard breaks
        const HARDBREAKS = 1 << 21;

        /// GitHub pre-lang
        const GITHUB_PRE_LANG = 1 << 22;

        /// Full info string
        const FULL_INFO_STRING = 1 << 23;

        /// Width option
        const WIDTH = 1 << 24;

        /// Unsafe HTML
        const UNSAFE = 1 << 25;

        /// Escape raw HTML
        const ESCAPE = 1 << 26;

        /// List style
        const LIST_STYLE = 1 << 27;

        /// Source position
        const SOURCE_POS = 1 << 28;

        /// Ignore setext headers
        const IGNORE_SETEXT = 1 << 29;

        /// Ignore empty links
        const IGNORE_EMPTY_LINKS = 1 << 30;

        /// GFM quirks
        const GFM_QUIRKS = 1 << 31;
    }
}

impl Extensions {
    /// Create an empty extension set.
    pub fn new() -> Self {
        Self::empty()
    }

    /// Create with all GFM extensions.
    pub fn gfm() -> Self {
        Self::TABLE
            | Self::STRIKETHROUGH
            | Self::TASKLIST
            | Self::AUTOLINK
            | Self::TAG_FILTER
            | Self::ALERTS
    }

    /// Create with common extensions.
    pub fn common() -> Self {
        Self::gfm() | Self::FOOTNOTES | Self::YAML_FRONT_MATTER | Self::TOC
    }

    /// Parse from a string representation.
    ///
    /// Supports `+extension` and `-extension` syntax.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::extensions::Extensions;
    ///
    /// let ext = Extensions::parse("+table-strikethrough").unwrap();
    /// assert!(ext.contains(Extensions::TABLE));
    /// assert!(!ext.contains(Extensions::STRIKETHROUGH));
    /// ```
    pub fn parse(s: &str) -> ClmdResult<Self> {
        let mut result = Self::empty();

        // Split by comma or parse +/- syntax
        let parts: Vec<String> = if s.contains(',') {
            s.split(',').map(|p| p.trim().to_string()).collect()
        } else {
            // Split by +/- boundaries
            let mut parts = Vec::new();
            let mut current = String::new();

            for ch in s.chars() {
                if ch == '+' || ch == '-' {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                current.push(ch);
            }
            if !current.is_empty() {
                parts.push(current);
            }
            parts.into_iter().map(|s| s.trim().to_string()).collect()
        };

        for part in parts.iter().map(|s| s.as_str()) {
            if part.is_empty() {
                continue;
            }

            let (enable, name) = if part.starts_with('+') {
                (true, &part[1..])
            } else if part.starts_with('-') {
                (false, &part[1..])
            } else {
                (true, part)
            };

            let name = name.trim();
            if name.is_empty() {
                continue;
            }

            let flag = Self::from_extension_name(name).ok_or_else(|| {
                ClmdError::config_error(format!("Unknown extension: {}", name))
            })?;

            if enable {
                result |= flag;
            } else {
                result &= !flag;
            }
        }

        Ok(result)
    }

    /// Get extension flag from name.
    pub fn from_extension_name(name: &str) -> Option<Self> {
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            "table" | "tables" => Some(Self::TABLE),
            "strikethrough" | "strike" => Some(Self::STRIKETHROUGH),
            "tasklist" | "tasklists" => Some(Self::TASKLIST),
            "autolink" | "autolinks" => Some(Self::AUTOLINK),
            "footnotes" | "footnote" => Some(Self::FOOTNOTES),
            "definition_list" | "definition_lists" | "deflist" => {
                Some(Self::DEFINITION_LIST)
            }
            "tagfilter" | "tag_filter" => Some(Self::TAG_FILTER),
            "superscript" => Some(Self::SUPERSCRIPT),
            "subscript" => Some(Self::SUBSCRIPT),
            "underline" => Some(Self::UNDERLINE),
            "highlight" => Some(Self::HIGHLIGHT),
            "math" => Some(Self::MATH),
            "wikilinks" | "wikilink" => Some(Self::WIKILINKS),
            "alerts" | "alert" => Some(Self::ALERTS),
            "yaml_front_matter" | "front_matter" | "yaml" => {
                Some(Self::YAML_FRONT_MATTER)
            }
            "abbreviation" | "abbr" => Some(Self::ABBREVIATION),
            "attributes" | "attr" => Some(Self::ATTRIBUTES),
            "toc" | "table_of_contents" => Some(Self::TOC),
            "emoji" | "emojis" => Some(Self::EMOJI),
            "shortcodes" | "shortcode" => Some(Self::SHORTCODES),
            "smart" => Some(Self::SMART),
            "hardbreaks" | "hard_breaks" => Some(Self::HARDBREAKS),
            "github_pre_lang" => Some(Self::GITHUB_PRE_LANG),
            "full_info_string" => Some(Self::FULL_INFO_STRING),
            "width" => Some(Self::WIDTH),
            "unsafe" | "unsafe_" => Some(Self::UNSAFE),
            "escape" => Some(Self::ESCAPE),
            "list_style" => Some(Self::LIST_STYLE),
            "source_pos" => Some(Self::SOURCE_POS),
            "ignore_setext" => Some(Self::IGNORE_SETEXT),
            "ignore_empty_links" => Some(Self::IGNORE_EMPTY_LINKS),
            "gfm_quirks" => Some(Self::GFM_QUIRKS),
            _ => None,
        }
    }

    /// Get extension name from flag.
    pub fn to_name(&self) -> Option<&'static str> {
        match *self {
            Self::TABLE => Some("table"),
            Self::STRIKETHROUGH => Some("strikethrough"),
            Self::TASKLIST => Some("tasklist"),
            Self::AUTOLINK => Some("autolink"),
            Self::FOOTNOTES => Some("footnotes"),
            Self::DEFINITION_LIST => Some("definition_list"),
            Self::TAG_FILTER => Some("tagfilter"),
            Self::SUPERSCRIPT => Some("superscript"),
            Self::SUBSCRIPT => Some("subscript"),
            Self::UNDERLINE => Some("underline"),
            Self::HIGHLIGHT => Some("highlight"),
            Self::MATH => Some("math"),
            Self::WIKILINKS => Some("wikilinks"),
            Self::ALERTS => Some("alerts"),
            Self::YAML_FRONT_MATTER => Some("yaml_front_matter"),
            Self::ABBREVIATION => Some("abbreviation"),
            Self::ATTRIBUTES => Some("attributes"),
            Self::TOC => Some("toc"),
            Self::EMOJI => Some("emoji"),
            Self::SHORTCODES => Some("shortcodes"),
            Self::SMART => Some("smart"),
            Self::HARDBREAKS => Some("hardbreaks"),
            Self::GITHUB_PRE_LANG => Some("github_pre_lang"),
            Self::FULL_INFO_STRING => Some("full_info_string"),
            Self::WIDTH => Some("width"),
            Self::UNSAFE => Some("unsafe"),
            Self::ESCAPE => Some("escape"),
            Self::LIST_STYLE => Some("list_style"),
            Self::SOURCE_POS => Some("source_pos"),
            Self::IGNORE_SETEXT => Some("ignore_setext"),
            Self::IGNORE_EMPTY_LINKS => Some("ignore_empty_links"),
            Self::GFM_QUIRKS => Some("gfm_quirks"),
            _ => None,
        }
    }

    /// Get all enabled extension names.
    pub fn enabled_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        for flag in Self::all_flags() {
            if self.contains(flag) {
                if let Some(name) = flag.to_name() {
                    names.push(name);
                }
            }
        }
        names
    }

    /// Get all individual flags.
    fn all_flags() -> Vec<Self> {
        vec![
            Self::TABLE,
            Self::STRIKETHROUGH,
            Self::TASKLIST,
            Self::AUTOLINK,
            Self::FOOTNOTES,
            Self::DEFINITION_LIST,
            Self::TAG_FILTER,
            Self::SUPERSCRIPT,
            Self::SUBSCRIPT,
            Self::UNDERLINE,
            Self::HIGHLIGHT,
            Self::MATH,
            Self::WIKILINKS,
            Self::ALERTS,
            Self::YAML_FRONT_MATTER,
            Self::ABBREVIATION,
            Self::ATTRIBUTES,
            Self::TOC,
            Self::EMOJI,
            Self::SHORTCODES,
            Self::SMART,
            Self::HARDBREAKS,
            Self::GITHUB_PRE_LANG,
            Self::FULL_INFO_STRING,
            Self::WIDTH,
            Self::UNSAFE,
            Self::ESCAPE,
            Self::LIST_STYLE,
            Self::SOURCE_POS,
            Self::IGNORE_SETEXT,
            Self::IGNORE_EMPTY_LINKS,
            Self::GFM_QUIRKS,
        ]
    }
}

impl fmt::Display for Extensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self.enabled_names();
        if names.is_empty() {
            write!(f, "none")
        } else {
            write!(f, "{}", names.join("+"))
        }
    }
}

impl FromStr for Extensions {
    type Err = ClmdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Extension difference for modifying extension sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtensionsDiff {
    /// Extensions to add.
    pub add: Extensions,
    /// Extensions to remove.
    pub remove: Extensions,
}

impl ExtensionsDiff {
    /// Create an empty diff.
    pub fn new() -> Self {
        Self {
            add: Extensions::empty(),
            remove: Extensions::empty(),
        }
    }

    /// Parse from a string.
    pub fn parse(s: &str) -> ClmdResult<Self> {
        let mut add = Extensions::empty();
        let mut remove = Extensions::empty();

        let parts: Vec<String> = if s.contains(',') {
            s.split(',').map(|p| p.trim().to_string()).collect()
        } else {
            let mut parts = Vec::new();
            let mut current = String::new();

            for ch in s.chars() {
                if ch == '+' || ch == '-' {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                current.push(ch);
            }
            if !current.is_empty() {
                parts.push(current);
            }
            parts.into_iter().map(|s| s.trim().to_string()).collect()
        };

        for part in parts.iter().map(|s| s.as_str()) {
            if part.is_empty() {
                continue;
            }

            let (is_add, name) = if part.starts_with('+') {
                (true, &part[1..])
            } else if part.starts_with('-') {
                (false, &part[1..])
            } else {
                (true, part)
            };

            let name = name.trim();
            if name.is_empty() {
                continue;
            }

            if let Some(flag) = Extensions::from_extension_name(name) {
                if is_add {
                    add |= flag;
                    remove &= !flag;
                } else {
                    remove |= flag;
                    add &= !flag;
                }
            }
        }

        Ok(Self { add, remove })
    }

    /// Apply this diff to an extension set.
    pub fn apply(&self, ext: Extensions) -> Extensions {
        (ext | self.add) & !self.remove
    }

    /// Check if this diff is empty.
    pub fn is_empty(&self) -> bool {
        self.add.is_empty() && self.remove.is_empty()
    }
}

impl Default for ExtensionsDiff {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ExtensionsDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        for flag in Extensions::all_flags() {
            if self.add.contains(flag) {
                if let Some(name) = flag.to_name() {
                    parts.push(format!("+{}", name));
                }
            }
            if self.remove.contains(flag) {
                if let Some(name) = flag.to_name() {
                    parts.push(format!("-{}", name));
                }
            }
        }

        write!(f, "{}", parts.join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extensions_empty() {
        let ext = Extensions::empty();
        assert!(!ext.contains(Extensions::TABLE));
    }

    #[test]
    fn test_extensions_gfm() {
        let ext = Extensions::gfm();
        assert!(ext.contains(Extensions::TABLE));
        assert!(ext.contains(Extensions::STRIKETHROUGH));
        assert!(ext.contains(Extensions::TASKLIST));
        assert!(ext.contains(Extensions::AUTOLINK));
    }

    #[test]
    fn test_extensions_common() {
        let ext = Extensions::common();
        assert!(ext.contains(Extensions::TABLE));
        assert!(ext.contains(Extensions::FOOTNOTES));
        assert!(ext.contains(Extensions::YAML_FRONT_MATTER));
    }

    #[test]
    fn test_extensions_from_name() {
        assert_eq!(
            Extensions::from_extension_name("table"),
            Some(Extensions::TABLE)
        );
        assert_eq!(
            Extensions::from_extension_name("TABLE"),
            Some(Extensions::TABLE)
        );
        assert_eq!(Extensions::from_extension_name("unknown"), None);
    }

    #[test]
    fn test_extensions_to_name() {
        assert_eq!(Extensions::TABLE.to_name(), Some("table"));
        assert_eq!(Extensions::empty().to_name(), None);
    }

    #[test]
    fn test_extensions_parse() {
        let ext = Extensions::parse("+table+strikethrough").unwrap();
        assert!(ext.contains(Extensions::TABLE));
        assert!(ext.contains(Extensions::STRIKETHROUGH));
    }

    #[test]
    fn test_extensions_parse_disable() {
        let base = Extensions::gfm();
        let diff = ExtensionsDiff::parse("-strikethrough").unwrap();
        let result = diff.apply(base);
        assert!(!result.contains(Extensions::STRIKETHROUGH));
        assert!(result.contains(Extensions::TABLE));
    }

    #[test]
    fn test_extensions_display() {
        let ext = Extensions::TABLE | Extensions::STRIKETHROUGH;
        let s = ext.to_string();
        assert!(s.contains("table"));
        assert!(s.contains("strikethrough"));
    }

    #[test]
    fn test_extensions_diff() {
        let diff = ExtensionsDiff::parse("+table-strikethrough").unwrap();
        let base = Extensions::gfm();
        let result = diff.apply(base);

        assert!(result.contains(Extensions::TABLE));
        assert!(!result.contains(Extensions::STRIKETHROUGH));
    }

    #[test]
    fn test_extensions_enabled_names() {
        let ext = Extensions::TABLE | Extensions::STRIKETHROUGH;
        let names = ext.enabled_names();
        assert!(names.contains(&"table"));
        assert!(names.contains(&"strikethrough"));
    }
}
