//! Stateful parsing utilities.
//!
//! This module provides utilities for parsers that need to maintain state,
//! such as tracking indentation levels, brace nesting, or custom state.

use super::{ParseError, ParseResult, Position};

/// A parser that maintains state.
pub trait StatefulParser<S, T>: Fn(&str, Position, &mut S) -> ParseResult<T> {}

impl<S, T, F> StatefulParser<S, T> for F where
    F: Fn(&str, Position, &mut S) -> ParseResult<T>
{
}

/// A boxed stateful parser for type erasure.
pub type BoxedStatefulParser<S, T> = Box<dyn StatefulParser<S, T>>;

/// Parse with indentation tracking.
///
/// This parser tracks the current indentation level and can be used
/// to parse indentation-sensitive formats like YAML or Python.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{with_indentation, indent_level};
///
/// let parser = with_indentation(indent_level(4));
/// let result = parser.parse("    hello").unwrap();
/// assert_eq!(result, "hello");
/// ```ignore
pub fn with_indentation<T>(
    parser: impl Fn(&str, Position, &mut IndentationState) -> ParseResult<T> + 'static,
) -> impl Fn(&str, Position) -> ParseResult<T>
where
    T: 'static,
{
    move |input: &str, pos: Position| {
        let mut state = IndentationState::new();
        parser(input, pos, &mut state)
    }
}

/// State for indentation tracking.
#[derive(Debug, Clone)]
pub struct IndentationState {
    /// Current indentation level (in spaces).
    pub level: usize,
    /// Stack of indentation levels.
    pub stack: Vec<usize>,
    /// Whether to use tabs or spaces.
    pub use_tabs: bool,
}

impl IndentationState {
    /// Create a new indentation state.
    pub fn new() -> Self {
        Self {
            level: 0,
            stack: vec![0],
            use_tabs: false,
        }
    }

    /// Push a new indentation level.
    pub fn push(&mut self, level: usize) {
        self.stack.push(level);
        self.level = level;
    }

    /// Pop the current indentation level.
    pub fn pop(&mut self) -> Option<usize> {
        if self.stack.len() > 1 {
            self.stack.pop();
            self.level = *self.stack.last().unwrap_or(&0);
            Some(self.level)
        } else {
            None
        }
    }

    /// Check if the current level matches.
    pub fn at_level(&self, level: usize) -> bool {
        self.level == level
    }

    /// Get the current indentation string.
    pub fn indent_string(&self) -> String {
        if self.use_tabs {
            "\t".repeat(self.level)
        } else {
            " ".repeat(self.level)
        }
    }
}

impl Default for IndentationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse content at a specific indentation level.
pub fn indent_level<T>(
    level: usize,
) -> impl Fn(&str, Position, &mut IndentationState) -> ParseResult<(String, Position)> {
    move |input: &str, pos: Position, state: &mut IndentationState| {
        let mut current_pos = pos;
        let mut indent_count = 0;

        // Count leading whitespace
        while let Some(ch) = input[current_pos.offset..].chars().next() {
            if ch == ' ' {
                indent_count += 1;
                current_pos.advance(ch);
            } else if ch == '\t' {
                indent_count += 4; // Treat tab as 4 spaces
                current_pos.advance(ch);
            } else {
                break;
            }
        }

        if indent_count != level {
            return Err(ParseError::at(
                pos.line,
                pos.column,
                format!(
                    "Expected indentation level {}, found {}",
                    level, indent_count
                ),
            ));
        }

        // Parse the rest of the line
        let mut result = String::new();
        while let Some(ch) = input[current_pos.offset..].chars().next() {
            if ch == '\n' {
                break;
            }
            result.push(ch);
            current_pos.advance(ch);
        }

        // Update state
        state.push(level);

        Ok((result, current_pos))
    }
}

/// Parse with brace nesting tracking.
///
/// This parser tracks opening and closing braces/parens/brackets
/// and can be used to parse nested structures.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{with_nesting, nested_content};
///
/// let parser = with_nesting(nested_content('{', '}'));
/// let result = parser.parse("{hello {world}}").unwrap();
/// assert_eq!(result, "hello {world}");
/// ```ignore
pub fn with_nesting<T>(
    parser: impl Fn(&str, Position, &mut NestingState) -> ParseResult<T> + 'static,
) -> impl Fn(&str, Position) -> ParseResult<T>
where
    T: 'static,
{
    move |input: &str, pos: Position| {
        let mut state = NestingState::new();
        parser(input, pos, &mut state)
    }
}

/// State for brace nesting tracking.
#[derive(Debug, Clone)]
pub struct NestingState {
    /// Stack of opening characters.
    pub stack: Vec<char>,
    /// Maximum nesting depth allowed.
    pub max_depth: usize,
}

