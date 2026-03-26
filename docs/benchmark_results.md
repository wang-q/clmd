# Benchmark Results

This document records the performance benchmark results for clmd.

## Test Environment

- **Date**: 2026-03-27
- **CPU**: Apple Silicon (M-series)
- **OS**: macOS
- **Rust Version**: Latest stable
- **Optimization**: Release mode (`--release`)

## Benchmark Methodology

Benchmarks are based on cmark's sample files, organized into three categories:
- **Block-level**: Parsing of block elements (quotes, lists, code blocks, etc.)
- **Inline**: Parsing of inline elements (emphasis, links, entities, etc.)
- **Full Document**: Complete document parsing

## Latest Results (2026-03-27)

### Block-level Benchmarks

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

### Inline Benchmarks

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

### Full Document Benchmarks

| Test | Time | Description |
|------|------|-------------|
| lorem1_full_document | **14.90 µs** | Complete Lorem Ipsum document (~1KB) |
| rawtabs | **5.15 µs** | Document with raw tabs |
| lorem_large_7kb | **114.90 µs** | Large document (~7KB) |
| lorem_xlarge_110kb | **1.64 ms** | Extra large document (~110KB) |
| fair_comparison_doc | **134.07 µs** | Fair comparison document |

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
| **clmd (Rust)** | **1.9 ms** | 1.19x (19% slower) |
| **commonmark.js (JS)** | **64.7 ms** | 39.9x (40x slower) |

### Large File (lorem-xlarge.md, ~110KB)

| Implementation | Time | Relative Speed |
|----------------|------|----------------|
| **cmark (C)** | **2.2 ms** | 1.00x (fastest) |
| **clmd (Rust)** | **3.1 ms** | 1.40x (40% slower) |
| **commonmark.js (JS)** | **75.2 ms** | 34.3x (34x slower) |

### Key Observations

1. **Small files**: clmd is very close to cmark (only 19% slower)
2. **Large files**: Performance gap is 40% vs cmark
3. **commonmark.js**: Consistently much slower (34-40x), mainly due to Node.js startup time
4. **Stable performance**: clmd maintains consistent throughput across document sizes

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
1. SIMD acceleration for string operations
2. Parallel parsing for large documents
3. Memory pool for temporary allocations
4. Compare with other Rust Markdown parsers (pulldown-cmark, comrak)
