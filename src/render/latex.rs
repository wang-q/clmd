//! LaTeX renderer

use crate::arena::{NodeArena, NodeId};
use crate::node::{ListType, NodeData, NodeType};

/// Render a node tree as LaTeX
pub fn render(arena: &NodeArena, root: NodeId, options: u32) -> String {
    let mut renderer = LatexRenderer::new(arena, options);
    renderer.render(root)
}

/// LaTeX renderer state
struct LatexRenderer<'a> {
    arena: &'a NodeArena,
    #[allow(dead_code)]
    options: u32,
    output: String,
    beginning_of_line: bool,
    list_stack: Vec<ListType>,
    need_blank_line: bool,
}

impl<'a> LatexRenderer<'a> {
    fn new(arena: &'a NodeArena, options: u32) -> Self {
        LatexRenderer {
            arena,
            options,
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
            && node.node_type.is_block()
            && !matches!(
                node.node_type,
                NodeType::Document | NodeType::List | NodeType::Item
            )
        {
            self.output.push('\n');
            self.beginning_of_line = true;
            self.need_blank_line = false;
        }

        match node.node_type {
            NodeType::Document => {
                self.writeln("\\begin{document}");
            }
            NodeType::BlockQuote => {
                self.writeln("\\begin{quote}");
            }
            NodeType::List => {
                if let NodeData::List { list_type, .. } = &node.data {
                    match list_type {
                        ListType::Bullet => {
                            self.writeln("\\begin{itemize}");
                        }
                        ListType::Ordered => {
                            self.writeln("\\begin{enumerate}");
                        }
                        _ => {}
                    }
                    self.list_stack.push(*list_type);
                }
            }
            NodeType::Item => {
                self.write("\\item ");
            }
            NodeType::CodeBlock => {
                self.render_code_block(node_id);
                self.need_blank_line = true;
            }
            NodeType::HtmlBlock => {}
            NodeType::Paragraph => {}
            NodeType::Heading => {
                self.render_heading(node_id);
                self.need_blank_line = true;
            }
            NodeType::ThematicBreak => {
                self.writeln("\\hrule");
                self.need_blank_line = true;
            }
            NodeType::Text => {
                if let NodeData::Text { literal } = &node.data {
                    self.write(&escape_latex(literal));
                }
            }
            NodeType::SoftBreak => {
                self.write(" ");
            }
            NodeType::LineBreak => {
                self.writeln("\\\\");
            }
            NodeType::Code => {
                if let NodeData::Code { literal } = &node.data {
                    self.write("\\texttt{");
                    self.write(&escape_latex(literal));
                    self.write("}");
                }
            }
            NodeType::HtmlInline => {}
            NodeType::Emph => {
                self.write("\\emph{");
            }
            NodeType::Strong => {
                self.write("\\textbf{");
            }
            NodeType::Link => {
                if let NodeData::Link { url, .. } = &node.data {
                    self.write("\\href{");
                    self.write(&escape_latex(url));
                    self.write("}{");
                }
            }
            NodeType::Image => {
                if let NodeData::Image { url, .. } = &node.data {
                    self.write("\\includegraphics{");
                    self.write(&escape_latex(url));
                    self.write("}");
                }
            }
            _ => {}
        }
    }

    fn exit_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);

        match node.node_type {
            NodeType::Document => {
                self.writeln("\\end{document}");
            }
            NodeType::BlockQuote => {
                self.writeln("\\end{quote}");
                self.need_blank_line = true;
            }
            NodeType::List => {
                if let Some(list_type) = self.list_stack.pop() {
                    match list_type {
                        ListType::Bullet => {
                            self.writeln("\\end{itemize}");
                        }
                        ListType::Ordered => {
                            self.writeln("\\end{enumerate}");
                        }
                        _ => {}
                    }
                }
                self.need_blank_line = true;
            }
            NodeType::Item => {
                self.writeln("");
            }
            NodeType::Paragraph => {
                self.writeln("");
                self.writeln("");
            }
            NodeType::Emph => {
                self.write("}");
            }
            NodeType::Strong => {
                self.write("}");
            }
            NodeType::Link => {
                self.write("}");
            }
            _ => {}
        }
    }

    fn render_code_block(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        if let NodeData::CodeBlock { info, literal } = &node.data {
            if !info.is_empty() {
                let lang = info.split_whitespace().next().unwrap_or("");
                self.write("\\begin{lstlisting}[language=");
                self.write(&escape_latex(lang));
                self.writeln("]");
            } else {
                self.writeln("\\begin{verbatim}");
            }

            for line in literal.lines() {
                self.writeln(line);
            }

            if !info.is_empty() {
                self.writeln("\\end{lstlisting}");
            } else {
                self.writeln("\\end{verbatim}");
            }
        }
    }

    fn render_heading(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        if let NodeData::Heading { level, .. } = &node.data {
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
    use crate::arena::{Node, NodeArena, TreeOps};
    use crate::node::{NodeData, NodeType};

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello world".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("Hello world"));
    }

    #[test]
    fn test_render_emph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let emph = arena.alloc(Node::new(NodeType::Emph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "emphasized".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, emph, text);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\emph{emphasized}"));
    }

    #[test]
    fn test_render_strong() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let strong = arena.alloc(Node::new(NodeType::Strong));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "strong".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, strong, text);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\textbf{strong}"));
    }

    #[test]
    fn test_render_code() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let code = arena.alloc(Node::with_data(
            NodeType::Code,
            NodeData::Code {
                literal: "code".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\texttt{code}"));
    }

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let heading = arena.alloc(Node::with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 2,
                content: "Heading".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, heading);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\subsection*{"));
    }

    #[test]
    fn test_render_link() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let link = arena.alloc(Node::with_data(
            NodeType::Link,
            NodeData::Link {
                url: "https://example.com".to_string(),
                title: "".to_string(),
            },
        ));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "link".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\href{https://example.com}{link}"));
    }

    #[test]
    fn test_render_blockquote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let blockquote = arena.alloc(Node::new(NodeType::BlockQuote));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Quote".to_string(),
            },
        ));

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
        let root = arena.alloc(Node::new(NodeType::Document));
        let code_block = arena.alloc(Node::with_data(
            NodeType::CodeBlock,
            NodeData::CodeBlock {
                info: "".to_string(),
                literal: "fn main() {}".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, code_block);

        let latex = render(&arena, root, 0);
        assert!(latex.contains("\\begin{verbatim}"));
        assert!(latex.contains("fn main() {}"));
        assert!(latex.contains("\\end{verbatim}"));
    }

    #[test]
    fn test_render_bullet_list() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let list = arena.alloc(Node::with_data(
            NodeType::List,
            NodeData::List {
                list_type: ListType::Bullet,
                delim: crate::node::DelimType::None,
                start: 0,
                tight: true,
                bullet_char: '-',
            },
        ));
        let item = arena.alloc(Node::new(NodeType::Item));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Item".to_string(),
            },
        ));

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
