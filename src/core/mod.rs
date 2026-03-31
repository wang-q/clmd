//! Core abstraction layer for clmd.
//!
//! This module provides the fundamental abstractions and types used throughout
//! the clmd library, inspired by Pandoc's core architecture.
//!
//! # Modules
//!
//! - [`monad`]: Monad abstraction for IO and pure operations
//! - [`state`]: Common state management
//! - [`options`]: Reader and writer options
//!
//! # Example
//!
//! ```
//! use clmd::core::{ClmdMonad, ClmdIO, Verbosity, CommonState, ReaderOptions, WriterOptions};
//!
//! let monad = ClmdIO::with_verbosity(Verbosity::Info);
//! monad.log_info("Starting document conversion");
//!
//! let mut state = CommonState::new();
//! state.add_input_file("document.md");
//!
//! let reader_opts = ReaderOptions::for_format("gfm");
//! let writer_opts = WriterOptions::for_format("html");
//! ```

pub mod monad;
pub mod options;
pub mod state;
pub mod walk;

// Re-export commonly used types
pub use monad::{
    share_monad, ClmdIO, ClmdMonad, ClmdPure, LogMessage as MonadLogMessage,
    SharedMonad, Verbosity,
};
pub use options::{
    EmailObfuscation, ExtensionConfig, LineEnding, MarkdownFlavor, ReaderOptions,
    ReferenceLocation, UnifiedOptions, WrapOption, WriterOptions,
};
pub use state::{
    CommonState, ExtensionData, LogLevel, LogMessage, TrackChanges, Translations,
};
pub use walk::{
    collect_nodes, query, query_any, walk, walk_m, walk_with_context, Walkable, Walker,
};
