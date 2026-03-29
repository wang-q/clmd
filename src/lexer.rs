/// Lexer for CommonMark documents
///
/// The lexer processes input text line by line, tracking position information
/// and providing helper methods for parsing.

// Code indent threshold (4 spaces or 1 tab)
pub const CODE_INDENT: usize = 4;

/// Tab stop size
pub const TAB_STOP: usize = 4;

/// Check if a character is a space or tab
pub fn is_space_or_tab(c: char) -> bool {
    c == ' ' || c == '\t'
}
