//! Markdown extensions management using bitflags.
//!
//! This module provides a unified way to manage Markdown extensions,
//! inspired by Pandoc's extension system.
//!
//! # Example
//!
//! ```ignore
//! use clmd::extensions::ExtensionFlags;
//!
//! let mut extensions = ExtensionFlags::empty();
//! extensions |= ExtensionFlags::TABLES;
//! extensions |= ExtensionFlags::STRIKETHROUGH;
//!
//! assert!(extensions.contains(ExtensionFlags::TABLES));
//! ```

// Re-export everything from ext::flags for backward compatibility
pub use crate::ext::flags::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_flags_default() {
        let ext = ExtensionFlags::default();
        assert!(ext.is_empty());
    }

    #[test]
    fn test_extension_flags_gfm() {
        let ext = ExtensionFlags::gfm();
        assert!(ext.contains(ExtensionFlags::TABLES));
        assert!(ext.contains(ExtensionFlags::TASKLISTS));
        assert!(ext.contains(ExtensionFlags::STRIKETHROUGH));
        assert!(ext.contains(ExtensionFlags::AUTOLINKS));
        assert!(ext.contains(ExtensionFlags::TAGFILTER));
    }

    #[test]
    fn test_extension_kind_from_str() {
        assert_eq!(ExtensionKind::from_str("table"), Some(ExtensionKind::Table));
        assert_eq!(ExtensionKind::from_str("TABLE"), Some(ExtensionKind::Table));
        assert_eq!(ExtensionKind::from_str("unknown"), None);
    }

    #[test]
    fn test_extension_kind_as_str() {
        assert_eq!(ExtensionKind::Table.as_str(), "table");
        assert_eq!(ExtensionKind::Strikethrough.as_str(), "strikethrough");
    }

    #[test]
    fn test_extension_kind_all() {
        let all = ExtensionKind::all();
        assert!(!all.is_empty());
        assert!(all.contains(&ExtensionKind::Table));
    }
}
