use md::{options, parse_document, render_html};
use md::iterator::NodeWalker;

#[test]
fn debug_indented_code_block() {
    // Test case 1: Simple indented code block
    let input = "    foo\n";
    println!("Input: {:?}", input);
    println!("Input bytes: {:?}", input.as_bytes());

    let doc = parse_document(input, options::DEFAULT);

    // Walk the tree to see structure
    let mut walker = NodeWalker::new(doc.clone());
    println!("\nAST Structure:");
    while let Some(event) = walker.next() {
        let node = event.node.borrow();
        let indent = if event.entering { "Enter: " } else { "Exit:  " };
        let content = match &node.data {
            md::node::NodeData::Text { literal } => format!(" (content: {:?})", literal),
            md::node::NodeData::CodeBlock { literal, info } => format!(" (code: {:?}, info: {:?})", literal, info),
            _ => String::new(),
        };
        println!("{}{:?}{}", indent, node.node_type, content);
    }

    let result = render_html(&doc, options::DEFAULT);
    println!("\nHTML Result:\n{}", result);
}

#[test]
fn debug_simple_paragraph() {
    let input = "Hello world\n";
    println!("\n=== Simple Paragraph ===");
    println!("Input: {:?}", input);

    let doc = parse_document(input, options::DEFAULT);

    let mut walker = NodeWalker::new(doc.clone());
    println!("AST Structure:");
    while let Some(event) = walker.next() {
        let node = event.node.borrow();
        let indent = if event.entering { "Enter: " } else { "Exit:  " };
        let content = match &node.data {
            md::node::NodeData::Text { literal } => format!(" (content: {:?})", literal),
            _ => String::new(),
        };
        println!("{}{:?}{}", indent, node.node_type, content);
    }

    let result = render_html(&doc, options::DEFAULT);
    println!("HTML Result:\n{}", result);
}

#[test]
fn debug_tabs() {
    // Test case with tabs
    let input = "foo\tbar\tbaz\n";
    println!("\n=== Tabs Test ===");
    println!("Input: {:?}", input);
    println!("Input bytes: {:?}", input.as_bytes());

    let doc = parse_document(input, options::DEFAULT);

    let mut walker = NodeWalker::new(doc.clone());
    println!("AST Structure:");
    while let Some(event) = walker.next() {
        let node = event.node.borrow();
        let indent = if event.entering { "Enter: " } else { "Exit:  " };
        let content = match &node.data {
            md::node::NodeData::Text { literal } => format!(" (content: {:?})", literal),
            _ => String::new(),
        };
        println!("{}{:?}{}", indent, node.node_type, content);
    }

    let result = render_html(&doc, options::DEFAULT);
    println!("HTML Result:\n{}", result);
    println!("HTML Result bytes: {:?}", result.as_bytes());

    let expected = "<p>foo\tbar\tbaz</p>";
    println!("Expected:\n{}", expected);
    println!("Expected bytes: {:?}", expected.as_bytes());

    if result == expected {
        println!("MATCH!");
    } else {
        println!("DIFFERENT!");
        // Find first difference
        for (i, (a, b)) in result.bytes().zip(expected.bytes()).enumerate() {
            if a != b {
                println!("First difference at byte {}: got {:?}, expected {:?}", i, a, b);
                break;
            }
        }
    }
}

#[test]
fn debug_fenced_code_block() {
    let input = "```\nfoo\n```\n";
    println!("\n=== Fenced Code Block ===");
    println!("Input: {:?}", input);

    let doc = parse_document(input, options::DEFAULT);

    let mut walker = NodeWalker::new(doc.clone());
    println!("AST Structure:");
    while let Some(event) = walker.next() {
        let node = event.node.borrow();
        let indent = if event.entering { "Enter: " } else { "Exit:  " };
        let content = match &node.data {
            md::node::NodeData::CodeBlock { literal, info } => format!(" (code: {:?}, info: {:?})", literal, info),
            _ => String::new(),
        };
        println!("{}{:?}{}", indent, node.node_type, content);
    }

    let result = render_html(&doc, options::DEFAULT);
    println!("HTML Result:\n{}", result);
}

