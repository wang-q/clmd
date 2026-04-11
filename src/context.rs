//! Context module for clmd.
//!
//! This module provides runtime context, configuration, and utility services
//! for the clmd Markdown parser.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

use crate::core::error::{ClmdError, ClmdResult};
use crate::io::format::mime::get_mime_type_def;

// Re-export config types from the unified options::serde module
pub use crate::options::serde::{
    Config, ExtensionConfig, FormatConfig, ParseConfig, RenderConfig, SyntaxConfig,
    TransformConfig, WriterConfig,
};

// ============================================================================
// Log Level and Message Types
// ============================================================================

/// Log level for messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    /// Debug information - verbose details for debugging.
    Debug,
    /// General information - normal operation messages.
    Info,
    /// Warning - potential issues that don't prevent operation.
    Warning,
    /// Error - problems that prevented an operation from completing.
    Error,
}

impl LogLevel {
    /// Get the string representation of the log level.
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// Check if this level should be logged given a verbosity setting.
    pub fn should_log(&self, verbosity: u8) -> bool {
        match self {
            LogLevel::Error => true,
            LogLevel::Warning => verbosity >= 1,
            LogLevel::Info => verbosity >= 1,
            LogLevel::Debug => verbosity >= 2,
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A log message.
#[derive(Debug, Clone)]
pub struct LogMessage {
    /// The log level.
    pub level: LogLevel,
    /// The log message.
    pub message: String,
    /// The timestamp.
    pub timestamp: SystemTime,
}

/// Verbosity level for context operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Quiet mode - only errors.
    Quiet,
    /// Normal mode - errors and warnings.
    #[default]
    Normal,
    /// Info mode - errors, warnings, and info.
    Info,
    /// Debug mode - all messages.
    Debug,
}

impl Verbosity {
    /// Convert to a numeric value.
    pub fn as_u8(&self) -> u8 {
        match self {
            Verbosity::Quiet => 0,
            Verbosity::Normal => 1,
            Verbosity::Info => 1,
            Verbosity::Debug => 2,
        }
    }
}

// ============================================================================
// MediaBag - Resource Management System
// ============================================================================

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
    fn new<P: AsRef<Path>>(
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

/// Determine MIME type from file path based on extension.
pub fn mime_type_from_path<P: AsRef<Path>>(path: P) -> String {
    get_mime_type_def(path.as_ref()).to_string()
}

// ============================================================================
// ClmdContext Trait and Common State
// ============================================================================

/// The ClmdContext trait defines the interface for context operations.
///
/// This trait abstracts over IO operations, logging, and resource management,
/// allowing for both real IO operations and pure/mock implementations.
pub trait ClmdContext {
    /// The error type returned by operations.
    type Error;

    /// Read a file's contents.
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Self::Error>;

    /// Write content to a file.
    fn write_file(&self, path: &Path, content: &[u8]) -> Result<(), Self::Error>;

    /// Check if a file exists.
    fn file_exists(&self, path: &Path) -> bool;

    /// Get the modification time of a file.
    fn get_modification_time(&self, path: &Path) -> Result<SystemTime, Self::Error>;

    /// Find a file in the search path.
    fn find_file(&self, filename: &str) -> Option<PathBuf>;

    /// Report a log message.
    fn report(&self, level: LogLevel, message: String);

    /// Get all logged messages.
    fn get_logs(&self) -> Vec<LogMessage>;

    /// Get the current verbosity level.
    fn get_verbosity(&self) -> Verbosity;

    /// Set the verbosity level.
    fn set_verbosity(&mut self, verbosity: Verbosity);

    /// Get a reference to the common state.
    fn get_state(&self) -> &CommonState;

    /// Get a mutable reference to the common state.
    fn get_state_mut(&mut self) -> &mut CommonState;

    /// Insert media into the media bag.
    fn insert_media(
        &self,
        path: &Path,
        mime_type: Option<&str>,
        data: Vec<u8>,
    ) -> Result<String, Self::Error>;

    /// Lookup media in the media bag.
    fn lookup_media(&self, path: &Path) -> Option<MediaItem>;

    /// Get the current time.
    fn get_current_time(&self) -> SystemTime;

    /// Get random bytes.
    fn get_random_bytes(&self, len: usize) -> Vec<u8>;
}

/// Common state shared between context implementations.
#[derive(Debug, Clone)]
pub struct CommonState {
    /// The user data directory.
    pub user_data_dir: Option<PathBuf>,
    /// The media bag for storing binary resources.
    pub media_bag: Arc<Mutex<MediaBag>>,
    /// The verbosity level.
    pub verbosity: Verbosity,
    /// Log messages.
    pub logs: Arc<Mutex<Vec<LogMessage>>>,
    /// Search path for files.
    pub search_path: Vec<PathBuf>,
    /// Environment variables.
    pub env_vars: HashMap<String, String>,
}

impl Default for CommonState {
    fn default() -> Self {
        Self {
            user_data_dir: None,
            media_bag: Arc::new(Mutex::new(MediaBag::new())),
            verbosity: Verbosity::Normal,
            logs: Arc::new(Mutex::new(Vec::new())),
            search_path: Vec::new(),
            env_vars: std::env::vars().collect(),
        }
    }
}

impl CommonState {
    /// Find a file in the search path.
    pub fn find_file(&self, filename: &str) -> Option<PathBuf> {
        // First check if it's an absolute path
        let path = Path::new(filename);
        if path.is_absolute() && path.exists() {
            return Some(path.to_path_buf());
        }

        // Search in the search path
        for dir in &self.search_path {
            let full_path = dir.join(filename);
            if full_path.exists() {
                return Some(full_path);
            }
        }

        // Check in user data directory
        if let Some(user_dir) = &self.user_data_dir {
            let full_path = user_dir.join(filename);
            if full_path.exists() {
                return Some(full_path);
            }
        }

        None
    }

    /// Log a message.
    pub fn log(&self, level: LogLevel, message: String) {
        if level.should_log(self.verbosity.as_u8()) {
            let mut logs = self.logs.lock().unwrap();
            logs.push(LogMessage {
                level,
                message,
                timestamp: SystemTime::now(),
            });
        }
    }

    /// Get all logs.
    pub fn get_logs(&self) -> Vec<LogMessage> {
        self.logs.lock().unwrap().clone()
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Get the default user data directory.
pub fn default_user_data_dir() -> Option<PathBuf> {
    // Try XDG_DATA_HOME first
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        let path = PathBuf::from(xdg_data).join("clmd");
        return Some(path);
    }

    // Try home directory
    if let Some(home) = dirs::home_dir() {
        // Check for XDG default location
        let xdg_path = home.join(".local").join("share").join("clmd");
        if xdg_path.exists() {
            return Some(xdg_path);
        }

        // Return XDG default as the preferred location
        return Some(xdg_path);
    }

    None
}

/// Check if a string is a data URI.
pub fn is_data_uri(s: &str) -> bool {
    s.starts_with("data:")
}

/// Canonicalize a path.
pub fn canonicalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Generate a hash-based path for data URIs.
///
/// This is used by `insert_media` implementations to generate a unique
/// path for data URI content based on the content hash.
///
/// # Arguments
///
/// * `data` - The binary content to hash
/// * `mime_type` - Optional MIME type for determining file extension
///
/// # Returns
///
/// A hash-based path string (e.g., "abc123.png")
pub fn generate_hash_path(data: &[u8], mime_type: Option<&str>) -> String {
    let hash = format!("{:x}", md5::compute(data));
    let ext = mime_type
        .and_then(|m| m.split('/').nth(1))
        .map(|s| s.split(';').next().unwrap_or(s))
        .map(|s| format!(".{}", s))
        .unwrap_or_default();
    format!("{}{}", hash, ext)
}

// ============================================================================
// Configuration Support
// ============================================================================

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

// ============================================================================
// Data File Access
// ============================================================================

/// Cache for embedded data files.
static EMBEDDED_DATA_CACHE: OnceLock<HashMap<&'static str, &'static [u8]>> =
    OnceLock::new();

/// Initialize the embedded data cache with built-in files.
fn init_embedded_data() -> HashMap<&'static str, &'static [u8]> {
    let mut map: HashMap<&'static str, &'static [u8]> = HashMap::new();

    let html_bytes: &[u8] = include_bytes!("../data/templates/default.html");
    map.insert("templates/default.html", html_bytes);

    map
}

/// Get the embedded data cache, initializing it if necessary.
fn get_embedded_data() -> &'static HashMap<&'static str, &'static [u8]> {
    EMBEDDED_DATA_CACHE.get_or_init(init_embedded_data)
}

/// Read a file from the default data files.
fn read_default_data_file(fname: &str) -> ClmdResult<Vec<u8>> {
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
fn get_data_dir() -> ClmdResult<PathBuf> {
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
fn get_user_data_dir() -> ClmdResult<Option<PathBuf>> {
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

// ============================================================================
// IoContext Implementation
// ============================================================================

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
                user_data_dir: default_user_data_dir(),
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
        if is_data_uri(&path_str) {
            let new_path = generate_hash_path(&data, mime_type);
            bag.insert_opt(PathBuf::from(&new_path), mime_type, data);
            return Ok(new_path);
        }

        let canonical = canonicalize_path(path);
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

// ============================================================================
// PureContext Implementation
// ============================================================================

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
        let canonical = canonicalize_path(path);
        let files = self.files.lock().unwrap();
        files.get(&canonical).cloned().ok_or_else(|| {
            ClmdError::io_error(format!("File not found: {}", path.display()))
        })
    }

    fn write_file(&self, path: &Path, content: &[u8]) -> Result<(), Self::Error> {
        let canonical = canonicalize_path(path);
        let mut files = self.files.lock().unwrap();
        files.insert(canonical, content.to_vec());
        Ok(())
    }

    fn file_exists(&self, path: &Path) -> bool {
        let canonical = canonicalize_path(path);
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
        if is_data_uri(&path_str) {
            let new_path = generate_hash_path(&data, mime_type);
            bag.insert_opt(PathBuf::from(&new_path), mime_type, data);
            return Ok(new_path);
        }

        let canonical = canonicalize_path(path);
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

// ============================================================================
// Version Information
// ============================================================================

/// The full version string of clmd (e.g., "0.1.0").
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_should_log() {
        assert!(LogLevel::Error.should_log(0));
        assert!(!LogLevel::Warning.should_log(0));
        assert!(LogLevel::Warning.should_log(1));
        assert!(!LogLevel::Debug.should_log(1));
        assert!(LogLevel::Debug.should_log(2));
    }

    #[test]
    fn test_verbosity_as_u8() {
        assert_eq!(Verbosity::Quiet.as_u8(), 0);
        assert_eq!(Verbosity::Normal.as_u8(), 1);
        assert_eq!(Verbosity::Info.as_u8(), 1);
        assert_eq!(Verbosity::Debug.as_u8(), 2);
    }

    #[test]
    fn test_is_data_uri() {
        assert!(is_data_uri("data:image/png;base64,abc"));
        assert!(!is_data_uri("https://example.com"));
    }

    #[test]
    fn test_canonicalize_path() {
        let path = Path::new("dir\\file.txt");
        assert_eq!(canonicalize_path(path), "dir/file.txt");
    }
}