impl NestingState {
    /// Create a new nesting state.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            max_depth: 100,
        }
    }

    /// Set maximum nesting depth.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Push an opening character.
    pub fn push(&mut self, ch: char) -> Result<(), ParseError> {
        if self.stack.len() >= self.max_depth {
            return Err(ParseError::at(0, 0, "Maximum nesting depth exceeded"));
        }
        self.stack.push(ch);
        Ok(())
    }

    /// Pop and check if it matches the expected closing character.
    pub fn pop(&mut self, expected_open: char) -> Result<(), ParseError> {
        match self.stack.pop() {
            Some(open) if open == expected_open => Ok(()),
            _ => Err(ParseError::at(0, 0, "Mismatched closing character")),
        }
    }

    /// Get current nesting depth.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Check if we're at the top level.
    pub fn is_top_level(&self) -> bool {
        self.stack.is_empty()
    }

    /// Get the expected closing character for the current level.
    pub fn expected_close(&self) -> Option<char> {
        self.stack.last().map(|&open| match open {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            '<' => '>',
            _ => open,
        })
    }
}

impl Default for NestingState {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse nested content between matching delimiters.
pub fn nested_content(
    open: char,
    close: char,
) -> impl Fn(&str, Position, &mut NestingState) -> ParseResult<(String, Position)> {
    move |input: &str, pos: Position, state: &mut NestingState| {
        let mut current_pos = pos;

        // Check opening delimiter
        if let Some(ch) = input[current_pos.offset..].chars().next() {
            if ch != open {
                return Err(ParseError::at(
                    current_pos.line,
                    current_pos.column,
                    format!("Expected '{}'", open),
                ));
            }
            current_pos.advance(ch);
            state.push(open)?;
        } else {
            return Err(ParseError::at(
                current_pos.line,
                current_pos.column,
                "Unexpected end of input",
            ));
        }

        let mut result = String::new();
        let mut depth = 1;

        while let Some(ch) = input[current_pos.offset..].chars().next() {
            if ch == open {
                depth += 1;
                state.push(open)?;
                result.push(ch);
                current_pos.advance(ch);
            } else if ch == close {
                depth -= 1;
                state.pop(open)?;
                current_pos.advance(ch);
                if depth == 0 {
                    return Ok((result, current_pos));
                }
                result.push(ch);
            } else {
                result.push(ch);
                current_pos.advance(ch);
            }
        }

        Err(ParseError::at(
            current_pos.line,
            current_pos.column,
            format!("Unclosed delimiter, expected '{}'", close),
        ))
    }
}

/// Parse with a custom state.
///
/// This is a general-purpose stateful parser wrapper.
pub fn with_state<S: Clone, T>(
    initial: S,
    parser: impl Fn(&str, Position, &mut S) -> ParseResult<T>,
) -> impl Fn(&str, Position) -> ParseResult<T> {
    move |input: &str, pos: Position| {
        let mut state = initial.clone();
        parser(input, pos, &mut state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indentation_state() {
        let mut state = IndentationState::new();
        assert_eq!(state.level, 0);

        state.push(4);
        assert_eq!(state.level, 4);

        state.push(8);
        assert_eq!(state.level, 8);

        state.pop();
        assert_eq!(state.level, 4);
    }

    #[test]
    fn test_nesting_state() {
        let mut state = NestingState::new();
        assert!(state.is_top_level());

        state.push('(').unwrap();
        assert_eq!(state.depth(), 1);
        assert_eq!(state.expected_close(), Some(')'));

        state.push('{').unwrap();
        assert_eq!(state.depth(), 2);
        assert_eq!(state.expected_close(), Some('}'));

        state.pop('{').unwrap();
        assert_eq!(state.depth(), 1);

        state.pop('(').unwrap();
        assert!(state.is_top_level());
    }

    #[test]
    fn test_nesting_state_mismatch() {
        let mut state = NestingState::new();
        state.push('(').unwrap();
        assert!(state.pop(')').is_err()); // Should fail - we pushed '(' not ')'
    }

    #[test]
    fn test_nested_content() {
        let parser = with_nesting(nested_content('{', '}'));
        let result = parser("{hello}", Position::start()).unwrap().0;
        assert_eq!(result, "hello");

        let result = parser("{hello {world}}", Position::start()).unwrap().0;
        assert_eq!(result, "hello {world}");
    }

    #[test]
    fn test_nested_content_unclosed() {
        let parser = with_nesting(nested_content('{', '}'));
        assert!(parser("{hello", Position::start()).is_err());
    }

    #[test]
    fn test_indentation_state_at_level() {
        let mut state = IndentationState::new();
        assert!(state.at_level(0));

        state.push(4);
        assert!(state.at_level(4));
        assert!(!state.at_level(0));
    }

    #[test]
    fn test_indentation_state_default() {
        let state: IndentationState = Default::default();
        assert_eq!(state.level, 0);
        assert!(!state.stack.is_empty());
    }

    #[test]
    fn test_nesting_state_push_pop_multiple() {
        let mut state = NestingState::new();

        // Push multiple delimiters
        state.push('(').unwrap();
        state.push('[').unwrap();
        state.push('{').unwrap();

        assert_eq!(state.depth(), 3);
        assert_eq!(state.expected_close(), Some('}'));

        // Pop in reverse order
        state.pop('{').unwrap();
        assert_eq!(state.expected_close(), Some(']'));

        state.pop('[').unwrap();
        assert_eq!(state.expected_close(), Some(')'));

        state.pop('(').unwrap();
        assert!(state.is_top_level());
    }

    #[test]
    fn test_nesting_state_default() {
        let state: NestingState = Default::default();
        assert!(state.is_top_level());
        assert_eq!(state.depth(), 0);
    }

    #[test]
    fn test_nesting_state_debug() {
        let mut state = NestingState::new();
        state.push('(').unwrap();
        state.push('[').unwrap();

        let debug = format!("{:?}", state);
        assert!(debug.contains("["));
        assert!(debug.contains("("));
    }

    #[test]
    fn test_position_advance() {
        let mut pos = Position::start();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 0);

        pos.advance('a');
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 2);
        assert_eq!(pos.offset, 1);

        pos.advance('\n');
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 2);
    }

