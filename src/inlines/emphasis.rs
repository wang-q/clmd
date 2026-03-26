//! Emphasis processing for inline parsing

use crate::arena::{NodeArena, NodeId, TreeOps};
use crate::inlines::utils::{is_punctuation, DelimScanResult};
use crate::node::{NodeData, NodeType};
use smallvec::SmallVec;

/// Delimiter struct for tracking emphasis markers
/// This is a singly-linked list using Box for ownership
pub struct Delimiter {
    /// Previous delimiter in stack
    pub previous: Option<Box<Delimiter>>,
    /// The inline text node containing the delimiter
    pub inl_text: NodeId,
    /// Position in the subject
    pub position: usize,
    /// Number of delimiter characters
    pub num_delims: usize,
    /// Original number of delimiter characters
    pub orig_delims: usize,
    /// The delimiter character (* or _)
    pub delim_char: char,
    /// Whether this can open emphasis
    pub can_open: bool,
    /// Whether this can close emphasis
    pub can_close: bool,
}

/// Delimiter information tuple: (inl_text, delim_char, can_open, can_close, orig_delims, num_delims)
type DelimInfo = (NodeId, char, bool, bool, usize, usize);

/// Collect delimiter information from the delimiter stack
fn collect_delimiters(delimiters: &Option<Box<Delimiter>>) -> SmallVec<[DelimInfo; 32]> {
    let mut delims: SmallVec<[DelimInfo; 32]> = SmallVec::new();
    let mut current = delimiters.as_ref();

    while let Some(d) = current {
        delims.push((
            d.inl_text,
            d.delim_char,
            d.can_open,
            d.can_close,
            d.orig_delims,
            d.num_delims,
        ));
        current = d.previous.as_ref();
    }

    // Reverse to get them in order from oldest to newest
    delims.reverse();
    delims
}

/// Find the starting index based on stack_bottom
fn find_start_index(delims: &[DelimInfo], stack_bottom: Option<&Delimiter>) -> usize {
    if let Some(sb) = stack_bottom {
        delims
            .iter()
            .position(|(node_id, _, _, _, orig, _)| {
                *node_id == sb.inl_text && *orig == sb.orig_delims
            })
            .map(|i| i + 1)
            .unwrap_or(0)
    } else {
        0
    }
}

/// Get the openers_bottom index for a given closer
fn get_openers_bottom_index(
    closer_char: char,
    closer_can_open: bool,
    closer_orig_delims: usize,
) -> Option<usize> {
    match closer_char {
        '"' => Some(0),
        '\'' => Some(1),
        '_' => Some(2 + if closer_can_open { 3 } else { 0 } + (closer_orig_delims % 3)),
        '*' => Some(8 + if closer_can_open { 3 } else { 0 } + (closer_orig_delims % 3)),
        _ => None,
    }
}

/// Check if the match is invalid due to the odd match rule
fn is_odd_match(
    closer_can_open: bool,
    opener_can_close: bool,
    closer_orig_delims: usize,
    opener_orig_delims: usize,
) -> bool {
    (closer_can_open || opener_can_close)
        && !closer_orig_delims.is_multiple_of(3)
        && (opener_orig_delims + closer_orig_delims).is_multiple_of(3)
}

/// Find a matching opener for the given closer
fn find_matching_opener(
    delims: &[DelimInfo],
    closer_idx: usize,
    openers_bottom_idx: usize,
    openers_bottom: &[usize; 14],
    closer_char: char,
    closer_can_open: bool,
    _closer_can_close: bool,
    closer_orig_delims: usize,
) -> Option<usize> {
    let bottom = openers_bottom[openers_bottom_idx];

    // First, try to find an opener with the same number of delimiters
    let mut opener_idx = closer_idx;
    while opener_idx > bottom {
        opener_idx -= 1;
        let (_, opener_char, opener_can_open, opener_can_close, opener_orig_delims, _) =
            delims[opener_idx];

        if opener_char == closer_char
            && opener_can_open
            && opener_orig_delims == closer_orig_delims
        {
            if !is_odd_match(
                closer_can_open,
                opener_can_close,
                closer_orig_delims,
                opener_orig_delims,
            ) {
                return Some(opener_idx);
            }
        }
    }

    // If no exact match found, look for any matching opener
    let mut opener_idx = closer_idx;
    while opener_idx > bottom {
        opener_idx -= 1;
        let (_, opener_char, opener_can_open, opener_can_close, opener_orig_delims, _) =
            delims[opener_idx];

        if opener_char == closer_char && opener_can_open {
            if !is_odd_match(
                closer_can_open,
                opener_can_close,
                closer_orig_delims,
                opener_orig_delims,
            ) {
                return Some(opener_idx);
            }
        }
    }

    None
}

