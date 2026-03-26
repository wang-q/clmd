//! Block info management for BlockParser
//!
//! This module provides methods to access and modify BlockInfo data
//! associated with parser nodes.

use crate::arena::NodeId;
use crate::blocks::{BlockInfo, BlockParser};

impl<'a> BlockParser<'a> {
    /// Get the index for a node, creating a new slot if needed
    #[inline]
    pub(crate) fn get_or_create_index(&mut self, node_id: NodeId) -> usize {
        if let Some(&index) = self.node_to_index.get(&node_id) {
            index
        } else {
            let index = self.next_index;
            self.node_to_index.insert(node_id, index);
            self.block_info.push(None);
            self.next_index += 1;
            index
        }
    }

    #[inline]
    pub(crate) fn get_index(&self, node_id: NodeId) -> Option<usize> {
        self.node_to_index.get(&node_id).copied()
    }

    pub(crate) fn get_block_info(&self, node_id: NodeId) -> Option<&BlockInfo> {
        self.get_index(node_id)
            .and_then(|idx| self.block_info.get(idx))
            .and_then(|opt| opt.as_ref())
    }

    pub(crate) fn get_block_info_mut(
        &mut self,
        node_id: NodeId,
    ) -> Option<&mut BlockInfo> {
        if let Some(idx) = self.get_index(node_id) {
            if let Some(Some(ref mut info)) = self.block_info.get_mut(idx) {
                return Some(info);
            }
        }
        None
    }

    pub(crate) fn set_block_info(&mut self, node_id: NodeId, info: BlockInfo) {
        let idx = self.get_or_create_index(node_id);
        if idx < self.block_info.len() {
            self.block_info[idx] = Some(info);
        }
    }

    pub(crate) fn is_open(&self, node_id: NodeId) -> bool {
        self.get_block_info(node_id)
            .is_some_and(|info| info.is_open)
    }

    pub(crate) fn set_open(&mut self, node_id: NodeId, open: bool) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.is_open = open;
        }
    }

    pub(crate) fn get_string_content(&self, node_id: NodeId) -> String {
        self.get_block_info(node_id)
            .map_or(String::new(), |info| info.string_content.clone())
    }

    pub(crate) fn set_string_content(&mut self, node_id: NodeId, content: String) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.string_content = content;
        }
    }

    pub(crate) fn append_string_content(&mut self, node_id: NodeId, value: &str) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.string_content.push_str(value);
        }
    }

    pub(crate) fn is_fenced_code_block(&self, node_id: NodeId) -> bool {
        self.get_block_info(node_id)
            .is_some_and(|info| info.fence_length > 0)
    }

    pub(crate) fn get_fence_info(&self, node_id: NodeId) -> (char, usize, usize) {
        self.get_block_info(node_id).map_or(('\0', 0, 0), |info| {
            (info.fence_char, info.fence_length, info.fence_offset)
        })
    }

    pub(crate) fn set_fence_info(
        &mut self,
        node_id: NodeId,
        fence_char: char,
        fence_length: usize,
        fence_offset: usize,
    ) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.fence_char = fence_char;
            info.fence_length = fence_length;
            info.fence_offset = fence_offset;
        }
    }

    pub(crate) fn get_list_data(&self, item: NodeId) -> (usize, usize) {
        self.get_block_info(item)
            .map_or((0, 2), |info| (info.marker_offset, info.padding))
    }

    pub(crate) fn set_list_data(
        &mut self,
        item: NodeId,
        marker_offset: usize,
        padding: usize,
    ) {
        if let Some(info) = self.get_block_info_mut(item) {
            info.marker_offset = marker_offset;
            info.padding = padding;
        }
    }

    pub(crate) fn get_html_block_type(&self, node_id: NodeId) -> u8 {
        self.get_block_info(node_id)
            .map_or(0, |info| info.html_block_type)
    }

    pub(crate) fn set_html_block_type(&mut self, node_id: NodeId, block_type: u8) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.html_block_type = block_type;
        }
    }

    pub(crate) fn is_setext(&self, node_id: NodeId) -> bool {
        self.get_block_info(node_id)
            .is_some_and(|info| info.is_setext)
    }

    pub(crate) fn set_setext(&mut self, node_id: NodeId, setext: bool) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.is_setext = setext;
        }
    }

    pub(crate) fn get_last_line_blank(&self, node_id: NodeId) -> bool {
        self.get_block_info(node_id)
            .is_some_and(|info| info.last_line_blank)
    }

    pub(crate) fn set_last_line_blank(&mut self, node_id: NodeId, blank: bool) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.last_line_blank = blank;
        }
    }

    pub(crate) fn get_start_line(&self, node_id: NodeId) -> usize {
        self.arena.get(node_id).source_pos.start_line as usize
    }
}
