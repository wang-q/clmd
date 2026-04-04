//! TeX token types and utilities for clmd.
//!
//! This module provides TeX/LaTeX token types and parsing utilities,
//! inspired by Pandoc's Text.Pandoc.TeX module.
//!
//! # Example
//!
//! ```
//! use clmd::io::format::tex::{Token, TokenType};
//!
//! let token = Token::word("hello");
//! assert!(matches!(token.token_type, TokenType::Word));
//! ```

use std::fmt;

/// A TeX token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The type of token.
    pub token_type: TokenType,
    /// The literal text of the token.
    pub literal: String,
    /// Line number in source.
    pub line: usize,
    /// Column number in source.
    pub column: usize,
}

impl Token {
    /// Create a new token.
    pub fn new(token_type: TokenType, literal: impl Into<String>) -> Self {
        Self {
            token_type,
            literal: literal.into(),
            line: 0,
            column: 0,
        }
    }

    /// Create a word token.
    pub fn word(text: impl Into<String>) -> Self {
        Self::new(TokenType::Word, text)
    }

    /// Create a control sequence token.
    pub fn control_sequence(name: impl Into<String>) -> Self {
        Self::new(TokenType::ControlSequence, name)
    }

    /// Create a symbol token.
    pub fn symbol(c: char) -> Self {
        Self::new(TokenType::Symbol, c.to_string())
    }

    /// Create a whitespace token.
    pub fn whitespace(text: impl Into<String>) -> Self {
        Self::new(TokenType::Whitespace, text)
    }

    /// Create a comment token.
    pub fn comment(text: impl Into<String>) -> Self {
        Self::new(TokenType::Comment, text)
    }

    /// Create a newline token.
    pub fn newline() -> Self {
        Self::new(TokenType::Newline, "\n")
    }

    /// Set source position.
    pub fn with_position(mut self, line: usize, column: usize) -> Self {
        self.line = line;
        self.column = column;
        self
    }

    /// Check if this token is a control sequence.
    pub fn is_control_sequence(&self) -> bool {
        matches!(self.token_type, TokenType::ControlSequence)
    }

    /// Check if this token is whitespace.
    pub fn is_whitespace(&self) -> bool {
        matches!(self.token_type, TokenType::Whitespace | TokenType::Newline)
    }

    /// Check if this token is a comment.
    pub fn is_comment(&self) -> bool {
        matches!(self.token_type, TokenType::Comment)
    }

    /// Get the token as a string slice.
    pub fn as_str(&self) -> &str {
        &self.literal
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.literal)
    }
}

/// Types of TeX tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    /// A word (sequence of letters).
    Word,
    /// A control sequence (backslash + name).
    ControlSequence,
    /// A single character symbol.
    Symbol,
    /// Whitespace (spaces, tabs).
    Whitespace,
    /// A newline.
    Newline,
    /// A comment (% to end of line).
    Comment,
    /// A group start ({).
    GroupStart,
    /// A group end (}).
    GroupEnd,
    /// Math shift ($).
    MathShift,
    /// Alignment tab (&).
    AlignmentTab,
    /// Parameter (#).
    Parameter,
    /// Superscript (^).
    Superscript,
    /// Subscript (_).
    Subscript,
    /// An active character (~).
    ActiveCharacter,
    /// A begin group character (catcode 1).
    BeginGroup,
    /// An end group character (catcode 2).
    EndGroup,
    /// A math shift character (catcode 3).
    MathShiftChar,
    /// An alignment tab character (catcode 4).
    AlignmentTabChar,
    /// An end of line character (catcode 5).
    EndOfLine,
    /// A parameter character (catcode 6).
    ParameterChar,
    /// A superscript character (catcode 7).
    SuperscriptChar,
    /// A subscript character (catcode 8).
    SubscriptChar,
    /// An ignored character (catcode 9).
    Ignored,
    /// A space character (catcode 10).
    Space,
    /// A letter (catcode 11).
    Letter,
    /// An other character (catcode 12).
    Other,
    /// An active character (catcode 13).
    Active,
    /// A comment character (catcode 14).
    CommentChar,
    /// An invalid character (catcode 15).
    Invalid,
    /// End of input.
    EOF,
}

