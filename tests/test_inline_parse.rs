// Test inline parsing directly
use md::node::{Node, NodeType, NodeData};
use md::inlines::parse_inlines;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_inline_parse_emphasis() {
    let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
    parse_inlines(&parent, "*foo bar*", 1, 0);
    
    // Check the structure
    let parent_ref = parent.borrow();
    let first_child = parent_ref.first_child.borrow().clone();
    
    if let Some(child) = first_child {
        let child_ref = child.borrow();
        println!("First child type: {:?}", child_ref.node_type);
        
        match &child_ref.data {
            NodeData::Text { literal } => {
                println!("First child literal: '{}'", literal);
            }
            _ => {
                println!("First child is not text");
            }
        }
    } else {
        println!("No children!");
    }
}
