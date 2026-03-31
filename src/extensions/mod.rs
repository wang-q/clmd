//! Markdown extensions management using bitflags.
//!
//! This module provides a unified way to manage Markdown extensions,
//! inspired by Pandoc's extension system. Extensions can be enabled or
//! disabled individually or in groups.
//!
//! # Example
//!
//! ```
//! use clmd::extensions::{Extensions, ExtensionConfig};
//!
//! // Enable multiple extensions
//! let ext = Extensions::TABLE | Extensions::STRIKETHROUGH | Extensions::TASKLIST;
//!
//! // Check if an extension is enabled
//! assert!(ext.contains(Extensions::TABLE));
//!
//! // Parse extension spec string
//! let config = ExtensionConfig::parse("markdown+table-strikethrough").unwrap();
//! ```

use bitflags::bitflags;
use std::fmt;
use std::str::FromStr;

bitflags! {
    /// Markdown extensions using bitflags for efficient storage and operations.
    ///
    /// Each extension is represented by a single bit, allowing efficient
    /// combination and checking of multiple extensions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct Extensions: u64 {
        /// Tables (GFM)
        const TABLE = 1 << 0;

        /// Strikethrough text (GFM)
        const STRIKETHROUGH = 1 << 1;

        /// Task lists (GFM)
        const TASKLIST = 1 << 2;

        /// Autolinks (GFM)
        const AUTOLINK = 1 << 3;

        /// Footnotes
        const FOOTNOTES = 1 << 4;

        /// Definition lists
        const DEFINITION_LIST = 1 << 5;

        /// Smart punctuation
        const SMART_PUNCTUATION = 1 << 6;

        /// YAML front matter
        const YAML_FRONT_MATTER = 1 << 7;

        /// Abbreviations
        const ABBREVIATION = 1 << 8;

        /// Attributes (header attributes, etc.)
        const ATTRIBUTES = 1 << 9;

        /// Table of contents
        const TOC = 1 << 10;

        /// Shortcodes
        const SHORTCODES = 1 << 11;

        /// Tag filtering
        const TAG_FILTER = 1 << 12;

        /// Math (LaTeX math support)
        const MATH = 1 << 13;

        /// Superscript
        const SUPERSCRIPT = 1 << 14;

        /// Subscript
        const SUBSCRIPT = 1 << 15;

        /// Underline
        const UNDERLINE = 1 << 16;

        /// Highlight/mark
        const HIGHLIGHT = 1 << 17;

        /// Emoji
        const EMOJI = 1 << 18;

        /// Wikilinks
        const WIKILINKS = 1 << 19;

        /// Alerts (GFM-style alerts)
        const ALERTS = 1 << 20;

        /// Mermaid diagrams
        const MERMAID = 1 << 21;

        /// Embedded content (iframe, etc.)
        const EMBED = 1 << 22;

        /// Hard line breaks
        const HARDBREAKS = 1 << 23;

        /// GitHub Flavored Markdown (all GFM extensions)
        const GFM = Self::TABLE.bits()
            | Self::STRIKETHROUGH.bits()
            | Self::TASKLIST.bits()
            | Self::AUTOLINK.bits()
            | Self::ALERTS.bits();

        /// CommonMark with all extensions
        const ALL = Self::GFM.bits()
            | Self::FOOTNOTES.bits()
            | Self::DEFINITION_LIST.bits()
            | Self::SMART_PUNCTUATION.bits()
            | Self::YAML_FRONT_MATTER.bits()
            | Self::ABBREVIATION.bits()
            | Self::ATTRIBUTES.bits()
            | Self::TOC.bits()
            | Self::SHORTCODES.bits()
            | Self::TAG_FILTER.bits()
            | Self::MATH.bits()
            | Self::SUPERSCRIPT.bits()
            | Self::SUBSCRIPT.bits()
            | Self::UNDERLINE.bits()
            | Self::HIGHLIGHT.bits()
            | Self::EMOJI.bits()
            | Self::WIKILINKS.bits()
            | Self::MERMAID.bits()
            | Self::EMBED.bits();
    }
}

