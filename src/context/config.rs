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
//! params = { shift = 1 }
//! ```

// Re-export all config types from the unified options::serde module
pub use crate::options::serde::{
    Config, ExtensionConfig, FormatConfig, ParseConfig, ReaderConfig, RenderConfig,
    SyntaxConfig, TransformConfig, WriterConfig,
};

use crate::core::error::{ClmdError, ClmdResult};
use crate::ext::flags::ExtensionFlags;
use std::fs;
use std::path::{Path, PathBuf};

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
        options.extension.multiline_block_quotes =
            self.extensions.multiline_block_quotes;
        options.extension.description_lists = self.extensions.description_lists;
        options.extension.shortcodes = self.extensions.shortcodes;

        // Apply parse options
        options.parse.smart = self.parse.smart;
        options.parse.relaxed_tasklist_matching = self.parse.relaxed_tasklist_matching;
        options.parse.relaxed_autolinks = self.parse.relaxed_autolinks;
        options.parse.sourcepos = self.parse.sourcepos;

        // Apply render options
        options.render.hardbreaks = self.render.hardbreaks;
        options.render.r#unsafe = self.render.r#unsafe;
        options.render.github_pre_lang = self.render.github_pre_lang;
        options.render.full_info_string = self.render.full_info_string;
        options.render.sourcepos = self.render.sourcepos;
        options.render.compact_html = self.render.compact;
        options.render.escape = self.render.escape;
        options.render.width = self.render.width;
        options.render.cjk_spacing = self.render.cjk_spacing;
    }

    /// Get extensions as bitflags
    pub fn get_extensions(&self) -> ExtensionFlags {
        self.extensions.to_extensions()
    }

    /// Set extensions from bitflags
    pub fn set_extensions(&mut self, ext: ExtensionFlags) {
        self.extensions = ExtensionConfig::from_extensions(ext);
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
            extensions: ExtensionConfig {
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
            extensions: ExtensionConfig {
                table: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let override_config = Config {
            extensions: ExtensionConfig {
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
    fn test_extensions_from_bitflags() {
        let ext = ExtensionFlags::TABLES | ExtensionFlags::STRIKETHROUGH;
        let config = ExtensionConfig::from_extensions(ext);

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