impl TokenType {
    /// Get the category code (catcode) for this token type.
    pub fn catcode(self) -> Option<u8> {
        match self {
            TokenType::BeginGroup | TokenType::GroupStart => Some(1),
            TokenType::EndGroup | TokenType::GroupEnd => Some(2),
            TokenType::MathShiftChar | TokenType::MathShift => Some(3),
            TokenType::AlignmentTabChar | TokenType::AlignmentTab => Some(4),
            TokenType::EndOfLine | TokenType::Newline => Some(5),
            TokenType::ParameterChar | TokenType::Parameter => Some(6),
            TokenType::SuperscriptChar | TokenType::Superscript => Some(7),
            TokenType::SubscriptChar | TokenType::Subscript => Some(8),
            TokenType::Ignored => Some(9),
            TokenType::Space | TokenType::Whitespace => Some(10),
            TokenType::Letter | TokenType::Word => Some(11),
            TokenType::Other | TokenType::Symbol => Some(12),
            TokenType::Active | TokenType::ActiveCharacter => Some(13),
            TokenType::CommentChar | TokenType::Comment => Some(14),
            TokenType::Invalid => Some(15),
            TokenType::ControlSequence | TokenType::EOF => None,
        }
    }

    /// Check if this token type represents a control word.
    pub fn is_control_word(self) -> bool {
        self == TokenType::ControlSequence
    }

    /// Check if this token type represents a group delimiter.
    pub fn is_group_delimiter(self) -> bool {
        matches!(
            self,
            TokenType::GroupStart
                | TokenType::GroupEnd
                | TokenType::BeginGroup
                | TokenType::EndGroup
        )
    }

    /// Check if this token type represents math mode delimiter.
    pub fn is_math_delimiter(self) -> bool {
        matches!(self, TokenType::MathShift | TokenType::MathShiftChar)
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TokenType::Word => "word",
            TokenType::ControlSequence => "control sequence",
            TokenType::Symbol => "symbol",
            TokenType::Whitespace => "whitespace",
            TokenType::Newline => "newline",
            TokenType::Comment => "comment",
            TokenType::GroupStart => "group start",
            TokenType::GroupEnd => "group end",
            TokenType::MathShift => "math shift",
            TokenType::AlignmentTab => "alignment tab",
            TokenType::Parameter => "parameter",
            TokenType::Superscript => "superscript",
            TokenType::Subscript => "subscript",
            TokenType::ActiveCharacter => "active character",
            TokenType::BeginGroup => "begin group",
            TokenType::EndGroup => "end group",
            TokenType::MathShiftChar => "math shift char",
            TokenType::AlignmentTabChar => "alignment tab char",
            TokenType::EndOfLine => "end of line",
            TokenType::ParameterChar => "parameter char",
            TokenType::SuperscriptChar => "superscript char",
            TokenType::SubscriptChar => "subscript char",
            TokenType::Ignored => "ignored",
            TokenType::Space => "space",
            TokenType::Letter => "letter",
            TokenType::Other => "other",
            TokenType::Active => "active",
            TokenType::CommentChar => "comment char",
            TokenType::Invalid => "invalid",
            TokenType::EOF => "EOF",
        };
        write!(f, "{}", name)
    }
}

/// A TeX parser for tokenizing input.
#[derive(Debug)]
pub struct TeXParser {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl TeXParser {
    /// Create a new TeX parser.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenize the input into tokens.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while !self.is_eof() {
            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }
        tokens
            .push(Token::new(TokenType::EOF, "").with_position(self.line, self.column));
        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        if self.is_eof() {
            return None;
        }

        let c = self.current_char()?;
        let start_line = self.line;
        let start_column = self.column;

