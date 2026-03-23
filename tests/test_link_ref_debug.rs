use md::inlines::parse_reference;
use std::collections::HashMap;

#[test]
fn test_parse_reference_debug() {
    let mut refmap = HashMap::new();

    // Test 1: Basic reference
    let input1 = "[foo]: /bar\n";
    let consumed1 = parse_reference(input1, &mut refmap);
    println!("Input 1: {:?}", input1);
    println!("Consumed 1: {}", consumed1);
    println!("Refmap 1: {:?}", refmap);

    // Test 2: Reference with title
    refmap.clear();
    let input2 = "[foo]: /bar \"title\"\n";
    let consumed2 = parse_reference(input2, &mut refmap);
    println!("\nInput 2: {:?}", input2);
    println!("Consumed 2: {}", consumed2);
    println!("Refmap 2: {:?}", refmap);

    // Test 3: Reference without newline
    refmap.clear();
    let input3 = "[foo]: /bar";
    let consumed3 = parse_reference(input3, &mut refmap);
    println!("\nInput 3: {:?}", input3);
    println!("Consumed 3: {}", consumed3);
    println!("Refmap 3: {:?}", refmap);

    // Test 4: Just the label
    refmap.clear();
    let input4 = "[foo]";
    let consumed4 = parse_reference(input4, &mut refmap);
    println!("\nInput 4: {:?}", input4);
    println!("Consumed 4: {}", consumed4);
    println!("Refmap 4: {:?}", refmap);
}
