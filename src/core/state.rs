//! Common state management for clmd.
//!
//! This module provides a unified state container for clmd operations,
//! inspired by Pandoc's CommonState. It holds shared state that is
//! accessible throughout the document processing pipeline.
//!
//! # Example
//!
//! ```ignore
//! use clmd::core::{CommonState, Verbosity};
//!
//! let mut state = CommonState::new();
//! state.verbosity = Verbosity::Info;
//! state.add_input_file("document.md");
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::context::mediabag::MediaBag;
use crate::io::format::mime::MimeType;

use super::monad::Verbosity;

/// Common state shared across clmd operations.
///
/// This structure holds configuration and state that is used throughout
/// the document processing pipeline. It's similar to Pandoc's CommonState.
#[derive(Debug, Clone)]
pub struct CommonState {
    /// Verbosity level for logging.
    pub verbosity: Verbosity,

    /// Resource search paths.
    pub resource_path: Vec<PathBuf>,

    /// User data directory.
    pub user_data_dir: Option<PathBuf>,

    /// Input files to process.
    pub input_files: Vec<PathBuf>,

    /// Output file (None for stdout).
    pub output_file: Option<PathBuf>,

    /// Source URL for remote resources.
    pub source_url: Option<String>,

    /// Request headers for HTTP requests.
    pub request_headers: Vec<(String, String)>,

    /// Media bag for storing binary resources.
    pub media_bag: MediaBag,

    /// Translations for localization.
    pub translations: Option<Translations>,

    /// Current language for localization.
    pub lang: Option<String>,

    /// Whether to trace operations.
    pub trace: bool,

    /// Whether to check SSL certificates.
    pub no_check_certificate: bool,

    /// Accumulated log messages.
    pub log_messages: Vec<LogMessage>,

    /// Current timestamp.
    pub timestamp: SystemTime,

    /// Track changes mode.
    pub track_changes: TrackChanges,

    /// Abbreviations for smart punctuation.
    pub abbreviations: Vec<String>,

    /// Default image extension.
    pub default_image_extension: String,

    /// Tab stop width.
    pub tab_stop: usize,

    /// Column width for wrapping.
    pub columns: usize,

    /// Extension-specific data.
    pub extensions_data: HashMap<String, ExtensionData>,
}

// Re-export LogMessage and LogLevel from error module for backward compatibility
pub use crate::core::error::{LogLevel, LogMessage};

/// Track changes mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrackChanges {
    /// Accept all changes.
    AcceptChanges,
    /// Reject all changes.
    RejectChanges,
    /// Track changes (default).
    #[default]
    TrackChanges,
}

/// Translations for localization.
#[derive(Debug, Clone, Default)]
pub struct Translations {
    /// Language code (e.g., "en-US").
    pub lang: String,
    /// Translation map: term -> translated string.
    pub terms: HashMap<String, String>,
}

/// Extension-specific data container.
#[derive(Debug, Clone)]
pub struct ExtensionData {
    /// Arbitrary data stored as strings.
    pub data: HashMap<String, String>,
}

impl CommonState {
    /// Create a new CommonState with default values.
    pub fn new() -> Self {
        Self {
            verbosity: Verbosity::Warning,
            resource_path: vec![PathBuf::from(".")],
            user_data_dir: dirs::data_dir().map(|d| d.join("clmd")),
            input_files: Vec::new(),
            output_file: None,
            source_url: None,
            request_headers: Vec::new(),
            media_bag: MediaBag::new(),
            translations: None,
            lang: Some("en-US".to_string()),
            trace: false,
            no_check_certificate: false,
            log_messages: Vec::new(),
            timestamp: SystemTime::now(),
            track_changes: TrackChanges::default(),
            abbreviations: Vec::new(),
            default_image_extension: String::new(),
            tab_stop: 4,
            columns: 80,
            extensions_data: HashMap::new(),
        }
    }

    /// Create a new CommonState for testing.
    pub fn test_state() -> Self {
        Self {
            verbosity: Verbosity::Warning,
            resource_path: vec![PathBuf::from("/test")],
            user_data_dir: Some(PathBuf::from("/test/.local/share/clmd")),
            input_files: Vec::new(),
            output_file: None,
            source_url: None,
            request_headers: Vec::new(),
            media_bag: MediaBag::new(),
            translations: None,
            lang: Some("en-US".to_string()),
            trace: false,
            no_check_certificate: false,
            log_messages: Vec::new(),
            timestamp: SystemTime::UNIX_EPOCH,
            track_changes: TrackChanges::default(),
            abbreviations: Vec::new(),
            default_image_extension: String::new(),
            tab_stop: 4,
            columns: 80,
            extensions_data: HashMap::new(),
        }
    }

    /// Add an input file.
    pub fn add_input_file<P: AsRef<Path>>(&mut self, path: P) {
        self.input_files.push(path.as_ref().to_path_buf());
    }

    /// Set the output file.
    pub fn set_output_file<P: AsRef<Path>>(&mut self, path: P) {
        self.output_file = Some(path.as_ref().to_path_buf());
    }

    /// Add a resource path.
    pub fn add_resource_path<P: AsRef<Path>>(&mut self, path: P) {
        self.resource_path.push(path.as_ref().to_path_buf());
    }

    /// Add a request header.
    pub fn add_request_header(&mut self, key: String, value: String) {
        self.request_headers.push((key, value));
    }

