//! Feature-specific Benchmark Tests
//!
//! Benchmarks for specific Markdown features.

use clmd::{markdown_to_html, Options};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

// Smart punctuation benchmark
fn bench_smart_punctuation(c: &mut Criterion) {
    let input = r#"'This here a real "quote"'
And -- if you're interested -- some em-dashes.
Wait --- she actually said that?
Wow... Becky is so 'mean'!"#;

    let mut group = c.benchmark_group("smart_punctuation");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("clmd", |b| {
        b.iter(|| markdown_to_html(black_box(input), &Options::default()))
    });
    group.finish();
}

// Links and emphasis benchmark
fn bench_links_and_emphasis(c: &mut Criterion) {
    let input = "This is a [link](example.com). **Cool!**\n\n\
                 This is a [link](example.com). **Cool!**\n\n\
                 This is a [link](example.com). **Cool!**";

    let mut group = c.benchmark_group("links_and_emphasis");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("clmd", |b| {
        b.iter(|| markdown_to_html(black_box(input), &Options::default()))
    });
    group.finish();
}

// Code blocks benchmark
fn bench_code_blocks(c: &mut Criterion) {
    let input = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n\n\
                 ```python\ndef hello():\n    print('Hello')\n```";

    let mut group = c.benchmark_group("code_blocks");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("clmd", |b| {
        b.iter(|| markdown_to_html(black_box(input), &Options::default()))
    });
    group.finish();
}

// Tables benchmark
fn bench_tables(c: &mut Criterion) {
    let input = "| Header 1 | Header 2 | Header 3 |\n\
                 |----------|----------|----------|\n\
                 | Cell 1   | Cell 2   | Cell 3   |\n\
                 | Cell 4   | Cell 5   | Cell 6   |";

    let mut group = c.benchmark_group("tables");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("clmd", |b| {
        b.iter(|| markdown_to_html(black_box(input), &Options::default()))
    });
    group.finish();
}

// Autolinks benchmark
fn bench_autolinks(c: &mut Criterion) {
    let input = "Visit https://example.com for more info.\n\
                 Contact us at mailto:test@example.com\n\
                 Check out http://rust-lang.org";

    let mut group = c.benchmark_group("autolinks");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("clmd", |b| {
        b.iter(|| markdown_to_html(black_box(input), &Options::default()))
    });
    group.finish();
}

// HTML entities benchmark
fn bench_html_entities(c: &mut Criterion) {
    let input = "Copyright &copy; 2024. All rights reserved.\n\
                 Use &lt;tag&gt; for HTML.\n\
                 Price: &euro;100 or &pound;85 or &yen;15000";

    let mut group = c.benchmark_group("html_entities");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("clmd", |b| {
        b.iter(|| markdown_to_html(black_box(input), &Options::default()))
    });
    group.finish();
}

criterion_group!(
    feature_benchmarks,
    bench_smart_punctuation,
    bench_links_and_emphasis,
    bench_code_blocks,
    bench_tables,
    bench_autolinks,
    bench_html_entities,
);

criterion_main!(feature_benchmarks);
