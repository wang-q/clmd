//! Common state management for clmd.
//!
//! This module provides a unified state container for clmd operations,
//! inspired by Pandoc's CommonState. It holds shared state that is
//! accessible throughout the document processing pipeline.

use std::path::PathBuf;

use crate::context::MediaBag;

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

    /// Source URL for remote resources.
    pub source_url: Option<String>,

    /// Media bag for storing binary resources.
    pub media_bag: MediaBag,

    /// Whether to trace operations.
    pub trace: bool,

    /// Whether to check SSL certificates.
    pub no_check_certificate: bool,

    /// Current timestamp.
    pub timestamp: std::time::SystemTime,

    /// Abbreviations for smart punctuation.
    pub abbreviations: Vec<String>,

    /// Default image extension.
    pub default_image_extension: String,

    /// Tab stop width.
    pub tab_stop: usize,

    /// Column width for wrapping.
    pub columns: usize,
}

impl CommonState {
    /// Create a new CommonState with default values.
    pub fn new() -> Self {
        Self {
            verbosity: Verbosity::Warning,
            resource_path: vec![PathBuf::from(".")],
            user_data_dir: dirs::data_dir().map(|d| d.join("clmd")),
            source_url: None,
            media_bag: MediaBag::new(),
            trace: false,
            no_check_certificate: false,
            timestamp: std::time::SystemTime::now(),
            abbreviations: Vec::new(),
            default_image_extension: String::new(),
            tab_stop: 4,
            columns: 80,
        }
    }
}

impl Default for CommonState {
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
    }

    #[test]
    fn test_common_state_default_trait() {
        let state: CommonState = Default::default();
        assert_eq!(state.verbosity, Verbosity::Warning);
        assert_eq!(state.tab_stop, 4);
    }
}
