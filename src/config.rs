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
//! [extension]
//! table = true
//! strikethrough = true
//! tasklist = true
//! footnotes = true
//! autolink = true
//! tagfilter = true
//!
//! [parse]
//! smart = true
//!
//! [render]
//! hardbreaks = false
//! unsafe = false
//! github_pre_lang = true
//!
//! [syntax]
//! theme = "css"  # or a theme name like "base16-ocean.dark"
//! ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration structure for clmd
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// Extension options
    #[serde(default)]
    pub extension: ExtensionConfig,

    /// Parse options
    #[serde(default)]
    pub parse: ParseConfig,

    /// Render options
    #[serde(default)]
    pub render: RenderConfig,

    /// Syntax highlighting options
    #[serde(default)]
    pub syntax: SyntaxConfig,
}

/// Extension configuration
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
    pub multiline_block_quote: bool,

    /// Enable description list extension
    #[serde(default)]
    pub description_list: bool,

    /// Enable shortcode extension
    #[serde(default)]
    pub shortcode: bool,
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
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SyntaxConfig {
    /// Theme name for syntax highlighting
    /// Use "css" for CSS class mode, or a theme name for inline styles
    #[serde(default)]
    pub theme: Option<String>,
}

impl Config {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
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
        options.extension.table = self.extension.table;
        options.extension.strikethrough = self.extension.strikethrough;
        options.extension.tasklist = self.extension.tasklist;
        options.extension.footnotes = self.extension.footnotes;
        options.extension.autolink = self.extension.autolink;
        options.extension.tagfilter = self.extension.tagfilter;
        options.extension.superscript = self.extension.superscript;
        options.extension.subscript = self.extension.subscript;
        options.extension.underline = self.extension.underline;
        options.extension.highlight = self.extension.highlight;
        options.extension.insert = self.extension.insert;
        options.extension.math_dollars = self.extension.math;
        options.extension.wikilinks_title_after_pipe = self.extension.wikilink;
        options.extension.spoiler = self.extension.spoiler;
        options.extension.greentext = self.extension.greentext;
        options.extension.alerts = self.extension.alerts;
        options.extension.multiline_block_quotes = self.extension.multiline_block_quote;
        options.extension.description_lists = self.extension.description_list;
        options.extension.shortcodes = self.extension.shortcode;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_from_file() {
        let config_content = r#"
[extension]
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
        assert!(config.extension.table);
        assert!(config.extension.strikethrough);
        assert!(!config.extension.tasklist);
        assert!(config.parse.smart);
        assert!(!config.render.hardbreaks);
    }

    #[test]
    fn test_config_apply_to_options() {
        let config = Config {
            extension: ExtensionConfig {
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
            syntax: SyntaxConfig::default(),
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
        assert!(!config.extension.table);
        assert!(!config.parse.smart);
        assert!(!config.render.hardbreaks);
    }
}
