//! LaTeX document reader.
//!
//! This module provides a reader for LaTeX format.
//!
//! # Example
//!
//! ```ignore
//! use clmd::readers::LaTeXReader;
//! use clmd::options::ReaderOptions;
//!
//! let reader = LaTeXReader;
//! let input = r#"\documentclass{article}
//! \begin{document}
//! \section{Hello}
//! World
//! \end{document}"#;
//! let (arena, root) = reader.read(input, &ReaderOptions::default()).unwrap();
//! ```

use crate::core::arena::{Node, NodeArena, NodeId, TreeOps};
use crate::core::error::ClmdResult;
use crate::core::nodes::{NodeCodeBlock, NodeHeading, NodeValue};
use crate::options::{InputFormat, ReaderOptions};
use crate::readers::Reader;

/// LaTeX document reader.
#[derive(Debug, Clone, Copy)]
pub struct LaTeXReader;

impl LaTeXReader {
    /// Create a new LaTeX reader.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LaTeXReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for LaTeXReader {
    fn read(
        &self,
        input: &str,
        _options: &ReaderOptions,
    ) -> ClmdResult<(NodeArena, NodeId)> {
        let mut arena = NodeArena::new();
        let root = parse_latex(input, &mut arena)?;
        Ok((arena, root))
    }

    fn format(&self) -> &'static str {
        "latex"
    }

    fn extensions(&self) -> &[&'static str] {
        &["tex", "latex"]
    }

    fn input_format(&self) -> InputFormat {
        InputFormat::Latex
    }
}

/// Parse LaTeX content into an AST.
fn parse_latex(input: &str, arena: &mut NodeArena) -> ClmdResult<NodeId> {
    let root = arena.alloc(Node::with_value(NodeValue::Document));
    let mut parser = LatexParser::new(input, arena, root);
    parser.parse()?;
    Ok(root)
}

/// LaTeX parser state.
struct LatexParser<'a, 'b> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    arena: &'b mut NodeArena,
    root: NodeId,
    current_container: NodeId,
}

