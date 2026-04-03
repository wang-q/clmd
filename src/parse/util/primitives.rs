//! Primitive parsers for common patterns.
//!
//! This module provides parsers for common patterns like strings, numbers,
//! identifiers, and more.

use crate::parse::util::char::digit;
use crate::parse::util::combinator::{many, many1};
use crate::parse::util::{BoxedParser, ClmdError, ClmdResult, Position};

/// Parse a string literal (double-quoted).
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::string;
///
/// let result = string.parse("\"hello world\"").unwrap();
/// assert_eq!(result, "hello world");
/// ```ignore
pub fn string(input: &str, pos: Position) -> ClmdResult<(String, Position)> {
    let mut current_pos = pos;

    // Opening quote
    if !input[current_pos.offset..].starts_with('"') {
        return Err(ClmdError::parse_error(current_pos, "Expected opening quote"));
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
                    return Err(ClmdError::parse_error(
                        current_pos,
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

    Err(ClmdError::parse_error(current_pos, "Unclosed string literal"))
}

/// Parse an identifier (starts with letter, followed by alphanumeric or _).
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::identifier;
///
/// let result = identifier.parse("hello_world").unwrap();
/// assert_eq!(result, "hello_world");
/// ```ignore
pub fn identifier(input: &str, pos: Position) -> ClmdResult<(String, Position)> {
    let mut current_pos = pos;

    // First character must be alphabetic
    let first = input[current_pos.offset..].chars().next().ok_or_else(|| {
        ClmdError::parse_error(current_pos, "Expected identifier")
    })?;

    if !first.is_alphabetic() && first != '_' {
        return Err(ClmdError::parse_error(
            current_pos,
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
/// use clmd::parse::util::uint;
///
/// let result = uint.parse("12345").unwrap();
/// assert_eq!(result, 12345u64);
/// ```ignore
pub fn uint(input: &str, pos: Position) -> ClmdResult<(u64, Position)> {
    let digits_parser = many1(Box::new(digit));
    let (digits, new_pos) = digits_parser(input, pos)?;

    let num_str: String = digits.into_iter().collect();
    num_str
        .parse::<u64>()
        .map(|n| Ok((n, new_pos)))
        .unwrap_or_else(|_| {
            Err(ClmdError::parse_error(new_pos, "Invalid unsigned integer"))
        })
}

/// Parse a signed integer.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::int;
///
/// assert_eq!(int.parse("-123").unwrap(), -123i64);
/// assert_eq!(int.parse("456").unwrap(), 456i64);
/// ```ignore
pub fn int(input: &str, pos: Position) -> ClmdResult<(i64, Position)> {
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
        .unwrap_or_else(|_| Err(ClmdError::parse_error(new_pos, "Invalid integer")))
}

/// Parse a floating-point number.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::float;
///
/// let result = float.parse("3.14159").unwrap();
/// assert!((result - 3.14159).abs() < 0.00001);
/// ```ignore
pub fn float(input: &str, pos: Position) -> ClmdResult<(f64, Position)> {
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
        return Err(ClmdError::parse_error(pos, "Expected float"));
    }

    full_num
        .parse::<f64>()
        .map(|n| Ok((n, current_pos)))
        .unwrap_or_else(|_| {
            Err(ClmdError::parse_error(current_pos, "Invalid float"))
        })
}

/// Parse whitespace (one or more characters).
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::whitespace1;
///
/// let result = whitespace1.parse("   hello").unwrap();
/// assert_eq!(result.len(), 3);
/// ```ignore
pub fn whitespace1(input: &str, pos: Position) -> ClmdResult<(String, Position)> {
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
        return Err(ClmdError::parse_error(pos, "Expected whitespace"));
    }

    Ok((result, current_pos))
}

/// Parse until a specific string is found.
///
/// # Example
///
/// ```ignore
/// use clmd::parse::util::until;
///
/// let parser = until("-->");
/// let result = parser.parse("Hello world-->").unwrap();
/// assert_eq!(result, "Hello world");
/// ```ignore
pub fn until(end: &'static str) -> BoxedParser<String> {
    Box::new(move |input: &str, pos: Position| {
        let mut current_pos = pos;
        let mut result = String::new();

        while !input[current_pos.offset..].starts_with(end) {
            if let Some(ch) = input[current_pos.offset..].chars().next() {
                result.push(ch);
                current_pos.advance(ch);
            } else {
                return Err(ClmdError::parse_error(
                    current_pos,
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
/// use clmd::parse::util::line_comment;
///
/// let parser = line_comment("//");
/// let result = parser.parse("// This is a comment\n").unwrap();
/// assert_eq!(result, " This is a comment");
/// ```ignore
pub fn line_comment(prefix: &'static str) -> BoxedParser<String> {
    Box::new(move |input: &str, pos: Position| {
        let mut current_pos = pos;

        if !input[current_pos.offset..].starts_with(prefix) {
            return Err(ClmdError::parse_error(
                current_pos,
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
/// use clmd::parse::util::block_comment;
///
/// let parser = block_comment("/*", "*/");
/// let result = parser.parse("/* comment */").unwrap();
/// assert_eq!(result, " comment ");
/// ```ignore
pub fn block_comment(start: &'static str, end: &'static str) -> BoxedParser<String> {
    Box::new(move |input: &str, pos: Position| {
        let mut current_pos = pos;

        if !input[current_pos.offset..].starts_with(start) {
            return Err(ClmdError::parse_error(
                current_pos,
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
                return Err(ClmdError::parse_error(
                    current_pos,
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

    #[test]
    fn test_string() {
        assert_eq!(string("\"hello\"", Position::start()).unwrap().0, "hello");
        assert_eq!(
            string("\"hello world\"", Position::start()).unwrap().0,
            "hello world"
        );
        assert_eq!(
            string("\"hello\\nworld\"", Position::start()).unwrap().0,
            "hello\nworld"
        );
        assert!(string("\"unclosed", Position::start()).is_err());
    }

    #[test]
    fn test_identifier() {
        assert_eq!(identifier("hello", Position::start()).unwrap().0, "hello");
        assert_eq!(
            identifier("hello_world", Position::start()).unwrap().0,
            "hello_world"
        );
        assert_eq!(
            identifier("_private", Position::start()).unwrap().0,
            "_private"
        );
        assert_eq!(
            identifier("test123", Position::start()).unwrap().0,
            "test123"
        );
        assert!(identifier("123test", Position::start()).is_err());
    }

    #[test]
    fn test_uint() {
        assert_eq!(uint("123", Position::start()).unwrap().0, 123u64);
        assert_eq!(uint("0", Position::start()).unwrap().0, 0u64);
        assert!(uint("-123", Position::start()).is_err());
        assert!(uint("abc", Position::start()).is_err());
    }

    #[test]
    fn test_int() {
        assert_eq!(int("123", Position::start()).unwrap().0, 123i64);
        assert_eq!(int("-123", Position::start()).unwrap().0, -123i64);
        assert_eq!(int("+456", Position::start()).unwrap().0, 456i64);
        assert!(int("abc", Position::start()).is_err());
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_float() {
        let (result, _) = float("3.14", Position::start()).unwrap();
        assert!((result - 3.14_f64).abs() < 0.0001);
        let (result, _) = float("-3.14", Position::start()).unwrap();
        assert!((result - (-3.14_f64)).abs() < 0.0001);
        let (result, _) = float("1e10", Position::start()).unwrap();
        assert!((result - 1e10_f64).abs() < 0.0001);
        let (result, _) = float("1.5e-3", Position::start()).unwrap();
        assert!((result - 0.0015_f64).abs() < 0.0001);
    }

    #[test]
    fn test_whitespace1() {
        let result = whitespace1("   hello", Position::start()).unwrap();
        assert_eq!(result.0.len(), 3);
        assert!(whitespace1("hello", Position::start()).is_err());
    }

    #[test]
    fn test_until() {
        let parser = until("-->");
        let result = parser("Hello world-->", Position::start()).unwrap();
        assert_eq!(result.0, "Hello world");
    }

    #[test]
    fn test_line_comment() {
        let parser = line_comment("//");
        assert_eq!(
            parser("// comment\n", Position::start()).unwrap().0,
            " comment"
        );
        assert_eq!(
            parser("// comment", Position::start()).unwrap().0,
            " comment"
        );
    }

    #[test]
    fn test_block_comment() {
        let parser = block_comment("/*", "*/");
        assert_eq!(
            parser("/* comment */", Position::start()).unwrap().0,
            " comment "
        );
        assert!(parser("/* unclosed", Position::start()).is_err());
    }
}
