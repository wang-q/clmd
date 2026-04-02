//! Command-line subcommands for the clmd binary.
//!
//! This module provides implementations for various subcommands
//! including convert, extract, stats, toc, fmt, validate, and transform.

pub mod complete;
pub mod convert;
pub mod extract;
pub mod fmt;
pub mod stats;
pub mod toc;
pub mod transform;
pub mod utils;
pub mod validate;

/// Deprecated: Use `convert` instead.
#[allow(unused_imports)]
pub mod from {
    pub use crate::cmd::convert::*;
}

/// Deprecated: Use `convert` instead.
#[allow(unused_imports)]
pub mod to {
    pub use crate::cmd::convert::*;
}
