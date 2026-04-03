//! Parser combinators for building complex parsers.
//!
//! This module provides combinators for combining parsers, inspired by
//! Pandoc's parsing infrastructure and functional parsing libraries.

use crate::parse::util::{BoxedParser, ClmdError, ClmdResult, Position};

/// Parse with the first successful parser.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{choice, char_lit, digit, Parser};
///
/// let parser = choice(vec![char_lit('a'), char_lit('b'), digit]);
/// assert!(parser.parse("a").is_ok());
/// assert!(parser.parse("b").is_ok());
/// assert!(parser.parse("1").is_ok());
/// assert!(parser.parse("x").is_err());
/// ```ignore
pub fn choice<T>(parsers: Vec<BoxedParser<T>>) -> BoxedParser<T>
where
    T: 'static,
{
    Box::new(move |input: &str, pos: Position| {
        let mut last_error = None;
        for parser in &parsers {
            match parser(input, pos) {
                Ok(result) => return Ok(result),
                Err(e) => last_error = Some(e),
            }
        }
        Err(last_error.unwrap_or_else(|| {
            ClmdError::parse_error(pos, "No parser succeeded in choice")
        }))
    })
}

/// Parse a sequence of parsers, returning the result of the first.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{seq, char_lit, Parser};
///
/// let parser = seq(vec![char_lit('a'), char_lit('b')]);
/// assert_eq!(parser.parse("ab").unwrap(), 'a');
/// ```ignore
pub fn seq<T>(parsers: Vec<BoxedParser<T>>) -> BoxedParser<T>
where
    T: 'static,
{
    Box::new(move |input: &str, mut pos: Position| {
        let mut first_result = None;
        for parser in &parsers {
            match parser(input, pos) {
                Ok((result, new_pos)) => {
                    if first_result.is_none() {
                        first_result = Some(result);
                    }
                    pos = new_pos;
                }
                Err(e) => return Err(e),
            }
        }
        first_result.map(|r| Ok((r, pos))).unwrap_or_else(|| {
            Err(ClmdError::parse_error(pos, "Empty sequence"))
        })
    })
}

/// Parse with two parsers and combine their results.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{pair, char_lit, digit, Parser};
///
/// let parser = pair(char_lit('a'), digit);
/// let (letter, num) = parser.parse("a1").unwrap();
/// assert_eq!(letter, 'a');
/// assert_eq!(num, '1');
/// ```ignore
pub fn pair<A, B>(first: BoxedParser<A>, second: BoxedParser<B>) -> BoxedParser<(A, B)>
where
    A: 'static,
    B: 'static,
{
    Box::new(move |input: &str, pos: Position| {
        let (a, pos1) = first(input, pos)?;
        let (b, pos2) = second(input, pos1)?;
        Ok(((a, b), pos2))
    })
}

/// Parse with two parsers, returning the result of the second.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{right, char_lit, digit, Parser};
///
/// let parser = right(char_lit('a'), digit);
/// assert_eq!(parser.parse("a1").unwrap(), '1');
/// ```ignore
pub fn right<L, R>(left: BoxedParser<L>, right: BoxedParser<R>) -> BoxedParser<R>
where
    L: 'static,
    R: 'static,
{
    Box::new(move |input: &str, pos: Position| {
        let (_, pos1) = left(input, pos)?;
        right(input, pos1)
    })
}

/// Parse with two parsers, returning the result of the first.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{left, char_lit, digit, Parser};
///
/// let parser = left(char_lit('a'), digit);
/// assert_eq!(parser.parse("a1").unwrap(), 'a');
/// ```ignore
pub fn left<L, R>(left: BoxedParser<L>, right: BoxedParser<R>) -> BoxedParser<L>
where
    L: 'static,
    R: 'static,
{
    Box::new(move |input: &str, pos: Position| {
        let (l, pos1) = left(input, pos)?;
        let (_, pos2) = right(input, pos1)?;
        Ok((l, pos2))
    })
}

