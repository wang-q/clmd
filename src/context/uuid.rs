//! UUID generation utilities for clmd.
//!
//! This module provides RFC 4122 UUID Version 4 generation,
//! inspired by Pandoc's UUID module. UUIDs can be used for
//! document identification and other purposes.
//!
//! # Example
//!
//! ```ignore
//! use clmd::uuid::UUID;
//!
//! // Generate a new random UUID
//! let uuid = UUID::new_v4();
//! println!("UUID: {}", uuid.to_hyphenated_string());
//! ```

use rand::Rng;
use std::fmt;

/// A UUID (Universally Unique Identifier) as defined by RFC 4122.
///
/// This is a 128-bit value that can be used to uniquely identify
/// documents, resources, or other entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UUID([u8; 16]);

impl UUID {
    /// Creates a new random UUID (Version 4).
    ///
    /// This generates a random UUID according to RFC 4122 Section 4.4.
    /// The UUID contains 122 bits of randomness.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::uuid::UUID;
    ///
    /// let uuid = UUID::new_v4();
    /// assert_eq!(uuid.version(), 4);
    /// ```
    pub fn new_v4() -> Self {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);

        // Set version (4) in the most significant 4 bits of byte 6
        bytes[6] = (bytes[6] & 0x0F) | 0x40;

        // Set variant (10) in the most significant 2 bits of byte 8
        bytes[8] = (bytes[8] & 0x3F) | 0x80;

        Self(bytes)
    }

    /// Creates a UUID from a 16-byte array.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The 16 bytes of the UUID
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::uuid::UUID;
    ///
    /// let bytes = [0x55, 0x44, 0x33, 0x22, 0x11, 0x00, 0x99, 0x88,
    ///              0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00];
    /// let uuid = UUID::from_bytes(bytes);
    /// ```
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Returns the UUID as a 16-byte array.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::uuid::UUID;
    ///
    /// let uuid = UUID::new_v4();
    /// let bytes = uuid.as_bytes();
    /// assert_eq!(bytes.len(), 16);
    /// ```
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Returns the UUID version number.
    ///
    /// For UUIDs created with `new_v4()`, this will return 4.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::uuid::UUID;
    ///
    /// let uuid = UUID::new_v4();
    /// assert_eq!(uuid.version(), 4);
    /// ```
    pub fn version(&self) -> u8 {
        (self.0[6] >> 4) & 0x0F
    }

    /// Returns the UUID variant.
    ///
    /// Returns the variant number as defined by RFC 4122:
    /// - 0: NCS backward compatibility (0b0xxxxxxx)
    /// - 1: RFC 4122 variant (0b10xxxxxx)
    /// - 2: Microsoft backward compatibility (0b110xxxxx)
    /// - 3: Reserved for future use (0b111xxxxx)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::uuid::UUID;
    ///
    /// let uuid = UUID::new_v4();
    /// assert_eq!(uuid.variant(), 1); // RFC 4122
    /// ```
    pub fn variant(&self) -> u8 {
        let byte = self.0[8];
        if byte & 0b1000_0000 == 0 {
            0 // NCS
        } else if byte & 0b0100_0000 == 0 {
            1 // RFC 4122 (10xxxxxx)
        } else if byte & 0b0010_0000 == 0 {
            2 // Microsoft (110xxxxx)
        } else {
            3 // Reserved (111xxxxx)
        }
    }

    /// Returns the UUID as a hyphenated string.
    ///
    /// Format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::uuid::UUID;
    ///
    /// let uuid = UUID::new_v4();
    /// let s = uuid.to_hyphenated_string();
    /// assert_eq!(s.len(), 36);
    /// assert!(s.contains('-'));
    /// ```
    pub fn to_hyphenated_string(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3],
            self.0[4], self.0[5],
            self.0[6], self.0[7],
            self.0[8], self.0[9],
            self.0[10], self.0[11], self.0[12], self.0[13], self.0[14], self.0[15]
        )
    }

    /// Returns the UUID as a simple string without hyphens.
    ///
    /// Format: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::uuid::UUID;
    ///
    /// let uuid = UUID::new_v4();
    /// let s = uuid.to_simple_string();
    /// assert_eq!(s.len(), 32);
    /// assert!(!s.contains('-'));
    /// ```
    pub fn to_simple_string(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3],
            self.0[4], self.0[5], self.0[6], self.0[7],
            self.0[8], self.0[9], self.0[10], self.0[11],
            self.0[12], self.0[13], self.0[14], self.0[15]
        )
    }

    /// Returns the UUID as a URN string.
    ///
    /// Format: urn:uuid:xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::uuid::UUID;
    ///
    /// let uuid = UUID::new_v4();
    /// let s = uuid.to_urn_string();
    /// assert!(s.starts_with("urn:uuid:"));
    /// ```
    pub fn to_urn_string(&self) -> String {
        format!("urn:uuid:{}", self.to_hyphenated_string())
    }

    /// Parses a UUID from a string.
    ///
    /// Accepts the following formats:
    /// - Simple: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    /// - Hyphenated: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    /// - URN: urn:uuid:xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    ///
    /// # Arguments
    ///
    /// * `s` - The string to parse
    ///
    /// # Returns
    ///
    /// `Some(UUID)` if parsing succeeds, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::uuid::UUID;
    ///
    /// let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    /// let uuid = UUID::parse(uuid_str);
    /// assert!(uuid.is_some());
    /// ```
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();

        // Handle URN prefix
        let s = if let Some(stripped) = s.strip_prefix("urn:uuid:") {
            stripped
        } else {
            s
        };

        // Remove hyphens
        let hex: String = s.chars().filter(|&c| c != '-').collect();

        if hex.len() != 32 {
            return None;
        }

        let mut bytes = [0u8; 16];
        for (i, byte) in bytes.iter_mut().enumerate() {
            let start = i * 2;
            let end = start + 2;
            if let Ok(val) = u8::from_str_radix(&hex[start..end], 16) {
                *byte = val;
            } else {
                return None;
            }
        }

        Some(Self(bytes))
    }

    /// Returns the nil UUID (all zeros).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::uuid::UUID;
    ///
    /// let nil = UUID::nil();
    /// assert_eq!(nil.to_simple_string(), "00000000000000000000000000000000");
    /// ```
    pub const fn nil() -> Self {
        Self([0; 16])
    }

    /// Checks if this is the nil UUID.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::uuid::UUID;
    ///
    /// let nil = UUID::nil();
    /// assert!(nil.is_nil());
    ///
    /// let uuid = UUID::new_v4();
    /// assert!(!uuid.is_nil());
    /// ```
    pub fn is_nil(&self) -> bool {
        self.0 == [0; 16]
    }
}