impl Extensions {
    /// Get the name of an extension.
    pub fn name(&self) -> Option<&'static str> {
        match *self {
            Self::TABLE => Some("table"),
            Self::STRIKETHROUGH => Some("strikethrough"),
            Self::TASKLIST => Some("tasklist"),
            Self::AUTOLINK => Some("autolink"),
            Self::FOOTNOTES => Some("footnotes"),
            Self::DEFINITION_LIST => Some("definition_list"),
            Self::SMART_PUNCTUATION => Some("smart_punctuation"),
            Self::YAML_FRONT_MATTER => Some("yaml_front_matter"),
            Self::ABBREVIATION => Some("abbreviation"),
            Self::ATTRIBUTES => Some("attributes"),
            Self::TOC => Some("toc"),
            Self::SHORTCODES => Some("shortcodes"),
            Self::TAG_FILTER => Some("tag_filter"),
            Self::MATH => Some("math"),
            Self::SUPERSCRIPT => Some("superscript"),
            Self::SUBSCRIPT => Some("subscript"),
            Self::UNDERLINE => Some("underline"),
            Self::HIGHLIGHT => Some("highlight"),
            Self::EMOJI => Some("emoji"),
            Self::WIKILINKS => Some("wikilinks"),
            Self::ALERTS => Some("alerts"),
            Self::MERMAID => Some("mermaid"),
            Self::EMBED => Some("embed"),
            Self::HARDBREAKS => Some("hardbreaks"),
            _ => None,
        }
    }

    /// Parse an extension name into an Extensions value.
    pub fn parse_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "table" | "tables" => Some(Self::TABLE),
            "strikethrough" | "strike" => Some(Self::STRIKETHROUGH),
            "tasklist" | "tasklists" => Some(Self::TASKLIST),
            "autolink" | "autolinks" => Some(Self::AUTOLINK),
            "footnotes" | "footnote" => Some(Self::FOOTNOTES),
            "definition_list" | "definition_lists" | "deflist" => {
                Some(Self::DEFINITION_LIST)
            }
            "smart_punctuation" | "smart" => Some(Self::SMART_PUNCTUATION),
            "yaml_front_matter" | "yaml" | "front_matter" => {
                Some(Self::YAML_FRONT_MATTER)
            }
            "abbreviation" | "abbreviations" => Some(Self::ABBREVIATION),
            "attributes" | "attr" => Some(Self::ATTRIBUTES),
            "toc" | "table_of_contents" => Some(Self::TOC),
            "shortcodes" | "shortcode" => Some(Self::SHORTCODES),
            "tag_filter" | "tagfilter" => Some(Self::TAG_FILTER),
            "math" => Some(Self::MATH),
            "superscript" => Some(Self::SUPERSCRIPT),
            "subscript" => Some(Self::SUBSCRIPT),
            "underline" => Some(Self::UNDERLINE),
            "highlight" | "mark" => Some(Self::HIGHLIGHT),
            "emoji" | "emojis" => Some(Self::EMOJI),
            "wikilinks" | "wikilink" => Some(Self::WIKILINKS),
            "alerts" | "alert" => Some(Self::ALERTS),
            "mermaid" => Some(Self::MERMAID),
            "embed" | "embeds" => Some(Self::EMBED),
            "hardbreaks" | "hardbreak" => Some(Self::HARDBREAKS),
            "gfm" => Some(Self::GFM),
            "all" => Some(Self::ALL),
            _ => None,
        }
    }

    /// Get all individual extension flags.
    pub fn all_individual() -> Vec<Self> {
        vec![
            Self::TABLE,
            Self::STRIKETHROUGH,
            Self::TASKLIST,
            Self::AUTOLINK,
            Self::FOOTNOTES,
            Self::DEFINITION_LIST,
            Self::SMART_PUNCTUATION,
            Self::YAML_FRONT_MATTER,
            Self::ABBREVIATION,
            Self::ATTRIBUTES,
            Self::TOC,
            Self::SHORTCODES,
            Self::TAG_FILTER,
            Self::MATH,
            Self::SUPERSCRIPT,
            Self::SUBSCRIPT,
            Self::UNDERLINE,
            Self::HIGHLIGHT,
            Self::EMOJI,
            Self::WIKILINKS,
            Self::ALERTS,
            Self::MERMAID,
            Self::EMBED,
            Self::HARDBREAKS,
        ]
    }

    /// Get a list of enabled extension names.
    pub fn enabled_names(&self) -> Vec<&'static str> {
        Self::all_individual()
            .into_iter()
            .filter(|ext| self.contains(*ext))
            .filter_map(|ext| ext.name())
            .collect()
    }
}

