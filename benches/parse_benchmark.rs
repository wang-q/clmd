//! Benchmarks for parsing performance

use clmd::Document;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn small_document() -> &'static str {
    "# Hello World\n\nThis is a **small** document with *some* formatting.\n\n- Item 1\n- Item 2\n- Item 3\n\n> A blockquote\n> with multiple lines\n"
}

fn medium_document() -> &'static str {
    r#"# Introduction

This is a medium-sized document with various Markdown features.

## Section 1: Text Formatting

You can write text in **bold**, *italic*, or ***both***.
You can also use ~~strikethrough~~ and `inline code`.

## Section 2: Lists

### Unordered Lists

- First item
- Second item
  - Nested item 1
  - Nested item 2
- Third item

### Ordered Lists

1. First step
2. Second step
3. Third step

## Section 3: Code Blocks

Here's a code block:

```rust
fn main() {
    println!("Hello, world!");
}
```

## Section 4: Links and Images

Visit [example.com](https://example.com) for more information.

![Alt text](image.png)

## Section 5: Blockquotes

> This is a blockquote.
> It can span multiple lines.
>
> > And can be nested too!

## Section 6: Tables

| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |

## Conclusion

That's all for this medium document!
"#
}

fn large_document() -> String {
    // Generate a large document by repeating the medium document
    medium_document().repeat(10)
}

fn benchmark_small_document(c: &mut Criterion) {
    c.bench_function("parse_small_document", |b| {
        b.iter(|| {
            let doc = Document::parse(black_box(small_document())).unwrap();
            black_box(doc.to_html());
        });
    });
}

fn benchmark_medium_document(c: &mut Criterion) {
    c.bench_function("parse_medium_document", |b| {
        b.iter(|| {
            let doc = Document::parse(black_box(medium_document())).unwrap();
            black_box(doc.to_html());
        });
    });
}

fn benchmark_large_document(c: &mut Criterion) {
    let large = large_document();
    c.bench_function("parse_large_document", |b| {
        b.iter(|| {
            let doc = Document::parse(black_box(&large)).unwrap();
            black_box(doc.to_html());
        });
    });
}

fn benchmark_nested_lists(c: &mut Criterion) {
    let nested = "- Item\n".repeat(100);
    c.bench_function("parse_nested_lists", |b| {
        b.iter(|| {
            let doc = Document::parse(black_box(&nested)).unwrap();
            black_box(doc.to_html());
        });
    });
}

fn benchmark_code_blocks(c: &mut Criterion) {
    let code = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n".repeat(50);
    c.bench_function("parse_code_blocks", |b| {
        b.iter(|| {
            let doc = Document::parse(black_box(&code)).unwrap();
            black_box(doc.to_html());
        });
    });
}

fn benchmark_tables(c: &mut Criterion) {
    let table =
        "| H1 | H2 | H3 |\n|---|---|---|\n| A | B | C |\n| D | E | F |\n".repeat(20);
    c.bench_function("parse_tables", |b| {
        b.iter(|| {
            let doc = Document::parse(black_box(&table)).unwrap();
            black_box(doc.to_html());
        });
    });
}

criterion_group!(
    benches,
    benchmark_small_document,
    benchmark_medium_document,
    benchmark_large_document,
    benchmark_nested_lists,
    benchmark_code_blocks,
    benchmark_tables
);
criterion_main!(benches);
