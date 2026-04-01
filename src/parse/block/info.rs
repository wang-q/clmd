//! Block info for tracking fenced code blocks and list items

/// Block info for tracking fenced code blocks and list items
#[derive(Debug, Clone)]
pub struct BlockInfo {
    /// Block is open
    pub is_open: bool,
    /// For fenced code blocks: fence character
    pub fence_char: char,
    /// For fenced code blocks: fence length
    pub fence_length: usize,
    /// For fenced code blocks: fence offset
    pub fence_offset: usize,
    /// For list items: marker offset
    pub marker_offset: usize,
    /// For list items: padding
    pub padding: usize,
    /// For HTML blocks: block type (1-7)
    pub html_block_type: u8,
    /// For headings: setext flag
    pub is_setext: bool,
    /// Last line blank flag
    pub last_line_blank: bool,
    /// String content accumulator
    pub string_content: String,
}

impl BlockInfo {
    /// Create a new BlockInfo with default values
    pub fn new() -> Self {
        BlockInfo {
            is_open: true,
            fence_char: '\0',
            fence_length: 0,
            fence_offset: 0,
            marker_offset: 0,
            padding: 0,
            html_block_type: 0,
            is_setext: false,
            last_line_blank: false,
            string_content: String::new(),
        }
    }
}

impl Default for BlockInfo {
    fn default() -> Self {
        Self::new()
    }
}
