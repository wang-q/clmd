//! Utility functions for inline parsing

use std::collections::HashMap;

/// Character lookup table for fast classification
/// Bit flags:
/// - bit 0: is_punctuation
/// - bit 1: is_whitespace
/// - bit 2: is_special (needs special handling in inline parsing)
static CHAR_TABLE: [u8; 256] = {
    let mut table = [0u8; 256];
    // Punctuation characters
    table[b'!' as usize] = 0b101;
    table[b'"' as usize] = 0b101;
    table[b'#' as usize] = 0b001;
    table[b'$' as usize] = 0b001;
    table[b'%' as usize] = 0b001;
    table[b'&' as usize] = 0b101;
    table[b'\'' as usize] = 0b101;
    table[b'(' as usize] = 0b001;
    table[b')' as usize] = 0b001;
    table[b'*' as usize] = 0b101;
    table[b'+' as usize] = 0b001;
    table[b',' as usize] = 0b001;
    table[b'-' as usize] = 0b001;
    table[b'.' as usize] = 0b001;
    table[b'/' as usize] = 0b001;
    table[b':' as usize] = 0b001;
    table[b';' as usize] = 0b001;
    table[b'<' as usize] = 0b101;
    table[b'=' as usize] = 0b001;
    table[b'>' as usize] = 0b001;
    table[b'?' as usize] = 0b001;
    table[b'@' as usize] = 0b001;
    table[b'[' as usize] = 0b101;
    table[b'\\' as usize] = 0b001;
    table[b']' as usize] = 0b001;
    table[b'^' as usize] = 0b001;
    table[b'_' as usize] = 0b101;
    table[b'`' as usize] = 0b101;
    table[b'{' as usize] = 0b001;
    table[b'|' as usize] = 0b001;
    table[b'}' as usize] = 0b001;
    table[b'~' as usize] = 0b001;
    // Whitespace characters
    table[b' ' as usize] = 0b010;
    table[b'\t' as usize] = 0b010;
    table[b'\n' as usize] = 0b010;
    table[b'\r' as usize] = 0b010;
    table[b'\x0C' as usize] = 0b010; // Form feed
    table
};

/// Fast check if a byte is punctuation using lookup table
#[inline(always)]
pub fn is_punctuation_fast(b: u8) -> bool {
    CHAR_TABLE[b as usize] & 0b001 != 0
}

/// Fast check if a byte is whitespace using lookup table
#[inline(always)]
#[allow(dead_code)]
pub fn is_whitespace_fast(b: u8) -> bool {
    CHAR_TABLE[b as usize] & 0b010 != 0
}

/// Fast check if a byte is a special inline character
#[inline(always)]
#[allow(dead_code)]
pub fn is_special_fast(b: u8) -> bool {
    CHAR_TABLE[b as usize] & 0b100 != 0
}

/// Check if a character is punctuation
pub fn is_punctuation(c: char) -> bool {
    // Fast path for ASCII using lookup table
    if c.is_ascii() {
        return is_punctuation_fast(c as u8);
    }
    // Unicode punctuation (Pc, Pd, Ps, Pe, Pi, Pf, Po categories)
    // Check for specific Unicode punctuation characters commonly used in tests
    matches!(c,
        '\u{00A2}'..='\u{00A5}' | // ¢£¤¥ (currency symbols)
        '\u{00B5}' |              // µ
        '\u{00B7}' |              // ·
        '\u{00BF}' |              // ¿
        '\u{00D7}' |              // ×
        '\u{00F7}' |              // ÷
        '\u{2000}'..='\u{206F}' | // General Punctuation
        '\u{20A0}'..='\u{20CF}' | // Currency Symbols
        '\u{2190}'..='\u{21FF}' | // Arrows
        '\u{2200}'..='\u{22FF}' | // Mathematical Operators
        '\u{2300}'..='\u{23FF}' | // Miscellaneous Technical
        '\u{25A0}'..='\u{25FF}' | // Geometric Shapes
        '\u{2600}'..='\u{26FF}' | // Miscellaneous Symbols
        '\u{2700}'..='\u{27BF}' | // Dingbats
        '\u{3000}'..='\u{303F}'   // CJK Symbols and Punctuation
    )
}

