//! Arena vs Rc<RefCell> Performance Comparison
//!
//! This example compares the performance of the two implementations.

use std::time::Instant;

fn main() {
    println!("Arena vs Rc<RefCell> Performance Comparison\n");
    println!("=============================================\n");

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

    // Test Rc<RefCell> version
    println!("Testing Rc<RefCell> version...");
    let start = Instant::now();
    for _ in 0..iterations {
        let html = clmd::markdown_to_html(input, clmd::options::DEFAULT);
        // Prevent optimization
        std::hint::black_box(html);
    }
    let rc_time = start.elapsed();
    println!("  Time: {:?} ({:?} per iteration)", rc_time, rc_time / iterations);

    // Test Arena version
    println!("\nTesting Arena version...");
    let start = Instant::now();
    for _ in 0..iterations {
        let html = clmd::arena_api::markdown_to_html(input, 0);
        // Prevent optimization
        std::hint::black_box(html);
    }
    let arena_time = start.elapsed();
    println!("  Time: {:?} ({:?} per iteration)", arena_time, arena_time / iterations);

    // Compare
    println!("\n---------------------------------------------");
    if arena_time < rc_time {
        let speedup = rc_time.as_secs_f64() / arena_time.as_secs_f64();
        let improvement = ((rc_time.as_secs_f64() - arena_time.as_secs_f64()) / rc_time.as_secs_f64()) * 100.0;
        println!("Arena is {:.2}x faster ({:.1}% improvement)", speedup, improvement);
    } else {
        let slowdown = arena_time.as_secs_f64() / rc_time.as_secs_f64();
        let regression = ((arena_time.as_secs_f64() - rc_time.as_secs_f64()) / rc_time.as_secs_f64()) * 100.0;
        println!("Arena is {:.2}x slower ({:.1}% regression)", slowdown, regression);
    }

    // Output sample
    println!("\n---------------------------------------------");
    println!("Sample output (Arena version):");
    let html = clmd::arena_api::markdown_to_html(input, 0);
    println!("{}", &html[..html.len().min(500)]);
    if html.len() > 500 {
        println!("... (truncated)");
    }
}
