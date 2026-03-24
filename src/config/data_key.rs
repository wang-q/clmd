//! Type-safe configuration system
//!
//! Design inspired by flexmark-java's DataKey system.
//! Provides a type-safe way to manage configuration options.

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

/// A type-safe key for configuration values
///
/// DataKey provides a way to store and retrieve typed values
/// from a data holder. It's similar to flexmark-java's DataKey.
///
/// # Example
///
/// ```
/// use clmd::config::DataKey;
///
/// const SOURCEPOS: DataKey<bool> = DataKey::new("sourcepos");
/// const SMART: DataKey<bool> = DataKey::with_default("smart", false);
/// ```
#[derive(Debug, Clone)]
pub struct DataKey<T: Clone + 'static> {
    name: &'static str,
    default_value: Option<T>,
}

impl<T: Clone + 'static> DataKey<T> {
    /// Create a new data key without a default value
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            default_value: None,
        }
    }

    /// Create a new data key with a default value
    pub const fn with_default(name: &'static str, default: T) -> Self {
        Self {
            name,
            default_value: Some(default),
        }
    }

    /// Get the key name
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Get the default value if any
    pub fn default_value(&self) -> Option<T> {
        self.default_value.clone()
    }
}

impl<T: Clone + 'static> PartialEq for DataKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<T: Clone + 'static> Eq for DataKey<T> {}

impl<T: Clone + 'static> std::hash::Hash for DataKey<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// Trait for data holders
///
/// Implement this trait to provide a storage mechanism for data keys.
pub trait DataHolder {
    /// Get a value by key
    ///
    /// Returns the default value if the key is not set.
    fn get<T: Clone + 'static>(&self, key: &DataKey<T>) -> T;

    /// Set a value for a key
    fn set<T: Clone + 'static>(&mut self, key: &DataKey<T>, value: T);

    /// Check if a key has a value set
    fn contains<T: Clone + 'static>(&self, key: &DataKey<T>) -> bool;

    /// Remove a key's value
    fn remove<T: Clone + 'static>(&mut self, key: &DataKey<T>) -> Option<T>;
}

/// A mutable data set that can hold typed values
///
/// This is the main implementation of DataHolder.
/// It's similar to flexmark-java's MutableDataSet.
#[derive(Debug, Default)]
pub struct MutableDataSet {
    data: HashMap<&'static str, Box<dyn Any>>,
}

impl MutableDataSet {
    /// Create a new empty data set
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Create a new data set with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the data set is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Merge another data set into this one
    ///
    /// Values from the other set will overwrite values in this set.
    pub fn merge(&mut self, other: &Self) {
        for (key, value) in &other.data {
            self.data.insert(key, clone_box(value));
        }
    }
}

impl DataHolder for MutableDataSet {
    fn get<T: Clone + 'static>(&self, key: &DataKey<T>) -> T {
        if let Some(value) = self.data.get(key.name()) {
            if let Some(typed) = value.downcast_ref::<T>() {
                return typed.clone();
            }
        }

        key.default_value().unwrap_or_else(|| {
            panic!("No value for key: {} and no default provided", key.name())
        })
    }

    fn set<T: Clone + 'static>(&mut self, key: &DataKey<T>, value: T) {
        self.data.insert(key.name(), Box::new(value));
    }

    fn contains<T: Clone + 'static>(&self, key: &DataKey<T>) -> bool {
        self.data.contains_key(key.name())
    }

    fn remove<T: Clone + 'static>(&mut self, key: &DataKey<T>) -> Option<T> {
        self.data
            .remove(key.name())
            .and_then(|v| v.downcast::<T>().ok().map(|b| *b))
    }
}

/// Clone a boxed Any value
fn clone_box(_value: &Box<dyn Any>) -> Box<dyn Any> {
    // This is a workaround for cloning Box<dyn Any>
    // In practice, we should use Arc for shared ownership
    // or require Clone trait bounds
    unimplemented!(
        "Cloning Box<dyn Any> is not supported. Use Arc for shared ownership."
    )
}

/// Immutable data set (for sharing)
///
/// This is similar to flexmark-java's DataSet.
#[derive(Debug)]
pub struct DataSet {
    data: Arc<HashMap<&'static str, Box<dyn Any>>>,
}