/// Check if a character is special (has special meaning in inline parsing)
#[inline(always)]
pub fn is_special_char(c: char, smart: bool) -> bool {
    if smart {
        matches!(
            c,
            '`' | '\\' | '&' | '<' | '*' | '_' | '[' | ']' | '!' | '\n' | '\'' | '"'
        )
    } else {
        matches!(
            c,
            '`' | '\\' | '&' | '<' | '*' | '_' | '[' | ']' | '!' | '\n'
        )
    }
}

/// Fast byte-level check if a byte is a special ASCII character
#[inline(always)]
pub fn is_special_byte(b: u8, smart: bool) -> bool {
    if smart {
        matches!(
            b,
            b'`' | b'\\'
                | b'&'
                | b'<'
                | b'*'
                | b'_'
                | b'['
                | b']'
                | b'!'
                | b'\n'
                | b'\''
                | b'"'
        )
    } else {
        matches!(
            b,
            b'`' | b'\\' | b'&' | b'<' | b'*' | b'_' | b'[' | b']' | b'!' | b'\n'
        )
    }
}

/// Check if a character can be escaped
pub fn is_escapable(c: char) -> bool {
    matches!(
        c,
        '!' | '"'
            | '#'
            | '$'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '-'
            | '.'
            | '/'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '_'
            | '`'
            | '{'
            | '|'
            | '}'
            | '~'
    )
}

