//! Configuration file support for clmd
//!
//! This module provides support for loading configuration from TOML files.
//! Configuration files can be used to set default options for clmd.
//!
//! # Configuration File Locations
//!
//! The configuration file is searched in the following order:
//!
//! 1. Path specified by `--config` command line option
//! 2. `$XDG_CONFIG_HOME/clmd/config.toml` (Linux/macOS)
//! 3. `~/.config/clmd/config.toml` (Linux/macOS fallback)
//! 4. `%APPDATA%\clmd\config.toml` (Windows)
//!
//! # Configuration Format
//!
//! ```toml
//! # Input/Output formats
//! input_format = "markdown"
//! output_format = "html"
//!
//! # Extensions
//! [extensions]
//! table = true
//! strikethrough = true
//! tasklist = true
//! footnotes = true
//! autolink = true
//! tagfilter = true
//!
//! # Parse options
//! [parse]
//! smart = true
//!
//! # Render options
//! [render]
//! hardbreaks = false
//! unsafe = false
//! github_pre_lang = true
//!
//! # Syntax highlighting
//! [syntax]
//! theme = "css"  # or a theme name like "base16-ocean.dark"
//!
//! # Reader options
//! [reader]
//! default_format = "markdown"
//!
//! # Writer options
//! [writer]
//! default_format = "html"
//! template = "default.html"
//!
//! # Pipeline transforms
//! [[transforms]]
//! name = "header_shift"
//! shift = 1
//! ```

use crate::core::error::{ClmdError, ClmdResult};
use crate::extensions::Extensions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration structure for clmd
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
    pub extensions: ExtensionsConfig,

    /// Parse options
    #[serde(default)]
    pub parse: ParseConfig,

    /// Render options
    #[serde(default)]
    pub render: RenderConfig,

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

