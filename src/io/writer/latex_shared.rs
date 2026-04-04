//! Shared LaTeX rendering core for LaTeX and Beamer writers.
//!
//! This module provides a unified rendering core that supports both LaTeX and Beamer
//! output formats through a state-driven approach, inspired by Pandoc's Writer system.
//!
//! The key design principle is "unified core + differentiated extensions":
//! - Both formats share the same rendering logic
//! - Differences are handled through the `is_beamer` flag in `LatexState`
//!
//! # Example
//!
//! ```ignore
//! use clmd::io::writer::latex_shared::{LatexRenderer, LatexState};
//!
//! // LaTeX mode
//! let state = LatexState::new();
//! let renderer = LatexRenderer::new(&arena, state);
//!
//! // Beamer mode
//! let state = LatexState::beamer();
//! let renderer = LatexRenderer::new(&arena, state);
//! ```

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::{ListType, NodeHeading, NodeList, NodeValue};

/// LaTeX rendering state.
///
/// This struct holds the state used during LaTeX/Beamer rendering.
/// The `is_beamer` flag determines whether to render in LaTeX or Beamer mode.
#[derive(Debug, Clone)]
pub struct LatexState {
    /// Whether to render in Beamer mode (for slides) or standard LaTeX mode.
    pub is_beamer: bool,
    /// Whether to use incremental display for lists in Beamer mode.
    pub incremental: bool,
    /// The slide level for Beamer (which heading level starts a new slide).
    pub slide_level: u8,
    /// Stack of list types for proper nesting.
    pub list_stack: Vec<ListType>,
    /// Whether a blank line is needed before the next block element.
    pub need_blank_line: bool,
    /// Whether we are at the beginning of a line.
    pub beginning_of_line: bool,
    /// Document class for LaTeX mode (e.g., "article", "report", "book").
    pub document_class: String,
    /// Additional packages to include in the preamble.
    pub packages: Vec<String>,
}

impl Default for LatexState {
    fn default() -> Self {
        Self::new()
    }
}

impl LatexState {
    /// Create a new state for standard LaTeX rendering.
    pub fn new() -> Self {
        LatexState {
            is_beamer: false,
            incremental: false,
            slide_level: 2,
            list_stack: Vec::new(),
            need_blank_line: false,
            beginning_of_line: true,
            document_class: "article".to_string(),
            packages: Vec::new(),
        }
    }

    /// Create a new state for Beamer (slides) rendering.
    pub fn beamer() -> Self {
        LatexState {
            is_beamer: true,
            incremental: false,
            slide_level: 2,
            list_stack: Vec::new(),
            need_blank_line: false,
            beginning_of_line: true,
            document_class: "beamer".to_string(),
            packages: vec!["hyperref".to_string(), "ulem".to_string()],
        }
    }

    /// Set the document class.
    pub fn with_document_class(mut self, class: &str) -> Self {
        self.document_class = class.to_string();
        self
    }

    /// Enable incremental display for Beamer lists.
    pub fn with_incremental(mut self, incremental: bool) -> Self {
        self.incremental = incremental;
        self
    }

    /// Set the slide level for Beamer.
    pub fn with_slide_level(mut self, level: u8) -> Self {
        self.slide_level = level;
        self
    }

    /// Add a package to include.
    pub fn add_package(&mut self, package: &str) {
        self.packages.push(package.to_string());
    }
}

/// LaTeX renderer that supports both LaTeX and Beamer output.
///
/// This renderer uses the `LatexState` to determine whether to render
/// in standard LaTeX mode or Beamer slide mode.
#[derive(Debug)]
pub struct LatexRenderer<'a> {
    arena: &'a NodeArena,
    state: LatexState,
    output: String,
}

impl<'a> LatexRenderer<'a> {
    /// Create a new LaTeX renderer with the given state.
    pub fn new(arena: &'a NodeArena, state: LatexState) -> Self {
        LatexRenderer {
            arena,
            state,
            output: String::new(),
        }
    }

