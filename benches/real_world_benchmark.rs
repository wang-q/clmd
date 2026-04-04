//! Real-World Benchmark Tests
//!
//! Benchmarks using real-world Markdown documents.

use clmd::{markdown_to_html, Options, Plugins};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::fs::{read_dir, read_to_string};
use std::path::Path;

// Benchmark all files in the samples directory
fn bench_samples(c: &mut Criterion) {
    let samples_dir = Path::new("benches/samples");

    if let Ok(entries) = read_dir(samples_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if !metadata.is_file() {
                    continue;
                }

                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();

                if !filename_str.ends_with(".md") {
                    continue;
                }

                if let Ok(content) = read_to_string(entry.path()) {
                    let mut group =
                        c.benchmark_group(format!("sample_{}", filename_str));
                    group.throughput(Throughput::Bytes(content.len() as u64));

                    group.bench_function("clmd", |b| {
                        b.iter(|| {
                            markdown_to_html(
                                black_box(&content),
                                &Options::default(),
                                &Plugins::default(),
                            )
                        })
                    });

                    group.finish();
                }
            }
        }
    }
}

// Benchmark specific real-world documents
fn bench_lorem1(c: &mut Criterion) {
    let input = include_str!("samples/lorem1.md");
    let mut group = c.benchmark_group("realworld_lorem1");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("clmd", |b| {
        b.iter(|| {
            markdown_to_html(black_box(input), &Options::default(), &Plugins::default())
        })
    });

    group.finish();
}

fn bench_lorem_large(c: &mut Criterion) {
    let input = include_str!("samples/lorem-large.md");
    let mut group = c.benchmark_group("realworld_lorem_large");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("clmd", |b| {
        b.iter(|| {
            markdown_to_html(black_box(input), &Options::default(), &Plugins::default())
        })
    });

    group.finish();
}

fn bench_lorem_xlarge(c: &mut Criterion) {
    let input = include_str!("samples/lorem-xlarge.md");
    let mut group = c.benchmark_group("realworld_lorem_xlarge");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("clmd", |b| {
        b.iter(|| {
            markdown_to_html(black_box(input), &Options::default(), &Plugins::default())
        })
    });

    group.finish();
}

criterion_group!(
    real_world_benchmarks,
    bench_samples,
    bench_lorem1,
    bench_lorem_large,
    bench_lorem_xlarge,
);

criterion_main!(real_world_benchmarks);
