//! Character parsing utilities.
//!
//! This module provides parsers for individual characters and character classes.

use super::{BoxedParser, ParseError, ParseResult, Position};

/// Parse a single character matching a predicate.
///
/// # Example
///
/// ```
/// use clmd::parsing::char;
///
/// let parser = char(|c| c.is_digit(10));
/// let result = parser.parse("123");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), '1');
/// ```
pub fn char<F>(predicate: F) -> BoxedParser<char>
where
    F: Fn(char) -> bool + 'static,
{
    Box::new(move |input: &str, pos: Position| {
        if let Some(ch) = input[pos.offset..].chars().next() {
            if predicate(ch) {
                let mut new_pos = pos;
                new_pos.advance(ch);
                return Ok((ch, new_pos));
            }
        }
        Err(ParseError::at(
            pos.line,
            pos.column,
            "Expected character matching predicate",
        ))
    })
}

/// Parse any character.
///
/// # Example
///
/// ```
/// use clmd::parsing::any_char;
///
/// let result = any_char.parse("abc");
/// assert_eq!(result.unwrap(), 'a');
/// ```
pub fn any_char(input: &str, pos: Position) -> ParseResult<char> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        let mut new_pos = pos;
        new_pos.advance(ch);
        Ok((ch, new_pos))
    } else {
        Err(ParseError::at(
            pos.line,
            pos.column,
            "Unexpected end of input",
        ))
    }
}

/// Parse a specific character.
///
/// # Example
///
/// ```
/// use clmd::parsing::char_lit;
///
/// let parser = char_lit('a');
/// let result = parser.parse("abc");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), 'a');
/// ```
pub fn char_lit(expected: char) -> BoxedParser<char> {
    Box::new(move |input: &str, pos: Position| {
        if let Some(ch) = input[pos.offset..].chars().next() {
            if ch == expected {
                let mut new_pos = pos;
                new_pos.advance(ch);
                return Ok((ch, new_pos));
            }
        }
        Err(ParseError::at(
            pos.line,
            pos.column,
            format!("Expected '{}'", expected),
        ))
    })
}

/// Parse a digit character.
///
/// # Example
///
/// ```
/// use clmd::parsing::digit;
///
/// let result = digit.parse("123");
/// assert_eq!(result.unwrap(), '1');
/// ```
pub fn digit(input: &str, pos: Position) -> ParseResult<char> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_ascii_digit() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ParseError::at(pos.line, pos.column, "Expected digit"))
}

/// Parse an alphabetic character.
///
/// # Example
///
/// ```
/// use clmd::parsing::alpha;
///
/// let result = alpha.parse("abc");
/// assert_eq!(result.unwrap(), 'a');
/// ```
pub fn alpha(input: &str, pos: Position) -> ParseResult<char> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_alphabetic() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ParseError::at(pos.line, pos.column, "Expected letter"))
}

/// Parse an alphanumeric character.
///
/// # Example
///
/// ```
/// use clmd::parsing::alphanumeric;
///
/// let result = alphanumeric.parse("abc123");
/// assert_eq!(result.unwrap(), 'a');
/// ```
pub fn alphanumeric(input: &str, pos: Position) -> ParseResult<char> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_alphanumeric() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ParseError::at(
        pos.line,
        pos.column,
        "Expected alphanumeric character",
    ))
}

/// Parse a whitespace character.
///
/// # Example
///
/// ```
/// use clmd::parsing::whitespace;
///
/// let result = whitespace.parse("  hello");
/// assert_eq!(result.unwrap(), ' ');
/// ```
pub fn whitespace(input: &str, pos: Position) -> ParseResult<char> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_whitespace() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ParseError::at(pos.line, pos.column, "Expected whitespace"))
}

/// Parse a newline character (\n or \r\n).
///
/// # Example
///
/// ```
/// use clmd::parsing::newline;
///
/// let result = newline.parse("\nhello");
/// assert!(result.is_ok());
/// ```
pub fn newline(input: &str, pos: Position) -> ParseResult<char> {
    let remaining = &input[pos.offset..];
    if remaining.starts_with("\r\n") {
        let mut new_pos = pos;
        new_pos.advance('\r');
        new_pos.advance('\n');
        Ok(('\n', new_pos))
    } else if let Some('\n') = remaining.chars().next() {
        let mut new_pos = pos;
        new_pos.advance('\n');
        Ok(('\n', new_pos))
    } else {
        Err(ParseError::at(pos.line, pos.column, "Expected newline"))
    }
}

