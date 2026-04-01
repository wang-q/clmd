# Cross-Language Comparison

Comparison of clmd with Markdown parsers in other languages.

## Tested Implementations

| Implementation | Language | Version | Architecture |
|----------------|----------|---------|--------------|
| cmark | C | 0.31.2 | AST |
| clmd | Rust | 0.1.0 | AST + Arena |
| commonmark.js | JavaScript | 0.31.2 | AST |

## Performance Results (2026-04-02)

### Small File (lorem1.md, ~3.8KB)

| Implementation | Time | Relative | Notes |
|----------------|------|----------|-------|
| **cmark (C)** | **2.1 ms** | 1.00x | Native performance |
| **clmd (Rust)** | **2.3 ms** | 1.10x | +10% vs cmark |
| **commonmark.js (JS)** | **73.5 ms** | 35.0x | ~35x slower |

### Large File (lorem-xlarge.md, ~113KB)

| Implementation | Time | Relative | Notes |
|----------------|------|----------|-------|
| **cmark (C)** | **3.6 ms** | 1.00x | Native performance |
| **clmd (Rust)** | **4.6 ms** | 1.30x | +30% vs cmark |
| **commonmark.js (JS)** | **87.1 ms** | 24.2x | ~24x slower |

## Key Observations

### Rust vs C

- clmd is within 30% of cmark performance
- Both use native code with no GC overhead
- clmd's Arena-based memory management is efficient
- Room for optimization: SIMD, better cache utilization

### Rust vs JavaScript

- clmd is 24-35x faster than commonmark.js
- Difference includes Node.js startup overhead
- Even with startup overhead, Rust implementation is significantly faster
- JavaScript's GC and dynamic typing contribute to overhead

## Why C is Faster

1. **Mature Optimization**: cmark has been optimized for years
2. **Direct Pointer Access**: No Rust's borrow checker overhead
3. **Custom Allocators**: Fine-tuned memory management
4. **SIMD Usage**: Vectorized operations where applicable

## Why JavaScript is Slower

1. **Startup Overhead**: Node.js initialization (~50-60ms)
2. **GC Pauses**: Garbage collection interruptions
3. **Dynamic Typing**: Runtime type checking overhead
4. **JIT Warmup**: Just-in-time compilation needs warmup

## Running the Comparison

```bash
# Build clmd benchmark
cargo build --release --example benchmark

# Run hyperfine comparison
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/benchmark benches/samples/lorem1.md' \
  'cmark benches/samples/lorem1.md' \
  'node examples/bench/bench_commonmark.js benches/samples/lorem1.md'
```

## Historical Trends

| Date | clmd vs cmark (small) | clmd vs cmark (large) |
|------|----------------------|----------------------|
| 2026-03-25 | +27% | +65% |
| 2026-03-26 | +18% | +39% |
| 2026-03-27 | +19% | +40% |
| 2026-03-28 | +14% | +36% |
| 2026-03-29 | +19% | +40% |
| 2026-04-02 | +10% | +30% |

clmd has improved significantly and is now competitive with cmark.

## Micro-Benchmark Results (2026-04-02)

Using Criterion.rs for precise measurements (without process startup):

| Test | clmd Time | Description |
|------|-----------|-------------|
| parse_small (lorem1) | **0.95 µs** | ~3.8KB document |
| parse_medium | **6.73 µs** | ~7.5KB document |
| parse_large | **75.48 µs** | ~113KB document |

### Throughput Analysis

| Document Size | Time | Throughput |
|---------------|------|------------|
| ~3.8KB | 0.95 µs | ~4,000 MB/s |
| ~7.5KB | 6.73 µs | ~1,100 MB/s |
| ~113KB | 75.48 µs | ~1,500 MB/s |

clmd maintains high throughput across different document sizes. Note: Micro-benchmarks measure only the parsing logic without I/O overhead, hence the much higher throughput compared to end-to-end measurements.