impl fmt::Display for Extensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self.enabled_names();
        if names.is_empty() {
            write!(f, "none")
        } else {
            write!(f, "{}", names.join(", "))
        }
    }
}

impl FromStr for Extensions {
    type Err = ExtensionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Extensions::parse_name(s)
            .ok_or_else(|| ExtensionError::UnknownExtension(s.to_string()))
    }
}

/// Error type for extension operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionError {
    /// Unknown extension name.
    UnknownExtension(String),
    /// Invalid extension spec format.
    InvalidSpec(String),
    /// Extension not supported for format.
    UnsupportedForFormat {
        /// Extension name.
        extension: String,
        /// Format name.
        format: String,
    },
}

impl fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownExtension(name) => write!(f, "Unknown extension: {}", name),
            Self::InvalidSpec(spec) => write!(f, "Invalid extension spec: {}", spec),
            Self::UnsupportedForFormat { extension, format } => {
                write!(
                    f,
                    "Extension '{}' is not supported for format '{}'",
                    extension, format
                )
            }
        }
    }
}

impl std::error::Error for ExtensionError {}

/// Extension configuration for a format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtensionConfig {
    /// Default extensions for this format.
    pub default: Extensions,
    /// Supported extensions for this format.
    pub supported: Extensions,
}

impl ExtensionConfig {
    /// Create a new extension configuration.
    pub fn new(default: Extensions, supported: Extensions) -> Self {
        Self { default, supported }
    }

    /// Create configuration for CommonMark.
    pub fn commonmark() -> Self {
        Self {
            default: Extensions::empty(),
            supported: Extensions::ALL,
        }
    }

    /// Create configuration for GFM.
    pub fn gfm() -> Self {
        Self {
            default: Extensions::GFM,
            supported: Extensions::ALL,
        }
    }

    /// Create configuration for plain Markdown.
    pub fn plain() -> Self {
        Self {
            default: Extensions::empty(),
            supported: Extensions::empty(),
        }
    }

    /// Parse an extension spec string.
    ///
    /// Format: `format+ext1-ext2+ext3`
    ///
    /// Returns the format name, extensions to enable, and extensions to disable.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::extensions::ExtensionConfig;
    ///
    /// let (format, to_enable, to_disable) = ExtensionConfig::parse("markdown+table-strikethrough").unwrap();
    /// assert_eq!(format, "markdown");
    /// assert!(to_enable.contains(clmd::extensions::Extensions::TABLE));
    /// assert!(to_disable.contains(clmd::extensions::Extensions::STRIKETHROUGH));
    /// ```
    pub fn parse(
        spec: &str,
    ) -> Result<(String, Extensions, Extensions), ExtensionError> {
        if spec.is_empty() {
            return Err(ExtensionError::InvalidSpec(spec.to_string()));
        }

        // Find the first + or - to separate format from extensions
        let mut format_end = spec.len();
        let mut first_sign_pos = None;

        for (i, c) in spec.char_indices() {
            if c == '+' || c == '-' {
                format_end = i;
                first_sign_pos = Some(i);
                break;
            }
        }

        let format = spec[..format_end].to_string();
        let mut to_enable = Extensions::empty();
        let mut to_disable = Extensions::empty();

        // Parse extensions if there are any
        if let Some(start) = first_sign_pos {
            let mut sign = spec.chars().nth(start).unwrap();
            let mut ext_start = start + 1;

            for (i, c) in spec[start + 1..].char_indices() {
                let actual_pos = start + 1 + i;
                if c == '+' || c == '-' {
                    // Process the extension before this sign
                    if ext_start < actual_pos {
                        let ext_name = &spec[ext_start..actual_pos];
                        if let Some(ext) = Extensions::parse_name(ext_name) {
                            if sign == '+' {
                                to_enable |= ext;
                            } else {
                                to_disable |= ext;
                            }
                        } else {
                            return Err(ExtensionError::UnknownExtension(
                                ext_name.to_string(),
                            ));
                        }
                    }
                    sign = c;
                    ext_start = actual_pos + 1;
                }
            }

            // Process the last extension
            if ext_start < spec.len() {
                let ext_name = &spec[ext_start..];
                if let Some(ext) = Extensions::parse_name(ext_name) {
                    if sign == '+' {
                        to_enable |= ext;
                    } else {
                        to_disable |= ext;
                    }
                } else {
                    return Err(ExtensionError::UnknownExtension(ext_name.to_string()));
                }
            }
        }

        Ok((format, to_enable, to_disable))
    }

