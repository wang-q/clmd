//! Resource management system for clmd.
//!
//! This module provides a unified way to manage binary resources like images,
//! inspired by Pandoc's MediaBag. Resources are stored with their MIME type
//! and can be looked up by their path.
//!
//! # Example
//!
//! ```ignore
//! use clmd::mediabag::{MediaBag, MediaItem};
//!
//! let mut bag = MediaBag::new();
//!
//! // Insert a resource
//! bag.insert("image.png", "image/png", vec![0x89, 0x50, 0x4E, 0x47]);
//!
//! // Look up a resource
//! if let Some(item) = bag.lookup("image.png") {
//!     assert_eq!(item.mime_type(), "image/png");
//! }
//! ```

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::io::format::mime::{self, get_mime_type_def};

/// A media item stored in the MediaBag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaItem {
    /// The MIME type of the resource.
    mime_type: String,
    /// The original path of the resource.
    path: PathBuf,
    /// The contents of the resource.
    contents: Vec<u8>,
    /// SHA-256 hash of the contents (lazy computed).
    hash: Option<String>,
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
            hash: None,
        }
    }

    /// Get or compute the SHA-256 hash of the contents.
    ///
    /// The hash is computed lazily and cached for subsequent calls.
    pub fn hash(&mut self) -> &str {
        if self.hash.is_none() {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&self.contents);
            let result = hasher.finalize();
            self.hash = Some(format!("{:x}", result));
        }
        self.hash.as_ref().unwrap()
    }

    /// Get the hash if already computed.
    pub fn get_hash(&self) -> Option<&str> {
        self.hash.as_deref()
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

    /// Get the size of the contents in bytes.
    pub fn size(&self) -> usize {
        self.contents.len()
    }

    /// Convert contents to a string if valid UTF-8.
    pub fn contents_to_string(&self) -> Option<String> {
        String::from_utf8(self.contents.clone()).ok()
    }

    /// Get the file extension from the path.
    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|e| e.to_str())
    }

    /// Get the file name from the path.
    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name().and_then(|n| n.to_str())
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
///
/// MediaBag stores resources like images, stylesheets, etc. that are
/// referenced by documents. It provides efficient lookup by path and
/// tracks MIME types for proper handling.
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

    /// Create a new MediaBag with a specific capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: HashMap::with_capacity(capacity),
        }
    }

    /// Insert a media item into the bag.
    ///
    /// If an item with the same canonical path already exists, it will be replaced.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::mediabag::MediaBag;
    ///
    /// let mut bag = MediaBag::new();
    /// bag.insert("image.png", "image/png", vec![0x89, 0x50, 0x4E, 0x47]);
    /// ```
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
    /// If an item with the same canonical path already exists, it will be replaced.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::mediabag::MediaBag;
    ///
    /// let mut bag = MediaBag::new();
    /// bag.insert_opt("image.png", Some("image/png"), vec![0x89, 0x50, 0x4E, 0x47]);
    /// bag.insert_opt("style.css", None::<&str>, vec![]); // Auto-detect MIME type
    /// ```
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

    /// Insert a media item with automatic MIME type detection.
    ///
    /// The MIME type is determined from the file extension.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::mediabag::MediaBag;
    ///
    /// let mut bag = MediaBag::new();
    /// bag.insert_auto("image.png", vec![0x89, 0x50, 0x4E, 0x47]);
    /// ```
    pub fn insert_auto<P: AsRef<Path>>(
        &mut self,
        path: P,
        contents: impl Into<Vec<u8>>,
    ) -> Option<MediaItem> {
        let mime_type = mime_type_from_path(&path);
        self.insert(path, mime_type, contents)
    }

    /// Look up a media item by path.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::mediabag::MediaBag;
    ///
    /// let mut bag = MediaBag::new();
    /// bag.insert("image.png", "image/png", vec![]);
    ///
    /// assert!(bag.lookup("image.png").is_some());
    /// assert!(bag.lookup("nonexistent.png").is_none());
    /// ```
    pub fn lookup<P: AsRef<Path>>(&self, path: P) -> Option<&MediaItem> {
        let canonical = canonicalize_path(path.as_ref());
        self.items.get(&canonical)
    }

    /// Get a media item by path (alias for lookup).
    ///
    /// This is a convenience method that provides the same functionality
    /// as `lookup` with a more idiomatic name.
    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<&MediaItem> {
        self.lookup(path)
    }

    /// Look up a media item mutably by path.
    pub fn lookup_mut<P: AsRef<Path>>(&mut self, path: P) -> Option<&mut MediaItem> {
        let canonical = canonicalize_path(path.as_ref());
        self.items.get_mut(&canonical)
    }

    /// Check if a path exists in the bag.
    pub fn contains<P: AsRef<Path>>(&self, path: P) -> bool {
        let canonical = canonicalize_path(path.as_ref());
        self.items.contains_key(&canonical)
    }

    /// Remove a media item by path.
    ///
    /// Returns the removed item if it existed.
    pub fn remove<P: AsRef<Path>>(&mut self, path: P) -> Option<MediaItem> {
        let canonical = canonicalize_path(path.as_ref());
        self.items.remove(&canonical)
    }

    /// Get the number of items in the bag.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the bag is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get all items in the bag.
    pub fn items(&self) -> &HashMap<String, MediaItem> {
        &self.items
    }

    /// Get an iterator over all items.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &MediaItem)> {
        self.items.iter()
    }

    /// Get all paths in the bag.
    pub fn paths(&self) -> Vec<&String> {
        self.items.keys().collect()
    }

    /// Get all items as a vector.
    pub fn to_vec(&self) -> Vec<&MediaItem> {
        self.items.values().collect()
    }

    /// Get the total size of all contents in bytes.
    pub fn total_size(&self) -> usize {
        self.items.values().map(|item| item.size()).sum()
    }

    /// Clear all items from the bag.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Merge another MediaBag into this one.
    ///
    /// Items from the other bag will overwrite items in this bag if they have
    /// the same canonical path.
    pub fn merge(&mut self, other: MediaBag) {
        self.items.extend(other.items);
    }

    /// Filter items by MIME type prefix.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::mediabag::MediaBag;
    ///
    /// let mut bag = MediaBag::new();
    /// bag.insert("image.png", "image/png", vec![]);
    /// bag.insert("image.jpg", "image/jpeg", vec![]);
    /// bag.insert("style.css", "text/css", vec![]);
    ///
    /// let images: Vec<_> = bag.filter_by_mime_type("image/").collect();
    /// assert_eq!(images.len(), 2);
    /// ```
    pub fn filter_by_mime_type<'a>(
        &'a self,
        prefix: &'a str,
    ) -> impl Iterator<Item = &'a MediaItem> + 'a {
        self.items
            .values()
            .filter(move |item| item.mime_type().starts_with(prefix))
    }

    /// Get items grouped by MIME type.
    pub fn group_by_mime_type(&self) -> HashMap<String, Vec<&MediaItem>> {
        let mut groups: HashMap<String, Vec<&MediaItem>> = HashMap::new();
        for item in self.items.values() {
            groups
                .entry(item.mime_type().to_string())
                .or_default()
                .push(item);
        }
        groups
    }

    /// Get a directory listing of all items.
    ///
    /// Returns a vector of (path, mime_type, size) tuples.
    pub fn directory(&self) -> Vec<(&Path, &str, usize)> {
        self.items
            .values()
            .map(|item| (item.path(), item.mime_type(), item.size()))
            .collect()
    }

    /// Load a file from the file system into the MediaBag.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to load
    /// * `media_path` - Optional path to use as the key in the MediaBag (defaults to file name)
    ///
    /// # Errors
    ///
    /// Returns an IO error if the file cannot be read.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::mediabag::MediaBag;
    ///
    /// let mut bag = MediaBag::new();
    /// bag.load_file("/path/to/image.png", None).unwrap();
    /// bag.load_file("/path/to/photo.jpg", Some("images/photo.jpg")).unwrap();
    /// ```
    pub fn load_file<P: AsRef<Path>, M: AsRef<Path>>(
        &mut self,
        path: P,
        media_path: Option<M>,
    ) -> std::io::Result<()> {
        let contents = std::fs::read(&path)?;
        let key_path =
            media_path
                .map(|p| p.as_ref().to_path_buf())
                .unwrap_or_else(|| {
                    path.as_ref()
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.as_ref().to_path_buf())
                });
        self.insert_auto(key_path, contents);
        Ok(())
    }

    /// Load all files from a directory into the MediaBag.
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory to load files from
    /// * `recursive` - Whether to recursively load subdirectories
    ///
    /// # Errors
    ///
    /// Returns an IO error if the directory cannot be read.
    pub fn load_directory<P: AsRef<Path>>(
        &mut self,
        dir: P,
        recursive: bool,
    ) -> std::io::Result<usize> {
        let mut count = 0;
        let dir = dir.as_ref();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let relative_path = path.strip_prefix(dir).unwrap_or(&path);
                if self.load_file(&path, Some(relative_path)).is_ok() {
                    count += 1;
                }
            } else if recursive && path.is_dir() {
                count += self.load_directory(&path, recursive)?;
            }
        }

        Ok(count)
    }

    /// Insert a resource from a data URI.
    ///
    /// Data URIs have the format: `data:[<mediatype>][;base64],<data>`
    ///
    /// # Arguments
    ///
    /// * `path` - Path to use as the key
    /// * `data_uri` - The data URI string
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the data URI was successfully parsed and inserted,
    /// `Ok(false)` if the string is not a valid data URI,
    /// `Err` if the base64 decoding fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::mediabag::MediaBag;
    ///
    /// let mut bag = MediaBag::new();
    /// let data_uri = "data:image/png;base64,iVBORw0KGgo=";
    /// bag.insert_data_uri("image.png", data_uri).unwrap();
    /// assert!(bag.contains("image.png"));
    /// ```
    pub fn insert_data_uri<P: AsRef<Path>>(
        &mut self,
        path: P,
        data_uri: &str,
    ) -> Result<bool, base64::DecodeError> {
        use crate::text::uri::parse_data_uri;

        if let Some((mime_type, data)) = parse_data_uri(data_uri) {
            let contents = if data_uri.contains(";base64,") {
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data)?
            } else {
                // URL-encoded data
                urlencoding::decode(data)
                    .map(|s| s.into_owned())
                    .unwrap_or_else(|_| data.to_string())
                    .into_bytes()
            };

            let final_mime: &str = if mime_type.is_empty() {
                "application/octet-stream"
            } else {
                mime_type
            };

            self.insert(path, final_mime, contents);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Convert a media item to a data URI.
    ///
    /// # Arguments
    ///
    /// * `path` - Path of the media item
    ///
    /// # Returns
    ///
    /// Some(data_uri) if the item exists, None otherwise.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::mediabag::MediaBag;
    ///
    /// let mut bag = MediaBag::new();
    /// bag.insert("image.png", "image/png", vec![0x89, 0x50, 0x4E, 0x47]);
    ///
    /// let data_uri = bag.to_data_uri("image.png");
    /// assert!(data_uri.is_some());
    /// assert!(data_uri.unwrap().starts_with("data:image/png;base64,"));
    /// ```
    pub fn to_data_uri<P: AsRef<Path>>(&self, path: P) -> Option<String> {
        self.lookup(&path).map(|item| {
            let base64_data = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                item.contents(),
            );
            format!("data:{};base64,{}", item.mime_type(), base64_data)
        })
    }

    /// Get all image items in the bag.
    pub fn images(&self) -> Vec<&MediaItem> {
        self.filter_by_mime_type("image/").collect()
    }

    /// Get all font items in the bag.
    pub fn fonts(&self) -> Vec<&MediaItem> {
        self.items
            .values()
            .filter(|item| {
                item.mime_type().starts_with("font/")
                    || item.mime_type().contains("font")
                    || item.mime_type() == "application/vnd.ms-fontobject"
            })
            .collect()
    }

    /// Get all stylesheet items in the bag.
    pub fn stylesheets(&self) -> Vec<&MediaItem> {
        self.filter_by_mime_type("text/css").collect()
    }

    /// Get all JavaScript items in the bag.
    pub fn scripts(&self) -> Vec<&MediaItem> {
        self.items
            .values()
            .filter(|item| {
                item.mime_type() == "application/javascript"
                    || item.mime_type() == "text/javascript"
            })
            .collect()
    }

    /// Check if all items have valid content (non-empty).
    pub fn all_valid(&self) -> bool {
        self.items.values().all(|item| !item.contents().is_empty())
    }

    /// Get items that are missing or have empty content.
    pub fn get_invalid_items(&self) -> Vec<&String> {
        self.items
            .iter()
            .filter(|(_, item)| item.contents().is_empty())
            .map(|(path, _)| path)
            .collect()
    }

    /// Serialize the MediaBag to a JSON representation.
    ///
    /// Note: Contents are base64-encoded.
    #[cfg(feature = "serde")]
    #[allow(dead_code)]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        use serde::Serialize;

        #[derive(Serialize)]
        struct MediaItemSer {
            path: String,
            mime_type: String,
            contents: String, // base64 encoded
        }

        let items: Vec<MediaItemSer> = self
            .items
            .values()
            .map(|item| MediaItemSer {
                path: item.path().to_string_lossy().to_string(),
                mime_type: item.mime_type().to_string(),
                contents: base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    item.contents(),
                ),
            })
            .collect();

        serde_json::to_string(&items)
    }

    /// Find duplicate items based on content hash.
    ///
    /// Returns a map from hash to list of paths that have identical content.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::mediabag::MediaBag;
    ///
    /// let mut bag = MediaBag::new();
    /// bag.insert("a.png", "image/png", vec![0x89, 0x50]);
    /// bag.insert("b.png", "image/png", vec![0x89, 0x50]); // Same content
    /// bag.insert("c.png", "image/png", vec![0x89, 0x51]); // Different content
    ///
    /// let duplicates = bag.find_duplicates();
    /// assert_eq!(duplicates.len(), 1); // One group of duplicates
    /// ```
    pub fn find_duplicates(&mut self) -> HashMap<String, Vec<String>> {
        use sha2::{Digest, Sha256};
        let mut hash_to_paths: HashMap<String, Vec<String>> = HashMap::new();

        for (path, item) in &self.items {
            let mut hasher = Sha256::new();
            hasher.update(item.contents());
            let hash = format!("{:x}", hasher.finalize());
            hash_to_paths.entry(hash).or_default().push(path.clone());
        }

        // Filter to only keep hashes with multiple paths
        hash_to_paths
            .into_iter()
            .filter(|(_, paths)| paths.len() > 1)
            .collect()
    }

    /// Deduplicate items by content hash.
    ///
    /// Keeps only the first occurrence of each unique content and returns
    /// the removed items.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::mediabag::MediaBag;
    ///
    /// let mut bag = MediaBag::new();
    /// bag.insert("a.png", "image/png", vec![0x89, 0x50]);
    /// bag.insert("b.png", "image/png", vec![0x89, 0x50]); // Duplicate
    /// bag.insert("c.png", "image/png", vec![0x89, 0x51]); // Unique
    ///
    /// let removed = bag.deduplicate();
    /// assert_eq!(bag.len(), 2);
    /// assert_eq!(removed.len(), 1);
    /// ```
    pub fn deduplicate(&mut self) -> Vec<MediaItem> {
        use sha2::{Digest, Sha256};
        let mut seen_hashes: HashMap<String, String> = HashMap::new(); // hash -> first path
        let mut to_remove = Vec::new();

        for (path, item) in &self.items {
            let mut hasher = Sha256::new();
            hasher.update(item.contents());
            let hash = format!("{:x}", hasher.finalize());

            if seen_hashes.contains_key(&hash) {
                // This is a duplicate
                to_remove.push(path.clone());
            } else {
                seen_hashes.insert(hash, path.clone());
            }
        }

        let mut removed = Vec::new();
        for path in to_remove {
            if let Some(item) = self.items.remove(&path) {
                removed.push(item);
            }
        }
        removed
    }

    /// Insert a media item with hash-based path generation for external URLs.
    ///
    /// When the path is an external URL or contains unsafe characters,
    /// generates a new path based on the content hash.
    ///
    /// # Arguments
    ///
    /// * `path` - The original path or URL
    /// * `mime_type` - The MIME type of the content
    /// * `contents` - The binary content
    ///
    /// # Returns
    ///
    /// The canonical path used to store the item (may be different from input).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::mediabag::MediaBag;
    ///
    /// let mut bag = MediaBag::new();
    /// let contents = vec![0x89u8, 0x50, 0x4E, 0x47];
    /// let stored_path = bag.insert_with_hash_path("https://example.com/image.png", "image/png", contents);
    /// // stored_path will be a hash-based name like "abc123.png"
    /// assert!(bag.contains(&stored_path));
    /// ```
    pub fn insert_with_hash_path<P: AsRef<Path>>(
        &mut self,
        path: P,
        mime_type: impl Into<String>,
        contents: impl Into<Vec<u8>>,
    ) -> String {
        use sha2::{Digest, Sha256};

        let path_ref = path.as_ref();
        let contents_vec: Vec<u8> = contents.into();
        let mime: String = mime_type.into();

        // Check if we need to use hash-based path
        let needs_hash_path = path_ref.to_string_lossy().starts_with("http://")
            || path_ref.to_string_lossy().starts_with("https://")
            || path_ref.to_string_lossy().starts_with("data:")
            || path_ref.to_string_lossy().contains("..")
            || path_ref.to_string_lossy().contains('%');

        let final_path = if needs_hash_path {
            // Generate hash-based path
            let mut hasher = Sha256::new();
            hasher.update(&contents_vec);
            let hash = format!("{:x}", hasher.finalize());
            let short_hash = &hash[..16]; // Use first 16 chars of hash

            // Get extension from MIME type or original path
            let ext = extension_from_mime_type(&mime)
                .or_else(|| path_ref.extension().and_then(|e| e.to_str()))
                .unwrap_or("bin");

            format!("{}.{}", short_hash, ext)
        } else {
            canonicalize_path(path_ref)
        };

        let item = MediaItem::new(&final_path, mime, contents_vec);
        self.items.insert(final_path.clone(), item);
        final_path
    }

    /// Get the canonical path for a given path.
    ///
    /// This is useful for normalizing paths before lookup.
    pub fn canonicalize<P: AsRef<Path>>(&self, path: P) -> String {
        canonicalize_path(path.as_ref())
    }

    /// Check if an item with the given content hash exists.
    ///
    /// # Arguments
    ///
    /// * `hash` - The SHA-256 hash to look for
    ///
    /// # Returns
    ///
    /// `Some(path)` if an item with this hash exists, `None` otherwise.
    pub fn find_by_hash(&self, hash: &str) -> Option<&str> {
        use sha2::{Digest, Sha256};

        for (path, item) in &self.items {
            let mut hasher = Sha256::new();
            hasher.update(item.contents());
            let item_hash = format!("{:x}", hasher.finalize());
            if item_hash == hash {
                return Some(path);
            }
        }
        None
    }
}

