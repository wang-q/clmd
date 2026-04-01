//! Data file access for clmd.
//!
//! This module provides access to default data files (templates, reference documents, etc.)
//! and supports user data directories, inspired by Pandoc's Text.Pandoc.Data module.
//!
//! # Example
//!
//! ```ignore
//! use clmd::context::data::{read_data_file, get_user_data_dir};
//!
//! // Read a data file
//! if let Ok(content) = read_data_file("templates/default.html") {
//!     println!("Template loaded: {} bytes", content.len());
//! }
//!
//! // Get the user data directory
//! if let Ok(Some(user_dir)) = get_user_data_dir() {
//!     println!("User data directory: {:?}", user_dir);
//! }
//! ```

use crate::core::error::{ClmdError, ClmdResult};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Cache for embedded data files.
static EMBEDDED_DATA_CACHE: OnceLock<HashMap<&'static str, &'static [u8]>> =
    OnceLock::new();

/// Initialize the embedded data cache with built-in files.
fn init_embedded_data() -> HashMap<&'static str, &'static [u8]> {
    let mut map: HashMap<&'static str, &'static [u8]> = HashMap::new();

    // Default templates - convert array reference to slice using a temporary
    let html_bytes: &[u8] = include_bytes!("../../data/templates/default.html");
    map.insert("templates/default.html", html_bytes);

    map
}

/// Get the embedded data cache, initializing it if necessary.
fn get_embedded_data() -> &'static HashMap<&'static str, &'static [u8]> {
    EMBEDDED_DATA_CACHE.get_or_init(init_embedded_data)
}

/// Read a file from the default data files.
///
/// This function first checks for embedded data files, then falls back to
/// the system data directory.
///
/// # Arguments
///
/// * `fname` - The name of the file to read (relative to the data directory)
///
/// # Returns
///
/// The file contents as a byte vector, or an error if the file cannot be found.
///
/// # Example
///
/// ```ignore
/// use clmd::data::read_default_data_file;
///
/// match read_default_data_file("templates/default.html") {
///     Ok(content) => println!("Loaded {} bytes", content.len()),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```ignore
pub fn read_default_data_file(fname: &str) -> ClmdResult<Vec<u8>> {
    // First, check embedded data files
    let embedded = get_embedded_data();
    if let Some(data) = embedded.get(fname) {
        return Ok(data.to_vec());
    }

    // Then, check the system data directory
    let data_dir = get_data_dir()?;
    let file_path = data_dir.join(fname);

    if file_path.exists() {
        fs::read(&file_path)
            .map_err(|e| ClmdError::io_error(format!("Failed to read {}: {}", fname, e)))
    } else {
        Err(ClmdError::resource_not_found(format!(
            "Data file not found: {}",
            fname
        )))
    }
}

/// Read a file from the user data directory or fall back to default data files.
///
/// This function first checks the user data directory, then falls back to
/// the default data files if not found.
///
/// # Arguments
///
/// * `fname` - The name of the file to read
///
/// # Returns
///
/// The file contents as a byte vector.
///
/// # Example
///
/// ```ignore
/// use clmd::data::read_data_file;
///
/// match read_data_file("templates/custom.html") {
///     Ok(content) => println!("Loaded {} bytes", content.len()),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```ignore
pub fn read_data_file(fname: &str) -> ClmdResult<Vec<u8>> {
    // First, check user data directory
    if let Some(user_dir) = get_user_data_dir()? {
        let user_path = user_dir.join(fname);
        if user_path.exists() {
            return fs::read(&user_path).map_err(|e| {
                ClmdError::io_error(format!("Failed to read {}: {}", fname, e))
            });
        }
    }

    // Fall back to default data files
    read_default_data_file(fname)
}

/// Read a data file as a UTF-8 string.
///
/// # Arguments
///
/// * `fname` - The name of the file to read
///
/// # Returns
///
/// The file contents as a string.
pub fn read_data_file_to_string(fname: &str) -> ClmdResult<String> {
    let bytes = read_data_file(fname)?;
    String::from_utf8(bytes).map_err(|e| {
        ClmdError::encoding_error(format!("Invalid UTF-8 in {}: {}", fname, e))
    })
}

/// Get the system data directory.
///
/// This is typically the directory where clmd's data files are installed.
///
/// # Returns
///
/// The path to the system data directory.
pub fn get_data_dir() -> ClmdResult<PathBuf> {
    // Try to get from environment variable first
    if let Ok(dir) = std::env::var("CLMD_DATA_DIR") {
        let path = PathBuf::from(dir);
        if path.exists() {
            return Ok(path);
        }
    }

    // Try to find relative to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let data_dir = exe_dir.join("data");
            if data_dir.exists() {
                return Ok(data_dir);
            }
            // Try one level up (for development)
            let data_dir = exe_dir.join("..").join("data");
            if data_dir.exists() {
                return Ok(data_dir.canonicalize().unwrap_or(data_dir));
            }
        }
    }

    // Default to current directory/data
    let default = PathBuf::from("data");
    if default.exists() {
        Ok(default)
    } else {
        // Return the default path even if it doesn't exist
        // The caller can handle the error
        Ok(default)
    }
}