    #[test]
    fn test_position_advance_multi_byte() {
        let mut pos = Position::start();

        // Test multi-byte character (2 bytes in UTF-8)
        pos.advance('é');
        assert_eq!(pos.column, 2);
        assert_eq!(pos.offset, 2);

        // Test 3-byte character
        pos.advance('€');
        assert_eq!(pos.column, 3);
        assert_eq!(pos.offset, 5);
    }

    #[test]
    fn test_position_start() {
        let pos = Position::start();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 0);
    }

    #[test]
    fn test_parse_error_creation() {
        let err = ParseError::at(5, 10, "test error");
        // ParseError is an enum, we need to match on it
        match &err {
            ParseError::ParseError { position, message } => {
                assert_eq!(position.line, 5);
                assert_eq!(position.column, 10);
                assert_eq!(message, "test error");
            }
            _ => panic!("Expected ParseError variant"),
        }

        let display = format!("{}", err);
        assert!(display.contains("5"));
        assert!(display.contains("10"));
        assert!(display.contains("test error"));
    }

    #[test]
    fn test_parse_error_new() {
        let pos = Position::new(3, 7);
        let err = ParseError::new(pos, "new error");
        match &err {
            ParseError::ParseError { position, message } => {
                assert_eq!(position.line, 3);
                assert_eq!(position.column, 7);
                assert_eq!(message, "new error");
            }
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_parse_result_ok() {
        let result: ParseResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert!(!result.is_err());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_parse_result_err() {
        let result: ParseResult<i32> = Err(ParseError::at(1, 1, "error"));
        assert!(!result.is_ok());
        assert!(result.is_err());
    }

    #[test]
    fn test_with_nesting_wrapper() {
        let parser = with_nesting(|input: &str, pos: Position, _state: &mut NestingState| {
            if input.get(pos.offset..pos.offset + 1) == Some("x") {
                let mut new_pos = pos;
                new_pos.advance('x');
                Ok(("found x", new_pos))
            } else {
                Err(ParseError::at(pos.line, pos.column, "expected x"))
            }
        });

        let result = parser("x", Position::start());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "found x");

        let result = parser("y", Position::start());
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_content_empty() {
        let parser = with_nesting(nested_content('{', '}'));
        let result = parser("{}", Position::start());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "");
    }

    #[test]
    fn test_nested_content_deep_nesting() {
        let parser = with_nesting(nested_content('{', '}'));
        let result = parser("{{{{deep}}}}", Position::start());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "{{{deep}}}");
    }

    #[test]
    fn test_nested_content_wrong_close() {
        let parser = with_nesting(nested_content('{', '}'));
        // Opening with { but closing with ]
        let result = parser("{wrong]", Position::start());
        assert!(result.is_err());
    }

    #[test]
    fn test_position_advance_returning() {
        // Test that advance modifies the position correctly
        let mut pos = Position::start();
        pos.advance('a');

        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 2);
        assert_eq!(pos.offset, 1);
    }

    #[test]
    fn test_nesting_state_push_error_on_max_depth() {
        let mut state = NestingState::new();

        // Push to max depth (100)
        for _ in 0..100 {
            state.push('(').unwrap();
        }

        // Next push should fail
        assert!(state.push('(').is_err());
    }
}
