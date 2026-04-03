//! Character parsing utilities.
//!
//! This module provides parsers for individual characters and character classes.

use crate::parse::util::{BoxedParser, ClmdError, ClmdResult, Position};

/// Parse a single character matching a predicate.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{char, Parser};
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
        Err(ClmdError::parse_error(
            pos,
            "Expected character matching predicate",
        ))
    })
}

/// Parse any character.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{any_char, Parser};
///
/// let result = any_char.parse_partial("abc");
/// assert_eq!(result.unwrap().0, 'a');
/// ```ignore
pub fn any_char(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        let mut new_pos = pos;
        new_pos.advance(ch);
        Ok((ch, new_pos))
    } else {
        Err(ClmdError::parse_error(pos, "Unexpected end of input"))
    }
}

/// Parse a specific character.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{char_lit, Parser};
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
        Err(ClmdError::parse_error(
            pos,
            format!("Expected '{}'", expected),
        ))
    })
}

/// Parse a digit character.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{digit, Parser};
///
/// let result = digit.parse_partial("123");
/// assert_eq!(result.unwrap().0, '1');
/// ```ignore
pub fn digit(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_ascii_digit() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ClmdError::parse_error(pos, "Expected digit"))
}

/// Parse an alphabetic character.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{alpha, Parser};
///
/// let result = alpha.parse_partial("abc");
/// assert_eq!(result.unwrap().0, 'a');
/// ```ignore
pub fn alpha(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_alphabetic() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ClmdError::parse_error(pos, "Expected letter"))
}

/// Parse an alphanumeric character.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{alphanumeric, Parser};
///
/// let result = alphanumeric.parse_partial("abc123");
/// assert_eq!(result.unwrap().0, 'a');
/// ```ignore
pub fn alphanumeric(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_alphanumeric() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ClmdError::parse_error(
        pos,
        "Expected alphanumeric character",
    ))
}

/// Parse a whitespace character.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{whitespace, Parser};
///
/// let result = whitespace.parse_partial("  hello");
/// assert_eq!(result.unwrap().0, ' ');
/// ```ignore
pub fn whitespace(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_whitespace() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ClmdError::parse_error(pos, "Expected whitespace"))
}

/// Parse a newline character (\n or \r\n).
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{newline, Parser};
///
/// let result = newline.parse_partial("\nhello");
/// assert!(result.is_ok());
/// ```ignore
pub fn newline(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
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
        Err(ClmdError::parse_error(pos, "Expected newline"))
    }
}

/// Parse any character except the specified one.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{not_char, Parser};
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
        Err(ClmdError::parse_error(
            pos,
            format!("Expected any character except '{}'", forbidden),
        ))
    })
}

/// Parse any character except those in the given set.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{none_of, Parser};
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
        Err(ClmdError::parse_error(
            pos,
            format!("Expected character not in {:?}", forbidden),
        ))
    })
}

/// Parse one of the given characters.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{one_of, Parser};
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
        Err(ClmdError::parse_error(
            pos,
            format!("Expected one of {:?}", allowed),
        ))
    })
}

/// Parse a character in a range.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{char_range, Parser};
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
        Err(ClmdError::parse_error(
            pos,
            format!("Expected character between '{}' and '{}'", start, end),
        ))
    })
}

/// Parse an uppercase letter.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{upper, Parser};
///
/// let result = upper.parse_partial("Hello");
/// assert_eq!(result.unwrap().0, 'H');
/// ```ignore
pub fn upper(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_uppercase() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ClmdError::parse_error(pos, "Expected uppercase letter"))
}

/// Parse a lowercase letter.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{lower, Parser};
///
/// let result = lower.parse_partial("hello");
/// assert_eq!(result.unwrap().0, 'h');
/// ```ignore
pub fn lower(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_lowercase() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ClmdError::parse_error(pos, "Expected lowercase letter"))
}

/// Parse a hexadecimal digit.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{hex_digit, Parser};
///
/// let result = hex_digit.parse_partial("abc");
/// assert_eq!(result.unwrap().0, 'a');
/// ```ignore
pub fn hex_digit(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_ascii_hexdigit() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ClmdError::parse_error(pos, "Expected hexadecimal digit"))
}

/// Parse an octal digit.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{oct_digit, Parser};
///
/// let result = oct_digit.parse_partial("777");
/// assert_eq!(result.unwrap().0, '7');
/// ```ignore
pub fn oct_digit(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ('0'..='7').contains(&ch) {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ClmdError::parse_error(pos, "Expected octal digit"))
}

/// Parse a tab character.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{tab, Parser};
///
/// let result = tab.parse_partial("\thello");
/// assert_eq!(result.unwrap().0, '\t');
/// ```ignore
pub fn tab(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some('\t') = input[pos.offset..].chars().next() {
        let mut new_pos = pos;
        new_pos.advance('\t');
        Ok(('\t', new_pos))
    } else {
        Err(ClmdError::parse_error(pos, "Expected tab"))
    }
}

/// Parse a space character.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{space, Parser};
///
/// let result = space.parse_partial(" hello");
/// assert_eq!(result.unwrap().0, ' ');
/// ```ignore
pub fn space(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(' ') = input[pos.offset..].chars().next() {
        let mut new_pos = pos;
        new_pos.advance(' ');
        Ok((' ', new_pos))
    } else {
        Err(ClmdError::parse_error(pos, "Expected space"))
    }
}

/// Satisfy a predicate for a character.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::{satisfy, Parser};
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
        Err(ClmdError::parse_error(
            pos,
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
