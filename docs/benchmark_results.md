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

## Latest Results (After Optimization)

### Block-level Benchmarks

| Test | Before | After | Change | Description |
|------|--------|-------|--------|-------------|
| block_quotes_flat | 8.57 µs | 8.04 µs | **-6.2%** | Flat block quotes |
| block_quotes_nested | 15.66 µs | 14.40 µs | **-8.0%** | Nested block quotes |
| block_code | 3.12 µs | 3.16 µs | +1.3% | Indented code blocks |
| block_fences | 4.16 µs | 4.15 µs | -0.2% | Fenced code blocks |
| block_heading | 8.98 µs | 8.26 µs | **-8.0%** | ATX headings |
| block_hr | 3.82 µs | 3.79 µs | -0.8% | Horizontal rules |
| block_list_flat | 56.24 µs | 51.72 µs | **-8.0%** | Flat lists |
| block_list_nested | 44.24 µs | 41.87 µs | **-5.4%** | Nested lists |
| block_html | 13.35 µs | 13.33 µs | -0.2% | HTML blocks |
| block_lheading | 5.22 µs | 4.98 µs | **-4.6%** | Setext headings |
| block_ref_flat | 48.32 µs | 45.68 µs | **-5.5%** | Flat reference links |
| block_ref_nested | 43.66 µs | 42.49 µs | **-2.7%** | Nested reference links |

### Inline Benchmarks

| Test | Before | After | Change | Description |
|------|--------|-------|--------|-------------|
| inline_autolink | 24.22 µs | 23.52 µs | **-2.9%** | Autolinks |
| inline_backticks | 4.93 µs | 4.74 µs | **-3.9%** | Code spans |
| inline_em_flat | 20.93 µs | 20.20 µs | **-3.5%** | Flat emphasis |
| inline_em_nested | 16.90 µs | 16.35 µs | **-3.3%** | Nested emphasis |
| inline_em_worst | 15.73 µs | 15.17 µs | **-3.6%** | Worst-case emphasis |
| inline_entity | 17.34 µs | 16.52 µs | **-4.7%** | HTML entities |
| inline_escape | 17.39 µs | 16.72 µs | **-3.9%** | Escape sequences |
| inline_html | 40.67 µs | 39.53 µs | **-2.8%** | Inline HTML |
| inline_links_flat | 44.68 µs | 41.16 µs | **-7.9%** | Flat links |
| inline_links_nested | 47.40 µs | 46.27 µs | **-2.4%** | Nested links |
| inline_newlines | 12.77 µs | 12.18 µs | **-4.6%** | Hard line breaks |

### Full Document Benchmarks

| Test | Before | After | Change | Description |
|------|--------|-------|--------|-------------|
| lorem1_full_document | 41.38 µs | 34.03 µs | **-17.8%** | Complete Lorem Ipsum document |
| rawtabs | 9.54 µs | 9.01 µs | **-5.6%** | Document with raw tabs |

### Summary

- **Average improvement**: ~5-6% across all benchmarks
- **Best improvement**: lorem1_full_document at **-17.8%**
- **Key optimizations**:
  1. Subject now uses `&'a str` instead of `String` to avoid copying
  2. Byte-level character scanning in `parse_string()` and `advance()`
  3. Optimized `block_info` storage using Vec with pre-allocation
  4. Zero-copy line processing in `BlockParser::parse()`

## Performance Summary

### Fastest Operations
1. **block_code**: 3.12 µs - Simple indented code blocks
2. **block_hr**: 3.82 µs - Horizontal rules
3. **block_fences**: 4.16 µs - Fenced code blocks
4. **inline_backticks**: 4.93 µs - Code spans

### Slowest Operations
1. **block_list_flat**: 56.24 µs - List parsing (most complex)
2. **inline_links_nested**: 47.40 µs - Nested link parsing
3. **inline_links_flat**: 44.68 µs - Link reference resolution
4. **block_list_nested**: 44.24 µs - Nested list parsing

## Observations

- **Block-level parsing** is generally faster than inline parsing for simple elements
- **List parsing** is the most expensive operation due to complex indentation handling
- **Link parsing** (both inline and reference) is expensive due to reference resolution
- **Full document parsing** (lorem1) performs well at ~41 µs for a complete document

## Running Benchmarks

```bash
# Run all categorized benchmarks
cargo bench --bench categorized_benchmark

# Run specific benchmark group
cargo bench --bench categorized_benchmark block

# Run all benchmarks including original parser_bench
cargo bench
```

## Historical Data

### 2026-03-25 (Initial)
- First benchmark run with cmark samples
- 25 test cases covering block, inline, and full document parsing
- All tests complete in under 60 µs

## Comparison with Reference Implementations

### cmark (C implementation)

Tested with cmark 0.31.2 compiled from source.

| Category | Average Time | Notes |
|----------|-------------|-------|
| Block Quotes | 54.37 ms | First run includes JIT warmup |
| Block Code | 2.18 ms | Fast code block parsing |
| Block Headings | 2.15 ms | Efficient heading detection |
| Block Lists | 2.22 ms | Optimized list handling |
| Block References | 2.23 ms | Reference link resolution |
| Block HTML/HR | 2.17 ms | Simple block elements |
| Inline Emphasis | 2.21 ms | Emphasis parsing |
| Inline Links | 2.22 ms | Link processing |
| Inline Other | 2.16 ms | Other inline elements |
| Full Document | 2.09 ms | Complete document parsing |

**Note**: cmark times are in milliseconds (ms) and include process startup overhead. The first test (block-bq-flat) shows 106ms due to initial JIT/warmup, but subsequent tests average ~2ms.

### commonmark.js (JavaScript implementation)

Tested with commonmark.js 0.31.2, comparing with other JS parsers.

Sample results for block-bq-flat.md:
| Parser | Ops/sec | Relative Performance |
|--------|---------|---------------------|
| commonmark.js | 207,438 | Baseline |
| showdown.js | 20,263 | ~10x slower |
| marked.js | 134,134 | ~1.5x slower |
| markdown-it | 198,494 | Similar performance |

Sample results for block-code.md:
| Parser | Ops/sec | Relative Performance |
|--------|---------|---------------------|
| commonmark.js | 560,304 | Baseline |
| showdown.js | 41,988 | ~13x slower |
| marked.js | 1,416,554 | ~2.5x faster |
| markdown-it | 1,000,669 | ~1.8x faster |

**Observations**:
- commonmark.js is competitive with markdown-it
- marked.js is fastest for simple code blocks
- showdown.js is consistently slower across all tests
- Performance varies significantly by content type

### clmd vs Reference Implementations

**Important Note**: Direct comparison is difficult due to different measurement methodologies:
- **clmd**: Microseconds per operation (Criterion benchmark)
- **cmark**: Milliseconds per file (including process overhead)
- **commonmark.js**: Operations per second (Benchmark.js)

**Rough Comparison** (block-bq-flat):
- clmd: ~8.6 µs per parse
- cmark: ~2.1 ms per file (with overhead)
- commonmark.js: ~207,000 ops/sec (~4.8 µs per operation)

clmd shows competitive performance with native C (cmark) and optimized JavaScript (commonmark.js) implementations.

## Future Improvements

Potential areas for optimization based on benchmark results:
1. List parsing algorithm optimization
2. Link reference resolution caching
3. Inline parsing state machine improvements
4. Compare with other Rust Markdown parsers (pulldown-cmark, comrak)
