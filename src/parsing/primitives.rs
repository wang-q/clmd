//! Primitive parsers for common patterns.
//!
//! This module provides parsers for common patterns like strings, numbers,
//! identifiers, and more.

use super::{BoxedParser, ParseError, ParseResult, Position};
use crate::parsing::char::digit;
use crate::parsing::combinator::{many, many1};

/// Parse a string literal (double-quoted).
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::string;
///
/// let result = string.parse("\"hello world\"").unwrap();
/// assert_eq!(result, "hello world");
/// ```
pub fn string(input: &str, pos: Position) -> ParseResult<String> {
    let mut current_pos = pos;

    // Opening quote
    if !input[current_pos.offset..].starts_with('"') {
        return Err(ParseError::at(
            current_pos.line,
            current_pos.column,
            "Expected opening quote",
        ));
    }
    current_pos.advance('"');

    let mut result = String::new();

    while let Some(ch) = input[current_pos.offset..].chars().next() {
        match ch {
            '"' => {
                current_pos.advance('"');
                return Ok((result, current_pos));
            }
            '\\' => {
                current_pos.advance('\\');
                if let Some(escaped) = input[current_pos.offset..].chars().next() {
                    let unescaped = match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        c => c,
                    };
                    result.push(unescaped);
                    current_pos.advance(escaped);
                } else {
                    return Err(ParseError::at(
                        current_pos.line,
                        current_pos.column,
                        "Unexpected end of string",
                    ));
                }
            }
            c => {
                result.push(c);
                current_pos.advance(c);
            }
        }
    }

    Err(ParseError::at(
        current_pos.line,
        current_pos.column,
        "Unclosed string literal",
    ))
}

/// Parse an identifier (starts with letter, followed by alphanumeric or _).
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::identifier;
///
/// let result = identifier.parse("hello_world").unwrap();
/// assert_eq!(result, "hello_world");
/// ```
pub fn identifier(input: &str, pos: Position) -> ParseResult<String> {
    let mut current_pos = pos;

    // First character must be alphabetic
    let first = input[current_pos.offset..].chars().next().ok_or_else(|| {
        ParseError::at(current_pos.line, current_pos.column, "Expected identifier")
    })?;

    if !first.is_alphabetic() && first != '_' {
        return Err(ParseError::at(
            current_pos.line,
            current_pos.column,
            "Identifier must start with letter or underscore",
        ));
    }

    let mut result = String::new();
    result.push(first);
    current_pos.advance(first);

    // Rest can be alphanumeric or underscore
    while let Some(ch) = input[current_pos.offset..].chars().next() {
        if ch.is_alphanumeric() || ch == '_' {
            result.push(ch);
            current_pos.advance(ch);
        } else {
            break;
        }
    }

    Ok((result, current_pos))
}

/// Parse an unsigned integer.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::uint;
///
/// let result = uint.parse("12345").unwrap();
/// assert_eq!(result, 12345u64);
/// ```
pub fn uint(input: &str, pos: Position) -> ParseResult<u64> {
    let digits_parser = many1(Box::new(digit));
    let (digits, new_pos) = digits_parser(input, pos)?;

    let num_str: String = digits.into_iter().collect();
    num_str
        .parse::<u64>()
        .map(|n| Ok((n, new_pos)))
        .unwrap_or_else(|_| {
            Err(ParseError::at(
                new_pos.line,
                new_pos.column,
                "Invalid unsigned integer",
            ))
        })
}

/// Parse a signed integer.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::int;
///
/// assert_eq!(int.parse("-123").unwrap(), -123i64);
/// assert_eq!(int.parse("456").unwrap(), 456i64);
/// ```
pub fn int(input: &str, pos: Position) -> ParseResult<i64> {
    let mut current_pos = pos;

    // Optional sign
    let negative = if input[current_pos.offset..].starts_with('-') {
        current_pos.advance('-');
        true
    } else if input[current_pos.offset..].starts_with('+') {
        current_pos.advance('+');
        false
    } else {
        false
    };

    let (digits, new_pos) = many1(Box::new(digit))(input, current_pos)?;
    let num_str: String = digits.into_iter().collect();

    num_str
        .parse::<i64>()
        .map(|n| {
            let result = if negative { -n } else { n };
            Ok((result, new_pos))
        })
        .unwrap_or_else(|_| {
            Err(ParseError::at(
                new_pos.line,
                new_pos.column,
                "Invalid integer",
            ))
        })
}

