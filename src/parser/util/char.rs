//! Character parsing utilities.
//!
//! This module provides parsers for individual characters and character classes.

use crate::parser::util::{BoxedParser, ParseError, ParseResult, Position};

/// Parse a single character matching a predicate.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{char, Parser};
///
/// let parser = char(|c| c.is_digit(10));
/// let result = parser.parse_partial("123");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().0, '1');
/// ```ignore
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
/// ```ignore
/// use clmd::parsing::{any_char, Parser};
///
/// let result = any_char.parse_partial("abc");
/// assert_eq!(result.unwrap().0, 'a');
/// ```ignore
pub fn any_char(input: &str, pos: Position) -> ParseResult<(char, Position)> {
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
/// ```ignore
/// use clmd::parsing::{char_lit, Parser};
///
/// let parser = char_lit('a');
/// let result = parser.parse_partial("abc");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().0, 'a');
/// ```ignore
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
/// ```ignore
/// use clmd::parsing::{digit, Parser};
///
/// let result = digit.parse_partial("123");
/// assert_eq!(result.unwrap().0, '1');
/// ```ignore
pub fn digit(input: &str, pos: Position) -> ParseResult<(char, Position)> {
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
/// ```ignore
/// use clmd::parsing::{alpha, Parser};
///
/// let result = alpha.parse_partial("abc");
/// assert_eq!(result.unwrap().0, 'a');
/// ```ignore
pub fn alpha(input: &str, pos: Position) -> ParseResult<(char, Position)> {
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
/// ```ignore
/// use clmd::parsing::{alphanumeric, Parser};
///
/// let result = alphanumeric.parse_partial("abc123");
/// assert_eq!(result.unwrap().0, 'a');
/// ```ignore
pub fn alphanumeric(input: &str, pos: Position) -> ParseResult<(char, Position)> {
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
/// ```ignore
/// use clmd::parsing::{whitespace, Parser};
///
/// let result = whitespace.parse_partial("  hello");
/// assert_eq!(result.unwrap().0, ' ');
/// ```ignore
pub fn whitespace(input: &str, pos: Position) -> ParseResult<(char, Position)> {
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
/// ```ignore
/// use clmd::parsing::{newline, Parser};
///
/// let result = newline.parse_partial("\nhello");
/// assert!(result.is_ok());
/// ```ignore
pub fn newline(input: &str, pos: Position) -> ParseResult<(char, Position)> {
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
/// ```ignore
/// use clmd::parsing::{not_char, Parser};
///
/// let parser = not_char('x');
/// let result = parser.parse_partial("abc");
/// assert_eq!(result.unwrap().0, 'a');
/// ```ignore
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
/// ```ignore
/// use clmd::parsing::{none_of, Parser};
///
/// let parser = none_of(&['x', 'y', 'z']);
/// let result = parser.parse_partial("abc");
/// assert_eq!(result.unwrap().0, 'a');
/// ```ignore
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
/// ```ignore
/// use clmd::parsing::{one_of, Parser};
///
/// let parser = one_of(&['a', 'b', 'c']);
/// let result = parser.parse_partial("b");
/// assert_eq!(result.unwrap().0, 'b');
/// ```ignore
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
/// ```ignore
/// use clmd::parsing::{char_range, Parser};
///
/// let parser = char_range('a', 'z');
/// let result = parser.parse_partial("hello");
/// assert_eq!(result.unwrap().0, 'h');
/// ```ignore
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
/// ```ignore
/// use clmd::parsing::{upper, Parser};
///
/// let result = upper.parse_partial("Hello");
/// assert_eq!(result.unwrap().0, 'H');
/// ```ignore
pub fn upper(input: &str, pos: Position) -> ParseResult<(char, Position)> {
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
/// ```ignore
/// use clmd::parsing::{lower, Parser};
///
/// let result = lower.parse_partial("hello");
/// assert_eq!(result.unwrap().0, 'h');
/// ```ignore
pub fn lower(input: &str, pos: Position) -> ParseResult<(char, Position)> {
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
/// ```ignore
/// use clmd::parsing::{hex_digit, Parser};
///
/// let result = hex_digit.parse_partial("abc");
/// assert_eq!(result.unwrap().0, 'a');
/// ```ignore
pub fn hex_digit(input: &str, pos: Position) -> ParseResult<(char, Position)> {
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
/// ```ignore
/// use clmd::parsing::{oct_digit, Parser};
///
/// let result = oct_digit.parse_partial("777");
/// assert_eq!(result.unwrap().0, '7');
/// ```ignore
pub fn oct_digit(input: &str, pos: Position) -> ParseResult<(char, Position)> {
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
/// ```ignore
/// use clmd::parsing::{tab, Parser};
///
/// let result = tab.parse_partial("\thello");
/// assert_eq!(result.unwrap().0, '\t');
/// ```ignore
pub fn tab(input: &str, pos: Position) -> ParseResult<(char, Position)> {
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
/// ```ignore
/// use clmd::parsing::{space, Parser};
///
/// let result = space.parse_partial(" hello");
/// assert_eq!(result.unwrap().0, ' ');
/// ```ignore
pub fn space(input: &str, pos: Position) -> ParseResult<(char, Position)> {
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
/// ```ignore
/// use clmd::parsing::{satisfy, Parser};
///
/// let parser = satisfy(|c| c.is_ascii_punctuation());
/// let result = parser.parse_partial("!hello");
/// assert_eq!(result.unwrap().0, '!');
/// ```ignore
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

    #[test]
    fn test_char_lit() {
        let parser = char_lit('a');
        let result = parser("abc", Position::start()).unwrap();
        assert_eq!(result.0, 'a');
        assert!(parser("xyz", Position::start()).is_err());
    }

    #[test]
    fn test_digit() {
        let result = digit("123", Position::start()).unwrap();
        assert_eq!(result.0, '1');
        assert!(digit("abc", Position::start()).is_err());
    }

    #[test]
    fn test_alpha() {
        let result = alpha("abc", Position::start()).unwrap();
        assert_eq!(result.0, 'a');
        assert!(alpha("123", Position::start()).is_err());
    }

    #[test]
    fn test_whitespace() {
        let result = whitespace("  hello", Position::start()).unwrap();
        assert_eq!(result.0, ' ');
        let result = whitespace("\thello", Position::start()).unwrap();
        assert_eq!(result.0, '\t');
        assert!(whitespace("hello", Position::start()).is_err());
    }

    #[test]
    fn test_newline() {
        assert_eq!(newline("\n", Position::start()).unwrap().0, '\n');
        assert_eq!(newline("\r\n", Position::start()).unwrap().0, '\n');
        assert!(newline("hello", Position::start()).is_err());
    }

    #[test]
    fn test_one_of() {
        let parser = one_of(&['a', 'b', 'c']);
        let result = parser("b", Position::start()).unwrap();
        assert_eq!(result.0, 'b');
        assert!(parser("x", Position::start()).is_err());
    }

    #[test]
    fn test_none_of() {
        let parser = none_of(&['x', 'y', 'z']);
        let result = parser("a", Position::start()).unwrap();
        assert_eq!(result.0, 'a');
        assert!(parser("x", Position::start()).is_err());
    }

    #[test]
    fn test_char_range() {
        let parser = char_range('a', 'z');
        let result = parser("m", Position::start()).unwrap();
        assert_eq!(result.0, 'm');
        assert!(parser("M", Position::start()).is_err());
    }

    #[test]
    fn test_upper_lower() {
        let result = upper("Hello", Position::start()).unwrap();
        assert_eq!(result.0, 'H');
        let result = lower("hello", Position::start()).unwrap();
        assert_eq!(result.0, 'h');
        assert!(upper("hello", Position::start()).is_err());
        assert!(lower("Hello", Position::start()).is_err());
    }

    #[test]
    fn test_hex_digit() {
        let result = hex_digit("a", Position::start()).unwrap();
        assert_eq!(result.0, 'a');
        let result = hex_digit("F", Position::start()).unwrap();
        assert_eq!(result.0, 'F');
        let result = hex_digit("9", Position::start()).unwrap();
        assert_eq!(result.0, '9');
        assert!(hex_digit("g", Position::start()).is_err());
    }

    #[test]
    fn test_satisfy() {
        let parser = satisfy(|c| c == '@');
        let result = parser("@hello", Position::start()).unwrap();
        assert_eq!(result.0, '@');
        assert!(parser("hello", Position::start()).is_err());
    }
}
