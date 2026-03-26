//! HTML entity parsing for inline elements

use crate::inlines::utils::{get_html5_entity, is_escapable};
use htmlescape::decode_html;

/// Parse an HTML entity at the start of a string
/// Returns (decoded_char, chars_consumed) or None
/// Returns None for invalid entities (like out-of-range numeric entities)
pub fn parse_entity(s: &str) -> Option<(String, usize)> {
    if !s.starts_with('#') && !s.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return None;
    }

    // Numeric entity: &#123; or &#x7B;
    if s.starts_with('#') {
        let rest = &s[1..];
        if rest.starts_with('x') || rest.starts_with('X') {
            // Hex entity: #x7B; (rest starts with x)
            let hex_digits_start = 1; // Skip the 'x' or 'X'
            let hex_end = rest[hex_digits_start..]
                .find(|c: char| !c.is_ascii_hexdigit())
                .map(|i| hex_digits_start + i)
                .unwrap_or(rest.len());

            if hex_end > hex_digits_start && rest[hex_end..].starts_with(';') {
                let hex_str = &rest[hex_digits_start..hex_end];
                if let Ok(codepoint) = u32::from_str_radix(hex_str, 16) {
                    // Handle invalid codepoints
                    // codepoint == 0 (NUL): replacement character
                    // codepoint > 0x10ffff: preserve original entity
                    if codepoint == 0 {
                        return Some((
                            '\u{FFFD}'.to_string(),
                            2 + hex_end - hex_digits_start + 1,
                        ));
                    }
                    if codepoint > 0x10ffff {
                        return None; // Preserve original entity
                    }
                    let c = char::from_u32(codepoint).unwrap_or('\u{FFFD}');
                    // Total length: # (1) + x (1) + hex_digits + ; (1)
                    return Some((c.to_string(), 2 + hex_end - hex_digits_start + 1));
                }
            }
        } else {
            // Decimal entity: #123;
            let dec_end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());

            if dec_end > 0 && rest[dec_end..].starts_with(';') {
                let dec_str = &rest[..dec_end];
                if let Ok(codepoint) = dec_str.parse::<u32>() {
                    // Handle invalid codepoints
                    // codepoint == 0 (NUL): replacement character
                    // codepoint > 0x10ffff: preserve original entity
                    if codepoint == 0 {
                        return Some(('\u{FFFD}'.to_string(), 1 + dec_end + 1));
                    }
                    if codepoint > 0x10ffff {
                        return None; // Preserve original entity
                    }
                    let c = char::from_u32(codepoint).unwrap_or('\u{FFFD}');
                    return Some((c.to_string(), 1 + dec_end + 1));
                }
            }
        }
    } else {
        // Named entity
        let name_end = s
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(s.len());

        if name_end > 0 && s[name_end..].starts_with(';') {
            let name = &s[..name_end];

            // First try our HTML5 entity table
            if let Some(decoded) = get_html5_entity(name) {
                return Some((decoded.to_string(), name_end + 1));
            }

            // Then try htmlescape
            let entity_str = format!("&{};", name);
            if let Ok(decoded) = decode_html(&entity_str) {
                // Only return if htmlescape actually decoded it
                if decoded != entity_str {
                    return Some((decoded, name_end + 1));
                }
            }
        }
    }

    None
}

/// Unescape a string by processing backslash escapes and entities
/// Based on commonmark.js unescapeString
pub fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next_c) = chars.peek() {
                if is_escapable(next_c) {
                    chars.next();
                    result.push(next_c);
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        } else if c == '&' {
            // Try to parse an entity
            let remaining: String = chars.clone().collect();
            if let Some((entity, consumed)) = parse_entity(&remaining) {
                result.push_str(&entity);
                // Skip consumed characters
                for _ in 0..consumed {
                    chars.next();
                }
            } else {
                // Not a valid entity, keep the & as is
                // The HTML renderer will escape it if needed
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Parse an HTML entity and return the decoded string and length
/// Uses htmlescape crate and our entity table to support all HTML5 named entities
/// Returns None if this is not an entity pattern at all
/// Returns Some((decoded, len)) for valid entities
/// For invalid entities (like &#87654321;), returns Some((original, len)) to preserve them
pub fn parse_entity_char(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('&') {
        return None;
    }

    // Find the end of the entity (semicolon or end of string)
    let end = input.find(';').map(|i| i + 1).unwrap_or(input.len());
    if end <= 1 {
        return None;
    }

    let entity_str = &input[..end];

    // Try numeric entity first: &#123; or &#x7B;
    if entity_str.starts_with("&#") {
        // Check if it's a valid numeric entity format
        let rest = &entity_str[2..]; // Skip "&#"

        if rest.starts_with('x') || rest.starts_with('X') {
            // Hex entity: &#x7B;
            let hex_digits = &rest[1..rest.len() - 1]; // Skip 'x' and ';'
            if !hex_digits.is_empty()
                && hex_digits.chars().all(|c| c.is_ascii_hexdigit())
            {
                // Valid hex format, use parse_entity
                if let Some((decoded, _)) = parse_entity(&entity_str[1..]) {
                    return Some((decoded, entity_str.len()));
                }
                // Invalid hex entity (e.g., out of range) - preserve as-is
                return Some((entity_str.to_string(), entity_str.len()));
            }
        } else {
            // Decimal entity: &#123;
            let dec_digits = &rest[..rest.len() - 1]; // Remove ';'
            if !dec_digits.is_empty() && dec_digits.chars().all(|c| c.is_ascii_digit()) {
                // Valid decimal format, use parse_entity
                if let Some((decoded, _)) = parse_entity(&entity_str[1..]) {
                    return Some((decoded, entity_str.len()));
                }
                // Invalid decimal entity (e.g., out of range) - preserve as-is
                return Some((entity_str.to_string(), entity_str.len()));
            }
        }
        // Invalid numeric entity format - don't consume
        return None;
    }

    // Try named entity from our table
    if entity_str.len() > 2 {
        let name = &entity_str[1..entity_str.len() - 1]; // Remove & and ;
        if !name.is_empty() {
            if let Some(decoded) = get_html5_entity(name) {
                return Some((decoded.to_string(), entity_str.len()));
            }
        }
    }

    // Try to decode using htmlescape crate
    match decode_html(entity_str) {
        Ok(decoded) => {
            // If decoding produced a different result, it's a valid entity
            if decoded != entity_str {
                Some((decoded, end))
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
