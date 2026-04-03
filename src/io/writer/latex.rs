//! LaTeX writer.
//!
//! This module provides a writer for LaTeX format.

use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::core::nodes::{ListType, NodeHeading, NodeList, NodeValue};
use crate::parse::options::WriterOptions;

/// Render a node tree as LaTeX.
pub fn render(arena: &NodeArena, root: NodeId, options: u32) -> String {
    let mut renderer = LatexRenderer::new(arena, options);
    renderer.render(root)
}

/// Write a document as LaTeX.
pub fn write_latex(
    arena: &NodeArena,
    root: NodeId,
    _options: &WriterOptions,
) -> ClmdResult<String> {
    Ok(render(arena, root, 0))
}

/// LaTeX renderer state
struct LatexRenderer<'a> {
    arena: &'a NodeArena,
    output: String,
    beginning_of_line: bool,
    list_stack: Vec<ListType>,
    need_blank_line: bool,
}

impl<'a> LatexRenderer<'a> {
    fn new(arena: &'a NodeArena, _options: u32) -> Self {
        LatexRenderer {
            arena,
            output: String::new(),
            beginning_of_line: true,
            list_stack: Vec::new(),
            need_blank_line: false,
        }
    }

    fn render(&mut self, root: NodeId) -> String {
        self.render_node(root, true);

        while self.output.ends_with('\n') || self.output.ends_with(' ') {
            self.output.pop();
        }
        self.output.push('\n');

        self.output.clone()
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

        if self.need_blank_line
            && node.value.is_block()
            && !matches!(
                node.value,
                NodeValue::Document | NodeValue::List(..) | NodeValue::Item(..)
            )
        {
            self.output.push('\n');
            self.beginning_of_line = true;
            self.need_blank_line = false;
        }

        match &node.value {
            NodeValue::Document => {
                self.writeln("\\begin{document}");
            }
            NodeValue::BlockQuote => {
                self.writeln("\\begin{quote}");
            }
            NodeValue::List(NodeList { list_type, .. }) => {
                match list_type {
                    ListType::Bullet => {
                        self.writeln("\\begin{itemize}");
                    }
                    ListType::Ordered => {
                        self.writeln("\\begin{enumerate}");
                    }
                }
                self.list_stack.push(*list_type);
            }
            NodeValue::Item(..) => {
                self.write("\\item ");
            }
            NodeValue::CodeBlock(..) => {
                self.render_code_block(node_id);
                self.need_blank_line = true;
            }
            NodeValue::HtmlBlock(..) => {}
            NodeValue::Paragraph => {}
            NodeValue::Heading(..) => {
                self.render_heading(node_id);
                self.need_blank_line = true;
            }
            NodeValue::ThematicBreak(..) => {
                self.writeln("\\hrule");
                self.need_blank_line = true;
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
            NodeValue::HtmlInline(..) => {}
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
            _ => {}
        }
    }

    fn exit_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        match &node.value {
            NodeValue::Document => {
                self.writeln("\\end{document}");
            }
            NodeValue::BlockQuote => {
                self.writeln("\\end{quote}");
                self.need_blank_line = true;
            }
            NodeValue::List(..) => {
                if let Some(list_type) = self.list_stack.pop() {
                    match list_type {
                        ListType::Bullet => {
                            self.writeln("\\end{itemize}");
                        }
                        ListType::Ordered => {
                            self.writeln("\\end{enumerate}");
                        }
                    }
                }
                self.need_blank_line = true;
            }
            NodeValue::Item(..) => {
                self.writeln("");
            }
            NodeValue::Paragraph => {
                self.writeln("");
                self.writeln("");
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
            _ => {}
        }
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

    fn render_heading(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        if let NodeValue::Heading(NodeHeading { level, .. }) = &node.value {
            let cmd = match level {
                1 => "\\section*{",
                2 => "\\subsection*{",
                3 => "\\subsubsection*{",
                4 => "\\paragraph*{",
                5 => "\\subparagraph*{",
                _ => "\\paragraph*{",
            };
            self.write(cmd);
        }
    }

    fn write(&mut self, text: &str) {
        self.output.push_str(text);
        self.beginning_of_line = false;
    }

    fn writeln(&mut self, text: &str) {
        self.output.push_str(text);
        self.output.push('\n');
        self.beginning_of_line = true;
    }
}

/// Escape LaTeX special characters
fn escape_latex(text: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{NodeCode, NodeCodeBlock, NodeHeading, NodeLink};

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let latex = render(&arena, root, 0);
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

        let latex = render(&arena, root, 0);
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

        let latex = render(&arena, root, 0);
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

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\texttt{code}"));
    }

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));

        TreeOps::append_child(&mut arena, root, heading);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\subsection*{"));
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

        let latex = render(&arena, root, 0);
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

        let latex = render(&arena, root, 0);
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

        let latex = render(&arena, root, 0);
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

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\begin{itemize}"));
        assert!(latex.contains("\\item"));
        assert!(latex.contains("\\end{itemize}"));
    }

    #[test]
    fn test_escape_latex() {
        assert_eq!(escape_latex("$100"), "\\$100");
        assert_eq!(escape_latex("100%"), "100\\%");
        assert_eq!(escape_latex("a_b"), "a\\_b");
        assert_eq!(escape_latex("\\"), "\\textbackslash{}");
    }
}