/// Parse any character except the specified one.
///
/// # Example
///
/// ```
/// use clmd::parsing::not_char;
///
/// let parser = not_char('x');
/// let result = parser.parse("abc");
/// assert_eq!(result.unwrap(), 'a');
/// ```
pub fn not_char(forbidden: char) -> BoxedParser<char> {
    Box::new(move |input: &str, pos: Position| {
        if let Some(ch) = input[pos.offset..].chars().next() {
            if ch != forbidden {
                let mut new_pos = pos;
                new_pos.advance(ch);
                return Ok((ch, new_pos));
            }
        }
        Err(ParseError::at(
            pos.line,
            pos.column,
            format!("Expected any character except '{}'", forbidden),
        ))
    })
}

/// Parse any character except those in the given set.
///
/// # Example
///
/// ```
/// use clmd::parsing::none_of;
///
/// let parser = none_of(&['x', 'y', 'z']);
/// let result = parser.parse("abc");
/// assert_eq!(result.unwrap(), 'a');
/// ```
pub fn none_of(forbidden: &'static [char]) -> BoxedParser<char> {
    Box::new(move |input: &str, pos: Position| {
        if let Some(ch) = input[pos.offset..].chars().next() {
            if !forbidden.contains(&ch) {
                let mut new_pos = pos;
                new_pos.advance(ch);
                return Ok((ch, new_pos));
            }
        }
        Err(ParseError::at(
            pos.line,
            pos.column,
            format!("Expected character not in {:?}", forbidden),
        ))
    })
}

/// Parse one of the given characters.
///
/// # Example
///
/// ```
/// use clmd::parsing::one_of;
///
/// let parser = one_of(&['a', 'b', 'c']);
/// let result = parser.parse("b");
/// assert_eq!(result.unwrap(), 'b');
/// ```
pub fn one_of(allowed: &'static [char]) -> BoxedParser<char> {
    Box::new(move |input: &str, pos: Position| {
        if let Some(ch) = input[pos.offset..].chars().next() {
            if allowed.contains(&ch) {
                let mut new_pos = pos;
                new_pos.advance(ch);
                return Ok((ch, new_pos));
            }
        }
        Err(ParseError::at(
            pos.line,
            pos.column,
            format!("Expected one of {:?}", allowed),
        ))
    })
}

/// Parse a character in a range.
///
/// # Example
///
/// ```
/// use clmd::parsing::char_range;
///
/// let parser = char_range('a', 'z');
/// let result = parser.parse("hello");
/// assert_eq!(result.unwrap(), 'h');
/// ```
pub fn char_range(start: char, end: char) -> BoxedParser<char> {
    Box::new(move |input: &str, pos: Position| {
        if let Some(ch) = input[pos.offset..].chars().next() {
            if ch >= start && ch <= end {
                let mut new_pos = pos;
                new_pos.advance(ch);
                return Ok((ch, new_pos));
            }
        }
        Err(ParseError::at(
            pos.line,
            pos.column,
            format!("Expected character between '{}' and '{}'", start, end),
        ))
    })
}

/// Parse an uppercase letter.
///
/// # Example
///
/// ```
/// use clmd::parsing::upper;
///
/// let result = upper.parse("Hello");
/// assert_eq!(result.unwrap(), 'H');
/// ```
pub fn upper(input: &str, pos: Position) -> ParseResult<char> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_uppercase() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ParseError::at(
        pos.line,
        pos.column,
        "Expected uppercase letter",
    ))
}

/// Parse a lowercase letter.
///
/// # Example
///
/// ```
/// use clmd::parsing::lower;
///
/// let result = lower.parse("hello");
/// assert_eq!(result.unwrap(), 'h');
/// ```
pub fn lower(input: &str, pos: Position) -> ParseResult<char> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_lowercase() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ParseError::at(
        pos.line,
        pos.column,
        "Expected lowercase letter",
    ))
}

/// Parse a hexadecimal digit.
///
/// # Example
///
/// ```
/// use clmd::parsing::hex_digit;
///
/// let result = hex_digit.parse("abc");
/// assert_eq!(result.unwrap(), 'a');
/// ```
pub fn hex_digit(input: &str, pos: Position) -> ParseResult<char> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_ascii_hexdigit() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ParseError::at(
        pos.line,
        pos.column,
        "Expected hexadecimal digit",
    ))
}

/// Parse an octal digit.
///
/// # Example
///
/// ```
/// use clmd::parsing::oct_digit;
///
/// let result = oct_digit.parse("777");
/// assert_eq!(result.unwrap(), '7');
/// ```
pub fn oct_digit(input: &str, pos: Position) -> ParseResult<char> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ('0'..='7').contains(&ch) {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ParseError::at(pos.line, pos.column, "Expected octal digit"))
}

