//! Compatibility layer for AST refactoring
//!
//! This module provides bridges between the old and new AST systems,
//! allowing gradual migration without breaking existing code.

pub mod node_compat;
pub mod options_compat;

pub use node_compat::*;
pub use options_compat::*;
