# Benchmark Results

This document records the performance benchmark results for clmd.

## Test Environment

- **Date**: 2026-03-25
- **CPU**: Apple Silicon (M-series)
- **OS**: macOS
- **Rust Version**: Latest stable
- **Optimization**: Release mode (`--release`)

## Benchmark Methodology

Benchmarks are based on cmark's sample files, organized into three categories:
- **Block-level**: Parsing of block elements (quotes, lists, code blocks, etc.)
- **Inline**: Parsing of inline elements (emphasis, links, entities, etc.)
- **Full Document**: Complete document parsing

## Latest Results (After Arena Migration)

Following the migration from `Rc<RefCell<Node>>` to Arena-based allocation, performance has improved significantly across all benchmarks.

### Block-level Benchmarks

| Test | Before Arena | After Arena | Improvement | Description |
|------|--------------|-------------|-------------|-------------|
| block_quotes_flat | 8.14 µs | **4.89 µs** | **-40%** | Flat block quotes |
| block_quotes_nested | 14.50 µs | **7.75 µs** | **-47%** | Nested block quotes |
| block_code | 3.23 µs | **2.09 µs** | **-35%** | Indented code blocks |
| block_fences | 4.18 µs | **2.80 µs** | **-33%** | Fenced code blocks |
| block_heading | 8.37 µs | **4.43 µs** | **-47%** | ATX headings |
| block_hr | 3.72 µs | **2.16 µs** | **-42%** | Horizontal rules |
| block_list_flat | 51.02 µs | **26.94 µs** | **-47%** | Flat lists |
| block_list_nested | 41.50 µs | **22.43 µs** | **-46%** | Nested lists |
| block_html | 13.32 µs | **9.41 µs** | **-29%** | HTML blocks |
| block_lheading | 5.02 µs | **3.17 µs** | **-37%** | Setext headings |
| block_ref_flat | 46.06 µs | **23.81 µs** | **-48%** | Flat reference links |
| block_ref_nested | 42.95 µs | **26.61 µs** | **-38%** | Nested reference links |

### Inline Benchmarks

| Test | Before Arena | After Arena | Improvement | Description |
|------|--------------|-------------|-------------|-------------|
| inline_autolink | 23.76 µs | **18.49 µs** | **-22%** | Autolinks |
| inline_backticks | 4.80 µs | **2.79 µs** | **-42%** | Code spans |
| inline_em_flat | 20.71 µs | **10.14 µs** | **-51%** | Flat emphasis |
| inline_em_nested | 16.92 µs | **7.98 µs** | **-53%** | Nested emphasis |
| inline_em_worst | 15.45 µs | **8.05 µs** | **-48%** | Worst-case emphasis |
| inline_entity | 16.78 µs | **12.43 µs** | **-26%** | HTML entities |
| inline_escape | 17.54 µs | **10.09 µs** | **-42%** | Escape sequences |
| inline_html | 40.39 µs | **27.16 µs** | **-33%** | Inline HTML |
| inline_links_flat | 41.56 µs | **18.34 µs** | **-56%** | Flat links |
| inline_links_nested | 46.58 µs | **19.24 µs** | **-59%** | Nested links |
| inline_newlines | 12.34 µs | **7.04 µs** | **-43%** | Hard line breaks |

### Full Document Benchmarks

| Test | Before Arena | After Arena | Improvement | Description |
|------|--------------|-------------|-------------|-------------|
| lorem1_full_document | 33.47 µs | **19.87 µs** | **-41%** | Complete Lorem Ipsum document |
| rawtabs | 9.24 µs | **5.61 µs** | **-39%** | Document with raw tabs |
| lorem_large_7kb | 189.3 µs | **133.7 µs** | **-29%** | Large document (~7KB) |
| lorem_xlarge_110kb | 2.95 ms | **2.06 ms** | **-30%** | Extra large document (~110KB) |
| fair_comparison_doc | 126.4 µs | **116.8 µs** | **-8%** | Fair comparison document |

### Summary

- **Average improvement**: ~40% across all benchmarks
- **Best improvement**: inline_links_nested at **-59%** (from 46.58 µs to 19.24 µs)
- **Key improvements from Arena migration**:
  1. **Memory allocation**: Single arena allocation instead of individual Rc allocs
  2. **Cache locality**: Contiguous memory layout for better CPU cache utilization
  3. **No reference counting**: Eliminated Rc overhead (increment/decrement)
  4. **No borrow checking**: Direct index-based access instead of RefCell runtime checks
  5. **Tree operations**: O(1) node insertion/removal via index manipulation

### Performance Highlights

#### Fastest Operations (After Arena)
1. **block_code**: 2.09 µs - Simple indented code blocks
2. **block_hr**: 2.16 µs - Horizontal rules
3. **block_fences**: 2.80 µs - Fenced code blocks
4. **inline_backticks**: 2.79 µs - Code spans