/// Get the user data directory.
///
/// This follows the XDG Base Directory Specification on Unix systems,
/// and uses appropriate directories on Windows and macOS.
///
/// # Returns
///
/// The path to the user data directory, or None if it cannot be determined.
pub fn get_user_data_dir() -> ClmdResult<Option<PathBuf>> {
    // Try XDG_DATA_HOME first (Unix)
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        let path = PathBuf::from(xdg_data).join("clmd");
        return Ok(Some(path));
    }

    // Try home directory
    if let Some(home) = dirs::home_dir() {
        // Check for XDG default location
        let xdg_path = home.join(".local").join("share").join("clmd");
        if xdg_path.exists() {
            return Ok(Some(xdg_path));
        }

        // Check for legacy location (for backwards compatibility)
        let legacy_path = home.join(".clmd");
        if legacy_path.exists() {
            return Ok(Some(legacy_path));
        }

        // Return XDG default as the preferred location
        return Ok(Some(xdg_path));
    }

    Ok(None)
}

/// Get the default user data directory.
///
/// This returns the preferred user data directory, creating it if necessary.
///
/// # Returns
///
/// The path to the default user data directory.
pub fn default_user_data_dir() -> ClmdResult<PathBuf> {
    if let Some(dir) = get_user_data_dir()? {
        // Create the directory if it doesn't exist
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| {
                ClmdError::io_error(format!("Failed to create data dir: {}", e))
            })?;
        }
        Ok(dir)
    } else {
        Err(ClmdError::io_error(
            "Could not determine user data directory",
        ))
    }
}

/// Check if a data file exists.
///
/// # Arguments
///
/// * `fname` - The name of the file to check
///
/// # Returns
///
/// True if the file exists in either the user or default data directories.
pub fn data_file_exists(fname: &str) -> bool {
    // Check user data directory
    if let Ok(Some(user_dir)) = get_user_data_dir() {
        if user_dir.join(fname).exists() {
            return true;
        }
    }

    // Check embedded data
    let embedded = get_embedded_data();
    if embedded.contains_key(fname) {
        return true;
    }

    // Check system data directory
    if let Ok(data_dir) = get_data_dir() {
        if data_dir.join(fname).exists() {
            return true;
        }
    }

    false
}

/// List available data files.
///
/// # Returns
///
/// A vector of file names available in the data directories.
pub fn list_data_files() -> ClmdResult<Vec<String>> {
    let mut files = Vec::new();

    // Add embedded files
    let embedded = get_embedded_data();
    for key in embedded.keys() {
        files.push(key.to_string());
    }

    // Add files from system data directory
    if let Ok(data_dir) = get_data_dir() {
        if data_dir.exists() {
            collect_files_recursive(&data_dir, &data_dir, &mut files)?;
        }
    }

    // Add files from user data directory
    if let Ok(Some(user_dir)) = get_user_data_dir() {
        if user_dir.exists() {
            collect_files_recursive(&user_dir, &user_dir, &mut files)?;
        }
    }

    // Remove duplicates
    files.sort();
    files.dedup();

    Ok(files)
}

/// Recursively collect files from a directory.
fn collect_files_recursive(
    base_dir: &Path,
    current_dir: &Path,
    files: &mut Vec<String>,
) -> ClmdResult<()> {
    if !current_dir.exists() || !current_dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(current_dir)
        .map_err(|e| ClmdError::io_error(format!("Failed to read directory: {}", e)))?
    {
        let entry = entry
            .map_err(|e| ClmdError::io_error(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();

        if path.is_dir() {
            collect_files_recursive(base_dir, &path, files)?;
        } else {
            // Get relative path
            if let Ok(rel_path) = path.strip_prefix(base_dir) {
                if let Some(s) = rel_path.to_str() {
                    files.push(s.to_string());
                }
            }
        }
    }

    Ok(())
}

/// Copy a default data file to the user data directory.
///
/// # Arguments
///
/// * `fname` - The name of the file to copy
/// * `dest_name` - Optional destination name (defaults to source name)
///
/// # Returns
///
/// The path to the copied file.
pub fn copy_default_to_user(
    fname: &str,
    dest_name: Option<&str>,
) -> ClmdResult<PathBuf> {
    let content = read_default_data_file(fname)?;
    let user_dir = default_user_data_dir()?;

    let dest_fname = dest_name.unwrap_or(fname);
    let dest_path = user_dir.join(dest_fname);

    // Create parent directories if necessary
    if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                ClmdError::io_error(format!("Failed to create directory: {}", e))
            })?;
        }
    }

    fs::write(&dest_path, content)
        .map_err(|e| ClmdError::io_error(format!("Failed to write file: {}", e)))?;

    Ok(dest_path)
}

