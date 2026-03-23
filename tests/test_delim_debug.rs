// Debug test for delimiter handling
use md::node::{Node, NodeType, NodeData};
use md::inlines::parse_inlines;
use std::rc::Rc;
use std::cell::RefCell;

fn print_tree(node: &Rc<RefCell<Node>>, indent: usize) {
    let node_ref = node.borrow();
    let indent_str = "  ".repeat(indent);
    
    match &node_ref.data {
        NodeData::Text { literal } => {
            println!("{}Text: '{}'", indent_str, literal);
        }
        NodeData::Emph => {
            println!("{}Emph:", indent_str);
        }
        NodeData::Strong => {
            println!("{}Strong:", indent_str);
        }
        _ => {
            println!("{}Other: {:?}", indent_str, node_ref.node_type);
        }
    }
    
    let first_child = node_ref.first_child.borrow().clone();
    if let Some(child) = first_child {
        print_tree(&child, indent + 1);
    }
    
    let next = node_ref.next.borrow().clone();
    if let Some(next_node) = next {
        print_tree(&next_node, indent);
    }
}

#[test]
fn test_emphasis_tree() {
    let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
    parse_inlines(&parent, "*foo bar*", 1, 0);
    
    println!("\n=== AST Tree for '*foo bar*' ===");
    print_tree(&parent, 0);
}
