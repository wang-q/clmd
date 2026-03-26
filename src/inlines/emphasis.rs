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
    // Collect all delimiter info into a vector (safer than raw pointers)
    // We store: (inl_text, delim_char, can_open, can_close, orig_delims, num_delims)
    // Use SmallVec to avoid heap allocation for small stacks
    let mut delims: SmallVec<[(NodeId, char, bool, bool, usize, usize); 32]> =
        SmallVec::new();

    // Traverse the delimiter stack and collect info
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

    // Find the starting index based on stack_bottom
    // stack_bottom is a delimiter that should NOT be processed (it's the boundary)
    let start_idx = if let Some(sb) = stack_bottom {
        // Find the position of stack_bottom in delims
        delims
            .iter()
            .position(|(node_id, _, _, _, orig, _)| {
                *node_id == sb.inl_text && *orig == sb.orig_delims
            })
            .map(|i| i + 1)
            .unwrap_or(0)
    } else {
        0
    };

    // Initialize openers_bottom for each delimiter type
    // Index mapping: 0=" 1=' 2-7=_ (based on can_open and length % 3) 8-13=* (based on can_open and length % 3)
    let mut openers_bottom: [usize; 14] = [start_idx; 14];

    // Process closers from left to right, starting from start_idx
    let mut closer_idx = start_idx;
    while closer_idx < delims.len() {
        let (
            closer_inl,
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
        let openers_bottom_idx = match closer_char {
            '"' => 0,
            '\'' => 1,
            '_' => 2 + if closer_can_open { 3 } else { 0 } + (closer_orig_delims % 3),
            '*' => 8 + if closer_can_open { 3 } else { 0 } + (closer_orig_delims % 3),
            _ => {
                closer_idx += 1;
                continue;
            }
        };

        // Look for matching opener
        // First, try to find an opener with the same number of delimiters
        let mut opener_idx = closer_idx;
        let mut opener_found = false;

        while opener_idx > openers_bottom[openers_bottom_idx] {
            opener_idx -= 1;
            let (
                _,
                opener_char,
                opener_can_open,
                opener_can_close,
                opener_orig_delims,
                _,
            ) = delims[opener_idx];

            if opener_char == closer_char
                && opener_can_open
                && opener_orig_delims == closer_orig_delims
            {
                // Check for odd match rule
                let odd_match = (closer_can_open || opener_can_close)
                    && closer_orig_delims % 3 != 0
                    && (opener_orig_delims + closer_orig_delims) % 3 == 0;

                if !odd_match {
                    opener_found = true;
                    break;
                }
            }
        }

        // If no exact match found, look for any matching opener
        if !opener_found {
            opener_idx = closer_idx;
            while opener_idx > openers_bottom[openers_bottom_idx] {
                opener_idx -= 1;
                let (
                    _,
                    opener_char,
                    opener_can_open,
                    opener_can_close,
                    opener_orig_delims,
                    _,
                ) = delims[opener_idx];

                if opener_char == closer_char && opener_can_open {
                    // Check for odd match rule
                    let odd_match = (closer_can_open || opener_can_close)
                        && closer_orig_delims % 3 != 0
                        && (opener_orig_delims + closer_orig_delims) % 3 == 0;

                    if !odd_match {
                        opener_found = true;
                        break;
                    }
                }
            }
        }

        let old_closer_idx = closer_idx;

        if closer_char == '*' || closer_char == '_' {
            if opener_found {
                let (opener_inl, _, _, _, opener_orig_delims, _) = delims[opener_idx];

                // Calculate number of delimiters to use
                let use_delims = if opener_orig_delims >= 2 && closer_orig_delims >= 2 {
                    2
                } else {
                    1
                };

                // Update the text nodes to remove used delimiters
                let opener_text = {
                    let node = arena.get_mut(opener_inl);
                    if let NodeData::Text { ref mut literal } = node.data {
                        let new_len = literal.len().saturating_sub(use_delims);
                        literal.truncate(new_len);
                        literal.clone()
                    } else {
                        String::new()
                    }
                };

                let closer_text = {
                    let node = arena.get_mut(closer_inl);
                    if let NodeData::Text { ref mut literal } = node.data {
                        let new_len = literal.len().saturating_sub(use_delims);
                        literal.truncate(new_len);
                        literal.clone()
                    } else {
                        String::new()
                    }
                };

                // Create emphasis or strong node
                let emph_type = if use_delims == 1 {
                    NodeType::Emph
                } else {
                    NodeType::Strong
                };
                let emph_node = arena.alloc(crate::arena::Node::new(emph_type));

                // Move nodes between opener and closer into the emphasis node
                let mut current_child = arena.get(opener_inl).next;
                while let Some(child_id) = current_child {
                    if child_id == closer_inl {
                        break;
                    }
                    let next_child = arena.get(child_id).next;

                    // Unlink from current position and append to emphasis
                    TreeOps::unlink(arena, child_id);
                    TreeOps::append_child(arena, emph_node, child_id);

                    current_child = next_child;
                }

                // Insert emphasis node after opener
                TreeOps::insert_after(arena, opener_inl, emph_node);

                // Remove delimiter inline nodes if they are now empty
                if opener_text.is_empty() {
                    TreeOps::unlink(arena, opener_inl);
                }
                if closer_text.is_empty() {
                    TreeOps::unlink(arena, closer_inl);
                }
                // Always advance to next closer
                closer_idx += 1;
            } else {
                closer_idx += 1;
            }
        } else if closer_char == '\'' || closer_char == '"' {
            // Smart quote handling
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

            if opener_found {
                let (opener_inl, _, _, _, _, _) = delims[opener_idx];
                let open_quote = if closer_char == '\'' {
                    '\u{2018}'
                } else {
                    '\u{201C}'
                };
                {
                    let node = arena.get_mut(opener_inl);
                    if let NodeData::Text { ref mut literal } = node.data {
                        *literal = open_quote.to_string();
                    }
                }
            }

            closer_idx += 1;
        }

        if !opener_found {
            openers_bottom[openers_bottom_idx] = old_closer_idx;
        }
    }

    // Rebuild the delimiter stack, keeping only delimiters up to start_idx
    // These are the delimiters that were before stack_bottom (or all if stack_bottom is None)
    if start_idx > 0 {
        // Keep delimiters from 0 to start_idx-1
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
        // Clear delimiter stack
        *delimiters = None;
    }
}
