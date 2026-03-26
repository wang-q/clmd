# Benchmark Results

This document records the performance benchmark results for clmd.

## Test Environment

- **Date**: 2026-03-27
- **CPU**: Apple Silicon (M-series)
- **OS**: macOS
- **Rust Version**: Latest stable
- **Optimization**: Release mode (`--release`)

## Latest Results (2026-03-27)

### Categorized Benchmarks

#### Block-level Benchmarks

| Test | Time | Description |
|------|------|-------------|
| block_quotes_flat | **4.49 µs** | Flat block quotes |
| block_quotes_nested | **8.19 µs** | Nested block quotes |
| block_code | **1.64 µs** | Indented code blocks |
| block_fences | **3.31 µs** | Fenced code blocks |
| block_heading | **5.41 µs** | ATX headings |
| block_hr | **1.92 µs** | Horizontal rules |
| block_list_flat | **26.66 µs** | Flat lists |
| block_list_nested | **21.67 µs** | Nested lists |
| block_html | **7.79 µs** | HTML blocks |
| block_lheading | **3.14 µs** | Setext headings |
| block_ref_flat | **22.72 µs** | Flat reference links |
| block_ref_nested | **28.84 µs** | Nested reference links |

#### Inline Benchmarks

| Test | Time | Description |
|------|------|-------------|
| inline_autolink | **15.61 µs** | Autolinks |
| inline_backticks | **2.95 µs** | Code spans |
| inline_em_flat | **11.40 µs** | Flat emphasis |
| inline_em_nested | **9.40 µs** | Nested emphasis |
| inline_em_worst | **8.81 µs** | Worst-case emphasis |
| inline_entity | **7.40 µs** | HTML entities |
| inline_escape | **6.16 µs** | Escape sequences |
| inline_html | **13.36 µs** | Inline HTML |
| inline_links_flat | **20.20 µs** | Flat links |
| inline_links_nested | **19.88 µs** | Nested links |
| inline_newlines | **7.27 µs** | Hard line breaks |

#### Full Document Benchmarks

| Test | Time | Description |
|------|------|-------------|
| lorem1_full_document | **14.90 µs** | Complete Lorem Ipsum document (~1KB) |
| rawtabs | **5.15 µs** | Document with raw tabs |
| lorem_large_7kb | **114.90 µs** | Large document (~7KB) |
| lorem_xlarge_110kb | **1.64 ms** | Extra large document (~110KB) |

#### Synthetic Benchmarks

| Test | Time | Description |
|------|------|-------------|
| synthetic_small_document | **6.20 µs** | Small synthetic document |
| synthetic_medium_document | **27.92 µs** | Medium synthetic document |
| synthetic_large_document | **173.99 µs** | Large synthetic document |

### Feature Benchmarks

| Test | Time | Throughput | Description |
|------|------|------------|-------------|
| smart_punctuation | **2.30 µs** | 55.6 MiB/s | Smart quotes, dashes, ellipses |
| links_and_emphasis | **6.36 µs** | 18.6 MiB/s | Links and emphasis |
| code_blocks | **5.13 µs** | 17.7 MiB/s | Fenced code blocks |
| tables | **2.19 µs** | 60.5 MiB/s | GFM tables |
| autolinks | **1.95 µs** | 53.4 MiB/s | Automatic links |
| html_entities | **3.01 µs** | 35.8 MiB/s | HTML entity decoding |

### Pathological Benchmarks

Stress tests with extreme inputs:

| Test | Parameter | Time | Throughput |
|------|-----------|------|------------|
| nested_emphasis | 10 | **2.15 µs** | 13.9 MiB/s |
| nested_emphasis | 50 | **6.38 µs** | 23.5 MiB/s |
| nested_emphasis | 100 | **11.85 µs** | 25.3 MiB/s |
| nested_emphasis | 200 | **22.78 µs** | 26.3 MiB/s |
| many_link_defs | 100 | **33.42 µs** | 9.1 MiB/s |
| many_link_defs | 500 | **155.8 µs** | 9.8 MiB/s |
| many_link_defs | 1000 | **311.5 µs** | 9.8 MiB/s |
| deep_nested_lists | 10 | **3.45 µs** | 8.7 MiB/s |
| deep_nested_lists | 50 | **95.8 µs** | 12.5 MiB/s |
| deep_nested_lists | 100 | **650.5 µs** | 15.5 MiB/s |
| long_inline_code | 100 | **1.15 µs** | 84.9 MiB/s |
| long_inline_code | 500 | **2.05 µs** | 233.8 MiB/s |
| long_inline_code | 1000 | **3.14 µs** | 304.7 MiB/s |
| long_inline_code | 5000 | **11.27 µs** | 423.3 MiB/s |
| many_backticks | 10 | **866 ns** | 110.1 MiB/s |
| many_backticks | 50 | **2.07 µs** | 230.4 MiB/s |
| many_backticks | 100 | **3.62 µs** | 263.8 MiB/s |
| wide_tables | 10 | **1.80 µs** | 66.9 MiB/s |
| wide_tables | 50 | **2.89 µs** | 226.4 MiB/s |
| wide_tables | 100 | **4.05 µs** | 326.0 MiB/s |

### Real-World Benchmarks

Tests using actual Markdown documents:

