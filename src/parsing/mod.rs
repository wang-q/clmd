//! Parsing utilities for clmd.
//!
//! This module provides low-level parsing primitives, combinators, and utilities
//! for building parsers. It includes character handling, scanning functions,
//! and parsing state management.

// Core parsing primitives
pub mod char;
pub mod combinator;
pub mod primitives;
pub mod scanners;
pub mod state;

// Source and chunk handling
pub mod chunks;
pub mod sources;

// Re-export error types
pub use crate::core::error::{ParseError, ParseResult, Position};

// Re-export commonly used types
pub use chunks::Chunk;
pub use sources::{Source, SourcePos};

/// A boxed parser for type erasure.
///
/// The parser takes an input string and a starting position, and returns either:
/// - Ok((value, new_position)) on success
/// - Err(ParseError) on failure
pub type BoxedParser<T> = Box<dyn Fn(&str, Position) -> ParseResult<(T, Position)>>;

/// Extension trait for parser functions.
///
/// This trait provides convenience methods for calling parsers.
pub trait Parser<T> {
    /// Parse from the beginning of the input.
    fn parse(&self, input: &str) -> ParseResult<T>;

    /// Parse from a specific position in the input.
    fn parse_at(&self, input: &str, pos: Position) -> ParseResult<(T, Position)>;

    /// Parse from the beginning, returning the result and new position.
    fn parse_partial(&self, input: &str) -> ParseResult<(T, Position)>;
}

impl<T> Parser<T> for BoxedParser<T> {
    fn parse(&self, input: &str) -> ParseResult<T> {
        self(input, Position::start()).map(|(val, _)| val)
    }

    fn parse_at(&self, input: &str, pos: Position) -> ParseResult<(T, Position)> {
        self(input, pos)
    }

    fn parse_partial(&self, input: &str) -> ParseResult<(T, Position)> {
        self(input, Position::start())
    }
}

// Implement Parser for function pointers
impl<T> Parser<T> for fn(&str, Position) -> ParseResult<(T, Position)> {
    fn parse(&self, input: &str) -> ParseResult<T> {
        self(input, Position::start()).map(|(val, _)| val)
    }

    fn parse_at(&self, input: &str, pos: Position) -> ParseResult<(T, Position)> {
        self(input, pos)
    }

    fn parse_partial(&self, input: &str) -> ParseResult<(T, Position)> {
        self(input, Position::start())
    }
}