#[test]
fn debug_code_block_with_multiple_lines() {
    let input = "    foo\n    bar\n";
    println!("\n=== Indented Code Block with Multiple Lines ===");
    println!("Input: {:?}", input);

    let doc = parse_document(input, options::DEFAULT);

    let mut walker = NodeWalker::new(doc.clone());
    println!("AST Structure:");
    while let Some(event) = walker.next() {
        let node = event.node.borrow();
        let indent = if event.entering { "Enter: " } else { "Exit:  " };
        let content = match &node.data {
            md::node::NodeData::CodeBlock { literal, info } => format!(" (code: {:?}, info: {:?})", literal, info),
            _ => String::new(),
        };
        println!("{}{:?}{}", indent, node.node_type, content);
    }

    let result = render_html(&doc, options::DEFAULT);
    println!("HTML Result:\n{}", result);
}

#[test]
fn debug_loose_list() {
    // Test case #4 from CommonMark spec
    let input = "  - foo\n\n\tbar\n";
    println!("\n=== Loose List Test (Spec #4) ===");
    println!("Input: {:?}", input);
    println!("Input bytes: {:?}", input.as_bytes());

    let doc = parse_document(input, options::DEFAULT);

    let mut walker = NodeWalker::new(doc.clone());
    println!("AST Structure:");
    while let Some(event) = walker.next() {
        let node = event.node.borrow();
        let indent = if event.entering { "Enter: " } else { "Exit:  " };
        let content = match &node.data {
            md::node::NodeData::Text { literal } => format!(" (text: {:?})", literal),
            md::node::NodeData::List { list_type, tight, .. } => format!(" (type: {:?}, tight: {:?})", list_type, tight),
            _ => String::new(),
        };
        let source_pos = format!(" [lines {}-{}]", node.source_pos.start_line, node.source_pos.end_line);
        println!("{}{:?}{}{}", indent, node.node_type, content, source_pos);
    }

    let result = render_html(&doc, options::DEFAULT);
    println!("\nHTML Result:\n{}", result);
    println!("HTML Result escaped: {:?}", result);

    let expected = "<ul>\n<li>\n<p>foo</p>\n<p>bar</p>\n</li>\n</ul>";
    println!("\nExpected:\n{}", expected);
    println!("Expected escaped: {:?}", expected);

    if result == expected {
        println!("\nMATCH!");
    } else {
        println!("\nDIFFERENT!");
    }
}

#[test]
fn debug_tight_list() {
    // Test case: tight list (no blank lines)
    let input = "- foo\n- bar\n";
    println!("\n=== Tight List Test ===");
    println!("Input: {:?}", input);

    let doc = parse_document(input, options::DEFAULT);

    let mut walker = NodeWalker::new(doc.clone());
    println!("AST Structure:");
    while let Some(event) = walker.next() {
        let node = event.node.borrow();
        let indent = if event.entering { "Enter: " } else { "Exit:  " };
        let content = match &node.data {
            md::node::NodeData::Text { literal } => format!(" (text: {:?})", literal),
            md::node::NodeData::List { list_type, tight, .. } => format!(" (type: {:?}, tight: {:?})", list_type, tight),
            _ => String::new(),
        };
        let source_pos = format!(" [lines {}-{}]", node.source_pos.start_line, node.source_pos.end_line);
        println!("{}{:?}{}{}", indent, node.node_type, content, source_pos);
    }

    let result = render_html(&doc, options::DEFAULT);
    println!("\nHTML Result:\n{}", result);
    println!("HTML Result escaped: {:?}", result);

    let expected = "<ul>\n<li>foo</li>\n<li>bar</li>\n</ul>";
    println!("\nExpected:\n{}", expected);
    println!("Expected escaped: {:?}", expected);

    if result == expected {
        println!("\nMATCH!");
    } else {
        println!("\nDIFFERENT!");
    }
}
