use clmd::unescape_string;

/// Test logging macro - only prints when VERBOSE_TESTS is set
macro_rules! test_log {
    ($($arg:tt)*) => {
        if std::env::var("VERBOSE_TESTS").is_ok() {
            std::println!($($arg)*);
        }
    };
}

#[test]
fn test_unescape_plus() {
    // In the test, the input is "foo\+bar" (with a literal backslash)
    let input = "foo\\+bar";
    test_log!("Input bytes: {:?}", input.as_bytes());
    let result = unescape_string(input);
    test_log!("Result: {:?}", result);
    test_log!("Result bytes: {:?}", result.as_bytes());

    // The \+ should become +
    assert_eq!(result, "foo+bar");
}
