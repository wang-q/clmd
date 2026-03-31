//! Roff character escaping utilities for clmd.
//!
//! This module provides character escaping for roff/groff format,
//! inspired by Pandoc's Text.Pandoc.RoffChar module.
//!
//! Roff (also known as troff/groff) is the format used for Unix manual pages.
//! Special characters need to be escaped to be properly rendered.
//!
//! # Example
//!
//! ```
//! use clmd::roff_char::{escape_roff, standard_escape};
//!
//! // Escape a string for roff output
//! let escaped = escape_roff("It's a test");
//! assert!(escaped.contains("\\(aq"));
//!
//! // Get escape sequence for a character
//! assert_eq!(standard_escape('\''), Some("\\(aq"));
//! ```

use std::collections::HashMap;

/// Standard roff escapes for special characters.
///
/// These are the escapes specifically mentioned in groff_man(7),
/// plus @ and ellipsis. We use the \(xx form when possible (with
/// two-letter escapes), because these are compatible with all forms
/// of roff.
const STANDARD_ESCAPES: &[(char, &str)] = &[
    ('\u{00A0}', "\\ "),       // Non-breaking space
    ('\'', "\\(aq"),           // Apostrophe quote
    ('\u{2018}', "\\(oq"),     // Left single quotation mark
    ('\u{2019}', "\\(cq"),     // Right single quotation mark
    ('"', "\\(dq"),            // Double quote
    ('\u{201C}', "\\(lq"),     // Left double quotation mark
    ('\u{201D}', "\\(rq"),     // Right double quotation mark
    ('\u{2014}', "\\(em"),     // Em dash
    ('\u{2013}', "\\(en"),     // En dash
    ('`', "\\(ga"),            // Grave accent
    ('^', "\\(ha"),            // Hat (circumflex)
    ('~', "\\(ti"),            // Tilde
    ('\\', "\\(rs"),           // Reverse solidus (backslash)
    ('@', "\\(at"),            // At sign (used as table/math delimiter)
    ('\u{2026}', "\\&..."),     // Ellipsis (u2026 doesn't render on tty)
];

/// Character codes for special characters in roff.
///
/// These are character escape sequences using the \[xx] format.
const CHARACTER_CODES: &[(char, &str)] = &[
    ('Ð', "-D"),      // Capital eth
    ('ð', "Sd"),      // Small eth
    ('Þ', "TP"),      // Capital thorn
    ('þ', "Tp"),      // Small thorn
    ('ß', "ss"),      // German sharp s
    ('\u{FB00}', "ff"),  // Latin small ligature ff
    ('\u{FB01}', "fi"),  // Latin small ligature fi
    ('\u{FB02}', "fl"),  // Latin small ligature fl
    ('\u{FB03}', "Fi"),  // Latin small ligature ffi
    ('\u{FB04}', "Fl"),  // Latin small ligature ffl
    ('Ł', "/L"),      // Capital L with stroke
    ('ł', "/l"),      // Small l with stroke
    ('Ø', "/O"),      // Capital O with stroke
    ('ø', "/o"),      // Small o with stroke
    ('Æ', "AE"),      // Capital AE
    ('æ', "ae"),      // Small ae
    ('Œ', "OE"),      // Capital OE
    ('œ', "oe"),      // Small oe
    ('Ĳ', "IJ"),      // Capital IJ
    ('ĳ', "ij"),      // Small ij
    ('ı', ".i"),      // Dotless i
    ('ȷ', ".j"),      // Dotless j
    // Accented capital letters
    ('Á', "'A"), ('Ć', "'C"), ('É', "'E"), ('Í', "'I"),
    ('Ó', "'O"), ('Ú', "'U"), ('Ý', "'Y"),
    // Accented small letters
    ('á', "'a"), ('ć', "'c"), ('é', "'e"), ('í', "'i"),
    ('ó', "'o"), ('ú', "'u"), ('ý', "'y"),
    // Diaeresis
    ('Ä', ":A"), ('Ë', ":E"), ('Ï', ":I"), ('Ö', ":O"),
    ('Ü', ":U"), ('Ÿ', ":Y"),
    ('ä', ":a"), ('ë', ":e"), ('ï', ":i"), ('ö', ":o"),
    ('ü', ":u"), ('ÿ', ":y"),
    // Circumflex
    ('Â', "^A"), ('Ê', "^E"), ('Î', "^I"), ('Ô', "^O"), ('Û', "^U"),
    ('â', "^a"), ('ê', "^e"), ('î', "^i"), ('ô', "^o"), ('û', "^u"),
    // Grave
    ('À', "`A"), ('È', "`E"), ('Ì', "`I"), ('Ò', "`O"), ('Ù', "`U"),
    ('à', "`a"), ('è', "`e"), ('ì', "`i"), ('ò', "`o"), ('ù', "`u"),
    // Tilde
    ('Ã', "~A"), ('Ñ', "~N"), ('Õ', "~O"),
    ('ã', "~a"), ('ñ', "~n"), ('õ', "~o"),
    // Caron
    ('Š', "vS"), ('š', "vs"), ('Ž', "vZ"), ('ž', "vz"),
    // Cedilla
    ('Ç', ",C"), ('ç', ",c"),
    // Ring
    ('Å', "oA"), ('å', "oa"),
];