/// Parse a tab character.
///
/// # Example
///
/// ```
/// use clmd::parsing::tab;
///
/// let result = tab.parse("\thello");
/// assert_eq!(result.unwrap(), '\t');
/// ```
pub fn tab(input: &str, pos: Position) -> ParseResult<char> {
    if let Some('\t') = input[pos.offset..].chars().next() {
        let mut new_pos = pos;
        new_pos.advance('\t');
        Ok(('\t', new_pos))
    } else {
        Err(ParseError::at(pos.line, pos.column, "Expected tab"))
    }
}

/// Parse a space character.
///
/// # Example
///
/// ```
/// use clmd::parsing::space;
///
/// let result = space.parse(" hello");
/// assert_eq!(result.unwrap(), ' ');
/// ```
pub fn space(input: &str, pos: Position) -> ParseResult<char> {
    if let Some(' ') = input[pos.offset..].chars().next() {
        let mut new_pos = pos;
        new_pos.advance(' ');
        Ok((' ', new_pos))
    } else {
        Err(ParseError::at(pos.line, pos.column, "Expected space"))
    }
}

/// Satisfy a predicate for a character.
///
/// # Example
///
/// ```
/// use clmd::parsing::satisfy;
///
/// let parser = satisfy(|c| c.is_ascii_punctuation());
/// let result = parser.parse("!hello");
/// assert_eq!(result.unwrap(), '!');
/// ```
pub fn satisfy<F>(predicate: F) -> BoxedParser<char>
where
    F: Fn(char) -> bool + 'static,
{
    Box::new(move |input: &str, pos: Position| {
        if let Some(ch) = input[pos.offset..].chars().next() {
            if predicate(ch) {
                let mut new_pos = pos;
                new_pos.advance(ch);
                return Ok((ch, new_pos));
            }
        }
        Err(ParseError::at(
            pos.line,
            pos.column,
            "Character did not satisfy predicate",
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::Parser;

    #[test]
    fn test_char_lit() {
        let parser = char_lit('a');
        let result = parser.parse_partial("abc").unwrap();
        assert_eq!(result.0, 'a');
        assert!(parser.parse("xyz").is_err());
    }

    #[test]
    fn test_digit() {
        let result = digit.parse_partial("123").unwrap();
        assert_eq!(result.0, '1');
        assert!(digit.parse("abc").is_err());
    }

    #[test]
    fn test_alpha() {
        let result = alpha.parse_partial("abc").unwrap();
        assert_eq!(result.0, 'a');
        assert!(alpha.parse("123").is_err());
    }

    #[test]
    fn test_whitespace() {
        let result = whitespace.parse_partial("  hello").unwrap();
        assert_eq!(result.0, ' ');
        let result = whitespace.parse_partial("\thello").unwrap();
        assert_eq!(result.0, '\t');
        assert!(whitespace.parse("hello").is_err());
    }

    #[test]
    fn test_newline() {
        assert_eq!(newline.parse("\n").unwrap(), '\n');
        assert_eq!(newline.parse("\r\n").unwrap(), '\n');
        assert!(newline.parse("hello").is_err());
    }

    #[test]
    fn test_one_of() {
        let parser = one_of(&['a', 'b', 'c']);
        let result = parser.parse_partial("b").unwrap();
        assert_eq!(result.0, 'b');
        assert!(parser.parse("x").is_err());
    }

    #[test]
    fn test_none_of() {
        let parser = none_of(&['x', 'y', 'z']);
        let result = parser.parse_partial("a").unwrap();
        assert_eq!(result.0, 'a');
        assert!(parser.parse("x").is_err());
    }

    #[test]
    fn test_char_range() {
        let parser = char_range('a', 'z');
        let result = parser.parse_partial("m").unwrap();
        assert_eq!(result.0, 'm');
        assert!(parser.parse("M").is_err());
    }

    #[test]
    fn test_upper_lower() {
        let result = upper.parse_partial("Hello").unwrap();
        assert_eq!(result.0, 'H');
        let result = lower.parse_partial("hello").unwrap();
        assert_eq!(result.0, 'h');
        assert!(upper.parse("hello").is_err());
        assert!(lower.parse("Hello").is_err());
    }

    #[test]
    fn test_hex_digit() {
        let result = hex_digit.parse_partial("a").unwrap();
        assert_eq!(result.0, 'a');
        let result = hex_digit.parse_partial("F").unwrap();
        assert_eq!(result.0, 'F');
        let result = hex_digit.parse_partial("9").unwrap();
        assert_eq!(result.0, '9');
        assert!(hex_digit.parse("g").is_err());
    }

    #[test]
    fn test_satisfy() {
        let parser = satisfy(|c| c == '@');
        let result = parser.parse_partial("@hello").unwrap();
        assert_eq!(result.0, '@');
        assert!(parser.parse("hello").is_err());
    }
}
