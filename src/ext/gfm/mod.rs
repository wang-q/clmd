//! GitHub Flavored Markdown (GFM) extensions.
//!
//! This module provides GFM-specific extensions including tables, strikethrough,
//! task lists, autolinks, and tag filtering.

/// Table extension for GFM.
pub mod table;

/// Strikethrough extension for GFM.
pub mod strikethrough;

/// Task list extension for GFM.
pub mod tasklist;

/// Autolink extension for GFM.
pub mod autolink;

/// Tag filter extension for GFM.
pub mod tagfilter;
