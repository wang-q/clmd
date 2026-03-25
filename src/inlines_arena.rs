//! Inline parsing with Arena allocation
//!
//! This is the Arena-based version of inline parsing.

use crate::arena::{NodeArena, NodeId, TreeOps};
use crate::arena::Node;
use crate::node::{NodeData, NodeType};

/// Inline parser using Arena allocation
pub struct InlineParser<'a> {
    arena: &'a mut NodeArena,
    input: &'a str,
    pos: usize,
}

impl<'a> InlineParser<'a> {
    /// Create a new inline parser
    pub fn new(arena: &'a mut NodeArena, input: &'a str) -> Self {
        Self { arena, input, pos: 0 }
    }

    /// Parse inline content and append to parent node
    pub fn parse(&mut self, parent_id: NodeId) {
        while self.pos < self.input.len() {
            let c = self.peek_char();
            
            match c {
                '*' | '_' => self.parse_emphasis(parent_id),
                '`' => self.parse_code(parent_id),
                '[' => self.parse_link(parent_id),
                '!' => self.parse_image(parent_id),
                '<' => self.parse_autolink_or_html(parent_id),
                '\\' => self.parse_escape(parent_id),
                '&' => self.parse_entity(parent_id),
                '\n' => self.parse_softbreak(parent_id),
                _ => self.parse_text(parent_id),
            }
        }
    }

