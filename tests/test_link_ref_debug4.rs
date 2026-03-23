use md::inlines::{parse_inlines_with_refmap, Subject};
use md::node::{Node, NodeData, NodeType};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_link_parsing_debug() {
    let mut refmap = HashMap::new();
    refmap.insert("FOO".to_string(), ("/bar".to_string(), "".to_string()));

    let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));

    println!("Parsing: '[foo]' with refmap: {:?}", refmap);
    parse_inlines_with_refmap(&parent, "[foo]", 1, 0, refmap.clone());

    // Print the result
    println!("\nResult:");
    print_tree(&parent, 0);
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
