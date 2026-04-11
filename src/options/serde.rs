//! Serializable configuration for TOML file support.
//!
//! This module provides configuration structures that can be serialized
//! and deserialized from TOML files, with conversion to/from runtime options.

use crate::ext::flags::ExtensionFlags;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration structure for clmd (serializable for TOML)
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default input format
    #[serde(default)]
    pub input_format: Option<String>,

    /// Default output format
    #[serde(default)]
    pub output_format: Option<String>,

    /// Extension options
    #[serde(default)]
    pub extensions: ExtensionConfig,

    /// Parse options
    #[serde(default)]
    pub parse: ParseConfig,

    /// Render options
    #[serde(default)]
    pub render: RenderConfig,

    /// Format options
    #[serde(default)]
    pub format: FormatConfig,

    /// Syntax highlighting options
    #[serde(default)]
    pub syntax: SyntaxConfig,

    /// Reader options
    #[serde(default)]
    pub reader: ReaderConfig,

    /// Writer options
    #[serde(default)]
    pub writer: WriterConfig,

    /// Pipeline transforms
    #[serde(default)]
    pub transforms: Vec<TransformConfig>,

    /// Additional options as key-value pairs
    #[serde(default)]
    pub options: HashMap<String, toml::Value>,
}

/// Extensions configuration (serializable)
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ExtensionConfig {
    /// Enable table extension
    #[serde(default)]
    pub table: bool,

    /// Enable strikethrough extension
    #[serde(default)]
    pub strikethrough: bool,

    /// Enable tasklist extension
    #[serde(default)]
    pub tasklist: bool,

    /// Enable footnotes extension
    #[serde(default)]
    pub footnotes: bool,

    /// Enable autolink extension
    #[serde(default)]
    pub autolink: bool,

    /// Enable tagfilter extension
    #[serde(default)]
    pub tagfilter: bool,

    /// Enable superscript extension
    #[serde(default)]
    pub superscript: bool,

    /// Enable subscript extension
    #[serde(default)]
    pub subscript: bool,

    /// Enable underline extension
    #[serde(default)]
    pub underline: bool,

    /// Enable highlight extension
    #[serde(default)]
    pub highlight: bool,

    /// Enable insert extension
    #[serde(default)]
    pub insert: bool,

    /// Enable math extension
    #[serde(default)]
    pub math: bool,

    /// Enable wikilink extension
    #[serde(default)]
    pub wikilink: bool,

    /// Enable spoiler extension
    #[serde(default)]
    pub spoiler: bool,

    /// Enable greentext extension
    #[serde(default)]
    pub greentext: bool,

    /// Enable alerts extension
    #[serde(default)]
    pub alerts: bool,

    /// Enable multiline block quote extension
    #[serde(default)]
    pub multiline_block_quotes: bool,

    /// Enable description list extension
    #[serde(default)]
    pub description_lists: bool,

    /// Enable shortcode extension
    #[serde(default)]
    pub shortcodes: bool,

    /// Enable YAML front matter extension
    #[serde(default)]
    pub yaml_front_matter: bool,

    /// Enable abbreviation extension
    #[serde(default)]
    pub abbreviation: bool,

    /// Enable attributes extension
    #[serde(default)]
    pub attributes: bool,

    /// Enable TOC extension
    #[serde(default)]
    pub toc: bool,

    /// Enable emoji extension
    #[serde(default)]
    pub emoji: bool,
}