/// Extensions configuration
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ExtensionsConfig {
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
    pub multiline_block_quote: bool,

    /// Enable description list extension
    #[serde(default)]
    pub description_list: bool,

    /// Enable shortcode extension
    #[serde(default)]
    pub shortcode: bool,

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

impl ExtensionsConfig {
    /// Convert to Extensions bitflags
    pub fn to_extensions(&self) -> Extensions {
        let mut ext = Extensions::empty();
        if self.table {
            ext |= Extensions::TABLES;
        }
        if self.strikethrough {
            ext |= Extensions::STRIKETHROUGH;
        }
        if self.tasklist {
            ext |= Extensions::TASKLISTS;
        }
        if self.autolink {
            ext |= Extensions::AUTOLINKS;
        }
        if self.footnotes {
            ext |= Extensions::FOOTNOTES;
        }
        if self.description_list {
            ext |= Extensions::DESCRIPTION_LISTS;
        }
        if self.tagfilter {
            ext |= Extensions::TAGFILTER;
        }
        if self.superscript {
            ext |= Extensions::SUPERSCRIPT;
        }
        if self.subscript {
            ext |= Extensions::SUBSCRIPT;
        }
        if self.underline {
            ext |= Extensions::UNDERLINE;
        }
        if self.highlight {
            ext |= Extensions::HIGHLIGHT;
        }
        if self.math {
            ext |= Extensions::MATH_DOLLARS;
        }
        if self.wikilink {
            ext |= Extensions::WIKILINKS_TITLE_AFTER_PIPE;
        }
        if self.alerts {
            ext |= Extensions::ALERTS;
        }
        if self.yaml_front_matter {
            ext |= Extensions::YAML_FRONT_MATTER;
        }
        if self.abbreviation {
            ext |= Extensions::ABBREVIATIONS;
        }
        if self.attributes {
            ext |= Extensions::ATTRIBUTES;
        }
        if self.toc {
            ext |= Extensions::TOC;
        }
        if self.emoji {
            ext |= Extensions::SHORTCODES;
        }
        if self.insert {
            ext |= Extensions::INSERT;
        }
        if self.spoiler {
            ext |= Extensions::SPOILER;
        }
        if self.greentext {
            ext |= Extensions::GREENTEXT;
        }
        if self.multiline_block_quote {
            ext |= Extensions::MULTILINE_BLOCK_QUOTES;
        }
        if self.shortcode {
            ext |= Extensions::SHORTCODES;
        }
        ext
    }

    /// Create from Extensions bitflags
    pub fn from_extensions(ext: Extensions) -> Self {
        Self {
            table: ext.contains(Extensions::TABLES),
            strikethrough: ext.contains(Extensions::STRIKETHROUGH),
            tasklist: ext.contains(Extensions::TASKLISTS),
            autolink: ext.contains(Extensions::AUTOLINKS),
            footnotes: ext.contains(Extensions::FOOTNOTES),
            tagfilter: ext.contains(Extensions::TAGFILTER),
            superscript: ext.contains(Extensions::SUPERSCRIPT),
            subscript: ext.contains(Extensions::SUBSCRIPT),
            underline: ext.contains(Extensions::UNDERLINE),
            highlight: ext.contains(Extensions::HIGHLIGHT),
            insert: ext.contains(Extensions::INSERT),
            math: ext.contains(Extensions::MATH_DOLLARS),
            wikilink: ext.contains(Extensions::WIKILINKS_TITLE_AFTER_PIPE)
                || ext.contains(Extensions::WIKILINKS_TITLE_BEFORE_PIPE),
            spoiler: ext.contains(Extensions::SPOILER),
            greentext: ext.contains(Extensions::GREENTEXT),
            alerts: ext.contains(Extensions::ALERTS),
            multiline_block_quote: ext.contains(Extensions::MULTILINE_BLOCK_QUOTES),
            description_list: ext.contains(Extensions::DESCRIPTION_LISTS),
            shortcode: ext.contains(Extensions::SHORTCODES),
            yaml_front_matter: ext.contains(Extensions::YAML_FRONT_MATTER),
            abbreviation: ext.contains(Extensions::ABBREVIATIONS),
            attributes: ext.contains(Extensions::ATTRIBUTES),
            toc: ext.contains(Extensions::TOC),
            emoji: ext.contains(Extensions::SHORTCODES),
        }
    }
}

/// Parse configuration
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ParseConfig {
    /// Enable smart punctuation
    #[serde(default)]
    pub smart: bool,

    /// Enable relaxed tasklist matching
    #[serde(default)]
    pub relaxed_tasklist: bool,

    /// Enable relaxed autolinks
    #[serde(default)]
    pub relaxed_autolinks: bool,

    /// Maximum nesting depth (0 = unlimited)
    #[serde(default)]
    pub max_nesting_depth: usize,
}

/// Render configuration
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
}

/// Syntax highlighting configuration
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

/// Reader configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReaderConfig {
    /// Default input format
    #[serde(default)]
    pub default_format: String,

    /// Additional reader options
    #[serde(default)]
    pub options: HashMap<String, toml::Value>,
}

/// Writer configuration
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

/// Transform configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    /// Transform name
    pub name: String,

    /// Transform parameters
    #[serde(default)]
    pub params: HashMap<String, toml::Value>,
}

impl Config {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> ClmdResult<Self> {
        let content = fs::read_to_string(&path).map_err(|e| {
            ClmdError::config_error(format!("Failed to read config file: {}", e))
        })?;
        let config: Config = toml::from_str(&content).map_err(|e| {
            ClmdError::config_error(format!("Failed to parse config file: {}", e))
        })?;
        Ok(config)
    }

    /// Find and load the default configuration file
    pub fn load_default() -> Option<Self> {
        if let Some(path) = find_config_file() {
            match Self::from_file(&path) {
                Ok(config) => return Some(config),
                Err(e) => eprintln!(
                    "Warning: failed to load config file {}: {}",
                    path.display(),
                    e
                ),
            }
        }
        None
    }

