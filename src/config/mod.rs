//! Configuration options
//!
//! Provides a type-safe configuration system inspired by flexmark-java's DataKey.
//!
//! # Example
//!
//! ```
//! use clmd::config::{DataKey, MutableDataSet, DataHolder};
//!
//! const SOURCEPOS: DataKey<bool> = DataKey::with_default("sourcepos", false);
//! const SMART: DataKey<bool> = DataKey::with_default("smart", false);
//!
//! let mut options = MutableDataSet::new();
//! options.set(&SOURCEPOS, true);
//!
//! assert_eq!(options.get(&SOURCEPOS), true);
//! assert_eq!(options.get(&SMART), false); // Uses default
//! ```

pub mod data_key;

pub use data_key::{DataHolder, DataKey, DataSet, MutableDataSet};
