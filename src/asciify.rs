//! ASCII transliteration utilities for clmd.
//!
//! This module provides ASCII transliteration capabilities, inspired by Pandoc's
//! asciify functionality. It converts Unicode characters to their ASCII equivalents.
//!
//! # Example
//!
//! ```
//! use clmd::asciify::asciify;
//!
//! let ascii = asciify("café résumé naïve");
//! assert_eq!(ascii, "cafe resume naive");
//! ```

use std::collections::HashMap;

/// A transliterator for converting Unicode to ASCII.
#[derive(Debug, Clone)]
pub struct Transliterator {
    mappings: HashMap<char, String>,
}

impl Default for Transliterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Transliterator {
    /// Create a new transliterator with built-in mappings.
    pub fn new() -> Self {
        let mut mappings = HashMap::new();
        Self::populate_latin_characters(&mut mappings);
        Self { mappings }
    }

    /// Create an empty transliterator.
    pub fn empty() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Add a custom character mapping.
    pub fn add_mapping(&mut self, from: char, to: impl Into<String>) {
        self.mappings.insert(from, to.into());
    }

    /// Transliterate a string to ASCII.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::asciify::Transliterator;
    ///
    /// let transliterator = Transliterator::new();
    /// let result = transliterator.transliterate("café");
    /// assert_eq!(result, "cafe");
    /// ```
    pub fn transliterate(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());

        for ch in input.chars() {
            if ch.is_ascii() {
                result.push(ch);
            } else if let Some(replacement) = self.mappings.get(&ch) {
                result.push_str(replacement);
            } else {
                // Skip unknown characters
            }
        }

