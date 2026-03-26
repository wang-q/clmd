# Benchmarks

This directory contains performance benchmarks for clmd.

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench categorized_benchmark
cargo bench --bench feature_benchmark
cargo bench --bench pathological_benchmark
cargo bench --bench real_world_benchmark
```

## Benchmark Categories

- **categorized_benchmark**: Block-level, inline, full document, and synthetic parsing
- **feature_benchmark**: Specific Markdown features (smart punctuation, tables, code blocks, etc.)
- **pathological_benchmark**: Stress tests with extreme inputs (nested emphasis, many links, deep lists)
- **real_world_benchmark**: Real-world document samples

## Sample Files

The `samples/` directory contains test files from cmark and other sources:

- `block-*.md`: Block-level element tests
- `inline-*.md`: Inline element tests
- `lorem*.md`: Full document tests
- `pathological/`: Stress test samples

## Cross-Language Comparison

For cross-language and cross-library comparisons, see:

- `examples/bench/cross_language_bench.rs` - Compare with cmark (C) and commonmark.js (JS)
- `examples/bench/cross_rust_bench.rs` - Compare with comrak and pulldown-cmark (Rust)

Run with:

```bash
# Build cross-language benchmark
cargo build --release --example cross_language_bench

# Build cross-Rust benchmark
cargo build --release --example cross_rust_bench --features "comrak pulldown-cmark"

# Run hyperfine comparison
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_language_bench benches/samples/lorem1.md' \
  './target/release/examples/cross_rust_bench benches/samples/lorem1.md'
```

## Documentation

For detailed benchmark results and methodology, see:

- [docs/benchmark/results.md](../docs/benchmark/results.md) - Latest benchmark results
- [docs/benchmark/methodology.md](../docs/benchmark/methodology.md) - Testing methodology
- [docs/benchmark/history.md](../docs/benchmark/history.md) - Historical performance data