impl DataSet {
    /// Create a new empty data set
    pub fn new() -> Self {
        Self {
            data: Arc::new(HashMap::new()),
        }
    }
}

impl Default for DataSet {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for DataSet {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

impl DataHolder for DataSet {
    fn get<T: Clone + 'static>(&self, key: &DataKey<T>) -> T {
        if let Some(value) = self.data.get(key.name()) {
            if let Some(typed) = value.downcast_ref::<T>() {
                return typed.clone();
            }
        }

        key.default_value().unwrap_or_else(|| {
            panic!("No value for key: {} and no default provided", key.name())
        })
    }

    fn set<T: Clone + 'static>(&mut self, _key: &DataKey<T>, _value: T) {
        panic!("Cannot modify immutable DataSet. Use MutableDataSet instead.")
    }

    fn contains<T: Clone + 'static>(&self, key: &DataKey<T>) -> bool {
        self.data.contains_key(key.name())
    }

    fn remove<T: Clone + 'static>(&mut self, _key: &DataKey<T>) -> Option<T> {
        panic!("Cannot modify immutable DataSet. Use MutableDataSet instead.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_key_creation() {
        let key: DataKey<bool> = DataKey::new("test");
        assert_eq!(key.name(), "test");
        assert!(key.default_value().is_none());

        let key_with_default: DataKey<bool> = DataKey::with_default("test2", true);
        assert_eq!(key_with_default.name(), "test2");
        assert_eq!(key_with_default.default_value(), Some(true));
    }

    #[test]
    fn test_mutable_data_set_basic() {
        let mut data = MutableDataSet::new();
        let key: DataKey<i32> = DataKey::new("count");

        // Set and get
        data.set(&key, 42);
        assert_eq!(data.get(&key), 42);

        // Contains
        assert!(data.contains(&key));

        // Remove
        let removed = data.remove(&key);
        assert_eq!(removed, Some(42));
        assert!(!data.contains(&key));
    }

    #[test]
    fn test_mutable_data_set_with_default() {
        let data = MutableDataSet::new();
        let key: DataKey<bool> = DataKey::with_default("enabled", false);

        // Should return default when not set
        assert_eq!(data.get(&key), false);

        // After setting, should return set value
        let mut data2 = MutableDataSet::new();
        data2.set(&key, true);
        assert_eq!(data2.get(&key), true);
    }

    #[test]
    #[should_panic(expected = "No value for key")]
    fn test_data_key_no_default() {
        let data = MutableDataSet::new();
        let key: DataKey<i32> = DataKey::new("missing");
        let _ = data.get(&key); // Should panic
    }

    #[test]
    fn test_mutable_data_set_multiple_types() {
        let mut data = MutableDataSet::new();

        let bool_key: DataKey<bool> = DataKey::new("flag");
        let string_key: DataKey<String> = DataKey::new("name");
        let int_key: DataKey<i32> = DataKey::new("count");

        data.set(&bool_key, true);
        data.set(&string_key, "test".to_string());
        data.set(&int_key, 42);

        assert_eq!(data.get(&bool_key), true);
        assert_eq!(data.get(&string_key), "test".to_string());
        assert_eq!(data.get(&int_key), 42);
    }

    #[test]
    fn test_data_set_len_and_empty() {
        let mut data = MutableDataSet::new();
        assert!(data.is_empty());
        assert_eq!(data.len(), 0);

        let key: DataKey<i32> = DataKey::new("test");
        data.set(&key, 1);
        assert!(!data.is_empty());
        assert_eq!(data.len(), 1);

        data.clear();
        assert!(data.is_empty());
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn test_data_key_equality() {
        let key1: DataKey<i32> = DataKey::new("same");
        let key2: DataKey<i32> = DataKey::new("same");
        let key3: DataKey<i32> = DataKey::new("different");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_data_key_hash() {
        use std::collections::HashSet;

        let key1: DataKey<i32> = DataKey::new("test");
        let key2: DataKey<i32> = DataKey::new("test");
        let key3: DataKey<i32> = DataKey::new("other");

        let mut set = HashSet::new();
        set.insert(key1);
        assert!(set.contains(&key2));
        assert!(!set.contains(&key3));
    }
}
