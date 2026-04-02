//! Version information for clmd.
//!
//! This module provides version information for the clmd crate,
//! inspired by Pandoc's version module. It includes compile-time
//! version information from Cargo.
//!
//! # Example
//!
//! ```ignore
//! use clmd::context::version::{VERSION, VERSION_MAJOR, VERSION_MINOR};
//!
//! println!("clmd version: {}", VERSION);
//! println!("Major: {}, Minor: {}", VERSION_MAJOR, VERSION_MINOR);
//! ```

/// The full version string of clmd (e.g., "0.1.0").
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The major version number.
pub const VERSION_MAJOR: u32 = parse_u32(env!("CARGO_PKG_VERSION_MAJOR"));

/// The minor version number.
pub const VERSION_MINOR: u32 = parse_u32(env!("CARGO_PKG_VERSION_MINOR"));

/// The patch version number.
pub const VERSION_PATCH: u32 = parse_u32(env!("CARGO_PKG_VERSION_PATCH"));

/// The pre-release version string (e.g., "alpha.1" or empty string).
pub const VERSION_PRE: &str = env!("CARGO_PKG_VERSION_PRE");

/// The package name.
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// The package description.
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// The package authors.
pub const PKG_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// The package repository URL.
pub const PKG_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// The package homepage URL.
pub const PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

/// The package license.
pub const PKG_LICENSE: &str = env!("CARGO_PKG_LICENSE");

/// Compile-time constant helper to parse u32.
const fn parse_u32(s: &str) -> u32 {
    let mut result: u32 = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let digit = (bytes[i] as u32).wrapping_sub(b'0' as u32);
        if digit <= 9 {
            result = result * 10 + digit;
        }
        i += 1;
    }
    result
}

/// Returns the full version string.
///
/// This includes the pre-release version if present.
///
/// # Example
///
/// ```ignore
/// use clmd::version::version_string;
///
/// let version = version_string();
/// assert!(version.starts_with("clmd "));
/// ```
pub fn version_string() -> String {
    let pre = if VERSION_PRE.is_empty() {
        String::new()
    } else {
        format!("-{}", VERSION_PRE)
    };
    format!(
        "{} {}.{}.{}{}",
        PKG_NAME, VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH, pre
    )
}

/// Returns a detailed version string with build information.
///
/// # Example
///
/// ```ignore
/// use clmd::version::version_string_detailed;
///
/// let detailed = version_string_detailed();
/// assert!(detailed.contains("clmd"));
/// ```
pub fn version_string_detailed() -> String {
    let target = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    format!(
        "{} {}\nCompiled for {} on {}",
        PKG_NAME, VERSION, target, os
    )
}

/// Check if the current version satisfies a minimum version requirement.
///
/// # Arguments
///
/// * `major` - The minimum major version.
/// * `minor` - The minimum minor version.
/// * `patch` - The minimum patch version.
///
/// # Returns
///
/// `true` if the current version is greater than or equal to the requirement.
///
/// # Example
///
/// ```ignore
/// use clmd::version::satisfies;
///
/// // Check if version is at least 0.1.0
/// assert!(satisfies(0, 1, 0));
/// ```
#[allow(clippy::absurd_extreme_comparisons)]
pub const fn satisfies(major: u32, minor: u32, patch: u32) -> bool {
    let current = VERSION_MAJOR;
    if current > major {
        return true;
    }
    if current < major {
        return false;
    }

    let current = VERSION_MINOR;
    if current > minor {
        return true;
    }
    if current < minor {
        return false;
    }

    VERSION_PATCH >= patch
}

/// Version information struct for runtime access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    /// Major version number.
    pub major: u32,
    /// Minor version number.
    pub minor: u32,
    /// Patch version number.
    pub patch: u32,
    /// Pre-release version string.
    pub pre: String,
    /// Full version string.
    pub full: String,
}

impl VersionInfo {
    /// Create a new VersionInfo with the current crate version.
    pub fn current() -> Self {
        Self {
            major: VERSION_MAJOR,
            minor: VERSION_MINOR,
            patch: VERSION_PATCH,
            pre: VERSION_PRE.to_string(),
            full: VERSION.to_string(),
        }
    }

    /// Check if this version satisfies a minimum requirement.
    pub fn satisfies(&self, major: u32, minor: u32, patch: u32) -> bool {
        if self.major > major {
            return true;
        }
        if self.major < major {
            return false;
        }
        if self.minor > minor {
            return true;
        }
        if self.minor < minor {
            return false;
        }
        self.patch >= patch
    }
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self::current()
    }
}

impl std::fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre)?;
        }
        Ok(())
    }
}

/// Build information struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildInfo {
    /// Target architecture (e.g., "x86_64", "aarch64").
    pub target_arch: &'static str,
    /// Target operating system (e.g., "linux", "macos", "windows").
    pub target_os: &'static str,
    /// Target family (e.g., "unix", "windows").
    pub target_family: Option<&'static str>,
}

impl BuildInfo {
    /// Create a new BuildInfo with current build information.
    pub const fn current() -> Self {
        Self {
            target_arch: std::env::consts::ARCH,
            target_os: std::env::consts::OS,
            target_family: Some(std::env::consts::FAMILY),
        }
    }
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self::current()
    }
}

impl std::fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} on {}", self.target_arch, self.target_os)?;
        if let Some(family) = self.target_family {
            write!(f, " ({})", family)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        assert!(!VERSION.is_empty());
        // These assertions are always true for unsigned types,
        // but we keep them for documentation purposes
        let _ = VERSION_MAJOR;
        let _ = VERSION_MINOR;
        let _ = VERSION_PATCH;
    }

    #[test]
    fn test_version_string() {
        let s = version_string();
        assert!(s.contains(PKG_NAME));
        assert!(s.contains(&VERSION_MAJOR.to_string()));
    }

    #[test]
    fn test_version_string_detailed() {
        let s = version_string_detailed();
        assert!(s.contains(PKG_NAME));
        assert!(s.contains(std::env::consts::ARCH));
    }

    #[test]
    fn test_satisfies() {
        // Current version should satisfy itself
        assert!(satisfies(VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH));
        // Should satisfy older versions
        assert!(satisfies(0, 0, 0));
    }

    #[test]
    fn test_version_info() {
        let info = VersionInfo::current();
        assert_eq!(info.major, VERSION_MAJOR);
        assert_eq!(info.minor, VERSION_MINOR);
        assert_eq!(info.patch, VERSION_PATCH);
        assert!(info.satisfies(0, 0, 0));
    }

    #[test]
    fn test_version_info_display() {
        let info = VersionInfo::current();
        let s = info.to_string();
        assert!(s.contains(&VERSION_MAJOR.to_string()));
        assert!(s.contains(&VERSION_MINOR.to_string()));
    }

    #[test]
    fn test_build_info() {
        let info = BuildInfo::current();
        assert_eq!(info.target_arch, std::env::consts::ARCH);
        assert_eq!(info.target_os, std::env::consts::OS);
    }

    #[test]
    fn test_parse_u32() {
        assert_eq!(parse_u32("0"), 0);
        assert_eq!(parse_u32("1"), 1);
        assert_eq!(parse_u32("10"), 10);
        assert_eq!(parse_u32("100"), 100);
    }
}