    /// Create a new LaTeX renderer for standard LaTeX output.
    pub fn new_latex(arena: &'a NodeArena) -> Self {
        Self::new(arena, LatexState::new())
    }

    /// Create a new LaTeX renderer for Beamer output.
    pub fn new_beamer(arena: &'a NodeArena) -> Self {
        Self::new(arena, LatexState::beamer())
    }

    /// Render the document starting from the given root node.
    pub fn render(mut self, root: NodeId) -> String {
        self.render_node(root, true);

        // Clean up trailing whitespace
        while self.output.ends_with('\n') || self.output.ends_with(' ') {
            self.output.pop();
        }
        self.output.push('\n');

        self.output
    }

    /// Get the current output buffer.
    pub fn output(&self) -> &str {
        &self.output
    }

    /// Get a mutable reference to the output buffer.
    pub fn output_mut(&mut self) -> &mut String {
        &mut self.output
    }

    /// Get a reference to the state.
    pub fn state(&self) -> &LatexState {
        &self.state
    }

    /// Get a mutable reference to the state.
    pub fn state_mut(&mut self) -> &mut LatexState {
        &mut self.state
    }

    /// Check if we are in Beamer mode.
    pub fn is_beamer(&self) -> bool {
        self.state.is_beamer
    }

    fn render_node(&mut self, node_id: NodeId, entering: bool) {
        if entering {
            self.enter_node(node_id);
            let node = self.arena.get(node_id);
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                self.render_node(child_id, true);
                child_opt = self.arena.get(child_id).next;
            }
            self.exit_node(node_id);
        }
    }

    fn enter_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        // Insert blank line before block elements if needed
        if self.state.need_blank_line
            && node.value.is_block()
            && !matches!(
                node.value,
                NodeValue::Document | NodeValue::List(..) | NodeValue::Item(..)
            )
        {
            self.output.push('\n');
            self.state.beginning_of_line = true;
            self.state.need_blank_line = false;
        }

        match &node.value {
            NodeValue::Document => {
                // Document environment is handled separately
            }
            NodeValue::BlockQuote => {
                self.writeln("\\begin{quote}");
            }
            NodeValue::List(NodeList { list_type, .. }) => {
                self.enter_list(*list_type);
            }
            NodeValue::Item(..) => {
                self.write("\\item ");
            }
            NodeValue::CodeBlock(..) => {
                self.render_code_block(node_id);
                self.state.need_blank_line = true;
            }
            NodeValue::HtmlBlock(..) => {
                // Raw HTML blocks are ignored in LaTeX
            }
            NodeValue::Paragraph => {
                // Paragraphs don't have explicit markup in LaTeX
            }
            NodeValue::Heading(..) => {
                self.render_heading_enter(node_id);
                self.state.need_blank_line = true;
            }
            NodeValue::ThematicBreak(..) => {
                self.writeln("\\hrule");
                self.state.need_blank_line = true;
            }
            NodeValue::Text(literal) => {
                self.write(&escape_latex(literal));
            }
            NodeValue::SoftBreak => {
                self.write(" ");
            }
            NodeValue::HardBreak => {
                self.writeln("\\\\");
            }
            NodeValue::Code(code) => {
                self.write("\\texttt{");
                self.write(&escape_latex(&code.literal));
                self.write("}");
            }
            NodeValue::HtmlInline(..) => {
                // Raw HTML inline is ignored in LaTeX
            }
            NodeValue::Emph => {
                self.write("\\emph{");
            }
            NodeValue::Strong => {
                self.write("\\textbf{");
            }
            NodeValue::Link(link) => {
                self.write("\\href{");
                self.write(&escape_latex(&link.url));
                self.write("}{");
            }
            NodeValue::Image(link) => {
                self.write("\\includegraphics{");
                self.write(&escape_latex(&link.url));
                self.write("}");
            }
            NodeValue::Strikethrough => {
                self.write("\\sout{");
            }
            NodeValue::Underline => {
                self.write("\\underline{");
            }
            _ => {}
        }
    }

    fn exit_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        match &node.value {
            NodeValue::Document => {}
            NodeValue::BlockQuote => {
                self.writeln("\\end{quote}");
                self.state.need_blank_line = true;
            }
            NodeValue::List(..) => {
                self.exit_list();
            }
            NodeValue::Item(..) => {
                self.writeln("");
            }
            NodeValue::Paragraph => {
                self.writeln("");
                self.writeln("");
            }
            NodeValue::Heading(..) => {
                self.render_heading_exit(node_id);
            }
            NodeValue::Emph => {
                self.write("}");
            }
            NodeValue::Strong => {
                self.write("}");
            }
            NodeValue::Link(..) => {
                self.write("}");
            }
            NodeValue::Strikethrough => {
                self.write("}");
            }
            NodeValue::Underline => {
                self.write("}");
            }
            _ => {}
        }
    }

    fn enter_list(&mut self, list_type: ListType) {
        match list_type {
            ListType::Bullet => {
                if self.state.is_beamer && self.state.incremental {
                    self.writeln("\\begin{itemize}[<+->]");
                } else {
                    self.writeln("\\begin{itemize}");
                }
            }
            ListType::Ordered => {
                if self.state.is_beamer && self.state.incremental {
                    self.writeln("\\begin{enumerate}[<+->]");
                } else {
                    self.writeln("\\begin{enumerate}");
                }
            }
        }
        self.state.list_stack.push(list_type);
    }

    fn exit_list(&mut self) {
        if let Some(list_type) = self.state.list_stack.pop() {
            match list_type {
                ListType::Bullet => {
                    self.writeln("\\end{itemize}");
                }
                ListType::Ordered => {
                    self.writeln("\\end{enumerate}");
                }
            }
        }
        self.state.need_blank_line = true;
    }

    fn render_code_block(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        if let NodeValue::CodeBlock(code_block) = &node.value {
            if !code_block.info.is_empty() {
                let lang = code_block.info.split_whitespace().next().unwrap_or("");
                self.write("\\begin{lstlisting}[language=");
                self.write(&escape_latex(lang));
                self.writeln("]");
            } else {
                self.writeln("\\begin{verbatim}");
            }

            for line in code_block.literal.lines() {
                self.writeln(line);
            }

            if !code_block.info.is_empty() {
                self.writeln("\\end{lstlisting}");
            } else {
                self.writeln("\\end{verbatim}");
            }
        }
    }

    fn render_heading_enter(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        if let NodeValue::Heading(NodeHeading { level, .. }) = &node.value {
            let cmd = if self.state.is_beamer {
                match level {
                    1 => "\\section{",
                    2 => "\\subsection{",
                    3 => "\\subsubsection{",
                    _ => "\\paragraph{",
                }
            } else {
                match level {
                    1 => "\\section*{",
                    2 => "\\subsection*{",
                    3 => "\\subsubsection*{",
                    4 => "\\paragraph*{",
                    5 => "\\subparagraph*{",
                    _ => "\\paragraph*{",
                }
            };
            self.write(cmd);
        }
    }

    fn render_heading_exit(&mut self, _node_id: NodeId) {
        self.write("}");
    }

    fn write(&mut self, text: &str) {
        self.output.push_str(text);
        self.state.beginning_of_line = false;
    }

    fn writeln(&mut self, text: &str) {
        self.output.push_str(text);
        self.output.push('\n');
        self.state.beginning_of_line = true;
    }
}

