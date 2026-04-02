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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_info_new() {
        let info = BlockInfo::new();
        assert!(info.is_open);
        assert_eq!(info.fence_char, '\0');
        assert_eq!(info.fence_length, 0);
        assert_eq!(info.fence_offset, 0);
        assert_eq!(info.marker_offset, 0);
        assert_eq!(info.padding, 0);
        assert_eq!(info.html_block_type, 0);
        assert!(!info.is_setext);
        assert!(!info.last_line_blank);
        assert!(info.string_content.is_empty());
    }

    #[test]
    fn test_block_info_default() {
        let info: BlockInfo = Default::default();
        assert!(info.is_open);
        assert_eq!(info.fence_char, '\0');
        assert_eq!(info.fence_length, 0);
        assert_eq!(info.fence_offset, 0);
        assert_eq!(info.marker_offset, 0);
        assert_eq!(info.padding, 0);
        assert_eq!(info.html_block_type, 0);
        assert!(!info.is_setext);
        assert!(!info.last_line_blank);
        assert!(info.string_content.is_empty());
    }

    #[test]
    fn test_block_info_clone() {
        let mut info = BlockInfo::new();
        info.fence_char = '`';
        info.fence_length = 3;
        info.string_content = "test content".to_string();

        let cloned = info.clone();
        assert_eq!(cloned.fence_char, '`');
        assert_eq!(cloned.fence_length, 3);
        assert_eq!(cloned.string_content, "test content");
    }

    #[test]
    fn test_block_info_debug() {
        let info = BlockInfo::new();
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("BlockInfo"));
        assert!(debug_str.contains("is_open"));
        assert!(debug_str.contains("fence_char"));
    }

    #[test]
    fn test_block_info_for_fenced_code() {
        let mut info = BlockInfo::new();
        info.fence_char = '`';
        info.fence_length = 3;
        info.fence_offset = 4;
        info.string_content = "code block content".to_string();

        assert_eq!(info.fence_char, '`');
        assert_eq!(info.fence_length, 3);
        assert_eq!(info.fence_offset, 4);
        assert_eq!(info.string_content, "code block content");
    }

    #[test]
    fn test_block_info_for_list_item() {
        let mut info = BlockInfo::new();
        info.marker_offset = 2;
        info.padding = 4;

        assert_eq!(info.marker_offset, 2);
        assert_eq!(info.padding, 4);
    }

    #[test]
    fn test_block_info_for_html_block() {
        let mut info = BlockInfo::new();
        info.html_block_type = 6;

        assert_eq!(info.html_block_type, 6);
    }

    #[test]
    fn test_block_info_for_setext_heading() {
        let mut info = BlockInfo::new();
        info.is_setext = true;
        info.last_line_blank = true;

        assert!(info.is_setext);
        assert!(info.last_line_blank);
    }

    #[test]
    fn test_block_info_modify_string_content() {
        let mut info = BlockInfo::new();
        info.string_content.push_str("line 1\n");
        info.string_content.push_str("line 2\n");

        assert_eq!(info.string_content, "line 1\nline 2\n");
    }
}