/// Parse with a parser between two other parsers.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{between, char_lit, digit, Parser};
///
/// let parser = between(char_lit('('), char_lit(')'), digit);
/// assert_eq!(parser.parse("(1)").unwrap(), '1');
/// ```ignore
pub fn between<O, L, R>(
    open: BoxedParser<L>,
    close: BoxedParser<R>,
    parser: BoxedParser<O>,
) -> BoxedParser<O>
where
    O: 'static,
    L: 'static,
    R: 'static,
{
    Box::new(move |input: &str, pos: Position| {
        let (_, pos1) = open(input, pos)?;
        let (result, pos2) = parser(input, pos1)?;
        let (_, pos3) = close(input, pos2)?;
        Ok((result, pos3))
    })
}

/// Parse zero or more occurrences.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{many, digit, Parser};
///
/// let parser = many(digit);
/// let result = parser.parse("123abc").unwrap();
/// assert_eq!(result, vec!['1', '2', '3']);
/// ```ignore
pub fn many<T>(parser: BoxedParser<T>) -> BoxedParser<Vec<T>>
where
    T: 'static,
{
    Box::new(move |input: &str, mut pos: Position| {
        let mut results = Vec::new();
        while let Ok((result, new_pos)) = parser(input, pos) {
            results.push(result);
            pos = new_pos;
        }
        Ok((results, pos))
    })
}

/// Parse one or more occurrences.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{many1, digit, Parser};
///
/// let parser = many1(digit);
/// let result = parser.parse("123abc").unwrap();
/// assert_eq!(result, vec!['1', '2', '3']);
/// assert!(parser.parse("abc").is_err());
/// ```ignore
pub fn many1<T>(parser: BoxedParser<T>) -> BoxedParser<Vec<T>>
where
    T: 'static,
{
    Box::new(move |input: &str, pos: Position| {
        let (first, mut current_pos) = parser(input, pos)?;
        let mut results = vec![first];

        while let Ok((result, new_pos)) = parser(input, current_pos) {
            results.push(result);
            current_pos = new_pos;
        }

        Ok((results, current_pos))
    })
}

/// Parse an optional value.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{optional, char_lit, Parser};
///
/// let parser = optional(char_lit('a'));
/// assert_eq!(parser.parse("abc").unwrap(), Some('a'));
/// assert_eq!(parser.parse("xyz").unwrap(), None);
/// ```ignore
pub fn optional<T>(parser: BoxedParser<T>) -> BoxedParser<Option<T>>
where
    T: 'static,
{
    Box::new(move |input: &str, pos: Position| match parser(input, pos) {
        Ok((result, new_pos)) => Ok((Some(result), new_pos)),
        Err(_) => Ok((None, pos)),
    })
}

/// Parse with two parsers and apply a function to combine results.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{map2, char_lit, digit, Parser};
///
/// let parser = map2(char_lit('a'), digit, |a, d| format!("{}{}", a, d));
/// assert_eq!(parser.parse("a1").unwrap(), "a1");
/// ```ignore
pub fn map2<A, B, F, R>(
    first: BoxedParser<A>,
    second: BoxedParser<B>,
    f: F,
) -> BoxedParser<R>
where
    A: 'static,
    B: 'static,
    F: Fn(A, B) -> R + 'static,
    R: 'static,
{
    Box::new(move |input: &str, pos: Position| {
        let (a, pos1) = first(input, pos)?;
        let (b, pos2) = second(input, pos1)?;
        Ok((f(a, b), pos2))
    })
}

/// Parse with three parsers and apply a function to combine results.
pub fn map3<A, B, C, F, R>(
    first: BoxedParser<A>,
    second: BoxedParser<B>,
    third: BoxedParser<C>,
    f: F,
) -> BoxedParser<R>
where
    A: 'static,
    B: 'static,
    C: 'static,
    F: Fn(A, B, C) -> R + 'static,
    R: 'static,
{
    Box::new(move |input: &str, pos: Position| {
        let (a, pos1) = first(input, pos)?;
        let (b, pos2) = second(input, pos1)?;
        let (c, pos3) = third(input, pos2)?;
        Ok((f(a, b, c), pos3))
    })
}

