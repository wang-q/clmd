//! Character parsing utilities.
//!
//! This module provides parsers for individual characters and character classes,
//! as well as parser combinators for combining and transforming parsers.

use crate::parse::util::{BoxedParser, ClmdError, ClmdResult, Position};

/// Parse a digit character.
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

/// Parse zero or more occurrences of a parser.
pub fn many<T>(parser: BoxedParser<T>) -> BoxedParser<Vec<T>>
where
    T: 'static,
{
    Box::new(move |input: &str, pos| {
        let mut results = Vec::new();
        let mut current_pos = pos;

        loop {
            match parser(input, current_pos) {
                Ok((value, new_pos)) => {
                    results.push(value);
                    current_pos = new_pos;
                }
                Err(_) => break,
            }
        }

        Ok((results, current_pos))
    })
}

/// Parse one or more occurrences of a parser.
pub fn many1<T>(parser: BoxedParser<T>) -> BoxedParser<Vec<T>>
where
    T: 'static,
{
    Box::new(move |input: &str, pos| {
        let mut results = Vec::new();
        let mut current_pos = pos;

        // Must have at least one
        match parser(input, current_pos) {
            Ok((value, new_pos)) => {
                results.push(value);
                current_pos = new_pos;
            }
            Err(e) => return Err(e),
        }

        // Then parse more
        loop {
            match parser(input, current_pos) {
                Ok((value, new_pos)) => {
                    results.push(value);
                    current_pos = new_pos;
                }
                Err(_) => break,
            }
        }

        Ok((results, current_pos))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_alphanumeric() {
        let result = alphanumeric("abc", Position::start()).unwrap();
        assert_eq!(result.0, 'a');
        let result = alphanumeric("123", Position::start()).unwrap();
        assert_eq!(result.0, '1');
        assert!(alphanumeric(" ", Position::start()).is_err());
    }

    #[test]
    fn test_many() {
        let parser = many(Box::new(digit));
        let result = parser("123abc", Position::start()).unwrap();
        assert_eq!(result.0, vec!['1', '2', '3']);

        // Zero matches is ok
        let result = parser("abc", Position::start()).unwrap();
        assert!(result.0.is_empty());
    }

    #[test]
    fn test_many1() {
        let parser = many1(Box::new(digit));
        let result = parser("123abc", Position::start()).unwrap();
        assert_eq!(result.0, vec!['1', '2', '3']);

        // Must have at least one
        assert!(parser("abc", Position::start()).is_err());
    }
}
