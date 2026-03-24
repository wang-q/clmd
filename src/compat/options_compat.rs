//! Options compatibility layer
//!
//! Bridges the old u32-based options system with the new DataKey-based system.

use crate::config::{DataHolder, DataKey, MutableDataSet};
use crate::options;

/// New-style parser options using DataKey
///
/// This provides a type-safe alternative to the u32-based options.
///
/// # Example
///
/// ```
/// use clmd::compat::ParserOptions;
/// use clmd::config::DataHolder;
///
/// let mut options = ParserOptions::new();
/// options.set_sourcepos(true);
/// options.set_smart(true);
///
/// assert!(options.get_sourcepos());
/// assert!(options.get_smart());
/// ```
#[derive(Debug)]
pub struct ParserOptions {
    data: MutableDataSet,
}

impl ParserOptions {
    /// DataKey for sourcepos option
    pub const SOURCEPOS_KEY: DataKey<bool> = DataKey::with_default("sourcepos", false);
    /// DataKey for smart option
    pub const SMART_KEY: DataKey<bool> = DataKey::with_default("smart", false);
    /// DataKey for unsafe option
    pub const UNSAFE_KEY: DataKey<bool> = DataKey::with_default("unsafe", false);
    /// DataKey for validate_utf8 option
    pub const VALIDATE_UTF8_KEY: DataKey<bool> = DataKey::with_default("validate_utf8", false);

    /// Create a new parser options with defaults
    pub fn new() -> Self {
        Self {
            data: MutableDataSet::new(),
        }
    }

    /// Create parser options from old-style u32 options
    pub fn from_u32(options: u32) -> Self {
        let mut opts = Self::new();
        opts.set_sourcepos((options & crate::options::SOURCEPOS) != 0);
        opts.set_smart((options & crate::options::SMART) != 0);
        opts.set_unsafe((options & crate::options::UNSAFE) != 0);
        opts
    }

    /// Convert to old-style u32 options
    pub fn to_u32(&self) -> u32 {
        let mut result = 0u32;
        if self.get_sourcepos() {
            result |= crate::options::SOURCEPOS;
        }
        if self.get_smart() {
            result |= crate::options::SMART;
        }
        if self.get_unsafe() {
            result |= crate::options::UNSAFE;
        }
        result
    }

    /// Get sourcepos option
    pub fn get_sourcepos(&self) -> bool {
        self.data.get(&Self::SOURCEPOS_KEY)
    }

    /// Set sourcepos option
    pub fn set_sourcepos(&mut self, value: bool) -> &mut Self {
        self.data.set(&Self::SOURCEPOS_KEY, value);
        self
    }

    /// Get smart option
    pub fn get_smart(&self) -> bool {
        self.data.get(&Self::SMART_KEY)
    }

    /// Set smart option
    pub fn set_smart(&mut self, value: bool) -> &mut Self {
        self.data.set(&Self::SMART_KEY, value);
        self
    }

    /// Get unsafe option
    pub fn get_unsafe(&self) -> bool {
        self.data.get(&Self::UNSAFE_KEY)
    }

    /// Set unsafe option
    pub fn set_unsafe(&mut self, value: bool) -> &mut Self {
        self.data.set(&Self::UNSAFE_KEY, value);
        self
    }

    /// Get validate_utf8 option
    pub fn get_validate_utf8(&self) -> bool {
        self.data.get(&Self::VALIDATE_UTF8_KEY)
    }

    /// Set validate_utf8 option
    pub fn set_validate_utf8(&mut self, value: bool) -> &mut Self {
        self.data.set(&Self::VALIDATE_UTF8_KEY, value);
        self
    }

    /// Get a generic option value
    pub fn get<T: Clone + 'static>(&self, key: &DataKey<T>) -> T {
        self.data.get(key)
    }

    /// Set a generic option value
    pub fn set<T: Clone + 'static>(&mut self, key: &DataKey<T>, value: T) -> &mut Self {
        self.data.set(key, value);
        self
    }

    /// Check if an option is set
    pub fn contains<T: Clone + 'static>(&self, key: &DataKey<T>) -> bool {
        self.data.contains(key)
    }
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for ParserOptions
///
/// # Example
///
/// ```
/// use clmd::compat::ParserOptionsBuilder;
///
/// let options = ParserOptionsBuilder::new()
///     .sourcepos(true)
///     .smart(true)
///     .build();
///
/// assert!(options.get_sourcepos());
/// assert!(options.get_smart());
/// ```
pub struct ParserOptionsBuilder {
    options: ParserOptions,
}

impl ParserOptionsBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            options: ParserOptions::new(),
        }
    }

    /// Set sourcepos option
    pub fn sourcepos(mut self, value: bool) -> Self {
        self.options.set_sourcepos(value);
        self
    }

    /// Set smart option
    pub fn smart(mut self, value: bool) -> Self {
        self.options.set_smart(value);
        self
    }

    /// Set unsafe option
    pub fn unsafe_(mut self, value: bool) -> Self {
        self.options.set_unsafe(value);
        self
    }

    /// Set validate_utf8 option
    pub fn validate_utf8(mut self, value: bool) -> Self {
        self.options.set_validate_utf8(value);
        self
    }

    /// Build the options
    pub fn build(self) -> ParserOptions {
        self.options
    }
}

impl Default for ParserOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_options_default() {
        let options = ParserOptions::new();
        assert!(!options.get_sourcepos());
        assert!(!options.get_smart());
        assert!(!options.get_unsafe());
    }

    #[test]
    fn test_parser_options_setters() {
        let mut options = ParserOptions::new();
        options.set_sourcepos(true).set_smart(true);

        assert!(options.get_sourcepos());
        assert!(options.get_smart());
        assert!(!options.get_unsafe());
    }

    #[test]
    fn test_parser_options_from_u32() {
        let options = ParserOptions::from_u32(crate::options::SOURCEPOS | crate::options::SMART);

        assert!(options.get_sourcepos());
        assert!(options.get_smart());
        assert!(!options.get_unsafe());
    }

    #[test]
    fn test_parser_options_to_u32() {
        let mut options = ParserOptions::new();
        options.set_sourcepos(true).set_smart(true);

        let u32_opts = options.to_u32();
        assert!((u32_opts & crate::options::SOURCEPOS) != 0);
        assert!((u32_opts & crate::options::SMART) != 0);
        assert!((u32_opts & crate::options::UNSAFE) == 0);
    }

    #[test]
    fn test_parser_options_builder() {
        let options = ParserOptionsBuilder::new()
            .sourcepos(true)
            .smart(true)
            .build();

        assert!(options.get_sourcepos());
        assert!(options.get_smart());
        assert!(!options.get_unsafe());
    }

    #[test]
    fn test_parser_options_custom_key() {
        let mut options = ParserOptions::new();
        let custom_key: DataKey<String> = DataKey::new("custom");

        options.set(&custom_key, "value".to_string());
        assert_eq!(options.get(&custom_key), "value");
    }

    #[test]
    fn test_round_trip_conversion() {
        let original = crate::options::SOURCEPOS | crate::options::SMART;
        let options = ParserOptions::from_u32(original);
        let converted = options.to_u32();
        assert_eq!(original, converted);
    }
}