/// Create an emphasis or strong node
fn create_emphasis_node(arena: &mut NodeArena, use_delims: usize) -> NodeId {
    let emph_type = if use_delims == 1 {
        NodeType::Emph
    } else {
        NodeType::Strong
    };
    arena.alloc(crate::arena::Node::new(emph_type))
}

/// Update delimiter text node by removing used delimiters
fn update_delimiter_text(
    arena: &mut NodeArena,
    node_id: NodeId,
    use_delims: usize,
) -> String {
    let node = arena.get_mut(node_id);
    if let NodeData::Text { ref mut literal } = node.data {
        let new_len = literal.len().saturating_sub(use_delims);
        literal.truncate(new_len);
        literal.clone()
    } else {
        String::new()
    }
}

/// Move nodes between opener and closer into the emphasis node
fn move_nodes_to_emphasis(
    arena: &mut NodeArena,
    emph_node: NodeId,
    opener_inl: NodeId,
    closer_inl: NodeId,
) {
    let mut current_child = arena.get(opener_inl).next;
    while let Some(child_id) = current_child {
        if child_id == closer_inl {
            break;
        }
        let next_child = arena.get(child_id).next;

        TreeOps::unlink(arena, child_id);
        TreeOps::append_child(arena, emph_node, child_id);

        current_child = next_child;
    }
}

/// Process an emphasis pair (opener and closer)
fn process_emphasis_pair(
    arena: &mut NodeArena,
    delims: &[DelimInfo],
    opener_idx: usize,
    closer_idx: usize,
) {
    let (opener_inl, _, _, _, opener_orig_delims, _) = delims[opener_idx];
    let (closer_inl, _, _, _, closer_orig_delims, _) = delims[closer_idx];

    let use_delims = if opener_orig_delims >= 2 && closer_orig_delims >= 2 {
        2
    } else {
        1
    };

    let opener_text = update_delimiter_text(arena, opener_inl, use_delims);
    let closer_text = update_delimiter_text(arena, closer_inl, use_delims);

    let emph_node = create_emphasis_node(arena, use_delims);
    move_nodes_to_emphasis(arena, emph_node, opener_inl, closer_inl);

    TreeOps::insert_after(arena, opener_inl, emph_node);

    if opener_text.is_empty() {
        TreeOps::unlink(arena, opener_inl);
    }
    if closer_text.is_empty() {
        TreeOps::unlink(arena, closer_inl);
    }
}

/// Process smart quotes
fn process_smart_quotes(
    arena: &mut NodeArena,
    delims: &[DelimInfo],
    opener_idx: Option<usize>,
    closer_idx: usize,
    closer_char: char,
) {
    let (closer_inl, _, _, _, _, _) = delims[closer_idx];

    let quote_char = if closer_char == '\'' {
        '\u{2019}'
    } else {
        '\u{201D}'
    };

    {
        let node = arena.get_mut(closer_inl);
        if let NodeData::Text { ref mut literal } = node.data {
            *literal = quote_char.to_string();
        }
    }

    if let Some(opener_idx) = opener_idx {
        let (opener_inl, _, _, _, _, _) = delims[opener_idx];
        let open_quote = if closer_char == '\'' {
            '\u{2018}'
        } else {
            '\u{201C}'
        };

        let node = arena.get_mut(opener_inl);
        if let NodeData::Text { ref mut literal } = node.data {
            *literal = open_quote.to_string();
        }
    }
}

/// Rebuild the delimiter stack after processing
fn rebuild_delimiter_stack(
    delimiters: &mut Option<Box<Delimiter>>,
    delims: SmallVec<[DelimInfo; 32]>,
    start_idx: usize,
) {
    if start_idx > 0 {
        let delims_to_keep: Vec<_> = delims.into_iter().take(start_idx).collect();
        *delimiters = None;
        for (node_id, char, can_open, can_close, orig_delims, num_delims) in
            delims_to_keep
        {
            let delim = Box::new(Delimiter {
                previous: delimiters.take(),
                inl_text: node_id,
                position: 0,
                num_delims,
                orig_delims,
                delim_char: char,
                can_open,
                can_close,
            });
            *delimiters = Some(delim);
        }
    } else {
        *delimiters = None;
    }
}

