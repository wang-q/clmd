use md::parser::Parser;
use md::options;

#[test]
fn test_link_reference_integration() {
    let parser = Parser::new(options::DEFAULT);
    let doc = parser.parse("[foo]\n\n[foo]: /bar\n");

    // Print the document structure
    println!("Document structure:");
    print_node(&doc, 0);
}

fn print_node(node: &std::rc::Rc<std::cell::RefCell<md::node::Node>>, depth: usize) {
    let indent = "  ".repeat(depth);
    let node_ref = node.borrow();
    println!("{}{:?}", indent, node_ref.node_type);

    // Print node data if it's a text node
    use md::node::NodeData;
    match &node_ref.data {
        NodeData::Text { literal } => {
            println!("{}  literal: {:?}", indent, literal);
        }
        NodeData::Link { url, title } => {
            println!("{}  url: {:?}, title: {:?}", indent, url, title);
        }
        _ => {}
    }

    // Print children
    let mut child_opt = node_ref.first_child.borrow().clone();
    while let Some(child) = child_opt {
        print_node(&child, depth + 1);
        child_opt = child.borrow().next.borrow().clone();
    }
}
