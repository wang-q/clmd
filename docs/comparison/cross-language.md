# Cross-Language Comparison

Comparison of clmd with Markdown parsers in other languages.

## Tested Implementations

| Implementation | Language | Version | Architecture |
|----------------|----------|---------|--------------|
| cmark | C | 0.31.2 | AST |
| clmd | Rust | 0.3.0+ | AST + Arena |
| commonmark.js | JavaScript | 0.31.2 | AST |

## Performance Results (2026-03-29)

### Small File (lorem1.md, ~1KB)

| Implementation | Time | Relative | Notes |
|----------------|------|----------|-------|
| **cmark (C)** | **1.6 ms** | 1.00x | Native performance |
| **clmd (Rust)** | **1.9 ms** | 1.19x | +19% vs cmark |
| **commonmark.js (JS)** | **64.7 ms** | 39.9x | ~40x slower |

### Large File (lorem-xlarge.md, ~110KB)

| Implementation | Time | Relative | Notes |
|----------------|------|----------|-------|
| **cmark (C)** | **2.2 ms** | 1.00x | Native performance |
| **clmd (Rust)** | **3.1 ms** | 1.40x | +40% vs cmark |
| **commonmark.js (JS)** | **75.2 ms** | 34.3x | ~34x slower |

## Key Observations

### Rust vs C

- clmd is within 40% of cmark performance
- Both use native code with no GC overhead
- clmd's Arena-based memory management is efficient
- Room for optimization: SIMD, better cache utilization

### Rust vs JavaScript

- clmd is 34-40x faster than commonmark.js
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
cargo build --release --example cross_language_bench

# Run hyperfine comparison
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_language_bench benches/samples/lorem1.md' \
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

clmd has improved significantly and is now competitive with cmark.

## Micro-Benchmark Results (2026-03-29)

Using Criterion.rs for precise measurements (without process startup):

| Test | clmd Time | Description |
|------|-----------|-------------|
| lorem1_full_document | **18.31 µs** | ~1KB document |
| lorem_large_7kb | **129.04 µs** | ~7KB document |
| lorem_xlarge_110kb | **1.85 ms** | ~110KB document |

### Throughput Analysis

| Document Size | Time | Throughput |
|---------------|------|------------|
| ~1KB | 18.31 µs | ~54 MB/s |
| ~7KB | 129.04 µs | ~54 MB/s |
| ~110KB | 1.85 ms | ~59 MB/s |

clmd maintains stable throughput (~54-59 MB/s) across different document sizes.