    fn peek_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let c = self.peek_char();
            self.pos += c.len_utf8();
            Some(c)
        } else {
            None
        }
    }

    fn parse_emphasis(&mut self, parent_id: NodeId) {
        let start = self.pos;
        let c = self.peek_char();
        let mut count = 0;
        
        while self.peek_char() == c && count < 3 {
            self.advance();
            count += 1;
        }

        // Simple implementation: just create text node
        let text = &self.input[start..self.pos];
        self.create_text_node(parent_id, text);
    }

    fn parse_code(&mut self, parent_id: NodeId) {
        let start = self.pos;
        let mut backticks = 0;
        
        while self.peek_char() == '`' {
            self.advance();
            backticks += 1;
        }

        // Find closing backticks
        let content_start = self.pos;
        let mut found = false;
        
        while self.pos < self.input.len() {
            if self.input[self.pos..].starts_with(&"`".repeat(backticks)) {
                found = true;
                break;
            }
            self.advance();
        }

        if found {
            let content = &self.input[content_start..self.pos];
            let code_node = self.arena.alloc(Node::with_data(
                NodeType::Code,
                NodeData::Code { literal: content.to_string() }
            ));
            TreeOps::append_child(self.arena, parent_id, code_node);
            // Skip closing backticks
            for _ in 0..backticks {
                self.advance();
            }
        } else {
            // No closing backticks, treat as text
            self.pos = start;
            self.parse_text(parent_id);
        }
    }

    fn parse_link(&mut self, parent_id: NodeId) {
        self.advance(); // Skip '['
        
        let text_start = self.pos;
        let mut depth = 1;
        
        while self.pos < self.input.len() && depth > 0 {
            match self.peek_char() {
                '[' => depth += 1,
                ']' => depth -= 1,
                _ => {}
            }
            if depth > 0 {
                self.advance();
            }
        }

        if depth == 0 {
            let text = &self.input[text_start..self.pos];
            self.advance(); // Skip ']'
            
            // Check for link destination
            if self.peek_char() == '(' {
                self.advance(); // Skip '('
                let url_start = self.pos;
                
                while self.pos < self.input.len() && self.peek_char() != ')' {
                    self.advance();
                }
                
                let url = &self.input[url_start..self.pos];
                if self.peek_char() == ')' {
                    self.advance(); // Skip ')'
                }
                
                let link_node = self.arena.alloc(Node::with_data(
                    NodeType::Link,
                    NodeData::Link { 
                        url: url.to_string(), 
                        title: String::new() 
                    }
                ));
                TreeOps::append_child(self.arena, parent_id, link_node);
                
                // Parse link text
                let mut inner_parser = InlineParser::new(self.arena, text);
                inner_parser.parse(link_node);
            } else {
                // Not a valid link, treat as text
                self.create_text_node(parent_id, "[");
                self.pos = text_start;
            }
        } else {
            // Unclosed bracket
            self.create_text_node(parent_id, "[");
        }
    }

    fn parse_image(&mut self, parent_id: NodeId) {
        self.advance(); // Skip '!'
        
        if self.peek_char() == '[' {
            self.advance(); // Skip '['
            
            let alt_start = self.pos;
            let mut depth = 1;
            
            while self.pos < self.input.len() && depth > 0 {
                match self.peek_char() {
                    '[' => depth += 1,
                    ']' => depth -= 1,
                    _ => {}
                }
                if depth > 0 {
                    self.advance();
                }
            }

            if depth == 0 {
                let _alt = &self.input[alt_start..self.pos];
                self.advance(); // Skip ']'
                
                if self.peek_char() == '(' {
                    self.advance(); // Skip '('
                    let url_start = self.pos;
                    
                    while self.pos < self.input.len() && self.peek_char() != ')' {
                        self.advance();
                    }
                    
                    let url = &self.input[url_start..self.pos];
                    if self.peek_char() == ')' {
                        self.advance(); // Skip ')'
                    }
                    
                    let img_node = self.arena.alloc(Node::with_data(
                        NodeType::Image,
                        NodeData::Image { 
                            url: url.to_string(), 
                            title: String::new() 
                        }
                    ));
                    TreeOps::append_child(self.arena, parent_id, img_node);
                } else {
                    self.create_text_node(parent_id, "![");
                    self.pos = alt_start;
                }
            } else {
                self.create_text_node(parent_id, "![");
            }
        } else {
            self.create_text_node(parent_id, "!");
        }
    }

    fn parse_autolink_or_html(&mut self, parent_id: NodeId) {
        let start = self.pos;
        self.advance(); // Skip '<'
        
        // Simple check for autolink
        if self.input[self.pos..].contains('>') {
            let end = self.input[self.pos..].find('>').unwrap() + self.pos;
            let content = &self.input[self.pos..end];
            
            if content.contains("http") || content.contains("@") {
                // Autolink
                let link_node = self.arena.alloc(Node::with_data(
                    NodeType::Link,
                    NodeData::Link { 
                        url: content.to_string(), 
                        title: String::new() 
                    }
                ));
                TreeOps::append_child(self.arena, parent_id, link_node);
                
                let text_node = self.arena.alloc(Node::with_data(
                    NodeType::Text,
                    NodeData::Text { literal: content.to_string() }
                ));
                TreeOps::append_child(self.arena, link_node, text_node);
                
                self.pos = end + 1;
            } else {
                // Inline HTML (simplified)
                let html_node = self.arena.alloc(Node::with_data(
                    NodeType::HtmlInline,
                    NodeData::HtmlInline { literal: format!("<{}>", content) }
                ));
                TreeOps::append_child(self.arena, parent_id, html_node);
                self.pos = end + 1;
            }
        } else {
            self.pos = start;
            self.parse_text(parent_id);
        }
    }

    fn parse_escape(&mut self, parent_id: NodeId) {
        self.advance(); // Skip '\'
        if let Some(c) = self.advance() {
            self.create_text_node(parent_id, &c.to_string());
        } else {
            self.create_text_node(parent_id, "\\");
        }
    }

    fn parse_entity(&mut self, parent_id: NodeId) {
        let start = self.pos;
        self.advance(); // Skip '&'
        
        if let Some(end) = self.input[self.pos..].find(';') {
            let entity = &self.input[start..self.pos + end + 1];
            // Simple entity handling
            let resolved = match entity {
                "&amp;" => "&",
                "&lt;" => "<",
                "&gt;" => ">",
                "&quot;" => "\"",
                _ => entity,
            };
            self.create_text_node(parent_id, resolved);
            self.pos += end + 1;
        } else {
            self.create_text_node(parent_id, "&");
        }
    }

    fn parse_softbreak(&mut self, parent_id: NodeId) {
        self.advance(); // Skip '\n'
        
        let softbreak = self.arena.alloc(Node::new(NodeType::SoftBreak));
        TreeOps::append_child(self.arena, parent_id, softbreak);
    }

    fn parse_text(&mut self, parent_id: NodeId) {
        let start = self.pos;
        
        while self.pos < self.input.len() {
            match self.peek_char() {
                '*' | '_' | '`' | '[' | '!' | '<' | '\\' | '&' | '\n' => break,
                _ => { self.advance(); }
            }
        }
        
        if self.pos > start {
            let text = &self.input[start..self.pos];
            self.create_text_node(parent_id, text);
        }
    }

    fn create_text_node(&mut self, parent_id: NodeId, text: &str) {
        let text_node = self.arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text { literal: text.to_string() }
        ));
        TreeOps::append_child(self.arena, parent_id, text_node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text() {
        let mut arena = NodeArena::new();
        let doc = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        TreeOps::append_child(&mut arena, doc, para);
        
        let mut parser = InlineParser::new(&mut arena, "Hello world");
        parser.parse(para);
        
        assert!(arena.get(para).first_child.is_some());
    }

    #[test]
    fn test_parse_code() {
        let mut arena = NodeArena::new();
        let doc = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        TreeOps::append_child(&mut arena, doc, para);
        
        let mut parser = InlineParser::new(&mut arena, "`code`");
        parser.parse(para);
        
        let child = arena.get(para).first_child.unwrap();
        assert_eq!(arena.get(child).node_type, NodeType::Code);
    }

    #[test]
    fn test_parse_link() {
        let mut arena = NodeArena::new();
        let doc = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        TreeOps::append_child(&mut arena, doc, para);
        
        let mut parser = InlineParser::new(&mut arena, "[text](url)");
        parser.parse(para);
        
        let child = arena.get(para).first_child.unwrap();
        assert_eq!(arena.get(child).node_type, NodeType::Link);
    }
}