/// Get the standard escape sequence for a character.
///
/// Returns `Some("\\(xx")` if the character has a standard escape,
/// or `None` if it doesn't.
///
/// # Example
///
/// ```
/// use clmd::roff_char::standard_escape;
///
/// assert_eq!(standard_escape('\''), Some("\\(aq"));
/// assert_eq!(standard_escape('\\'), Some("\\(rs"));
/// assert_eq!(standard_escape('a'), None);
/// ```
pub fn standard_escape(c: char) -> Option<&'static str> {
    STANDARD_ESCAPES
        .iter()
        .find(|(ch, _)| *ch == c)
        .map(|(_, escape)| *escape)
}

/// Get the character code escape for a character.
///
/// Returns `Some("xx")` if the character has a code escape,
/// or `None` if it doesn't.
///
/// # Example
///
/// ```
/// use clmd::roff_char::character_code;
///
/// assert_eq!(character_code('á'), Some("'a"));
/// assert_eq!(character_code('ö'), Some(":o"));
/// assert_eq!(character_code('z'), None);
/// ```
pub fn character_code(c: char) -> Option<&'static str> {
    CHARACTER_CODES
        .iter()
        .find(|(ch, _)| *ch == c)
        .map(|(_, code)| *code)
}

/// Escape a string for roff output.
///
/// This function replaces special characters with their roff escape sequences.
/// Characters that don't have special escapes are left as-is.
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// The escaped string
///
/// # Example
///
/// ```
/// use clmd::roff_char::escape_roff;
///
/// let escaped = escape_roff("It's a test");
/// assert!(escaped.contains("\\(aq"));
///
/// let escaped = escape_roff("Hello — World");
/// assert!(escaped.contains("\\(em"));
/// ```
pub fn escape_roff(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);

    for c in s.chars() {
        if let Some(escape) = standard_escape(c) {
            result.push_str(escape);
        } else {
            result.push(c);
        }
    }

    result
}

/// Escape a string using character codes.
///
/// This function uses \[xx] format escapes for special characters.
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// The escaped string with character codes
///
/// # Example
///
/// ```
/// use clmd::roff_char::escape_roff_with_codes;
///
/// let escaped = escape_roff_with_codes("café");
/// assert!(escaped.contains("\\['e]"));
/// ```
pub fn escape_roff_with_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);

    for c in s.chars() {
        if let Some(code) = character_code(c) {
            result.push_str("\\[");
            result.push_str(code);
            result.push(']');
        } else if let Some(escape) = standard_escape(c) {
            result.push_str(escape);
        } else {
            result.push(c);
        }
    }

    result
}

/// Check if a character needs to be escaped in roff.
///
/// # Example
///
/// ```
/// use clmd::roff_char::needs_escape;
///
/// assert!(needs_escape('\''));
/// assert!(needs_escape('\\'));
/// assert!(!needs_escape('a'));
/// ```
pub fn needs_escape(c: char) -> bool {
    standard_escape(c).is_some() || character_code(c).is_some()
}

/// A roff escaper that can be configured with custom escapes.
#[derive(Debug, Clone)]
pub struct RoffEscaper {
    custom_escapes: HashMap<char, String>,
    use_character_codes: bool,
}

impl RoffEscaper {
    /// Create a new escaper with default settings.
    pub fn new() -> Self {
        Self {
            custom_escapes: HashMap::new(),
            use_character_codes: false,
        }
    }

    /// Create a new escaper that uses character codes.
    pub fn with_character_codes() -> Self {
        Self {
            custom_escapes: HashMap::new(),
            use_character_codes: true,
        }
    }

    /// Add a custom escape sequence.
    pub fn add_escape(&mut self, c: char, escape: impl Into<String>) {
        self.custom_escapes.insert(c, escape.into());
    }

    /// Set whether to use character codes.
    pub fn set_use_character_codes(&mut self, use_codes: bool) {
        self.use_character_codes = use_codes;
    }

    /// Escape a string.
    pub fn escape(&self, s: &str) -> String {
        let mut result = String::with_capacity(s.len() * 2);

        for c in s.chars() {
            // Check custom escapes first
            if let Some(escape) = self.custom_escapes.get(&c) {
                result.push_str(escape);
                continue;
            }

            // Then check standard escapes
            if let Some(escape) = standard_escape(c) {
                result.push_str(escape);
                continue;
            }

            // Then check character codes if enabled
            if self.use_character_codes {
                if let Some(code) = character_code(c) {
                    result.push_str("\\[");
                    result.push_str(code);
                    result.push(']');
                    continue;
                }
            }

            // Otherwise, just add the character
            result.push(c);
        }

        result
    }
}