    /// Apply configuration to Options
    pub fn apply_to_options(&self, options: &mut crate::Options) {
        // Apply extension options
        options.extension.table = self.extensions.table;
        options.extension.strikethrough = self.extensions.strikethrough;
        options.extension.tasklist = self.extensions.tasklist;
        options.extension.footnotes = self.extensions.footnotes;
        options.extension.autolink = self.extensions.autolink;
        options.extension.tagfilter = self.extensions.tagfilter;
        options.extension.superscript = self.extensions.superscript;
        options.extension.subscript = self.extensions.subscript;
        options.extension.underline = self.extensions.underline;
        options.extension.highlight = self.extensions.highlight;
        options.extension.insert = self.extensions.insert;
        options.extension.math_dollars = self.extensions.math;
        options.extension.wikilinks_title_after_pipe = self.extensions.wikilink;
        options.extension.spoiler = self.extensions.spoiler;
        options.extension.greentext = self.extensions.greentext;
        options.extension.alerts = self.extensions.alerts;
        options.extension.multiline_block_quotes = self.extensions.multiline_block_quote;
        options.extension.description_lists = self.extensions.description_list;
        options.extension.shortcodes = self.extensions.shortcode;

        // Apply parse options
        options.parse.smart = self.parse.smart;
        options.parse.relaxed_tasklist_matching = self.parse.relaxed_tasklist;
        options.parse.relaxed_autolinks = self.parse.relaxed_autolinks;

        // Apply render options
        options.render.hardbreaks = self.render.hardbreaks;
        options.render.r#unsafe = self.render.r#unsafe;
        options.render.github_pre_lang = self.render.github_pre_lang;
        options.render.full_info_string = self.render.full_info_string;
        options.render.sourcepos = self.render.sourcepos;
        options.render.compact_html = self.render.compact;
        options.render.escape = self.render.escape;
    }

    /// Merge another configuration into this one
    pub fn merge(&mut self, other: &Config) {
        if other.input_format.is_some() {
            self.input_format = other.input_format.clone();
        }
        if other.output_format.is_some() {
            self.output_format = other.output_format.clone();
        }
        // Merge extensions (other takes precedence for true values)
        self.extensions = ExtensionsConfig {
            table: other.extensions.table || self.extensions.table,
            strikethrough: other.extensions.strikethrough
                || self.extensions.strikethrough,
            tasklist: other.extensions.tasklist || self.extensions.tasklist,
            footnotes: other.extensions.footnotes || self.extensions.footnotes,
            autolink: other.extensions.autolink || self.extensions.autolink,
            tagfilter: other.extensions.tagfilter || self.extensions.tagfilter,
            superscript: other.extensions.superscript || self.extensions.superscript,
            subscript: other.extensions.subscript || self.extensions.subscript,
            underline: other.extensions.underline || self.extensions.underline,
            highlight: other.extensions.highlight || self.extensions.highlight,
            insert: other.extensions.insert || self.extensions.insert,
            math: other.extensions.math || self.extensions.math,
            wikilink: other.extensions.wikilink || self.extensions.wikilink,
            spoiler: other.extensions.spoiler || self.extensions.spoiler,
            greentext: other.extensions.greentext || self.extensions.greentext,
            alerts: other.extensions.alerts || self.extensions.alerts,
            multiline_block_quote: other.extensions.multiline_block_quote
                || self.extensions.multiline_block_quote,
            description_list: other.extensions.description_list
                || self.extensions.description_list,
            shortcode: other.extensions.shortcode || self.extensions.shortcode,
            yaml_front_matter: other.extensions.yaml_front_matter
                || self.extensions.yaml_front_matter,
            abbreviation: other.extensions.abbreviation || self.extensions.abbreviation,
            attributes: other.extensions.attributes || self.extensions.attributes,
            toc: other.extensions.toc || self.extensions.toc,
            emoji: other.extensions.emoji || self.extensions.emoji,
        };
        // Merge other fields...
        self.transforms.extend(other.transforms.clone());
        self.options.extend(other.options.clone());
    }

    /// Get extensions as bitflags
    pub fn get_extensions(&self) -> Extensions {
        self.extensions.to_extensions()
    }

    /// Set extensions from bitflags
    pub fn set_extensions(&mut self, ext: Extensions) {
        self.extensions = ExtensionsConfig::from_extensions(ext);
    }
}

