//! String pool for efficient string reuse
//!
//! This module provides a string pool to reduce memory allocations
//! by reusing String buffers.

/// A simple string pool for reusing String buffers
pub struct StringPool {
    buffer: String,
}

impl StringPool {
    /// Create a new string pool with the given initial capacity
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buffer: String::with_capacity(cap),
        }
    }

    /// Clear the buffer without deallocating
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get a mutable reference to the buffer
    pub fn buffer_mut(&mut self) -> &mut String {
        &mut self.buffer
    }

    /// Get the buffer as a string slice
    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    /// Append a string to the buffer
    pub fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    /// Push a character to the buffer
    pub fn push(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// Take the buffer content and clear it
    pub fn take(&mut self) -> String {
        let result = self.buffer.clone();
        self.buffer.clear();
        result
    }

    /// Take the buffer content as a new string without cloning
    pub fn take_string(&mut self) -> String {
        std::mem::take(&mut self.buffer)
    }
}

impl Default for StringPool {
    fn default() -> Self {
        Self::with_capacity(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_pool_basic() {
        let mut pool = StringPool::with_capacity(100);
        pool.push_str("Hello");
        assert_eq!(pool.as_str(), "Hello");
        
        pool.clear();
        assert_eq!(pool.as_str(), "");
        assert!(pool.buffer_mut().capacity() >= 100);
    }

    #[test]
    fn test_string_pool_take() {
        let mut pool = StringPool::with_capacity(100);
        pool.push_str("World");
        
        let s = pool.take_string();
        assert_eq!(s, "World");
        assert_eq!(pool.as_str(), "");
    }
}