        result
    }

    /// Check if a string contains non-ASCII characters.
    pub fn has_non_ascii(&self, input: &str) -> bool {
        input.chars().any(|ch| !ch.is_ascii())
    }

    /// Get the number of mappings.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Check if the transliterator is empty.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Populate Latin character mappings.
    fn populate_latin_characters(mappings: &mut HashMap<char, String>) {
        // Accented vowels - lowercase
        mappings.insert('á', "a".to_string());
        mappings.insert('à', "a".to_string());
        mappings.insert('â', "a".to_string());
        mappings.insert('ä', "ae".to_string());
        mappings.insert('ã', "a".to_string());
        mappings.insert('å', "a".to_string());
        mappings.insert('ā', "a".to_string());
        mappings.insert('ă', "a".to_string());
        mappings.insert('ą', "a".to_string());

        mappings.insert('é', "e".to_string());
        mappings.insert('è', "e".to_string());
        mappings.insert('ê', "e".to_string());
        mappings.insert('ë', "e".to_string());
        mappings.insert('ē', "e".to_string());
        mappings.insert('ĕ', "e".to_string());
        mappings.insert('ė', "e".to_string());
        mappings.insert('ę', "e".to_string());

        mappings.insert('í', "i".to_string());
        mappings.insert('ì', "i".to_string());
        mappings.insert('î', "i".to_string());
        mappings.insert('ï', "i".to_string());
        mappings.insert('ī', "i".to_string());
        mappings.insert('ĭ', "i".to_string());
        mappings.insert('į', "i".to_string());

        mappings.insert('ó', "o".to_string());
        mappings.insert('ò', "o".to_string());
        mappings.insert('ô', "o".to_string());
        mappings.insert('ö', "oe".to_string());
        mappings.insert('õ', "o".to_string());
        mappings.insert('ø', "o".to_string());
        mappings.insert('ō', "o".to_string());
        mappings.insert('ŏ', "o".to_string());
        mappings.insert('ő', "o".to_string());

        mappings.insert('ú', "u".to_string());
        mappings.insert('ù', "u".to_string());
        mappings.insert('û', "u".to_string());
        mappings.insert('ü', "ue".to_string());
        mappings.insert('ū', "u".to_string());
        mappings.insert('ŭ', "u".to_string());
        mappings.insert('ů', "u".to_string());
        mappings.insert('ű', "u".to_string());
        mappings.insert('ų', "u".to_string());

        mappings.insert('ý', "y".to_string());
        mappings.insert('ÿ', "y".to_string());
        mappings.insert('ŷ', "y".to_string());

        // Accented vowels - uppercase
        mappings.insert('Á', "A".to_string());
        mappings.insert('À', "A".to_string());
        mappings.insert('Â', "A".to_string());
        mappings.insert('Ä', "AE".to_string());
        mappings.insert('Ã', "A".to_string());
        mappings.insert('Å', "A".to_string());
        mappings.insert('Ā', "A".to_string());

        mappings.insert('É', "E".to_string());
        mappings.insert('È', "E".to_string());
        mappings.insert('Ê', "E".to_string());
        mappings.insert('Ë', "E".to_string());
        mappings.insert('Ē', "E".to_string());

        mappings.insert('Í', "I".to_string());
        mappings.insert('Ì', "I".to_string());
        mappings.insert('Î', "I".to_string());
        mappings.insert('Ï', "I".to_string());
        mappings.insert('Ī', "I".to_string());

        mappings.insert('Ó', "O".to_string());
        mappings.insert('Ò', "O".to_string());
        mappings.insert('Ô', "O".to_string());
        mappings.insert('Ö', "OE".to_string());
        mappings.insert('Õ', "O".to_string());
        mappings.insert('Ø', "O".to_string());
        mappings.insert('Ō', "O".to_string());

        mappings.insert('Ú', "U".to_string());
        mappings.insert('Ù', "U".to_string());
        mappings.insert('Û', "U".to_string());
        mappings.insert('Ü', "UE".to_string());
        mappings.insert('Ū', "U".to_string());

        mappings.insert('Ý', "Y".to_string());
        mappings.insert('Ÿ', "Y".to_string());

        // Accented consonants
        mappings.insert('ç', "c".to_string());
        mappings.insert('ć', "c".to_string());
        mappings.insert('ĉ', "c".to_string());
        mappings.insert('ċ', "c".to_string());
        mappings.insert('č', "c".to_string());

        mappings.insert('Ç', "C".to_string());
        mappings.insert('Ć', "C".to_string());
        mappings.insert('Ĉ', "C".to_string());
        mappings.insert('Ċ', "C".to_string());
        mappings.insert('Č', "C".to_string());

        mappings.insert('ñ', "n".to_string());
        mappings.insert('ń', "n".to_string());
        mappings.insert('ņ', "n".to_string());
        mappings.insert('ň', "n".to_string());

        mappings.insert('Ñ', "N".to_string());
        mappings.insert('Ń', "N".to_string());
        mappings.insert('Ņ', "N".to_string());
        mappings.insert('Ň', "N".to_string());

        mappings.insert('ś', "s".to_string());
        mappings.insert('ŝ', "s".to_string());
        mappings.insert('ş', "s".to_string());
        mappings.insert('š', "s".to_string());
        mappings.insert('ß', "ss".to_string());

        mappings.insert('Ś', "S".to_string());
        mappings.insert('Ŝ', "S".to_string());
        mappings.insert('Ş', "S".to_string());
        mappings.insert('Š', "S".to_string());

        mappings.insert('ź', "z".to_string());
        mappings.insert('ż', "z".to_string());
        mappings.insert('ž', "z".to_string());

        mappings.insert('Ź', "Z".to_string());
        mappings.insert('Ż', "Z".to_string());
        mappings.insert('Ž', "Z".to_string());

        mappings.insert('ł', "l".to_string());
        mappings.insert('Ł', "L".to_string());

        // Currency symbols
        mappings.insert('€', "EUR".to_string());
        mappings.insert('£', "GBP".to_string());
        mappings.insert('¥', "JPY".to_string());
        mappings.insert('¢', "c".to_string());

        // Punctuation using Unicode escape sequences
        mappings.insert('«', "<<".to_string());
        mappings.insert('»', ">>".to_string());
        // Double low-9 quotation mark
        mappings.insert('\u{201E}', "\"".to_string());
        // Left double quotation mark
        mappings.insert('\u{201C}', "\"".to_string());
        // Right double quotation mark
        mappings.insert('\u{201D}', "\"".to_string());
        // Left single quotation mark
        mappings.insert('\u{2018}', "'".to_string());
        // Right single quotation mark
        mappings.insert('\u{2019}', "'".to_string());
        // En dash
        mappings.insert('\u{2013}', "-".to_string());
        // Em dash
        mappings.insert('\u{2014}', "--".to_string());
        // Horizontal ellipsis
        mappings.insert('\u{2026}', "...".to_string());

        // Math symbols
        mappings.insert('×', "x".to_string());
        mappings.insert('÷', "/".to_string());
        mappings.insert('±', "+/-".to_string());
        mappings.insert('≠', "!=".to_string());
        mappings.insert('≤', "<=".to_string());
        mappings.insert('≥', ">=".to_string());
        mappings.insert('∞', "infinity".to_string());
        mappings.insert('∑', "sum".to_string());
        mappings.insert('∏', "product".to_string());
        mappings.insert('∫', "integral".to_string());
        mappings.insert('√', "sqrt".to_string());
        mappings.insert('∂', "d".to_string());
        mappings.insert('∆', "delta".to_string());
        mappings.insert('∇', "nabla".to_string());
        mappings.insert('∈', "in".to_string());
        mappings.insert('∉', "not in".to_string());
        mappings.insert('∋', "contains".to_string());

        // Greek letters - lowercase
        mappings.insert('α', "alpha".to_string());
        mappings.insert('β', "beta".to_string());
        mappings.insert('γ', "gamma".to_string());
        mappings.insert('δ', "delta".to_string());
        mappings.insert('ε', "epsilon".to_string());
        mappings.insert('ζ', "zeta".to_string());
        mappings.insert('η', "eta".to_string());
        mappings.insert('θ', "theta".to_string());
        mappings.insert('ι', "iota".to_string());
        mappings.insert('κ', "kappa".to_string());
        mappings.insert('λ', "lambda".to_string());
        mappings.insert('μ', "mu".to_string());
        mappings.insert('ν', "nu".to_string());
        mappings.insert('ξ', "xi".to_string());
        mappings.insert('ο', "omicron".to_string());
        mappings.insert('π', "pi".to_string());
        mappings.insert('ρ', "rho".to_string());
        mappings.insert('σ', "sigma".to_string());
        mappings.insert('τ', "tau".to_string());
        mappings.insert('υ', "upsilon".to_string());
        mappings.insert('φ', "phi".to_string());
        mappings.insert('χ', "chi".to_string());
        mappings.insert('ψ', "psi".to_string());
        mappings.insert('ω', "omega".to_string());

        // Greek letters - uppercase
        mappings.insert('Α', "Alpha".to_string());
        mappings.insert('Β', "Beta".to_string());
        mappings.insert('Γ', "Gamma".to_string());
        mappings.insert('Δ', "Delta".to_string());
        mappings.insert('Ε', "Epsilon".to_string());
        mappings.insert('Ζ', "Zeta".to_string());
        mappings.insert('Η', "Eta".to_string());
        mappings.insert('Θ', "Theta".to_string());
        mappings.insert('Ι', "Iota".to_string());
        mappings.insert('Κ', "Kappa".to_string());
        mappings.insert('Λ', "Lambda".to_string());
        mappings.insert('Μ', "Mu".to_string());
        mappings.insert('Ν', "Nu".to_string());
        mappings.insert('Ξ', "Xi".to_string());
        mappings.insert('Ο', "Omicron".to_string());
        mappings.insert('Π', "Pi".to_string());
        mappings.insert('Ρ', "Rho".to_string());
        mappings.insert('Σ', "Sigma".to_string());
        mappings.insert('Τ', "Tau".to_string());
        mappings.insert('Υ', "Upsilon".to_string());
        mappings.insert('Φ', "Phi".to_string());
        mappings.insert('Χ', "Chi".to_string());
        mappings.insert('Ψ', "Psi".to_string());
        mappings.insert('Ω', "Omega".to_string());
    }
}

