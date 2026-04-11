//! IO Context implementation for real file operations.
//!
//! This module provides the [`IoContext`] struct, which implements
//! the [`ClmdContext`] trait for real file system operations.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::context::mediabag::MediaItem;
use crate::context::{
    common, ClmdContext, CommonState, LogLevel, LogMessage, Verbosity,
};
use crate::core::error::ClmdError;

/// IO Context for real file operations.
#[derive(Debug, Clone)]
pub struct IoContext {
    /// The common state for this context.
    state: CommonState,
}

impl IoContext {
    /// Create a new IO context with default settings.
    pub fn new() -> Self {
        Self {
            state: CommonState {
                user_data_dir: common::default_user_data_dir(),
                ..Default::default()
            },
        }
    }
}

impl Default for IoContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ClmdContext for IoContext {
    type Error = ClmdError;

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Self::Error> {
        fs::read(path).map_err(|e| {
            ClmdError::io_error(format!("Failed to read {}: {}", path.display(), e))
        })
    }

    fn write_file(&self, path: &Path, content: &[u8]) -> Result<(), Self::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ClmdError::io_error(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        fs::write(path, content).map_err(|e| {
            ClmdError::io_error(format!("Failed to write {}: {}", path.display(), e))
        })
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn get_modification_time(&self, path: &Path) -> Result<SystemTime, Self::Error> {
        fs::metadata(path)
            .map_err(|e| {
                ClmdError::io_error(format!(
                    "Failed to get metadata for {}: {}",
                    path.display(),
                    e
                ))
            })?
            .modified()
            .map_err(|e| {
                ClmdError::io_error(format!(
                    "Failed to get modification time for {}: {}",
                    path.display(),
                    e
                ))
            })
    }

    fn find_file(&self, filename: &str) -> Option<PathBuf> {
        self.state.find_file(filename)
    }

    fn report(&self, level: LogLevel, message: String) {
        self.state.log(level, message.clone());

        if level.should_log(self.state.verbosity.as_u8()) {
            match level {
                LogLevel::Error => eprintln!("[ERROR] {}", message),
                LogLevel::Warning => eprintln!("[WARNING] {}", message),
                LogLevel::Info => println!("[INFO] {}", message),
                LogLevel::Debug => println!("[DEBUG] {}", message),
            }
        }
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
        SystemTime::now()
    }

    fn get_random_bytes(&self, len: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut bytes = vec![0u8; len];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }
}
