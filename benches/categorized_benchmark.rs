//! Categorized Benchmark Tests
//!
//! Benchmarks based on cmark's sample files, organized by category.

use clmd::{markdown_to_html_with_options, Options};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Block-level benchmarks
fn bench_block_quotes_flat(c: &mut Criterion) {
    let input = include_str!("samples/block-bq-flat.md");
    let options = Options::default();
    c.bench_function("block_quotes_flat", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_block_quotes_nested(c: &mut Criterion) {
    let input = include_str!("samples/block-bq-nested.md");
    let options = Options::default();
    c.bench_function("block_quotes_nested", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_block_code(c: &mut Criterion) {
    let input = include_str!("samples/block-code.md");
    let options = Options::default();
    c.bench_function("block_code", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_block_fences(c: &mut Criterion) {
    let input = include_str!("samples/block-fences.md");
    let options = Options::default();
    c.bench_function("block_fences", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_block_heading(c: &mut Criterion) {
    let input = include_str!("samples/block-heading.md");
    let options = Options::default();
    c.bench_function("block_heading", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_block_hr(c: &mut Criterion) {
    let input = include_str!("samples/block-hr.md");
    let options = Options::default();
    c.bench_function("block_hr", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_block_list_flat(c: &mut Criterion) {
    let input = include_str!("samples/block-list-flat.md");
    let options = Options::default();
    c.bench_function("block_list_flat", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_block_list_nested(c: &mut Criterion) {
    let input = include_str!("samples/block-list-nested.md");
    let options = Options::default();
    c.bench_function("block_list_nested", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_block_html(c: &mut Criterion) {
    let input = include_str!("samples/block-html.md");
    let options = Options::default();
    c.bench_function("block_html", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_block_lheading(c: &mut Criterion) {
    let input = include_str!("samples/block-lheading.md");
    let options = Options::default();
    c.bench_function("block_lheading", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_block_ref_flat(c: &mut Criterion) {
    let input = include_str!("samples/block-ref-flat.md");
    let options = Options::default();
    c.bench_function("block_ref_flat", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_block_ref_nested(c: &mut Criterion) {
    let input = include_str!("samples/block-ref-nested.md");
    let options = Options::default();
    c.bench_function("block_ref_nested", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

// Inline benchmarks
fn bench_inline_autolink(c: &mut Criterion) {
    let input = include_str!("samples/inline-autolink.md");
    let options = Options::default();
    c.bench_function("inline_autolink", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_inline_backticks(c: &mut Criterion) {
    let input = include_str!("samples/inline-backticks.md");
    let options = Options::default();
    c.bench_function("inline_backticks", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_inline_em_flat(c: &mut Criterion) {
    let input = include_str!("samples/inline-em-flat.md");
    let options = Options::default();
    c.bench_function("inline_em_flat", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_inline_em_nested(c: &mut Criterion) {
    let input = include_str!("samples/inline-em-nested.md");
    let options = Options::default();
    c.bench_function("inline_em_nested", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_inline_em_worst(c: &mut Criterion) {
    let input = include_str!("samples/inline-em-worst.md");
    let options = Options::default();
    c.bench_function("inline_em_worst", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_inline_entity(c: &mut Criterion) {
    let input = include_str!("samples/inline-entity.md");
    let options = Options::default();
    c.bench_function("inline_entity", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_inline_escape(c: &mut Criterion) {
    let input = include_str!("samples/inline-escape.md");
    let options = Options::default();
    c.bench_function("inline_escape", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_inline_html(c: &mut Criterion) {
    let input = include_str!("samples/inline-html.md");
    let options = Options::default();
    c.bench_function("inline_html", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_inline_links_flat(c: &mut Criterion) {
    let input = include_str!("samples/inline-links-flat.md");
    let options = Options::default();
    c.bench_function("inline_links_flat", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_inline_links_nested(c: &mut Criterion) {
    let input = include_str!("samples/inline-links-nested.md");
    let options = Options::default();
    c.bench_function("inline_links_nested", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_inline_newlines(c: &mut Criterion) {
    let input = include_str!("samples/inline-newlines.md");
    let options = Options::default();
    c.bench_function("inline_newlines", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

// Full document benchmark
fn bench_lorem1(c: &mut Criterion) {
    let input = include_str!("samples/lorem1.md");
    let options = Options::default();
    c.bench_function("lorem1_full_document", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

// Large document benchmarks
fn bench_lorem_large(c: &mut Criterion) {
    let input = include_str!("samples/lorem-large.md");
    let options = Options::default();
    c.bench_function("lorem_large_7kb", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_lorem_xlarge(c: &mut Criterion) {
    let input = include_str!("samples/lorem-xlarge.md");
    let options = Options::default();
    c.bench_function("lorem_xlarge_110kb", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

// Group benchmarks
criterion_group!(
    block_benchmarks,
    bench_block_quotes_flat,
    bench_block_quotes_nested,
    bench_block_code,
    bench_block_fences,
    bench_block_heading,
    bench_block_hr,
    bench_block_list_flat,
    bench_block_list_nested,
    bench_block_html,
    bench_block_lheading,
    bench_block_ref_flat,
    bench_block_ref_nested,
);

criterion_group!(
    inline_benchmarks,
    bench_inline_autolink,
    bench_inline_backticks,
    bench_inline_em_flat,
    bench_inline_em_nested,
    bench_inline_em_worst,
    bench_inline_entity,
    bench_inline_escape,
    bench_inline_html,
    bench_inline_links_flat,
    bench_inline_links_nested,
    bench_inline_newlines,
);

criterion_group!(
    full_document_benchmarks,
    bench_lorem1,
    bench_lorem_large,
    bench_lorem_xlarge,
);

// Synthetic benchmarks (from parse_benchmark.rs)
fn bench_small_document(c: &mut Criterion) {
    let input = "# Hello World\n\nThis is a **small** document with *some* formatting.\n\n- Item 1\n- Item 2\n- Item 3\n\n> A blockquote\n> with multiple lines\n";
    let options = Options::default();
    c.bench_function("synthetic_small_document", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_medium_document(c: &mut Criterion) {
    let input = r#"# Introduction

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
"#;
    let options = Options::default();
    c.bench_function("synthetic_medium_document", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(input), &options))
    });
}

fn bench_large_document(c: &mut Criterion) {
    let medium = r#"# Introduction

This is a medium-sized document with various Markdown features.

## Section 1: Text Formatting

You can write text in **bold**, *italic*, or ***both***.

## Section 2: Lists

- First item
- Second item
- Third item

## Section 3: Code Blocks

```rust
fn main() {
    println!("Hello, world!");
}
```

## Section 4: Links

Visit [example.com](https://example.com) for more information.

## Section 5: Blockquotes

> This is a blockquote.

## Section 6: Tables

| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
"#;
    let large = medium.repeat(10);
    let options = Options::default();
    c.bench_function("synthetic_large_document", |b| {
        b.iter(|| markdown_to_html_with_options(black_box(&large), &options))
    });
}

criterion_group!(
    synthetic_benchmarks,
    bench_small_document,
    bench_medium_document,
    bench_large_document,
);

criterion_main!(
    block_benchmarks,
    inline_benchmarks,
    full_document_benchmarks,
    synthetic_benchmarks,
);
