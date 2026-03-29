# Benchmark Methodology

This document describes the benchmarking methodology used for clmd performance testing.

## Testing Principles

1. **Consistent Measurement**: All parsers measure the same operation (parse + HTML render)
2. **Same Output**: All generate HTML output
3. **Same Iterations**: Use identical warmup and iteration counts
4. **Same Input**: Use identical test files

## Test Categories

### 1. Categorized Benchmarks

Tests organized by Markdown element type:
- **Block-level**: quotes, lists, code blocks, headings, etc.
- **Inline**: emphasis, links, entities, escapes, etc.
- **Full Document**: complete documents of various sizes
- **Synthetic**: programmatically generated test cases

### 2. Feature Benchmarks

Tests for specific Markdown features:
- Smart punctuation
- Links and emphasis
- Code blocks
- Tables
- Autolinks
- HTML entities

### 3. Pathological Benchmarks

Stress tests with extreme inputs:
- Nested emphasis (10-200 levels)
- Many link definitions (100-1000)
- Deep nested lists (10-100 levels)
- Long inline code (100-5000 chars)
- Many backticks
- Wide tables (10-100 columns)

### 4. Real-World Benchmarks

Tests using actual Markdown documents from various sources.

## Cross-Language Comparison

### Tested Implementations

| Implementation | Language | Architecture |
|----------------|----------|--------------|
| cmark | C | AST |
| clmd | Rust | AST + Arena |
| comrak | Rust | AST + Arena |
| pulldown-cmark | Rust | Event-driven |
| commonmark.js | JavaScript | AST |

### Running Comparisons

```bash
# Cross-language comparison
cargo build --release --example cross_language_bench

hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_language_bench benches/samples/lorem1.md' \
  'cmark benches/samples/lorem1.md' \
  'node examples/bench/bench_commonmark.js benches/samples/lorem1.md'

# Cross-Rust comparison
cargo build --release --example cross_rust_bench --features "comrak pulldown-cmark"

hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_rust_bench benches/samples/lorem1.md' \
  './target/release/examples/comrak_bench benches/samples/lorem1.md' \
  './target/release/examples/pulldown_cmark_bench benches/samples/lorem1.md'
```

## Test Files

Located in `benches/samples/`:

- `lorem1.md` - Complete document (~1KB)
- `lorem-large.md` - Large document (~7KB)
- `lorem-xlarge.md` - Extra large document (~110KB)
- `block-*.md` - Block element tests
- `inline-*.md` - Inline element tests
- `pathological/` - Stress test samples

## Measurement Tools

### Criterion.rs (Micro-benchmarks)

For precise measurements without process startup overhead:

```bash
cargo bench --bench categorized_benchmark
cargo bench --bench feature_benchmark
cargo bench --bench pathological_benchmark
cargo bench --bench real_world_benchmark
```

### Hyperfine (End-to-end)

For realistic measurements including process startup:

```bash
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_language_bench benches/samples/lorem1.md' \
  'cmark benches/samples/lorem1.md'
```

## Key Metrics

1. **Time**: Wall-clock time for parsing + rendering
2. **Throughput**: MB/s processing rate
3. **Relative Speed**: Comparison to baseline (cmark)

## Best Practices

1. Always run in release mode (`--release`)
2. Use proper warmup to stabilize CPU/cache
3. Run on a quiet system without interference
4. Report statistical significance (mean ± σ)
5. Include throughput for cross-document comparison
