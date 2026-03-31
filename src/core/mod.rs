//! Core abstraction layer for clmd.
//!
//! This module provides the fundamental abstractions and types used throughout
//! the clmd library, inspired by Pandoc's core architecture.
//!
//! # Modules
//!
//! - [`monad`]: Monad abstraction for IO and pure operations
//!
//! # Example
//!
//! ```
//! use clmd::core::{ClmdMonad, ClmdIO, Verbosity};
//!
//! let monad = ClmdIO::with_verbosity(Verbosity::Info);
//! monad.log_info("Starting document conversion");
//! ```

pub mod monad;

// Re-export commonly used types
pub use monad::{
    share_monad, ClmdIO, ClmdMonad, ClmdPure, LogMessage, SharedMonad, Verbosity,
};
