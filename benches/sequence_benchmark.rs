//! Benchmark comparing String operations vs BasedSequence
//!
//! This benchmark tests whether BasedSequence provides performance benefits
//! over standard String operations for typical Markdown parsing scenarios.

use clmd::sequence::BasedSequence;
use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};

// Sample Markdown text for testing
const SAMPLE_TEXT: &str = r#"# Heading 1

This is a paragraph with some **bold** and *italic* text.
It also has a [link](https://example.com) and some `inline code`.

## Heading 2

- List item 1
- List item 2 with **bold**
- List item 3 with [a link](http://test.com)

> This is a blockquote with multiple lines.
> It continues here and has some *emphasis*.

```rust
fn main() {
    println!("Hello, world!");
}
```

| Table | Header |
|-------|--------|
| Cell1 | Cell2  |
| Cell3 | Cell4  |
"#;

// Benchmark 1: Creating slices/substrings
fn bench_slicing(c: &mut Criterion) {
    let mut group = c.benchmark_group("slicing");
    let text = SAMPLE_TEXT;

    group.throughput(Throughput::Bytes(text.len() as u64));

    // Standard String slicing (creates new String)
    group.bench_function("string_slice", |b| {
        b.iter(|| {
            let text = black_box(text);
            let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
            let _result: Vec<String> = lines
                .iter()
                .map(|line| {
                    if line.len() > 10 {
                        line[0..10].to_string()
                    } else {
                        line.clone()
                    }
                })
                .collect();
        });
    });

    // BasedSequence slicing (zero-copy)
    group.bench_function("based_sequence_slice", |b| {
        b.iter(|| {
            let text = black_box(text);
            let seq = BasedSequence::new(text);
            let lines: Vec<BasedSequence<'_>> = seq.lines().collect();
            let _result: Vec<BasedSequence<'_>> = lines
                .iter()
                .map(|line: &BasedSequence<'_>| {
                    if line.len() > 10 {
                        line.sub_sequence(0, 10)
                    } else {
                        *line
                    }
                })
                .collect();
        });
    });

    group.finish();
}

// Benchmark 2: Trim operations
fn bench_trim(c: &mut Criterion) {
    let mut group = c.benchmark_group("trim");
    let text = "   This is a line with whitespace   ";

    group.throughput(Throughput::Bytes(text.len() as u64));

    group.bench_function("string_trim", |b| {
        b.iter(|| {
            let text = black_box(text);
            let _result = text.trim();
        });
    });

    group.bench_function("based_sequence_trim", |b| {
        b.iter(|| {
            let text = black_box(text);
            let seq = BasedSequence::new(text);
            let _result = seq.trim();
        });
    });

    group.finish();
}

// Benchmark 3: Finding patterns
fn bench_find(c: &mut Criterion) {
    let mut group = c.benchmark_group("find");
    let text = SAMPLE_TEXT;

    group.throughput(Throughput::Bytes(text.len() as u64));

    group.bench_function("string_find", |b| {
        b.iter(|| {
            let text = black_box(text);
            let _result = text.find("**");
            let _result = text.find("[");
            let _result = text.find("```");
        });
    });

    group.bench_function("based_sequence_find", |b| {
        b.iter(|| {
            let text = black_box(text);
            let seq = BasedSequence::new(text);
            let _result = seq.find("**");
            let _result = seq.find("[");
            let _result = seq.find("```");
        });
    });

    group.finish();
}

// Benchmark 4: Splitting lines (common in Markdown parsing)
fn bench_lines(c: &mut Criterion) {
    let mut group = c.benchmark_group("lines");
    let text = SAMPLE_TEXT;

    group.throughput(Throughput::Bytes(text.len() as u64));

    // Standard approach: collect into Vec<String>
    group.bench_function("string_lines_collect", |b| {
        b.iter(|| {
            let text = black_box(text);
            let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
            let _count = lines.len();
        });
    });

    // BasedSequence approach: zero-copy iteration
    group.bench_function("based_sequence_lines", |b| {
        b.iter(|| {
            let text = black_box(text);
            let seq = BasedSequence::new(text);
            let lines: Vec<BasedSequence> = seq.lines().collect();
            let _count = lines.len();
        });
    });

    group.finish();
}

// Benchmark 5: Complex parsing simulation (like actual Markdown parsing)
fn bench_parsing_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_simulation");
    let text = SAMPLE_TEXT;

    group.throughput(Throughput::Bytes(text.len() as u64));

    // Simulate parsing with String operations
    group.bench_function("string_parsing", |b| {
        b.iter(|| {
            let text = black_box(text);
            let lines: Vec<&str> = text.lines().collect();
            let mut results = Vec::new();

            for line in lines {
                let trimmed = line.trim();
                if trimmed.starts_with("#") {
                    // Heading
                    let level = trimmed.chars().take_while(|&c| c == '#').count();
                    let content = trimmed.trim_start_matches('#').trim().to_string();
                    results.push(("heading", level, content));
                } else if trimmed.starts_with("-") {
                    // List item
                    let content = trimmed.trim_start_matches('-').trim().to_string();
                    results.push(("list", 0, content));
                } else if !trimmed.is_empty() {
                    // Paragraph
                    results.push(("paragraph", 0, trimmed.to_string()));
                }
            }
            black_box(results);
        });
    });

    // Simulate parsing with BasedSequence
    group.bench_function("based_sequence_parsing", |b| {
        b.iter(|| {
            let text = black_box(text);
            let seq = BasedSequence::new(text);
            let lines: Vec<BasedSequence<'_>> = seq.lines().collect();
            let mut results = Vec::new();

            for line in lines {
                let trimmed: BasedSequence<'_> = line.trim();
                if trimmed.starts_with("#") {
                    // Heading
                    let level =
                        trimmed.as_str().chars().take_while(|&c| c == '#').count();
                    let content = trimmed.sub_sequence(level, trimmed.len()).trim();
                    results.push(("heading", level, content.as_str().to_string()));
                } else if trimmed.starts_with("-") {
                    // List item
                    let content = trimmed.sub_sequence(1, trimmed.len()).trim();
                    results.push(("list", 0, content.as_str().to_string()));
                } else if !trimmed.is_empty() {
                    // Paragraph
                    results.push(("paragraph", 0, trimmed.as_str().to_string()));
                }
            }
            black_box(results);
        });
    });

    group.finish();
}