/// Parse a floating-point number.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::float;
///
/// let result = float.parse("3.14159").unwrap();
/// assert!((result - 3.14159).abs() < 0.00001);
/// ```
pub fn float(input: &str, pos: Position) -> ParseResult<f64> {
    let mut current_pos = pos;
    let mut full_num = String::new();

    // Optional sign
    if input[current_pos.offset..].starts_with('-') {
        full_num.push('-');
        current_pos.advance('-');
    } else if input[current_pos.offset..].starts_with('+') {
        full_num.push('+');
        current_pos.advance('+');
    }

    // Integer part
    let (int_digits, new_pos) = many(Box::new(digit))(input, current_pos)?;
    let int_part: String = int_digits.into_iter().collect();
    full_num.push_str(&int_part);
    current_pos = new_pos;

    // Decimal part
    if input[current_pos.offset..].starts_with('.') {
        full_num.push('.');
        current_pos.advance('.');
        let (frac_digits, new_pos) = many1(Box::new(digit))(input, current_pos)?;
        let frac_part: String = frac_digits.into_iter().collect();
        full_num.push_str(&frac_part);
        current_pos = new_pos;
    }

    // Exponent part
    if input[current_pos.offset..].starts_with('e')
        || input[current_pos.offset..].starts_with('E')
    {
        full_num.push('e');
        let exp_char = input[current_pos.offset..].chars().next().unwrap();
        current_pos.advance(exp_char);

        // Optional sign for exponent
        if input[current_pos.offset..].starts_with('-') {
            full_num.push('-');
            current_pos.advance('-');
        } else if input[current_pos.offset..].starts_with('+') {
            full_num.push('+');
            current_pos.advance('+');
        }

        let (exp_digits, new_pos) = many1(Box::new(digit))(input, current_pos)?;
        let exp_part: String = exp_digits.into_iter().collect();
        full_num.push_str(&exp_part);
        current_pos = new_pos;
    }

    if full_num.is_empty() || full_num == "." || full_num == "-" || full_num == "+" {
        return Err(ParseError::at(pos.line, pos.column, "Expected float"));
    }

    full_num
        .parse::<f64>()
        .map(|n| Ok((n, current_pos)))
        .unwrap_or_else(|_| {
            Err(ParseError::at(
                current_pos.line,
                current_pos.column,
                "Invalid float",
            ))
        })
}

/// Parse whitespace (one or more characters).
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::whitespace1;
///
/// let result = whitespace1.parse("   hello").unwrap();
/// assert_eq!(result.len(), 3);
/// ```
pub fn whitespace1(input: &str, pos: Position) -> ParseResult<String> {
    let mut current_pos = pos;
    let mut result = String::new();

    while let Some(ch) = input[current_pos.offset..].chars().next() {
        if ch.is_whitespace() {
            result.push(ch);
            current_pos.advance(ch);
        } else {
            break;
        }
    }

    if result.is_empty() {
        return Err(ParseError::at(pos.line, pos.column, "Expected whitespace"));
    }

    Ok((result, current_pos))
}

/// Parse until a specific string is found.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::until;
///
/// let parser = until("-->");
/// let result = parser.parse("Hello world-->").unwrap();
/// assert_eq!(result, "Hello world");
/// ```
pub fn until(end: &'static str) -> BoxedParser<String> {
    Box::new(move |input: &str, pos: Position| {
        let mut current_pos = pos;
        let mut result = String::new();

        while !input[current_pos.offset..].starts_with(end) {
            if let Some(ch) = input[current_pos.offset..].chars().next() {
                result.push(ch);
                current_pos.advance(ch);
            } else {
                return Err(ParseError::at(
                    current_pos.line,
                    current_pos.column,
                    format!("Expected '{}'", end),
                ));
            }
        }

        Ok((result, current_pos))
    })
}

/// Parse a line comment starting with the given prefix.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::line_comment;
///
/// let parser = line_comment("//");
/// let result = parser.parse("// This is a comment\n").unwrap();
/// assert_eq!(result, " This is a comment");
/// ```
pub fn line_comment(prefix: &'static str) -> BoxedParser<String> {
    Box::new(move |input: &str, pos: Position| {
        let mut current_pos = pos;

        if !input[current_pos.offset..].starts_with(prefix) {
            return Err(ParseError::at(
                current_pos.line,
                current_pos.column,
                format!("Expected '{}'", prefix),
            ));
        }

        for _ in 0..prefix.len() {
            if let Some(ch) = input[current_pos.offset..].chars().next() {
                current_pos.advance(ch);
            }
        }

        let mut result = String::new();
        while let Some(ch) = input[current_pos.offset..].chars().next() {
            if ch == '\n' {
                break;
            }
            result.push(ch);
            current_pos.advance(ch);
        }

        Ok((result, current_pos))
    })
}

