//! General parsing utilities and combinators.
//!
//! This module provides general-purpose parsing utilities inspired by Pandoc's
//! parsing infrastructure. It includes parser combinators, character utilities,
//! and common parsing patterns that can be used across different parsers.
//!
//! # Example
//!
//! ```
//! use clmd::parsing::{Parser, char, string, many, choice};
//!
//! // Parse a simple identifier
//! let parser = many(char(|c| c.is_alphanumeric()));
//! let result = parser.parse("hello123");
//! assert!(result.is_ok());
//! ```

pub mod char;
pub mod combinator;
pub mod primitives;
pub mod state;

pub use char::*;
pub use combinator::*;
pub use primitives::*;
pub use state::*;

use std::fmt;

/// A parsing error.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// The position where the error occurred.
    pub position: Position,
    /// The error message.
    pub message: String,
    /// Expected tokens or patterns.
    pub expected: Vec<String>,
}

impl ParseError {
    /// Create a new parse error.
    pub fn new(position: Position, message: impl Into<String>) -> Self {
        Self {
            position,
            message: message.into(),
            expected: Vec::new(),
        }
    }

    /// Add an expected token.
    pub fn expect(mut self, expected: impl Into<String>) -> Self {
        self.expected.push(expected.into());
        self
    }

    /// Create an error at the given position with a message.
    pub fn at(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::new(Position::new(line, column), message)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at {}: {}", self.position, self.message)?;
        if !self.expected.is_empty() {
            write!(f, " (expected: {})", self.expected.join(", "))?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

/// A position in the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Position {
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed).
    pub column: usize,
    /// Byte offset in the input.
    pub offset: usize,
}

impl Position {
    /// Create a new position.
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            offset: 0,
        }
    }

    /// Create a position from an offset.
    pub fn from_offset(input: &str, offset: usize) -> Self {
        let mut line = 1;
        let mut column = 1;

        for (i, ch) in input.chars().enumerate() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        Self {
            line,
            column,
            offset,
        }
    }

    /// Advance by a character.
    pub fn advance(&mut self, ch: char) {
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// The result of a parse operation.
pub type ParseResult<T> = Result<(T, Position), ParseError>;

/// A parser function type.
pub type ParserFn<T> = dyn Fn(&str, Position) -> ParseResult<T>;

/// A parser that can parse input into a value.
pub trait Parser<T>: Fn(&str, Position) -> ParseResult<T> {
    /// Parse the entire input.
    fn parse(&self, input: &str) -> Result<T, ParseError> {
        let (result, pos) = (self)(input, Position::default())?;
        // Check if we've consumed all input (ignoring trailing whitespace)
        let remaining = &input[pos.offset..].trim();
        if !remaining.is_empty() {
            return Err(ParseError::at(
                pos.line,
                pos.column,
                format!("Unexpected: {}", &remaining[..remaining.len().min(20)]),
            ));
        }
        Ok(result)
    }

    /// Parse without requiring full consumption.
    fn parse_partial(&self, input: &str) -> ParseResult<T> {
        (self)(input, Position::default())
    }

    /// Map the result to a different type.
    fn map<F, U>(self, f: F) -> BoxedParser<U>
    where
        Self: Sized + 'static,
        F: Fn(T) -> U + 'static,
        T: 'static,
        U: 'static,
    {
        Box::new(move |input: &str, pos: Position| {
            let (result, new_pos) = (self)(input, pos)?;
            Ok((f(result), new_pos))
        })
    }

    /// Chain with another parser.
    fn and_then<F, U>(self, f: F) -> BoxedParser<U>
    where
        Self: Sized + 'static,
        F: Fn(T) -> BoxedParser<U> + 'static,
        T: 'static,
        U: 'static,
    {
        Box::new(move |input: &str, pos: Position| {
            let (result, new_pos) = (self)(input, pos)?;
            f(result)(input, new_pos)
        })
    }

    /// Try this parser, or another if it fails.
    fn or<P>(self, other: P) -> BoxedParser<T>
    where
        Self: Sized + 'static,
        P: Parser<T> + 'static,
        T: 'static,
    {
        Box::new(move |input: &str, pos: Position| {
            (self)(input, pos.clone()).or_else(|_| other(input, pos))
        })
    }

    /// Make the parser optional.
    fn optional(self) -> BoxedParser<Option<T>>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        Box::new(
            move |input: &str, pos: Position| match (self)(input, pos.clone()) {
                Ok((result, new_pos)) => Ok((Some(result), new_pos)),
                Err(_) => Ok((None, pos)),
            },
        )
    }

    /// Require the parser to succeed at least `n` times.
    fn many(self) -> BoxedParser<Vec<T>>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
    {
        Box::new(move |input: &str, pos: Position| {
            let mut results = Vec::new();
            let mut current_pos = pos;

            loop {
                match (self.clone())(input, current_pos.clone()) {
                    Ok((result, new_pos)) => {
                        results.push(result);
                        current_pos = new_pos;
                    }
                    Err(_) => break,
                }
            }

            Ok((results, current_pos))
        })
    }

    /// Require the parser to succeed at least once.
    fn many1(self) -> BoxedParser<Vec<T>>
    where
        Self: Sized + Clone + 'static,
        T: 'static,
    {
        Box::new(move |input: &str, pos: Position| {
            let (first, mut current_pos) = (self.clone())(input, pos)?;
            let mut results = vec![first];

            loop {
                match (self.clone())(input, current_pos.clone()) {
                    Ok((result, new_pos)) => {
                        results.push(result);
                        current_pos = new_pos;
                    }
                    Err(_) => break,
                }
            }

            Ok((results, current_pos))
        })
    }

    /// Parse with this parser separated by another.
    fn separated_by<S>(self, sep: S) -> BoxedParser<Vec<T>>
    where
        Self: Sized + Clone + 'static,
        S: Parser<()> + Clone + 'static,
        T: 'static,
    {
        Box::new(move |input: &str, pos: Position| {
            let mut results = Vec::new();
            let mut current_pos = pos;

            // Parse first element
            match (self.clone())(input, current_pos.clone()) {
                Ok((result, new_pos)) => {
                    results.push(result);
                    current_pos = new_pos;
                }
                Err(e) => return Err(e),
            }

            // Parse (separator, element)*
            loop {
                // Try separator
                match sep.clone()(input, current_pos.clone()) {
                    Ok((_, sep_pos)) => {
                        // Try element after separator
                        match (self.clone())(input, sep_pos.clone()) {
                            Ok((result, elem_pos)) => {
                                results.push(result);
                                current_pos = elem_pos;
                            }
                            Err(_) => break,
                        }
                    }
                    Err(_) => break,
                }
            }

            Ok((results, current_pos))
        })
    }
}

impl<T, F> Parser<T> for F where F: Fn(&str, Position) -> ParseResult<T> {}

/// A boxed parser for type erasure.
pub type BoxedParser<T> = Box<dyn Parser<T>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_advance() {
        let mut pos = Position::new(1, 1);
        pos.advance('a');
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 2);

        pos.advance('\n');
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn test_position_from_offset() {
        let input = "line1\nline2\nline3";
        let pos = Position::from_offset(input, 6); // Start of "line2" (after "line1\n")
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::at(1, 5, "unexpected token").expect("identifier");
        let msg = err.to_string();
        assert!(msg.contains("line 1, column 5"));
        assert!(msg.contains("unexpected token"));
        assert!(msg.contains("identifier"));
    }
}