/// Transliterate a string to ASCII using the default transliterator.
///
/// This is a convenience function for simple use cases.
///
/// # Example
///
/// ```ignore
/// use clmd::asciify::asciify;
///
/// let ascii = asciify("café résumé naïve");
/// assert_eq!(ascii, "cafe resume naive");
/// ```ignore
pub fn asciify(input: &str) -> String {
    let transliterator = Transliterator::new();
    transliterator.transliterate(input)
}

/// Check if a string contains non-ASCII characters.
///
/// # Example
///
/// ```ignore
/// use clmd::asciify::has_non_ascii;
///
/// assert!(has_non_ascii("café"));
/// assert!(!has_non_ascii("cafe"));
/// ```ignore
pub fn has_non_ascii(input: &str) -> bool {
    input.chars().any(|ch| !ch.is_ascii())
}

/// Create a slug from a string (ASCII-only, lowercase, hyphenated).
///
/// # Example
///
/// ```ignore
/// use clmd::asciify::slugify;
///
/// let slug = slugify("Hello World!");
/// assert_eq!(slug, "hello-world");
/// ```ignore
pub fn slugify(input: &str) -> String {
    let transliterator = Transliterator::new();
    let ascii = transliterator.transliterate(input);

    ascii
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asciify() {
        assert_eq!(asciify("café"), "cafe");
        assert_eq!(asciify("résumé"), "resume");
        assert_eq!(asciify("naïve"), "naive");
    }

    #[test]
    fn test_transliterator_accented_vowels() {
        let t = Transliterator::new();
        assert_eq!(t.transliterate("áàâäãå"), "aaaaeaa");
        assert_eq!(t.transliterate("éèêë"), "eeee");
        assert_eq!(t.transliterate("íìîï"), "iiii");
        assert_eq!(t.transliterate("óòôöõø"), "ooooeoo");
        assert_eq!(t.transliterate("úùûü"), "uuuue");
    }

    #[test]
    fn test_transliterator_german() {
        let t = Transliterator::new();
        assert_eq!(t.transliterate("Größe"), "Groesse");
        assert_eq!(t.transliterate("Über"), "UEber");
    }

    #[test]
    fn test_transliterator_currency() {
        let t = Transliterator::new();
        assert_eq!(t.transliterate("€100"), "EUR100");
        assert_eq!(t.transliterate("£50"), "GBP50");
    }

    #[test]
    fn test_transliterator_greek() {
        let t = Transliterator::new();
        assert_eq!(t.transliterate("αβγ"), "alphabetagamma");
        assert_eq!(t.transliterate("Δ"), "Delta");
    }

    #[test]
    fn test_has_non_ascii() {
        assert!(has_non_ascii("café"));
        assert!(!has_non_ascii("cafe"));
        assert!(has_non_ascii("日本語"));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("café résumé"), "cafe-resume");
        assert_eq!(slugify("Multiple   Spaces"), "multiple-spaces");
        assert_eq!(slugify("Special!@#Chars"), "special-chars");
    }

    #[test]
    fn test_empty_transliterator() {
        let t = Transliterator::empty();
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn test_add_custom_mapping() {
        let mut t = Transliterator::empty();
        t.add_mapping('\u{2605}', "star");
        assert_eq!(t.transliterate("\u{2605}\u{2605}\u{2605}"), "starstarstar");
    }

    #[test]
    fn test_ascii_passthrough() {
        let t = Transliterator::new();
        assert_eq!(t.transliterate("Hello World 123"), "Hello World 123");
    }

    #[test]
    fn test_punctuation() {
        let t = Transliterator::new();
        assert_eq!(t.transliterate("Hello\u{2014}World"), "Hello--World");
        assert_eq!(t.transliterate("«quote»"), "<<quote>>");
    }
}
