//! Formatter Benchmark Tests
//!
//! Benchmarks for the CommonMark formatter performance.

use clmd::{markdown_to_commonmark, Options, Plugins};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

/// Generate a large document with mixed content
fn generate_large_document(paragraphs: usize) -> String {
    let mut doc = String::new();

    for i in 0..paragraphs {
        doc.push_str(&format!("# Heading {}\n\n", i));
        doc.push_str(&format!(
            "This is paragraph {} with **bold** and *italic* text. ",
            i
        ));
        doc.push_str("It also contains [a link](https://example.com) ");
        doc.push_str("and `inline code`.\n\n");

        if i % 3 == 0 {
            doc.push_str("## Subheading\n\n");
            doc.push_str("> This is a blockquote with some important information.\n\n");
        }

        if i % 5 == 0 {
            doc.push_str("- Item 1\n");
            doc.push_str("- Item 2\n");
            doc.push_str("  - Nested item\n");
            doc.push_str("- Item 3\n\n");
        }

        if i % 7 == 0 {
            doc.push_str("```rust\n");
            doc.push_str(&format!("fn function_{}() {{\n", i));
            doc.push_str("    println!(\"Hello, World!\");\n");
            doc.push_str("}\n");
            doc.push_str("```\n\n");
        }
    }

    doc
}

/// Generate a document with complex nested lists
fn generate_nested_lists(depth: usize, items_per_level: usize) -> String {
    fn generate_list(level: usize, max_depth: usize, items: usize) -> String {
        if level > max_depth {
            return String::new();
        }

        let mut list = String::new();
        for i in 0..items {
            let indent = "  ".repeat(level);
            list.push_str(&format!("{}- Item {} at level {}\n", indent, i, level));

            if level < max_depth {
                list.push_str(&generate_list(level + 1, max_depth, items));
            }
        }
        list
    }

    generate_list(0, depth, items_per_level)
}

/// Generate a complex table
fn generate_complex_table(rows: usize, cols: usize) -> String {
    let mut table = String::new();

    // Header
    for c in 0..cols {
        table.push_str(&format!("| Header {} ", c));
    }
    table.push_str("|\n");

    // Separator
    for _ in 0..cols {
        table.push_str("|----------");
    }
    table.push_str("|\n");

    // Rows
    for r in 0..rows {
        for c in 0..cols {
            table.push_str(&format!("| Cell {},{} ", r, c));
        }
        table.push_str("|\n");
    }

    table
}