    /// Apply extension diff to the default set.
    ///
    /// The `to_enable` and `to_disable` parameters specify which extensions
    /// should be enabled or disabled relative to the default set.
    pub fn apply_diff(
        &self,
        to_enable: Extensions,
        to_disable: Extensions,
    ) -> Extensions {
        // Start with default
        let mut result = self.default;

        // Add enabled extensions that are supported
        result |= to_enable & self.supported;

        // Remove disabled extensions
        result &= !to_disable;

        result
    }

    /// Apply extensions to the default set (legacy method).
    ///
    /// This method simply ORs the extensions with the default set.
    /// For proper enable/disable semantics, use `apply_diff`.
    pub fn apply(&self, extensions: Extensions) -> Extensions {
        // Start with default
        let mut result = self.default;

        // Add enabled extensions that are supported
        result |= extensions & self.supported;

        result
    }

    /// Check if an extension is supported.
    pub fn is_supported(&self, extension: Extensions) -> bool {
        self.supported.contains(extension)
    }
}

impl Default for ExtensionConfig {
    fn default() -> Self {
        Self::commonmark()
    }
}

/// Extension registry for managing format-specific extensions.
#[derive(Debug, Clone)]
pub struct ExtensionRegistry {
    configs: std::collections::HashMap<String, ExtensionConfig>,
}

impl ExtensionRegistry {
    /// Create a new extension registry with default configurations.
    pub fn new() -> Self {
        let mut registry = Self {
            configs: std::collections::HashMap::new(),
        };

        // Register default formats
        registry.register("commonmark", ExtensionConfig::commonmark());
        registry.register("markdown", ExtensionConfig::commonmark());
        registry.register("gfm", ExtensionConfig::gfm());
        registry.register("plain", ExtensionConfig::plain());

        registry
    }

    /// Register a format with its extension configuration.
    pub fn register(&mut self, format: &str, config: ExtensionConfig) {
        self.configs.insert(format.to_lowercase(), config);
    }

    /// Get extension configuration for a format.
    pub fn get(&self, format: &str) -> Option<&ExtensionConfig> {
        self.configs.get(&format.to_lowercase())
    }

    /// Check if a format is registered.
    pub fn has_format(&self, format: &str) -> bool {
        self.configs.contains_key(&format.to_lowercase())
    }

    /// Get supported formats.
    pub fn formats(&self) -> Vec<&String> {
        self.configs.keys().collect()
    }

