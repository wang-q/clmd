//! Configuration file support for clmd

// Re-export all config types from the unified options::serde module
pub use crate::options::serde::{
    Config, ExtensionConfig, FormatConfig, ParseConfig, ReaderConfig, RenderConfig,
    SyntaxConfig, TransformConfig, WriterConfig,
};

use crate::core::error::{ClmdError, ClmdResult};
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

        options.parse.smart = self.parse.smart;
        options.parse.relaxed_tasklist_matching = self.parse.relaxed_tasklist_matching;
        options.parse.relaxed_autolinks = self.parse.relaxed_autolinks;
        options.parse.sourcepos = self.parse.sourcepos;

        options.render.hardbreaks = self.render.hardbreaks;
        options.render.r#unsafe = self.render.r#unsafe;
        options.render.github_pre_lang = self.render.github_pre_lang;
        options.render.full_info_string = self.render.full_info_string;
        options.render.sourcepos = self.render.sourcepos;
        options.render.compact_html = self.render.compact;
        options.render.escape = self.render.escape;
        options.render.width = self.render.width;
    }
}

/// Find the default configuration file path
fn find_config_file() -> Option<PathBuf> {
    if let Some(config_dir) = dirs::config_dir() {
        let path = config_dir.join("clmd").join("config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let path = home.join(".config").join("clmd").join("config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    None
}
