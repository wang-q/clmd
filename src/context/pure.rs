//! Pure Context implementation for testing.
//!
//! This module provides the [`PureContext`] struct, which implements
//! the [`ClmdContext`] trait using in-memory storage.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::context::mediabag::MediaItem;
use crate::context::{
    common, ClmdContext, CommonState, LogLevel, LogMessage, Verbosity,
};
use crate::core::error::ClmdError;

/// Pure Context for testing and pure functional code.
///
/// This context stores files in memory and performs no actual IO operations.
#[derive(Debug, Clone)]
pub struct PureContext {
    /// The common state for this context.
    state: CommonState,
    /// In-memory file storage.
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    /// Simulated current time for testing.
    current_time: Arc<Mutex<SystemTime>>,
    /// Simulated random bytes for testing.
    random_bytes: Arc<Mutex<Vec<u8>>>,
}

impl PureContext {
    /// Create a new pure context with default settings.
    pub fn new() -> Self {
        Self {
            state: CommonState::default(),
            files: Arc::new(Mutex::new(HashMap::new())),
            current_time: Arc::new(Mutex::new(SystemTime::UNIX_EPOCH)),
            random_bytes: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for PureContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ClmdContext for PureContext {
    type Error = ClmdError;

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Self::Error> {
        let canonical = common::canonicalize_path(path);
        let files = self.files.lock().unwrap();
        files.get(&canonical).cloned().ok_or_else(|| {
            ClmdError::io_error(format!("File not found: {}", path.display()))
        })
    }

    fn write_file(&self, path: &Path, content: &[u8]) -> Result<(), Self::Error> {
        let canonical = common::canonicalize_path(path);
        let mut files = self.files.lock().unwrap();
        files.insert(canonical, content.to_vec());
        Ok(())
    }

    fn file_exists(&self, path: &Path) -> bool {
        let canonical = common::canonicalize_path(path);
        let files = self.files.lock().unwrap();
        files.contains_key(&canonical)
    }

    fn get_modification_time(&self, path: &Path) -> Result<SystemTime, Self::Error> {
        if self.file_exists(path) {
            let time = *self.current_time.lock().unwrap();
            Ok(time)
        } else {
            Err(ClmdError::io_error(format!(
                "File not found: {}",
                path.display()
            )))
        }
    }

    fn find_file(&self, filename: &str) -> Option<PathBuf> {
        self.state.find_file(filename)
    }

    fn report(&self, level: LogLevel, message: String) {
        self.state.log(level, message);
    }

    fn get_logs(&self) -> Vec<LogMessage> {
        self.state.get_logs()
    }

    fn get_verbosity(&self) -> Verbosity {
        self.state.verbosity
    }

    fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.state.verbosity = verbosity;
    }

    fn get_state(&self) -> &CommonState {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut CommonState {
        &mut self.state
    }

    fn insert_media(
        &self,
        path: &Path,
        mime_type: Option<&str>,
        data: Vec<u8>,
    ) -> Result<String, Self::Error> {
        let mut bag = self.state.media_bag.lock().unwrap();

        let path_str = path.to_string_lossy();
        if common::is_data_uri(&path_str) {
            let new_path = common::generate_hash_path(&data, mime_type);
            bag.insert_opt(PathBuf::from(&new_path), mime_type, data);
            return Ok(new_path);
        }

        let canonical = common::canonicalize_path(path);
        bag.insert_opt(path, mime_type, data);
        Ok(canonical)
    }

    fn lookup_media(&self, path: &Path) -> Option<MediaItem> {
        let bag = self.state.media_bag.lock().unwrap();
        bag.lookup(path).cloned()
    }

    fn get_current_time(&self) -> SystemTime {
        *self.current_time.lock().unwrap()
    }

    fn get_random_bytes(&self, len: usize) -> Vec<u8> {
        let random_bytes = self.random_bytes.lock().unwrap();
        if random_bytes.is_empty() {
            (0..len).map(|i| (i % 256) as u8).collect()
        } else {
            (0..len)
                .map(|i| random_bytes[i % random_bytes.len()])
                .collect()
        }
    }
}