        let token = match c {
            '\\' => self.read_control_sequence(),
            '{' => {
                self.advance();
                Token::new(TokenType::GroupStart, "{")
                    .with_position(start_line, start_column)
            }
            '}' => {
                self.advance();
                Token::new(TokenType::GroupEnd, "}")
                    .with_position(start_line, start_column)
            }
            '$' => {
                self.advance();
                Token::new(TokenType::MathShift, "$")
                    .with_position(start_line, start_column)
            }
            '%' => self.read_comment(),
            '&' => {
                self.advance();
                Token::new(TokenType::AlignmentTab, "&")
                    .with_position(start_line, start_column)
            }
            '#' => {
                self.advance();
                Token::new(TokenType::Parameter, "#")
                    .with_position(start_line, start_column)
            }
            '^' => {
                self.advance();
                Token::new(TokenType::Superscript, "^")
                    .with_position(start_line, start_column)
            }
            '_' => {
                self.advance();
                Token::new(TokenType::Subscript, "_")
                    .with_position(start_line, start_column)
            }
            '\n' | '\r' => {
                self.advance();
                Token::new(TokenType::Newline, "\n")
                    .with_position(start_line, start_column)
            }
            ' ' => self.read_whitespace(),
            '\t' => {
                self.advance();
                Token::new(TokenType::Whitespace, "\t")
                    .with_position(start_line, start_column)
            }
            '~' => {
                self.advance();
                Token::new(TokenType::ActiveCharacter, "~")
                    .with_position(start_line, start_column)
            }
            c if c.is_ascii_alphabetic() => self.read_word(),
            c => {
                self.advance();
                Token::new(TokenType::Symbol, c).with_position(start_line, start_column)
            }
        };

        Some(token)
    }

    fn read_control_sequence(&mut self) -> Token {
        let start_line = self.line;
        let start_column = self.column;
        self.advance();

        let mut name = String::new();

        if let Some(c) = self.current_char() {
            if c.is_ascii_alphabetic() {
                while let Some(c) = self.current_char() {
                    if !c.is_ascii_alphabetic() && !c.is_ascii_digit() {
                        break;
                    }
                    name.push(c);
                    self.advance();
                }
            } else {
                name.push(c);
                self.advance();
            }
        }

        Token::new(TokenType::ControlSequence, name)
            .with_position(start_line, start_column)
    }

    fn read_word(&mut self) -> Token {
        let start_line = self.line;
        let start_column = self.column;
        let mut word = String::new();

        while let Some(c) = self.current_char() {
            if c.is_ascii_alphabetic() || c == '\'' || c == '-' {
                word.push(c);
                self.advance();
            } else {
                break;
            }
        }

        Token::new(TokenType::Word, word).with_position(start_line, start_column)
    }

    fn read_whitespace(&mut self) -> Token {
        let start_line = self.line;
        let start_column = self.column;
        let mut whitespace = String::new();

        while let Some(c) = self.current_char() {
            if c == ' ' {
                whitespace.push(c);
                self.advance();
            } else {
                break;
            }
        }

        Token::new(TokenType::Whitespace, whitespace)
            .with_position(start_line, start_column)
    }

    fn read_comment(&mut self) -> Token {
        let start_line = self.line;
        let start_column = self.column;
        self.advance();

        let mut comment = String::new();

        while let Some(c) = self.current_char() {
            if c == '\n' || c == '\r' {
                break;
            }
            comment.push(c);
            self.advance();
        }

        Token::new(TokenType::Comment, comment).with_position(start_line, start_column)
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        if let Some(c) = self.current_char() {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }
}

