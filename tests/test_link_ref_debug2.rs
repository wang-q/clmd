use md::blocks::BlockParser;

#[test]
fn test_link_reference_debug() {
    let mut parser = BlockParser::new();

    let input = "[foo]\n\n[foo]: /bar\n";
    let lines: Vec<&str> = input.lines().collect();

    for line in &lines {
        println!("Processing line: {:?}", line);
        parser.process_line(line);
    }
    parser.process_line(""); // Finalize
    parser.finalize_document();

    println!("\nRefmap: {:?}", parser.refmap);

    // Print document structure
    println!("\nDocument structure:");
    print_tree(&parser.doc, 0);
}

fn print_tree(node: &std::rc::Rc<std::cell::RefCell<md::node::Node>>, depth: usize) {
    let indent = "  ".repeat(depth);
    let node_ref = node.borrow();
    println!("{}{:?}", indent, node_ref.node_type);

    use md::node::NodeData;
    match &node_ref.data {
        NodeData::Text { literal } => {
            println!("{}  literal: {:?}", indent, literal);
        }
        _ => {}
    }

    let mut child_opt = node_ref.first_child.borrow().clone();
    while let Some(child) = child_opt {
        print_tree(&child, depth + 1);
        child_opt = child.borrow().next.borrow().clone();
    }
}
