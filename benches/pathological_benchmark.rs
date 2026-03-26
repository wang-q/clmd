//! Pathological Benchmark Tests
//!
//! Stress tests with extreme inputs to test parser robustness.

use clmd::{markdown_to_html, options};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput, BenchmarkId};

// Nested emphasis stress test
fn bench_nested_emphasis(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_emphasis");

    for depth in [10, 50, 100, 200] {
        let input = "*".repeat(depth) + "text" + &"*".repeat(depth);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(depth), &input, |b, input| {
            b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
        });
    }

    group.finish();
}

// Many link definitions stress test
fn bench_many_link_defs(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_link_defs");

    for count in [100, 500, 1000] {
        let mut input = String::new();
        for i in 0..count {
            input.push_str(&format!("[link{}]: https://example.com/{}\n", i, i));
        }
        input.push_str("[link0]");

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &input, |b, input| {
            b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
        });
    }

    group.finish();
}

// Deep nested lists stress test
fn bench_deep_nested_lists(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_nested_lists");

    for depth in [10, 50, 100] {
        let mut input = String::new();
        for i in 0..depth {
            input.push_str(&"  ".repeat(i));
            input.push_str("- item\n");
        }

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(depth), &input, |b, input| {
            b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
        });
    }

    group.finish();
}

// Long inline code stress test
fn bench_long_inline_code(c: &mut Criterion) {
    let mut group = c.benchmark_group("long_inline_code");

    for length in [100, 500, 1000, 5000] {
        let code = "x".repeat(length);
        let input = format!("`{}`", code);

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(length), &input, |b, input| {
            b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
        });
    }

    group.finish();
}

// Many backticks stress test
fn bench_many_backticks(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_backticks");

    for count in [10, 50, 100] {
        let input = "`".repeat(count * 10);

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &input, |b, input| {
            b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
        });
    }

    group.finish();
}

// Table with many columns stress test
fn bench_wide_tables(c: &mut Criterion) {
    let mut group = c.benchmark_group("wide_tables");

    for cols in [10, 50, 100] {
        let mut input = String::new();
        // Header
        for i in 0..cols {
            input.push_str(&format!("| H{}", i));
        }
        input.push_str("|\n");
        // Separator
        for _ in 0..cols {
            input.push_str("|---");
        }
        input.push_str("|\n");
        // Row
        for i in 0..cols {
            input.push_str(&format!("| C{}", i));
        }
        input.push_str("|\n");

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(cols), &input, |b, input| {
            b.iter(|| markdown_to_html(black_box(input), options::DEFAULT))
        });
    }

    group.finish();
}

criterion_group!(
    pathological_benchmarks,
    bench_nested_emphasis,
    bench_many_link_defs,
    bench_deep_nested_lists,
    bench_long_inline_code,
    bench_many_backticks,
    bench_wide_tables,
);

criterion_main!(pathological_benchmarks);