/// Tokenize TeX input into tokens.
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut parser = TeXParser::new(input);
    parser.tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let t = Token::word("hello");
        assert!(matches!(t.token_type, TokenType::Word));
        assert_eq!(t.as_str(), "hello");
    }

    #[test]
    fn test_control_sequence() {
        let t = Token::control_sequence("textbf");
        assert!(matches!(t.token_type, TokenType::ControlSequence));
        assert_eq!(t.as_str(), "textbf");
        assert!(t.is_control_sequence());
    }

    #[test]
    fn test_token_position() {
        let t = Token::word("test").with_position(5, 10);
        assert_eq!(t.line, 5);
        assert_eq!(t.column, 10);
    }

    #[test]
    fn test_token_type_display() {
        assert_eq!(TokenType::Word.to_string(), "word");
        assert_eq!(TokenType::ControlSequence.to_string(), "control sequence");
        assert_eq!(TokenType::GroupStart.to_string(), "group start");
    }

    #[test]
    fn test_token_type_catcode() {
        assert_eq!(TokenType::GroupStart.catcode(), Some(1));
        assert_eq!(TokenType::GroupEnd.catcode(), Some(2));
        assert_eq!(TokenType::MathShift.catcode(), Some(3));
        assert_eq!(TokenType::AlignmentTab.catcode(), Some(4));
        assert_eq!(TokenType::Newline.catcode(), Some(5));
        assert_eq!(TokenType::Parameter.catcode(), Some(6));
        assert_eq!(TokenType::Superscript.catcode(), Some(7));
        assert_eq!(TokenType::Subscript.catcode(), Some(8));
        assert_eq!(TokenType::Ignored.catcode(), Some(9));
        assert_eq!(TokenType::Whitespace.catcode(), Some(10));
        assert_eq!(TokenType::Word.catcode(), Some(11));
        assert_eq!(TokenType::Symbol.catcode(), Some(12));
        assert_eq!(TokenType::ActiveCharacter.catcode(), Some(13));
        assert_eq!(TokenType::Comment.catcode(), Some(14));
        assert_eq!(TokenType::Invalid.catcode(), Some(15));
        assert_eq!(TokenType::ControlSequence.catcode(), None);
    }

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("Hello world");
        // Hello, space, world, EOF
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].token_type, TokenType::Word));
        assert!(matches!(tokens[1].token_type, TokenType::Whitespace));
        assert!(matches!(tokens[2].token_type, TokenType::Word));
        assert!(matches!(tokens[3].token_type, TokenType::EOF));
    }

    #[test]
    fn test_tokenize_control_sequence() {
        let tokens = tokenize(r"\textbf{Hello}");
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0].token_type, TokenType::ControlSequence));
        assert_eq!(tokens[0].as_str(), "textbf");
        assert!(matches!(tokens[1].token_type, TokenType::GroupStart));
        assert!(matches!(tokens[2].token_type, TokenType::Word));
        assert!(matches!(tokens[3].token_type, TokenType::GroupEnd));
    }

    #[test]
    fn test_tokenize_math() {
        let tokens = tokenize("$x + y$");
        // $, x, space, +, space, y, $, EOF
        assert_eq!(tokens.len(), 8);
        assert!(matches!(tokens[0].token_type, TokenType::MathShift));
        assert!(matches!(tokens[1].token_type, TokenType::Word));
        assert!(matches!(tokens[2].token_type, TokenType::Whitespace));
        assert!(matches!(tokens[3].token_type, TokenType::Symbol)); // +
        assert!(matches!(tokens[4].token_type, TokenType::Whitespace));
        assert!(matches!(tokens[5].token_type, TokenType::Word));
        assert!(matches!(tokens[6].token_type, TokenType::MathShift));
        assert!(matches!(tokens[7].token_type, TokenType::EOF));
    }

    #[test]
    fn test_tokenize_comment() {
        let tokens = tokenize("Hello % comment\nworld");
        assert!(tokens
            .iter()
            .any(|t| matches!(t.token_type, TokenType::Comment)));
        assert!(tokens.iter().any(|t| t.as_str().contains("comment")));
    }

    #[test]
    fn test_tokenize_newline() {
        let tokens = tokenize("line1\nline2");
        assert!(tokens
            .iter()
            .any(|t| matches!(t.token_type, TokenType::Newline)));
    }

    #[test]
    fn test_tokenize_eof() {
        let tokens = tokenize("test");
        assert!(tokens
            .last()
            .is_some_and(|t| matches!(t.token_type, TokenType::EOF)));
    }

    #[test]
    fn test_token_display() {
        let t = Token::word("hello");
        assert_eq!(format!("{}", t), "hello");

        let t2 = Token::control_sequence("cmd");
        assert_eq!(format!("{}", t2), "cmd");
    }

    #[test]
    fn test_token_equality() {
        let t1 = Token::word("test");
        let t2 = Token::word("test");
        let t3 = Token::word("other");

        assert_eq!(t1, t2);
        assert_ne!(t1, t3);
    }
}
