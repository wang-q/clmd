//! Arena-based Performance Test
//!
//! This example tests the performance of the Arena-based implementation.

use std::time::Instant;

fn main() {
    println!("Arena-based Performance Test\n");
    println!("=============================\n");

    // Test input - simple markdown
    let input = r#"# Heading 1

This is a paragraph with some text.

## Heading 2

- List item 1
- List item 2
- List item 3

> This is a blockquote.
> It has multiple lines.

```rust
fn main() {
    println!("Hello, world!");
}
```

Another paragraph with `inline code` and a [link](https://example.com).
"#;

    let iterations = 1000;

    // Test Arena version
    println!("Testing Arena version...");
    let start = Instant::now();
    for _ in 0..iterations {
        let html = clmd::markdown_to_html(input, 0);
        // Prevent optimization
        std::hint::black_box(html);
    }
    let arena_time = start.elapsed();
    println!(
        "  Time: {:?} ({:?} per iteration)",
        arena_time,
        arena_time / iterations
    );

    // Output sample
    println!("\n---------------------------------------------");
    println!("Sample output:");
    let html = clmd::markdown_to_html(input, 0);
    println!("{}", &html[..html.len().min(500)]);
    if html.len() > 500 {
        println!("... (truncated)");
    }
}