/// Parse a block comment with start and end markers.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::block_comment;
///
/// let parser = block_comment("/*", "*/");
/// let result = parser.parse("/* comment */").unwrap();
/// assert_eq!(result, " comment ");
/// ```
pub fn block_comment(start: &'static str, end: &'static str) -> BoxedParser<String> {
    Box::new(move |input: &str, pos: Position| {
        let mut current_pos = pos;

        if !input[current_pos.offset..].starts_with(start) {
            return Err(ParseError::at(
                current_pos.line,
                current_pos.column,
                format!("Expected '{}'", start),
            ));
        }

        for _ in 0..start.len() {
            if let Some(ch) = input[current_pos.offset..].chars().next() {
                current_pos.advance(ch);
            }
        }

        let mut result = String::new();
        while !input[current_pos.offset..].starts_with(end) {
            if let Some(ch) = input[current_pos.offset..].chars().next() {
                result.push(ch);
                current_pos.advance(ch);
            } else {
                return Err(ParseError::at(
                    current_pos.line,
                    current_pos.column,
                    format!("Unclosed block comment, expected '{}'", end),
                ));
            }
        }

        for _ in 0..end.len() {
            if let Some(ch) = input[current_pos.offset..].chars().next() {
                current_pos.advance(ch);
            }
        }

        Ok((result, current_pos))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::Parser;

    #[test]
    fn test_string() {
        assert_eq!(string.parse("\"hello\"").unwrap(), "hello");
        assert_eq!(string.parse("\"hello world\"").unwrap(), "hello world");
        assert_eq!(string.parse("\"hello\\nworld\"").unwrap(), "hello\nworld");
        assert!(string.parse("\"unclosed").is_err());
    }

    #[test]
    fn test_identifier() {
        assert_eq!(identifier.parse("hello").unwrap(), "hello");
        assert_eq!(identifier.parse("hello_world").unwrap(), "hello_world");
        assert_eq!(identifier.parse("_private").unwrap(), "_private");
        assert_eq!(identifier.parse("test123").unwrap(), "test123");
        assert!(identifier.parse("123test").is_err());
    }

    #[test]
    fn test_uint() {
        assert_eq!(uint.parse("123").unwrap(), 123u64);
        assert_eq!(uint.parse("0").unwrap(), 0u64);
        assert!(uint.parse("-123").is_err());
        assert!(uint.parse("abc").is_err());
    }

    #[test]
    fn test_int() {
        assert_eq!(int.parse("123").unwrap(), 123i64);
        assert_eq!(int.parse("-123").unwrap(), -123i64);
        assert_eq!(int.parse("+456").unwrap(), 456i64);
        assert!(int.parse("abc").is_err());
    }

    #[test]
    fn test_float() {
        let (result, _) = float.parse_partial("3.14").unwrap();
        assert!((result - 3.14).abs() < 0.0001);
        let (result, _) = float.parse_partial("-3.14").unwrap();
        assert!((result - (-3.14)).abs() < 0.0001);
        let (result, _) = float.parse_partial("1e10").unwrap();
        assert!((result - 1e10).abs() < 0.0001);
        let (result, _) = float.parse_partial("1.5e-3").unwrap();
        assert!((result - 0.0015).abs() < 0.0001);
    }

    #[test]
    fn test_whitespace1() {
        let result = whitespace1.parse_partial("   hello").unwrap();
        assert_eq!(result.0.len(), 3);
        assert!(whitespace1.parse("hello").is_err());
    }

    #[test]
    fn test_until() {
        let parser = until("-->");
        let result = parser.parse_partial("Hello world-->").unwrap();
        assert_eq!(result.0, "Hello world");
        assert!(parser.parse("Hello world-->").is_err()); // parse requires full consumption
    }

    #[test]
    fn test_line_comment() {
        let parser = line_comment("//");
        assert_eq!(parser.parse("// comment\n").unwrap(), " comment");
        assert_eq!(parser.parse("// comment").unwrap(), " comment");
    }

    #[test]
    fn test_block_comment() {
        let parser = block_comment("/*", "*/");
        assert_eq!(parser.parse("/* comment */").unwrap(), " comment ");
        assert!(parser.parse("/* unclosed").is_err());
    }
}