/// Parse zero or more occurrences separated by a delimiter.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{separated_by, char_lit, digit, Parser};
///
/// let parser = separated_by(digit, char_lit(','));
/// let result = parser.parse("1,2,3").unwrap();
/// assert_eq!(result, vec!['1', '2', '3']);
/// ```ignore
pub fn separated_by<T, S>(
    parser: BoxedParser<T>,
    sep: BoxedParser<S>,
) -> BoxedParser<Vec<T>>
where
    T: 'static,
    S: 'static,
{
    Box::new(move |input: &str, pos: Position| {
        let mut results = Vec::new();
        let mut current_pos = pos;

        // Parse first element
        match parser(input, current_pos) {
            Ok((result, new_pos)) => {
                results.push(result);
                current_pos = new_pos;
            }
            Err(_) => return Ok((results, current_pos)),
        }

        // Parse (separator, element)*
        while let Ok((_, sep_pos)) = sep(input, current_pos) {
            match parser(input, sep_pos) {
                Ok((result, elem_pos)) => {
                    results.push(result);
                    current_pos = elem_pos;
                }
                Err(_) => break,
            }
        }

        Ok((results, current_pos))
    })
}

/// Parse one or more occurrences separated by a delimiter.
pub fn separated_by1<T, S>(
    parser: BoxedParser<T>,
    sep: BoxedParser<S>,
) -> BoxedParser<Vec<T>>
where
    T: 'static,
    S: 'static,
{
    Box::new(move |input: &str, pos: Position| {
        let (first, mut current_pos) = parser(input, pos)?;
        let mut results = vec![first];

        while let Ok((_, sep_pos)) = sep(input, current_pos) {
            match parser(input, sep_pos) {
                Ok((result, elem_pos)) => {
                    results.push(result);
                    current_pos = elem_pos;
                }
                Err(_) => break,
            }
        }

        Ok((results, current_pos))
    })
}

/// Skip whitespace before parsing.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{skip_whitespace, char_lit, Parser};
///
/// let parser = skip_whitespace(char_lit('a'));
/// assert_eq!(parser.parse("   a").unwrap(), 'a');
/// ```ignore
pub fn skip_whitespace<T>(parser: BoxedParser<T>) -> BoxedParser<T>
where
    T: 'static,
{
    Box::new(move |input: &str, pos: Position| {
        let mut current_pos = pos;
        while let Some(ch) = input[current_pos.offset..].chars().next() {
            if ch.is_whitespace() {
                current_pos.advance(ch);
            } else {
                break;
            }
        }
        parser(input, current_pos)
    })
}

/// Parse end of input.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{eof, Parser};
///
/// assert!(eof.parse("").is_ok());
/// assert!(eof.parse("a").is_err());
/// ```ignore
pub fn eof(input: &str, pos: Position) -> ClmdResult<((), Position)> {
    if pos.offset >= input.len() {
        Ok(((), pos))
    } else {
        Err(ClmdError::parse_error(pos, "Expected end of input"))
    }
}

/// Always succeed with a value, consuming no input.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{success, char_lit, Parser};
///
/// let parser = success(42);
/// assert_eq!(parser.parse("anything").unwrap(), 42);
/// ```ignore
pub fn success<T: Clone + 'static>(value: T) -> BoxedParser<T> {
    Box::new(move |_input: &str, pos: Position| Ok((value.clone(), pos)))
}

/// Always fail with a message.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{failure, Parser};
///
/// let parser = failure("custom error");
/// assert!(parser.parse("").is_err());
/// ```ignore
pub fn failure<T>(message: &'static str) -> BoxedParser<T>
where
    T: 'static,
{
    Box::new(move |_input: &str, pos: Position| {
        Err(ClmdError::parse_error(pos, message))
    })
}

/// Try a parser, returning None on failure without consuming input.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{peek, char_lit, Parser};
///
/// let parser = peek(char_lit('a'));
/// assert_eq!(parser.parse("abc").unwrap(), Some('a'));
/// // Input not consumed
/// let result = parser.parse_partial("abc").unwrap();
/// assert_eq!(result.1.offset, 0);
/// ```ignore
pub fn peek<T: Clone + 'static>(parser: BoxedParser<T>) -> BoxedParser<Option<T>> {
    Box::new(move |input: &str, pos: Position| match parser(input, pos) {
        Ok((result, _)) => Ok((Some(result), pos)),
        Err(_) => Ok((None, pos)),
    })
}

