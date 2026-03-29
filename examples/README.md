# Examples

This directory contains example programs demonstrating clmd usage and performance characteristics.

## Directory Structure

```
examples/
└── bench/              # Benchmarking examples
    └── benchmark.rs    # Comprehensive benchmark suite
```

## Comprehensive Benchmark

A single, feature-rich benchmark example that provides multiple benchmark modes:

### Features

- **Cross-Language Comparison**: Compare with cmark (C) and commonmark.js (JS)
- **Cross-Rust Parser Comparison**: Compare with comrak and pulldown-cmark (Rust)
- **Flamegraph Generation**: Profiling support for performance analysis
- **Arena Performance Test**: Benchmark the arena-based memory management

### Build

```bash
# Build with default features
cargo build --release --example benchmark

# Build with optional Rust parser comparisons (comrak and pulldown-cmark)
cargo build --release --example benchmark --features "comrak pulldown-cmark"
```

### Usage

```bash
./target/release/examples/benchmark [OPTIONS] [FILE]

Options:
  -h, --help                Show this help message
  --mode MODE               Benchmark mode: cross-language, cross-rust, flamegraph, arena
  -i, --iterations N        Number of iterations (for flamegraph mode)
  --no-optimization         Prevent compiler optimizations
```

### Mode Examples

#### Cross-Language Benchmark

```bash
# Run with custom input file
./target/release/examples/benchmark --mode cross-language benches/samples/lorem1.md

# Compare with other implementations using hyperfine
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/benchmark --mode cross-language benches/samples/lorem1.md' \
  'cmark benches/samples/lorem1.md' \
  'node -e "require(\'commonmark\').parse(require(\'fs\').readFileSync(\'benches/samples/lorem1.md\', \'utf8\'))"'
```

#### Cross-Rust Parser Benchmark

```bash
# Run with optional Rust parser comparisons
./target/release/examples/benchmark --mode cross-rust benches/samples/lorem1.md

# Benchmark all Rust parsers with hyperfine
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/benchmark --mode cross-rust benches/samples/lorem1.md' \
  './target/release/examples/comrak_bench benches/samples/lorem1.md' \
  './target/release/examples/pulldown_cmark_bench benches/samples/lorem1.md'
```

#### Flamegraph Benchmark

```bash
# Install cargo-flamegraph if not already installed
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --example benchmark -- --mode flamegraph -i 10000

# View the generated flamegraph.svg
open flamegraph.svg
```

#### Arena Performance Test

```bash
# Run arena-based performance test
./target/release/examples/benchmark --mode arena

# Run with custom iterations
./target/release/examples/benchmark --mode arena -i 5000
```

## Notes

- Run benchmark examples in release mode (`--release`) for accurate performance results
- The `cross-rust` mode requires the `comrak` and/or `pulldown-cmark` features to be enabled
- See `benches/` directory for Criterion-based benchmarks
- Use `--help` to see all available options and modes
- The benchmark example supports custom input files for testing with different content