impl ExtensionConfig {
    /// Convert to ExtensionFlags bitflags
    pub fn to_extensions(&self) -> ExtensionFlags {
        let mut ext = ExtensionFlags::empty();
        if self.table {
            ext |= ExtensionFlags::TABLES;
        }
        if self.strikethrough {
            ext |= ExtensionFlags::STRIKETHROUGH;
        }
        if self.tasklist {
            ext |= ExtensionFlags::TASKLISTS;
        }
        if self.autolink {
            ext |= ExtensionFlags::AUTOLINKS;
        }
        if self.footnotes {
            ext |= ExtensionFlags::FOOTNOTES;
        }
        if self.description_lists {
            ext |= ExtensionFlags::DESCRIPTION_LISTS;
        }
        if self.tagfilter {
            ext |= ExtensionFlags::TAGFILTER;
        }
        if self.superscript {
            ext |= ExtensionFlags::SUPERSCRIPT;
        }
        if self.subscript {
            ext |= ExtensionFlags::SUBSCRIPT;
        }
        if self.underline {
            ext |= ExtensionFlags::UNDERLINE;
        }
        if self.highlight {
            ext |= ExtensionFlags::HIGHLIGHT;
        }
        if self.math {
            ext |= ExtensionFlags::MATH_DOLLARS;
        }
        if self.wikilink {
            ext |= ExtensionFlags::WIKILINKS_TITLE_AFTER_PIPE;
        }
        if self.alerts {
            ext |= ExtensionFlags::ALERTS;
        }
        if self.yaml_front_matter {
            ext |= ExtensionFlags::YAML_FRONT_MATTER;
        }
        if self.abbreviation {
            ext |= ExtensionFlags::ABBREVIATIONS;
        }
        if self.attributes {
            ext |= ExtensionFlags::ATTRIBUTES;
        }
        if self.toc {
            ext |= ExtensionFlags::TOC;
        }
        if self.emoji || self.shortcodes {
            ext |= ExtensionFlags::SHORTCODES;
        }
        if self.insert {
            ext |= ExtensionFlags::INSERT;
        }
        if self.spoiler {
            ext |= ExtensionFlags::SPOILER;
        }
        if self.greentext {
            ext |= ExtensionFlags::GREENTEXT;
        }
        if self.multiline_block_quotes {
            ext |= ExtensionFlags::MULTILINE_BLOCK_QUOTES;
        }
        ext
    }

    /// Create from ExtensionFlags bitflags
    pub fn from_extensions(ext: ExtensionFlags) -> Self {
        Self {
            table: ext.contains(ExtensionFlags::TABLES),
            strikethrough: ext.contains(ExtensionFlags::STRIKETHROUGH),
            tasklist: ext.contains(ExtensionFlags::TASKLISTS),
            autolink: ext.contains(ExtensionFlags::AUTOLINKS),
            footnotes: ext.contains(ExtensionFlags::FOOTNOTES),
            tagfilter: ext.contains(ExtensionFlags::TAGFILTER),
            superscript: ext.contains(ExtensionFlags::SUPERSCRIPT),
            subscript: ext.contains(ExtensionFlags::SUBSCRIPT),
            underline: ext.contains(ExtensionFlags::UNDERLINE),
            highlight: ext.contains(ExtensionFlags::HIGHLIGHT),
            insert: ext.contains(ExtensionFlags::INSERT),
            math: ext.contains(ExtensionFlags::MATH_DOLLARS),
            wikilink: ext.contains(ExtensionFlags::WIKILINKS_TITLE_AFTER_PIPE)
                || ext.contains(ExtensionFlags::WIKILINKS_TITLE_BEFORE_PIPE),
            spoiler: ext.contains(ExtensionFlags::SPOILER),
            greentext: ext.contains(ExtensionFlags::GREENTEXT),
            alerts: ext.contains(ExtensionFlags::ALERTS),
            multiline_block_quotes: ext.contains(ExtensionFlags::MULTILINE_BLOCK_QUOTES),
            description_lists: ext.contains(ExtensionFlags::DESCRIPTION_LISTS),
            shortcodes: ext.contains(ExtensionFlags::SHORTCODES),
            yaml_front_matter: ext.contains(ExtensionFlags::YAML_FRONT_MATTER),
            abbreviation: ext.contains(ExtensionFlags::ABBREVIATIONS),
            attributes: ext.contains(ExtensionFlags::ATTRIBUTES),
            toc: ext.contains(ExtensionFlags::TOC),
            emoji: ext.contains(ExtensionFlags::SHORTCODES),
        }
    }
}

