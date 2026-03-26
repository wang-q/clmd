# Benchmark Documentation

This directory contains benchmark-related documentation for clmd.

## Files

- **results.md**: Latest benchmark results
- **methodology.md**: Testing methodology and cross-language comparison
- **history.md**: Historical performance data

## Quick Links

- [Latest Results](results.md)
- [Testing Methodology](methodology.md)
- [Cross-Language Comparison](../comparison/cross-language.md)

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

1. **categorized_benchmark**: Block-level, inline, full document, and synthetic parsing
2. **feature_benchmark**: Specific Markdown features (smart punctuation, tables, etc.)
3. **pathological_benchmark**: Stress tests with extreme inputs
4. **real_world_benchmark**: Real-world document samples

## Cross-Language Comparison

For comparing clmd with other Markdown parsers:

```bash
# Build cross-language benchmark
cargo build --release --example cross_language_bench

# Run with hyperfine
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_language_bench benches/samples/lorem1.md' \
  'cmark benches/samples/lorem1.md' \
  'node bench_commonmark.js benches/samples/lorem1.md'
```