| Document | Size | Time | Throughput |
|----------|------|------|------------|
| lorem1.md | ~1KB | **14.74 µs** | 245.2 MiB/s |
| lorem-large.md | ~7KB | **123.7 µs** | 58.1 MiB/s |
| lorem-xlarge.md | ~110KB | **1.60 ms** | 67.2 MiB/s |

### Performance Highlights

#### Fastest Operations
1. **block_code**: 1.64 µs - Simple indented code blocks
2. **block_hr**: 1.92 µs - Horizontal rules
3. **block_lheading**: 3.14 µs - Setext headings
4. **block_fences**: 3.31 µs - Fenced code blocks
5. **inline_backticks**: 2.95 µs - Code spans

#### Throughput Analysis

| Document Size | Time | Throughput |
|---------------|------|------------|
| ~1KB | 14.90 µs | ~67 MB/s |
| ~7KB | 114.90 µs | ~61 MB/s |
| ~110KB | 1.64 ms | ~67 MB/s |

**Conclusion**: clmd maintains stable throughput (~61-67 MB/s) across different document sizes.

## Cross-Language Comparison (2026-03-27)

### Small File (lorem1.md, ~1KB)

Using hyperfine for fair comparison (includes process startup and file IO):

| Implementation | Time | Relative Speed |
|----------------|------|----------------|
| **cmark (C)** | **1.6 ms** | 1.00x (fastest) |
| **pulldown-cmark (Rust)** | **1.7 ms** | 1.06x (6% slower) |
| **comrak (Rust)** | **1.8 ms** | 1.12x (12% slower) |
| **clmd (Rust)** | **1.9 ms** | 1.19x (19% slower) |
| **commonmark.js (JS)** | **64.7 ms** | 39.9x (40x slower) |

### Large File (lorem-xlarge.md, ~110KB)

| Implementation | Time | Relative Speed |
|----------------|------|----------------|
| **cmark (C)** | **2.2 ms** | 1.00x (fastest) |
| **pulldown-cmark (Rust)** | **2.5 ms** | 1.14x (14% slower) |
| **comrak (Rust)** | **2.8 ms** | 1.27x (27% slower) |
| **clmd (Rust)** | **3.1 ms** | 1.40x (40% slower) |
| **commonmark.js (JS)** | **75.2 ms** | 34.3x (34x slower) |

### Key Observations

1. **Small files**: clmd is very close to cmark (only 19% slower)
2. **Large files**: Performance gap is 40% vs cmark
3. **Rust implementations**: All three Rust parsers (clmd, comrak, pulldown-cmark) are competitive with cmark
4. **commonmark.js**: Consistently much slower (34-40x), mainly due to Node.js startup time
5. **Stable performance**: clmd maintains consistent throughput across document sizes

## Arena-Based Implementation

clmd uses an Arena-based memory management system for optimal performance.

### Architecture

- **NodeArena**: A bump allocator for all AST nodes
- **NodeId (u32)**: Index-based node references instead of `Rc<RefCell>`
- **Contiguous memory**: Better cache locality and fewer allocations

### Benefits

1. **Memory allocation**: Single arena allocation instead of individual Rc allocs
2. **Cache locality**: Contiguous memory layout for better CPU cache utilization
3. **No reference counting**: Eliminated Rc overhead (increment/decrement)
4. **No borrow checking**: Direct index-based access instead of RefCell runtime checks
5. **Tree operations**: O(1) node insertion/removal via index manipulation

## Historical Performance Data

### 2026-03-27 (Latest Results)
- Current performance baseline
- Small file: 14.90 µs for ~1KB document
- Large file: 1.64 ms for ~110KB document
- Cross-language comparison shows clmd is competitive with cmark

### 2026-03-26 (Previous Optimization Round)
- Small file gap vs cmark: 18% slower
- Large file gap vs cmark: 39% slower
- Average inline parsing improvement: ~8.5%

### 2026-03-25 (Arena Migration Complete)
- Arena-based implementation becomes default
- 40% average performance improvement over Rc<RefCell>
- All 365+ tests passing

## Running Benchmarks

```bash
# Run all categorized benchmarks
cargo bench --bench categorized_benchmark

# Run specific benchmark group
cargo bench --bench categorized_benchmark block

# Run all benchmarks including original parser_bench
cargo bench

# Cross-language comparison (requires cmark and Node.js)
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_language_bench benches/samples/lorem1.md' \
  'cmark benches/samples/lorem1.md' \
  'node bench_commonmark.js benches/samples/lorem1.md'
```

## Future Improvements

Potential areas for further optimization:
1. SIMD acceleration for string operations (reference: pulldown-cmark's memchr usage)
2. Parallel parsing for large documents
3. Memory pool for temporary allocations
4. Two-pass parsing strategy (reference: pulldown-cmark)
5. Small string optimization (reference: pulldown-cmark's CowStr)

## Comparison with Other Rust Parsers

See [comparison documentation](../comparison/) for detailed analysis:
- [comrak](../comparison/comrak.md): Feature-rich with GFM support
- [pulldown-cmark](../comparison/pulldown-cmark.md): Event-driven, low memory
- [Cross-Language](../comparison/cross-language.md): cmark (C), commonmark.js (JS)
