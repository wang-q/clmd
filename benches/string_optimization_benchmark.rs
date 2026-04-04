//! String Processing Optimization Benchmarks
//!
//! Benchmarks specifically designed to test the string processing optimizations
//! implemented in clmd. These benchmarks help verify the performance improvements
//! from:
//! - Pre-allocated string buffers
//! - Avoiding unnecessary String allocations
//! - Using write! instead of format!
//! - Efficient text node merging

use clmd::{markdown_to_html, Options, Plugins};
use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};

// Benchmark text-heavy documents (tests parse_string optimization)
fn bench_text_heavy_documents(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_heavy");

    // Plain text without special characters (tests fast path)
    let plain_text = "This is a paragraph with plain text content. ".repeat(100);
    group.throughput(Throughput::Bytes(plain_text.len() as u64));
    group.bench_function("plain_text_5kb", |b| {
        b.iter(|| {
            markdown_to_html(
                black_box(&plain_text),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });

    // Text with some formatting (tests mixed path)
    let formatted_text =
        "This is **bold** and *italic* text with `code` and [links](http://example.com). ".repeat(50);
    group.throughput(Throughput::Bytes(formatted_text.len() as u64));
    group.bench_function("formatted_text_5kb", |b| {
        b.iter(|| {
            markdown_to_html(
                black_box(&formatted_text),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });

    group.finish();
}

// Benchmark text node merging (tests merge_adjacent_text_nodes optimization)
fn bench_text_node_merging(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_node_merging");

    // Many small text segments that need merging
    for segment_count in [10, 50, 100, 200] {
        let mut input = String::new();
        for i in 0..segment_count {
            input.push_str(&format!("word{} ", i));
        }

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(segment_count),
            &input,
            |b, input| {
                b.iter(|| {
                    markdown_to_html(
                        black_box(input),
                        &Options::default(),
                        &Plugins::default(),
                    )
                })
            },
        );
    }

    group.finish();
}

// Benchmark HTML output generation (tests write! vs format! optimization)
fn bench_html_output_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("html_output");

    // Many headings (tests heading tag generation)
    let headings_doc = (1..=100)
        .map(|i| format!("# Heading {}\n\nParagraph {} content here.\n\n", i, i))
        .collect::<String>();
    group.throughput(Throughput::Bytes(headings_doc.len() as u64));
    group.bench_function("many_headings", |b| {
        b.iter(|| {
            markdown_to_html(
                black_box(&headings_doc),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });

    // Many task items (tests checkbox generation)
    let task_list = (1..=100)
        .map(|i| format!("- [x] Task {}\n", i))
        .collect::<String>();
    group.throughput(Throughput::Bytes(task_list.len() as u64));
    group.bench_function("many_task_items", |b| {
        b.iter(|| {
            markdown_to_html(
                black_box(&task_list),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });

    // Tables with alignment (tests table header generation)
    for col_count in [5, 10, 20] {
        let mut table = String::new();
        // Header
        for i in 0..col_count {
            table.push_str(&format!("| Header {} ", i));
        }
        table.push_str("|\n");
        // Separator with alignments
        for _ in 0..col_count {
            table.push_str("| :---: ");
        }
        table.push_str("|\n");
        // Data row
        for i in 0..col_count {
            table.push_str(&format!("| Cell {} ", i));
        }
        table.push_str("|\n");

        group.throughput(Throughput::Bytes(table.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("table_with_alignment", col_count),
            &table,
            |b, table| {
                b.iter(|| {
                    markdown_to_html(
                        black_box(table),
                        &Options::default(),
                        &Plugins::default(),
                    )
                })
            },
        );
    }

    group.finish();
}

// Benchmark line processing (tests process_line optimization)
fn bench_line_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_processing");

    // Document with many lines (tests line processing fast path)
    let many_lines = (1..=1000)
        .map(|i| format!("Line {} of the document\n", i))
        .collect::<String>();
    group.throughput(Throughput::Bytes(many_lines.len() as u64));
    group.bench_function("many_lines_10k", |b| {
        b.iter(|| {
            markdown_to_html(
                black_box(&many_lines),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });

    // Document with CRLF line endings (tests line ending normalization)
    let crlf_lines = (1..=1000)
        .map(|i| format!("Line {} of the document\r\n", i))
        .collect::<String>();
    group.throughput(Throughput::Bytes(crlf_lines.len() as u64));
    group.bench_function("crlf_lines_10k", |b| {
        b.iter(|| {
            markdown_to_html(
                black_box(&crlf_lines),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });

    group.finish();
}

// Benchmark memory allocation patterns
fn bench_memory_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    // Large document (tests output buffer pre-allocation)
    let large_doc = "# Large Document\n\n".to_string()
        + &"This is a paragraph with some content. ".repeat(1000)
        + "\n\n## Section 2\n\n"
        + &"More content here. ".repeat(1000);
    group.throughput(Throughput::Bytes(large_doc.len() as u64));
    group.bench_function("large_doc_50kb", |b| {
        b.iter(|| {
            markdown_to_html(
                black_box(&large_doc),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });

    // Document with many footnotes (tests footnote reference generation)
    let mut footnote_doc = String::from("# Document with Footnotes\n\n");
    for i in 0..50 {
        footnote_doc.push_str(&format!("Text with footnote[^{}] and more text. ", i));
    }
    footnote_doc.push_str("\n\n");
    for i in 0..50 {
        footnote_doc.push_str(&format!("[^{}]: Footnote {} content\n", i, i));
    }
    group.throughput(Throughput::Bytes(footnote_doc.len() as u64));
    group.bench_function("many_footnotes", |b| {
        b.iter(|| {
            markdown_to_html(
                black_box(&footnote_doc),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });

    group.finish();
}

// Benchmark comparing with/without smart punctuation
fn bench_smart_punctuation_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("smart_punctuation_comparison");

    let text_with_quotes = "'This is a quote' and \"another quote\". ".repeat(100);

    // Without smart punctuation (tests optimized path)
    let options_no_smart = Options::default();
    group.throughput(Throughput::Bytes(text_with_quotes.len() as u64));
    group.bench_function("without_smart_punctuation", |b| {
        b.iter(|| {
            markdown_to_html(
                black_box(&text_with_quotes),
                &options_no_smart,
                &Plugins::default(),
            )
        })
    });

    // With smart punctuation (tests regular path)
    let mut options_smart = Options::default();
    options_smart.parse.smart = true;
    group.throughput(Throughput::Bytes(text_with_quotes.len() as u64));
    group.bench_function("with_smart_punctuation", |b| {
        b.iter(|| {
            markdown_to_html(
                black_box(&text_with_quotes),
                &options_smart,
                &Plugins::default(),
            )
        })
    });

    group.finish();
}

// Benchmark append_text operations (tests text node appending)
fn bench_append_text_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("append_text");

    // Document with many inline code segments (creates many text nodes)
    let code_heavy = (1..=100)
        .map(|i| format!("Text `code{}` more text `code{}` end. ", i, i + 1))
        .collect::<String>();
    group.throughput(Throughput::Bytes(code_heavy.len() as u64));
    group.bench_function("code_heavy_document", |b| {
        b.iter(|| {
            markdown_to_html(
                black_box(&code_heavy),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });

    // Document with emphasis (creates text nodes around emphasis)
    let emphasis_heavy = (1..=100)
        .map(|i| format!("Text *emphasis{}* more text **bold{}** end. ", i, i))
        .collect::<String>();
    group.throughput(Throughput::Bytes(emphasis_heavy.len() as u64));
    group.bench_function("emphasis_heavy_document", |b| {
        b.iter(|| {
            markdown_to_html(
                black_box(&emphasis_heavy),
                &Options::default(),
                &Plugins::default(),
            )
        })
    });

    group.finish();
}

criterion_group!(
    string_optimization_benchmarks,
    bench_text_heavy_documents,
    bench_text_node_merging,
    bench_html_output_generation,
    bench_line_processing,
    bench_memory_allocation_patterns,
    bench_smart_punctuation_comparison,
    bench_append_text_operations,
);

criterion_main!(string_optimization_benchmarks);