/// Scan delimiter sequence and determine if it can open/close
pub fn scan_delims(input: &str, pos: usize, c: char) -> (DelimScanResult, usize) {
    let start_pos = pos;
    let mut num_delims = 0;
    let bytes = input.as_bytes();
    let mut new_pos = pos;

    // Count delimiters
    while new_pos < bytes.len() {
        let b = bytes[new_pos];
        if b == c as u8 {
            num_delims += 1;
            // For ASCII characters (0-127), advance by 1
            // For UTF-8 multi-byte sequences, calculate the length
            new_pos += if b < 0x80 {
                1
            } else if b < 0xE0 {
                2
            } else if b < 0xF0 {
                3
            } else {
                4
            };
        } else {
            break;
        }
    }

    if num_delims == 0 {
        return (
            DelimScanResult {
                num_delims: 0,
                can_open: false,
                can_close: false,
            },
            pos,
        );
    }

    // Determine char before and after
    // Note: start_pos is a byte position, not character position
    let char_before = if start_pos == 0 {
        '\n'
    } else {
        // Get the character that ends right before start_pos
        input[..start_pos].chars().last().unwrap_or('\n')
    };

    let char_after = if new_pos < input.len() {
        input[new_pos..].chars().next().unwrap_or('\n')
    } else {
        '\n'
    };

    let before_is_whitespace = char_before.is_whitespace();
    let before_is_punctuation = is_punctuation(char_before);
    let after_is_whitespace = char_after.is_whitespace();
    let after_is_punctuation = is_punctuation(char_after);

    let left_flanking = !after_is_whitespace
        && (!after_is_punctuation || before_is_whitespace || before_is_punctuation);
    let right_flanking = !before_is_whitespace
        && (!before_is_punctuation || after_is_whitespace || after_is_punctuation);

    let (can_open, can_close) = if c == '_' {
        (
            left_flanking && (!right_flanking || before_is_punctuation),
            right_flanking && (!left_flanking || after_is_punctuation),
        )
    } else {
        (left_flanking, right_flanking)
    };

    (
        DelimScanResult {
            num_delims,
            can_open,
            can_close,
        },
        new_pos,
    )
}

/// Process emphasis delimiters
/// Based on commonmark.js processEmphasis function and cmark implementation
pub fn process_emphasis(
    arena: &mut NodeArena,
    delimiters: &mut Option<Box<Delimiter>>,
    stack_bottom: Option<&Delimiter>,
) {
    // Collect all delimiter info
    let delims = collect_delimiters(delimiters);

    // Find the starting index based on stack_bottom
    let start_idx = find_start_index(&delims, stack_bottom);

    // Initialize openers_bottom for each delimiter type
    // Index mapping: 0=" 1=' 2-7=_ (based on can_open and length % 3) 8-13=* (based on can_open and length % 3)
    let mut openers_bottom: [usize; 14] = [start_idx; 14];

    // Process closers from left to right, starting from start_idx
    let mut closer_idx = start_idx;
    while closer_idx < delims.len() {
        let (
            _closer_inl,
            closer_char,
            closer_can_open,
            closer_can_close,
            closer_orig_delims,
            _,
        ) = delims[closer_idx];

        if !closer_can_close {
            closer_idx += 1;
            continue;
        }

        // Determine openers_bottom index based on closer type
        let openers_bottom_idx = match get_openers_bottom_index(
            closer_char,
            closer_can_open,
            closer_orig_delims,
        ) {
            Some(idx) => idx,
            None => {
                closer_idx += 1;
                continue;
            }
        };

        // Look for matching opener
        let opener_idx = find_matching_opener(
            &delims,
            closer_idx,
            openers_bottom_idx,
            &openers_bottom,
            closer_char,
            closer_can_open,
            closer_can_close,
            closer_orig_delims,
        );

        let old_closer_idx = closer_idx;

        match closer_char {
            '*' | '_' => {
                if let Some(opener_idx) = opener_idx {
                    process_emphasis_pair(arena, &delims, opener_idx, closer_idx);
                }
                closer_idx += 1;
            }
            '\'' | '"' => {
                process_smart_quotes(
                    arena,
                    &delims,
                    opener_idx,
                    closer_idx,
                    closer_char,
                );
                closer_idx += 1;
            }
            _ => closer_idx += 1,
        }

        if opener_idx.is_none() {
            openers_bottom[openers_bottom_idx] = old_closer_idx;
        }
    }

    // Rebuild the delimiter stack
    rebuild_delimiter_stack(delimiters, delims, start_idx);
}