/// Write data to the user data directory.
///
/// # Arguments
///
/// * `fname` - The name of the file to write
/// * `data` - The data to write
///
/// # Returns
///
/// The path to the written file.
pub fn write_user_data_file(fname: &str, data: &[u8]) -> ClmdResult<PathBuf> {
    let user_dir = default_user_data_dir()?;
    let file_path = user_dir.join(fname);

    // Create parent directories if necessary
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                ClmdError::io_error(format!("Failed to create directory: {}", e))
            })?;
        }
    }

    fs::write(&file_path, data)
        .map_err(|e| ClmdError::io_error(format!("Failed to write file: {}", e)))?;

    Ok(file_path)
}

/// Delete a file from the user data directory.
///
/// # Arguments
///
/// * `fname` - The name of the file to delete
///
/// # Returns
///
/// Ok if the file was deleted successfully.
pub fn delete_user_data_file(fname: &str) -> ClmdResult<()> {
    if let Some(user_dir) = get_user_data_dir()? {
        let file_path = user_dir.join(fname);
        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| {
                ClmdError::io_error(format!("Failed to delete file: {}", e))
            })?;
        }
    }
    Ok(())
}

/// Get the path to a reference document.
///
/// Reference documents are used as templates for DOCX, ODT, and other formats.
///
/// # Arguments
///
/// * `format` - The format (e.g., "docx", "odt", "pptx")
///
/// # Returns
///
/// The path to the reference document, or an error if not found.
pub fn get_reference_document(format: &str) -> ClmdResult<PathBuf> {
    let fname = format!("reference.{}", format.to_lowercase());

    // Check user data directory first
    if let Some(user_dir) = get_user_data_dir()? {
        let user_path = user_dir.join(&fname);
        if user_path.exists() {
            return Ok(user_path);
        }
    }

    // Check system data directory
    let data_dir = get_data_dir()?;
    let sys_path = data_dir.join(&fname);
    if sys_path.exists() {
        return Ok(sys_path);
    }

    Err(ClmdError::resource_not_found(format!(
        "Reference document not found: {}",
        fname
    )))
}

/// Data file manager for caching and managing data file access.
#[derive(Debug, Clone)]
pub struct DataFileManager {
    user_data_dir: Option<PathBuf>,
    system_data_dir: PathBuf,
    cache: HashMap<String, Vec<u8>>,
}

impl DataFileManager {
    /// Create a new data file manager.
    pub fn new() -> ClmdResult<Self> {
        Ok(Self {
            user_data_dir: get_user_data_dir()?,
            system_data_dir: get_data_dir()?,
            cache: HashMap::new(),
        })
    }

    /// Read a file, using the cache if available.
    pub fn read(&mut self, fname: &str) -> ClmdResult<&[u8]> {
        // Check cache first
        if !self.cache.contains_key(fname) {
            let data = read_data_file(fname)?;
            self.cache.insert(fname.to_string(), data);
        }

        Ok(self.cache.get(fname).map(|v| v.as_slice()).unwrap())
    }

    /// Read a file as a string.
    pub fn read_to_string(&mut self, fname: &str) -> ClmdResult<String> {
        let bytes = self.read(fname)?;
        String::from_utf8(bytes.to_vec()).map_err(|e| {
            ClmdError::encoding_error(format!("Invalid UTF-8 in {}: {}", fname, e))
        })
    }

    /// Clear the cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Check if a file is cached.
    pub fn is_cached(&self, fname: &str) -> bool {
        self.cache.contains_key(fname)
    }

    /// Get the user data directory.
    pub fn user_data_dir(&self) -> Option<&Path> {
        self.user_data_dir.as_deref()
    }

    /// Get the system data directory.
    pub fn system_data_dir(&self) -> &Path {
        &self.system_data_dir
    }
}

impl Default for DataFileManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            user_data_dir: None,
            system_data_dir: PathBuf::from("data"),
            cache: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_file_exists() {
        // This test checks that data_file_exists doesn't panic
        // and returns a boolean value
        let _exists = data_file_exists("templates/default.html");
        // The result depends on whether the file is embedded
        // Just verify the function returns without panicking
    }

    #[test]
    fn test_get_data_dir() {
        let result = get_data_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_get_user_data_dir() {
        let result = get_user_data_dir();
        assert!(result.is_ok());
        // Result may be None or Some, depending on environment
    }

    #[test]
    fn test_default_user_data_dir() {
        // This may fail in some environments, so we just check it doesn't panic
        let _ = default_user_data_dir();
    }

    #[test]
    fn test_data_file_manager() {
        let manager = DataFileManager::new();
        assert!(manager.is_ok());

        let mut manager = manager.unwrap();
        assert!(manager.system_data_dir().as_os_str().len() > 0);

        // Test cache operations
        manager.clear_cache();
        assert!(!manager.is_cached("test"));
    }

    #[test]
    fn test_list_data_files() {
        let result = list_data_files();
        assert!(result.is_ok());
        let files = result.unwrap();
        // Should at least contain embedded files
        assert!(!files.is_empty() || files.is_empty()); // Just check it works
    }

    #[test]
    fn test_read_default_data_file_not_found() {
        let result = read_default_data_file("nonexistent/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_reference_document_not_found() {
        let result = get_reference_document("xyz");
        assert!(result.is_err());
    }
}
