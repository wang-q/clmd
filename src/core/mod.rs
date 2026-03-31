//! Core abstraction layer for clmd.
//!
//! This module provides the fundamental abstractions and types used throughout
//! the clmd library, inspired by Pandoc's core architecture.
//!
//! # Modules
//!
//! - [`monad`]: Monad abstraction for IO and pure operations
//! - [`state`]: Common state management
//!
//! # Example
//!
//! ```
//! use clmd::core::{ClmdMonad, ClmdIO, Verbosity, CommonState};
//!
//! let monad = ClmdIO::with_verbosity(Verbosity::Info);
//! monad.log_info("Starting document conversion");
//!
//! let mut state = CommonState::new();
//! state.add_input_file("document.md");
//! ```

pub mod monad;
pub mod state;

// Re-export commonly used types
pub use monad::{
    share_monad, ClmdIO, ClmdMonad, ClmdPure, LogMessage as MonadLogMessage,
    SharedMonad, Verbosity,
};
pub use state::{
    CommonState, ExtensionData, LogLevel, LogMessage, TrackChanges, Translations,
};
