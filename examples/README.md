# Examples

This directory contains example programs demonstrating clmd usage and performance characteristics.

## Directory Structure

```
examples/
├── bench/              # Benchmarking examples
│   ├── cross_language_bench.rs    # Compare with cmark (C) and commonmark.js (JS)
│   ├── cross_rust_bench.rs        # Compare with comrak and pulldown-cmark (Rust)
│   └── flamegraph_bench.rs        # Profiling with flamegraph
└── compare/            # Comparison examples
    └── arena_comparison.rs        # Arena-based performance test
```

## Benchmark Examples

### Cross-Language Benchmark

Compare clmd with other language implementations:

```bash
# Build
cargo build --release --example cross_language_bench

# Run with hyperfine
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_language_bench benches/samples/lorem1.md' \
  'cmark benches/samples/lorem1.md' \
  'node bench_commonmark.js benches/samples/lorem1.md'
```

### Cross-Rust Benchmark

Compare clmd with other Rust Markdown parsers:

```bash
# Build with all features
cargo build --release --example cross_rust_bench --features "comrak pulldown-cmark"

# Run with hyperfine
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_rust_bench benches/samples/lorem1.md' \
  './target/release/examples/comrak_bench benches/samples/lorem1.md' \
  './target/release/examples/pulldown_cmark_bench benches/samples/lorem1.md'
```

### Flamegraph Benchmark

Generate flamegraph for profiling:

```bash
# Install cargo-flamegraph if not already installed
cargo install flamegraph

# Run with flamegraph
cargo flamegraph --example flamegraph_bench

# View the generated flamegraph.svg
open flamegraph.svg
```

## Comparison Examples

### Arena Comparison

Test Arena-based implementation performance:

```bash
cargo run --example arena_comparison --release
```

## Notes

- All benchmark examples should be run in release mode (`--release`) for accurate results
- The `cross_rust_bench` example requires the `comrak` and/or `pulldown-cmark` features to be enabled
- See `benches/` directory for Criterion-based benchmarks