/// Benchmark: Large document formatting
fn bench_large_document(c: &mut Criterion) {
    let input = generate_large_document(100);

    let mut group = c.benchmark_group("formatter_large_document");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("markdown_to_commonmark", |b| {
        b.iter(|| {
            markdown_to_commonmark(
                black_box(&input),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });
    group.finish();
}

/// Benchmark: Simple paragraphs
fn bench_simple_paragraphs(c: &mut Criterion) {
    let input = "This is a simple paragraph.\n\n\
                 This is another paragraph with some text.\n\n\
                 And here is a third paragraph for good measure.";

    let mut group = c.benchmark_group("formatter_simple_paragraphs");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("markdown_to_commonmark", |b| {
        b.iter(|| {
            markdown_to_commonmark(
                black_box(input),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });
    group.finish();
}

/// Benchmark: Emphasis and strong
fn bench_emphasis(c: &mut Criterion) {
    let input = "This is *italic* and this is **bold**.\n\n\
                 Here is more *emphasis* and **strong** text.\n\n\
                 And some ***bold italic*** text too.";

    let mut group = c.benchmark_group("formatter_emphasis");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("markdown_to_commonmark", |b| {
        b.iter(|| {
            markdown_to_commonmark(
                black_box(input),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });
    group.finish();
}

/// Benchmark: Links
fn bench_links(c: &mut Criterion) {
    let input = "[Link 1](https://example.com)\n\n\
                 [Link 2](https://example.org)\n\n\
                 [Link with title](https://example.net \"Title\")";

    let mut group = c.benchmark_group("formatter_links");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("markdown_to_commonmark", |b| {
        b.iter(|| {
            markdown_to_commonmark(
                black_box(input),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });
    group.finish();
}

/// Benchmark: Code blocks
fn bench_code_blocks(c: &mut Criterion) {
    let input = "```rust\n\
                 fn main() {\n\
                     println!(\"Hello\");\n\
                 }\n\
                 ```\n\n\
                 ```python\n\
                 def hello():\n\
                     print('Hello')\n\
                 ```";

    let mut group = c.benchmark_group("formatter_code_blocks");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("markdown_to_commonmark", |b| {
        b.iter(|| {
            markdown_to_commonmark(
                black_box(input),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });
    group.finish();
}

/// Benchmark: Tables
fn bench_tables(c: &mut Criterion) {
    let input = generate_complex_table(10, 5);

    let mut group = c.benchmark_group("formatter_tables");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("markdown_to_commonmark", |b| {
        let mut opts = Options::default();
        opts.extension.table = true;
        b.iter(|| markdown_to_commonmark(black_box(&input), &opts, &Plugins::default()))
    });
    group.finish();
}

/// Benchmark: Nested lists
fn bench_nested_lists(c: &mut Criterion) {
    let input = generate_nested_lists(5, 3);

    let mut group = c.benchmark_group("formatter_nested_lists");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("markdown_to_commonmark", |b| {
        b.iter(|| {
            markdown_to_commonmark(
                black_box(&input),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });
    group.finish();
}

/// Benchmark: Headings
fn bench_headings(c: &mut Criterion) {
    let input = "# Heading 1\n\n\
                 ## Heading 2\n\n\
                 ### Heading 3\n\n\
                 #### Heading 4\n\n\
                 ##### Heading 5\n\n\
                 ###### Heading 6";

    let mut group = c.benchmark_group("formatter_headings");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("markdown_to_commonmark", |b| {
        b.iter(|| {
            markdown_to_commonmark(
                black_box(input),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });
    group.finish();
}

/// Benchmark: Blockquotes
fn bench_blockquotes(c: &mut Criterion) {
    let input = "> This is a blockquote.\n\
                 > It spans multiple lines.\n\
                 >\n\
                 > And has multiple paragraphs.";

    let mut group = c.benchmark_group("formatter_blockquotes");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("markdown_to_commonmark", |b| {
        b.iter(|| {
            markdown_to_commonmark(
                black_box(input),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });
    group.finish();
}

/// Benchmark: Task lists
fn bench_task_lists(c: &mut Criterion) {
    let input = "- [ ] Unchecked task\n\
                 - [x] Checked task\n\
                 - [ ] Another unchecked task\n\
                 - [x] Another checked task";

    let mut group = c.benchmark_group("formatter_task_lists");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("markdown_to_commonmark", |b| {
        b.iter(|| {
            markdown_to_commonmark(
                black_box(input),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });
    group.finish();
}

/// Benchmark: HeadingsMixed content (realistic document)
fn bench_mixed_content(c: &mut Criterion) {
    let input = r#"# Document Title

This is an introduction paragraph with **bold** and *italic* text.

## Section 1: Lists

- Bullet item 1
- Bullet item 2
  - Nested item A
  - Nested item B

## Section 2: Code

```rust
fn hello() {
    println!("Hello, World!");
}
```

## Section 3: Table

| Name  | Value |
|-------|-------|
| One   | 1     |
| Two   | 2     |

> A blockquote with some text.

## Section 4: Task List

- [ ] Task 1
- [x] Task 2
- [ ] Task 3
"#;

    let mut group = c.benchmark_group("formatter_mixed_content");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("markdown_to_commonmark", |b| {
        let mut opts = Options::default();
        opts.extension.table = true;
        b.iter(|| markdown_to_commonmark(black_box(input), &opts, &Plugins::default()))
    });
    group.finish();
}

criterion_group!(
    formatter_benchmarks,
    bench_large_document,
    bench_simple_paragraphs,
    bench_emphasis,
    bench_links,
    bench_code_blocks,
    bench_tables,
    bench_nested_lists,
    bench_headings,
    bench_blockquotes,
    bench_task_lists,
    bench_mixed_content,
);

criterion_main!(formatter_benchmarks);
