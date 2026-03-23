use md::blocks::BlockParser;
use md::inlines::parse_inlines_with_refmap;
use md::iterator::NodeWalker;
use md::node::{Node, NodeData, NodeType};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_link_reference_debug3() {
    let mut parser = BlockParser::new();

    let input = "[foo]\n\n[foo]: /bar\n";
    let lines: Vec<&str> = input.lines().collect();

    for line in &lines {
        parser.process_line(line);
    }
    parser.process_line(""); // Finalize
    parser.finalize_document();

    let doc = parser.doc.clone();
    let refmap = parser.refmap.clone();

    println!("Refmap before inline parsing: {:?}", refmap);

    // Process inlines manually with debug output
    let mut walker = NodeWalker::new(doc.clone());

    while let Some(event) = walker.next() {
        if event.entering {
            let node_type;
            let literal_opt;
            {
                let node = event.node.borrow();
                node_type = node.node_type;
                literal_opt = match &node.data {
                    NodeData::Text { literal } if !literal.is_empty() => Some(literal.clone()),
                    _ => None,
                };
            }
            match node_type {
                NodeType::Paragraph => {
                    if let Some(literal) = literal_opt {
                        println!("Processing paragraph with content: {:?}", literal);
                        println!("Refmap: {:?}", refmap);
                        parse_inlines_with_refmap(
                            &event.node.clone(),
                            &literal,
                            1,
                            0,
                            refmap.clone(),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    // Print final document structure
    println!("\nFinal document structure:");
    print_tree(&doc, 0);
}

fn print_tree(node: &Rc<RefCell<Node>>, depth: usize) {
    let indent = "  ".repeat(depth);
    let node_ref = node.borrow();
    println!("{}{:?}", indent, node_ref.node_type);

    use md::node::NodeData;
    match &node_ref.data {
        NodeData::Text { literal } => {
            if !literal.is_empty() {
                println!("{}  literal: {:?}", indent, literal);
            }
        }
        NodeData::Link { url, title } => {
            println!("{}  url: {:?}, title: {:?}", indent, url, title);
        }
        _ => {}
    }

    let mut child_opt = node_ref.first_child.borrow().clone();
    while let Some(child) = child_opt {
        print_tree(&child, depth + 1);
        child_opt = child.borrow().next.borrow().clone();
    }
}