/// Find the default configuration file path
fn find_config_file() -> Option<PathBuf> {
    // Try XDG config directory first
    if let Some(config_dir) = dirs::config_dir() {
        let path = config_dir.join("clmd").join("config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    // Try home directory
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".config").join("clmd").join("config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Get the default configuration file path (for creating new config)
pub fn default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("clmd").join("config.toml"))
}

/// Configuration loader that handles multiple config sources
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    /// Base configuration (lowest priority)
    base_config: Option<Config>,
    /// User configuration file
    user_config: Option<Config>,
    /// CLI configuration (highest priority)
    cli_config: Option<Config>,
}

impl ConfigLoader {
    /// Create a new config loader
    pub fn new() -> Self {
        Self {
            base_config: None,
            user_config: Config::load_default(),
            cli_config: None,
        }
    }

    /// Load configuration from a file
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> ClmdResult<Self> {
        self.user_config = Some(Config::from_file(path)?);
        Ok(self)
    }

    /// Set CLI configuration
    pub fn with_cli(mut self, config: Config) -> Self {
        self.cli_config = Some(config);
        self
    }

    /// Build the final configuration
    pub fn build(self) -> Config {
        let mut result = self.base_config.unwrap_or_default();

        if let Some(user) = self.user_config {
            result.merge(&user);
        }

        if let Some(cli) = self.cli_config {
            result.merge(&cli);
        }

        result
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_from_file() {
        let config_content = r#"
input_format = "markdown"
output_format = "html"

[extensions]
table = true
strikethrough = true

[parse]
smart = true

[render]
hardbreaks = false
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(config.input_format, Some("markdown".to_string()));
        assert_eq!(config.output_format, Some("html".to_string()));
        assert!(config.extensions.table);
        assert!(config.extensions.strikethrough);
        assert!(!config.extensions.tasklist);
        assert!(config.parse.smart);
        assert!(!config.render.hardbreaks);
    }

    #[test]
    fn test_config_apply_to_options() {
        let config = Config {
            extensions: ExtensionsConfig {
                table: true,
                strikethrough: true,
                ..Default::default()
            },
            parse: ParseConfig {
                smart: true,
                ..Default::default()
            },
            render: RenderConfig {
                hardbreaks: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut options = crate::Options::default();
        config.apply_to_options(&mut options);

        assert!(options.extension.table);
        assert!(options.extension.strikethrough);
        assert!(options.parse.smart);
        assert!(options.render.hardbreaks);
    }

    #[test]
    fn test_empty_config() {
        let config = Config::default();
        assert!(!config.extensions.table);
        assert!(!config.parse.smart);
        assert!(!config.render.hardbreaks);
    }

    #[test]
    fn test_config_merge() {
        let mut base = Config {
            extensions: ExtensionsConfig {
                table: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let override_config = Config {
            extensions: ExtensionsConfig {
                strikethrough: true,
                ..Default::default()
            },
            ..Default::default()
        };

        base.merge(&override_config);

        assert!(base.extensions.table); // Preserved
        assert!(base.extensions.strikethrough); // Added
    }

    #[test]
    fn test_extensions_to_bitflags() {
        let config = ExtensionsConfig {
            table: true,
            strikethrough: true,
            ..Default::default()
        };

        let ext = config.to_extensions();
        assert!(ext.contains(Extensions::TABLES));
        assert!(ext.contains(Extensions::STRIKETHROUGH));
        assert!(!ext.contains(Extensions::TASKLISTS));
    }

    #[test]
    fn test_extensions_from_bitflags() {
        let ext = Extensions::TABLES | Extensions::STRIKETHROUGH;
        let config = ExtensionsConfig::from_extensions(ext);

        assert!(config.table);
        assert!(config.strikethrough);
        assert!(!config.tasklist);
    }

    #[test]
    fn test_config_loader() {
        let loader = ConfigLoader::new();
        let config = loader.build();
        // Should return default config
        assert!(config.input_format.is_none());
    }
}
