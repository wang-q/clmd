// Test delimiter scanning directly
use md::inlines::Subject;

#[test]
fn test_scan_delims() {
    let mut subject = Subject::new("*foo bar*", 1, 0);
    
    // First character should be *
    let c = subject.peek().unwrap();
    println!("First char: '{}'", c);
    
    // Call handle_delim
    // We need a parent node to test this properly
}