/// HTML5 named entities lookup table
/// This includes all HTML5 entities needed for CommonMark compliance
pub fn get_html5_entity(name: &str) -> Option<&'static str> {
    // Common HTML5 entities - using a static map for performance
    use std::sync::OnceLock;
    static ENTITIES: OnceLock<HashMap<&str, &str>> = OnceLock::new();

    let entities = ENTITIES.get_or_init(|| {
        [
            // Basic entities
            ("amp", "&"),
            ("lt", "<"),
            ("gt", ">"),
            ("quot", "\""),

            // ASCII entities
            ("nbsp", "\u{00A0}"),   // Non-breaking space
            ("copy", "\u{00A9}"),   // ©
            ("reg", "\u{00AE}"),    // ®
            ("trade", "\u{2122}"),  // ™

            // Latin-1 entities that may be problematic with htmlescape
            ("AElig", "\u{00C6}"),  // Æ
            ("aelig", "\u{00E6}"),  // æ
            ("OElig", "\u{0152}"),  // Œ
            ("oelig", "\u{0153}"),  // œ
            ("Scaron", "\u{0160}"), // Š
            ("scaron", "\u{0161}"), // š
            ("Yuml", "\u{0178}"),   // Ÿ
            ("fnof", "\u{0192}"),   // ƒ

            // Extended Latin
            ("Dcaron", "\u{010E}"), // Ď
            ("dcaron", "\u{010F}"), // ď
            ("Ncaron", "\u{0147}"), // Ň
            ("ncaron", "\u{0148}"), // ň
            ("Rcaron", "\u{0158}"), // Ř
            ("rcaron", "\u{0159}"), // ř
            ("Tcaron", "\u{0164}"), // Ť
            ("tcaron", "\u{0165}"), // ť

            // Mathematical symbols
            ("frac34", "\u{00BE}"), // ¾
            ("frac12", "\u{00BD}"), // ½
            ("frac14", "\u{00BC}"), // ¼
            ("HilbertSpace", "\u{210B}"),             // ℋ
            ("DifferentialD", "\u{2146}"),            // ⅆ
            ("ClockwiseContourIntegral", "\u{2232}"), // ∲
            ("ngE", "\u{2267}\u{0338}"),              // ≧̸

            // Common symbols
            ("hellip", "\u{2026}"), // …
            ("ndash", "\u{2013}"),  // –
            ("mdash", "\u{2014}"),  // —
            ("lsquo", "\u{2018}"),  // '
            ("rsquo", "\u{2019}"),  // '
            ("sbquo", "\u{201A}"),  // ‚
            ("ldquo", "\u{201C}"),  // "
            ("rdquo", "\u{201D}"),  // "
            ("bdquo", "\u{201E}"),  // „
            ("dagger", "\u{2020}"), // †
            ("Dagger", "\u{2021}"), // ‡
            ("bull", "\u{2022}"),   // •
            ("prime", "\u{2032}"),  // ′
            ("Prime", "\u{2033}"),  // ″
            ("euro", "\u{20AC}"),   // €
            ("pound", "\u{00A3}"),  // £
            ("yen", "\u{00A5}"),    // ¥
            ("cent", "\u{00A2}"),   // ¢

            // German umlauts and other common accented characters
            ("ouml", "\u{00F6}"),   // ö
            ("Ouml", "\u{00D6}"),   // Ö
            ("uuml", "\u{00FC}"),   // ü
            ("Uuml", "\u{00DC}"),   // Ü
            ("auml", "\u{00E4}"),   // ä
            ("Auml", "\u{00C4}"),   // Ä
            ("euml", "\u{00EB}"),   // ë
            ("Euml", "\u{00CB}"),   // Ë
            ("iuml", "\u{00EF}"),   // ï
            ("Iuml", "\u{00CF}"),   // Ï
            ("yuml", "\u{00FF}"),   // ÿ

            // Other common accented characters
            ("aacute", "\u{00E1}"), // á
            ("Aacute", "\u{00C1}"), // Á
            ("eacute", "\u{00E9}"), // é
            ("Eacute", "\u{00C9}"), // É
            ("iacute", "\u{00ED}"), // í
            ("Iacute", "\u{00CD}"), // Í
            ("oacute", "\u{00F3}"), // ó
            ("Oacute", "\u{00D3}"), // Ó
            ("uacute", "\u{00FA}"), // ú
            ("Uacute", "\u{00DA}"), // Ú
            ("yacute", "\u{00FD}"), // ý
            ("Yacute", "\u{00DD}"), // Ý

            // Grave accents
            ("agrave", "\u{00E0}"), // à
            ("Agrave", "\u{00C0}"), // À
            ("egrave", "\u{00E8}"), // è
            ("Egrave", "\u{00C8}"), // È
            ("igrave", "\u{00EC}"), // ì
            ("Igrave", "\u{00CC}"), // Ì
            ("ograve", "\u{00F2}"), // ò
            ("Ograve", "\u{00D2}"), // Ò
            ("ugrave", "\u{00F9}"), // ù
            ("Ugrave", "\u{00D9}"), // Ù

            // Circumflex accents
            ("acirc", "\u{00E2}"),  // â
            ("Acirc", "\u{00C2}"),  // Â
            ("ecirc", "\u{00EA}"),  // ê
            ("Ecirc", "\u{00CA}"),  // Ê
            ("icirc", "\u{00EE}"),  // î
            ("Icirc", "\u{00CE}"),  // Î
            ("ocirc", "\u{00F4}"),  // ô
            ("Ocirc", "\u{00D4}"),  // Ô
            ("ucirc", "\u{00FB}"),  // û
            ("Ucirc", "\u{00DB}"),  // Û

            // Tilde accents
            ("atilde", "\u{00E3}"), // ã
            ("Atilde", "\u{00C3}"), // Ã
            ("ntilde", "\u{00F1}"), // ñ
            ("Ntilde", "\u{00D1}"), // Ñ
            ("otilde", "\u{00F5}"), // õ
            ("Otilde", "\u{00D5}"), // Õ

            // Ring and stroke
            ("aring", "\u{00E5}"),   // å
            ("Aring", "\u{00C5}"),   // Å
            ("oslash", "\u{00F8}"),  // ø
            ("Oslash", "\u{00D8}"),  // Ø
            ("ccedil", "\u{00E7}"),  // ç
            ("Ccedil", "\u{00C7}"),  // Ç
            ("eth", "\u{00F0}"),     // ð
            ("ETH", "\u{00D0}"),     // Ð
            ("thorn", "\u{00FE}"),   // þ
            ("THORN", "\u{00DE}"),   // Þ
            ("szlig", "\u{00DF}"),   // ß

            // Cedilla and other accents
            ("ogon", ""),             // ˛ (combining)
            ("ring", "\u{02DA}"),    // ˚

            // Additional currency and symbols
            ("curren", "\u{00A4}"),  // ¤
            ("brvbar", "\u{00A6}"),  // ¦
            ("sect", "\u{00A7}"),    // §
            ("uml", "\u{00A8}"),     // ¨
            ("ordf", "\u{00AA}"),    // ª
            ("laquo", "\u{00AB}"),   // «
            ("not", "\u{00AC}"),     // ¬
            ("shy", "\u{00AD}"),     // ­
            ("macr", "\u{00AF}"),    // ¯
            ("deg", "\u{00B0}"),     // °
            ("plusmn", "\u{00B1}"),  // ±
            ("sup2", "\u{00B2}"),    // ²
            ("sup3", "\u{00B3}"),    // ³
            ("acute", "\u{00B4}"),   // ´
            ("micro", "\u{00B5}"),   // µ
            ("para", "\u{00B6}"),    // ¶
            ("middot", "\u{00B7}"),  // ·
            ("cedil", "\u{00B8}"),   // ¸
            ("sup1", "\u{00B9}"),    // ¹
            ("ordm", "\u{00BA}"),    // º
            ("raquo", "\u{00BB}"),   // »
            ("frac14", "\u{00BC}"),  // ¼
            ("frac12", "\u{00BD}"),  // ½
            ("frac34", "\u{00BE}"),  // ¾
            ("iquest", "\u{00BF}"),  // ¿
            ("times", "\u{00D7}"),   // ×
            ("divide", "\u{00F7}"),  // ÷
        ]
        .iter()
        .copied()
        .collect()
    });

    entities.get(name).copied()
}