/// Escape LaTeX special characters.
///
/// Converts LaTeX special characters to their escaped equivalents.
pub fn escape_latex(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);

    for c in text.chars() {
        match c {
            '\\' => result.push_str("\\textbackslash{}"),
            '{' => result.push_str("\\{"),
            '}' => result.push_str("\\}"),
            '$' => result.push_str("\\$"),
            '&' => result.push_str("\\&"),
            '#' => result.push_str("\\#"),
            '^' => result.push_str("\\textasciicircum{}"),
            '_' => result.push_str("\\_"),
            '~' => result.push_str("\\textasciitilde{}"),
            '%' => result.push_str("\\%"),
            '<' => result.push_str("\\textless{}"),
            '>' => result.push_str("\\textgreater{}"),
            '|' => result.push_str("\\textbar{}"),
            '"' => result.push_str("\\textquotedbl{}"),
            '`' => result.push_str("\\textasciigrave{}"),
            '\'' => result.push_str("\\textquotesingle{}"),
            _ => result.push(c),
        }
    }

    result
}

/// Generate LaTeX document preamble.
///
/// # Arguments
///
/// * `state` - The LaTeX rendering state
///
/// # Returns
///
/// The preamble as a string.
pub fn generate_preamble(state: &LatexState) -> String {
    let mut preamble = String::new();

    preamble.push_str(&format!("\\documentclass{{{}}}\n\n", state.document_class));

    // Standard packages
    preamble.push_str("\\usepackage[utf8]{inputenc}\n");
    preamble.push_str("\\usepackage[T1]{fontenc}\n");
    preamble.push_str("\\usepackage{hyperref}\n");

    if state.is_beamer {
        preamble.push_str("\\usetheme{Madrid}\n");
        preamble.push_str("\\usecolortheme{default}\n");
        preamble.push_str("\\usepackage{ulem}\n");
        preamble.push_str("\\usepackage{listings}\n");
    } else {
        preamble.push_str("\\usepackage{listings}\n");
        preamble.push_str("\\usepackage{graphicx}\n");
    }

    // Additional packages
    for package in &state.packages {
        if package != "hyperref" && package != "ulem" {
            preamble.push_str(&format!("\\usepackage{{{}}}\n", package));
        }
    }

    preamble.push('\n');
    preamble
}

