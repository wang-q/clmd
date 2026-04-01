//! Context module for clmd.
//!
//! This module provides runtime context, configuration, and utility services
//! for the clmd Markdown parser. It includes configuration management, logging,
//! data file access, and system utilities.

// Common types and traits
pub mod common;

// Configuration and data
pub mod config;
pub mod data;

// IO and logging
pub mod io;
pub mod logging;

// System utilities
pub mod mediabag;
pub mod process;
pub mod pure;
pub mod uuid;
pub mod version;

// Re-export commonly used types
pub use common::{
    ClmdContext, CommonState, LogLevel, LogMessage, Verbosity,
};
pub use config::{Config, ConfigLoader};
pub use data::{read_data_file, DataFileManager};
pub use io::IoContext;
pub use logging::{Logger, LogLevel as LoggingLogLevel};
pub use mediabag::MediaBag;
pub use process::{pipe_process, ProcessOptions};
pub use pure::PureContext;
pub use uuid::UUID;
pub use version::{VersionInfo, VERSION};