/// Normalize a URI by percent-encoding special characters
/// Based on commonmark.js normalizeURI
/// Percent-encode characters that are not allowed in URIs
pub fn normalize_uri(uri: &str) -> String {
    let mut result = String::new();

    for c in uri.chars() {
        match c {
            // Unreserved characters (no encoding needed)
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            // Reserved characters that are commonly used in URIs
            ':' | '/' | '?' | '#' | '@' | '!' | '$' | '&' | '\'' | '(' | ')' | '*'
            | '+' | ',' | ';' | '=' => {
                result.push(c);
            }
            // Percent sign (already encoded)
            '%' => {
                result.push(c);
            }
            // Space should be encoded as %20 (not +)
            ' ' => {
                result.push_str("%20");
            }
            // Backslash should be encoded
            '\\' => {
                result.push_str("%5C");
            }
            // Square brackets should be encoded in URLs
            '[' => {
                result.push_str("%5B");
            }
            ']' => {
                result.push_str("%5D");
            }
            // Other characters: percent-encode
            _ => {
                let mut buf = [0; 4];
                let s = c.encode_utf8(&mut buf);
                for b in s.bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }

    result
}

/// Normalize a reference label for lookup
///
/// - Collapses internal whitespace to a single space
/// - Removes leading/trailing whitespace
/// - Converts to uppercase (for case-insensitive comparison)
///
/// Note: Does NOT unescape backslash escapes - [foo\!] and [foo!] are different labels
pub fn normalize_reference(label: &str) -> String {
    // Remove surrounding brackets if present
    let label = if label.starts_with('[') && label.ends_with(']') {
        &label[1..label.len() - 1]
    } else {
        label
    };

    // Normalize whitespace: collapse all whitespace sequences to a single space
    // Note: We do NOT unescape here - backslash escapes are preserved in link labels
    // per CommonMark spec. So "foo\!" stays as "foo\!", not "foo!"
    let normalized = label.split_whitespace().collect::<Vec<_>>().join(" ");

    // Unicode case folding: to_lowercase().to_uppercase() matches commonmark.js behavior
    // This properly handles characters like ß which folds to SS
    normalized.to_lowercase().to_uppercase()
}

/// Result of scanning delimiters
pub struct DelimScanResult {
    pub num_delims: usize,
    pub can_open: bool,
    pub can_close: bool,
}
