//! Emphasis processing for inline parsing

use crate::arena::{NodeArena, NodeId, TreeOps};
use crate::inlines::utils::{is_punctuation, DelimScanResult};
use crate::node_value::NodeValue;
use smallvec::SmallVec;

/// Delimiter struct for tracking emphasis markers
/// This is a singly-linked list using Box for ownership
#[derive(Debug)]
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

/// Delimiter information tuple: (inl_text, delim_char, can_open, can_close, orig_delims, num_delims, processed)
/// processed: true if this delimiter has been used in an emphasis pair
type DelimInfo = (NodeId, char, bool, bool, usize, usize, bool);

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
            false, // processed = false initially
        ));
        current = d.previous.as_ref();
    }

    // Reverse to get them in order from oldest to newest
    delims.reverse();
    delims
}

/// Find the starting index based on stack_bottom
/// stack_bottom_marker is (inl_text, orig_delims) to identify the delimiter
fn find_start_index(
    delims: &[DelimInfo],
    stack_bottom_marker: Option<(NodeId, usize)>,
) -> usize {
    if let Some((inl_text, orig_delims)) = stack_bottom_marker {
        delims
            .iter()
            .position(|(node_id, _, _, _, orig, _, _)| {
                *node_id == inl_text && *orig == orig_delims
            })
            .map(|i| i + 1)
            .unwrap_or(0)
    } else {
        0
    }
}

/// Get the openers_bottom index for a given closer
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
        let (
            _,
            opener_char,
            opener_can_open,
            opener_can_close,
            opener_orig_delims,
            _,
            processed,
        ) = delims[opener_idx];

        // Skip already processed delimiters
        if processed {
            continue;
        }

        if opener_char == closer_char
            && opener_can_open
            && opener_orig_delims == closer_orig_delims
            && !is_odd_match(
                closer_can_open,
                opener_can_close,
                closer_orig_delims,
                opener_orig_delims,
            )
        {
            return Some(opener_idx);
        }
    }

    // If no exact match found, look for any matching opener
    let mut opener_idx = closer_idx;
    while opener_idx > bottom {
        opener_idx -= 1;
        let (
            _,
            opener_char,
            opener_can_open,
            opener_can_close,
            opener_orig_delims,
            _,
            processed,
        ) = delims[opener_idx];

        // Skip already processed delimiters
        if processed {
            continue;
        }

        if opener_char == closer_char
            && opener_can_open
            && !is_odd_match(
                closer_can_open,
                opener_can_close,
                closer_orig_delims,
                opener_orig_delims,
            )
        {
            return Some(opener_idx);
        }
    }

    None
}

/// Create an emphasis or strong node
fn create_emphasis_node(arena: &mut NodeArena, use_delims: usize) -> NodeId {
    let value = if use_delims == 1 {
        NodeValue::Emph
    } else {
        NodeValue::Strong
    };
    arena.alloc(crate::arena::Node::with_value(value))
}

