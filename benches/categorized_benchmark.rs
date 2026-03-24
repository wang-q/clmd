//! Categorized Benchmark Tests
//!
//! Benchmarks based on cmark's sample files, organized by category.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use clmd::{markdown_to_html, options};

// Block-level benchmarks
fn bench_block_quotes_flat(c: &mut Criterion) {
    let input = include_str!("samples/block-bq-flat.md");
    c.bench_function("block_quotes_flat", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_block_quotes_nested(c: &mut Criterion) {
    let input = include_str!("samples/block-bq-nested.md");
    c.bench_function("block_quotes_nested", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_block_code(c: &mut Criterion) {
    let input = include_str!("samples/block-code.md");
    c.bench_function("block_code", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_block_fences(c: &mut Criterion) {
    let input = include_str!("samples/block-fences.md");
    c.bench_function("block_fences", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_block_heading(c: &mut Criterion) {
    let input = include_str!("samples/block-heading.md");
    c.bench_function("block_heading", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_block_hr(c: &mut Criterion) {
    let input = include_str!("samples/block-hr.md");
    c.bench_function("block_hr", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_block_list_flat(c: &mut Criterion) {
    let input = include_str!("samples/block-list-flat.md");
    c.bench_function("block_list_flat", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_block_list_nested(c: &mut Criterion) {
    let input = include_str!("samples/block-list-nested.md");
    c.bench_function("block_list_nested", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_block_html(c: &mut Criterion) {
    let input = include_str!("samples/block-html.md");
    c.bench_function("block_html", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_block_lheading(c: &mut Criterion) {
    let input = include_str!("samples/block-lheading.md");
    c.bench_function("block_lheading", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_block_ref_flat(c: &mut Criterion) {
    let input = include_str!("samples/block-ref-flat.md");
    c.bench_function("block_ref_flat", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_block_ref_nested(c: &mut Criterion) {
    let input = include_str!("samples/block-ref-nested.md");
    c.bench_function("block_ref_nested", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

// Inline benchmarks
fn bench_inline_autolink(c: &mut Criterion) {
    let input = include_str!("samples/inline-autolink.md");
    c.bench_function("inline_autolink", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_inline_backticks(c: &mut Criterion) {
    let input = include_str!("samples/inline-backticks.md");
    c.bench_function("inline_backticks", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_inline_em_flat(c: &mut Criterion) {
    let input = include_str!("samples/inline-em-flat.md");
    c.bench_function("inline_em_flat", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_inline_em_nested(c: &mut Criterion) {
    let input = include_str!("samples/inline-em-nested.md");
    c.bench_function("inline_em_nested", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_inline_em_worst(c: &mut Criterion) {
    let input = include_str!("samples/inline-em-worst.md");
    c.bench_function("inline_em_worst", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_inline_entity(c: &mut Criterion) {
    let input = include_str!("samples/inline-entity.md");
    c.bench_function("inline_entity", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_inline_escape(c: &mut Criterion) {
    let input = include_str!("samples/inline-escape.md");
    c.bench_function("inline_escape", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_inline_html(c: &mut Criterion) {
    let input = include_str!("samples/inline-html.md");
    c.bench_function("inline_html", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_inline_links_flat(c: &mut Criterion) {
    let input = include_str!("samples/inline-links-flat.md");
    c.bench_function("inline_links_flat", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_inline_links_nested(c: &mut Criterion) {
    let input = include_str!("samples/inline-links-nested.md");
    c.bench_function("inline_links_nested", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_inline_newlines(c: &mut Criterion) {
    let input = include_str!("samples/inline-newlines.md");
    c.bench_function("inline_newlines", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

// Full document benchmark
fn bench_lorem1(c: &mut Criterion) {
    let input = include_str!("samples/lorem1.md");
    c.bench_function("lorem1_full_document", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
    });
}

fn bench_rawtabs(c: &mut Criterion) {
    let input = include_str!("samples/rawtabs.md");
    c.bench_function("rawtabs", |b| {
        b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
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
    bench_rawtabs,
);

criterion_main!(
    block_benchmarks,
    inline_benchmarks,
    full_document_benchmarks,
);