/// Render a node tree as LaTeX.
///
/// This is a convenience function for rendering to standard LaTeX.
pub fn render_latex(arena: &NodeArena, root: NodeId) -> String {
    let renderer = LatexRenderer::new_latex(arena);
    renderer.render(root)
}

/// Render a node tree as Beamer.
///
/// This is a convenience function for rendering to Beamer slides.
pub fn render_beamer(arena: &NodeArena, root: NodeId) -> String {
    let renderer = LatexRenderer::new_beamer(arena);
    renderer.render(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{
        NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeValue,
    };

    #[test]
    fn test_latex_state_new() {
        let state = LatexState::new();
        assert!(!state.is_beamer);
        assert!(!state.incremental);
        assert_eq!(state.slide_level, 2);
        assert_eq!(state.document_class, "article");
    }

    #[test]
    fn test_latex_state_beamer() {
        let state = LatexState::beamer();
        assert!(state.is_beamer);
        assert!(!state.incremental);
        assert_eq!(state.document_class, "beamer");
        assert!(state.packages.contains(&"hyperref".to_string()));
    }

    #[test]
    fn test_latex_state_builder() {
        let state = LatexState::new()
            .with_document_class("report")
            .with_incremental(true)
            .with_slide_level(3);

        assert_eq!(state.document_class, "report");
        assert!(state.incremental);
        assert_eq!(state.slide_level, 3);
    }

    #[test]
    fn test_escape_latex() {
        assert_eq!(escape_latex("hello"), "hello");
        assert_eq!(escape_latex("$100"), "\\$100");
        assert_eq!(escape_latex("100%"), "100\\%");
        assert_eq!(escape_latex("a_b"), "a\\_b");
        assert_eq!(escape_latex("\\"), "\\textbackslash{}");
        assert_eq!(escape_latex("{"), "\\{");
        assert_eq!(escape_latex("}"), "\\}");
        assert_eq!(escape_latex("&"), "\\&");
        assert_eq!(escape_latex("#"), "\\#");
        assert_eq!(escape_latex("^"), "\\textasciicircum{}");
        assert_eq!(escape_latex("~"), "\\textasciitilde{}");
        assert_eq!(escape_latex("<"), "\\textless{}");
        assert_eq!(escape_latex(">"), "\\textgreater{}");
        assert_eq!(escape_latex("|"), "\\textbar{}");
        assert_eq!(escape_latex("\""), "\\textquotedbl{}");
        assert_eq!(escape_latex("'"), "\\textquotesingle{}");
        assert_eq!(escape_latex("`"), "\\textasciigrave{}");
    }

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let latex = render_latex(&arena, root);
        assert!(latex.contains("Hello world"));
    }

    #[test]
    fn test_render_emph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("emphasized")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, emph, text);

        let latex = render_latex(&arena, root);
        assert!(latex.contains("\\emph{emphasized}"));
    }

    #[test]
    fn test_render_strong() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("strong")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, strong, text);

        let latex = render_latex(&arena, root);
        assert!(latex.contains("\\textbf{strong}"));
    }

    #[test]
    fn test_render_code() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = arena.alloc(Node::with_value(NodeValue::Code(Box::new(NodeCode {
            num_backticks: 1,
            literal: "code".to_string(),
        }))));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let latex = render_latex(&arena, root);
        assert!(latex.contains("\\texttt{code}"));
    }

    #[test]
    fn test_render_heading_latex() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));

        TreeOps::append_child(&mut arena, root, heading);

        let latex = render_latex(&arena, root);
        assert!(latex.contains("\\subsection*{"));
    }

    #[test]
    fn test_render_heading_beamer() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));

        TreeOps::append_child(&mut arena, root, heading);

        let beamer = render_beamer(&arena, root);
        assert!(beamer.contains("\\subsection{"));
    }

    #[test]
    fn test_render_link() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com".to_string(),
            title: "".to_string(),
        }))));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("link")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let latex = render_latex(&arena, root);
        assert!(latex.contains("\\href{https://example.com}{link}"));
    }

    #[test]
    fn test_render_blockquote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let blockquote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Quote")));

        TreeOps::append_child(&mut arena, root, blockquote);
        TreeOps::append_child(&mut arena, blockquote, para);
        TreeOps::append_child(&mut arena, para, text);

        let latex = render_latex(&arena, root);
        assert!(latex.contains("\\begin{quote}"));
        assert!(latex.contains("\\end{quote}"));
    }

    #[test]
    fn test_render_code_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let code_block = arena.alloc(Node::with_value(NodeValue::CodeBlock(Box::new(
            NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            },
        ))));

        TreeOps::append_child(&mut arena, root, code_block);

        let latex = render_latex(&arena, root);
        assert!(latex.contains("\\begin{verbatim}"));
        assert!(latex.contains("fn main() {}"));
        assert!(latex.contains("\\end{verbatim}"));
    }

    #[test]
    fn test_render_bullet_list() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            delimiter: crate::core::nodes::ListDelimType::Period,
            start: 1,
            tight: true,
            bullet_char: b'-',
            marker_offset: 0,
            padding: 2,
            is_task_list: false,
        })));
        let item = arena.alloc(Node::with_value(NodeValue::Item(NodeList::default())));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Item")));

        TreeOps::append_child(&mut arena, root, list);
        TreeOps::append_child(&mut arena, list, item);
        TreeOps::append_child(&mut arena, item, para);
        TreeOps::append_child(&mut arena, para, text);

        let latex = render_latex(&arena, root);
        assert!(latex.contains("\\begin{itemize}"));
        assert!(latex.contains("\\item"));
        assert!(latex.contains("\\end{itemize}"));
    }

    #[test]
    fn test_generate_preamble_latex() {
        let state = LatexState::new();
        let preamble = generate_preamble(&state);
        assert!(preamble.contains("\\documentclass{article}"));
        assert!(preamble.contains("\\usepackage[utf8]{inputenc}"));
        assert!(preamble.contains("\\usepackage{hyperref}"));
    }

    #[test]
    fn test_generate_preamble_beamer() {
        let state = LatexState::beamer();
        let preamble = generate_preamble(&state);
        assert!(preamble.contains("\\documentclass{beamer}"));
        assert!(preamble.contains("\\usetheme{Madrid}"));
        assert!(preamble.contains("\\usepackage{ulem}"));
    }
}