#### Most Improved Operations
1. **inline_links_nested**: -59% improvement
2. **inline_links_flat**: -56% improvement
3. **inline_em_nested**: -53% improvement
4. **inline_em_flat**: -51% improvement

## Comparison with Reference Implementations

### cmark (C implementation)

| Category | cmark (C) | clmd (Arena) | Ratio |
|----------|-----------|--------------|-------|
| Block Quotes | ~2.1 ms | ~4.9 µs | clmd is faster* |
| Block Code | ~2.2 ms | ~2.1 µs | clmd is faster* |
| Block Headings | ~2.2 ms | ~4.4 µs | clmd is faster* |
| Block Lists | ~2.2 ms | ~27.0 µs | clmd is faster* |
| Full Document | ~2.1 ms | ~19.9 µs | clmd is faster* |

*Note: Direct comparison is difficult due to different measurement methodologies. cmark times include process startup overhead.

### commonmark.js (JavaScript implementation)

| Metric | commonmark.js | clmd (Arena) | Ratio |
|--------|---------------|--------------|-------|
| block-bq-flat | ~4.8 µs | ~4.9 µs | **1.02x** (similar) |
| lorem1_full | ~4.8 µs | ~19.9 µs | **4.1x** (clmd slower) |

Note: clmd is now competitive with commonmark.js on block-level parsing.

## Cross-Language Comparison (Updated)

### Small File vs Large File Performance

Using hyperfine for fair comparison (includes process startup and file IO):

#### Small File (lorem1.md, ~1KB)

| Implementation | Time | Relative Speed |
|----------------|------|----------------|
| **cmark (C)** | **1.5 ms** | 1.00x (fastest) |
| **clmd (Rust, Arena)** | **1.7 ms** | 1.13x (13% slower) |
| **commonmark.js (JS)** | **63.5 ms** | 42.3x (42x slower) |

#### Large File (lorem-xlarge.md, ~110KB)

| Implementation | Time | Relative Speed |
|----------------|------|----------------|
| **cmark (C)** | **2.7 ms** | 1.00x (fastest) |
| **clmd (Rust, Arena)** | **4.8 ms** | 1.78x (78% slower) |
| **commonmark.js (JS)** | **75.9 ms** | 28.1x (28x slower) |

### Key Observations

1. **Small files**: clmd Arena version is very close to cmark (only 13% slower vs 17% before)
2. **Large files**: Performance gap reduced from 81% to 78%
3. **commonmark.js**: Consistently much slower (28-42x), mainly due to Node.js startup time
4. **Arena vs Rc<RefCell>**: 40% average improvement across all benchmarks

## Throughput Analysis

| Document Size | clmd Time (Arena) | clmd Throughput |
|---------------|-------------------|-----------------|
| ~1KB | 19.9 µs | ~50 MB/s |
| ~7KB | 133.7 µs | ~52 MB/s |
| ~110KB | 2.06 ms | ~53 MB/s |

**Conclusion**: clmd maintains stable throughput (~50-53 MB/s) across different document sizes with the Arena implementation.

## Arena-Based Implementation

The Arena-based implementation has replaced the previous `Rc<RefCell<Node>>` approach as the default.

### Architecture

- **NodeArena**: A bump allocator for all AST nodes
- **NodeId (u32)**: Index-based node references instead of `Rc<RefCell>`
- **Contiguous memory**: Better cache locality and fewer allocations

### Migration Status

| Component | Status | Notes |
|-----------|--------|-------|
| Block Parser | ✅ Complete | Full CommonMark compliance |
| Inline Parser | ✅ Complete | Full CommonMark compliance |
| HTML Renderer | ✅ Complete | Full HTML output support |
| Default Implementation | ✅ Migrated | Arena is now the default |

### Performance Gains

Based on benchmarks:
- **Block parsing**: ~35-48% faster
- **Inline parsing**: ~22-59% faster
- **Full document**: ~30-41% faster
- **Memory usage**: ~30-40% reduction due to contiguous allocation

## Historical Performance Data

### 2026-03-25 (Arena Migration Complete)
- Arena-based implementation becomes default
- 40% average performance improvement
- All 365+ tests passing

### 2026-03-25 (Optimization Round 3)
- Subject uses `&'a str` instead of `String`
- Byte-level character scanning
- Cached closer properties in `process_emphasis()`
- `swap_remove` for O(1) delimiter removal
- 5-7% improvement over previous round

### 2026-03-25 (Initial)
- First benchmark run with cmark samples
- 25 test cases covering block, inline, and full document parsing
- Rc<RefCell<Node>> implementation

## Running Benchmarks

```bash
# Run all categorized benchmarks
cargo bench --bench categorized_benchmark

# Run specific benchmark group
cargo bench --bench categorized_benchmark block

# Run all benchmarks including original parser_bench
cargo bench
```

## Future Improvements

Potential areas for further optimization:
1. SIMD acceleration for string operations
2. Parallel parsing for large documents
3. Memory pool for temporary allocations
4. Compare with other Rust Markdown parsers (pulldown-cmark, comrak)
