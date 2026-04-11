//! Context module for clmd.
//!
//! This module provides runtime context, configuration, and utility services
//! for the clmd Markdown parser.

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
pub mod version;

// Re-export commonly used types
pub use common::{ClmdContext, CommonState, LogLevel, LogMessage, Verbosity};
pub use config::Config;
pub use data::read_data_file;
pub use io::IoContext;
pub use logging::{LogLevel as LoggingLogLevel, Logger};
pub use mediabag::{MediaBag, MediaItem};
pub use pure::PureContext;
pub use version::VERSION;
