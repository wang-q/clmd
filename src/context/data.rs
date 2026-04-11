//! Data file access for clmd.
//!
//! This module provides access to default data files (templates, reference documents, etc.)
//! and supports user data directories, inspired by Pandoc's Text.Pandoc.Data module.

use crate::core::error::{ClmdError, ClmdResult};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

/// Cache for embedded data files.
static EMBEDDED_DATA_CACHE: OnceLock<HashMap<&'static str, &'static [u8]>> =
    OnceLock::new();

/// Initialize the embedded data cache with built-in files.
fn init_embedded_data() -> HashMap<&'static str, &'static [u8]> {
    let mut map: HashMap<&'static str, &'static [u8]> = HashMap::new();

    let html_bytes: &[u8] = include_bytes!("../../data/templates/default.html");
    map.insert("templates/default.html", html_bytes);

    map
}

/// Get the embedded data cache, initializing it if necessary.
fn get_embedded_data() -> &'static HashMap<&'static str, &'static [u8]> {
    EMBEDDED_DATA_CACHE.get_or_init(init_embedded_data)
}

/// Read a file from the default data files.
pub fn read_default_data_file(fname: &str) -> ClmdResult<Vec<u8>> {
    let embedded = get_embedded_data();
    if let Some(data) = embedded.get(fname) {
        return Ok(data.to_vec());
    }

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
pub fn read_data_file(fname: &str) -> ClmdResult<Vec<u8>> {
    if let Some(user_dir) = get_user_data_dir()? {
        let user_path = user_dir.join(fname);
        if user_path.exists() {
            return fs::read(&user_path).map_err(|e| {
                ClmdError::io_error(format!("Failed to read {}: {}", fname, e))
            });
        }
    }

    read_default_data_file(fname)
}

/// Get the system data directory.
pub fn get_data_dir() -> ClmdResult<PathBuf> {
    if let Ok(dir) = std::env::var("CLMD_DATA_DIR") {
        let path = PathBuf::from(dir);
        if path.exists() {
            return Ok(path);
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let data_dir = exe_dir.join("data");
            if data_dir.exists() {
                return Ok(data_dir);
            }
            let data_dir = exe_dir.join("..").join("data");
            if data_dir.exists() {
                return Ok(data_dir.canonicalize().unwrap_or(data_dir));
            }
        }
    }

    let default = PathBuf::from("data");
    Ok(default)
}

/// Get the user data directory.
pub fn get_user_data_dir() -> ClmdResult<Option<PathBuf>> {
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        let path = PathBuf::from(xdg_data).join("clmd");
        return Ok(Some(path));
    }

    if let Some(home) = dirs::home_dir() {
        let xdg_path = home.join(".local").join("share").join("clmd");
        return Ok(Some(xdg_path));
    }

    Ok(None)
}
