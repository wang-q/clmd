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
cargo bench --bench string_optimization_benchmark
```

## Benchmark Categories

- **categorized_benchmark**: Block-level, inline, full document, and synthetic parsing
- **feature_benchmark**: Specific Markdown features (smart punctuation, tables, code blocks, etc.)
- **pathological_benchmark**: Stress tests with extreme inputs (nested emphasis, many links, deep lists)
- **real_world_benchmark**: Real-world document samples
- **string_optimization_benchmark**: String processing optimizations (text-heavy docs, HTML output, line processing)
- **sequence_benchmark**: BasedSequence performance vs standard String operations
- **ast_conversion_benchmark**: AST parsing performance benchmarks

## String Optimization Benchmarks

The `string_optimization_benchmark` specifically tests the performance improvements from recent optimizations:

- **text_heavy**: Tests `parse_string` optimization with plain and formatted text
- **text_node_merging**: Tests `merge_adjacent_text_nodes` optimization
- **html_output**: Tests `write!` vs `format!` optimization in HTML generation
- **line_processing**: Tests `process_line` optimization for line handling
- **memory_allocation**: Tests output buffer pre-allocation for large documents
- **smart_punctuation_comparison**: Compares performance with/without smart punctuation
- **append_text**: Tests text node appending performance

## Sample Files

The `samples/` directory contains test files from cmark and other sources:

- `block-*.md`: Block-level element tests
- `inline-*.md`: Inline element tests
- `lorem*.md`: Full document tests
- `pathological/`: Stress test samples

## Cross-Language Comparison

For cross-language comparisons, see:

- `examples/bench/cross_language_bench.rs` - Compare with cmark (C) and commonmark.js (JS)

Run with:

```bash
# Build cross-language benchmark
cargo build --release --example cross_language_bench

# Run hyperfine comparison
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_language_bench benches/samples/lorem1.md'
```

## Documentation

For detailed benchmark results and methodology, see:

- [docs/benchmark/results.md](../docs/benchmark/results.md) - Latest benchmark results
- [docs/benchmark/methodology.md](../docs/benchmark/methodology.md) - Testing methodology
- [docs/benchmark/history.md](../docs/benchmark/history.md) - Historical performance data