    /// Log a message.
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        let msg = LogMessage::new(level, message);
        self.log_messages.push(msg);
    }

    /// Log a debug message.
    pub fn log_debug(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Debug, message);
    }

    /// Log an info message.
    pub fn log_info(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Info, message);
    }

    /// Log a warning.
    pub fn log_warning(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Warning, message);
    }

    /// Log an error.
    pub fn log_error(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Error, message);
    }

    /// Get all log messages.
    pub fn get_logs(&self) -> &[LogMessage] {
        &self.log_messages
    }

    /// Get log messages at or above a certain level.
    pub fn get_logs_at_level(&self, level: LogLevel) -> Vec<&LogMessage> {
        self.log_messages
            .iter()
            .filter(|msg| msg.level >= level)
            .collect()
    }

    /// Clear all log messages.
    pub fn clear_logs(&mut self) {
        self.log_messages.clear();
    }

    /// Set translations.
    pub fn set_translations(&mut self, translations: Translations) {
        self.translations = Some(translations);
    }

    /// Get a translation for a term.
    pub fn translate(&self, term: &str) -> Option<&str> {
        self.translations
            .as_ref()
            .and_then(|t| t.terms.get(term))
            .map(|s| s.as_str())
    }

    /// Insert media into the media bag.
    pub fn insert_media(
        &mut self,
        path: String,
        mime_type: MimeType,
        contents: Vec<u8>,
    ) {
        self.media_bag.insert(path, mime_type, contents);
    }

    /// Get media from the media bag.
    pub fn get_media(&self, path: &str) -> Option<(&str, &[u8])> {
        self.media_bag
            .get(path)
            .map(|item| (item.mime_type(), item.contents()))
    }

    /// Set extension data.
    pub fn set_extension_data(&mut self, extension: &str, data: ExtensionData) {
        self.extensions_data.insert(extension.to_string(), data);
    }

    /// Get extension data.
    pub fn get_extension_data(&self, extension: &str) -> Option<&ExtensionData> {
        self.extensions_data.get(extension)
    }

    /// Get extension data mutably.
    pub fn get_extension_data_mut(
        &mut self,
        extension: &str,
    ) -> Option<&mut ExtensionData> {
        self.extensions_data.get_mut(extension)
    }

    /// Check if we should accept changes.
    pub fn accept_changes(&self) -> bool {
        matches!(self.track_changes, TrackChanges::AcceptChanges)
    }

    /// Check if we should reject changes.
    pub fn reject_changes(&self) -> bool {
        matches!(self.track_changes, TrackChanges::RejectChanges)
    }

    /// Check if we're tracking changes.
    pub fn track_changes(&self) -> bool {
        matches!(self.track_changes, TrackChanges::TrackChanges)
    }
}

impl Default for CommonState {
    fn default() -> Self {
        Self::new()
    }
}

impl Translations {
    /// Create new translations for a language.
    pub fn new(lang: impl Into<String>) -> Self {
        Self {
            lang: lang.into(),
            terms: HashMap::new(),
        }
    }

    /// Add a translation term.
    pub fn add_term(&mut self, term: impl Into<String>, translation: impl Into<String>) {
        self.terms.insert(term.into(), translation.into());
    }

    /// Get a translation.
    pub fn get(&self, term: &str) -> Option<&str> {
        self.terms.get(term).map(|s| s.as_str())
    }
}

impl ExtensionData {
    /// Create new extension data.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Set a value.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
    }

    /// Get a value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }
}

impl Default for ExtensionData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_state_default() {
        let state = CommonState::new();
        assert_eq!(state.verbosity, Verbosity::Warning);
        assert_eq!(state.tab_stop, 4);
        assert_eq!(state.columns, 80);
        assert!(state.log_messages.is_empty());
    }

    #[test]
    fn test_add_input_file() {
        let mut state = CommonState::new();
        state.add_input_file("test.md");
        assert_eq!(state.input_files.len(), 1);
        assert_eq!(state.input_files[0], PathBuf::from("test.md"));
    }

    #[test]
    fn test_logging() {
        let mut state = CommonState::new();
        state.log_debug("Debug message");
        state.log_info("Info message");
        state.log_warning("Warning message");
        state.log_error("Error message");

        assert_eq!(state.log_messages.len(), 4);
        assert_eq!(state.log_messages[0].level, LogLevel::Debug);
        assert_eq!(state.log_messages[3].level, LogLevel::Error);

        let warnings_and_errors = state.get_logs_at_level(LogLevel::Warning);
        assert_eq!(warnings_and_errors.len(), 2);
    }

    #[test]
    fn test_translations() {
        let mut translations = Translations::new("en-US");
        translations.add_term("Figure", "Figure");
        translations.add_term("Table", "Table");

        assert_eq!(translations.get("Figure"), Some("Figure"));
        assert_eq!(translations.get("Table"), Some("Table"));
        assert_eq!(translations.get("Unknown"), None);
    }

    #[test]
    fn test_extension_data() {
        let mut state = CommonState::new();
        let mut data = ExtensionData::new();
        data.set("key1", "value1");
        data.set("key2", "value2");

        state.set_extension_data("test_ext", data);

        let retrieved = state.get_extension_data("test_ext").unwrap();
        assert_eq!(retrieved.get("key1"), Some("value1"));
        assert_eq!(retrieved.get("key2"), Some("value2"));
    }

    #[test]
    fn test_track_changes() {
        let mut state = CommonState::new();
        assert!(state.track_changes());
        assert!(!state.accept_changes());
        assert!(!state.reject_changes());

        state.track_changes = TrackChanges::AcceptChanges;
        assert!(!state.track_changes());
        assert!(state.accept_changes());
        assert!(!state.reject_changes());
    }

    #[test]
    fn test_log_message_builder() {
        let msg = LogMessage::new(LogLevel::Info, "Test")
            .with_source("test.md")
            .with_position(10, 5);

        assert_eq!(msg.level, LogLevel::Info);
        assert_eq!(msg.message, "Test");
        assert_eq!(msg.source, Some("test.md".to_string()));
        assert_eq!(msg.line, Some(10));
        assert_eq!(msg.column, Some(5));
    }
}