impl Default for RoffEscaper {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape a string for use in man page headers/footers.
///
/// This is a specialized escape function for man page headers and footers,
/// which have additional restrictions.
///
/// # Example
///
/// ```
/// use clmd::roff_char::escape_header;
///
/// let escaped = escape_header("Section 1");
/// assert_eq!(escaped, "Section 1");
/// ```
pub fn escape_header(s: &str) -> String {
    // Headers have fewer restrictions, but we still need to escape
    // certain characters
    s.replace("\\", "\\(rs")
        .replace("'", "\\(aq")
        .replace("-", "\\-")
}

/// Get all standard escapes as a map.
///
/// This is useful for building custom escapers.
///
/// # Example
///
/// ```
/// use clmd::roff_char::standard_escapes_map;
///
/// let map = standard_escapes_map();
/// assert!(map.contains_key(&'\''));
/// ```
pub fn standard_escapes_map() -> HashMap<char, &'static str> {
    STANDARD_ESCAPES.iter().map(|(c, e)| (*c, *e)).collect()
}

/// Get all character codes as a map.
///
/// # Example
///
/// ```
/// use clmd::roff_char::character_codes_map;
///
/// let map = character_codes_map();
/// assert!(map.contains_key(&'á'));
/// ```
pub fn character_codes_map() -> HashMap<char, &'static str> {
    CHARACTER_CODES.iter().map(|(c, e)| (*c, *e)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_escape() {
        assert_eq!(standard_escape('\''), Some("\\(aq"));
        assert_eq!(standard_escape('\\'), Some("\\(rs"));
        assert_eq!(standard_escape('@'), Some("\\(at"));
        assert_eq!(standard_escape('a'), None);
    }

    #[test]
    fn test_character_code() {
        assert_eq!(character_code('á'), Some("'a"));
        assert_eq!(character_code('ö'), Some(":o"));
        assert_eq!(character_code('Æ'), Some("AE"));
        assert_eq!(character_code('z'), None);
    }

    #[test]
    fn test_escape_roff() {
        let escaped = escape_roff("It's a test");
        assert!(escaped.contains("\\(aq"));
        assert!(!escaped.contains("'"));

        let escaped = escape_roff("Hello — World");
        assert!(escaped.contains("\\(em"));

        let escaped = escape_roff("Path: C:\\Users");
        assert!(escaped.contains("\\(rs"));
    }

    #[test]
    fn test_escape_roff_with_codes() {
        let escaped = escape_roff_with_codes("café");
        assert!(escaped.contains("\\['e]"));

        let escaped = escape_roff_with_codes("naïve");
        assert!(escaped.contains("\\[:i]"));
    }

    #[test]
    fn test_needs_escape() {
        assert!(needs_escape('\''));
        assert!(needs_escape('\\'));
        assert!(needs_escape('á'));
        assert!(!needs_escape('a'));
        assert!(!needs_escape('1'));
    }

    #[test]
    fn test_roff_escaper() {
        let escaper = RoffEscaper::new();
        let escaped = escaper.escape("It's a test");
        assert!(escaped.contains("\\(aq"));

        let mut escaper = RoffEscaper::with_character_codes();
        let escaped = escaper.escape("café");
        assert!(escaped.contains("\\['e]"));
    }

    #[test]
    fn test_custom_escape() {
        let mut escaper = RoffEscaper::new();
        escaper.add_escape('★', "\\(st");

        let escaped = escaper.escape("★ Star");
        assert!(escaped.contains("\\(st"));
    }

    #[test]
    fn test_escape_header() {
        let escaped = escape_header("Section 1");
        assert_eq!(escaped, "Section 1");

        let escaped = escape_header("It's here");
        assert!(escaped.contains("\\(aq"));
    }

    #[test]
    fn test_standard_escapes_map() {
        let map = standard_escapes_map();
        assert!(map.contains_key(&'\''));
        assert!(map.contains_key(&'\\'));
        assert_eq!(map.get(&'\''), Some(&"\\(aq"));
    }

    #[test]
    fn test_character_codes_map() {
        let map = character_codes_map();
        assert!(map.contains_key(&'á'));
        assert!(map.contains_key(&'ö'));
    }

    #[test]
    fn test_unicode_escapes() {
        // Test smart quotes
        assert_eq!(standard_escape('\u{2018}'), Some("\\(oq"));
        assert_eq!(standard_escape('\u{2019}'), Some("\\(cq"));
        assert_eq!(standard_escape('\u{201C}'), Some("\\(lq"));
        assert_eq!(standard_escape('\u{201D}'), Some("\\(rq"));

        // Test dashes
        assert_eq!(standard_escape('\u{2014}'), Some("\\(em"));
        assert_eq!(standard_escape('\u{2013}'), Some("\\(en"));

        // Test ellipsis
        assert_eq!(standard_escape('\u{2026}'), Some("\\&..."));
    }

    #[test]
    fn test_no_escape_for_ascii() {
        // Most ASCII characters don't need escaping
        for c in 'a'..='z' {
            assert!(!needs_escape(c));
        }
        for c in 'A'..='Z' {
            assert!(!needs_escape(c));
        }
        for c in '0'..='9' {
            assert!(!needs_escape(c));
        }
    }
}
