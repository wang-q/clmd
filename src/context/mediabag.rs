//! Resource management system for clmd.
//!
//! This module provides a unified way to manage binary resources like images,
//! inspired by Pandoc's MediaBag.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::io::format::mime::get_mime_type_def;

/// A media item stored in the MediaBag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaItem {
    /// The MIME type of the resource.
    mime_type: String,
    /// The original path of the resource.
    path: PathBuf,
    /// The contents of the resource.
    contents: Vec<u8>,
}

impl MediaItem {
    /// Create a new media item.
    pub fn new<P: AsRef<Path>>(
        path: P,
        mime_type: impl Into<String>,
        contents: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            mime_type: mime_type.into(),
            contents: contents.into(),
        }
    }

    /// Get the MIME type.
    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }

    /// Get the original path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the contents.
    pub fn contents(&self) -> &[u8] {
        &self.contents
    }
}

impl fmt::Display for MediaItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({} bytes, {})",
            self.path.display(),
            self.contents.len(),
            self.mime_type
        )
    }
}

/// A container for managing binary resources.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaBag {
    /// Map from canonical path to media item.
    items: HashMap<String, MediaItem>,
}

impl MediaBag {
    /// Create a new empty MediaBag.
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// Insert a media item into the bag.
    pub fn insert<P: AsRef<Path>>(
        &mut self,
        path: P,
        mime_type: impl Into<String>,
        contents: impl Into<Vec<u8>>,
    ) -> Option<MediaItem> {
        let canonical = canonicalize_path(path.as_ref());
        let item = MediaItem::new(path, mime_type, contents);
        self.items.insert(canonical, item)
    }

    /// Insert a media item with optional MIME type into the bag.
    ///
    /// If mime_type is None, it will be automatically detected from the file extension.
    pub fn insert_opt<P: AsRef<Path>>(
        &mut self,
        path: P,
        mime_type: Option<&str>,
        contents: impl Into<Vec<u8>>,
    ) -> Option<MediaItem> {
        let mime = mime_type
            .map(|m| m.to_string())
            .unwrap_or_else(|| mime_type_from_path(path.as_ref()));
        self.insert(path, mime, contents)
    }

    /// Look up a media item by path.
    pub fn lookup<P: AsRef<Path>>(&self, path: P) -> Option<&MediaItem> {
        let canonical = canonicalize_path(path.as_ref());
        self.items.get(&canonical)
    }
}

impl Extend<(String, MediaItem)> for MediaBag {
    fn extend<T: IntoIterator<Item = (String, MediaItem)>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

impl fmt::Display for MediaBag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_size: usize =
            self.items.values().map(|item| item.contents().len()).sum();
        writeln!(
            f,
            "MediaBag ({} items, {} bytes):",
            self.items.len(),
            total_size
        )?;
        for (path, item) in &self.items {
            writeln!(
                f,
                "  {}: {} ({} bytes)",
                path,
                item.mime_type(),
                item.contents().len()
            )?;
        }
        Ok(())
    }
}

/// Canonicalize a path for use as a key in the MediaBag.
fn canonicalize_path(path: &Path) -> String {
    let mut result = String::new();

    for component in path.components() {
        use std::path::Component;
        match component {
            Component::Normal(name) => {
                if !result.is_empty() {
                    result.push('/');
                }
                result.push_str(&name.to_string_lossy());
            }
            Component::ParentDir => {
                if let Some(pos) = result.rfind('/') {
                    result.truncate(pos);
                } else {
                    result.clear();
                }
            }
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
        }
    }

    if result.is_empty() {
        path.to_string_lossy().into_owned()
    } else {
        result
    }
}

/// Determine MIME type from file path based on extension.
pub fn mime_type_from_path<P: AsRef<Path>>(path: P) -> String {
    get_mime_type_def(path.as_ref()).to_string()
}