/// Parse configuration (serializable)
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ParseConfig {
    /// Enable smart punctuation
    #[serde(default)]
    pub smart: bool,

    /// Enable relaxed tasklist matching
    #[serde(default)]
    pub relaxed_tasklist_matching: bool,

    /// Enable relaxed autolinks
    #[serde(default)]
    pub relaxed_autolinks: bool,

    /// Maximum nesting depth (0 = unlimited)
    #[serde(default)]
    pub max_nesting_depth: usize,

    /// Include source position
    #[serde(default)]
    pub sourcepos: bool,
}

/// Render configuration (serializable)
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct RenderConfig {
    /// Enable hard line breaks
    #[serde(default)]
    pub hardbreaks: bool,

    /// Allow unsafe URLs (javascript:, data:, etc.)
    #[serde(default)]
    pub r#unsafe: bool,

    /// Use GitHub-style pre lang attribute
    #[serde(default)]
    pub github_pre_lang: bool,

    /// Include full info string
    #[serde(default)]
    pub full_info_string: bool,

    /// Source position attribute
    #[serde(default)]
    pub sourcepos: bool,

    /// Compact HTML output
    #[serde(default)]
    pub compact: bool,

    /// Escape HTML instead of passing through
    #[serde(default)]
    pub escape: bool,

    /// Wrap width (0 = no wrapping)
    #[serde(default)]
    pub width: usize,
}

/// Format configuration (serializable)
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    /// Heading style (atx, setext, as_is)
    #[serde(default)]
    pub heading_style: Option<String>,

    /// Right margin for wrapping
    #[serde(default)]
    pub right_margin: usize,

    /// Maximum blank lines
    #[serde(default)]
    pub max_blank_lines: usize,

    /// List bullet marker (dash, asterisk, plus)
    #[serde(default)]
    pub list_bullet_marker: Option<String>,

    /// List spacing (tight, loose, as_is)
    #[serde(default)]
    pub list_spacing: Option<String>,
}

/// Syntax highlighting configuration (serializable)
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SyntaxConfig {
    /// Theme name for syntax highlighting
    /// Use "css" for CSS class mode, or a theme name for inline styles
    #[serde(default)]
    pub theme: Option<String>,

    /// Enable syntax highlighting
    #[serde(default)]
    pub enabled: bool,
}

/// Reader configuration (serializable)
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReaderConfig {
    /// Default input format
    #[serde(default)]
    pub default_format: String,

    /// Additional reader options
    #[serde(default)]
    pub options: HashMap<String, toml::Value>,
}

/// Writer configuration (serializable)
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WriterConfig {
    /// Default output format
    #[serde(default)]
    pub default_format: String,

    /// Template file path
    #[serde(default)]
    pub template: Option<PathBuf>,

    /// Additional writer options
    #[serde(default)]
    pub options: HashMap<String, toml::Value>,
}

/// Transform configuration (serializable)
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    /// Transform name
    pub name: String,

    /// Transform parameters
    #[serde(default)]
    pub params: HashMap<String, toml::Value>,
}

impl Config {
    /// Create an empty configuration
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.input_format.is_none());
        assert!(config.output_format.is_none());
        assert!(!config.extensions.table);
        assert!(!config.parse.smart);
        assert!(!config.render.hardbreaks);
    }

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert!(config.input_format.is_none());
    }

    #[test]
    fn test_extension_config_to_bitflags() {
        let config = ExtensionConfig {
            table: true,
            strikethrough: true,
            ..Default::default()
        };

        let ext = config.to_extensions();
        assert!(ext.contains(ExtensionFlags::TABLES));
        assert!(ext.contains(ExtensionFlags::STRIKETHROUGH));
        assert!(!ext.contains(ExtensionFlags::TASKLISTS));
    }

    #[test]
    fn test_extension_config_from_bitflags() {
        let ext = ExtensionFlags::TABLES | ExtensionFlags::STRIKETHROUGH;
        let config = ExtensionConfig::from_extensions(ext);

        assert!(config.table);
        assert!(config.strikethrough);
        assert!(!config.tasklist);
    }
}