impl fmt::Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hyphenated_string())
    }
}

impl Default for UUID {
    fn default() -> Self {
        Self::new_v4()
    }
}

impl AsRef<[u8]> for UUID {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 16]> for UUID {
    fn from(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

impl From<UUID> for [u8; 16] {
    fn from(uuid: UUID) -> Self {
        uuid.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_v4() {
        let uuid = UUID::new_v4();
        assert_eq!(uuid.version(), 4);
        assert_eq!(uuid.variant(), 1); // RFC 4122
    }

    #[test]
    fn test_unique() {
        let uuid1 = UUID::new_v4();
        let uuid2 = UUID::new_v4();
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_to_hyphenated_string() {
        let uuid = UUID::new_v4();
        let s = uuid.to_hyphenated_string();
        assert_eq!(s.len(), 36);
        assert_eq!(s.chars().filter(|&c| c == '-').count(), 4);
    }

    #[test]
    fn test_to_simple_string() {
        let uuid = UUID::new_v4();
        let s = uuid.to_simple_string();
        assert_eq!(s.len(), 32);
        assert!(!s.contains('-'));
    }

    #[test]
    fn test_to_urn_string() {
        let uuid = UUID::new_v4();
        let s = uuid.to_urn_string();
        assert!(s.starts_with("urn:uuid:"));
        assert_eq!(s.len(), 9 + 36);
    }

    #[test]
    fn test_parse_hyphenated() {
        let uuid = UUID::new_v4();
        let s = uuid.to_hyphenated_string();
        let parsed = UUID::parse(&s);
        assert_eq!(parsed, Some(uuid));
    }

    #[test]
    fn test_parse_simple() {
        let uuid = UUID::new_v4();
        let s = uuid.to_simple_string();
        let parsed = UUID::parse(&s);
        assert_eq!(parsed, Some(uuid));
    }

    #[test]
    fn test_parse_urn() {
        let uuid = UUID::new_v4();
        let s = uuid.to_urn_string();
        let parsed = UUID::parse(&s);
        assert_eq!(parsed, Some(uuid));
    }

    #[test]
    fn test_parse_invalid() {
        assert!(UUID::parse("").is_none());
        assert!(UUID::parse("not-a-uuid").is_none());
        assert!(UUID::parse("550e8400").is_none()); // Too short
    }

    #[test]
    fn test_nil() {
        let nil = UUID::nil();
        assert!(nil.is_nil());
        assert_eq!(nil.to_simple_string(), "00000000000000000000000000000000");
    }

    #[test]
    fn test_from_bytes() {
        let bytes = [
            0x55, 0x44, 0x33, 0x22, 0x11, 0x00, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44,
            0x33, 0x22, 0x11, 0x00,
        ];
        let uuid = UUID::from_bytes(bytes);
        assert_eq!(uuid.as_bytes(), &bytes);
    }

    #[test]
    fn test_display() {
        let uuid = UUID::new_v4();
        let s = format!("{}", uuid);
        assert_eq!(s.len(), 36);
        assert!(s.contains('-'));
    }

    #[test]
    fn test_default() {
        let uuid: UUID = Default::default();
        assert_eq!(uuid.version(), 4);
    }

    #[test]
    fn test_as_ref() {
        let uuid = UUID::new_v4();
        let bytes: &[u8] = uuid.as_ref();
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_from_array() {
        let bytes = [0u8; 16];
        let uuid: UUID = bytes.into();
        assert!(uuid.is_nil());
    }

    #[test]
    fn test_into_array() {
        let uuid = UUID::nil();
        let bytes: [u8; 16] = uuid.into();
        assert_eq!(bytes, [0u8; 16]);
    }
}