// Benchmark 6: Large file processing
fn bench_large_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_file");

    // Create a larger text (100KB)
    let large_text = SAMPLE_TEXT.repeat(1000);

    group.throughput(Throughput::Bytes(large_text.len() as u64));

    group.bench_function("string_large_file", |b| {
        b.iter(|| {
            let text = black_box(&large_text);
            let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
            let _count = lines.len();
        });
    });

    group.bench_function("based_sequence_large_file", |b| {
        b.iter(|| {
            let text = black_box(&large_text);
            let seq = BasedSequence::new(text);
            let lines: Vec<BasedSequence> = seq.lines().collect();
            let _count = lines.len();
        });
    });

    group.finish();
}

// Benchmark 7: Line/column position tracking
fn bench_position_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("position_tracking");
    let text = SAMPLE_TEXT;

    group.throughput(Throughput::Bytes(text.len() as u64));

    // Manual position tracking with String
    group.bench_function("string_position", |b| {
        b.iter(|| {
            let text = black_box(text);
            let mut positions = Vec::new();
            let mut line = 1;
            let mut col = 1;

            for (i, c) in text.char_indices() {
                if c == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                if i % 50 == 0 {
                    positions.push((line, col));
                }
            }
            black_box(positions);
        });
    });

    // BasedSequence built-in position tracking
    group.bench_function("based_sequence_position", |b| {
        b.iter(|| {
            let text = black_box(text);
            let seq = BasedSequence::new(text);
            let mut positions = Vec::new();

            for i in (0..seq.len()).step_by(50) {
                let sub = seq.sub_sequence(i, (i + 1).min(seq.len()));
                positions.push((sub.start_line(), sub.start_column()));
            }
            black_box(positions);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_slicing,
    bench_trim,
    bench_find,
    bench_lines,
    bench_parsing_simulation,
    bench_large_file,
    bench_position_tracking
);
criterion_main!(benches);