impl Extend<(String, MediaItem)> for MediaBag {
    fn extend<T: IntoIterator<Item = (String, MediaItem)>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

impl fmt::Display for MediaBag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "MediaBag ({} items, {} bytes):",
            self.len(),
            self.total_size()
        )?;
        for (path, item) in &self.items {
            writeln!(
                f,
                "  {}: {} ({} bytes)",
                path,
                item.mime_type(),
                item.size()
            )?;
        }
        Ok(())
    }
}

/// Canonicalize a path for use as a key in the MediaBag.
///
/// This normalizes the path by:
/// - Converting to forward slashes
/// - Removing leading slashes
/// - Removing `.` and `..` components
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
                // Remove last component if any
                if let Some(pos) = result.rfind('/') {
                    result.truncate(pos);
                } else {
                    result.clear();
                }
            }
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {
                // Skip these components
            }
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

/// Get file extension from MIME type.
pub fn extension_from_mime_type(mime_type: &str) -> Option<&'static str> {
    mime::extension_from_mime_type(mime_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_item() {
        let item =
            MediaItem::new("image.png", "image/png", vec![0x89, 0x50, 0x4E, 0x47]);
        assert_eq!(item.mime_type(), "image/png");
        assert_eq!(item.path().to_str(), Some("image.png"));
        assert_eq!(item.size(), 4);
        assert_eq!(item.extension(), Some("png"));
        assert_eq!(item.file_name(), Some("image.png"));
    }

    #[test]
    fn test_media_bag_basic() {
        let mut bag = MediaBag::new();
        assert!(bag.is_empty());

        bag.insert("image.png", "image/png", vec![0x89, 0x50]);
        assert_eq!(bag.len(), 1);
        assert!(!bag.is_empty());

        let item = bag.lookup("image.png").unwrap();
        assert_eq!(item.mime_type(), "image/png");
    }

    #[test]
    fn test_media_bag_replace() {
        let mut bag = MediaBag::new();
        bag.insert("image.png", "image/png", vec![0x89]);
        bag.insert("image.png", "image/jpeg", vec![0xFF]);

        let item = bag.lookup("image.png").unwrap();
        assert_eq!(item.mime_type(), "image/jpeg");
    }

    #[test]
    fn test_media_bag_remove() {
        let mut bag = MediaBag::new();
        bag.insert("image.png", "image/png", vec![]);
        assert!(bag.contains("image.png"));

        let removed = bag.remove("image.png");
        assert!(removed.is_some());
        assert!(!bag.contains("image.png"));
    }

    #[test]
    fn test_media_bag_auto_mime() {
        let mut bag = MediaBag::new();
        bag.insert_auto("image.png", vec![]);
        bag.insert_auto("style.css", vec![]);
        bag.insert_auto("script.js", vec![]);

        assert_eq!(bag.lookup("image.png").unwrap().mime_type(), "image/png");
        assert_eq!(bag.lookup("style.css").unwrap().mime_type(), "text/css");
        // JavaScript MIME type can be either text/javascript or application/javascript
        let js_mime = bag.lookup("script.js").unwrap().mime_type();
        assert!(js_mime == "text/javascript" || js_mime == "application/javascript");
    }

    #[test]
    fn test_media_bag_filter_by_mime_type() {
        let mut bag = MediaBag::new();
        bag.insert("a.png", "image/png", vec![]);
        bag.insert("b.jpg", "image/jpeg", vec![]);
        bag.insert("c.css", "text/css", vec![]);

        let images: Vec<_> = bag.filter_by_mime_type("image/").collect();
        assert_eq!(images.len(), 2);
    }

    #[test]
    fn test_media_bag_group_by_mime_type() {
        let mut bag = MediaBag::new();
        bag.insert("a.png", "image/png", vec![]);
        bag.insert("b.png", "image/png", vec![]);
        bag.insert("c.css", "text/css", vec![]);

        let groups = bag.group_by_mime_type();
        assert_eq!(groups.get("image/png").unwrap().len(), 2);
        assert_eq!(groups.get("text/css").unwrap().len(), 1);
    }

    #[test]
    fn test_media_bag_total_size() {
        let mut bag = MediaBag::new();
        bag.insert("a.png", "image/png", vec![0; 100]);
        bag.insert("b.png", "image/png", vec![0; 200]);

        assert_eq!(bag.total_size(), 300);
    }

    #[test]
    fn test_media_bag_merge() {
        let mut bag1 = MediaBag::new();
        bag1.insert("a.png", "image/png", vec![]);

        let mut bag2 = MediaBag::new();
        bag2.insert("b.png", "image/png", vec![]);

        bag1.merge(bag2);
        assert_eq!(bag1.len(), 2);
    }

    #[test]
    fn test_canonicalize_path() {
        assert_eq!(canonicalize_path(Path::new("image.png")), "image.png");
        assert_eq!(
            canonicalize_path(Path::new("path/to/image.png")),
            "path/to/image.png"
        );
        assert_eq!(
            canonicalize_path(Path::new("path/./image.png")),
            "path/image.png"
        );
        assert_eq!(
            canonicalize_path(Path::new("path/to/../image.png")),
            "path/image.png"
        );
    }

    #[test]
    fn test_mime_type_from_path() {
        assert_eq!(mime_type_from_path("image.png"), "image/png");
        assert_eq!(mime_type_from_path("image.jpg"), "image/jpeg");
        assert_eq!(mime_type_from_path("style.css"), "text/css");
        // JavaScript MIME type can be either text/javascript or application/javascript
        let js_mime = mime_type_from_path("script.js");
        assert!(js_mime == "text/javascript" || js_mime == "application/javascript");
        assert_eq!(mime_type_from_path("data.bin"), "application/octet-stream");
    }

    #[test]
    fn test_extension_from_mime_type() {
        assert_eq!(extension_from_mime_type("image/png"), Some("png"));
        assert_eq!(extension_from_mime_type("image/jpeg"), Some("jpg"));
        assert_eq!(extension_from_mime_type("text/css"), Some("css"));
        assert_eq!(extension_from_mime_type("unknown/type"), None);
    }

    #[test]
    fn test_media_bag_directory() {
        let mut bag = MediaBag::new();
        bag.insert("image.png", "image/png", vec![0; 100]);

        let dir = bag.directory();
        assert_eq!(dir.len(), 1);
        assert_eq!(dir[0].1, "image/png");
        assert_eq!(dir[0].2, 100);
    }

    #[test]
    fn test_media_bag_data_uri() {
        let mut bag = MediaBag::new();

        // Create a simple base64-encoded PNG (1x1 pixel transparent PNG)
        let data_uri = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

        let result = bag.insert_data_uri("pixel.png", data_uri);
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(bag.contains("pixel.png"));

        let item = bag.lookup("pixel.png").unwrap();
        assert_eq!(item.mime_type(), "image/png");
        assert!(item.size() > 0);
    }

    #[test]
    fn test_media_bag_to_data_uri() {
        let mut bag = MediaBag::new();
        let contents = vec![0x89, 0x50, 0x4E, 0x47];
        bag.insert("image.png", "image/png", contents.clone());

        let data_uri = bag.to_data_uri("image.png");
        assert!(data_uri.is_some());

        let uri = data_uri.unwrap();
        assert!(uri.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn test_media_bag_images() {
        let mut bag = MediaBag::new();
        bag.insert("a.png", "image/png", vec![]);
        bag.insert("b.jpg", "image/jpeg", vec![]);
        bag.insert("c.css", "text/css", vec![]);

        let images = bag.images();
        assert_eq!(images.len(), 2);
    }

    #[test]
    fn test_media_bag_fonts() {
        let mut bag = MediaBag::new();
        bag.insert("font.woff", "font/woff", vec![]);
        bag.insert("font.ttf", "font/ttf", vec![]);
        bag.insert("font.eot", "application/vnd.ms-fontobject", vec![]);
        bag.insert("image.png", "image/png", vec![]);

        let fonts = bag.fonts();
        assert_eq!(fonts.len(), 3);
    }

    #[test]
    fn test_media_bag_stylesheets() {
        let mut bag = MediaBag::new();
        bag.insert("style.css", "text/css", vec![]);
        bag.insert("theme.css", "text/css", vec![]);
        bag.insert("script.js", "application/javascript", vec![]);

        let stylesheets = bag.stylesheets();
        assert_eq!(stylesheets.len(), 2);
    }

    #[test]
    fn test_media_bag_scripts() {
        let mut bag = MediaBag::new();
        bag.insert("app.js", "application/javascript", vec![]);
        bag.insert("main.js", "text/javascript", vec![]);
        bag.insert("style.css", "text/css", vec![]);

        let scripts = bag.scripts();
        assert_eq!(scripts.len(), 2);
    }

    #[test]
    fn test_media_bag_all_valid() {
        let mut bag = MediaBag::new();
        bag.insert("a.png", "image/png", vec![0x89]);
        bag.insert("b.png", "image/png", vec![0x50]);

        assert!(bag.all_valid());

        bag.insert("c.png", "image/png", vec![]);
        assert!(!bag.all_valid());
    }

    #[test]
    fn test_media_bag_get_invalid_items() {
        let mut bag = MediaBag::new();
        bag.insert("a.png", "image/png", vec![0x89]);
        bag.insert("b.png", "image/png", vec![]);

        let invalid = bag.get_invalid_items();
        assert_eq!(invalid.len(), 1);
        assert!(invalid[0].contains("b.png"));
    }
}