impl<'a, 'b> LatexParser<'a, 'b> {
    fn new(input: &'a str, arena: &'b mut NodeArena, root: NodeId) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            arena,
            root,
            current_container: root,
        }
    }

    fn parse(&mut self) -> ClmdResult<()> {
        while self.chars.peek().is_some() {
            self.skip_whitespace_and_comments();

            if self.chars.peek().is_none() {
                break;
            }

            // Check for command
            if self.chars.peek() == Some(&'\\') {
                self.parse_command()?;
            } else {
                // Parse text content
                self.parse_text()?;
            }
        }

        Ok(())
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while let Some(&c) = self.chars.peek() {
                if c.is_whitespace() {
                    self.chars.next();
                } else {
                    break;
                }
            }

            // Skip LaTeX comments (% to end of line)
            if let Some(&'%') = self.chars.peek() {
                while let Some(c) = self.chars.next() {
                    if c == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn parse_command(&mut self) -> ClmdResult<()> {
        self.chars.next(); // consume \

        let cmd_name = self.parse_command_name();

        match cmd_name.as_str() {
            "section" | "subsection" | "subsubsection" => {
                let level = match cmd_name.as_str() {
                    "section" => 1,
                    "subsection" => 2,
                    "subsubsection" => 3,
                    _ => 1,
                };
                self.parse_section(level)?;
            }
            "paragraph" => {
                self.parse_paragraph()?;
            }
            "begin" => {
                self.parse_environment()?;
            }
            "item" => {
                self.parse_item()?;
            }
            "textbf" => {
                self.parse_inline_command("strong")?;
            }
            "textit" => {
                self.parse_inline_command("emph")?;
            }
            "texttt" => {
                self.parse_inline_command("code")?;
            }
            "href" => {
                self.parse_href()?;
            }
            "label" | "ref" | "cite" => {
                // Skip these commands with their arguments
                self.skip_brace_argument();
            }
            "documentclass" | "usepackage" | "title" | "author" | "date" => {
                // Skip preamble commands
                self.skip_brace_argument();
                if let Some(&'[') = self.chars.peek() {
                    self.skip_optional_argument();
                }
            }
            _ => {
                // Unknown command, try to handle gracefully
                if let Some(&'{') = self.chars.peek() {
                    self.skip_brace_argument();
                }
            }
        }

        Ok(())
    }

    fn parse_command_name(&mut self) -> String {
        let mut name = String::new();

        while let Some(&c) = self.chars.peek() {
            if c.is_alphabetic() || c == '*' {
                name.push(c);
                self.chars.next();
            } else {
                break;
            }
        }

        name
    }

    fn parse_section(&mut self, level: u8) -> ClmdResult<()> {
        self.skip_whitespace_and_comments();

        // Get section title
        let title = self.parse_brace_argument();

        // Create heading node
        let heading = Node::with_value(NodeValue::Heading(NodeHeading {
            level,
            setext: false,
            closed: false,
        }));
        let heading_id = self.arena.alloc(heading);

        // Add title as text
        if !title.is_empty() {
            let text = Node::with_value(NodeValue::Text(title.into_boxed_str()));
            let text_id = self.arena.alloc(text);
            TreeOps::append_child(self.arena, heading_id, text_id);
        }

        TreeOps::append_child(self.arena, self.current_container, heading_id);

        Ok(())
    }

    fn parse_paragraph(&mut self) -> ClmdResult<()> {
        self.skip_whitespace_and_comments();

        // Get paragraph title
        let title = self.parse_brace_argument();

        // Create paragraph heading (like a subparagraph)
        let heading = Node::with_value(NodeValue::Heading(NodeHeading {
            level: 4,
            setext: false,
            closed: false,
        }));
        let heading_id = self.arena.alloc(heading);

        if !title.is_empty() {
            let text = Node::with_value(NodeValue::Text(title.into_boxed_str()));
            let text_id = self.arena.alloc(text);
            TreeOps::append_child(self.arena, heading_id, text_id);
        }

        TreeOps::append_child(self.arena, self.current_container, heading_id);

        Ok(())
    }

    fn parse_environment(&mut self) -> ClmdResult<()> {
        self.skip_whitespace_and_comments();

        let env_name = self.parse_brace_argument();

        match env_name.as_str() {
            "document" => {
                // Document environment - content is already being parsed
            }
            "itemize" | "enumerate" => {
                self.parse_list(&env_name)?;
            }
            "verbatim" => {
                self.parse_verbatim()?;
            }
            "quote" | "quotation" => {
                self.parse_quote()?;
            }
            "center" => {
                self.parse_center()?;
            }
            _ => {
                // Unknown environment, skip content
                self.skip_environment_content(&env_name);
            }
        }

        Ok(())
    }

    fn parse_list(&mut self, list_type: &str) -> ClmdResult<()> {
        use crate::core::nodes::{ListDelimType, ListType, NodeList};

        let list_type_enum = if list_type == "enumerate" {
            ListType::Ordered
        } else {
            ListType::Bullet
        };

        let list = Node::with_value(NodeValue::List(NodeList {
            list_type: list_type_enum,
            marker_offset: 0,
            padding: 0,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        }));
        let list_id = self.arena.alloc(list);

        let parent_container = self.current_container;
        self.current_container = list_id;

        // Parse items until \end{...}
        loop {
            self.skip_whitespace_and_comments();

            // Check for \end command
            if self.chars.peek() == Some(&'\\') {
                let checkpoint = self.chars.clone();
                self.chars.next(); // consume \
                let cmd = self.parse_command_name();
                if cmd == "end" {
                    self.skip_whitespace_and_comments();
                    let end_env = self.parse_brace_argument();
                    if end_env == list_type {
                        break;
                    }
                }
                // Not our end command, restore and continue
                self.chars = checkpoint;
            }

            if self.chars.peek().is_none() {
                break;
            }

            // Continue parsing
            if self.chars.peek() == Some(&'\\') {
                self.parse_command()?;
            } else {
                self.parse_text()?;
            }
        }

        self.current_container = parent_container;
        TreeOps::append_child(self.arena, self.current_container, list_id);

        Ok(())
    }

    fn parse_item(&mut self) -> ClmdResult<()> {
        use crate::core::nodes::{ListDelimType, ListType, NodeList};

        // Check for optional argument
        if let Some(&'[') = self.chars.peek() {
            self.skip_optional_argument();
        }

        let item = Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        }));
        let item_id = self.arena.alloc(item);

        // Parse item content until next \item or \end
        let parent_container = self.current_container;
        self.current_container = item_id;

        loop {
            self.skip_whitespace_and_comments();

            if self.chars.peek().is_none() {
                break;
            }

            // Check for next item or end
            if self.chars.peek() == Some(&'\\') {
                let checkpoint = self.chars.clone();
                self.chars.next();
                let cmd = self.parse_command_name();
                if cmd == "item" || cmd == "end" {
                    // Restore position and break
                    self.chars = checkpoint;
                    break;
                }
                // Not a terminating command, restore and continue
                self.chars = checkpoint;
            }

            if self.chars.peek() == Some(&'\\') {
                self.parse_command()?;
            } else {
                self.parse_text()?;
            }
        }

        self.current_container = parent_container;
        TreeOps::append_child(self.arena, self.current_container, item_id);

        Ok(())
    }

    fn parse_verbatim(&mut self) -> ClmdResult<()> {
        let mut content = String::new();

        // Read verbatim content until \end{verbatim}
        loop {
            if self.chars.peek() == Some(&'\\') {
                let checkpoint = self.chars.clone();
                self.chars.next();
                let cmd = self.parse_command_name();
                if cmd == "end" {
                    self.skip_whitespace_and_comments();
                    let end_env = self.parse_brace_argument();
                    if end_env == "verbatim" {
                        break;
                    } else {
                        content.push('\\');
                        content.push_str(&cmd);
                        content.push('{');
                        content.push_str(&end_env);
                        content.push('}');
                    }
                } else {
                    content.push('\\');
                    content.push_str(&cmd);
                }
            } else if let Some(c) = self.chars.next() {
                content.push(c);
            } else {
                break;
            }
        }

        let code_block =
            Node::with_value(NodeValue::CodeBlock(Box::new(NodeCodeBlock {
                literal: content,
                info: "".into(),
                fence_length: 0,
                fence_offset: 0,
                fenced: false,
                fence_char: 0,
                closed: true,
            })));
        let code_id = self.arena.alloc(code_block);

        TreeOps::append_child(self.arena, self.current_container, code_id);

        Ok(())
    }

    fn parse_quote(&mut self) -> ClmdResult<()> {
        let quote = Node::with_value(NodeValue::BlockQuote);
        let quote_id = self.arena.alloc(quote);

        let parent_container = self.current_container;
        self.current_container = quote_id;

        // Parse content until \end{quote}
        loop {
            self.skip_whitespace_and_comments();

            if self.chars.peek() == Some(&'\\') {
                let checkpoint = self.chars.clone();
                self.chars.next();
                let cmd = self.parse_command_name();
                if cmd == "end" {
                    self.skip_whitespace_and_comments();
                    let end_env = self.parse_brace_argument();
                    if end_env == "quote" || end_env == "quotation" {
                        break;
                    }
                }
                self.chars = checkpoint;
            }

            if self.chars.peek().is_none() {
                break;
            }

            if self.chars.peek() == Some(&'\\') {
                self.parse_command()?;
            } else {
                self.parse_text()?;
            }
        }

        self.current_container = parent_container;
        TreeOps::append_child(self.arena, self.current_container, quote_id);

        Ok(())
    }

    fn parse_center(&mut self) -> ClmdResult<()> {
        // For now, treat center as a regular paragraph
        // Parse content until \end{center}
        loop {
            self.skip_whitespace_and_comments();

            if self.chars.peek() == Some(&'\\') {
                let checkpoint = self.chars.clone();
                self.chars.next();
                let cmd = self.parse_command_name();
                if cmd == "end" {
                    self.skip_whitespace_and_comments();
                    let end_env = self.parse_brace_argument();
                    if end_env == "center" {
                        break;
                    }
                }
                self.chars = checkpoint;
            }

            if self.chars.peek().is_none() {
                break;
            }

            if self.chars.peek() == Some(&'\\') {
                self.parse_command()?;
            } else {
                self.parse_text()?;
            }
        }

        Ok(())
    }

    fn parse_text(&mut self) -> ClmdResult<()> {
        let mut text = String::new();

        while let Some(&c) = self.chars.peek() {
            if c == '\\' || c == '\n' {
                break;
            }
            text.push(c);
            self.chars.next();
        }

        if !text.trim().is_empty() {
            // Create paragraph if needed
            let para = Node::with_value(NodeValue::Paragraph);
            let para_id = self.arena.alloc(para);

            let text_node = Node::with_value(NodeValue::Text(
                text.trim().to_string().into_boxed_str(),
            ));
            let text_id = self.arena.alloc(text_node);

            TreeOps::append_child(self.arena, para_id, text_id);
            TreeOps::append_child(self.arena, self.current_container, para_id);
        }

        // Handle newlines
        if self.chars.peek() == Some(&'\n') {
            self.chars.next();
        }

        Ok(())
    }

    fn parse_inline_command(&mut self, style: &str) -> ClmdResult<()> {
        self.skip_whitespace_and_comments();
        let content = self.parse_brace_argument();

        let style_node = match style {
            "strong" => Node::with_value(NodeValue::Strong),
            "emph" => Node::with_value(NodeValue::Emph),
            "code" => {
                let code = Node::with_value(NodeValue::Code(Box::new(
                    crate::core::nodes::NodeCode {
                        literal: content.clone(),
                        num_backticks: 1,
                    },
                )));
                let code_id = self.arena.alloc(code);

                // Add to current paragraph
                if let Some(last_child) = self.get_last_child(self.current_container) {
                    let node = self.arena.get(last_child);
                    if matches!(node.value, NodeValue::Paragraph) {
                        TreeOps::append_child(self.arena, last_child, code_id);
                    }
                }
                return Ok(());
            }
            _ => Node::with_value(NodeValue::Text(content.clone().into_boxed_str())),
        };

        let style_id = self.arena.alloc(style_node);

        if !content.is_empty() {
            let text = Node::with_value(NodeValue::Text(content.into_boxed_str()));
            let text_id = self.arena.alloc(text);
            TreeOps::append_child(self.arena, style_id, text_id);
        }

        // Add to current paragraph
        if let Some(last_child) = self.get_last_child(self.current_container) {
            let node = self.arena.get(last_child);
            if matches!(node.value, NodeValue::Paragraph) {
                TreeOps::append_child(self.arena, last_child, style_id);
            }
        }

        Ok(())
    }

    fn parse_href(&mut self) -> ClmdResult<()> {
        use crate::core::nodes::NodeLink;

        self.skip_whitespace_and_comments();
        let url = self.parse_brace_argument();
        self.skip_whitespace_and_comments();
        let text = self.parse_brace_argument();

        let link = Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url,
            title: "".into(),
        })));
        let link_id = self.arena.alloc(link);

        if !text.is_empty() {
            let text_node = Node::with_value(NodeValue::Text(text.into_boxed_str()));
            let text_id = self.arena.alloc(text_node);
            TreeOps::append_child(self.arena, link_id, text_id);
        }

        // Add to current paragraph
        if let Some(last_child) = self.get_last_child(self.current_container) {
            let node = self.arena.get(last_child);
            if matches!(node.value, NodeValue::Paragraph) {
                TreeOps::append_child(self.arena, last_child, link_id);
            }
        }

        Ok(())
    }

    fn parse_brace_argument(&mut self) -> String {
        self.skip_whitespace_and_comments();

        if self.chars.peek() != Some(&'{') {
            return String::new();
        }
        self.chars.next(); // consume {

        let mut content = String::new();
        let mut depth = 1;

        while let Some(c) = self.chars.next() {
            match c {
                '{' => {
                    depth += 1;
                    content.push(c);
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    content.push(c);
                }
                _ => content.push(c),
            }
        }

        content
    }

    fn skip_brace_argument(&mut self) {
        self.skip_whitespace_and_comments();

        if self.chars.peek() != Some(&'{') {
            return;
        }
        self.chars.next();

        let mut depth = 1;
        while let Some(c) = self.chars.next() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    fn skip_optional_argument(&mut self) {
        if self.chars.peek() != Some(&'[') {
            return;
        }
        self.chars.next();

        let mut depth = 1;
        while let Some(c) = self.chars.next() {
            match c {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    fn skip_environment_content(&mut self, env_name: &str) {
        loop {
            self.skip_whitespace_and_comments();

            if self.chars.peek() == Some(&'\\') {
                let checkpoint = self.chars.clone();
                self.chars.next();
                let cmd = self.parse_command_name();
                if cmd == "end" {
                    self.skip_whitespace_and_comments();
                    let end_env = self.parse_brace_argument();
                    if end_env == env_name {
                        break;
                    }
                } else {
                    self.chars = checkpoint;
                }
            }

            if self.chars.next().is_none() {
                break;
            }
        }
    }

    fn get_last_child(&self, node_id: NodeId) -> Option<NodeId> {
        let node = self.arena.get(node_id);
        node.last_child
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latex_reader_basic() {
        let reader = LaTeXReader::new();
        let options = ReaderOptions::default();

        let input = r#"\documentclass{article}
\begin{document}
\section{Hello}
World
\end{document}"#;
        let (arena, root) = reader.read(input, &options).unwrap();

        let node = arena.get(root);
        assert!(matches!(node.value, NodeValue::Document));
    }

    #[test]
    fn test_latex_reader_section() {
        let reader = LaTeXReader::new();
        let options = ReaderOptions::default();

        let input = r#"\section{Title}"#;
        let (arena, root) = reader.read(input, &options).unwrap();

        let doc = arena.get(root);
        assert!(doc.first_child.is_some());
    }

    #[test]
    fn test_latex_reader_format() {
        let reader = LaTeXReader::new();
        assert_eq!(reader.format(), "latex");
        assert!(reader.extensions().contains(&"tex"));
        assert!(reader.extensions().contains(&"latex"));
    }
}