/// Not followed by - succeed only if the second parser fails.
///
/// # Example
///
/// ```ignore
/// use clmd::parsing::{not_followed_by, char_lit, Parser};
///
/// let parser = not_followed_by(char_lit('a'), char_lit('b'));
/// assert!(parser.parse("ac").is_ok());
/// assert!(parser.parse("ab").is_err());
/// ```ignore
pub fn not_followed_by<T, U>(
    parser: BoxedParser<T>,
    not_followed: BoxedParser<U>,
) -> BoxedParser<T>
where
    T: 'static,
    U: 'static,
{
    Box::new(move |input: &str, pos: Position| {
        let (result, new_pos) = parser(input, pos)?;
        match not_followed(input, new_pos) {
            Ok(_) => Err(ClmdError::parse_error(
                new_pos,
                "Unexpected following pattern",
            )),
            Err(_) => Ok((result, new_pos)),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::util::char::{char_lit, digit};
    use crate::parse::util::Parser;

    #[test]
    fn test_choice() {
        let parser = choice(vec![
            Box::new(char_lit('a')),
            Box::new(char_lit('b')),
            Box::new(digit),
        ]);
        assert_eq!(parser.parse("a").unwrap(), 'a');
        assert_eq!(parser.parse("b").unwrap(), 'b');
        assert_eq!(parser.parse("1").unwrap(), '1');
        assert!(parser.parse("x").is_err());
    }

    #[test]
    fn test_pair() {
        let parser = pair(Box::new(char_lit('a')), Box::new(digit));
        let (a, d) = parser.parse("a1").unwrap();
        assert_eq!(a, 'a');
        assert_eq!(d, '1');
    }

    #[test]
    fn test_right() {
        let parser = right(Box::new(char_lit('a')), Box::new(digit));
        assert_eq!(parser.parse("a1").unwrap(), '1');
    }

    #[test]
    fn test_left() {
        let parser = left(Box::new(char_lit('a')), Box::new(digit));
        assert_eq!(parser.parse("a1").unwrap(), 'a');
    }

    #[test]
    fn test_between() {
        let parser = between(
            Box::new(char_lit('(')),
            Box::new(char_lit(')')),
            Box::new(digit),
        );
        assert_eq!(parser.parse("(1)").unwrap(), '1');
        assert!(parser.parse("(a)").is_err());
        assert!(parser.parse("1").is_err());
    }

    #[test]
    fn test_many() {
        let parser = many(Box::new(digit));
        let result = parser.parse_partial("123abc").unwrap();
        assert_eq!(result.0, vec!['1', '2', '3']);
        let result = parser.parse_partial("abc").unwrap();
        assert_eq!(result.0, vec![]);
    }

    #[test]
    fn test_many1() {
        let parser = many1(Box::new(digit));
        let result = parser.parse_partial("123abc").unwrap();
        assert_eq!(result.0, vec!['1', '2', '3']);
        assert!(parser.parse("abc").is_err());
    }

    #[test]
    fn test_optional() {
        let parser = optional(Box::new(char_lit('a')));
        let result = parser.parse_partial("abc").unwrap();
        assert_eq!(result.0, Some('a'));
        let result = parser.parse_partial("xyz").unwrap();
        assert_eq!(result.0, None);
    }

    #[test]
    fn test_separated_by() {
        let parser = separated_by(Box::new(digit), Box::new(char_lit(',')));
        assert_eq!(parser.parse("1,2,3").unwrap(), vec!['1', '2', '3']);
        assert_eq!(parser.parse("1").unwrap(), vec!['1']);
        let result = parser.parse_partial("").unwrap();
        assert_eq!(result.0, vec![]);
    }

    #[test]
    fn test_skip_whitespace() {
        let parser = skip_whitespace(Box::new(char_lit('a')));
        assert_eq!(parser.parse("   a").unwrap(), 'a');
        assert_eq!(parser.parse("a").unwrap(), 'a');
    }

    #[test]
    fn test_eof() {
        assert!(eof("", Position::start()).is_ok());
        assert!(eof("a", Position::start()).is_err());
    }

    #[test]
    fn test_success() {
        let parser = success(42);
        let result = parser.parse_partial("anything").unwrap();
        assert_eq!(result.0, 42);
    }

    #[test]
    fn test_failure() {
        let parser: BoxedParser<char> = failure("custom error");
        assert!(parser.parse("").is_err());
    }

    #[test]
    fn test_seq() {
        let parser = seq(vec![
            Box::new(char_lit('a')),
            Box::new(char_lit('b')),
            Box::new(char_lit('c')),
        ]);
        let result = parser.parse("abc").unwrap();
        assert_eq!(result, 'a');
    }

    #[test]
    fn test_seq_empty() {
        let parser: BoxedParser<char> = seq(vec![]);
        assert!(parser.parse("").is_err());
    }

    #[test]
    fn test_seq_failure() {
        let parser = seq(vec![Box::new(char_lit('a')), Box::new(char_lit('b'))]);
        assert!(parser.parse("ax").is_err());
    }

    #[test]
    fn test_map2() {
        let parser = map2(Box::new(char_lit('a')), Box::new(digit), |a, d| {
            format!("{}{}", a, d)
        });
        let result = parser.parse("a1").unwrap();
        assert_eq!(result, "a1");
    }

    #[test]
    fn test_map3() {
        let parser = map3(
            Box::new(char_lit('a')),
            Box::new(digit),
            Box::new(char_lit('b')),
            |a, d, b| format!("{}{}{}", a, d, b),
        );
        let result = parser.parse("a1b").unwrap();
        assert_eq!(result, "a1b");
    }

    #[test]
    fn test_separated_by1() {
        let parser = separated_by1(Box::new(digit), Box::new(char_lit(',')));
        let result = parser.parse("1,2,3").unwrap();
        assert_eq!(result, vec!['1', '2', '3']);
        assert!(parser.parse("").is_err());
        assert!(parser.parse("abc").is_err());
    }

    #[test]
    fn test_separated_by1_single() {
        let parser = separated_by1(Box::new(digit), Box::new(char_lit(',')));
        let result = parser.parse("1").unwrap();
        assert_eq!(result, vec!['1']);
    }

    #[test]
    fn test_peek() {
        let parser = peek(Box::new(char_lit('a')));
        let result = parser.parse_partial("abc").unwrap();
        assert_eq!(result.0, Some('a'));
        assert_eq!(result.1.offset, 0);
    }

    #[test]
    fn test_peek_failure() {
        let parser = peek(Box::new(char_lit('a')));
        let result = parser.parse_partial("xyz").unwrap();
        assert_eq!(result.0, None);
        assert_eq!(result.1.offset, 0);
    }

    #[test]
    fn test_not_followed_by() {
        let parser = not_followed_by(Box::new(char_lit('a')), Box::new(char_lit('b')));
        assert!(parser.parse("ac").is_ok());
        assert!(parser.parse("ab").is_err());
    }

    #[test]
    fn test_choice_empty() {
        let parser: BoxedParser<char> = choice(vec![]);
        assert!(parser.parse("a").is_err());
    }

    #[test]
    fn test_between_failure_open() {
        let parser = between(
            Box::new(char_lit('(')),
            Box::new(char_lit(')')),
            Box::new(digit),
        );
        assert!(parser.parse("1)").is_err());
    }

    #[test]
    fn test_between_failure_close() {
        let parser = between(
            Box::new(char_lit('(')),
            Box::new(char_lit(')')),
            Box::new(digit),
        );
        assert!(parser.parse("(1").is_err());
    }

    #[test]
    fn test_many_empty() {
        let parser = many(Box::new(digit));
        let result = parser.parse_partial("").unwrap();
        assert_eq!(result.0, vec![]);
    }

    #[test]
    fn test_optional_at_end() {
        let parser = optional(Box::new(char_lit('a')));
        let result = parser.parse_partial("a").unwrap();
        assert_eq!(result.0, Some('a'));
        assert_eq!(result.1.offset, 1);
    }

    #[test]
    fn test_separated_by_trailing() {
        let parser = separated_by(Box::new(digit), Box::new(char_lit(',')));
        let result = parser.parse_partial("1,2,").unwrap();
        assert_eq!(result.0, vec!['1', '2']);
    }

    #[test]
    fn test_skip_whitespace_no_whitespace() {
        let parser = skip_whitespace(Box::new(char_lit('a')));
        let result = parser.parse("a").unwrap();
        assert_eq!(result, 'a');
    }

    #[test]
    fn test_success_does_not_consume() {
        let parser = success(42);
        let result = parser.parse_partial("hello").unwrap();
        assert_eq!(result.0, 42);
        assert_eq!(result.1.offset, 0);
    }
}
