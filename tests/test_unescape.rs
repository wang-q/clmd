use md::inlines::unescape_string;

#[test]
fn test_unescape_plus() {
    // In the test, the input is "foo\+bar" (with a literal backslash)
    let input = "foo\\+bar";
    println!("Input bytes: {:?}", input.as_bytes());
    let result = unescape_string(input);
    println!("Result: {:?}", result);
    println!("Result bytes: {:?}", result.as_bytes());
    
    // The \+ should become +
    assert_eq!(result, "foo+bar");
}
