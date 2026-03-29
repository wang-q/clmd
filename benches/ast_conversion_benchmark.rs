//! Benchmark for AST conversion overhead
//!
//! This benchmark measures the performance impact of converting from
//! NodeArena to arena_tree::Node format.

use clmd::{parse_document, Arena, Options};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Small document benchmark
fn bench_small(c: &mut Criterion) {
    let input = "# Hello\n\nWorld";
    let options = Options::default();

    c.bench_function("parse_small", |b| {
        b.iter(|| {
            let arena = Arena::new();
            let _root = parse_document(&arena, black_box(input), &options);
        });
    });
}

/// Medium document benchmark
fn bench_medium(c: &mut Criterion) {
    let input = r#"# Heading 1

This is a paragraph with some **bold** and *italic* text.

## Heading 2

- List item 1
- List item 2
- List item 3

### Heading 3

```rust
fn main() {
    println!("Hello, world!");
}
```

[Link](https://example.com)
"#;
    let options = Options::default();

    c.bench_function("parse_medium", |b| {
        b.iter(|| {
            let arena = Arena::new();
            let _root = parse_document(&arena, black_box(input), &options);
        });
    });
}

/// Large document benchmark
fn bench_large(c: &mut Criterion) {
    let input = include_str!("samples/lorem-large.md");
    let options = Options::default();

    c.bench_function("parse_large", |b| {
        b.iter(|| {
            let arena = Arena::new();
            let _root = parse_document(&arena, black_box(input), &options);
        });
    });
}

/// Benchmark with different options
fn bench_with_options(c: &mut Criterion) {
    let input = r#"# Test

This is **bold** and *italic*.

- Item 1
- Item 2

[link](http://example.com)
"#;

    let mut group = c.benchmark_group("parse_options");

    // Default options
    let options = Options::default();
    group.bench_with_input(
        BenchmarkId::new("options", "default"),
        &options,
        |b, opts| {
            b.iter(|| {
                let arena = Arena::new();
                let _root = parse_document(&arena, black_box(input), opts);
            });
        },
    );

    // With source positions
    let mut options = Options::default();
    options.parse.sourcepos = true;
    group.bench_with_input(
        BenchmarkId::new("options", "sourcepos"),
        &options,
        |b, opts| {
            b.iter(|| {
                let arena = Arena::new();
                let _root = parse_document(&arena, black_box(input), opts);
            });
        },
    );

    // With smart punctuation
    let mut options = Options::default();
    options.parse.smart = true;
    group.bench_with_input(BenchmarkId::new("options", "smart"), &options, |b, opts| {
        b.iter(|| {
            let arena = Arena::new();
            let _root = parse_document(&arena, black_box(input), opts);
        });
    });

    group.finish();
}

/// Pathological cases
fn bench_pathological(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological");

    // Deeply nested emphasis
    let deep_emphasis = "*".repeat(1000) + "x" + &"*".repeat(1000);
    let options = Options::default();
    group.bench_function("deep_emphasis", |b| {
        b.iter(|| {
            let arena = Arena::new();
            let _root = parse_document(&arena, black_box(&deep_emphasis), &options);
        });
    });

    // Many links
    let many_links: String = (0..100)
        .map(|i| format!("[link{}](http://example.com/{}) ", i, i))
        .collect();
    group.bench_function("many_links", |b| {
        b.iter(|| {
            let arena = Arena::new();
            let _root = parse_document(&arena, black_box(&many_links), &options);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_small,
    bench_medium,
    bench_large,
    bench_with_options,
    bench_pathological
);
criterion_main!(benches);
