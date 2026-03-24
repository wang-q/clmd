use crate::iterator::{NodeWalker, WalkerEvent};
use crate::node::{ListType, Node, NodeData, NodeType};
use std::cell::RefCell;
use std::rc::Rc;

/// Render a node tree as LaTeX
pub fn render(root: &Rc<RefCell<Node>>, options: u32) -> String {
    let mut renderer = LatexRenderer::new(options);
    renderer.render(root)
}

/// LaTeX renderer state
struct LatexRenderer {
    #[allow(dead_code)]
    options: u32,
    output: String,
    beginning_of_line: bool,
    list_stack: Vec<ListType>,
    need_blank_line: bool,
}

impl LatexRenderer {
    fn new(options: u32) -> Self {
        LatexRenderer {
            options,
            output: String::new(),
            beginning_of_line: true,
            list_stack: Vec::new(),
            need_blank_line: false,
        }
    }

    fn render(&mut self, root: &Rc<RefCell<Node>>) -> String {
        let mut walker = NodeWalker::new(root.clone());

        while let Some(event) = walker.next() {
            if event.entering {
                self.enter_node(&event);
            } else {
                self.exit_node(&event);
            }
        }

        while self.output.ends_with('\n') || self.output.ends_with(' ') {
            self.output.pop();
        }
        self.output.push('\n');

        self.output.clone()
    }

    fn enter_node(&mut self, event: &WalkerEvent) {
        let node = event.node.borrow();

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
                self.render_code_block(&node);
                self.need_blank_line = true;
            }
            NodeType::HtmlBlock => {}
            NodeType::Paragraph => {}
            NodeType::Heading => {
                self.render_heading(&node);
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

    fn exit_node(&mut self, event: &WalkerEvent) {
        let node = event.node.borrow();

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

    fn render_code_block(&mut self, node: &std::cell::Ref<Node>) {
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

    fn render_heading(&mut self, node: &std::cell::Ref<Node>) {
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
    use crate::node::{append_child, Node, NodeData, NodeType};

    #[test]
    fn test_render_paragraph() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello world".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, text.clone());

        let latex = render(&root, 0);
        assert!(latex.contains("Hello world"));
    }

    #[test]
    fn test_render_emph() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let emph = Rc::new(RefCell::new(Node::new(NodeType::Emph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "emphasized".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, emph.clone());
        append_child(&emph, text.clone());

        let latex = render(&root, 0);
        assert!(latex.contains("\\emph{emphasized}"));
    }

    #[test]
    fn test_render_strong() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let strong = Rc::new(RefCell::new(Node::new(NodeType::Strong)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "strong".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, strong.clone());
        append_child(&strong, text.clone());

        let latex = render(&root, 0);
        assert!(latex.contains("\\textbf{strong}"));
    }

    #[test]
    fn test_render_code() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let code = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Code,
            NodeData::Code {
                literal: "code".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, code.clone());

        let latex = render(&root, 0);
        assert!(latex.contains("\\texttt{code}"));
    }

    #[test]
    fn test_render_heading() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let heading = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 2,
                content: "Heading".to_string(),
            },
        )));

        append_child(&root, heading.clone());

        let latex = render(&root, 0);
        assert!(latex.contains("\\subsection*{"));
    }

    #[test]
    fn test_render_link() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let link = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Link,
            NodeData::Link {
                url: "https://example.com".to_string(),
                title: "".to_string(),
            },
        )));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "link".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, link.clone());
        append_child(&link, text.clone());

        let latex = render(&root, 0);
        assert!(latex.contains("\\href{https://example.com}{link}"));
    }

    #[test]
    fn test_render_blockquote() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let blockquote = Rc::new(RefCell::new(Node::new(NodeType::BlockQuote)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Quote".to_string(),
            },
        )));

        append_child(&root, blockquote.clone());
        append_child(&blockquote, para.clone());
        append_child(&para, text.clone());

        let latex = render(&root, 0);
        assert!(latex.contains("\\begin{quote}"));
        assert!(latex.contains("\\end{quote}"));
    }

    #[test]
    fn test_render_code_block() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let code_block = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::CodeBlock,
            NodeData::CodeBlock {
                info: "".to_string(),
                literal: "fn main() {}".to_string(),
            },
        )));

        append_child(&root, code_block.clone());

        let latex = render(&root, 0);
        assert!(latex.contains("\\begin{verbatim}"));
        assert!(latex.contains("fn main() {}"));
        assert!(latex.contains("\\end{verbatim}"));
    }

    #[test]
    fn test_render_bullet_list() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let list = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::List,
            NodeData::List {
                list_type: ListType::Bullet,
                delim: crate::node::DelimType::None,
                start: 0,
                tight: true,
                bullet_char: '-',
            },
        )));
        let item = Rc::new(RefCell::new(Node::new(NodeType::Item)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Item".to_string(),
            },
        )));

        append_child(&root, list.clone());
        append_child(&list, item.clone());
        append_child(&item, para.clone());
        append_child(&para, text.clone());

        let latex = render(&root, 0);
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