    /// Parse and apply extension spec for a format.
    pub fn parse_for_format(
        &self,
        format: &str,
        spec: &str,
    ) -> Result<Extensions, ExtensionError> {
        let config = self.get(format).ok_or_else(|| {
            ExtensionError::InvalidSpec(format!("Unknown format: {}", format))
        })?;

        let (_, to_enable, to_disable) =
            ExtensionConfig::parse(&format!("{}{}", format, spec))?;
        Ok(config.apply_diff(to_enable, to_disable))
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extensions_basic() {
        let ext = Extensions::TABLE | Extensions::STRIKETHROUGH;
        assert!(ext.contains(Extensions::TABLE));
        assert!(ext.contains(Extensions::STRIKETHROUGH));
        assert!(!ext.contains(Extensions::TASKLIST));
    }

    #[test]
    fn test_extensions_from_name() {
        assert_eq!(Extensions::parse_name("table"), Some(Extensions::TABLE));
        assert_eq!(Extensions::parse_name("TABLE"), Some(Extensions::TABLE));
        assert_eq!(Extensions::parse_name("unknown"), None);
    }

    #[test]
    fn test_extensions_name() {
        assert_eq!(Extensions::TABLE.name(), Some("table"));
        assert_eq!((Extensions::TABLE | Extensions::STRIKETHROUGH).name(), None);
    }

    #[test]
    fn test_extensions_enabled_names() {
        let ext = Extensions::TABLE | Extensions::STRIKETHROUGH;
        let names = ext.enabled_names();
        assert!(names.contains(&"table"));
        assert!(names.contains(&"strikethrough"));
        assert!(!names.contains(&"tasklist"));
    }

    #[test]
    fn test_extensions_display() {
        let ext = Extensions::TABLE | Extensions::STRIKETHROUGH;
        let s = ext.to_string();
        assert!(s.contains("table"));
        assert!(s.contains("strikethrough"));

        let empty = Extensions::empty();
        assert_eq!(empty.to_string(), "none");
    }

    #[test]
    fn test_extensions_from_str() {
        let ext: Extensions = "table".parse().unwrap();
        assert_eq!(ext, Extensions::TABLE);

        let result: Result<Extensions, _> = "unknown".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_extension_config_parse() {
        let (format, to_enable, to_disable) =
            ExtensionConfig::parse("markdown+table-strikethrough").unwrap();
        assert_eq!(format, "markdown");
        assert!(to_enable.contains(Extensions::TABLE));
        assert!(to_disable.contains(Extensions::STRIKETHROUGH));
        assert!(!to_enable.contains(Extensions::TASKLIST));
    }

    #[test]
    fn test_extension_config_parse_disable() {
        // Parse the extension diff
        let (format, to_enable, to_disable) =
            ExtensionConfig::parse("gfm-table").unwrap();
        assert_eq!(format, "gfm");
        // The parsed result contains what to enable/disable
        assert!(to_disable.contains(Extensions::TABLE));
        // To get the final extensions, use apply_diff() with a base config
        let config = ExtensionConfig::gfm();
        let final_ext = config.apply_diff(to_enable, to_disable);
        assert!(!final_ext.contains(Extensions::TABLE));
        assert!(final_ext.contains(Extensions::STRIKETHROUGH));
    }

    #[test]
    fn test_extension_config_parse_complex() {
        let (format, to_enable, to_disable) =
            ExtensionConfig::parse("markdown+table+footnotes-strikethrough").unwrap();
        assert_eq!(format, "markdown");
        assert!(to_enable.contains(Extensions::TABLE));
        assert!(to_enable.contains(Extensions::FOOTNOTES));
        assert!(to_disable.contains(Extensions::STRIKETHROUGH));
    }

    #[test]
    fn test_extension_config_apply() {
        let config = ExtensionConfig::commonmark();
        let ext = Extensions::TABLE | Extensions::FOOTNOTES;
        let result = config.apply(ext);

        assert!(result.contains(Extensions::TABLE));
        assert!(result.contains(Extensions::FOOTNOTES));
    }

    #[test]
    fn test_extension_registry() {
        let registry = ExtensionRegistry::new();
        assert!(registry.has_format("markdown"));
        assert!(registry.has_format("gfm"));
        assert!(!registry.has_format("unknown"));
    }

    #[test]
    fn test_extension_registry_parse() {
        let registry = ExtensionRegistry::new();
        let ext = registry.parse_for_format("markdown", "+table").unwrap();
        assert!(ext.contains(Extensions::TABLE));
    }

    #[test]
    fn test_gfm_extensions() {
        let gfm = Extensions::GFM;
        assert!(gfm.contains(Extensions::TABLE));
        assert!(gfm.contains(Extensions::STRIKETHROUGH));
        assert!(gfm.contains(Extensions::TASKLIST));
        assert!(gfm.contains(Extensions::AUTOLINK));
        assert!(gfm.contains(Extensions::ALERTS));
    }

    #[test]
    fn test_all_extensions() {
        let all = Extensions::ALL;
        assert!(all.contains(Extensions::GFM));
        assert!(all.contains(Extensions::FOOTNOTES));
        assert!(all.contains(Extensions::MATH));
    }
}