/// Update delimiter text node by removing used delimiters
fn update_delimiter_text(
    arena: &mut NodeArena,
    node_id: NodeId,
    use_delims: usize,
) -> String {
    let node = arena.get_mut(node_id);
    if let NodeValue::Text(ref mut literal) = node.value {
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
    _delims: &mut [DelimInfo],
    opener_idx: usize,
    closer_idx: usize,
    use_delims: usize,
) {
    let (opener_inl, _, _, _, _, _, _) = _delims[opener_idx];
    let (closer_inl, _, _, _, _, _, _) = _delims[closer_idx];

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
    let (closer_inl, _, _, _, _, _, _) = delims[closer_idx];

    let quote_char = if closer_char == '\'' {
        '\u{2019}'
    } else {
        '\u{201D}'
    };

    {
        let node = arena.get_mut(closer_inl);
        if let NodeValue::Text(ref mut literal) = node.value {
            *literal = quote_char.to_string();
        }
    }

    if let Some(opener_idx) = opener_idx {
        let (opener_inl, _, _, _, _, _, _) = delims[opener_idx];
        let open_quote = if closer_char == '\'' {
            '\u{2018}'
        } else {
            '\u{201C}'
        };

        let node = arena.get_mut(opener_inl);
        if let NodeValue::Text(ref mut literal) = node.value {
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
    // Keep delimiters before start_idx (stack bottom) and any unprocessed delimiters
    // Unprocessed delimiters have num_delims > 0 AND are after start_idx
    // For delimiters before start_idx, we keep them regardless of num_delims
    // (they are part of the stack bottom)
    let delims_to_keep: Vec<_> = delims
        .into_iter()
        .enumerate()
        .filter(|(idx, (_, _, _, _, _, num_delims, _))| {
            if *idx < start_idx {
                // Keep stack bottom delimiters
                true
            } else {
                // For delimiters after start_idx, only keep if they still have delims
                // AND they were not processed (i.e., they didn't find a match)
                // Actually, if num_delims > 0 but they didn't find a match,
                // they should be removed from the stack (become regular text)
                // *num_delims > 0
                *num_delims > 0
            }
        })
        .map(|(_, delim)| delim)
        .collect();

    *delimiters = None;
    for (node_id, char, can_open, can_close, orig_delims, num_delims, _processed) in
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

/// Check if a delimiter can be used as a closer
fn is_valid_closer(delim: &DelimInfo) -> bool {
    let (_, _, _, can_close, _, num_delims, processed) = *delim;
    !processed && can_close && num_delims > 0
}

/// Check the odd match rule for emphasis
/// This rule prevents matching when:
/// - One delimiter can open AND the other can close
/// - Neither delimiter count is a multiple of 3
/// - The sum of delimiter counts IS a multiple of 3
fn check_odd_match(
    closer_can_open: bool,
    opener_can_close: bool,
    closer_orig_delims: usize,
    opener_orig_delims: usize,
) -> bool {
    (closer_can_open || opener_can_close)
        && closer_orig_delims % 3 != 0
        && (opener_orig_delims + closer_orig_delims) % 3 == 0
}

/// Find a matching opener for a closer in the new process_emphasis implementation
fn find_matching_opener_simple(
    delims: &[DelimInfo],
    closer_idx: usize,
    start_idx: usize,
    closer_char: char,
    closer_can_open: bool,
    closer_orig_delims: usize,
) -> Option<usize> {
    for j in (start_idx..closer_idx).rev() {
        let (
            _,
            opener_char,
            opener_can_open,
            opener_can_close,
            opener_orig_delims,
            opener_num_delims,
            opener_processed,
        ) = delims[j];

        // Skip already processed delimiters or delimiters with no remaining count
        if opener_processed || opener_num_delims == 0 {
            continue;
        }

        if !opener_can_open || opener_char != closer_char {
            continue;
        }

        // Check odd match rule
        if !check_odd_match(
            closer_can_open,
            opener_can_close,
            closer_orig_delims,
            opener_orig_delims,
        ) {
            return Some(j);
        }
    }
    None
}

/// Calculate how many delimiters to use for emphasis
fn calculate_delimiters_to_use(opener_count: usize, closer_count: usize) -> usize {
    if opener_count >= 2 && closer_count >= 2 {
        2 // Strong emphasis
    } else {
        1 // Regular emphasis
    }
}

/// Process a single emphasis match
fn process_emphasis_match(
    arena: &mut NodeArena,
    delims: &mut SmallVec<[DelimInfo; 32]>,
    opener_idx: usize,
    closer_idx: usize,
) {
    let opener_count = delims[opener_idx].5;
    let closer_count = delims[closer_idx].5;
    let use_delims = calculate_delimiters_to_use(opener_count, closer_count);

    process_emphasis_pair(arena, delims, opener_idx, closer_idx, use_delims);

    // Update delimiter counts
    delims[opener_idx].5 -= use_delims;
    delims[closer_idx].5 -= use_delims;

    // Mark delimiters between opener and closer as processed (inside emphasis)
    for k in (opener_idx + 1)..closer_idx {
        delims[k].5 = 0;
    }

    // Remove processed delimiters from vector if count is 0
    // Remove closer first (higher index) to avoid index shifting issues
    let closer_removed = if delims[closer_idx].5 == 0 {
        delims.remove(closer_idx);
        true
    } else {
        false
    };

    // Adjust opener_idx if closer was removed
    let adjusted_opener_idx = if closer_removed && opener_idx > closer_idx {
        opener_idx - 1
    } else {
        opener_idx
    };

    if delims[adjusted_opener_idx].5 == 0 {
        delims.remove(adjusted_opener_idx);
    }
}

/// Process a single closer delimiter
/// Returns true if a match was found and processing should restart
fn process_closer(
    arena: &mut NodeArena,
    delims: &mut SmallVec<[DelimInfo; 32]>,
    closer_idx: usize,
    start_idx: usize,
) -> bool {
    let (_, closer_char, closer_can_open, _, closer_orig_delims, _, _) =
        delims[closer_idx];

    // Find matching opener
    let opener_idx = find_matching_opener_simple(
        delims,
        closer_idx,
        start_idx,
        closer_char,
        closer_can_open,
        closer_orig_delims,
    );

    match closer_char {
        '*' | '_' => {
            if let Some(opener_idx) = opener_idx {
                process_emphasis_match(arena, delims, opener_idx, closer_idx);
                true // Restart processing
            } else {
                false // No match, continue to next closer
            }
        }
        '\'' | '"' => {
            process_smart_quotes(arena, delims, opener_idx, closer_idx, closer_char);
            // Mark both opener and closer as processed for smart quotes
            if let Some(opener_idx) = opener_idx {
                delims[opener_idx].6 = true;
            }
            delims[closer_idx].6 = true;
            false // Continue to next closer
        }
        _ => false, // Unknown delimiter type
    }
}

/// Main emphasis processing function
/// Based on commonmark.js processEmphasis function and cmark implementation
///
/// # Arguments
///
/// * `arena` - The node arena for creating emphasis nodes
/// * `delimiters` - The delimiter stack to process
/// * `stack_bottom_marker` - If provided, only process delimiters after this one
///
/// # Algorithm
///
/// 1. Collect all delimiters into a vector
/// 2. Find the starting index based on stack_bottom_marker
/// 3. Process closers from left to right
/// 4. For each closer, find a matching opener
/// 5. Create emphasis nodes for matches
/// 6. Rebuild the delimiter stack
pub fn process_emphasis(
    arena: &mut NodeArena,
    delimiters: &mut Option<Box<Delimiter>>,
    stack_bottom_marker: Option<(NodeId, usize)>,
) {
    // Collect all delimiter info into a mutable vector
    let mut delims = collect_delimiters(delimiters);

    // Find the starting index based on stack_bottom_marker
    let start_idx = find_start_index(&delims, stack_bottom_marker);

    // Process closers from left to right
    // Restart from start_idx when a match is found to handle updated counts
    let mut progress = true;
    while progress {
        progress = false;
        let mut closer_idx = start_idx;

        while closer_idx < delims.len() {
            // Skip invalid closers
            if !is_valid_closer(&delims[closer_idx]) {
                closer_idx += 1;
                continue;
            }

            // Process this closer
            if process_closer(arena, &mut delims, closer_idx, start_idx) {
                // Match found, restart processing from start_idx
                progress = true;
                break;
            }

            closer_idx += 1;
        }
    }

    // Rebuild the delimiter stack
    rebuild_delimiter_stack(delimiters, delims, start_idx);
}

/// Remove delimiters that are inside a link from the delimiter stack
/// This is called after processing emphasis inside link text
/// The `stack_bottom_marker` identifies the delimiter that was the top of stack before the link opener
/// After process_emphasis is called with the stack_bottom_marker, the delimiter stack
/// should already only contain delimiters up to and including the stack_bottom.
/// This function ensures any remaining delimiters inside the link are removed.
pub fn remove_delimiters_inside_link(
    delimiters: &mut Option<Box<Delimiter>>,
    stack_bottom_marker: Option<(NodeId, usize)>,
) {
    // If there's no stack bottom marker, remove all delimiters
    let Some((stack_inl_text, stack_orig_delims)) = stack_bottom_marker else {
        *delimiters = None;
        return;
    };

    // Find the delimiter matching the stack_bottom_marker
    // We need to find the delimiter with matching inl_text and orig_delims
    // and keep only that delimiter and its predecessors
    let mut found = false;

    // First, check if the top of stack is the stack_bottom
    if let Some(ref top) = delimiters {
        if top.inl_text == stack_inl_text && top.orig_delims == stack_orig_delims {
            // The top of stack is already the stack_bottom, nothing to remove
            found = true;
        }
    }

    if !found {
        // We need to search through the stack to find where to truncate
        // Since we can't easily traverse backwards in a singly-linked list,
        // we'll collect all delimiters and rebuild the stack

        // Collect all delimiters
        let all_delims = collect_delimiters(delimiters);

        // Find the index of the stack_bottom delimiter
        let mut stack_bottom_idx = None;
        for (idx, (inl_text, _, _, _, orig_delims, _, _)) in
            all_delims.iter().enumerate()
        {
            if *inl_text == stack_inl_text && *orig_delims == stack_orig_delims {
                stack_bottom_idx = Some(idx);
                break;
            }
        }

        // If we found the stack_bottom, keep only delimiters up to and including it
        if let Some(idx) = stack_bottom_idx {
            // Keep delimiters from 0 to idx (inclusive)
            let delims_to_keep: SmallVec<[DelimInfo; 32]> =
                all_delims.into_iter().take(idx + 1).collect();

            // Rebuild the delimiter stack
            *delimiters = None;
            for (node_id, char, can_open, can_close, orig_delims, num_delims, _) in
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
            // Stack bottom not found, this shouldn't happen in normal operation
            // But to be safe, we clear all delimiters
            *delimiters = None;
        }
    }
}
